use std::str::Chars;

use crate::html::tokens::{TagData, Token, TokenType};

pub struct Lexer<'a> {
    pos: usize,
    reconsume: Option<char>,
    input: Chars<'a>,
    state: LexerState,
    buffer: &'a mut Vec<char>,
    error_handler: &'a dyn Fn(&str),
    elements: Vec<Token>,
    return_state: Option<LexerState>,
}

enum LexerState {
    DataState,
    RCDATAState,
    RAWTEXTState,
    ScriptDataState,
    PLAINTEXTState,
    TagOpenState,
    EndTagOpenState,
    TagNameState,
    RCDATALessThanSignState,
    RCDATAEndTagOpenState,
    RCDATAEndTagNameState,
    ScriptDataLessThanSignState,
    ScriptDataEndTagOpenState,
    ScriptDataEndTagNameState,
    ScriptDataEscapeStartState,
    ScriptDataEscapeStartDashState,
    ScriptDataEscapedState,
    ScriptDataEscapedDashState,
    ScriptDataEscapedDashDashState,
    ScriptDataEscapedLessThanSignState,
    ScriptDataEscapedEndTagOpenState,
    ScriptDataEscapedEndTagNameState,
    ScriptDataDoubleEscapeStartState,
    ScriptDataDoubleEscapedState,
    ScriptDataDoubleEscapedDashState,
    ScriptDataDoubleEscapedDashDashState,
    ScriptDataDoubleEscapedLessThanSignState,
    ScriptDataDoubleEscapeEndState,
    BeforeAttributeNameState,
    AttributeNameState,
    AfterAttributeNameState,
    BeforeAttributeValueState,
    AttributeValueDoubleQuotedState,
    AttributeValueSingleQuotedState,
    AttributeValueUnquotedState,
    AfterAttributeValueQuotedState,
    SelfClosingStartTagState,
    BogusCommentState,
    MarkupDeclarationOpenState,
    CommentStartState,
    CommentStartDashState,
    CommentState,
    CommentLessThanSignState,
    CommentLessThanSignBangState,
    CommentLessThanSignBangDashState,
    CommentLessThanSignBangDashDashState,
    CommentEndDashState,
    CommentEndState,
    CommentEndBangState,
    DOCTYPEState,
    BeforeDOCTYPENameState,
    DOCTYPENameState,
    AfterDOCTYPENameState,
    AfterDOCTYPEPublicKeywordState,
    BeforeDOCTYPEPublicIdentifierState,
    DOCTYPEPublicIdentifierDoubleQuotedState,
    DOCTYPEPublicIdentifierSingleQuotedState,
    AfterDOCTYPEPublicIdentifierState,
    BetweenDOCTYPEPublicAndSystemIdentifiersState,
    AfterDOCTYPESystemKeywordState,
    BeforeDOCTYPESystemIdentifierState,
    DOCTYPESystemIdentifierDoubleQuotedState,
    DOCTYPESystemIdentifierSingleQuotedState,
    AfterDOCTYPESystemIdentifierState,
    BogusDOCTYPEState,
    CDATASectionState,
    CDATASectionBracketState,
    CDATASectionEndState,
    CharacterReferenceState,
    NamedCharacterReferenceState,
    AmbiguousAmpersandState,
    NumericCharacterReferenceState,
    HexadecimalCharacterReferenceStartState,
    DecimalCharacterReferenceStartState,
    HexadecimalCharacterReferenceState,
    DecimalCharacterReferenceState,
    NumericCharacterReferenceEndState,
}

impl<'a> Lexer<'a> {
    pub fn new(input: String, error_handler: &'a dyn Fn(&str)) -> Lexer<'a> {
        Lexer {
            pos: 0,
            reconsume: Option::None,
            input: input.chars(),
            state: LexerState::DataState,
            buffer: &mut vec![],
            error_handler,
            elements: vec![],
            return_state: Option::None,
        }
    }

    pub fn get_top_token(self) -> Token {
        return match self.elements.top {
            Some(element) => {
                element
            }
            None => {
                Token::EndOfFile()
            }
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
        loop {
            match &self.State {
                DataState => {
                    match self.input.next() {
                        Some(char) => {
                            match char {
                                '&' => {
                                    self.return_state = Option::Some(LexerState::DataState);
                                    self.buffer = &mut vec!['&'];
                                    self.state = LexerState::CharacterReferenceState;
                                }
                                '<' => {
                                    self.state = LexerState::TagOpenState;
                                }
                                '\u{0000}' => {
                                    (self.error_handler)("unexpected-null-character");
                                    return Token::Character('\u{0000}');
                                }
                                _ => {
                                    return Token::Character(char);
                                }
                            }
                        }
                        _ => {
                            return Token::EndOfFile();
                        }
                    }
                }
                RCDATAState => {
                    //TODO
                }
                RAWTEXTState => {
                    //TODO
                }
                ScriptDataState => {
                    //TODO
                }
                PLAINTEXTState => {
                    //TODO
                }
                TagOpenState => {
                    match self.input.next() {
                        Some(char) => {
                            match char {
                                'a'..='Z' => {
                                    self.reconsume = Option::Some(char);
                                    self.state = LexerState::TagNameState;
                                    return Token::StartTag();
                                }
                                '!' => {
                                    self.state = LexerState::MarkupDeclarationOpenState;
                                }
                                '/' => {
                                    self.state = LexerState::EndTagOpenState;
                                }
                                '?' => {
                                    (self.error_handler)("unexpected-question-mark-instead-of-tag-name");
                                    self.reconsume = Option::Some('?');
                                    self.state = LexerState::BogusCommentState;
                                    return Token::Comment(vec![]);
                                }
                                _ => {
                                    (self.error_handler)("invalid-first-character-of-tag-name");
                                    self.state = LexerState::DataState;
                                    self.reconsume = Option::Some(char);
                                    return Token::Character('>');
                                }
                            }
                        }
                        None => {
                            self.state = LexerState::DataState;
                            (self.error_handler)("eof-before-tag-name");
                            Token::Character('>')
                        }
                    }
                }
                EndTagOpenState => {
                    match self.next_character() {
                        Some(char) => {
                            match char {
                                'a'..='Z' => {
                                    self.state = LexerState::TagNameState;
                                    self.reconsume = Option::Some(char);
                                    return Token::EndTag();
                                }
                                '>' => {
                                    (self.error_handler)("missing-end-tag-name");
                                    self.state = LexerState::TagNameState;
                                }
                                _ => {
                                    (self.error_handler)("invalid-first-character-of-tag-name");
                                    self.state = LexerState::BogusCommentState;
                                    return Token::StartComment();
                                }
                            }
                        }
                        None => {
                            return if self.buffer.is_empty() {
                                (self.error_handler)("eof-before-tag-name");
                                self.buffer.push('<');
                                Token::Character('<')
                            } else {
                                self.state = LexerState::DataState;
                                Token::Character('\u{002F}')
                            }
                        }
                    }
                }
                TagNameState => {
                    match self.next_character() {
                        Some(char) => {
                            match char {
                                'A'..='Z' => {
                                    self.tag.tag_name.push(char.to_ascii_lowercase());
                                }
                                '\u{0009}' | '\u{000A}' | '\u{000C}' | ' ' => {
                                    self.state = LexerState::BeforeAttributeNameState;
                                    return self.next_token();
                                }
                                '/' => {
                                    self.state = LexerState::SelfClosingStartTagState;
                                    return self.next_token();
                                }
                                '>' => {
                                    self.state = LexerState::DataState;
                                    return Token::Tag(self.tag);
                                }
                                '\u{0000}' => {
                                    (self.error_handler)("unexpected-null-character");
                                    self.tag.tag_name.push('\u{FFFD}');
                                }
                                _ => {
                                    self.tag.tag_name.push(char);
                                }
                            }
                        }
                        None => {
                            (self.error_handler)("eof-in-tag");
                            return Token::EndOfFile();
                        }
                    }
                }
                RCDATALessThanSignState => {
                    //TODO
                }
                RCDATAEndTagOpenState => {
                    //TODO
                }
                RCDATAEndTagNameState => {
                    //TODO
                }
                ScriptDataLessThanSignState => {
                    //TODO
                }
                ScriptDataEndTagOpenState => {
                    //TODO
                }
                ScriptDataEndTagNameState => {
                    //TODO
                }
                ScriptDataEscapeStartState => {
                    //TODO
                }
                ScriptDataEscapeStartDashState => {
                    //TODO
                }
                ScriptDataEscapedState => {
                    //TODO
                }
                ScriptDataEscapedDashState => {
                    //TODO
                }
                ScriptDataEscapedDashDashState => {
                    //TODO
                }
                ScriptDataEscapedLessThanSignState => {
                    //TODO
                }
                ScriptDataEscapedEndTagOpenState => {
                    //TODO
                }
                ScriptDataEscapedEndTagNameState => {
                    //TODO
                }
                ScriptDataDoubleEscapeStartState => {
                    //TODO
                }
                ScriptDataDoubleEscapedState => {
                    //TODO
                }
                ScriptDataDoubleEscapedDashState => {
                    //TODO
                }
                ScriptDataDoubleEscapedDashDashState => {
                    //TODO
                }
                ScriptDataDoubleEscapedLessThanSignState => {
                    //TODO
                }
                ScriptDataDoubleEscapeEndState => {
                    //TODO
                }
                BeforeAttributeNameState => {
                    match self.next_character() {
                        Some(char) => {
                            match char {
                                '\u{0009}' | '\u{000A}' | '\u{000C}' | ' ' => {
                                    //Ignored
                                }
                                '/' | '>' => {
                                    self.reconsume = Option::Some(char);
                                    self.state = LexerState::AfterAttributeNameState;
                                    return self.next_token();
                                }
                                '=' => {
                                    (self.error_handler)("unexpected-equals-sign-before-attribute-name");
                                    self.state = LexerState::AttributeNameState;
                                    return Token::StartAttribute();
                                }
                                _ => {
                                    self.reconsume = Option::Some(char);
                                    self.State = LexerState::AttributeNameState;
                                }
                            }
                        }
                        None => {
                            self.state = LexerState::AfterAttributeNameState;
                        }
                    }
                }
                AttributeNameState => {
                    match self.next_character() {
                        Some(char) => {
                            match char {
                                '\u{0009}' | '\u{000A}' | '\u{000C}' | ' ' | '/' | '>' => {
                                    self.reconsume = Option::Some(char);
                                    self.state = LexerState::AfterAttributeNameState;
                                }
                                '=' => {
                                    self.state = LexerState::BeforeAttributeValueState;
                                }
                                'A'..='Z' => {
                                    match self.get_top_token() {
                                        Token::Attribute { mut name, value } => {
                                            name.push(char.to_ascii_lowercase());
                                        }
                                        _ => {} // Doesn't happen
                                    }
                                }
                                '\u{0000}' => {
                                    (self.error_handler)("unexpected-null-character");
                                    match self.get_top_token() {
                                        Token::Attribute { mut name, value } => {
                                            name.push('\u{FFFD}');
                                        }
                                        _ => {} // Doesn't happen
                                    }
                                }
                                '\"' | '\'' | '<' => {
                                    (self.error_handler)("unexpected-character-in-attribute-name");
                                    match self.get_top_token() {
                                        Token::Attribute { mut name, value } => {
                                            name.push(char);
                                        }
                                        _ => {} // Doesn't happen
                                    }
                                }
                                _ => {
                                    match self.get_top_token() {
                                        Token::Attribute { mut name, value } => {
                                            name.push(char);
                                        }
                                        _ => {} // Doesn't happen
                                    }
                                }
                            }
                        }
                        None => {
                            self.state = LexerState::AfterAttributeNameState;
                        }
                    }
                }
                AfterAttributeNameState => {
                    //TODO
                }
                BeforeAttributeValueState => {
                    //TODO
                }
                AttributeValueDoubleQuotedState => {
                    //TODO
                }
                AttributeValueSingleQuotedState => {
                    //TODO
                }
                AttributeValueUnquotedState => {
                    //TODO
                }
                AfterAttributeValueQuotedState => {
                    //TODO
                }
                SelfClosingStartTagState => {
                    //TODO
                }
                BogusCommentState => {
                    //TODO
                }
                MarkupDeclarationOpenState => {
                    //TODO
                }
                CommentStartState => {
                    //TODO
                }
                CommentStartDashState => {
                    //TODO
                }
                CommentState => {
                    //TODO
                }
                CommentLessThanSignState => {
                    //TODO
                }
                CommentLessThanSignBangState => {
                    //TODO
                }
                CommentLessThanSignBangDashState => {
                    //TODO
                }
                CommentLessThanSignBangDashDashState => {
                    //TODO
                }
                CommentEndDashState => {
                    //TODO
                }
                CommentEndState => {
                    //TODO
                }
                CommentEndBangState => {
                    //TODO
                }
                DOCTYPEState => {
                    //TODO
                }
                BeforeDOCTYPENameState => {
                    //TODO
                }
                DOCTYPENameState => {
                    //TODO
                }
                AfterDOCTYPENameState => {
                    //TODO
                }
                AfterDOCTYPEPublicKeywordState => {
                    //TODO
                }
                BeforeDOCTYPEPublicIdentifierState => {
                    //TODO
                }
                DOCTYPEPublicIdentifierDoubleQuotedState => {
                    //TODO
                }
                DOCTYPEPublicIdentifierSingleQuotedState => {
                    //TODO
                }
                AfterDOCTYPEPublicIdentifierState => {
                    //TODO
                }
                BetweenDOCTYPEPublicAndSystemIdentifiersState => {
                    //TODO
                }
                AfterDOCTYPESystemKeywordState => {
                    //TODO
                }
                BeforeDOCTYPESystemIdentifierState => {
                    //TODO
                }
                DOCTYPESystemIdentifierDoubleQuotedState => {
                    //TODO
                }
                DOCTYPESystemIdentifierSingleQuotedState => {
                    //TODO
                }
                AfterDOCTYPESystemIdentifierState => {
                    //TODO
                }
                BogusDOCTYPEState => {
                    //TODO
                }
                CDATASectionState => {
                    //TODO
                }
                CDATASectionBracketState => {
                    //TODO
                }
                CDATASectionEndState => {
                    //TODO
                }
                CharacterReferenceState => {
                    match self.input.next() {
                        Some(char) => {
                            match char {
                                'a'..='Z' => {
                                    self.reconsume = Option::Some(char);
                                    self.state = LexerState::NamedCharacterReferenceState;
                                }
                                '#' => {
                                    self.buffer.push('#');
                                    self.state = LexerState::NumericCharacterReferenceState;
                                }
                                _ => {
                                    self.reconsume = Option::Some(char);
                                    match self.return_state {
                                        Some(state) => {
                                            self.state = state;
                                        }
                                        None => {} //Never happens
                                    }
                                    self.buffer = &mut vec![];
                                    return Token::CharacterReference('&');
                                }
                            }
                        }
                        None => {
                            match self.return_state {
                                Some(state) => {
                                    self.state = state;
                                }
                                None => {} //Never happens
                            }
                            self.buffer = &mut vec![];
                            return Token::CharacterReference('&');
                        }
                    }
                }
                NamedCharacterReferenceState => {
                    //TODO
                }
                AmbiguousAmpersandState => {
                    //TODO
                }
                NumericCharacterReferenceState => {
                    //TODO
                }
                HexadecimalCharacterReferenceStartState => {
                    //TODO
                }
                DecimalCharacterReferenceStartState => {
                    //TODO
                }
                HexadecimalCharacterReferenceState => {
                    //TODO
                }
                DecimalCharacterReferenceState => {
                    //TODO
                }
                NumericCharacterReferenceEndState => {
                    //TODO
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
    fn next_token(mut self, mut self: Lexer) -> Option<Token> {
        loop {
            match self.next_character() {
                Some(char) => {
                    match char {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' => {
                            //Ignored
                        }
                        '/' => {
                            self.tag.attributes.insert(self.attribute.0, self.attribute.1);
                            self.State = Box::new(SelfClosingStartTagState { tag: self.tag });
                            return self.next_token();
                        }
                        '=' => {
                            self.State = Box::new(BeforeAttributeValueState { tag: self.tag, attribute: self.attribute });
                            return self.next_token();
                        }
                        '>' => {
                            self.tag.attributes.insert(self.attribute.0, self.attribute.1);
                            self.State = Box::new(DataState {});
                            return Token::Tag(self.tag);
                        }
                        _ => {
                            self.tag.attributes.insert(self.attribute.0, self.attribute.1);
                            self.State = Box::new(AttributeNameState { tag: self.tag, attribute: (vec![], vec![]) });
                            return self.next_token();
                        }
                    }
                }
                None => {
                    (self.error_handler)("eof-in-tag");
                    return Token::Tag(self.tag);
                }
            }
        }
    }
}

pub struct SelfClosingStartTagState {
    tag: TagData,
}

impl LexerState for SelfClosingStartTagState {
    fn next_token(mut self, mut self: Lexer) -> Option<Token> {
        match self.next_character() {
            Some(char) => {
                match char {
                    '>' => {
                        self.tag.self_closing = true;
                        self.State = Box::new(DataState {});
                        return Token::Tag(self.tag);
                    }
                    _ => {
                        (self.error_handler)("unexpected-solidus-in-tag");
                        self.reconsume = Option::Some(char);
                        self.State = Box::new(BeforeAttributeNameState { tag: self.tag });
                        return self.next_token();
                    }
                }
            }
            None => {
                (self.error_handler)("eof-in-tag");
                Token::EndOfFile()
            }
        }
    }
}

pub struct BeforeAttributeValueState {
    tag: TagData,
    attribute: (Vec<char>, Vec<char>),
}

impl LexerState for BeforeAttributeValueState {
    fn next_token(mut self, mut self: Lexer) -> Option<Token> {
        loop {
            match self.next_character() {
                Some(char) => {
                    match char {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | ' ' => {
                            //Ignored
                        }
                        '\"' => {
                            self.State = Box::new(AttributeValueDoubleQuotedState {
                                tag: self.tag,
                                attribute: self.attribute,
                                ending: false,
                            });
                            return self.next_token();
                        }
                        '\'' => {
                            self.State = Box::new(AttributeValueSingleQuotedState {
                                tag: self.tag,
                                attribute: self.attribute,
                                ending: false,
                            });
                            return self.next_token();
                        }
                        '>' => {
                            (self.error_handler)("missing-attribute-value");
                            self.State = Box::new(DataState {});
                            self.tag.attributes.insert(self.attribute.0, self.attribute.1);
                            return Token::Tag(self.tag);
                        }
                        _ => {
                            self.reconsume = Option::Some(char);
                            self.State = Box::new(AttributeValueUnquotedState {
                                tag: self.tag,
                                attribute: self.attribute,
                                ending: false,
                            });
                            return self.next_token();
                        }
                    }
                }
                None => {
                    self.State = Box::new(AttributeValueUnquotedState {
                        tag: self.tag,
                        attribute: self.attribute,
                        ending: false,
                    });
                    return self.next_token();
                }
            }
        }
    }
}

pub struct AttributeValueDoubleQuotedState {
    tag: TagData,
    attribute: (Vec<char>, Vec<char>),
    ending: bool,
}

impl LexerState for AttributeValueDoubleQuotedState {
    fn next_token(mut self, mut self: Lexer) -> Option<Token> {
        loop {
            match self.next_character() {
                Some(char) => {
                    match char {
                        '\"' => {
                            self.State = Box::new(AfterAttributeValueQuotedState {
                                tag: self.tag,
                                attribute: self.attribute,
                                ending: false,
                            });
                            return self.next_token();
                        }
                        '&' => {
                            self.State = Box::new(CharacterReferenceState { return_State: self.State });
                            return self.next_token();
                        }
                        '\u{0000}' => {
                            (self.error_handler)("unexpected-null-character");
                            self.attribute.1.push('\u{FFFD}');
                        }
                        _ => {
                            self.attribute.1.push(char);
                        }
                    }
                }
                None => {
                    return if !self.ending {
                        self.ending = true;
                        Token::Tag(self.tag)
                    } else {
                        (self.error_handler)("eof-in-tag");
                        Token::EndOfFile()
                    };
                }
            }
        }
    }
}

pub struct AttributeValueSingleQuotedState {
    tag: TagData,
    attribute: (Vec<char>, Vec<char>),
    ending: bool,
}

impl LexerState for AttributeValueSingleQuotedState {
    fn next_token(mut self, mut self: Lexer) -> Option<Token> {
        loop {
            match self.next_character() {
                Some(char) => {
                    match char {
                        '\'' => {
                            self.State = Box::new(AfterAttributeValueQuotedState {
                                tag: self.tag,
                                attribute: self.attribute,
                                ending: false,
                            });
                            return self.next_token();
                        }
                        '&' => {
                            self.State = Box::new(CharacterReferenceState { return_State: self.State });
                            return self.next_token();
                        }
                        '\u{0000}' => {
                            (self.error_handler)("unexpected-null-character");
                            self.attribute.1.push('\u{FFFD}');
                        }
                        _ => {
                            self.attribute.1.push(char);
                        }
                    }
                }
                None => {
                    return if !self.ending {
                        self.ending = true;
                        Token::Tag(self.tag)
                    } else {
                        (self.error_handler)("eof-in-tag");
                        Token::EndOfFile()
                    };
                }
            }
        }
    }
}

pub struct AttributeValueUnquotedState {
    tag: TagData,
    attribute: (Vec<char>, Vec<char>),
    ending: bool,
}

impl LexerState for AttributeValueUnquotedState {
    fn next_token(mut self, mut self: Lexer) -> Option<Token> {
        loop {
            match self.next_character() {
                Some(char) => {
                    match char {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | ' ' => {
                            self.State = Box::new(BeforeAttributeNameState { tag: self.tag });
                            return self.next_token();
                        }
                        '&' => {
                            self.State = Box::new(CharacterReferenceState { return_State: self.State });
                            return self.next_token();
                        }
                        '>' => {
                            self.State = Box::new(DataState {});
                            self.tag.attributes.insert(self.attribute.0, self.attribute.1);
                            return Token::Tag(self.tag);
                        }
                        '\u{0000}' => {
                            (self.error_handler)("unexpected-null-character");
                            self.attribute.1.push('\u{FFFD}');
                        }
                        '\"' | '\'' | '<' | '=' | '`' => {
                            (self.error_handler)("unexpected-character-in-unquoted-attribute-value");
                            self.attribute.1.push(char);
                        }
                        _ => {
                            self.attribute.1.push(char);
                        }
                    }
                }
                None => {
                    return if !self.ending {
                        self.ending = true;
                        Token::Tag(self.tag)
                    } else {
                        (self.error_handler)("eof-in-tag");
                        Token::EndOfFile()
                    };
                }
            }
        }
    }
}

pub struct AfterAttributeValueQuotedState {
    tag: TagData,
    attribute: (Vec<char>, Vec<char>),
    ending: bool,
}

impl LexerState for AfterAttributeValueQuotedState {
    fn next_token(mut self, mut self: Lexer) -> Option<Token> {
        loop {
            match self.next_character() {
                Some(char) => {
                    match char {
                        '\u{0009}' | '\u{000A}' | '\u{000C}' | ' ' => {
                            self.State = Box::new(BeforeAttributeNameState { tag: self.tag });
                            return self.next_token();
                        }
                        '/' => {
                            self.State = Box::new(SelfClosingStartTagState { tag: self.tag });
                            return self.next_token();
                        }
                        '\u{0000}' => {
                            self.attribute.1.push('\u{FFFD}');
                        }
                        _ => {
                            return if !self.ending {
                                self.ending = true;
                                Token::Tag(self.tag)
                            } else {
                                (self.error_handler)("missing-whitespace-between-attributes");
                                Token::EndOfFile()
                            };
                        }
                    }
                }
                None => {
                    (self.error_handler)("eof-in-tag");
                    return Token::EndOfFile();
                }
            }
        }
    }
}

pub struct NamedCharacterReferenceState {}

impl LexerState for NamedCharacterReferenceState {
    fn next_token(self, mut self: Lexer) -> Option<Token> {
        Token::EndOfFile()
    }
}

pub struct NumericCharacterReferenceState {}

impl LexerState for NumericCharacterReferenceState {
    fn next_token(self, mut self: Lexer) -> Option<Token> {
        Token::EndOfFile()
    }
}

pub struct MarkupDeclarationOpenState {}

impl LexerState for MarkupDeclarationOpenState {
    fn next_token(self, mut self: Lexer) -> Option<Token> {
        Token::EndOfFile()
    }
}

pub struct BogusCommentState {}

impl LexerState for BogusCommentState {
    fn next_token(self, mut self: Lexer) -> Option<Token> {
        Token::EndOfFile()
    }
}