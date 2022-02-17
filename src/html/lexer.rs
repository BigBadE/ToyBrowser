use std::iter::Peekable;
use std::str::Chars;
use crate::html::tokens::Token;

pub struct Lexer<'a> {
    pos: usize,
    input: Peekable<Chars<'a>>,
    state: &'a Box<dyn LexerState<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: String) -> Lexer<'a> {
        Lexer {
            pos: 0,
            input: input.chars().peekable(),
            state: &Box::new(DataState {}),
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.state.next_token(self)
    }
}

trait LexerState<'a> {
    fn next_token(&mut self, lexer: &'a mut Lexer) -> Token;
}

pub struct DataState {}

impl<'a> LexerState<'a> for DataState {
    fn next_token(&mut self, lexer: &'a mut Lexer) -> Token {
        match lexer.input.next() {
            Some(char) => {
                match char {
                    '&' => {
                        lexer.state = Box::new(CharacterReferenceState::new(&lexer.state));
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

pub struct CharacterReferenceState<'a> {
    return_state: &'a Box<dyn LexerState<'a>>,
}

impl<'a> LexerState<'a> for CharacterReferenceState<'a> {
    fn next_token(&mut self, lexer: &mut Lexer) -> Token {}
}

impl<'a> CharacterReferenceState<'a> {
    fn new(return_state: &'a Box<dyn LexerState<'a>>) -> CharacterReferenceState<'a> {
        CharacterReferenceState { return_state }
    }
}

pub struct TagOpenState {}

impl<'a> LexerState<'a> for TagOpenState {
    fn next_token(&mut self, lexer: &mut Lexer) -> Token {}
}