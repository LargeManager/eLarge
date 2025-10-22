use crate::{
    cursor::{Cursor, Pos},
    features::line_numbers::LineNumbers,
    highlight::lex_and_style,
    snippets::function,
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use ropey::Rope;
use strum::Display;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Display)]
pub enum Mode {
    Normal,
    Insert,
}

pub struct Document {
    pub editor: Editor,
    pub mode: Mode,
    pub cursor: Cursor,
}

impl Document {
    pub fn new(initial_text: &str) -> Self {
        let text = if initial_text.is_empty() {
            Rope::from_str("")
        } else {
            Rope::from_str(initial_text)
        };

        Self {
            editor: Editor { text },
            mode: Mode::Normal,
            cursor: Cursor::default(),
        }
    }

    pub fn ui(&self, f: &mut Frame, snippet: &function::Snippet) {
        let content_area = f.area();

        // rendering line numbers and setting up editor area
        let mut line_numbers = LineNumbers::new(&self.editor.text, &self.cursor);

        let (line_numbs, editor_area, status_bar) =
            layout_chunks(content_area, line_numbers.col_get_length());
        line_numbers.render_ui(line_numbs, f);

        let is_start_row = match self.cursor.pos.x.cmp(&0_usize) {
            std::cmp::Ordering::Less => false,
            std::cmp::Ordering::Equal => true,
            std::cmp::Ordering::Greater => false,
        };

        let is_emphty = match self.editor.text.len_chars().cmp(&0_usize) {
            std::cmp::Ordering::Less => false,
            std::cmp::Ordering::Equal => true,
            std::cmp::Ordering::Greater => false,
        };

        let cursor_char = self.cursor_get_char();
        let cursor_char = match cursor_char {
            Some(c) => &c.to_string(),
            None => "None",
        };

        // snippet logs -----------------------

        let snippet_str = toml::to_string_pretty(&snippet).unwrap();
        let lines: Vec<ListItem> = snippet_str
            .lines()
            .map(|line| ListItem::new(line.to_string()))
            .collect();
        let list = List::new(lines.clone()).block(
            Block::default()
                .title("Snippet Entity")
                .borders(Borders::ALL),
        );
        let vertical = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(1),         // Text + line numbers
                Constraint::Percentage(50), // Status bar
            ])
            .split(status_bar);

        let mut state = ListState::default();
        state.select(Some(0));

        f.render_stateful_widget(list, vertical[0], &mut state);
        let system_msg: String = match snippet.notifications {
            Some(ref val) => val.clone(),
            None => "None".to_string(),
        };
        let paragraph = Paragraph::new(format!(
            r#"
            row: {} + 1, 
            col: {} + 1, 
            idx: {},
            mode: {},
            is_start_row: {},
            is_empty: {},
            current_char: {},
            system_message: {}
            "#,
            self.cursor.pos.x,
            self.cursor.pos.y,
            self.cursor_get_idx(),
            self.mode,
            is_start_row,
            is_emphty,
            cursor_char,
            system_msg
        ))
        .block(Block::bordered().title("Editor State"));
        f.render_widget(paragraph, vertical[1]);

        let content = self.editor.text.to_string();
        let data = lex_and_style(&content);

        let editor_widget = Paragraph::new(data);
        f.render_widget(editor_widget, editor_area);

        let cursor_screen_row = self.cursor.pos.y.saturating_sub(self.cursor.offset_y) as u16;
        let x = editor_area.x + self.cursor.pos.x as u16;
        let y = editor_area.y + cursor_screen_row;

        if y < editor_area.y + editor_area.height {
            let max_x = editor_area.x + editor_area.width.saturating_sub(1);
            let set_x = if x > max_x { max_x } else { x };
            f.set_cursor_position((set_x, y));
        };
    }
}

impl Document {
    // helpers around some common operations
    pub fn cursor_get_idx(&self) -> usize {
        // returs index where the curosr is located
        self.editor.pos_to_idx(&self.cursor.pos)
    }
    pub fn cursor_get_loc(&self, row_idx: usize) -> Pos {
        self.editor.idx_to_pos(row_idx)
    }
    pub fn cursor_set_pos(&mut self, pos: Pos) {
        self.cursor.pos = pos;
    }
    pub fn cursor_pos_insert(&mut self, text: &str) {
        let mut idx = self.cursor_get_idx();
        // TODO find better solution later
        if idx > self.editor.text.len_chars() {
            idx = self.editor.text.len_chars()
        }
        self.editor.text.insert(idx, text);
    }
    pub fn offset_insert(&mut self, text: &str, offset: usize) {
        let mut idx = self.cursor_get_idx();
        // TODO find better solution later
        if idx > self.editor.text.len_chars() {
            idx = self.editor.text.len_chars()
        }
        self.editor.text.insert(idx, text);
    }
    pub fn cursor_idx_insert(&mut self, idx: usize, text: &str) {
        self.editor.text.insert(idx, text);
    }
    pub fn cursor_get_char(&self) -> Option<char> {
        let idx = self.cursor_get_idx();
        self.editor.text.get_char(idx)
    }
}

pub struct Editor {
    pub text: Rope,
}
impl Editor {
    pub fn row_get_count(&self) -> usize {
        self.text.len_lines() // lines is considered to be a line break
    }
    pub fn row_get_length(&self, row_idx: usize) -> usize {
        self.text.line(row_idx).len_chars()
    }
    pub fn pos_to_idx(&self, pos: &Pos) -> usize {
        // singe number that represents where we are it text
        let cursor_col = pos.y;
        let row_start = self.text.line_to_char(cursor_col);
        row_start + pos.x
    }
    pub fn idx_to_pos(&self, row_idx: usize) -> Pos {
        let idx = if row_idx > self.text.len_chars() {
            self.text.len_chars() // go to tha last characetr if cursor is somewhere else
        } else {
            row_idx
        };
        let row = self.text.char_to_line(idx);
        let col = idx - self.text.line_to_char(row);
        Pos { y: row, x: col }
    }
}

impl Editor {
    pub fn is_start_row(&self, idx: usize) -> bool {
        idx == 0 || self.text.char(idx - 1) == '\n'
    }
}

fn layout_chunks(area: Rect, line_numbers: u16) -> (Rect, Rect, Rect) {
    // Split vertically: main area and status bar
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),         // Text + line numbers
            Constraint::Percentage(70), // Status bar
        ])
        .split(area);

    // Split horizontally: line numbers and text
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(line_numbers), // Line numbers
            Constraint::Min(0),               // Text area
        ])
        .split(vertical[0]);

    let line_numbers = horizontal[0];
    let text_area = horizontal[1];
    let status_bar = vertical[1];

    (line_numbers, text_area, status_bar)
}
