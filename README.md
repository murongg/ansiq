# Ansiq

Ansiq is a runtime-first TUI framework for long-running terminal applications.

It is designed around one core idea:

> Streams, tasks, and input should be coordinated by a single terminal runtime.

Ansiq is not a widget-heavy component kit. It is a Rust-only runtime for streaming-first terminal UX, aimed at AI agents, developer tools, dashboards, and similar interactive systems.

## Positioning

Ansiq focuses on:

- async-native terminal applications
- retained UI tree rendering
- incremental framebuffer diffing
- focus-aware input routing
- background task integration through message passing
- streaming output as a first-class UI behavior
- a signal-first Rust API with function components and a `view!` macro

## Architecture

The render path is:

`retained tree -> layout -> framebuffer -> diff -> terminal`

The runtime path is:

`input / async message -> update -> rebuild tree -> layout -> diff render`

Key runtime rules:

- UI changes are coordinated by one runtime loop
- async tasks never mutate the UI directly
- background work sends messages back to the runtime through channels
- the renderer flushes only changed cells instead of clearing and repainting the whole screen
- terminal session policy lives in `ansiq-surface`, so the runtime does not invent its own terminal occupancy rules
- reactive handles are thread-affine and intentionally non-`Send`
- terminal input intake uses async event streams instead of blocking a Tokio worker with synchronous polling

## Runtime Boundary

Ansiq runtime is responsible for:

- app lifecycle and the single UI loop
- reactive flush scheduling
- dirty scope collection
- subtree rerender and replacement
- focus management and input routing
- partial relayout
- invalidated region tracking
- terminal session and viewport management
- framebuffer diff and terminal patch emission

Ansiq runtime is not responsible for:

- business domain modeling
- network protocol implementations
- higher-level AI agent orchestration
- complex rich-text parsing or markdown semantics
- persistence and external storage policy

The detailed version of this boundary is documented in
[`docs/runtime-boundary.md`](/Users/murong/Documents/myopensource/fluxion/docs/runtime-boundary.md).
The release process and publish order are documented in
[`docs/release-checklist.md`](/Users/murong/Documents/myopensource/fluxion/docs/release-checklist.md).

## Monorepo Layout

This repository is a Cargo workspace monorepo. Framework crates live under [`packages/`](/Users/murong/Documents/myopensource/fluxion/packages), and runnable demos live under [`examples/`](/Users/murong/Documents/myopensource/fluxion/examples).

- [`packages/ansiq-core`](/Users/murong/Documents/myopensource/fluxion/packages/ansiq-core)
  Core element model, geometry, style types, signal-first reactivity primitives, and function-component helpers.
- [`packages/ansiq-macros`](/Users/murong/Documents/myopensource/fluxion/packages/ansiq-macros)
  `view!` proc macro for JSX-like declarative trees that compile to the same retained `Element` model.
- [`packages/ansiq-runtime`](/Users/murong/Documents/myopensource/fluxion/packages/ansiq-runtime)
  Main UI loop, focus handling, input routing, async message integration, and tree rendering orchestration.
- [`packages/ansiq-surface`](/Users/murong/Documents/myopensource/fluxion/packages/ansiq-surface)
  Terminal session lifecycle, viewport policy, raw mode, cursor handling, and event intake via `crossterm`.
- [`packages/ansiq-render`](/Users/murong/Documents/myopensource/fluxion/packages/ansiq-render)
  Cell buffer, diff generation, and ANSI patch emission.
- [`packages/ansiq-layout`](/Users/murong/Documents/myopensource/fluxion/packages/ansiq-layout)
  Minimal row/column layout with fixed/fill sizing and pane inset handling.
- [`packages/ansiq-widgets`](/Users/murong/Documents/myopensource/fluxion/packages/ansiq-widgets)
  Builder-style primitives: `Box`, `Block`, `Text`, `Paragraph`, `List`, `Table`, `Tabs`, `Gauge`, `LineGauge`, `Sparkline`, `Scrollbar`, `Clear`, `Pane`, `ScrollView`, `StreamingText`, `Input`, `StatusBar`.
- [`examples`](/Users/murong/Documents/myopensource/fluxion/examples)
  Runnable scenario demos and example applications.

Each package keeps its tests in a dedicated `tests/` directory instead of inline `#[cfg(test)]` modules.

## Runtime Implementation Status

The current runtime is no longer just an idea sketch. It already has working implementations for the main kernel responsibilities, but the maturity level is not uniform across every subsystem.

### Implemented

- terminal session setup and teardown without taking over the whole terminal by default
- surface-side viewport planning for initial reserve, resize, reanchor, and fit operations
- history reanchor based on counted rendered rows instead of terminal cursor-position queries
- `ReservePreferred(n)` now returns to `n` after history commit reanchor instead of treating temporary growth as the new steady-state shell height
- `HistoryEntry::Text` now normalizes through the same commit-time `HistoryBlock` wrapping path as structured history blocks
- terminal session teardown clamps the final exit row to the visible terminal height before restoring the cursor
- framebuffer-based diff rendering
- retained `Element` tree
- signal-first component API via `Cx`: `cx.signal`, `cx.computed`, and `cx.effect`
- standalone signal-first reactivity core in `ansiq-core` (`signal`, `computed`, `effect`, explicit `flush`)

`effect` is not React's `useEffect`: there is no dependency array. Dependencies are tracked automatically from reactive reads performed while the effect runs.
- `Signal::set_if_changed(...)` for non-breaking same-value propagation suppression when `T: PartialEq`
- focusable widgets and tab traversal
- optional focus trapping to a continuity-keyed subtree
- focused input routing for the `Input` widget
- app-level `on_unhandled_key(...)` for keys not consumed by widgets or runtime focus traversal
- async app messages with `tokio`
- dirty component scope collection
- subtree rerender and replacement
- explicit continuity keys for preserving focus and local widget state across subtree and root rerenders
- component subtree replacement refreshing wrapper-node measured height before ancestor relayout
- partial relayout along dirty ancestor chains
- invalidated region tracking
- overlapping dirty paths normalized before relayout and damage collection
- layout-only containers avoiding full-rect self-invalidation when only descendants shift
- separation between rerendering the tree and redrawing in-place interactive widget state
- core-owned layout contracts consumed by `ansiq-layout`
- debug-time hardening of strict layout primitives such as three-slot `Shell`
- core-owned widget key routing contracts consumed by `ansiq-runtime`
- core-owned render math consumed by `ansiq-runtime::draw`
- core-owned text shaping helpers consumed by `ansiq-runtime::draw`
- runtime-side drawing and cursor lookup split into focused internal modules instead of one monolithic `draw.rs`
- function component API via `Cx`
- `view!` macro for declarative UI trees

### Implemented But Still Maturing

- partial redraw planning and damage tracking for complex shells
- viewport growth, scrollback flush, and inline session behavior across varied terminal environments
- more advanced viewport pinning and detached scrollback semantics for long-running shells
- committed scrollback history currently wraps at commit-time width and does not reflow after later terminal resizes
- widget parity with `ratatui`
- Unicode/grapheme correctness beyond common terminal cases
- higher-level session/shell composition patterns

### Not Yet Complete

- keyed reconciliation
- mouse support
- a fully mature shell/layout model for complex workspaces
- virtualization for very large scroll regions
- complete `ratatui` widget parity

## Engineering Bar

Ansiq started from an MVP direction, but ongoing implementation work should not aim for “good enough for a demo”.

The current expectation is:

- prefer stable, reusable behavior over scenario-specific patches
- prefer library-level abstractions over example-local workarounds
- prefer explicit runtime boundaries over convenient coupling
- treat partially correct behavior as unfinished, not “close enough”

## Current Limitations

The following areas are still intentionally minimal or incomplete:

- no keyed reconciliation
- no mouse support
- no rich text or multiline editing
- no advanced layout engine or full flexbox behavior
- no virtualization for very large scroll regions
- Unicode handling is conservative and optimized for common terminal text, not every grapheme edge case
- `ScrollView` is optimized for text-like children in this MVP
- the `view!` macro is intentionally smaller than JSX and does not yet support inline control flow

## Example

The examples crate now provides multiple runnable demos:

- `activity_monitor`: macOS-style process monitor with live CPU, memory, energy, disk, and network tabs
- `list_navigation`: interactive list selection
- `scroll_sync`: shared scroll state between `ScrollView` and `Scrollbar`
- `table_interaction`: keyboard-driven table selection

Run:

```bash
cargo run -p ansiq-examples --example activity_monitor
cargo run -p ansiq-examples --example list_navigation
cargo run -p ansiq-examples --example scroll_sync
cargo run -p ansiq-examples --example table_interaction
```

Controls:

- `Tab` / `Shift-Tab`: move focus
- `Left` / `Right`: switch tabs when `Tabs` is focused
- `Up` / `Down`: move selection in tables and lists
- `j` / `k`: move focus when the focused widget does not consume the key
- `Ctrl+C`: exit

The example entrypoints opt into scenario-appropriate viewport policies.

The framework default remains a conservative inline mode that preserves visible terminal content above the app.

## Future Work

Logical next steps after the MVP:

- keyed tree reconciliation
- richer layout and sizing policies
- better Unicode and grapheme support
- mouse input
- additional widgets
- higher-level reactive helpers built on top of the signal core
- smarter damage tracking for large scrolling regions
- richer `view!` syntax, including controlled support for conditional and repeated children
