


use dreg::*;
use unicode_segmentation::UnicodeSegmentation as _;



fn main() -> Result<(), Box<dyn std::error::Error>> {
    Terminal::new().run(App {
        shutdown: false,
        initialized: false,
        buffer: Buffer::new(include_str!("main.rs")),
    })
}



struct App {
    shutdown: bool,
    initialized: bool,
    buffer: Buffer,
}

impl Program for App {
    fn render(&mut self, frame: &mut Frame) {
        if self.shutdown {
            frame.should_exit = true;
            return;
        }
        if !self.initialized {
            frame.commands.push(Command::SetCursorStyle(CursorStyle::BlinkingBar));
        }

        let (_side_area, buffer_area) = frame.area().hsplit_portion(0.2);
        let (gutter_area, buffer_area) = buffer_area.hsplit_len(5);

        self.buffer.area = buffer_area;

        let max_gutter_width = gutter_area.w as usize;
        let max_line_width = buffer_area.w as usize;

        for (index, ((area, gutter), line)) in buffer_area.rows().into_iter()
            .zip(gutter_area.rows().into_iter())
            .zip(self.buffer.lines.iter().take(buffer_area.h as usize))
            .enumerate()
        {
            frame.buffer.set_stringn(
                gutter.x,
                gutter.y,
                &format!("{}", index + 1),
                max_gutter_width,
                Style::default().dim(),
            );
            frame.buffer.set_stringn(
                area.x,
                area.y,
                &line.content,
                max_line_width,
                Style::default(),
            );
        }

        frame.cursor = Some((
            buffer_area.x + self.buffer.cursor.index as u16,
            buffer_area.y + self.buffer.cursor.line as u16,
        ));
    }

    fn input(&mut self, input: Input) {
        match input {
            Input::KeyDown(Scancode::Q) => {
                self.shutdown = true;
            }
            Input::KeyDown(Scancode::LEFT) => {
                self.buffer.perform_action(EditAction::MoveLeft);
            }
            Input::KeyDown(Scancode::RIGHT) => {
                self.buffer.perform_action(EditAction::MoveRight);
            }
            Input::KeyDown(Scancode::UP) => {
                self.buffer.perform_action(EditAction::MoveUp);
            }
            Input::KeyDown(Scancode::DOWN) => {
                self.buffer.perform_action(EditAction::MoveDown);
            }
            Input::KeyDown(Scancode::L_BRACE) => {
                self.buffer.perform_action(EditAction::MovePrevWord);
            }
            Input::KeyDown(Scancode::R_BRACE) => {
                self.buffer.perform_action(EditAction::MoveNextWord);
            }
            _ => {}
        }
    }
}



struct Buffer {
    lines: Vec<Line>,
    cursor: Cursor,
    selection: Selection,
    area: Area,
}

impl Buffer {
    pub fn new(content: &str) -> Self {
        let lines = content.lines()
            .map(|s| Line { content: s.to_string() })
            .collect();

        Self {
            lines,
            cursor: Cursor { line: 0, index: 0 },
            selection: Selection::None,
            area: Area::ZERO, // Set by the render function in `App`.
        }
    }
}

impl Buffer {
    pub fn insert_at(&mut self, mut cursor: Cursor, content: &str) -> Cursor {
        let mut remaining_split_len = content.len();
        if remaining_split_len == 0 {
            return cursor;
        }

        // TODO: Ensure that the line exists.
        let line: &mut Line = &mut self.lines[cursor.line];
        let insert_line = cursor.line + 1;

        let after: Line = line.split_off(cursor.index);
        let after_len = after.content.len();

        // Append the inserted text, line by line
        // we want to see a blank entry if the string ends with a newline
        //TODO: adjust this to get line ending from data?
        let addendum = std::iter::once("").filter(|_| content.ends_with('\n'));
        let mut lines_iter = content.split_inclusive('\n').chain(addendum);
        if let Some(content_line) = lines_iter.next() {
            remaining_split_len -= content_line.len();
            line.append(Line {
                content: content_line.to_string(),
            });
        } else {
            panic!("str::lines() did not yield any elements");
        }
        if let Some(content_line) = lines_iter.next_back() {
            remaining_split_len -= content_line.len();
            let mut tmp = Line {
                content: content_line
                    .strip_suffix(char::is_control)
                    .unwrap_or(content_line)
                    .to_string(),
            };
            tmp.append(after);
            self.lines.insert(insert_line, tmp);
            cursor.line += 1;
        } else {
            line.append(after);
        }
        for content_line in lines_iter.rev() {
            remaining_split_len -= content_line.len();
            let tmp = Line {
                content: content_line
                    .strip_suffix(char::is_control)
                    .unwrap_or(content_line)
                    .to_string(),
            };
            self.lines.insert(insert_line, tmp);
            cursor.line += 1;
        }

        assert_eq!(remaining_split_len, 0);

        cursor.index = self.lines[cursor.line].content.len() - after_len;

        cursor
    }

    pub fn insert_string(&mut self, content: &str) {
        self.delete_selection();
        let next_cursor = self.insert_at(self.cursor, content);
        self.cursor = next_cursor;
    }

    pub fn selection_bounds(&self) -> Option<(Cursor, Cursor)> {
        match self.selection {
            Selection::None => None,
            Selection::Normal(select) => {
                match select.line.cmp(&self.cursor.line) {
                    std::cmp::Ordering::Greater => Some((self.cursor, select)),
                    std::cmp::Ordering::Less => Some((select, self.cursor)),
                    std::cmp::Ordering::Equal => {
                        /* select.line == cursor.line */
                        if select.index < self.cursor.index {
                            Some((select, self.cursor))
                        } else {
                            /* select.index >= cursor.index */
                            Some((self.cursor, select))
                        }
                    }
                }
            }
            Selection::Line(select) => {
                let start_line = std::cmp::min(select.line, self.cursor.line);
                let end_line = std::cmp::max(select.line, self.cursor.line);
                let end_index = self.lines[end_line].content.len();
                Some((
                    Cursor { line: start_line, index: 0 },
                    Cursor { line: end_line, index: end_index },
                ))
            }
            Selection::Word(_) => todo!(),
        }
    }

    pub fn delete_selection(&mut self) -> bool {
        let (start, end) = match self.selection_bounds() {
            Some(some) => some,
            None => return false,
        };

        self.cursor = start;
        self.selection = Selection::None;

        self.delete_range(start, end);

        true
    }

    pub fn delete_range(&mut self, start: Cursor, end: Cursor) {
        // Delete from the last line.
        let end_line_opt = if end.line > start.line {
            let after = self.lines[end.line].split_off(end.index);
            let _removed = self.lines.remove(end.line);

            Some(after)
        } else {
            None
        };

        for line_i in (start.line + 1..end.line).rev() {
            let _removed = self.lines.remove(line_i);
        }

        // Delete from the first line.
        {
            // Get part after selection if start line is also end line
            let after_opt = if start.line == end.line {
                Some(self.lines[start.line].split_off(end.index))
            } else {
                None
            };

            let _removed = self.lines[start.line].split_off(start.index);

            // Re-add part of line after the range.
            if let Some(after) = after_opt {
                self.lines[start.line].append(after);
            }
            // Re-add valid parts of end line
            if let Some(end_line) = end_line_opt {
                self.lines[start.line].append(end_line);
            }
        }
    }

    pub fn perform_action(&mut self, action: EditAction) {
        match action {
            EditAction::ClearSelection => {
                self.selection = Selection::None;
            }
            EditAction::DeleteSelection => {
                self.delete_selection();
            }
            EditAction::NewLine => {
                self.insert_string("\n");
            }
            EditAction::MoveLeft => {
                let line = self.lines.get(self.cursor.line).unwrap();
                if self.cursor.index > 0 {
                    let mut prev_index = 0;
                    for (i, _c) in line.content.chars().enumerate() {
                        if i < self.cursor.index {
                            prev_index = i;
                        } else {
                            break;
                        }
                    }

                    self.cursor.index = prev_index;
                } else if self.cursor.line > 0 {
                    self.cursor.line -= 1;
                    self.cursor.index = self.lines.get(self.cursor.line).unwrap().content.len();
                }
            }
            EditAction::MoveRight => {
                let line = self.lines.get(self.cursor.line).unwrap();
                if self.cursor.index < line.content.len() {
                    for (i, _c) in line.content.chars().enumerate() {
                        if i == self.cursor.index {
                            self.cursor.index += 1; // c.len()
                            break;
                        }
                    }
                } else if self.cursor.line + 1 < self.lines.len() {
                    self.cursor.line += 1;
                    self.cursor.index = 0;
                }
            }
            EditAction::MoveUp => {
                let line = self.lines.get(self.cursor.line).unwrap();
                if line.content.len() > self.area.w as usize {
                    // FIXME: This is likely a horribly inefficient way to do this.

                    let row_count = (line.content.len() as f32 / self.area.w as f32)
                        .ceil() as usize;
                    let mut row_index = 0;
                    for i in 0..row_count {
                        if (i * self.area.w as usize) < self.cursor.index {
                            row_index = i;
                        } else {
                            break;
                        }
                    }
                    if row_index > 0 {
                        self.cursor.index = self.area.w as usize * (row_index - 1);
                    } else {
                        if self.cursor.line > 0 {
                            self.cursor.line -= 1;
                            let line = self.lines.get(self.cursor.line).unwrap();
                            if line.content.len() < self.cursor.index {
                                self.cursor.index = line.content.len();
                            }
                        }
                    }
                } else {
                    if self.cursor.line > 0 {
                        self.cursor.line -= 1;
                        let line = self.lines.get(self.cursor.line).unwrap();
                        if line.content.len() < self.cursor.index {
                            self.cursor.index = line.content.len();
                        }
                    }
                }
            }
            EditAction::MoveDown => {
                let line = self.lines.get(self.cursor.line).unwrap();
                if line.content.len() > self.area.w as usize {
                    let row_count = (line.content.len() as f32 / self.area.w as f32)
                        .ceil() as usize;
                    let mut row_index = 0;
                    for i in 0..row_count {
                        if (i * self.area.w as usize) < self.cursor.index {
                            row_index = i;
                        } else {
                            break;
                        }
                    }
                    if row_index > 0 {
                        self.cursor.index = self.area.w as usize * (row_index + 1);
                    } else {
                        if self.cursor.line > 0 {
                            self.cursor.line += 1;
                            let line = self.lines.get(self.cursor.line).unwrap();
                            if line.content.len() < self.cursor.index {
                                self.cursor.index = line.content.len();
                            }
                        }
                    }
                } else {
                    if self.cursor.line + 1 < self.lines.len() {
                        self.cursor.line += 1;
                        let line = self.lines.get(self.cursor.line).unwrap();
                        if line.content.len() < self.cursor.index {
                            self.cursor.index = line.content.len();
                        }
                    }
                }
            }
            EditAction::MovePrevWord => {
                let line = self.lines.get(self.cursor.line).unwrap();
                if self.cursor.index > 0 {
                    self.cursor.index = line
                        .content
                        .unicode_word_indices()
                        .rev()
                        .map(|(i, _)| i)
                        .find(|&i| i < self.cursor.index)
                        .unwrap_or(0);
                } else if self.cursor.line > 0 {
                    self.cursor.line -= 1;
                    self.cursor.index = self.lines.get(self.cursor.line).unwrap().content.len();
                }
            }
            EditAction::MoveNextWord => {
                let line = self.lines.get(self.cursor.line).unwrap();
                if self.cursor.index < line.content.len() {
                    self.cursor.index = line
                        .content
                        .unicode_word_indices()
                        .map(|(i, word)| i + word.len())
                        .find(|&i| i > self.cursor.index)
                        .unwrap_or(line.content.len());
                } else if self.cursor.line + 1 < self.lines.len() {
                    self.cursor.line += 1;
                    self.cursor.index = 0;
                }
            }
        }
    }
}

pub struct Line {
    content: String,
}

impl Line {
    pub fn append(&mut self, other: Self) {
        self.content.push_str(&other.content);
    }

    pub fn split_off(&mut self, index: usize) -> Self {
        let text = self.content.split_off(index);

        Self {
            content: text,
        }
    }
}

pub enum EditAction {
    ClearSelection,
    DeleteSelection,
    NewLine,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MovePrevWord,
    MoveNextWord,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Cursor {
    pub line: usize,
    pub index: usize,
}

pub enum Selection {
    None,
    Normal(Cursor),
    Line(Cursor),
    Word(Cursor),
}
