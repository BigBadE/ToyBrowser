use std::collections::HashMap;

pub enum Token {
    Character(char),
    CharacterReference(char),
    StartTag(),
    EndTag(),
    StartComment(),
    Comment(Vec<char>),
    StartAttribute(),
    Attribute { name: Vec<char>, value: Vec<char> },
    Tag(TagData),
    EndOfFile()
}

pub struct TagData {
    pub tag_name: Vec<char>,
    pub self_closing: bool,
    pub attributes: HashMap<Vec<char>, Vec<char>>
}

impl TagData {
    pub fn new() -> TagData {
        TagData { tag_name: Vec::new(), self_closing: false, attributes: HashMap::new() }
    }
}