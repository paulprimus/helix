use std::borrow::Cow;

use helix_core::{
    graphemes::prev_grapheme_boundary, line_ending::rope_is_line_ending, movement::Movement,
    ropey::iter::Lines, Range, RopeSlice,
};

use super::Context;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Command {
    Yank,
    Delete,
    Change,
}

#[derive(Copy, Clone)]
enum SetMode {
    Normal,
    Insert,
}

#[derive(Eq, PartialEq)]
enum Modifier {
    InnerWord,
}

#[derive(Debug, Eq, PartialEq)]
enum Motion {
    PrevWordStart,
    NextWordEnd,
    PrevLongWordStart,
    NextLongWordEnd,
    LineStart,
    LineEnd,
}

#[derive(Debug)]
pub enum CollapseMode {
    Forward,
    Backward,
    ToAnchor,
    ToHead,
}

struct EvilContext {
    command: Option<Command>,
    motion: Option<Motion>,
    count: Option<usize>,
    modifiers: Vec<Modifier>,
    set_mode: Option<SetMode>,
}

impl EvilContext {
    pub fn reset(&mut self) {
        self.command = None;
        self.motion = None;
        self.count = None;
        self.modifiers.clear();
        self.set_mode = None;
    }
}

pub struct EvilCommands;

impl EvilCommands {
    fn trace<T>(cx: &mut Context, msg: T)
    where
        T: Into<Cow<'static, str>>,
    {
        cx.editor.set_status(msg);
    }

    pub fn is_enabled() -> bool {
        true
    }

    pub fn collapse_selections(cx: &mut Context, collapse_mode: CollapseMode) {
        let (view, doc) = current!(cx.editor);

        doc.set_selection(
            view.id,
            doc.selection(view.id).clone().transform(|mut range| {
                match collapse_mode {
                    CollapseMode::Forward => {
                        let end = range.anchor.max(range.head);
                        range.anchor = 0.max(end.saturating_sub(1));
                        range.head = end;
                    }
                    CollapseMode::Backward => {
                        let start = range.anchor.min(range.head);
                        range.anchor = start;
                        range.head = start.saturating_add(1);
                    }
                    CollapseMode::ToAnchor => {
                        if range.head > range.anchor {
                            range.head = range.anchor.saturating_add(1);
                        } else {
                            range.head = 0.max(range.anchor.saturating_sub(1));
                        }
                    }
                    CollapseMode::ToHead => {
                        if range.head > range.anchor {
                            range.anchor = 0.max(range.head.saturating_add(1))
                        } else {
                            range.anchor = range.head.saturating_add(1)
                        }
                    }
                }
                return range;
            }),
        );
    }
}

pub fn evil_movement_paragraph_forward(
    slice: RopeSlice,
    range: Range,
    count: usize,
    behaviour: Movement,
) -> Range {
    let mut line: usize = range.cursor_line(slice); // cursor line number
    let last_char: bool =
        prev_grapheme_boundary(slice, slice.line_to_char(line + 1)) == range.cursor(slice);

    let curr_line_empty = rope_is_line_ending(slice.line(line));

    let next_line_empty =
        rope_is_line_ending(slice.line(slice.len_lines().saturating_sub(1).min(line + 1)));

    let curr_empty_to_line = curr_line_empty && !next_line_empty;

    // skip character after paragraph boundary
    if curr_empty_to_line && last_char {
        line += 1;
    }

    let mut lines = slice.lines_at(line).map(rope_is_line_ending).peekable();
    let last_line = line;

    for _ in 0..count {
        while lines.next_if(|&e| e).is_some() {
            line += 1;
        }
        while lines.next_if(|&e| !e).is_some() {
            line += 1;
        }
        if lines.next_if(|&e| e).is_some() {
            line += 1;
        }
        if line == last_line {
            break;
        }
    }

    let head = slice.line_to_char(line);
    let anchor = if behaviour == Movement::Move {
        if curr_empty_to_line && last_char {
            range.head
        } else {
            range.cursor(slice)
        }
    } else {
        range.put_cursor(slice, head, true).anchor
    };

    Range::new(anchor, head)
}

pub fn evil_movement_paragraph_backward(
    slice: RopeSlice,
    range: Range,
    count: usize,
    movement: Movement,
) -> Range {
    let mut line_nr: usize = range.cursor_line(slice); // cursor line number
    let first_char: bool =
        prev_grapheme_boundary(slice, slice.line_to_char(line_nr)) == range.cursor(slice);

    let prev_line_empty = rope_is_line_ending(slice.line(line_nr.saturating_sub(1)));
    let current_line_empty = rope_is_line_ending(slice.line(line_nr));

    let prev_empty_to_line = prev_line_empty && !current_line_empty;

    if prev_empty_to_line && !first_char {
        line_nr += 1;
    }

    let lines: Lines<'static> = slice.lines_at(line_nr);
    lines.reverse();

    range.put_cursor(slice, lines, true);
    Range::new(range.head, lines)
}
