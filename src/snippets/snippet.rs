use std::io::{self, Error};

use crossterm::event::KeyCode;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Deserialize, Serialize)]
pub enum SnippetState {
    Create,   // create snippets from scratch
    Delete,   // delete nodes
    Swap,     // swap nodes (same as delete but cursor will follow)
    Refactor, // move nodes around
    Goto,     // for lsp actions
    None,
}
impl Default for SnippetState {
    fn default() -> Self {
        Self::None
    }
}
impl TryFrom<KeyCode> for SnippetState {
    type Error = io::Error;
    fn try_from(value: KeyCode) -> Result<Self, Self::Error> {
        match value {
            KeyCode::Char('c') => Ok(SnippetState::Create),
            KeyCode::Char('d') => Ok(SnippetState::Delete),
            KeyCode::Char('s') => Ok(SnippetState::Swap),
            // KeyCode::Char('r') => Ok(SnippetState::Refactor),
            _ => Err(Error::new(
                io::ErrorKind::InvalidInput,
                "invalid input for snippet state",
            )),
        }
    }
}
