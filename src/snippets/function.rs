use crossterm::event::KeyCode;
use serde::{Deserialize, Serialize, de};
use std::io::{self, Error};
use std::ops::Range;

use crate::editor::{Document, Mode};

use super::grammer::{Surround, Token};
use super::snippet::SnippetState;

#[derive(Deserialize, Serialize)]
struct TokenTable {
    token: Token,
    token_range: Range<usize>,
    value: String,
    value_range: Range<usize>,
    string_buff: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize)]
struct Entity {
    name: TokenTable,
    param: TokenTable,
    result: TokenTable,
    body: TokenTable,
    doc: TokenTable,
}

impl Entity {
    fn get_mut_table(&mut self, entity_state: &EntityState) -> &mut TokenTable {
        match entity_state {
            EntityState::None => panic!(),
            EntityState::Name => &mut self.name,
            EntityState::Param => &mut self.param,
            EntityState::Result => &mut self.result,
            EntityState::Body => &mut self.body,
        }
    }
    fn get_ref_table(&mut self, entity_state: &EntityState) -> &TokenTable {
        match entity_state {
            EntityState::None => panic!(),
            EntityState::Name => &self.name,
            EntityState::Param => &self.param,
            EntityState::Result => &self.result,
            EntityState::Body => &self.body,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, PartialEq)]
pub enum EntityState {
    None = 0,
    Name = 1,
    Param = 2,
    Result = 3,
    Body = 4,
}

impl Default for EntityState {
    fn default() -> Self {
        Self::None
    }
}
impl TryFrom<KeyCode> for EntityState {
    type Error = io::Error;
    fn try_from(value: KeyCode) -> Result<Self, Self::Error> {
        match value {
            KeyCode::Char('n') => Ok(EntityState::Name),
            KeyCode::Char('p') => Ok(EntityState::Param),
            KeyCode::Char('r') => Ok(EntityState::Result),
            KeyCode::Char('b') => Ok(EntityState::Body),
            _ => Err(Error::new(
                io::ErrorKind::InvalidInput,
                "invalid cursor direction",
            )),
        }
    }
}

#[derive(Deserialize, Serialize)]
enum InsertState {
    None,
    Enter,
    Exit,
}
impl Default for InsertState {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Deserialize, Serialize)]
pub struct EntityManager {
    entity: Entity,
    state: EntityState,
    order: [EntityState; 5], // in which order collet the user input
    insert_state: InsertState,
    char_buffer: Vec<char>,
}

impl EntityManager {
    // basically lower level helpers
    fn reset_state(&mut self, doc: &mut Document, snippet_state: &mut SnippetState) {
        *snippet_state = SnippetState::None;
        doc.mode = Mode::Insert;
        self.insert_state = InsertState::None
    }

    fn get_buffer(
        cursor_idx: usize,
        char_buffer: &mut Vec<char>,
        table: &mut TokenTable,
    ) -> String {
        let res: String = char_buffer.iter().collect();
        let insert_range = Range {
            start: cursor_idx - res.len(),
            end: cursor_idx,
        };
        table.value_range = insert_range;
        char_buffer.clear();
        res
    }

    pub fn check_snippet_state(snippet_state: &mut SnippetState) -> Result<(), ()> {
        let is_match = matches!(snippet_state, SnippetState::Create | SnippetState::Delete);
        match is_match {
            true => Ok(()),
            false => return Err(()),
        }
    }
    fn update_entity_range(&mut self) {
        // used when entity is modified after creating
    }
}

impl EntityManager {
    fn get_next_state(&mut self) -> EntityState {
        // getting and setting the state is done at the same time resetted at last index
        let next_state_idx: usize = self.state as usize + 1;
        let next_state = self.order.get(next_state_idx);
        let next_state = match next_state {
            Some(value) => *value,
            None => EntityState::None,
        };
        self.state = next_state;
        next_state
    }

    pub fn insert_chunk(&mut self, doc: &mut Document, snippet_state: &mut SnippetState) {
        let next_state = self.get_next_state(); // convert to referen option later
        if next_state == EntityState::None {
            self.reset_state(doc, snippet_state);
            return;
        }
        let table = self.entity.get_mut_table(&next_state);
        let insert_chunk = table.token.to_insert_chunk();

        match self.insert_state {
            // triggered when space is present
            InsertState::Enter => {
                doc.cursor.pos.x += insert_chunk.cursor_jump.enter;
                self.insert_state = InsertState::Exit;
            }
            InsertState::Exit => {
                doc.cursor.pos.x += insert_chunk.cursor_jump.exit;
                self.insert_state = InsertState::Enter;
            }
            _ => {}
        }
        doc.cursor_pos_insert(&insert_chunk.value);
        let cursor_idx = doc.cursor_get_idx(); // use this to delete insert_chunk later
        let token_range = Range {
            start: cursor_idx - insert_chunk.value.len(),
            end: cursor_idx,
        };
        table.token_range = token_range;
        doc.mode = Mode::Insert
    }

    pub fn collect_chunk_buffer(
        &mut self,
        character: char,
        doc: &mut Document,
        snippet_state: &mut SnippetState,
        system_msg: &mut Option<String>,
    ) -> Result<(), ()> {
        Self::check_snippet_state(snippet_state)?;
        if character == ' ' {
            let table: &mut TokenTable = self.entity.get_mut_table(&self.state);
            let token: &Token = &table.token;

            match self.char_buffer.is_empty() {
                true => match (token.deletable, token.skippable) {
                    (true, true) => doc.editor.text.remove(table.token_range.clone()),
                    (true, false) => {}
                    (false, true) => {}
                    (false, false) => {
                        *system_msg = Some("Value is Required in order to progress".to_string());
                        return Ok(());
                    }
                },
                false => {}
            }

            *system_msg = Some("None".to_string());
            let data = Self::get_buffer(doc.cursor_get_idx(), &mut self.char_buffer, table);
            table.value = data;
            self.insert_chunk(doc, snippet_state);
            Ok(())
        } else {
            self.char_buffer.push(character);
            Err(())
        }
    }
}

// used for key combinations
#[derive(Deserialize, Serialize)]
pub enum SnippetCommand {
    SnippetState(SnippetState),
    EntityState(EntityState),
    None,
}

impl Default for SnippetCommand {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Deserialize, Serialize, Default)]
pub struct Snippet {
    pub snippet_state: SnippetState,
    pub entity_manager: EntityManager,
    pub command_buffer: Vec<SnippetCommand>,
    pub notifications: Option<String>,
}
impl Snippet {
    pub fn start(&mut self, snippet_state: SnippetState, doc: &mut Document) {
        self.snippet_state = snippet_state;
        match self.is_trigger_state() {
            true => {}
            false => return,
        }
        self.entity_manager.insert_state = InsertState::Enter;
        self.entity_manager
            .insert_chunk(doc, &mut self.snippet_state);
    }

    pub fn is_trigger_state(&self) -> bool {
        matches!(
            self.snippet_state,
            SnippetState::Create | SnippetState::Delete
        )
    }

    // used in normal mode. starm method should be later moved here
    pub fn check_command(&mut self, snippet_command: SnippetCommand, doc: &mut Document) {
        self.command_buffer.push(snippet_command);
        if self.command_buffer.len() == 2_usize {
            // we got the both
            if let (
                SnippetCommand::SnippetState(snippet_state),
                SnippetCommand::EntityState(entity_state),
            ) = (&self.command_buffer[0], &self.command_buffer[1])
            {
                let table = self.entity_manager.entity.get_ref_table(entity_state);

                let mut swap_action = || {
                    let jump_index: usize = table.value_range.start;
                    doc.cursor_set_pos(doc.cursor_get_loc(jump_index));
                    doc.editor.text.remove(table.value_range.clone());
                    doc.mode = Mode::Insert;
                };

                match snippet_state {
                    SnippetState::Create => {}
                    SnippetState::Delete => {
                        if !table.token.skippable {
                            swap_action();
                        } else {
                            doc.editor.text.remove(table.value_range.clone());
                        }
                    }
                    SnippetState::Swap => {
                        swap_action();
                    }
                    SnippetState::None => {}
                    _ => {}
                }
            };
            self.command_buffer.clear();
        }
    }
}
impl Default for EntityManager {
    fn default() -> Self {
        Self {
            entity: Default::default(),
            state: Default::default(),
            order: [
                EntityState::None,
                EntityState::Name,
                EntityState::Param,
                EntityState::Result,
                EntityState::Body,
            ],
            char_buffer: Default::default(),
            insert_state: Default::default(),
        }
    }
}
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct CharPair {
    pub open: char,
    pub close: char,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StringPair {
    pub open: String,
    pub close: String,
}

pub struct KeyboardPairs {
    pub underscore_spaces: CharPair,
    pub spaces: CharPair,
    pub parentheses: CharPair,
    pub curly_braces: CharPair,
    pub square_brackets: CharPair,
    pub angle_brackets: CharPair,
    pub single_quotes: CharPair,
    pub double_quotes: CharPair,
}

const KEYBOARD_PAIRS: KeyboardPairs = KeyboardPairs {
    underscore_spaces: CharPair {
        open: '_',
        close: '_',
    },
    spaces: CharPair {
        open: ' ',
        close: ' ',
    },
    parentheses: CharPair {
        open: '(',
        close: ')',
    },
    curly_braces: CharPair {
        open: '{',
        close: '}',
    },
    square_brackets: CharPair {
        open: '[',
        close: ']',
    },
    angle_brackets: CharPair {
        open: '<',
        close: '>',
    },
    single_quotes: CharPair {
        open: '\'',
        close: '\'',
    },
    double_quotes: CharPair {
        open: '"',
        close: '"',
    },
};

#[derive(Serialize, Deserialize)]
pub enum Pair {
    CharPair(CharPair),
    StringPair(StringPair),
}

impl Default for Entity {
    fn default() -> Self {
        Self {
            name: TokenTable {
                token: Token {
                    keyword: "fn".to_string(),
                    syntax: Surround::Right(" ".to_string()), // any extra characters that keyword
                    // might require
                    format: Surround::None, // wraps around syntax without conflicting with it (yet to
                    // implement )
                    deletable: false, // can we delete entire token when value is skipped?
                    skippable: false, // can we skip token input ?
                    use_lsp: false,   // just placeholder for future functionality (possibly for lsp
                                      // autosuggestions on user input)
                },
                token_range: Range::default(),
                value_range: Range::default(),
                value: String::default(), // actual input
                string_buff: None,        // used for multiple key inputs (vectorized data)
            },
            param: TokenTable {
                token: Token {
                    keyword: "".to_string(),
                    syntax: Surround::Pair(Pair::CharPair(KEYBOARD_PAIRS.parentheses)),
                    format: Surround::None,
                    deletable: false,
                    skippable: true,
                    use_lsp: true,
                },
                token_range: Range::default(),
                value_range: Range::default(),
                value: String::default(),
                string_buff: Some(Vec::default()),
            },
            result: TokenTable {
                token: Token {
                    keyword: "->".to_string(),
                    syntax: Surround::None,
                    format: Surround::Pair(Pair::CharPair(KEYBOARD_PAIRS.spaces)),
                    deletable: true,
                    skippable: true,
                    use_lsp: true,
                },
                token_range: Range::default(),
                value_range: Range::default(),
                value: String::default(),
                string_buff: None,
            },
            body: TokenTable {
                token: Token {
                    keyword: "".to_string(),
                    syntax: Surround::Pair(Pair::CharPair(KEYBOARD_PAIRS.curly_braces)),
                    format: Surround::Left(" ".to_string()),
                    deletable: false,
                    skippable: true,
                    use_lsp: true,
                },
                token_range: Range::default(),
                value_range: Range::default(),
                value: String::default(),
                string_buff: Some(Vec::default()),
            },

            doc: TokenTable {
                token: Token {
                    keyword: " ".to_string(),
                    syntax: Surround::Right(" ".to_string()),
                    format: Surround::None,
                    deletable: false,
                    skippable: false,
                    use_lsp: true,
                },
                token_range: Range::default(),
                value_range: Range::default(),
                value: String::default(),
                string_buff: None,
            },
        }
    }
}
