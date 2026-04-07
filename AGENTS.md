# Repository Guidelines

## Project Structure & Module Organization
Ansiq is a Rust workspace. Core crates live under `packages/`:

- `ansiq-core`: reactive primitives, elements, styles, shared contracts
- `ansiq-runtime`: app loop, routing, redraw, subtree replacement
- `ansiq-layout`: measurement and relayout
- `ansiq-render`: framebuffer diff and terminal patch generation
- `ansiq-surface`: terminal session and viewport management
- `ansiq-widgets`: widget builders and higher-level primitives
- `ansiq-macros`: `view!` macro support
- `examples`: runnable demos under `examples/examples/`

Design notes and internal docs live in `docs/`. Prefer adding architecture notes there rather than inline README expansions.

## Build, Test, and Development Commands
- `cargo check --workspace` — fast whole-workspace type check
- `cargo test --workspace` — full test suite
- `cargo fmt --all` — format all crates
- `cargo run -p ansiq-examples --example list_navigation` — run a simple demo
- `cargo run -p ansiq-examples --example activity_monitor` — run a richer app shell

Run workspace checks before claiming completion.

## Coding Style & Naming Conventions
Use standard Rust formatting via `cargo fmt`. Prefer small focused modules over large mixed-responsibility files. Public API names should be explicit and user-facing; internal compatibility aliases are acceptable when migrating APIs. Keep comments short and explanatory, not narrative. Use `snake_case` for functions/modules, `CamelCase` for types, and align new APIs with existing `signal / computed / effect` naming.

## Testing Guidelines
Write tests close to the affected crate. Existing coverage includes:
- unit/integration tests in `packages/*/tests/`
- compile-fail tests via `trybuild` in `packages/ansiq-core/tests/ui/`

Name tests for behavior, e.g. `effect_reruns_on_flush_after_its_dependency_changes`. For behavior changes, add or update a failing test first, then implement.

## Commit & Pull Request Guidelines
This repository does not yet have an established commit history convention. Use concise Conventional Commit style messages such as `feat: add effect api` or `fix: preserve viewport height after history commit`. PRs should include:
- what changed
- why it changed
- verification commands run
- screenshots or terminal captures for UI-facing changes

## Architecture Notes
Keep the boundary clear: reactivity decides what is dirty; runtime decides how to update the screen. Avoid pushing app-specific behavior into runtime or surface layers.
