use serde::{Deserialize, Serialize};

use super::function::Pair;

#[derive(Deserialize, Serialize)]
pub struct StringRange {
    pub start: String,
    pub end: String,
}

#[derive(Deserialize, Serialize)]
pub enum Surround {
    None,
    Left(String),
    Right(String),
    Pair(Pair),
}

impl Default for Surround {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Default)]
pub struct CursorJump {
    // where cursor should end up when entring/exiting snippet chunks (insert mode)
    pub enter: usize,
    pub exit: usize,
}

pub struct InsertChunk {
    pub value: String,
    pub cursor_jump: CursorJump,
}

#[derive(Deserialize, Serialize, Default)]
pub struct Token {
    pub keyword: String,
    pub syntax: Surround,
    pub format: Surround, // TODO add more restriction as it might break next syntax chunk
    pub skippable: bool,
    pub deletable: bool,
    pub use_lsp: bool,
}

impl Token {
    pub fn to_insert_chunk(&self) -> InsertChunk {
        // returns what insert should be and position where cursor shoud lend after insert
        let target = &self.keyword;
        let mut res = String::from("");
        let mut cursor_jump = CursorJump::default();

        match self.syntax {
            Surround::Left(ref value) => {
                res.insert_str(0, value);
                res.push_str(target);
                cursor_jump.enter += res.len()
            }
            Surround::Right(ref value) => {
                res.push_str(target);
                res.push_str(value);
                cursor_jump.enter += res.len()
            }
            Surround::Pair(ref pair) => match pair {
                Pair::CharPair(char_pair) => {
                    res.push(char_pair.open);
                    res.push(char_pair.close);
                    cursor_jump.enter += 1;
                    cursor_jump.exit += 1;
                }
                Pair::StringPair(string_pair) => {
                    res.push_str(&string_pair.open);
                    res.push_str(&string_pair.close);
                    cursor_jump.enter += string_pair.open.len();
                    cursor_jump.exit += string_pair.close.len()
                }
            },
            Surround::None => {}
        };
        match self.format {
            Surround::Left(ref value) => {
                res.insert_str(0, value);
                res.push_str(target);
                cursor_jump.exit += value.len() + target.len()
            }
            Surround::Right(ref value) => {
                res.push_str(target);
                res.push_str(value);
                cursor_jump.exit += value.len() + target.len()
            }
            Surround::Pair(ref pair) => match pair {
                Pair::CharPair(char_pair) => {
                    res.push(char_pair.open);
                    res.push_str(target);
                    res.push(char_pair.close);
                    cursor_jump.enter += res.len() + 1;
                }
                Pair::StringPair(string_pair) => {
                    res.push_str(&string_pair.open);
                    res.push_str(target);
                    res.push_str(&string_pair.close);
                    cursor_jump.enter += res.len() + 1;
                }
            },
            Surround::None => {}
        }

        InsertChunk {
            value: res,
            cursor_jump,
        }
    }
}
