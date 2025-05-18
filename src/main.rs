


use dreg::*;



fn main() -> Result<(), Box<dyn std::error::Error>> {
    TerminalPlatform::new().run(App {
        shutdown: false,
    })
}



struct App {
    shutdown: bool,
}

impl Program for App {
    fn render(&mut self, frame: &mut Frame) {
        if self.shutdown {
            frame.should_exit = true;
            return;
        }

        Rectangle {
            area: frame.area(),
            fg: Color::from_rgb(89, 89, 109),
            style: RectangleStyle::Round,
        }.render(frame);

        let text_area = frame.area().inner_centered(13, 1);
        Text::new("Hello, World!")
            .with_position(text_area.x, text_area.y)
            .render(frame);
    }

    fn input(&mut self, input: Input) {
        match input {
            Input::KeyDown(Scancode::Q) => {
                self.shutdown = true;
            }
            _ => {}
        }
    }
}



struct Buffer {
    lines: Vec<Line>,
    cursor: Cursor,
    selection: Selection,
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
