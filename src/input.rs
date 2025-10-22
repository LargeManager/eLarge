use crate::snippets::function::{self, EntityState, SnippetCommand};
use crate::snippets::snippet::SnippetState;
use crate::{
    cursor::CursorDirection,
    editor::{Document, Mode},
};
use crossterm::ExecutableCommand;
use crossterm::cursor::SetCursorStyle;
use crossterm::event::{self, Event, KeyCode};
use ratatui::DefaultTerminal;
use std::{
    io::{self, Error},
    time::Duration,
};

macro_rules! one_of {
    ($key:expr, [$($ch:expr),*], $action:block) => {
        match $key {
            $(KeyCode::Char($ch))|* => $action,
            _ => {}
        }
    };
}

impl TryFrom<KeyCode> for CursorDirection {
    type Error = io::Error;
    fn try_from(value: KeyCode) -> Result<Self, Self::Error> {
        match value {
            KeyCode::Char('h') => Ok(CursorDirection::Left),
            KeyCode::Char('k') => Ok(CursorDirection::Up),
            KeyCode::Char('j') => Ok(CursorDirection::Down),
            KeyCode::Char('l') => Ok(CursorDirection::Right),
            _ => Err(Error::new(
                io::ErrorKind::InvalidInput,
                "invalid cursor direction",
            )),
        }
    }
}

/// Handle input; returns Ok(true) when quit requested.
pub fn handle_input(
    doc: &mut Document,
    snippet: &mut function::Snippet,
    terminal: &mut DefaultTerminal,
    viewport_height: Option<usize>,
) -> Result<bool, io::Error> {
    if event::poll(Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            match doc.mode {
                Mode::Normal => {
                    match key.code {
                        KeyCode::Char('q') => return Ok(true),
                        KeyCode::Char('i') => {
                            doc.mode = Mode::Insert;
                        }
                        KeyCode::Char('f') => {
                            // assuming lang is rust
                            snippet.start(SnippetState::Create, doc);
                        }
                        _ => {}
                    };

                    let keycode = key.code;

                    // used for key combination commands
                    if let Ok(snippet_state) = SnippetState::try_from(keycode) {
                        snippet.check_command(SnippetCommand::SnippetState(snippet_state), doc);
                    }

                    if let Ok(entity_state) = EntityState::try_from(keycode) {
                        snippet.check_command(SnippetCommand::EntityState(entity_state), doc);
                    }

                    // free cursor movement will be changed instead to act on snippet entities
                    if let Ok(cursor_direction) = CursorDirection::try_from(keycode) {
                        doc.cursor.move_cursor(&doc.editor.text, cursor_direction)
                    }
                }
                Mode::Insert => match key.code {
                    KeyCode::Esc => {
                        doc.mode = Mode::Normal;
                    }
                    KeyCode::Char(character) => {
                        let is_collected = snippet.entity_manager.collect_chunk_buffer(
                            character,
                            doc,
                            &mut snippet.snippet_state,
                            &mut snippet.notifications,
                        );
                        // Insert the character at current cursor position
                        if is_collected.is_err() {
                            doc.cursor_pos_insert(&character.to_string());
                            doc.cursor.pos.x += 1;
                        }
                    }
                    KeyCode::Enter => {
                        doc.cursor_pos_insert("\n");
                        doc.cursor.pos.y += 1;
                        doc.cursor.pos.x = 0;
                    }
                    KeyCode::Backspace => {
                        // TODO can be optimized as start row also checks for newlines
                        let idx = doc.cursor_get_idx();
                        let prev_idx = idx.saturating_sub(1);
                        let cursor_pos = &mut doc.cursor.pos;

                        if !doc.editor.is_start_row(idx) {
                            doc.editor.text.remove(prev_idx..idx);
                            cursor_pos.x -= 1;
                        } else if cursor_pos.y != 0 {
                            doc.editor.text.remove(prev_idx..idx);
                            let pos = doc.editor.idx_to_pos(prev_idx);
                            doc.cursor_set_pos(pos);
                        }
                        // todo backspace should not work for deleteion of syntax
                    }
                    _ => {}
                },
            }
            match doc.mode {
                Mode::Normal => terminal.backend_mut().execute(SetCursorStyle::SteadyBlock),
                Mode::Insert => terminal.backend_mut().execute(SetCursorStyle::SteadyBar),
            };
        }
    }
    Ok(false)
}
