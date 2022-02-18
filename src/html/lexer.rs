use std::iter::Peekable;
use std::str::Chars;
use crate::html::tokens::Token;

pub struct Lexer<'a> {
    pos: usize,
    reconsume: Option<char>,
    input: Chars<'a>,
    state: Box<dyn LexerState>,
    buffer: &'a mut Vec<char>,
    error_handler: &'a dyn Fn(&str)
}

impl<'a> Lexer<'a> {
    pub fn new(input: String, error_handler: &'a dyn Fn(&str)) -> Lexer<'a> {
        Lexer {
            pos: 0,
            reconsume: Option::None,
            input: input.chars(),
            state: Box::new(DataState {}),
            buffer: &mut Vec::new(),
            error_handler
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
        lexer.buffer = &mut Vec::new();
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
                if char > 'a' && char < 'Z' {
                    lexer.reconsume = Option::Some(char);
                    lexer.state = Box::new(NamedCharacterReferenceState {});
                    return lexer.next_token();
                }
                match char {
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
    ending_next: bool
}

impl LexerState for TagOpenState {
    fn next_token(self, mut lexer: Lexer) -> Token {
        match lexer.input.next() {
            Some(char) => {
                if char > 'a' && char < 'Z' {
                    lexer.reconsume = Option::Some(char);
                    lexer.state = Box::new(TagNameState {});
                    return Token::StartTag();
                }
                match char {
                    '!' => {
                        lexer.state = Box::new(MarkupDeclarationOpenState {});
                        lexer.next_token()
                    }
                    '/' => {
                        lexer.state = Box::new(EndTagOpenState {});
                        lexer.next_token()
                    }
                    '?' => {
                        (lexer.error_handler)("unexpected-question-mark-instead-of-tag-name");
                        lexer.reconsume = Option::Some('?');
                        lexer.state = Box::new(BogusCommentState {});
                        Token::Comment(Vec::new())
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
                    return Token::Character('>')
                }
                Token::EndOfFile()
            }
        }
    }
}

pub struct EndTagOpenState {}

impl LexerState for EndTagOpenState {
    fn next_token(self, mut lexer: Lexer) -> Token {
        Token::EndOfFile()
    }
}

pub struct TagNameState {}

impl LexerState for TagNameState {
    fn next_token(self, mut lexer: Lexer) -> Token {
        Token::EndOfFile()
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