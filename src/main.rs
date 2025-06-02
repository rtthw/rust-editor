//! A rust editor.



mod workspace;

use std::{collections::HashSet, ops::Range, path::PathBuf};

use bog::{prelude::*, render::FontFamily};
use unicode_segmentation::UnicodeSegmentation as _;

use workspace::*;


pub const GRAY_0: Color = Color::new(13, 13, 23, 255); // 0d0d17
pub const GRAY_1: Color = Color::new(29, 29, 39, 255); // 1d1d27
pub const GRAY_2: Color = Color::new(43, 43, 53, 255); // 2b2b35
pub const GRAY_3: Color = Color::new(59, 59, 67, 255); // 3b3b43
pub const GRAY_4: Color = Color::new(73, 73, 83, 255); // 494953
pub const GRAY_5: Color = Color::new(89, 89, 109, 255); // 59596d
pub const GRAY_6: Color = Color::new(113, 113, 127, 255); // 71717f
pub const GRAY_7: Color = Color::new(139, 139, 149, 255); // 8b8b95
pub const GRAY_8: Color = Color::new(163, 163, 173, 255); // a3a3ad
pub const GRAY_9: Color = Color::new(191, 191, 197, 255); // bfbfc5



fn main() -> Result<()> {
    let workspace_info = find_workspace();
    println!("WORKSPACE_DIR: {}", workspace_info.path.display());
    println!("HAS_VERSION_CONTROL: {}", workspace_info.has_vc);
    let workspace = read_workspace(workspace_info)?;

    let syntaxes = syntect::parsing::SyntaxSet::load_defaults_nonewlines();

    run_app(App {
        shutdown: false,
        initialized: false,
        workspace,
        buffers: BufferSet::new("./src/main.rs".into(), include_str!("main.rs")),
        syntaxes,
        keys_down: HashSet::with_capacity(3),
    })?;

    Ok(())
}



struct App {
    shutdown: bool,
    initialized: bool,

    workspace: Workspace,
    buffers: BufferSet,
    syntaxes: syntect::parsing::SyntaxSet,
    keys_down: HashSet<KeyCode>,
}

impl AppHandler for App {
    fn render<'pass>(&'pass mut self, cx: AppContext, layers: &mut LayerStack<'pass>) {
        let buffer = self.buffers.current_buffer_mut();
        if buffer.needs_reparse {
            buffer.parse(&self.syntaxes);
            buffer.needs_reparse = false;
        }

        let (side_area, buffer_area) = cx.renderer.viewport_rect().hsplit_portion(0.2);

        let (gutter_area, buffer_area) = buffer_area.hsplit_len(23.0);

        buffer.area = buffer_area;

        let max_gutter_width = gutter_area.w as usize;
        let max_line_width = buffer_area.w as usize;

        let mut cursor_row = 0;
        let mut last_line_index = 1;
        let selection = buffer.selection_bounds();
        let mut y_offset = 0.0;
        for (index, row) in buffer.visible_rows().enumerate() {
            if row.line_index != last_line_index {
                // layers.fill_text(Text {
                //     content: format!("{}", row.line_index + 1),
                //     color: GRAY_5,
                //     size: 17.0,
                //     pos: vec2(gutter_area.x, gutter_area.y + y_offset),
                //     bounds: gutter_area.size(),
                //     font_family: FontFamily::Monospace,
                //     ..Default::default()
                // });
            }
            layers.fill_text(Text {
                content: &*&row.content,
                color: GRAY_7,
                size: 17.0,
                pos: vec2(buffer_area.x, buffer_area.y + y_offset),
                bounds: buffer_area.size(),
                font_family: FontFamily::Monospace,
                ..Default::default()
            });
            last_line_index = row.line_index;
            if row.line_index == buffer.cursor.line {
                if (row.index * buffer.area.w as usize) <= buffer.cursor.index {
                    cursor_row = index;
                }
            }

            // Highlight selection.
            // if let Some((start_cursor, end_cursor)) = &selection {
            //     if row.line_index >= start_cursor.line && row.line_index <= end_cursor.line {
            //         let start_index = start_cursor.index as u16 % buffer.area.w;
            //         let end_index = end_cursor.index as u16 % buffer.area.w;

            //         if start_cursor.line == end_cursor.line {
            //             // Highlight from the start index to the end index.
            //             for i in start_index..end_index {
            //                 frame.buffer.get_mut(area.x + i, area.y)
            //                     .bg = Color::from_rgb(0x59, 0x59, 0x6d);
            //             }
            //         } else if row.line_index == start_cursor.line {
            //             // Highlight from the start index to the end of the row.
            //             for i in start_index..(row.content.len() as u16) {
            //                 frame.buffer.get_mut(area.x + i, area.y)
            //                     .bg = Color::from_rgb(0x59, 0x59, 0x6d);
            //             }
            //         } else if row.line_index > start_cursor.line
            //             && row.line_index < end_cursor.line
            //         {
            //             // Highlight the whole line.
            //             for i in 0..row.content.len() {
            //                 frame.buffer.get_mut(area.x + i as u16, area.y)
            //                     .bg = Color::from_rgb(0x59, 0x59, 0x6d);
            //             }
            //         } else if row.line_index == end_cursor.line {
            //             // Highlight from the start of the row to the end index.
            //             for i in 0..end_index {
            //                 frame.buffer.get_mut(area.x + i, area.y)
            //                     .bg = Color::from_rgb(0x59, 0x59, 0x6d);
            //             }
            //         }
            //     }
            // }

            y_offset += 23.0; // TODO: `line_height`.
        }

        // frame.cursor = Some((
        //     buffer_area.x + (buffer.cursor.index as u16 % buffer.area.w),
        //     buffer_area.y + cursor_row as u16,
        // ));
    }

    // fn on_primary_mouse_down(&mut self, cx: AppContext) {
    //     let buffer = self.buffers.current_buffer_mut();
    //     buffer.perform_action(EditAction::Click((
    //         self.mouse_pos.x.saturating_sub(buffer.area.x),
    //         self.mouse_pos.y.saturating_sub(buffer.area.y),
    //     )));
    // }

    fn on_key_down(&mut self, cx: AppContext, code: KeyCode, _repeat: bool) {
        cx.window.request_redraw();
        match code {
            KeyCode::C_ARROWLEFT => {
                if self.keys_down.contains(&KeyCode::C_LSHIFT) {
                    self.buffers.current_buffer_mut().start_or_continue_selection();
                }
                if self.keys_down.contains(&KeyCode::C_LCTRL) {
                    self.buffers.current_buffer_mut().perform_action(EditAction::MovePrevWord);
                } else {
                    self.buffers.current_buffer_mut().perform_action(EditAction::MoveLeft);
                }
            }
            KeyCode::C_ARROWRIGHT => {
                if self.keys_down.contains(&KeyCode::C_LSHIFT) {
                    self.buffers.current_buffer_mut().start_or_continue_selection();
                }
                if self.keys_down.contains(&KeyCode::C_LCTRL) {
                    self.buffers.current_buffer_mut().perform_action(EditAction::MoveNextWord);
                } else {
                    self.buffers.current_buffer_mut().perform_action(EditAction::MoveRight);
                }
            }
            KeyCode::C_ARROWUP => {
                if self.keys_down.contains(&KeyCode::C_LSHIFT) {
                    self.buffers.current_buffer_mut().start_or_continue_selection();
                }
                self.buffers.current_buffer_mut().perform_action(EditAction::MoveUp);
            }
            KeyCode::C_ARROWDOWN => {
                if self.keys_down.contains(&KeyCode::C_LSHIFT) {
                    self.buffers.current_buffer_mut().start_or_continue_selection();
                }
                self.buffers.current_buffer_mut().perform_action(EditAction::MoveDown);
            }

            KeyCode::C_BACKSPACE => {
                self.buffers.current_buffer_mut().perform_action(EditAction::Backspace);
            }
            KeyCode::C_DELETE => {
                self.buffers.current_buffer_mut().perform_action(EditAction::Delete);
            }

            KeyCode::C_SPACE => {
                self.buffers.current_buffer_mut().perform_action(EditAction::Insert(' '));
            }
            // TODO: Indentation.
            KeyCode::C_TAB => {
                self.buffers.current_buffer_mut().perform_action(EditAction::Insert('\t'));
            }
            KeyCode::C_ENTER => {
                self.buffers.current_buffer_mut().perform_action(EditAction::NewLine);
            }

            other => {
                if let Some(ch) = util::keycode_to_char(other) {
                    if self.keys_down.contains(&KeyCode::C_LCTRL) {
                        match ch {
                            '[' => {
                                self.buffers.goto_previous(true);
                            }
                            ']' => {
                                self.buffers.goto_next(true);
                            }
                            _ => {}
                        }
                    } else if self.keys_down.contains(&KeyCode::C_LSHIFT) {
                        let ch = util::shifted_char(ch);
                        self.buffers.current_buffer_mut().perform_action(EditAction::Insert(ch));
                    } else {
                        self.buffers.current_buffer_mut().perform_action(EditAction::Insert(ch));
                    }
                }
            }
        }
    }

    fn on_wheel_movement(&mut self, _cx: AppContext, movement: WheelMovement) {
        match movement {
            WheelMovement::Lines { y, .. } => {
                if y.is_sign_negative() {
                    self.buffers.current_buffer_mut().perform_action(EditAction::ScrollDown);
                } else {
                    self.buffers.current_buffer_mut().perform_action(EditAction::ScrollUp);
                }
            }
            WheelMovement::Pixels { y, .. } => {
                if y.is_sign_negative() {
                    self.buffers.current_buffer_mut().perform_action(EditAction::ScrollDown);
                } else {
                    self.buffers.current_buffer_mut().perform_action(EditAction::ScrollUp);
                }
            }
        }
    }

    fn window_desc(&self) -> WindowDescriptor {
        WindowDescriptor {
            title: "Rust Editor",
            ..Default::default()
        }
    }
}



pub struct BufferSet {
    buffers: Vec<Buffer>,
    current: usize,
}

impl BufferSet {
    pub fn new(initial_buffer_path: PathBuf, initial_buffer_content: &str) -> Self {
        let scratch_buffer = Buffer::new(BufferKind::Other, "");
        let current_buffer = Buffer::new(
            BufferKind::File(initial_buffer_path),
            initial_buffer_content,
        );

        Self {
            buffers: vec![scratch_buffer, current_buffer],
            current: 1,
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
    kind: BufferKind,
    lines: Vec<Line>,
    scopes: Vec<(usize, Range<usize>, SourceScope)>,
    needs_reparse: bool,
    cursor: Cursor,
    selection: Selection,
    area: Rect,
    scroll_y_offset: u16,
}

pub enum BufferKind {
    File(PathBuf),
    Other,
}

impl Buffer {
    pub fn new(kind: BufferKind, content: &str) -> Self {
        let mut lines: Vec<Line> = content.lines()
            .map(|s| Line { content: s.to_string() })
            .collect();
        if lines.len() < 1 {
            lines.push(Line { content: "".to_string() });
        }

        Self {
            kind,
            lines,
            scopes: vec![],
            needs_reparse: true,
            cursor: Cursor { line: 0, index: 0 },
            selection: Selection::None,
            area: Rect::NONE, // Set by the render function in `App`.
            scroll_y_offset: 0,
        }
    }

    pub fn parse(&mut self, syntaxes: &syntect::parsing::SyntaxSet) {
        let BufferKind::File(path) = &self.kind else { return; };

        let syntax = syntaxes.find_syntax_for_file(path).unwrap().unwrap();
        let mut parser = syntect::parsing::ParseState::new(syntax);
        let mut scopes = syntect::parsing::ScopeStack::new();

        let selectors = ScopeSelectors::default();

        for (line_index, line) in self.lines.iter().enumerate() {
            let ops = parser.parse_line(&line.content, syntaxes).unwrap();
            for (range, op) in syntect::easy::ScopeRangeIterator::new(&ops, &line.content) {
                scopes.apply(op).unwrap();
                if range.is_empty() {
                    continue;
                }
                if let Some(scope) = {
                    if selectors.comment.does_match(scopes.as_slice()).is_some() {
                        if selectors.doc_comment.does_match(scopes.as_slice()).is_some() {
                            Some(SourceScope::DocComment)
                        } else {
                            Some(SourceScope::Comment)
                        }
                    } else if selectors.function.does_match(scopes.as_slice()).is_some() {
                        Some(SourceScope::Function)
                    } else if selectors.keyword.does_match(scopes.as_slice()).is_some() {
                        Some(SourceScope::Keyword)
                    } else if selectors.types.does_match(scopes.as_slice()).is_some() {
                        Some(SourceScope::Type)
                    } else {
                        None
                    }
                } {
                    self.scopes.push((
                        line_index,
                        range,
                        scope,
                    ));
                }
            }
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

        // TODO: Optimize the parsing sequence before re-parsing so frequently.
        // self.needs_reparse = true;

        cursor
    }

    pub fn insert_string(&mut self, content: &str) {
        self.delete_selection();
        let next_cursor = self.insert_at(self.cursor, content);
        self.cursor = next_cursor;
    }

    pub fn start_or_continue_selection(&mut self) {
        if let Selection::None = &self.selection {
            self.selection = Selection::Normal(self.cursor);
        }
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

        // TODO: Optimize the parsing sequence before re-parsing so frequently.
        // self.needs_reparse = true;
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
    use bog::event::KeyCode;

    pub fn keycode_to_char(sc: KeyCode) -> Option<char> {
        Some(match sc {
            KeyCode::AN_1 => '1',
            KeyCode::AN_2 => '2',
            KeyCode::AN_3 => '3',
            KeyCode::AN_4 => '4',
            KeyCode::AN_5 => '5',
            KeyCode::AN_6 => '6',
            KeyCode::AN_7 => '7',
            KeyCode::AN_8 => '8',
            KeyCode::AN_9 => '9',
            KeyCode::AN_0 => '0',

            KeyCode::AN_MINUS => '-',
            KeyCode::AN_EQUAL => '=',
            KeyCode::AN_LBRACKET => '[',
            KeyCode::AN_RBRACKET => ']',
            KeyCode::AN_BACKSLASH => '\\',
            KeyCode::AN_SEMICOLON => ';',
            KeyCode::AN_APOSTROPHE => '\'',
            KeyCode::AN_COMMA => ',',
            KeyCode::AN_DOT => '.',
            KeyCode::AN_SLASH => '/',

            KeyCode::AN_A => 'a',
            KeyCode::AN_B => 'b',
            KeyCode::AN_C => 'c',
            KeyCode::AN_D => 'd',
            KeyCode::AN_E => 'e',
            KeyCode::AN_F => 'f',
            KeyCode::AN_G => 'g',
            KeyCode::AN_H => 'h',
            KeyCode::AN_I => 'i',
            KeyCode::AN_J => 'j',
            KeyCode::AN_K => 'k',
            KeyCode::AN_L => 'l',
            KeyCode::AN_M => 'm',
            KeyCode::AN_N => 'n',
            KeyCode::AN_O => 'o',
            KeyCode::AN_P => 'p',
            KeyCode::AN_Q => 'q',
            KeyCode::AN_R => 'r',
            KeyCode::AN_S => 's',
            KeyCode::AN_T => 't',
            KeyCode::AN_U => 'u',
            KeyCode::AN_V => 'v',
            KeyCode::AN_W => 'w',
            KeyCode::AN_X => 'x',
            KeyCode::AN_Y => 'y',
            KeyCode::AN_Z => 'z',

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



pub struct ScopeSelectors {
    pub comment: syntect::highlighting::ScopeSelector,
    pub doc_comment: syntect::highlighting::ScopeSelectors,
    pub function: syntect::highlighting::ScopeSelectors,
    pub keyword: syntect::highlighting::ScopeSelectors,
    pub types: syntect::highlighting::ScopeSelectors,
}

impl Default for ScopeSelectors {
    fn default() -> ScopeSelectors {
        ScopeSelectors {
            comment: "comment - comment.block.attribute".parse().unwrap(),
            doc_comment: "comment.line.documentation, comment.block.documentation".parse().unwrap(),
            function: "entity.name.function, support.function".parse().unwrap(),
            keyword: "keyword, storage".parse().unwrap(),
            types: "entity.name.class, entity.name.struct, entity.name.enum, entity.name.type"
                .parse().unwrap(),
        }
    }
}

pub enum SourceScope {
    Comment,
    DocComment,
    Function,
    Keyword,
    Type,
}

impl SourceScope {
    pub const fn color(&self) -> Color {
        match self {
            SourceScope::Comment => Color::new(0x59, 0x59, 0x6d, 0xff),
            SourceScope::DocComment => Color::new(0x87, 0xb6, 0x97, 0xff),
            SourceScope::Function => Color::new(0x95, 0xb7, 0xdf, 0xff),
            SourceScope::Keyword => Color::new(0xd9, 0x6d, 0x81, 0xff),
            SourceScope::Type => Color::new(0x8b, 0x8b, 0x95, 0xff),
        }
    }
}
