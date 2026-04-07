use std::io;
use std::time::Duration;

use ansiq_core::Rect;
use ansiq_render::{
    FrameBuffer, diff_buffers, diff_buffers_in_regions, frame_patches, render_cursor_at_origin,
    render_patches_at_origin,
};
use ansiq_surface::{InputEvent, InputEventStream, TerminalSession, Viewport, ViewportPolicy};

use crate::{App, Engine, cursor_position, draw_tree, draw_tree_in_regions};

pub fn viewport_bounds(viewport: Viewport) -> Rect {
    Rect::new(0, 0, viewport.width, viewport.height)
}

pub fn exit_row_for_content(viewport: Viewport, required_height: u16) -> u16 {
    let used_height = required_height.clamp(1, viewport.height);
    viewport
        .origin_y
        .saturating_add(used_height.saturating_sub(1))
}

pub async fn run_app<A: App>(app: A) -> io::Result<()> {
    run_app_with_policy(app, ViewportPolicy::PreserveVisible).await
}

pub async fn run_app_with_policy<A: App>(app: A, policy: ViewportPolicy) -> io::Result<()> {
    let mut terminal = TerminalSession::enter(policy)?;
    let mut viewport = terminal.viewport();
    let mut input_stream = InputEventStream::default();

    let mut engine = Engine::new(app);
    engine.mount();
    engine.set_bounds(viewport_bounds(viewport));
    engine.render_tree();
    if flush_committed_history(&mut terminal, &mut viewport, policy, &mut engine)? {
        engine.render_tree();
    }
    sync_viewport_to_content(&mut terminal, &mut viewport, policy, &mut engine)?;

    let mut previous = FrameBuffer::new(viewport.width, viewport.height);
    render_current_frame(&mut terminal, &engine, viewport.origin_y, &mut previous)?;

    loop {
        if engine.drain_requests() {
            break;
        }

        if let Some(event) = input_stream.next_event(Duration::from_millis(16)).await? {
            match event {
                InputEvent::Resize(width, height) => {
                    viewport = terminal.resize(policy, (width, height));
                    engine.set_bounds(viewport_bounds(viewport));
                    previous = FrameBuffer::new(viewport.width, viewport.height);
                }
                InputEvent::Key(key) => {
                    if engine.handle_input(key) {
                        break;
                    }
                }
            }
        }

        if engine.drain_requests() {
            break;
        }

        if engine.is_dirty() {
            engine.render_tree();
            if flush_committed_history(&mut terminal, &mut viewport, policy, &mut engine)? {
                previous = FrameBuffer::new(viewport.width, viewport.height);
                engine.render_tree();
            }
            if sync_viewport_to_content(&mut terminal, &mut viewport, policy, &mut engine)? {
                previous = FrameBuffer::new(viewport.width, viewport.height);
            }
            render_current_frame(&mut terminal, &engine, viewport.origin_y, &mut previous)?;
        }

        tokio::task::yield_now().await;
    }

    Ok(())
}

fn flush_committed_history<A: App>(
    terminal: &mut TerminalSession,
    viewport: &mut Viewport,
    policy: ViewportPolicy,
    engine: &mut Engine<A>,
) -> io::Result<bool> {
    let history = engine.take_pending_history();
    if history.is_empty() {
        return Ok(false);
    }

    // Completed turns become terminal scrollback instead of inflating the live viewport.
    *viewport = terminal.commit_history_blocks(history, policy)?;
    engine.set_bounds(viewport_bounds(*viewport));
    Ok(true)
}

fn sync_viewport_to_content<A: App>(
    terminal: &mut TerminalSession,
    viewport: &mut Viewport,
    policy: ViewportPolicy,
    engine: &mut Engine<A>,
) -> io::Result<bool> {
    let Some(requested_height) = policy.requested_height(viewport.height, engine.required_height())
    else {
        return Ok(false);
    };

    // ReservePreferred is the app shell mode: when content naturally grows,
    // expand the inline viewport so the stacked transcript is not clipped early.
    terminal.reserve_inline_space(requested_height)?;
    *viewport = terminal.viewport();
    engine.set_bounds(viewport_bounds(*viewport));
    engine.render_tree();
    Ok(true)
}

fn render_current_frame<A: App>(
    terminal: &mut TerminalSession,
    engine: &Engine<A>,
    origin_y: u16,
    previous: &mut FrameBuffer,
) -> io::Result<()> {
    let Some(tree) = engine.tree() else {
        return Ok(());
    };

    let next = if let Some(regions) = engine.redraw_regions() {
        let mut next = previous.clone();
        draw_tree_in_regions(tree, engine.focused(), &mut next, regions);
        next
    } else {
        let mut next = FrameBuffer::new(previous.width(), previous.height());
        draw_tree(tree, engine.focused(), &mut next);
        next
    };
    let cursor = cursor_position(tree, engine.focused());

    let patches = if let Some(regions) = engine.redraw_regions() {
        diff_buffers_in_regions(previous, &next, regions)
    } else if previous.is_blank() {
        frame_patches(&next)
    } else {
        diff_buffers(previous, &next)
    };
    let mut bytes = Vec::new();
    if !patches.is_empty() {
        render_patches_at_origin(&mut bytes, &patches, origin_y)?;
    }
    render_cursor_at_origin(&mut bytes, cursor, origin_y)?;
    let output = String::from_utf8(bytes).expect("rendered patches should be valid utf-8");
    terminal.write_ansi(&output)?;
    terminal.set_exit_row(exit_row_for_content(
        terminal.viewport(),
        engine.required_height(),
    ));

    *previous = next;
    Ok(())
}
