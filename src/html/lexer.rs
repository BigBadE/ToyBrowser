use std::str::Chars;
use crate::html::tokens::Token;

pub struct Lexer<'a> {
    pos: usize,
    input: Chars<'a>,
    state: Box<dyn LexerState>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: String) -> Lexer<'a> {
        Lexer {
            pos: 0,
            input: input.chars(),
            state: Box::new(DataState { current_data: Vec::new() }),
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.state.next_token(self)
    }
}

trait LexerState {
    fn next_token(&mut self, lexer: &mut Lexer) -> Token;
}

pub struct DataState {
    current_data: Vec<u8>,
}

impl LexerState for DataState {
    fn next_token(&mut self, lexer: &mut Lexer) -> Token {
        match lexer.input[lexer.pos] {
            '&' => {
                lexer.state = Box::new(CharacterReferenceState { return_state: lexer.state });
                lexer.state.next_token(lexer)
            }
        }
    }
}

pub struct CharacterReferenceState {
    return_state: Box<dyn LexerState>,
}

impl LexerState for CharacterReferenceState {
    fn next_token(&mut self, lexer: &mut Lexer) -> Token {

    }
}

pub struct TagOpenState {}

impl LexerState for TagOpenState {
    fn next_token(&mut self, lexer: &mut Lexer) -> Token {

    }
}