use ropey::Rope;

// cursor location
#[derive(Default)]
pub struct Pos {
    pub y: usize,
    pub x: usize,
}

#[derive(Default)]
pub struct Cursor {
    pub pos: Pos,
    pub offset_y: usize, // first visible row in viewport (vertical scroll)
}

pub enum CursorDirection {
    Left,
    Right,
    Down,
    Up,
}

impl Cursor {
    pub fn move_cursor(&mut self, text: &Rope, direction: CursorDirection) {
        let loc = &mut self.pos;
        match direction {
            CursorDirection::Right => {
                let line_len = text.line(loc.y).len_chars();
                if loc.x < line_len {
                    loc.x += 1;
                } else if loc.y + 1 < text.len_lines() {
                    // move to start of next line
                    loc.y += 1;
                    loc.x = 0;
                }
            }
            CursorDirection::Left => {
                if loc.x > 0 {
                    loc.x -= 1;
                } else if loc.y > 0 {
                    // move to end of previous line
                    loc.y -= 1;
                    let prev_len = text.line(loc.y).len_chars();
                    loc.x = prev_len;
                }
            }
            CursorDirection::Down => {
                if loc.y + 1 < text.len_lines() {
                    loc.y += 1;
                    // clamp col to new line length
                    let new_len = text.line(loc.y).len_chars();
                    if loc.x > new_len {
                        loc.x = new_len;
                    }
                }
            }
            CursorDirection::Up => {
                if loc.y > 0 {
                    loc.y -= 1;
                    let new_len = text.line(loc.y).len_chars();
                    if loc.x > new_len {
                        loc.x = new_len;
                    }
                }
            }
        }
    }
}
