#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TerminalCapabilities {
    pub supports_inline_reserve: bool,
}

pub fn detect_terminal_capabilities() -> TerminalCapabilities {
    // Start conservative but allow the current inline reserve strategy on
    // normal interactive terminals.
    TerminalCapabilities {
        supports_inline_reserve: true,
    }
}
