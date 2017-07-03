pub mod line;

use std::cmp::{max, min};
use std::mem;
use std::str;

use termbox_simple::Termbox;

pub use self::line::Line;
pub use self::line::SegStyle;

pub struct MsgArea {
    lines       : Vec<Line>,

    // Rendering related
    width       : i32,
    height      : i32,

    /// Vertical scroll: An offset from the last visible line.
    /// E.g. when this is 0, `self.lines[self.lines.len() - 1]` is drawn at the
    /// bottom of screen.
    scroll      : i32,

    line_buf    : Line,
}

impl MsgArea {
    pub fn new(width : i32, height : i32) -> MsgArea {
        MsgArea {
            lines: Vec::new(),
            width: width,
            height: height,
            scroll: 0,
            line_buf: Line::new(),
        }
    }

    pub fn resize(&mut self, width : i32, height : i32) {
        self.width = width;
        self.height = height;
    }

    pub fn draw(&self, tb : &mut Termbox, pos_x : i32, pos_y : i32) {
        let mut row = pos_y + self.height - 1;

        // Draw lines in reverse order
        let mut line_idx = ((self.lines.len() as i32) - 1) - self.scroll;
        while line_idx >= 0 && row >= pos_y {
            let line = unsafe { self.lines.get_unchecked(line_idx as usize) };

            // Where to start rendering this line?
            let line_height = line.rendered_height(self.width);
            let line_row = row - line_height + 1;

            // Do we have enough space to render this line?
            if line_row >= pos_y {
                // Render it
                line.draw(tb, pos_x, line_row, self.width);
                row = line_row - 1;
                line_idx -= 1;
            } else {
                // Maybe we can still render some part of it
                let render_from = pos_y - line_row;
                line.draw_from(tb, pos_x, line_row, render_from, self.width);
                break;
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Scrolling

impl MsgArea {
    pub fn scroll_up(&mut self) {
        if self.scroll < (self.lines.len() as i32) - 1 {
            self.scroll += 1;
        }
    }

    pub fn scroll_down(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
        }
    }

    pub fn scroll_top(&mut self) {
        if self.lines.is_empty() { return; }

        let mut height_left = self.height;
        let mut i = 0;
        while i < self.lines.len() {
            let rendered = self.lines[i].rendered_height(self.width);
            if height_left < rendered {
                break;
            }
            height_left -= rendered;
            i += 1;
        }

        self.scroll = (self.lines.len() - i) as i32;
    }

    pub fn scroll_bottom(&mut self) {
        self.scroll = 0;
    }

    pub fn page_up(&mut self) {
        self.scroll = max(0, min((self.lines.len() as i32) - 1, self.scroll + 10));
    }

    pub fn page_down(&mut self) {
        self.scroll = max(0, self.scroll - 10);
    }
}

////////////////////////////////////////////////////////////////////////////////
// Adding text

impl MsgArea {
    pub fn set_style(&mut self, style: SegStyle) {
        self.line_buf.set_style(style);
    }

    pub fn add_text(&mut self, str : &str) {
        self.line_buf.add_text(str);
    }

    pub fn add_char(&mut self, char : char) {
        self.line_buf.add_char(char);
    }

    pub fn flush_line(&mut self) -> usize {
        self.lines.push(mem::replace(&mut self.line_buf, Line::new()));
        if self.scroll != 0 {
            self.scroll += 1;
        }
        self.lines.len() - 1
    }

    #[inline]
    pub fn modify_line<F>(&mut self, idx : usize, f : F) where F : Fn(&mut Line) {
        f(&mut self.lines[idx]);
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn newline_scrolling() {
        let mut msg_area = MsgArea::new(100, 1);
        // Adding a new line when scroll is 0 should not change it
        assert_eq!(msg_area.scroll, 0);
        msg_area.add_text("line1");
        msg_area.flush_line();
        assert_eq!(msg_area.scroll, 0);

        msg_area.add_text("line2");
        msg_area.flush_line();
        assert_eq!(msg_area.scroll, 0);

        msg_area.scroll_up();
        assert_eq!(msg_area.scroll, 1);
        msg_area.add_text("line3");
        msg_area.flush_line();
        assert_eq!(msg_area.scroll, 2);
    }
}
