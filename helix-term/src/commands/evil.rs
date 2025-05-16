use helix_core::{movement::Movement, Range, RopeSlice};

struct EvilContext {
    command: Option<Command>,
    motion: Option<Motion>,
    count: Option<size>,
    modifiers: Vec<Modifier>,
    set_mode: Option<SetMode>,
}

pub fn evil_movement_paragraph_forward(
    slice: RopeSlice,
    range: Range,
    count: usize,
    behaviour: Movement,
) -> Range {
    let mut line = range.cursor_line(slice);
}
