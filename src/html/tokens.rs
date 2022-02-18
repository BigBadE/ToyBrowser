pub enum Token {
    Character(char),
    CharacterReference(char),
    Comment(Vec<char>),
    StartTag(),
    Tag(Vec<char>),
    EndOfFile()
}