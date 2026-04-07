use std::io::{self, Stdout, Write};

use crate::{TerminalCapabilities, detect_terminal_capabilities};
use ansiq_core::HistoryEntry;
use ansiq_render::render_history_entries;
use crossterm::{
    cursor, execute,
    terminal::{self, ClearType, ScrollUp},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InlineReservePlan {
    pub origin_y: u16,
    pub scroll_up: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Viewport {
    pub width: u16,
    pub height: u16,
    pub origin_y: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewportPolicy {
    PreserveVisible,
    ReservePreferred(u16),
    ReserveFitContent { min: u16, max: u16 },
}

impl ViewportPolicy {
    pub fn requested_height(self, current_height: u16, content_height: u16) -> Option<u16> {
        match self {
            Self::PreserveVisible => None,
            Self::ReservePreferred(_) if content_height > current_height => Some(content_height),
            Self::ReservePreferred(_) => None,
            Self::ReserveFitContent { min, max } => {
                let target = content_height.clamp(min.max(1), max.max(min.max(1)));
                (target != current_height).then_some(target)
            }
        }
    }

    pub fn resolve(
        self,
        size: (u16, u16),
        cursor_y: u16,
        capabilities: TerminalCapabilities,
    ) -> Viewport {
        let (width, height) = normalize_terminal_size(size);
        let cursor_y = cursor_y.min(height.saturating_sub(1));

        match self {
            Self::PreserveVisible => Viewport {
                width,
                height: height.saturating_sub(cursor_y).max(1),
                origin_y: cursor_y,
            },
            Self::ReservePreferred(preferred_height) if capabilities.supports_inline_reserve => {
                let plan = inline_reserve_plan(height, cursor_y, preferred_height);
                Viewport {
                    width,
                    height: preferred_height.clamp(1, height),
                    origin_y: plan.origin_y,
                }
            }
            Self::ReservePreferred(_) => Viewport {
                width,
                height: height.saturating_sub(cursor_y).max(1),
                origin_y: cursor_y,
            },
            Self::ReserveFitContent { min, .. } if capabilities.supports_inline_reserve => {
                let plan = inline_reserve_plan(height, cursor_y, min.max(1));
                Viewport {
                    width,
                    height: min.clamp(1, height),
                    origin_y: plan.origin_y,
                }
            }
            Self::ReserveFitContent { .. } => Viewport {
                width,
                height: height.saturating_sub(cursor_y).max(1),
                origin_y: cursor_y,
            },
        }
    }
}

pub fn initial_viewport_plan(
    policy: ViewportPolicy,
    size: (u16, u16),
    cursor_y: u16,
    capabilities: TerminalCapabilities,
) -> (Viewport, Option<InlineReservePlan>) {
    let viewport = policy.resolve(size, cursor_y, capabilities);
    let reserve_plan = match policy {
        ViewportPolicy::ReservePreferred(preferred_height)
            if capabilities.supports_inline_reserve =>
        {
            let (_, terminal_height) = normalize_terminal_size(size);
            Some(inline_reserve_plan(
                terminal_height,
                cursor_y,
                preferred_height,
            ))
        }
        ViewportPolicy::ReserveFitContent { min, .. } if capabilities.supports_inline_reserve => {
            let (_, terminal_height) = normalize_terminal_size(size);
            Some(inline_reserve_plan(terminal_height, cursor_y, min.max(1)))
        }
        _ => None,
    };

    (viewport, reserve_plan)
}

pub fn reanchor_viewport_plan(
    policy: ViewportPolicy,
    size: (u16, u16),
    cursor_y: u16,
    current: Viewport,
    capabilities: TerminalCapabilities,
) -> (Viewport, Option<InlineReservePlan>) {
    let (width, height) = normalize_terminal_size(size);
    let cursor_y = cursor_y.min(height.saturating_sub(1));

    match policy {
        ViewportPolicy::PreserveVisible => (
            Viewport {
                width,
                height: height.saturating_sub(cursor_y).max(1),
                origin_y: cursor_y,
            },
            None,
        ),
        ViewportPolicy::ReservePreferred(preferred_height)
            if capabilities.supports_inline_reserve =>
        {
            let target_height = preferred_height.clamp(1, height);
            let plan = inline_reserve_plan(height, cursor_y, target_height);
            (
                Viewport {
                    width,
                    height: target_height,
                    origin_y: plan.origin_y,
                },
                Some(plan),
            )
        }
        ViewportPolicy::ReserveFitContent { min, max } if capabilities.supports_inline_reserve => {
            let target_height = current
                .height
                .clamp(min.max(1), max.max(min.max(1)))
                .clamp(1, height);
            let plan = inline_reserve_plan(height, cursor_y, target_height);
            (
                Viewport {
                    width,
                    height: target_height,
                    origin_y: plan.origin_y,
                },
                Some(plan),
            )
        }
        ViewportPolicy::ReservePreferred(_) | ViewportPolicy::ReserveFitContent { .. } => (
            Viewport {
                width,
                height: height.saturating_sub(cursor_y).max(1),
                origin_y: cursor_y,
            },
            None,
        ),
    }
}

pub fn resize_viewport_plan(
    policy: ViewportPolicy,
    size: (u16, u16),
    current: Viewport,
    capabilities: TerminalCapabilities,
) -> Viewport {
    let (width, height) = normalize_terminal_size(size);

    match policy {
        ViewportPolicy::PreserveVisible => Viewport {
            width,
            height: height.saturating_sub(current.origin_y).max(1),
            origin_y: current.origin_y.min(height.saturating_sub(1)),
        },
        ViewportPolicy::ReservePreferred(preferred_height)
            if capabilities.supports_inline_reserve =>
        {
            fit_viewport_height(
                Viewport { width, ..current },
                height,
                current.height.max(preferred_height),
            )
        }
        ViewportPolicy::ReserveFitContent { min, max } if capabilities.supports_inline_reserve => {
            let target_height = current.height.clamp(min.max(1), max.max(min.max(1)));
            fit_viewport_height(Viewport { width, ..current }, height, target_height)
        }
        ViewportPolicy::ReservePreferred(_) | ViewportPolicy::ReserveFitContent { .. } => {
            Viewport {
                width,
                height: height.saturating_sub(current.origin_y).max(1),
                origin_y: current.origin_y.min(height.saturating_sub(1)),
            }
        }
    }
}

pub fn fit_viewport_height(
    current: Viewport,
    terminal_height: u16,
    preferred_height: u16,
) -> Viewport {
    let terminal_height = terminal_height.max(1);
    let target_height = preferred_height.clamp(1, terminal_height);

    if target_height <= current.height {
        Viewport {
            width: current.width,
            height: target_height,
            origin_y: current
                .origin_y
                .min(terminal_height.saturating_sub(target_height)),
        }
    } else {
        let plan = inline_reserve_plan(terminal_height, current.origin_y, target_height);
        Viewport {
            width: current.width,
            height: target_height,
            origin_y: plan.origin_y,
        }
    }
}

pub fn cursor_y_after_history_entries(origin_y: u16, rendered_rows: u16) -> u16 {
    origin_y.saturating_add(rendered_rows)
}

pub fn safe_exit_row(exit_row: u16, size: (u16, u16)) -> u16 {
    let (_, height) = normalize_terminal_size(size);
    exit_row.min(height.saturating_sub(1))
}

pub struct TerminalSession {
    stdout: Stdout,
    capabilities: TerminalCapabilities,
    viewport: Viewport,
    exit_row: u16,
}

impl TerminalSession {
    pub fn enter(policy: ViewportPolicy) -> io::Result<Self> {
        let (_, cursor_y) = cursor::position()?;
        terminal::enable_raw_mode()?;

        let mut stdout = io::stdout();
        execute!(stdout, cursor::Hide)?;

        let capabilities = detect_terminal_capabilities();
        let size = terminal::size()?;
        let (viewport, reserve_plan) = initial_viewport_plan(policy, size, cursor_y, capabilities);
        if let Some(plan) = reserve_plan {
            // ReservePreferred is the app-like mode: we may scroll to make room,
            // but that policy now lives entirely in the surface layer.
            if plan.scroll_up > 0 {
                execute!(stdout, ScrollUp(plan.scroll_up))?;
            }
        }

        Ok(Self {
            stdout,
            capabilities,
            viewport,
            exit_row: viewport
                .origin_y
                .saturating_add(viewport.height.saturating_sub(1)),
        })
    }

    pub fn size(&self) -> io::Result<(u16, u16)> {
        terminal::size()
    }

    pub fn capabilities(&self) -> TerminalCapabilities {
        self.capabilities
    }

    pub fn origin_y(&self) -> u16 {
        self.viewport.origin_y
    }

    pub fn viewport(&self) -> Viewport {
        self.viewport
    }

    pub fn resize(&mut self, policy: ViewportPolicy, size: (u16, u16)) -> Viewport {
        self.viewport = resize_viewport_plan(policy, size, self.viewport, self.capabilities);
        self.exit_row = self
            .viewport
            .origin_y
            .saturating_add(self.viewport.height.saturating_sub(1));
        self.viewport
    }

    pub fn reserve_inline_space(&mut self, preferred_height: u16) -> io::Result<()> {
        let (_, terminal_height) = self.size()?;
        let old_bottom = self
            .viewport
            .origin_y
            .saturating_add(self.viewport.height.saturating_sub(1));
        let target_viewport = fit_viewport_height(self.viewport, terminal_height, preferred_height);
        let plan = inline_reserve_plan(
            terminal_height,
            self.viewport.origin_y,
            target_viewport.height,
        );
        if plan.scroll_up > 0 {
            execute!(self.stdout, ScrollUp(plan.scroll_up))?;
        }
        let clear_from = self.viewport.origin_y.min(target_viewport.origin_y);
        self.viewport = target_viewport;
        let new_bottom = self
            .viewport
            .origin_y
            .saturating_add(self.viewport.height.saturating_sub(1));
        if self.viewport.origin_y != clear_from || new_bottom < old_bottom {
            execute!(
                self.stdout,
                cursor::MoveTo(0, clear_from),
                terminal::Clear(ClearType::FromCursorDown)
            )?;
        }
        self.exit_row = self
            .viewport
            .origin_y
            .saturating_add(self.viewport.height.saturating_sub(1));
        Ok(())
    }

    pub fn commit_history_blocks(
        &mut self,
        blocks: Vec<HistoryEntry>,
        policy: ViewportPolicy,
    ) -> io::Result<Viewport> {
        if blocks.is_empty() {
            return Ok(self.viewport);
        }

        execute!(
            self.stdout,
            cursor::MoveTo(0, self.viewport.origin_y),
            terminal::Clear(ClearType::FromCursorDown)
        )?;

        let rendered_rows = render_history_entries(&mut self.stdout, &blocks, self.viewport.width)?;
        let cursor_y = cursor_y_after_history_entries(self.viewport.origin_y, rendered_rows);
        self.reanchor(policy, cursor_y)
    }

    pub fn reanchor(&mut self, policy: ViewportPolicy, cursor_y: u16) -> io::Result<Viewport> {
        let size = self.size()?;
        let (viewport, reserve_plan) =
            reanchor_viewport_plan(policy, size, cursor_y, self.viewport, self.capabilities);
        if let Some(plan) = reserve_plan {
            if self.capabilities.supports_inline_reserve && plan.scroll_up > 0 {
                execute!(self.stdout, ScrollUp(plan.scroll_up))?;
            }
        }
        self.viewport = viewport;
        self.exit_row = self
            .viewport
            .origin_y
            .saturating_add(self.viewport.height.saturating_sub(1));
        Ok(self.viewport)
    }

    pub fn set_exit_row(&mut self, row: u16) {
        self.exit_row = row;
    }

    pub fn write_ansi(&mut self, output: &str) -> io::Result<()> {
        self.stdout.write_all(output.as_bytes())?;
        self.stdout.flush()
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let exit_row = terminal::size()
            .map(|size| safe_exit_row(self.exit_row, size))
            .unwrap_or(self.exit_row);
        let _ = execute!(self.stdout, cursor::MoveTo(0, exit_row), cursor::Show);
        let _ = writeln!(self.stdout);
        let _ = self.stdout.flush();
        let _ = terminal::disable_raw_mode();
    }
}

pub type TerminalGuard = TerminalSession;

pub fn inline_reserve_plan(
    terminal_height: u16,
    cursor_y: u16,
    preferred_height: u16,
) -> InlineReservePlan {
    // Keep the viewport anchored to the launch cursor when possible. Only ask the
    // terminal to scroll when a preferred inline working height cannot fit below it.
    let terminal_height = terminal_height.max(1);
    let cursor_y = cursor_y.min(terminal_height.saturating_sub(1));
    let target_height = preferred_height.clamp(1, terminal_height);
    let remaining = terminal_height.saturating_sub(cursor_y);

    if remaining >= target_height {
        InlineReservePlan {
            origin_y: cursor_y,
            scroll_up: 0,
        }
    } else {
        let scroll_up = target_height - remaining;
        InlineReservePlan {
            origin_y: cursor_y.saturating_sub(scroll_up),
            scroll_up,
        }
    }
}

fn normalize_terminal_size((width, height): (u16, u16)) -> (u16, u16) {
    let width = if width == 0 { 80 } else { width };
    let height = if height == 0 { 24 } else { height };
    (width, height)
}
