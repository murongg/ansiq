mod buffer;
mod diff;

pub use buffer::{Cell, FrameBuffer};
pub use diff::{
    Patch, diff_buffers, diff_buffers_in_regions, frame_patches, history_block_from_buffer,
    render_cursor, render_cursor_at_origin, render_history_entries, render_patches,
    render_patches_at_origin,
};
