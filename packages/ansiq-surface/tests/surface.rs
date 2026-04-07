use ansiq_surface::{
    InlineReservePlan, InputEvent, Key, TerminalCapabilities, TerminalMode, Viewport,
    ViewportPolicy, cursor_y_after_history_entries, fit_viewport_height, initial_viewport_plan,
    inline_reserve_plan, map_event, next_input_event_from_stream, reanchor_viewport_plan,
    resize_viewport_plan, safe_exit_row,
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use futures_util::stream;

#[test]
fn terminal_mode_enter_and_exit_toggle_flags() {
    let entered = TerminalMode::default().enter();
    assert!(entered.raw_mode);
    assert!(!entered.alternate_screen);

    let exited = entered.exit();
    assert!(!exited.raw_mode);
    assert!(!exited.alternate_screen);
}

#[test]
fn map_event_translates_common_keys() {
    assert_eq!(
        map_event(Event::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE
        ))),
        Some(InputEvent::Key(Key::Enter))
    );
    assert_eq!(
        map_event(Event::Key(KeyEvent::new(
            KeyCode::Backspace,
            KeyModifiers::NONE
        ))),
        Some(InputEvent::Key(Key::Backspace))
    );
    assert_eq!(
        map_event(Event::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE))),
        Some(InputEvent::Key(Key::Tab))
    );
    assert_eq!(
        map_event(Event::Key(KeyEvent::new(
            KeyCode::BackTab,
            KeyModifiers::SHIFT
        ))),
        Some(InputEvent::Key(Key::BackTab))
    );
    assert_eq!(
        map_event(Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))),
        Some(InputEvent::Key(Key::Esc))
    );
    assert_eq!(
        map_event(Event::Key(KeyEvent::new(
            KeyCode::Char('j'),
            KeyModifiers::NONE
        ))),
        Some(InputEvent::Key(Key::Char('j')))
    );
}

#[test]
fn map_event_emits_ctrl_c_and_resize() {
    assert_eq!(
        map_event(Event::Key(KeyEvent::new(
            KeyCode::Char('c'),
            KeyModifiers::CONTROL
        ))),
        Some(InputEvent::Key(Key::CtrlC))
    );
    assert_eq!(
        map_event(Event::Resize(120, 32)),
        Some(InputEvent::Resize(120, 32))
    );
}

#[test]
fn map_event_ignores_key_release_events() {
    let released = Event::Key(KeyEvent {
        code: KeyCode::Char('x'),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Release,
        state: KeyEventState::NONE,
    });

    assert_eq!(map_event(released), None);
}

#[test]
fn inline_reserve_plan_keeps_origin_when_there_is_enough_space() {
    assert_eq!(
        inline_reserve_plan(40, 10, 18),
        InlineReservePlan {
            origin_y: 10,
            scroll_up: 0,
        }
    );
}

#[test]
fn inline_reserve_plan_scrolls_terminal_up_to_make_room() {
    assert_eq!(
        inline_reserve_plan(24, 20, 18),
        InlineReservePlan {
            origin_y: 6,
            scroll_up: 14,
        }
    );
}

#[test]
fn inline_reserve_plan_uses_the_full_terminal_height_when_it_is_short() {
    assert_eq!(
        inline_reserve_plan(12, 11, 18),
        InlineReservePlan {
            origin_y: 0,
            scroll_up: 11,
        }
    );
}

#[test]
fn preserve_visible_policy_keeps_current_cursor_origin() {
    assert_eq!(
        ViewportPolicy::PreserveVisible.resolve((80, 24), 20, TerminalCapabilities::default(),),
        Viewport {
            width: 80,
            height: 4,
            origin_y: 20,
        }
    );
}

#[test]
fn reserve_preferred_policy_uses_inline_reserve_when_supported() {
    assert_eq!(
        ViewportPolicy::ReservePreferred(18).resolve(
            (80, 24),
            20,
            TerminalCapabilities {
                supports_inline_reserve: true,
            },
        ),
        Viewport {
            width: 80,
            height: 18,
            origin_y: 6,
        }
    );
}

#[test]
fn reserve_preferred_policy_degrades_to_preserve_visible_when_reserve_is_unsupported() {
    assert_eq!(
        ViewportPolicy::ReservePreferred(18)
            .resolve((80, 24), 20, TerminalCapabilities::default(),),
        Viewport {
            width: 80,
            height: 4,
            origin_y: 20,
        }
    );
}

#[test]
fn reserve_preferred_policy_requests_more_height_for_taller_content() {
    assert_eq!(
        ViewportPolicy::ReservePreferred(18).requested_height(18, 22),
        Some(22)
    );
}

#[test]
fn preserve_visible_policy_does_not_request_growth() {
    assert_eq!(
        ViewportPolicy::PreserveVisible.requested_height(4, 22),
        None
    );
}

#[test]
fn reserve_fit_content_requests_growth_and_shrink_within_bounds() {
    let policy = ViewportPolicy::ReserveFitContent { min: 6, max: 18 };

    assert_eq!(policy.requested_height(6, 12), Some(12));
    assert_eq!(policy.requested_height(12, 4), Some(6));
    assert_eq!(policy.requested_height(12, 40), Some(18));
    assert_eq!(policy.requested_height(12, 12), None);
}

#[test]
fn reserve_fit_content_resolves_to_its_minimum_initial_height() {
    assert_eq!(
        ViewportPolicy::ReserveFitContent { min: 6, max: 18 }.resolve(
            (80, 24),
            20,
            TerminalCapabilities {
                supports_inline_reserve: true,
            },
        ),
        Viewport {
            width: 80,
            height: 6,
            origin_y: 18,
        }
    );
}

#[test]
fn fit_viewport_height_keeps_the_top_edge_when_shrinking() {
    assert_eq!(
        fit_viewport_height(
            Viewport {
                width: 80,
                height: 18,
                origin_y: 6,
            },
            24,
            7,
        ),
        Viewport {
            width: 80,
            height: 7,
            origin_y: 6,
        }
    );
}

#[test]
fn fit_viewport_height_uses_inline_reserve_when_growing() {
    assert_eq!(
        fit_viewport_height(
            Viewport {
                width: 80,
                height: 7,
                origin_y: 6,
            },
            24,
            10,
        ),
        Viewport {
            width: 80,
            height: 10,
            origin_y: 6,
        }
    );
}

#[test]
fn initial_viewport_plan_does_not_scroll_when_terminal_has_enough_space() {
    let (viewport, plan) = initial_viewport_plan(
        ViewportPolicy::ReservePreferred(18),
        (80, 24),
        5,
        TerminalCapabilities {
            supports_inline_reserve: true,
        },
    );

    assert_eq!(
        viewport,
        Viewport {
            width: 80,
            height: 18,
            origin_y: 5,
        }
    );
    assert_eq!(
        plan,
        Some(InlineReservePlan {
            origin_y: 5,
            scroll_up: 0,
        })
    );
}

#[test]
fn initial_viewport_plan_uses_terminal_height_for_bottom_launches() {
    let (viewport, plan) = initial_viewport_plan(
        ViewportPolicy::ReservePreferred(18),
        (80, 24),
        20,
        TerminalCapabilities {
            supports_inline_reserve: true,
        },
    );

    assert_eq!(
        viewport,
        Viewport {
            width: 80,
            height: 18,
            origin_y: 6,
        }
    );
    assert_eq!(
        plan,
        Some(InlineReservePlan {
            origin_y: 6,
            scroll_up: 14,
        })
    );
}

#[test]
fn reanchor_viewport_plan_preserves_expanded_height_for_reserve_fit_content() {
    let (viewport, plan) = reanchor_viewport_plan(
        ViewportPolicy::ReserveFitContent { min: 6, max: 18 },
        (80, 24),
        5,
        Viewport {
            width: 80,
            height: 14,
            origin_y: 5,
        },
        TerminalCapabilities {
            supports_inline_reserve: true,
        },
    );

    assert_eq!(
        viewport,
        Viewport {
            width: 80,
            height: 14,
            origin_y: 5,
        }
    );
    assert_eq!(
        plan,
        Some(InlineReservePlan {
            origin_y: 5,
            scroll_up: 0,
        })
    );
}

#[test]
fn reanchor_viewport_plan_resets_reserve_preferred_back_to_its_target_height_after_commit() {
    let (viewport, plan) = reanchor_viewport_plan(
        ViewportPolicy::ReservePreferred(18),
        (80, 40),
        8,
        Viewport {
            width: 80,
            height: 24,
            origin_y: 6,
        },
        TerminalCapabilities {
            supports_inline_reserve: true,
        },
    );

    assert_eq!(
        viewport,
        Viewport {
            width: 80,
            height: 18,
            origin_y: 8,
        }
    );
    assert_eq!(
        plan,
        Some(InlineReservePlan {
            origin_y: 8,
            scroll_up: 0,
        })
    );
}

#[test]
fn reanchor_viewport_plan_keeps_current_height_when_cursor_moves_near_the_bottom() {
    let (viewport, plan) = reanchor_viewport_plan(
        ViewportPolicy::ReserveFitContent { min: 6, max: 18 },
        (80, 24),
        20,
        Viewport {
            width: 80,
            height: 14,
            origin_y: 5,
        },
        TerminalCapabilities {
            supports_inline_reserve: true,
        },
    );

    assert_eq!(
        viewport,
        Viewport {
            width: 80,
            height: 14,
            origin_y: 10,
        }
    );
    assert_eq!(
        plan,
        Some(InlineReservePlan {
            origin_y: 10,
            scroll_up: 10,
        })
    );
}

#[test]
fn resize_viewport_plan_preserves_expanded_height_for_reserve_fit_content() {
    let viewport = resize_viewport_plan(
        ViewportPolicy::ReserveFitContent { min: 6, max: 18 },
        (100, 30),
        Viewport {
            width: 80,
            height: 14,
            origin_y: 5,
        },
        TerminalCapabilities {
            supports_inline_reserve: true,
        },
    );

    assert_eq!(
        viewport,
        Viewport {
            width: 100,
            height: 14,
            origin_y: 5,
        }
    );
}

#[test]
fn resize_viewport_plan_clamps_the_preserved_height_to_the_new_terminal() {
    let viewport = resize_viewport_plan(
        ViewportPolicy::ReserveFitContent { min: 6, max: 18 },
        (100, 12),
        Viewport {
            width: 80,
            height: 14,
            origin_y: 5,
        },
        TerminalCapabilities {
            supports_inline_reserve: true,
        },
    );

    assert_eq!(
        viewport,
        Viewport {
            width: 100,
            height: 12,
            origin_y: 0,
        }
    );
}

#[test]
fn cursor_y_after_history_entries_advances_by_the_rendered_row_count() {
    assert_eq!(cursor_y_after_history_entries(6, 0), 6);
    assert_eq!(cursor_y_after_history_entries(6, 1), 7);
    assert_eq!(cursor_y_after_history_entries(6, 3), 9);
}

#[test]
fn safe_exit_row_clamps_to_the_visible_terminal_height() {
    assert_eq!(safe_exit_row(6, (80, 24)), 6);
    assert_eq!(safe_exit_row(40, (80, 24)), 23);
    assert_eq!(safe_exit_row(2, (80, 0)), 2);
}

#[tokio::test]
async fn next_input_event_from_stream_returns_none_after_timeout_without_blocking() {
    let mut stream = stream::pending::<std::io::Result<Event>>();

    let started = std::time::Instant::now();
    let event = next_input_event_from_stream(&mut stream, std::time::Duration::from_millis(10))
        .await
        .expect("polling should succeed");

    assert!(started.elapsed() >= std::time::Duration::from_millis(10));
    assert_eq!(event, None);
}

#[tokio::test]
async fn next_input_event_from_stream_yields_the_next_translated_event() {
    let mut stream = stream::iter([Ok(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )))]);

    let event = next_input_event_from_stream(&mut stream, std::time::Duration::from_secs(1))
        .await
        .expect("polling should succeed");

    assert_eq!(event, Some(InputEvent::Key(Key::Enter)));
}
