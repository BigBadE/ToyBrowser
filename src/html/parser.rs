use crate::html::node::{document, Document, Node};
use crate::html::lexer::Lexer;

pub struct Parser<'a> {
    stack: Vec<Node>,
    lexer: Lexer<'a>
}

impl<'a> Parser<'a> {
    pub fn new(input: String) -> Parser<'a> {
        Parser { stack: Vec::new(), lexer: Lexer::new(input) }
    }

    pub fn parse(&mut self, text: String) -> Node {
        self.stack.push(document(Vec::new()));

        match self.stack.pop() {
            None => { document(Vec::new()) }
            Some(value) => { value }
        }
    }
}