use std::borrow::Borrow;
use std::iter::Peekable;
use std::str::Chars;

use crate::html::tokens::{TagData, Token};

pub struct Lexer<'a> {
    pos: usize,
    reconsume: Option<char>,
    input: Chars<'a>,
    state: Box<dyn LexerState>,
    buffer: &'a mut Vec<char>,
    error_handler: &'a dyn Fn(&str),
}

impl<'a> Lexer<'a> {
    pub fn new(input: String, error_handler: &'a dyn Fn(&str)) -> Lexer<'a> {
        Lexer {
            pos: 0,
            reconsume: Option::None,
            input: input.chars(),
            state: Box::new(DataState {}),
            buffer: &mut vec![],
            error_handler,
        }
    }

    pub fn next_character(mut self) -> Option<char> {
        match self.reconsume {
            Some(char) => {
                self.reconsume = Option::None;
                Option::Some(char)
            }
            None => {
                self.input.next()
            }
        }
    }

    pub fn next_token(mut self) -> Token {
        self.state.next_token(self)
    }
}

trait LexerState {
    fn next_token(self, lexer: Lexer) -> Token;
}

pub struct DataState {}

impl LexerState for DataState {
    fn next_token(self, mut lexer: Lexer) -> Token {
        match lexer.input.next() {
            Some(char) => {
                match char {
                    '&' => {
                        lexer.state = Box::new(CharacterReferenceState::new(
                            lexer.state, &mut lexer));
                        lexer.state.next_token(lexer)
                    }
                    '<' => {
                        lexer.state = Box::new(TagOpenState { ending_next: false });
                        lexer.state.next_token(lexer)
                    }
                    '\u{0000}' => {
                        (lexer.error_handler)("unexpected-null-character");
                        Token::Character('\u{0000}')
                    }
                    _ => {
                        Token::Character(char)
                    }
                }
            }
            _ => Token::EndOfFile()
        }
    }
}

pub struct CharacterReferenceState {
    return_state: Box<dyn LexerState>,
}

impl CharacterReferenceState {
    fn new(return_state: Box<dyn LexerState>, lexer: &mut Lexer) -> CharacterReferenceState {
        lexer.buffer = &mut vec![];
        lexer.buffer.push('&');
        CharacterReferenceState { return_state }
    }

    fn end_state(self, mut lexer: Lexer) -> Token {
        match lexer.buffer.first() {
            Some(char) => {
                return Token::Character(*char);
            }
            None => {
                lexer.state = self.return_state;
                lexer.next_token()
            }
        }
    }
}

impl LexerState for CharacterReferenceState {
    fn next_token(self, mut lexer: Lexer) -> Token {
        match lexer.input.next() {
            Some(char) => {
                match char {
                    'a'..='Z' => {
                        lexer.reconsume = Option::Some(char);
                        lexer.state = Box::new(NamedCharacterReferenceState {});
                        lexer.next_token()
                    }
                    '#' => {
                        lexer.buffer.push('#');
                        lexer.state = Box::new(NumericCharacterReferenceState {});
                        lexer.next_token()
                    }
                    _ => {
                        self.end_state(lexer)
                    }
                }
            }
            None => {
                self.end_state(lexer)
            }
        }
    }
}

pub struct TagOpenState {
    ending_next: bool,
}

impl LexerState for TagOpenState {
    fn next_token(self, mut lexer: Lexer) -> Token {
        match lexer.input.next() {
            Some(char) => {
                match char {
                    'a'..='Z' => {
                        lexer.reconsume = Option::Some(char);
                        lexer.state = Box::new(TagNameState { tag: TagData::new() });
                        Token::StartTag()
                    }
                    '!' => {
                        lexer.state = Box::new(MarkupDeclarationOpenState {});
                        lexer.next_token()
                    }
                    '/' => {
                        lexer.state = Box::new(EndTagOpenState { end_count: 0 });
                        lexer.next_token()
                    }
                    '?' => {
                        (lexer.error_handler)("unexpected-question-mark-instead-of-tag-name");
                        lexer.reconsume = Option::Some('?');
                        lexer.state = Box::new(BogusCommentState {});
                        Token::Comment(vec![])
                    }
                    _ => {
                        (lexer.error_handler)("invalid-first-character-of-tag-name");
                        lexer.state = Box::new(DataState {});
                        lexer.reconsume = Option::Some(char);
                        Token::Character('>')
                    }
                }
            }
            None => {
                if !self.ending_next {
                    (lexer.error_handler)("eof-before-tag-name");
                    return Token::Character('>');
                }
                Token::EndOfFile()
            }
        }
    }
}

pub struct EndTagOpenState {
    end_count: i8,
}

impl LexerState for EndTagOpenState {
    fn next_token(mut self, mut lexer: Lexer) -> Token {
        match lexer.next_character() {
            Some(char) => {
                match char {
                    'a'..='Z' => {
                        lexer.state = Box::new(TagNameState { tag: TagData::new() });
                        lexer.reconsume = Option::Some(char);
                        Token::EndTag()
                    }
                    '>' => {
                        (lexer.error_handler)("missing-end-tag-name");
                        lexer.state = Box::new(TagNameState { tag: TagData::new() });
                        lexer.next_token()
                    }
                    _ => {
                        (lexer.error_handler)("invalid-first-character-of-tag-name");
                        lexer.state = Box::new(BogusCommentState {});
                        Token::StartComment()
                    }
                }
            }
            None => {
                self.end_count += 1;
                match self.end_count {
                    1 => {
                        Token::Character('<')
                    }
                    2 => {
                        Token::Character('\u{002F}')
                    }
                    _ => {
                        Token::EndOfFile()
                    }
                }
            }
        }
    }
}

pub struct TagNameState {
    tag: TagData,
}

impl LexerState for TagNameState {
    fn next_token(mut self, mut lexer: Lexer) -> Token {
        loop {
            match lexer.next_character() {
                Some(char) => {
                    match char {
                        'A'..='Z' => {
                            self.tag.tag_name.push(char.to_ascii_lowercase());
                        }
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | ' ' => {
                            lexer.state = Box::new(BeforeAttributeNameState { tag: self.tag });
                            return lexer.next_token();
                        }
                        '/' => {
                            lexer.state = Box::new(SelfClosingStartTagState { tag: self.tag });
                            return lexer.next_token();
                        }
                        '>' => {
                            lexer.state = Box::new(DataState {});
                            return Token::Tag(self.tag);
                        }
                        '\u{0000}' => {
                            (lexer.error_handler)("unexpected-null-character");
                            self.tag.tag_name.push('\u{FFFD}');
                        }
                        _ => {
                            self.tag.tag_name.push(char);
                        }
                    }
                }
                None => {
                    (lexer.error_handler)("eof-in-tag");
                    return Token::EndOfFile();
                }
            }
        }
    }
}

pub struct BeforeAttributeNameState {
    tag: TagData
}

impl LexerState for BeforeAttributeNameState {
    fn next_token(self, mut lexer: Lexer) -> Token {
        loop {
            match lexer.next_character() {
                Some(char) => {
                    match char {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | ' ' => {
                            //Ignored
                        }
                        '/' | '>' => {
                            lexer.reconsume = Option::Some(char);
                            lexer.state = Box::new(AfterAttributeNameState { tag: self.tag, attribute: (vec![], vec![]) });
                            return lexer.next_token();
                        }
                        '=' => {
                            (lexer.error_handler)("unexpected-equals-sign-before-attribute-name");
                            lexer.state = Box::new(AttributeNameState { tag: self.tag, attribute: (vec!['='], vec![]) });
                            return Token::StartAttribute();
                        }
                        _ => {
                            lexer.reconsume = Option::Some(char);
                            lexer.state = Box::new(AttributeNameState { tag: self.tag, attribute: (vec![], vec![]) });
                            return lexer.next_token();
                        }
                    }
                }
                None => {
                    lexer.state = Box::new(AfterAttributeNameState { tag: self.tag, attribute: (vec![], vec![]) });
                    return lexer.next_token();
                }
            }
        }
    }
}

pub struct AttributeNameState {
    tag: TagData,
    attribute: (Vec<char>, Vec<char>),
}

impl LexerState for AttributeNameState {
    fn next_token(mut self, mut lexer: Lexer) -> Token {
        loop {
            match lexer.next_character() {
                Some(char) => {
                    match char {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | ' ' | '/' | '>' => {
                            lexer.reconsume = Option::Some(char);
                            lexer.state = Box::new(AfterAttributeNameState { tag: self.tag, attribute: self.attribute });
                            return lexer.next_token();
                        }
                        '=' => {
                            lexer.state = Box::new(BeforeAttributeValueState { tag: self.tag, attribute: self.attribute });
                            return lexer.next_token();
                        }
                        'A'..='Z' => {
                            self.attribute.0.push(char.to_ascii_lowercase());
                        }
                        '\u{0000}' => {
                            (lexer.error_handler)("unexpected-null-character");
                            self.attribute.0.push('\u{FFFD}');
                        }
                        '\"' | '\'' | '<' => {
                            (lexer.error_handler)("unexpected-character-in-attribute-name");
                            self.attribute.0.push(char);
                        }
                        _ => {
                            self.attribute.0.push(char);
                        }
                    }
                }
                None => {
                    lexer.state = Box::new(AfterAttributeNameState { tag: self.tag, attribute: self.attribute });
                    return lexer.next_token();
                }
            }
        }
    }
}

pub struct AfterAttributeNameState {
    tag: TagData,
    attribute: (Vec<char>, Vec<char>),
}

impl LexerState for AfterAttributeNameState {
    fn next_token(mut self, mut lexer: Lexer) -> Token {
        loop {
            match lexer.next_character() {
                Some(char) => {
                    match char {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' => {
                            //Ignored
                        }
                        '/' => {
                            self.tag.attributes.insert(self.attribute.0, self.attribute.1);
                            lexer.state = Box::new(SelfClosingStartTagState { tag: self.tag });
                            return lexer.next_token();
                        }
                        '=' => {
                            lexer.state = Box::new(BeforeAttributeValueState { tag: self.tag, attribute: self.attribute });
                            return lexer.next_token();
                        }
                        '>' => {
                            self.tag.attributes.insert(self.attribute.0, self.attribute.1);
                            lexer.state = Box::new(DataState {});
                            return Token::Tag(self.tag);
                        }
                        _ => {
                            self.tag.attributes.insert(self.attribute.0, self.attribute.1);
                            lexer.state = Box::new(AttributeNameState { tag: self.tag, attribute: (vec![], vec![]) });
                            return lexer.next_token();
                        }
                    }
                }
                None => {
                    (lexer.error_handler)("eof-in-tag");
                    return Token::Tag(self.tag);
                }
            }
        }
    }
}

pub struct SelfClosingStartTagState {
    tag: TagData
}

impl LexerState for SelfClosingStartTagState {
    fn next_token(mut self, mut lexer: Lexer) -> Token {
        match lexer.next_character() {
            Some(char) => {
                match char {
                    '>' => {
                        self.tag.self_closing = true;
                        lexer.state = Box::new(DataState {});
                        return Token::Tag(self.tag);
                    }
                    _ => {
                        (lexer.error_handler)("unexpected-solidus-in-tag");
                        lexer.reconsume = Option::Some(char);
                        lexer.state = Box::new(BeforeAttributeNameState { tag: self.tag });
                        return lexer.next_token();
                    }
                }
            }
            None => {
                (lexer.error_handler)("eof-in-tag");
                Token::EndOfFile()
            }
        }
    }
}

pub struct BeforeAttributeValueState {
    tag: TagData,
    attribute: (Vec<char>, Vec<char>)
}

impl LexerState for BeforeAttributeValueState {
    fn next_token(mut self, mut lexer: Lexer) -> Token {
        loop {
            match lexer.next_character() {
                Some(char) => {
                    match char {
                        '\u{0009}'|'\u{000A}'|'\u{000C}'|' ' => {
                            //Ignored
                        }
                        '\"' => {
                            lexer.state = Box::new(AttributeValueDoubleQuotedState { tag: self.tag, attribute: self.attribute });
                            return lexer.next_token();
                        }
                        '\'' => {
                            lexer.state = Box::new(AttributeValueSingleQuotedState { tag: self.tag, attribute: self.attribute });
                            return lexer.next_token();
                        }
                        '>' => {
                            (lexer.error_handler)("missing-attribute-value");
                            lexer.state = Box::new(DataState {});
                            self.tag.attributes.insert(self.attribute.0, self.attribute.1);
                            return Token::Tag(self.tag);
                        }
                        _ => {
                            lexer.reconsume = Option::Some(char);
                            lexer.state = Box::new(AttributeValueUnquotedState { tag: self.tag, attribute: self.attribute });
                            return lexer.next_token();
                        }
                    }
                }
                None => {
                    lexer.state = Box::new(AttributeValueUnquotedState { tag: self.tag, attribute: self.attribute });
                    return lexer.next_token();
                }
            }
        }
    }
}

pub struct AttributeValueDoubleQuotedState {
    tag: TagData,
    attribute: (Vec<char>, Vec<char>)
}

impl LexerState for AttributeValueDoubleQuotedState {
    fn next_token(mut self, mut lexer: Lexer) -> Token {
        loop {
            match lexer.next_character() {
                Some(char) => {
                    match char {
                        '\"' => {
                            lexer.state = Box::new(AfterAttributeValueQuotedState { tag: self.tag, attribute: self.attribute});
                            return lexer.next_token()
                        }
                        '&' => {
                            lexer.state = Box::new(CharacterReferenceState { return_state: lexer.state });
                            return lexer.next_token()
                        }
                        '\u{0000}' => {
                            (lexer.error_handler)("unexpected-null-character");
                            self.attribute.1.push('\u{FFFD}');
                        }
                        _ => {
                            self.attribute.1.push(char);
                        }
                    }
                }
                None => {
                    (lexer.error_handler)("eof-in-tag");
                    return Token::EndOfFile()
                }
            }
        }
    }
}

pub struct AttributeValueSingleQuotedState {
    tag: TagData,
    attribute: (Vec<char>, Vec<char>)
}

impl LexerState for AttributeValueSingleQuotedState {
    fn next_token(mut self, mut lexer: Lexer) -> Token {
        loop {
            match lexer.next_character() {
                Some(char) => {
                    match char {
                        '\'' => {
                            lexer.state = Box::new(AfterAttributeValueQuotedState { tag: self.tag, attribute: self.attribute});
                            return lexer.next_token()
                        }
                        '&' => {
                            lexer.state = Box::new(CharacterReferenceState { return_state: lexer.state });
                            return lexer.next_token()
                        }
                        '\u{0000}' => {
                            (lexer.error_handler)("unexpected-null-character");
                            self.attribute.1.push('\u{FFFD}');
                        }
                        _ => {
                            self.attribute.1.push(char);
                        }
                    }
                }
                None => {
                    (lexer.error_handler)("eof-in-tag");
                    return Token::EndOfFile()
                }
            }
        }
    }
}

pub struct AttributeValueUnquotedState {
    tag: TagData,
    attribute: (Vec<char>, Vec<char>)
}

impl LexerState for AttributeValueUnquotedState {
    fn next_token(mut self, mut lexer: Lexer) -> Token {
        loop {
            match lexer.next_character() {
                Some(char) => {
                    match char {
                        '\u{0009}'|'\u{000A}'|'\u{000C}'|' ' => {
                            lexer.state = Box::new(BeforeAttributeNameState { tag: self.tag });
                            return lexer.next_token()
                        }
                        '&' => {
                            lexer.state = Box::new(CharacterReferenceState { return_state: lexer.state });
                            return lexer.next_token()
                        }
                        '>' => {
                            lexer.state = Box::new(DataState {});
                            self.tag.attributes.insert(self.attribute.0, self.attribute.1);
                            return Token::Tag(self.tag)
                        }
                        '\u{0000}' => {
                            (lexer.error_handler)("unexpected-null-character");
                            self.attribute.1.push('\u{FFFD}');
                        }
                        '\"'|'\''|'<'|'='|'`' => {
                            (lexer.error_handler)("unexpected-character-in-unquoted-attribute-value");
                            self.attribute.1.push(char);
                        }
                        _ => {
                            self.attribute.1.push(char);
                        }
                    }
                }
                None => {
                    (lexer.error_handler)("eof-in-tag");
                    return Token::EndOfFile()
                }
            }
        }
    }
}

pub struct NamedCharacterReferenceState {}

impl LexerState for NamedCharacterReferenceState {
    fn next_token(self, mut lexer: Lexer) -> Token {
        Token::EndOfFile()
    }
}

pub struct NumericCharacterReferenceState {}

impl LexerState for NumericCharacterReferenceState {
    fn next_token(self, mut lexer: Lexer) -> Token {
        Token::EndOfFile()
    }
}

pub struct MarkupDeclarationOpenState {}

impl LexerState for MarkupDeclarationOpenState {
    fn next_token(self, mut lexer: Lexer) -> Token {
        Token::EndOfFile()
    }
}

pub struct BogusCommentState {}

impl LexerState for BogusCommentState {
    fn next_token(self, mut lexer: Lexer) -> Token {
        Token::EndOfFile()
    }
}