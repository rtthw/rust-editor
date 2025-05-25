


mod workspace;

use dreg::*;
use unicode_segmentation::UnicodeSegmentation as _;

use workspace::*;



fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workspace_info = find_workspace();
    println!("WORKSPACE_DIR: {}", workspace_info.path.display());
    println!("HAS_VERSION_CONTROL: {}", workspace_info.has_vc);

    Terminal::new().run(App {
        shutdown: false,
        initialized: false,
        workspace_info,
        buffers: BufferSet::new("src/main.rs", include_str!("main.rs")),
        input_context: InputContext::default(),
    })
}



struct App {
    shutdown: bool,
    initialized: bool,

    workspace_info: WorkspaceInfo,
    buffers: BufferSet,
    input_context: InputContext,
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

        let buffer = self.buffers.current_buffer_mut();

        let (side_area, buffer_area) = frame.area().hsplit_portion(0.2);

        frame.buffer.set_stringn(
            side_area.x,
            side_area.y,
            self.workspace_info.path.file_name().and_then(|os_str| os_str.to_str()).unwrap(),
            side_area.w as _,
            Style::default().dim(),
        );
        frame.buffer.set_stringn(
            side_area.x,
            side_area.y + 1,
            &buffer.name,
            side_area.w as _,
            Style::default(),
        );

        let (gutter_area, buffer_area) = buffer_area.hsplit_len(5);

        buffer.area = buffer_area;

        let max_gutter_width = gutter_area.w as usize;
        let max_line_width = buffer_area.w as usize;

        let mut cursor_row = 0;
        let mut last_line_index = 1;
        let selection = buffer.selection_bounds();
        for (index, ((area, gutter), row)) in buffer_area.rows().into_iter()
            .zip(gutter_area.rows().into_iter())
            .zip(buffer.visible_rows())
            .enumerate()
        {
            if row.line_index != last_line_index {
                frame.buffer.set_stringn(
                    gutter.x,
                    gutter.y,
                    &format!("{}", row.line_index + 1),
                    max_gutter_width,
                    Style::default().dim(),
                );
            }
            frame.buffer.set_stringn(
                area.x,
                area.y,
                row.content,
                max_line_width,
                Style::default(),
            );
            last_line_index = row.line_index;
            if row.line_index == buffer.cursor.line {
                if (row.index * buffer.area.w as usize) <= buffer.cursor.index {
                    cursor_row = index;
                }
            }

            // Highlight selection.
            if let Some((start_cursor, end_cursor)) = &selection {
                if row.line_index >= start_cursor.line && row.line_index <= end_cursor.line {
                    let start_index = start_cursor.index as u16 % buffer.area.w;
                    let end_index = end_cursor.index as u16 % buffer.area.w;

                    if start_cursor.line == end_cursor.line {
                        // Highlight from the start index to the end index.
                        for i in start_index..end_index {
                            frame.buffer.get_mut(area.x + i, area.y)
                                .bg = Color::from_rgb(0x59, 0x59, 0x6d);
                        }
                    } else if row.line_index == start_cursor.line {
                        // Highlight from the start index to the end of the row.
                        for i in start_index..(row.content.len() as u16) {
                            frame.buffer.get_mut(area.x + i, area.y)
                                .bg = Color::from_rgb(0x59, 0x59, 0x6d);
                        }
                    } else if row.line_index > start_cursor.line
                        && row.line_index < end_cursor.line
                    {
                        // Highlight the whole line.
                        for i in 0..row.content.len() {
                            frame.buffer.get_mut(area.x + i as u16, area.y)
                                .bg = Color::from_rgb(0x59, 0x59, 0x6d);
                        }
                    } else if row.line_index == end_cursor.line {
                        // Highlight from the start of the row to the end index.
                        for i in 0..end_index {
                            frame.buffer.get_mut(area.x + i, area.y)
                                .bg = Color::from_rgb(0x59, 0x59, 0x6d);
                        }
                    }
                }
            }
        }

        frame.cursor = Some((
            buffer_area.x + (buffer.cursor.index as u16 % buffer.area.w),
            buffer_area.y + cursor_row as u16,
        ));

        self.input_context.end_frame();
    }

    fn input(&mut self, input: Input) {
        self.input_context.handle_input(input);

        match input {
            Input::KeyDown(Scancode::LEFT) => {
                if self.input_context.is_key_down(&Scancode::L_CTRL) {
                    self.buffers.current_buffer_mut().perform_action(EditAction::MovePrevWord);
                } else {
                    self.buffers.current_buffer_mut().perform_action(EditAction::MoveLeft);
                }
            }
            Input::KeyDown(Scancode::RIGHT) => {
                if self.input_context.is_key_down(&Scancode::L_CTRL) {
                    self.buffers.current_buffer_mut().perform_action(EditAction::MoveNextWord);
                } else {
                    self.buffers.current_buffer_mut().perform_action(EditAction::MoveRight);
                }
            }
            Input::KeyDown(Scancode::UP) => {
                self.buffers.current_buffer_mut().perform_action(EditAction::MoveUp);
            }
            Input::KeyDown(Scancode::DOWN) => {
                if self.input_context.is_key_down(&Scancode::L_SHIFT) {
                    self.buffers.current_buffer_mut().start_selection();
                }
                self.buffers.current_buffer_mut().perform_action(EditAction::MoveDown);
            }

            Input::KeyDown(Scancode::BACKSPACE) => {
                self.buffers.current_buffer_mut().perform_action(EditAction::Backspace);
            }
            Input::KeyDown(Scancode::DELETE) => {
                self.buffers.current_buffer_mut().perform_action(EditAction::Delete);
            }

            Input::KeyDown(Scancode::SPACE) => {
                self.buffers.current_buffer_mut().perform_action(EditAction::Insert(' '));
            }
            // TODO: Indentation.
            Input::KeyDown(Scancode::TAB) => {
                self.buffers.current_buffer_mut().perform_action(EditAction::Insert('\t'));
            }
            Input::KeyDown(Scancode::ENTER) => {
                self.buffers.current_buffer_mut().perform_action(EditAction::NewLine);
            }

            Input::MouseDown(MouseButton::Left) => {
                if let Some((mouse_x, mouse_y)) = self.input_context.mouse_pos() {
                    let buffer = self.buffers.current_buffer_mut();
                    buffer.perform_action(EditAction::Click((
                        mouse_x.saturating_sub(buffer.area.x),
                        mouse_y.saturating_sub(buffer.area.y),
                    )));
                }
            }
            Input::WheelUp => {
                self.buffers.current_buffer_mut().perform_action(EditAction::ScrollUp);
            }
            Input::WheelDown => {
                self.buffers.current_buffer_mut().perform_action(EditAction::ScrollDown);
            }

            Input::KeyDown(other) => {
                if let Some(ch) = util::scancode_to_char(other) {
                    if self.input_context.is_key_down(&Scancode::L_CTRL) {
                        match ch {
                            'q' => {
                                self.shutdown = true;
                            }
                            '[' => {
                                self.buffers.goto_previous(true);
                            }
                            ']' => {
                                self.buffers.goto_next(true);
                            }
                            _ => {}
                        }
                    } else if self.input_context.is_key_down(&Scancode::L_SHIFT) {
                        let ch = util::shifted_char(ch);
                        self.buffers.current_buffer_mut().perform_action(EditAction::Insert(ch));
                    } else {
                        self.buffers.current_buffer_mut().perform_action(EditAction::Insert(ch));
                    }
                }
            }

            _ => {}
        }
    }
}



pub struct BufferSet {
    buffers: Vec<Buffer>,
    current: usize,
}

impl BufferSet {
    pub fn new(initial_buffer_name: &str, initial_buffer_content: &str) -> Self {
        let scratch_buffer = Buffer::new("SCRATCH", "");
        let current_buffer = Buffer::new(initial_buffer_name, initial_buffer_content);

        Self {
            buffers: vec![scratch_buffer, current_buffer],
            current: 0,
        }
    }
}

impl BufferSet {
    pub fn current_buffer(&self) -> &Buffer {
        assert!(self.count() > 0); // Cannot close the only buffer.
        &self.buffers[self.current]
    }

    pub fn current_buffer_mut(&mut self) -> &mut Buffer {
        assert!(self.count() > 0); // Cannot close the only buffer.
        &mut self.buffers[self.current]
    }

    #[inline]
    pub fn count(&self) -> usize {
        self.buffers.len()
    }

    #[inline]
    pub fn current_is_first(&self) -> bool {
        self.current == 0
    }

    #[inline]
    pub fn current_is_last(&self) -> bool {
        self.count() == self.current + 1
    }
}

impl BufferSet {
    pub fn goto_next(&mut self, wrap_at_end: bool) -> bool {
        if self.count() == 1 {
            false
        } else if self.current_is_last() {
            if !wrap_at_end {
                false
            } else {
                self.current = 0;
                true
            }
        } else {
            self.current += 1;
            true
        }
    }

    pub fn goto_previous(&mut self, wrap_at_start: bool) -> bool {
        if self.count() == 1 {
            false
        } else if self.current_is_first() {
            if !wrap_at_start {
                false
            } else {
                self.current = self.count() - 1;
                true
            }
        } else {
            self.current -= 1;
            true
        }
    }
}



pub struct Buffer {
    name: String,
    lines: Vec<Line>,
    cursor: Cursor,
    selection: Selection,
    area: Area,
    scroll_y_offset: u16,
}

impl Buffer {
    pub fn new(name: impl Into<String>, content: &str) -> Self {
        let mut lines: Vec<Line> = content.lines()
            .map(|s| Line { content: s.to_string() })
            .collect();
        if lines.len() < 1 {
            lines.push(Line { content: "".to_string() });
        }

        Self {
            name: name.into(),
            lines,
            cursor: Cursor { line: 0, index: 0 },
            selection: Selection::None,
            area: Area::ZERO, // Set by the render function in `App`.
            scroll_y_offset: 0,
        }
    }

    pub fn rows(&self) -> impl DoubleEndedIterator<Item = Row> {
        let mut num = 0;
        self.lines.iter()
            .enumerate()
            .flat_map(move |(line_index, line)| {
                if line.content.len() > self.area.w as usize {
                    let mut rows = Vec::with_capacity(3);
                    let (first, mut last) = line.content.split_at(self.area.w as usize);
                    num += 1;
                    rows.push(Row {
                        num,
                        index: 0,
                        line_index,
                        content: first,
                    });
                    let mut index = 1;
                    while last.len() > self.area.w as usize {
                        let (next, then) = line.content.split_at(self.area.w as usize);
                        last = then;
                        num += 1;
                        rows.push(Row {
                            num,
                            index,
                            line_index,
                            content: next,
                        });
                        index += 1;
                    }
                    num += 1;
                    rows.push(Row {
                        num,
                        index,
                        line_index,
                        content: last,
                    });
                    rows.into_iter()
                } else {
                    num += 1;
                    // std::iter::once(line.content.as_str())
                    vec![
                        Row {
                            num,
                            index: 0,
                            line_index,
                            content: line.content.as_str(),
                        }
                    ].into_iter()
                }
            })
    }

    pub fn visible_rows(&self) -> impl Iterator<Item = Row> {
        self.rows()
            .skip(self.scroll_y_offset as usize)
            .take(self.area.h as usize)
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

    pub fn start_selection(&mut self) {
        self.selection = Selection::Normal(self.cursor);
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
            EditAction::Insert(ch) => {
                if ch == '\n' {
                    self.perform_action(EditAction::NewLine);
                } else {
                    let mut str_buf = [0u8; 8];
                    let str_ref = ch.encode_utf8(&mut str_buf);
                    self.insert_string(str_ref);
                }
            }
            EditAction::ClearSelection => {
                self.selection = Selection::None;
            }
            EditAction::DeleteSelection => {
                self.delete_selection();
            }
            EditAction::NewLine => {
                self.insert_string("\n");
            }
            EditAction::Backspace => {
                if self.delete_selection() {
                    // Deleted selection.
                } else {
                    let end = self.cursor;

                    if self.cursor.index > 0 {
                        // Move cursor to previous character index.
                        self.cursor.index = {
                            self.lines[self.cursor.line].content[..self.cursor.index]
                                .char_indices()
                                .next_back()
                                .map_or(0, |(i, _)| i)
                        };
                    } else if self.cursor.line > 0 {
                        // Move cursor to previous line.
                        self.cursor.line -= 1;
                        self.cursor.index = self.lines[self.cursor.line].content.len();
                    }

                    if self.cursor != end {
                        self.delete_range(self.cursor, end);
                    }
                }
            }
            EditAction::Delete => {
                if self.delete_selection() {
                    // Deleted selection.
                } else {
                    let mut start = self.cursor;
                    let mut end = self.cursor;

                    if start.index < self.lines[start.line].content.len() {
                        let line = &self.lines[start.line];

                        let range_opt = line
                            .content
                            .grapheme_indices(true)
                            .take_while(|(i, _)| *i <= start.index)
                            .last()
                            .map(|(i, c)| i..(i + c.len()));

                        if let Some(range) = range_opt {
                            start.index = range.start;
                            end.index = range.end;
                        }
                    } else if start.line + 1 < self.lines.len() {
                        end.line += 1;
                        end.index = 0;
                    }

                    if start != end {
                        self.cursor = start;
                        self.delete_range(start, end);
                    }
                }
            }
            EditAction::Click((x, y)) => {
                let mut new_line_index = self.cursor.line;
                let mut new_index = self.cursor.index;

                if let Some((_, row)) = self.visible_rows()
                    .enumerate()
                    .find(|(i, _)| *i == y as usize)
                {
                    new_line_index = row.line_index;
                    let addendum = if x as usize > row.content.len() {
                        row.content.len()
                    } else {
                        x as usize
                    };
                    new_index = (row.index * self.area.w as usize) + addendum;
                }

                self.cursor.line = new_line_index;
                self.cursor.index = new_index;
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
                        if (i * self.area.w as usize) <= self.cursor.index {
                            row_index = i;
                        } else {
                            break;
                        }
                    }
                    if row_index > 0 {
                        let row_offset = self.cursor.index % self.area.w as usize;
                        self.cursor.index = ((row_index - 1) * self.area.w as usize)
                            + row_offset;
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
                        let row_count = (line.content.len() as f32 / self.area.w as f32)
                            .ceil() as usize;
                        if row_count > 1 {
                            let row_len = line.content.len() % self.area.w as usize;
                            let row_offset = self.cursor.index;
                            if row_len < row_offset {
                                self.cursor.index = line.content.len();
                            } else {
                                self.cursor.index = ((row_count - 1) * self.area.w as usize)
                                    + row_offset;
                            }
                        } else {
                            if line.content.len() < self.cursor.index {
                                self.cursor.index = line.content.len();
                            }
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
                        if (i * self.area.w as usize) <= self.cursor.index {
                            row_index = i;
                        } else {
                            break;
                        }
                    }
                    if row_index + 1 < row_count {
                        let row_offset = self.cursor.index % self.area.w as usize;
                        self.cursor.index = ((row_index + 1) * self.area.w as usize)
                            + row_offset;
                    } else {
                        if self.cursor.line + 1 < self.lines.len() {
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
            EditAction::ScrollUp => {
                self.scroll_y_offset = self.scroll_y_offset.saturating_sub(1);
            }
            EditAction::ScrollDown => {
                self.scroll_y_offset = self.scroll_y_offset.saturating_add(1)
                    .min(self.lines.len() as u16);
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

pub struct Row<'a> {
    pub num: usize,
    pub index: usize,
    pub line_index: usize,
    pub content: &'a str,
}

pub enum EditAction {
    Insert(char),
    ClearSelection,
    DeleteSelection,
    NewLine,
    Backspace,
    Delete,
    Click((u16, u16)),
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MovePrevWord,
    MoveNextWord,
    ScrollUp,
    ScrollDown,
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



mod util {
    use dreg::Scancode;

    pub fn scancode_to_char(sc: Scancode) -> Option<char> {
        Some(match sc {
            Scancode::K_1 => '1',
            Scancode::K_2 => '2',
            Scancode::K_3 => '3',
            Scancode::K_4 => '4',
            Scancode::K_5 => '5',
            Scancode::K_6 => '6',
            Scancode::K_7 => '7',
            Scancode::K_8 => '8',
            Scancode::K_9 => '9',
            Scancode::K_0 => '0',

            Scancode::MINUS => '-',
            Scancode::EQUAL => '=',
            Scancode::L_BRACE => '[',
            Scancode::R_BRACE => ']',
            Scancode::BACKSLASH => '\\',
            Scancode::SEMICOLON => ';',
            Scancode::APOSTROPHE => '\'',
            Scancode::COMMA => ',',
            Scancode::DOT => '.',
            Scancode::SLASH => '/',

            Scancode::A => 'a',
            Scancode::B => 'b',
            Scancode::C => 'c',
            Scancode::D => 'd',
            Scancode::E => 'e',
            Scancode::F => 'f',
            Scancode::G => 'g',
            Scancode::H => 'h',
            Scancode::I => 'i',
            Scancode::J => 'j',
            Scancode::K => 'k',
            Scancode::L => 'l',
            Scancode::M => 'm',
            Scancode::N => 'n',
            Scancode::O => 'o',
            Scancode::P => 'p',
            Scancode::Q => 'q',
            Scancode::R => 'r',
            Scancode::S => 's',
            Scancode::T => 't',
            Scancode::U => 'u',
            Scancode::V => 'v',
            Scancode::W => 'w',
            Scancode::X => 'x',
            Scancode::Y => 'y',
            Scancode::Z => 'z',

            _ => None?,
        })
    }

    pub fn shifted_char(ch: char) -> char {
        match ch {
            '1' => '!',
            '2' => '@',
            '3' => '#',
            '4' => '$',
            '5' => '%',
            '6' => '^',
            '7' => '&',
            '8' => '8',
            '9' => '(',
            '0' => ')',

            '-' => '_',
            '=' => '+',
            '[' => '{',
            ']' => '}',
            '\\' => '|',
            ';' => ':',
            '\'' => '\"',
            ',' => '<',
            '.' => '>',
            '/' => '?',

            other => other.to_ascii_uppercase(),
        }
    }
}
