use std::iter::Peekable;
use std::str::Chars;
use crate::html::tokens::Token;

pub struct Lexer<'a> {
    pos: usize,
    input: Peekable<Chars<'a>>,
    state: Box<dyn LexerState>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: String) -> Lexer<'a> {
        Lexer {
            pos: 0,
            input: input.chars().peekable(),
            state: Box::new(DataState {}),
        }
    }

    pub fn next_token(mut self) -> Token {
        self.state.next_token(self)
    }
}

trait LexerState {
    fn next_token(&mut self, lexer: Lexer) -> Token;
}

pub struct DataState {}

impl LexerState for DataState {
    fn next_token(&mut self, mut lexer: Lexer) -> Token {
        match lexer.input.next() {
            Some(char) => {
                match char {
                    '&' => {
                        lexer.state = Box::new(CharacterReferenceState {
                            return_state: lexer.state,
                            buffer: Vec::new(),
                        });
                        lexer.state.next_token(lexer)
                    }
                    '<' => {
                        lexer.state = Box::new(TagOpenState {});
                        lexer.state.next_token(lexer)
                    }
                    '\u{0000}' => {
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
    buffer: Vec<char>,
}

impl LexerState for CharacterReferenceState {
    fn next_token(&mut self, mut lexer: Lexer) -> Token {

    }
}

pub struct TagOpenState {}

impl LexerState for TagOpenState {
    fn next_token(&mut self, mut lexer: Lexer) -> Token {}
}