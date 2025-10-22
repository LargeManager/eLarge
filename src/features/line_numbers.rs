use crate::cursor::Cursor;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::Paragraph,
};
use ropey::Rope;

pub struct LineNumbers<'a> {
    text: &'a Rope,
    cursor: &'a Cursor,
    style: Style,
    col_length: u16,
}

impl<'a> LineNumbers<'a> {
    pub fn new(text: &'a Rope, cursor: &'a Cursor) -> Self {
        Self {
            style: Style::default().fg(Color::Red),
            col_length: 0,
            text,
            cursor,
        }
    }

    pub fn col_get_length(&mut self) -> u16 {
        let row_count: usize = self.text.len_lines();
        let col_length: u16 = row_count.to_string().len() as u16;
        let col_length_extra = col_length + 2; // digits + padding

        self.col_length = col_length_extra;
        col_length_extra
    }

    pub fn render_ui(&self, render_space: Rect, f: &mut Frame) {
        // Determine visible range based on offset_row and editor height
        let start_row = self.cursor.offset_y;
        let end_row = self.text.len_lines();

        let line_numbers: Vec<Line> = (start_row..end_row)
            .map(|i| {
                let number_text =
                    format!("{:>width$} ", i + 1, width = (self.col_length - 2) as usize);

                // actual styling
                let mut line = Line::from(number_text).style(self.style);

                // highlight number where the cursor have landed
                if i == self.cursor.pos.y {
                    line = line.style(self.style.fg(Color::LightCyan).add_modifier(Modifier::BOLD));
                }
                line
            })
            .collect();

        let text = Paragraph::new(line_numbers);
        f.render_widget(text, render_space);
    }
}
