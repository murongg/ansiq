use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, EventStream, KeyCode, KeyEventKind, KeyModifiers};
use futures_util::{Stream, StreamExt};

mod detection;
mod session;

pub use detection::{TerminalCapabilities, detect_terminal_capabilities};
pub use session::{
    InlineReservePlan, TerminalGuard, TerminalSession, Viewport, ViewportPolicy,
    cursor_y_after_history_entries, fit_viewport_height, initial_viewport_plan,
    inline_reserve_plan, reanchor_viewport_plan, resize_viewport_plan, safe_exit_row,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TerminalMode {
    pub raw_mode: bool,
    pub alternate_screen: bool,
}

impl TerminalMode {
    pub fn enter(self) -> Self {
        Self {
            raw_mode: true,
            alternate_screen: false,
        }
    }

    pub fn exit(self) -> Self {
        Self {
            raw_mode: false,
            alternate_screen: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Backspace,
    Enter,
    Tab,
    BackTab,
    Esc,
    Up,
    Down,
    Left,
    Right,
    Char(char),
    CtrlC,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputEvent {
    Key(Key),
    Resize(u16, u16),
}

pub fn map_event(event: Event) -> Option<InputEvent> {
    match event {
        Event::Key(key) if key.kind != KeyEventKind::Press => None,
        Event::Key(key) => Some(InputEvent::Key(match (key.code, key.modifiers) {
            (KeyCode::Backspace, _) => Key::Backspace,
            (KeyCode::Enter, _) => Key::Enter,
            (KeyCode::Tab, _) => Key::Tab,
            (KeyCode::BackTab, _) => Key::BackTab,
            (KeyCode::Esc, _) => Key::Esc,
            (KeyCode::Up, _) => Key::Up,
            (KeyCode::Down, _) => Key::Down,
            (KeyCode::Left, _) => Key::Left,
            (KeyCode::Right, _) => Key::Right,
            (KeyCode::Char('c'), modifiers) if modifiers.contains(KeyModifiers::CONTROL) => {
                Key::CtrlC
            }
            (KeyCode::Char(ch), _) => Key::Char(ch),
            _ => return None,
        })),
        Event::Resize(width, height) => Some(InputEvent::Resize(width, height)),
        _ => None,
    }
}

pub fn poll_event(timeout: Duration) -> io::Result<Option<InputEvent>> {
    if event::poll(timeout)? {
        Ok(map_event(event::read()?))
    } else {
        Ok(None)
    }
}

pub async fn next_input_event_from_stream<S>(
    stream: &mut S,
    timeout: Duration,
) -> io::Result<Option<InputEvent>>
where
    S: Stream<Item = io::Result<Event>> + Unpin,
{
    tokio::select! {
        maybe_event = stream.next() => {
            match maybe_event {
                Some(Ok(event)) => Ok(map_event(event)),
                Some(Err(error)) => Err(error),
                None => Ok(None),
            }
        }
        _ = tokio::time::sleep(timeout) => Ok(None),
    }
}

pub struct InputEventStream {
    inner: EventStream,
}

impl Default for InputEventStream {
    fn default() -> Self {
        Self {
            inner: EventStream::new(),
        }
    }
}

impl InputEventStream {
    pub async fn next_event(&mut self, timeout: Duration) -> io::Result<Option<InputEvent>> {
        next_input_event_from_stream(&mut self.inner, timeout).await
    }
}
