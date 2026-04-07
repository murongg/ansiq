use ansiq_core::{
    Element, ElementKind, HistoryEntry, Line, ListItem, ListProps, ListState, ScrollViewProps,
    ScrollbarProps, TabsProps, TextProps, ViewCtx, reset_reactivity_for_testing,
};
use ansiq_runtime::{App, Engine, RuntimeHandle};
use ansiq_surface::Key;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

thread_local! {
    static NESTED_SIGNAL: RefCell<Option<ansiq_core::Signal<String>>> = const { RefCell::new(None) };
    static NESTED_RENDERS: Cell<u16> = const { Cell::new(0) };
    static WATCH_SOURCE: RefCell<Option<ansiq_core::Signal<u16>>> = const { RefCell::new(None) };
    static WATCH_TICKS: Cell<u16> = const { Cell::new(0) };
}

struct HookApp {
    signal: Option<ansiq_core::Signal<String>>,
}

impl App for HookApp {
    type Message = ();

    fn render(&mut self, cx: &mut ViewCtx<'_, Self::Message>) -> Element<Self::Message> {
        let value = cx.signal(|| "idle".to_string());
        self.signal = Some(value.clone());
        Element::new(ElementKind::Text(TextProps {
            content: value.get(),
        }))
    }

    fn update(&mut self, _message: Self::Message, _handle: &RuntimeHandle<Self::Message>) {}
}

#[test]
fn component_signal_updates_next_rendered_tree() {
    let mut engine = Engine::new(HookApp { signal: None });

    engine.render_tree();
    engine
        .app()
        .signal
        .as_ref()
        .unwrap()
        .set("ready".to_string());
    engine.render_tree();

    match &engine.tree().unwrap().element.kind {
        ElementKind::Text(props) => assert_eq!(props.content, "ready"),
        other => panic!("expected text, got {other:?}"),
    }
}

#[derive(Default)]
struct MountedApp {
    mounted: bool,
}

impl App for MountedApp {
    type Message = ();

    fn mount(&mut self, _handle: &RuntimeHandle<Self::Message>) {
        self.mounted = true;
    }

    fn render(&mut self, _cx: &mut ViewCtx<'_, Self::Message>) -> Element<Self::Message> {
        Element::new(ElementKind::Text(TextProps {
            content: if self.mounted {
                "mounted".to_string()
            } else {
                "idle".to_string()
            },
        }))
    }

    fn update(&mut self, _message: Self::Message, _handle: &RuntimeHandle<Self::Message>) {}
}

#[test]
fn mount_runs_before_first_render() {
    let mut engine = Engine::new(MountedApp::default());
    engine.mount();
    engine.render_tree();

    match &engine.tree().unwrap().element.kind {
        ElementKind::Text(props) => assert_eq!(props.content, "mounted"),
        other => panic!("expected text, got {other:?}"),
    }
}

struct MessageApp {
    status: String,
}

impl App for MessageApp {
    type Message = String;

    fn render(&mut self, _cx: &mut ViewCtx<'_, Self::Message>) -> Element<Self::Message> {
        Element::new(ElementKind::Text(TextProps {
            content: self.status.clone(),
        }))
    }

    fn update(&mut self, message: Self::Message, _handle: &RuntimeHandle<Self::Message>) {
        self.status = message;
    }
}

#[test]
fn emitted_messages_flow_through_update_and_rerender() {
    let mut engine = Engine::new(MessageApp {
        status: "idle".to_string(),
    });

    engine.render_tree();
    engine
        .handle()
        .emit("streaming".to_string())
        .expect("message send should succeed");
    engine.drain_requests();
    engine.render_tree();

    match &engine.tree().unwrap().element.kind {
        ElementKind::Text(props) => assert_eq!(props.content, "streaming"),
        other => panic!("expected text, got {other:?}"),
    }
}

#[test]
fn quit_request_is_reported_by_request_drain() {
    let mut engine = Engine::new(MessageApp {
        status: "idle".to_string(),
    });

    engine.render_tree();
    engine.handle().quit().expect("quit send should succeed");

    assert!(engine.drain_requests());
}

#[test]
fn committed_history_is_buffered_until_the_runtime_flushes_it() {
    let mut engine = Engine::new(MessageApp {
        status: "idle".to_string(),
    });

    engine.render_tree();
    engine
        .handle()
        .commit_history("assistant  committed chunk".to_string())
        .expect("history send should succeed");

    assert!(!engine.drain_requests());
    assert_eq!(
        engine.take_pending_history(),
        vec![HistoryEntry::Text("assistant  committed chunk".to_string())]
    );
    assert!(engine.take_pending_history().is_empty());
}

struct FocusApp;

impl App for FocusApp {
    type Message = ();

    fn render(&mut self, _cx: &mut ViewCtx<'_, Self::Message>) -> Element<Self::Message> {
        Element::new(ElementKind::Box(ansiq_core::BoxProps {
            direction: ansiq_core::Direction::Column,
            gap: 0,
        }))
        .with_children(vec![
            Element::new(ElementKind::Input(ansiq_core::InputProps {
                value: String::new(),
                placeholder: String::new(),
                on_change: None,
                on_submit: None,
                cursor: 0,
            }))
            .with_layout(ansiq_core::Layout {
                width: ansiq_core::Length::Fill,
                height: ansiq_core::Length::Fixed(3),
            })
            .with_focusable(true),
            Element::new(ElementKind::Input(ansiq_core::InputProps {
                value: String::new(),
                placeholder: String::new(),
                on_change: None,
                on_submit: None,
                cursor: 0,
            }))
            .with_layout(ansiq_core::Layout {
                width: ansiq_core::Length::Fill,
                height: ansiq_core::Length::Fixed(3),
            })
            .with_focusable(true),
        ])
    }

    fn update(&mut self, _message: Self::Message, _handle: &RuntimeHandle<Self::Message>) {}
}

#[test]
fn focus_navigation_marks_engine_dirty() {
    let mut engine = Engine::new(FocusApp);
    engine.render_tree();

    assert!(!engine.is_dirty());
    assert!(!engine.handle_input(Key::Tab));
    assert!(engine.is_dirty());
}

struct ScrollSyncApp;

impl App for ScrollSyncApp {
    type Message = ();

    fn render(&mut self, cx: &mut ViewCtx<'_, Self::Message>) -> Element<Self::Message> {
        let offset = cx.signal(|| 0usize);

        Element::new(ElementKind::Box(ansiq_core::BoxProps {
            direction: ansiq_core::Direction::Column,
            gap: 0,
        }))
        .with_children(vec![
            Element::new(ElementKind::ScrollView(ScrollViewProps {
                follow_bottom: false,
                offset: Some(offset.get()),
                on_scroll: Some(std::boxed::Box::new({
                    let offset = offset.clone();
                    move |next| {
                        offset.set(next);
                        None
                    }
                })),
            }))
            .with_layout(ansiq_core::Layout {
                width: ansiq_core::Length::Fill,
                height: ansiq_core::Length::Fixed(2),
            })
            .with_focusable(true)
            .with_children(vec![Element::new(ElementKind::Text(TextProps {
                content: "one\ntwo\nthree\nfour".to_string(),
            }))]),
            Element::new(ElementKind::Scrollbar(ScrollbarProps {
                state: ansiq_core::ScrollbarState::new(4)
                    .position(offset.get())
                    .viewport_content_length(2),
                orientation: ansiq_core::ScrollbarOrientation::VerticalRight,
                thumb_symbol: "█".to_string(),
                thumb_style: ansiq_core::Style::default(),
                track_symbol: Some("░".to_string()),
                track_style: ansiq_core::Style::default(),
                begin_symbol: Some("↑".to_string()),
                begin_style: ansiq_core::Style::default(),
                end_symbol: Some("↓".to_string()),
                end_style: ansiq_core::Style::default(),
                on_scroll: Some(std::boxed::Box::new({
                    let offset = offset.clone();
                    move |next| {
                        offset.set(next);
                        None
                    }
                })),
            }))
            .with_layout(ansiq_core::Layout {
                width: ansiq_core::Length::Fixed(1),
                height: ansiq_core::Length::Fill,
            })
            .with_focusable(true),
        ])
    }

    fn update(&mut self, _message: Self::Message, _handle: &RuntimeHandle<Self::Message>) {}
}

#[test]
fn scroll_view_and_scrollbar_can_share_the_same_scroll_signal() {
    let mut engine = Engine::new(ScrollSyncApp);
    engine.render_tree();

    assert!(!engine.handle_input(Key::Down));
    engine.render_tree();

    let tree = engine.tree().expect("tree should exist");
    match &tree.children[0].element.kind {
        ElementKind::ScrollView(props) => assert_eq!(props.offset, Some(1)),
        other => panic!("expected scroll view, got {other:?}"),
    }
    match &tree.children[1].element.kind {
        ElementKind::Scrollbar(props) => assert_eq!(props.state.get_position(), 1),
        other => panic!("expected scrollbar, got {other:?}"),
    }

    assert!(!engine.handle_input(Key::Tab));
    engine.render_tree();
    assert!(!engine.handle_input(Key::Down));
    engine.render_tree();

    let tree = engine.tree().expect("tree should exist");
    match &tree.children[0].element.kind {
        ElementKind::ScrollView(props) => assert_eq!(props.offset, Some(2)),
        other => panic!("expected scroll view, got {other:?}"),
    }
    match &tree.children[1].element.kind {
        ElementKind::Scrollbar(props) => assert_eq!(props.state.get_position(), 2),
        other => panic!("expected scrollbar, got {other:?}"),
    }
}

struct StatefulInputApp;

impl App for StatefulInputApp {
    type Message = ();

    fn render(&mut self, cx: &mut ViewCtx<'_, Self::Message>) -> Element<Self::Message> {
        let value = cx.signal(|| "hello".to_string());

        Element::new(ElementKind::Input(ansiq_core::InputProps {
            value: value.get(),
            placeholder: String::new(),
            on_change: Some(std::boxed::Box::new({
                let value = value.clone();
                move |next| value.set(next)
            })),
            on_submit: None,
            cursor: 0,
        }))
        .with_layout(ansiq_core::Layout {
            width: ansiq_core::Length::Fill,
            height: ansiq_core::Length::Fixed(3),
        })
        .with_focusable(true)
    }

    fn update(&mut self, _message: Self::Message, _handle: &RuntimeHandle<Self::Message>) {}
}

#[test]
fn rerender_preserves_input_cursor_after_mid_string_edit() {
    let mut engine = Engine::new(StatefulInputApp);
    engine.render_tree();

    assert!(!engine.handle_input(Key::Left));
    assert!(!engine.handle_input(Key::Left));
    assert!(!engine.handle_input(Key::Char('X')));
    assert!(!engine.drain_requests());
    engine.render_tree();

    let tree = engine.tree().expect("tree should exist");
    match &tree.element.kind {
        ElementKind::Input(props) => {
            assert_eq!(props.value, "helXlo");
            assert_eq!(props.cursor, 4);
        }
        other => panic!("expected input, got {other:?}"),
    }
}

struct UnhandledKeyApp {
    seen: Rc<RefCell<Vec<Key>>>,
    trap_on_mount: bool,
}

impl App for UnhandledKeyApp {
    type Message = ();

    fn mount(&mut self, handle: &RuntimeHandle<Self::Message>) {
        if self.trap_on_mount {
            handle.trap_focus_in("modal").expect("focus trap request");
        }
    }

    fn render(&mut self, _cx: &mut ViewCtx<'_, Self::Message>) -> Element<Self::Message> {
        Element::new(ElementKind::Box(ansiq_core::BoxProps {
            direction: ansiq_core::Direction::Column,
            gap: 0,
        }))
        .with_children(vec![
            Element::new(ElementKind::Box(ansiq_core::BoxProps {
                direction: ansiq_core::Direction::Column,
                gap: 0,
            }))
            .with_continuity_key("outside")
            .with_children(vec![
                Element::new(ElementKind::Input(ansiq_core::InputProps {
                    value: "outside".to_string(),
                    placeholder: String::new(),
                    on_change: None,
                    on_submit: None,
                    cursor: 0,
                }))
                .with_layout(ansiq_core::Layout {
                    width: ansiq_core::Length::Fill,
                    height: ansiq_core::Length::Fixed(3),
                })
                .with_focusable(true),
            ]),
            Element::new(ElementKind::Box(ansiq_core::BoxProps {
                direction: ansiq_core::Direction::Column,
                gap: 0,
            }))
            .with_continuity_key("modal")
            .with_children(vec![
                Element::new(ElementKind::Input(ansiq_core::InputProps {
                    value: "modal-a".to_string(),
                    placeholder: String::new(),
                    on_change: None,
                    on_submit: None,
                    cursor: 0,
                }))
                .with_layout(ansiq_core::Layout {
                    width: ansiq_core::Length::Fill,
                    height: ansiq_core::Length::Fixed(3),
                })
                .with_focusable(true),
                Element::new(ElementKind::Input(ansiq_core::InputProps {
                    value: "modal-b".to_string(),
                    placeholder: String::new(),
                    on_change: None,
                    on_submit: None,
                    cursor: 0,
                }))
                .with_layout(ansiq_core::Layout {
                    width: ansiq_core::Length::Fill,
                    height: ansiq_core::Length::Fixed(3),
                })
                .with_focusable(true),
            ]),
        ])
    }

    fn update(&mut self, _message: Self::Message, _handle: &RuntimeHandle<Self::Message>) {}

    fn on_unhandled_key(&mut self, key: Key, _handle: &RuntimeHandle<Self::Message>) -> bool {
        self.seen.borrow_mut().push(key);
        true
    }
}

#[test]
fn engine_calls_app_for_unhandled_keys() {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let mut engine = Engine::new(UnhandledKeyApp {
        seen: seen.clone(),
        trap_on_mount: false,
    });

    engine.mount();
    engine.render_tree();

    assert!(!engine.handle_input(Key::Esc));
    assert_eq!(&*seen.borrow(), &[Key::Esc]);
}

#[test]
fn focus_scope_request_traps_tab_navigation_inside_the_scoped_subtree() {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let mut engine = Engine::new(UnhandledKeyApp {
        seen,
        trap_on_mount: true,
    });

    engine.mount();
    assert!(!engine.drain_requests());
    engine.render_tree();

    let tree = engine.tree().expect("tree should exist");
    let modal_first = tree.children[1].children[0].id;
    let modal_second = tree.children[1].children[1].id;

    assert_eq!(engine.focused(), Some(modal_first));
    assert!(!engine.handle_input(Key::Tab));
    assert_eq!(engine.focused(), Some(modal_second));
    assert!(!engine.handle_input(Key::Tab));
    assert_eq!(engine.focused(), Some(modal_first));
}

struct ReactiveComponentApp {
    root_renders: Rc<Cell<u16>>,
    component_renders: Rc<Cell<u16>>,
    value: ansiq_core::Signal<String>,
}

impl App for ReactiveComponentApp {
    type Message = ();

    fn render(&mut self, _cx: &mut ViewCtx<'_, Self::Message>) -> Element<Self::Message> {
        self.root_renders.set(self.root_renders.get() + 1);

        Element::new(ElementKind::Box(ansiq_core::BoxProps {
            direction: ansiq_core::Direction::Column,
            gap: 0,
        }))
        .with_children(vec![ansiq_core::component("label", {
            let value = self.value.clone();
            let renders = self.component_renders.clone();
            move || {
                renders.set(renders.get() + 1);
                Element::new(ElementKind::Text(TextProps {
                    content: value.get(),
                }))
            }
        })])
    }

    fn update(&mut self, _message: Self::Message, _handle: &RuntimeHandle<Self::Message>) {}
}

#[test]
fn signal_changes_rerender_only_the_dirty_component_subtree() {
    reset_reactivity_for_testing();

    let value = ansiq_core::signal(String::from("idle"));
    let root_renders = Rc::new(Cell::new(0));
    let component_renders = Rc::new(Cell::new(0));
    let mut engine = Engine::new(ReactiveComponentApp {
        root_renders: root_renders.clone(),
        component_renders: component_renders.clone(),
        value: value.clone(),
    });

    engine.render_tree();
    assert_eq!(root_renders.get(), 1);
    assert_eq!(component_renders.get(), 1);

    value.set(String::from("streaming"));
    engine.render_tree();

    assert_eq!(root_renders.get(), 1);
    assert_eq!(component_renders.get(), 2);

    let tree = engine.tree().expect("tree should exist");
    let label = &tree.children[0].children[0];
    match &label.element.kind {
        ElementKind::Text(props) => assert_eq!(props.content, "streaming"),
        other => panic!("expected text child, got {other:?}"),
    }
}

struct ReactiveHookApp {
    renders: Rc<Cell<u16>>,
    signal: Option<ansiq_core::Signal<String>>,
}

impl App for ReactiveHookApp {
    type Message = ();

    fn render(&mut self, cx: &mut ViewCtx<'_, Self::Message>) -> Element<Self::Message> {
        self.renders.set(self.renders.get() + 1);
        let value = cx.signal(|| String::from("idle"));
        self.signal = Some(value.clone());

        Element::new(ElementKind::Text(TextProps {
            content: value.get(),
        }))
    }

    fn update(&mut self, _message: Self::Message, _handle: &RuntimeHandle<Self::Message>) {}
}

#[test]
fn component_signals_flow_through_reactivity_without_forcing_engine_dirty() {
    reset_reactivity_for_testing();

    let renders = Rc::new(Cell::new(0));
    let mut engine = Engine::new(ReactiveHookApp {
        renders: renders.clone(),
        signal: None,
    });

    engine.render_tree();
    assert_eq!(renders.get(), 1);
    assert!(!engine.is_dirty());

    engine
        .app()
        .signal
        .as_ref()
        .expect("signal should exist")
        .set(String::from("ready"));

    assert!(!engine.is_dirty());

    engine.render_tree();
    assert_eq!(renders.get(), 2);

    match &engine.tree().expect("tree should exist").element.kind {
        ElementKind::Text(props) => assert_eq!(props.content, "ready"),
        other => panic!("expected text, got {other:?}"),
    }
}

fn nested_label(cx: &mut ViewCtx<'_, ()>) -> Element<()> {
    NESTED_RENDERS.with(|renders| renders.set(renders.get() + 1));
    let value = cx.signal(|| String::from("idle"));
    NESTED_SIGNAL.with(|signal| *signal.borrow_mut() = Some(value.clone()));

    Element::new(ElementKind::Text(TextProps {
        content: value.get(),
    }))
}

struct NestedHookComponentApp {
    root_renders: Rc<Cell<u16>>,
}

impl App for NestedHookComponentApp {
    type Message = ();

    fn render(&mut self, _cx: &mut ViewCtx<'_, Self::Message>) -> Element<Self::Message> {
        self.root_renders.set(self.root_renders.get() + 1);

        Element::new(ElementKind::Box(ansiq_core::BoxProps {
            direction: ansiq_core::Direction::Column,
            gap: 0,
        }))
        .with_children(vec![ansiq_core::component_with_cx(
            "nested_label",
            nested_label,
        )])
    }

    fn update(&mut self, _message: Self::Message, _handle: &RuntimeHandle<Self::Message>) {}
}

#[test]
fn nested_function_component_with_cx_rerenders_without_restarting_root() {
    reset_reactivity_for_testing();
    NESTED_SIGNAL.with(|signal| *signal.borrow_mut() = None);
    NESTED_RENDERS.with(|renders| renders.set(0));

    let root_renders = Rc::new(Cell::new(0));
    let mut engine = Engine::new(NestedHookComponentApp {
        root_renders: root_renders.clone(),
    });

    engine.render_tree();
    assert_eq!(root_renders.get(), 1);
    assert_eq!(NESTED_RENDERS.with(|renders| renders.get()), 1);

    NESTED_SIGNAL.with(|signal| {
        signal
            .borrow()
            .as_ref()
            .expect("nested signal should exist")
            .set(String::from("ready"))
    });
    engine.render_tree();

    assert_eq!(root_renders.get(), 1);
    assert_eq!(NESTED_RENDERS.with(|renders| renders.get()), 2);

    let tree = engine.tree().expect("tree should exist");
    let label = &tree.children[0].children[0];
    match &label.element.kind {
        ElementKind::Text(props) => assert_eq!(props.content, "ready"),
        other => panic!("expected text, got {other:?}"),
    }
}

fn watching_child(cx: &mut ViewCtx<'_, ()>) -> Element<()> {
    cx.effect({
        let source = WATCH_SOURCE.with(|source| {
            source
                .borrow()
                .as_ref()
                .expect("effect source should be installed")
                .clone()
        });
        move || {
            let _ = source.get();
            WATCH_TICKS.with(|ticks| ticks.set(ticks.get() + 1));
        }
    });

    Element::new(ElementKind::Text(TextProps {
        content: String::from("watching"),
    }))
}

struct ConditionalWatchApp {
    show_child: ansiq_core::Signal<bool>,
}

impl App for ConditionalWatchApp {
    type Message = ();

    fn render(&mut self, _cx: &mut ViewCtx<'_, Self::Message>) -> Element<Self::Message> {
        let mut children = Vec::new();
        if self.show_child.get() {
            children.push(ansiq_core::component_with_cx(
                "watching_child",
                watching_child,
            ));
        }

        Element::new(ElementKind::Box(ansiq_core::BoxProps {
            direction: ansiq_core::Direction::Column,
            gap: 0,
        }))
        .with_children(children)
    }

    fn update(&mut self, _message: Self::Message, _handle: &RuntimeHandle<Self::Message>) {}
}

#[test]
fn unmounted_component_watchers_stop_firing_after_their_scope_is_removed() {
    reset_reactivity_for_testing();
    WATCH_TICKS.with(|ticks| ticks.set(0));

    let show_child = ansiq_core::signal(true);
    let watch_source = ansiq_core::signal(0);
    WATCH_SOURCE.with(|source| *source.borrow_mut() = Some(watch_source.clone()));

    let mut engine = Engine::new(ConditionalWatchApp {
        show_child: show_child.clone(),
    });

    engine.render_tree();
    assert_eq!(WATCH_TICKS.with(|ticks| ticks.get()), 1);

    show_child.set(false);
    engine.render_tree();
    assert_eq!(WATCH_TICKS.with(|ticks| ticks.get()), 1);

    watch_source.set(1);
    engine.render_tree();
    assert_eq!(WATCH_TICKS.with(|ticks| ticks.get()), 1);

    WATCH_SOURCE.with(|source| *source.borrow_mut() = None);
}

struct ComputedHookComponentApp {
    root_renders: Rc<Cell<u16>>,
    component_renders: Rc<Cell<u16>>,
    compute_runs: Rc<Cell<u16>>,
    value: ansiq_core::Signal<u16>,
}

impl App for ComputedHookComponentApp {
    type Message = ();

    fn render(&mut self, _cx: &mut ViewCtx<'_, Self::Message>) -> Element<Self::Message> {
        self.root_renders.set(self.root_renders.get() + 1);

        Element::new(ElementKind::Box(ansiq_core::BoxProps {
            direction: ansiq_core::Direction::Column,
            gap: 0,
        }))
        .with_children(vec![ansiq_core::component_with_cx("computed_label", {
            let value = self.value.clone();
            let component_renders = self.component_renders.clone();
            let compute_runs = self.compute_runs.clone();
            move |cx| {
                component_renders.set(component_renders.get() + 1);
                let doubled = cx.computed({
                    let value = value.clone();
                    let compute_runs = compute_runs.clone();
                    move || {
                        compute_runs.set(compute_runs.get() + 1);
                        value.get() * 2
                    }
                });

                Element::new(ElementKind::Text(TextProps {
                    content: doubled.get().to_string(),
                }))
            }
        })])
    }

    fn update(&mut self, _message: Self::Message, _handle: &RuntimeHandle<Self::Message>) {}
}

#[test]
fn computed_keeps_derived_values_inside_the_dirty_component_subtree() {
    reset_reactivity_for_testing();

    let root_renders = Rc::new(Cell::new(0));
    let component_renders = Rc::new(Cell::new(0));
    let compute_runs = Rc::new(Cell::new(0));
    let value = ansiq_core::signal(2);
    let mut engine = Engine::new(ComputedHookComponentApp {
        root_renders: root_renders.clone(),
        component_renders: component_renders.clone(),
        compute_runs: compute_runs.clone(),
        value: value.clone(),
    });

    engine.render_tree();
    assert_eq!(root_renders.get(), 1);
    assert_eq!(component_renders.get(), 1);
    assert_eq!(compute_runs.get(), 1);

    value.set(3);
    engine.render_tree();

    assert_eq!(root_renders.get(), 1);
    assert_eq!(component_renders.get(), 2);
    assert_eq!(compute_runs.get(), 2);

    let tree = engine.tree().expect("tree should exist");
    let label = &tree.children[0].children[0];
    match &label.element.kind {
        ElementKind::Text(props) => assert_eq!(props.content, "6"),
        other => panic!("expected text child, got {other:?}"),
    }
}

struct ExpandingComponentApp {
    expanded: ansiq_core::Signal<bool>,
}

impl App for ExpandingComponentApp {
    type Message = ();

    fn render(&mut self, _cx: &mut ViewCtx<'_, Self::Message>) -> Element<Self::Message> {
        Element::new(ElementKind::Box(ansiq_core::BoxProps {
            direction: ansiq_core::Direction::Column,
            gap: 0,
        }))
        .with_children(vec![
            ansiq_core::component("dynamic_panel", {
                let expanded = self.expanded.clone();
                move || {
                    if expanded.get() {
                        Element::new(ElementKind::Box(ansiq_core::BoxProps {
                            direction: ansiq_core::Direction::Column,
                            gap: 0,
                        }))
                        .with_children(vec![
                            Element::new(ElementKind::Text(TextProps {
                                content: String::from("first"),
                            })),
                            Element::new(ElementKind::Text(TextProps {
                                content: String::from("second"),
                            })),
                        ])
                    } else {
                        Element::new(ElementKind::Text(TextProps {
                            content: String::from("first"),
                        }))
                    }
                }
            }),
            Element::new(ElementKind::Input(ansiq_core::InputProps {
                value: String::from("hello"),
                placeholder: String::new(),
                on_change: None,
                on_submit: None,
                cursor: 0,
            }))
            .with_layout(ansiq_core::Layout {
                width: ansiq_core::Length::Fill,
                height: ansiq_core::Length::Fixed(3),
            })
            .with_focusable(true),
        ])
    }

    fn update(&mut self, _message: Self::Message, _handle: &RuntimeHandle<Self::Message>) {}
}

#[test]
fn dirty_component_relayout_preserves_unaffected_sibling_identity_and_cursor() {
    reset_reactivity_for_testing();

    let expanded = ansiq_core::signal(false);
    let mut engine = Engine::new(ExpandingComponentApp {
        expanded: expanded.clone(),
    });

    engine.render_tree();
    assert!(!engine.handle_input(Key::Tab));
    assert!(!engine.handle_input(Key::Left));
    assert!(!engine.handle_input(Key::Left));
    engine.render_tree();
    assert_eq!(engine.required_height(), 4);

    let (before_id, before_cursor) = {
        let tree = engine.tree().expect("tree should exist");
        let input = &tree.children[1];
        let cursor = match &input.element.kind {
            ElementKind::Input(props) => props.cursor,
            other => panic!("expected input, got {other:?}"),
        };
        (input.id, cursor)
    };
    assert_eq!(before_cursor, 3);

    expanded.set(true);
    engine.render_tree();
    assert_eq!(engine.required_height(), 5);

    let tree = engine.tree().expect("tree should exist");
    let input = &tree.children[1];
    let after_cursor = match &input.element.kind {
        ElementKind::Input(props) => props.cursor,
        other => panic!("expected input, got {other:?}"),
    };

    assert_eq!(input.id, before_id);
    assert_eq!(after_cursor, before_cursor);

    let redraw_regions = engine
        .redraw_regions()
        .expect("dirty subtree rerender should expose partial redraw regions");
    assert!(!redraw_regions.is_empty());
    assert!(
        redraw_regions
            .iter()
            .all(|rect| *rect != ansiq_core::Rect::new(0, 0, 12, 10))
    );
}

struct UniqueIdSubtreeApp {
    expanded: ansiq_core::Signal<bool>,
}

impl App for UniqueIdSubtreeApp {
    type Message = ();

    fn render(&mut self, _cx: &mut ViewCtx<'_, Self::Message>) -> Element<Self::Message> {
        Element::new(ElementKind::Box(ansiq_core::BoxProps {
            direction: ansiq_core::Direction::Column,
            gap: 0,
        }))
        .with_children(vec![
            Element::new(ElementKind::Input(ansiq_core::InputProps {
                value: String::from("root"),
                placeholder: String::new(),
                on_change: None,
                on_submit: None,
                cursor: 0,
            }))
            .with_layout(ansiq_core::Layout {
                width: ansiq_core::Length::Fill,
                height: ansiq_core::Length::Fixed(3),
            })
            .with_focusable(true),
            ansiq_core::component("dynamic_panel", {
                let expanded = self.expanded.clone();
                move || {
                    if expanded.get() {
                        Element::new(ElementKind::Box(ansiq_core::BoxProps {
                            direction: ansiq_core::Direction::Column,
                            gap: 0,
                        }))
                        .with_children(vec![
                            Element::new(ElementKind::Text(TextProps {
                                content: String::from("header"),
                            })),
                            Element::new(ElementKind::Input(ansiq_core::InputProps {
                                value: String::from("nested"),
                                placeholder: String::new(),
                                on_change: None,
                                on_submit: None,
                                cursor: 0,
                            }))
                            .with_layout(ansiq_core::Layout {
                                width: ansiq_core::Length::Fill,
                                height: ansiq_core::Length::Fixed(3),
                            })
                            .with_focusable(true),
                        ])
                    } else {
                        Element::new(ElementKind::Input(ansiq_core::InputProps {
                            value: String::from("nested"),
                            placeholder: String::new(),
                            on_change: None,
                            on_submit: None,
                            cursor: 0,
                        }))
                        .with_layout(ansiq_core::Layout {
                            width: ansiq_core::Length::Fill,
                            height: ansiq_core::Length::Fixed(3),
                        })
                        .with_focusable(true)
                    }
                }
            }),
        ])
    }

    fn update(&mut self, _message: Self::Message, _handle: &RuntimeHandle<Self::Message>) {}
}

#[test]
fn dirty_subtree_replacement_keeps_node_ids_globally_unique() {
    reset_reactivity_for_testing();

    let expanded = ansiq_core::signal(false);
    let mut engine = Engine::new(UniqueIdSubtreeApp {
        expanded: expanded.clone(),
    });

    engine.render_tree();
    expanded.set(true);
    engine.render_tree();

    let tree = engine.tree().expect("tree should exist");
    let mut ids = Vec::new();
    collect_node_ids(tree, &mut ids);
    let unique = ids
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(
        ids.len(),
        unique.len(),
        "node ids should remain globally unique after subtree replacement"
    );
}

struct FillContainerDamageApp {
    expanded: ansiq_core::Signal<bool>,
}

impl App for FillContainerDamageApp {
    type Message = ();

    fn render(&mut self, _cx: &mut ViewCtx<'_, Self::Message>) -> Element<Self::Message> {
        Element::new(ElementKind::Box(ansiq_core::BoxProps {
            direction: ansiq_core::Direction::Column,
            gap: 0,
        }))
        .with_children(vec![
            ansiq_core::component("dynamic_header", {
                let expanded = self.expanded.clone();
                move || {
                    Element::new(ElementKind::Box(ansiq_core::BoxProps {
                        direction: ansiq_core::Direction::Column,
                        gap: 0,
                    }))
                    .with_children(if expanded.get() {
                        vec![
                            Element::new(ElementKind::Text(TextProps {
                                content: String::from("header"),
                            })),
                            Element::new(ElementKind::Text(TextProps {
                                content: String::from("details"),
                            })),
                        ]
                    } else {
                        vec![Element::new(ElementKind::Text(TextProps {
                            content: String::from("header"),
                        }))]
                    })
                }
            }),
            Element::new(ElementKind::Box(ansiq_core::BoxProps {
                direction: ansiq_core::Direction::Column,
                gap: 0,
            }))
            .with_layout(ansiq_core::Layout {
                width: ansiq_core::Length::Fill,
                height: ansiq_core::Length::Fill,
            })
            .with_children(vec![Element::new(ElementKind::Text(TextProps {
                content: String::from("body"),
            }))]),
        ])
    }

    fn update(&mut self, _message: Self::Message, _handle: &RuntimeHandle<Self::Message>) {}
}

#[test]
fn partial_rerender_does_not_invalidate_entire_layout_only_fill_container() {
    reset_reactivity_for_testing();

    let expanded = ansiq_core::signal(false);
    let mut engine = Engine::new(FillContainerDamageApp {
        expanded: expanded.clone(),
    });

    engine.set_bounds(ansiq_core::Rect::new(0, 0, 12, 10));
    engine.render_tree();

    expanded.set(true);
    engine.render_tree();

    let redraw_regions = engine
        .redraw_regions()
        .expect("partial rerender should expose redraw regions");

    assert!(
        !redraw_regions.contains(&ansiq_core::Rect::new(0, 0, 12, 10)),
        "layout-only fill container shifts should not expand damage to the full viewport"
    );
    assert!(
        !redraw_regions.contains(&ansiq_core::Rect::new(0, 1, 12, 9)),
        "layout-only fill container should not invalidate its whole old rect"
    );
    assert!(
        redraw_regions
            .iter()
            .any(|rect| rect.intersection(ansiq_core::Rect::new(0, 2, 12, 1))
                == Some(ansiq_core::Rect::new(0, 2, 12, 1))),
        "shifted child content should still invalidate its new row"
    );
}

struct FocusPreservingSubtreeApp {
    expanded: ansiq_core::Signal<bool>,
}

impl App for FocusPreservingSubtreeApp {
    type Message = ();

    fn render(&mut self, _cx: &mut ViewCtx<'_, Self::Message>) -> Element<Self::Message> {
        Element::new(ElementKind::Box(ansiq_core::BoxProps {
            direction: ansiq_core::Direction::Column,
            gap: 0,
        }))
        .with_children(vec![
            Element::new(ElementKind::Input(ansiq_core::InputProps {
                value: String::from("root"),
                placeholder: String::new(),
                on_change: None,
                on_submit: None,
                cursor: 0,
            }))
            .with_layout(ansiq_core::Layout {
                width: ansiq_core::Length::Fill,
                height: ansiq_core::Length::Fixed(3),
            })
            .with_focusable(true),
            ansiq_core::component("editor_panel", {
                let expanded = self.expanded.clone();
                move || {
                    let mut children = Vec::new();
                    if expanded.get() {
                        children.push(Element::new(ElementKind::Text(TextProps {
                            content: String::from("context"),
                        })));
                    }
                    children.push(
                        Element::new(ElementKind::Input(ansiq_core::InputProps {
                            value: String::from("hello"),
                            placeholder: String::new(),
                            on_change: None,
                            on_submit: None,
                            cursor: 0,
                        }))
                        .with_layout(ansiq_core::Layout {
                            width: ansiq_core::Length::Fill,
                            height: ansiq_core::Length::Fixed(3),
                        })
                        .with_focusable(true),
                    );
                    Element::new(ElementKind::Box(ansiq_core::BoxProps {
                        direction: ansiq_core::Direction::Column,
                        gap: 0,
                    }))
                    .with_children(children)
                }
            }),
        ])
    }

    fn update(&mut self, _message: Self::Message, _handle: &RuntimeHandle<Self::Message>) {}
}

#[test]
fn dirty_subtree_replacement_preserves_focus_and_cursor_inside_the_replaced_component() {
    reset_reactivity_for_testing();

    let expanded = ansiq_core::signal(false);
    let mut engine = Engine::new(FocusPreservingSubtreeApp {
        expanded: expanded.clone(),
    });

    engine.render_tree();
    assert!(!engine.handle_input(Key::Tab));
    assert!(!engine.handle_input(Key::Left));
    assert!(!engine.handle_input(Key::Left));
    engine.render_tree();

    let focused_before = engine.focused().expect("nested input should be focused");
    let nested_cursor_before = {
        let tree = engine.tree().expect("tree should exist");
        let nested_input = &tree.children[1].children[0].children[0];
        match &nested_input.element.kind {
            ElementKind::Input(props) => props.cursor,
            other => panic!("expected nested input, got {other:?}"),
        }
    };
    assert_eq!(nested_cursor_before, 3);

    expanded.set(true);
    engine.render_tree();

    let focused_after = engine.focused().expect("focus should still exist");
    assert_ne!(
        focused_before, focused_after,
        "replaced subtree should get fresh node ids"
    );

    let tree = engine.tree().expect("tree should exist");
    let root_input = &tree.children[0];
    assert_ne!(
        root_input.id, focused_after,
        "focus should not jump back to the unaffected sibling"
    );

    let nested_input = &tree.children[1].children[0].children[1];
    assert_eq!(
        nested_input.id, focused_after,
        "focus should stay on the semantically same nested input"
    );
    match &nested_input.element.kind {
        ElementKind::Input(props) => assert_eq!(props.cursor, nested_cursor_before),
        other => panic!("expected nested input, got {other:?}"),
    }
}

struct KeyedContinuitySubtreeApp {
    remove_first: ansiq_core::Signal<bool>,
}

impl App for KeyedContinuitySubtreeApp {
    type Message = ();

    fn render(&mut self, _cx: &mut ViewCtx<'_, Self::Message>) -> Element<Self::Message> {
        Element::new(ElementKind::Box(ansiq_core::BoxProps {
            direction: ansiq_core::Direction::Column,
            gap: 0,
        }))
        .with_children(vec![ansiq_core::component("editor_panel", {
            let remove_first = self.remove_first.clone();
            move || {
                let mut children = Vec::new();
                if !remove_first.get() {
                    children.push(
                        Element::new(ElementKind::Input(ansiq_core::InputProps {
                            value: String::from("primary"),
                            placeholder: String::new(),
                            on_change: None,
                            on_submit: None,
                            cursor: 0,
                        }))
                        .with_continuity_key("primary")
                        .with_layout(ansiq_core::Layout {
                            width: ansiq_core::Length::Fill,
                            height: ansiq_core::Length::Fixed(3),
                        })
                        .with_focusable(true),
                    );
                }
                children.push(
                    Element::new(ElementKind::Input(ansiq_core::InputProps {
                        value: String::from("secondary"),
                        placeholder: String::new(),
                        on_change: None,
                        on_submit: None,
                        cursor: if remove_first.get() { 0 } else { 2 },
                    }))
                    .with_continuity_key("secondary")
                    .with_layout(ansiq_core::Layout {
                        width: ansiq_core::Length::Fill,
                        height: ansiq_core::Length::Fixed(3),
                    })
                    .with_focusable(true),
                );
                children.push(
                    Element::new(ElementKind::Input(ansiq_core::InputProps {
                        value: String::from("tertiary"),
                        placeholder: String::new(),
                        on_change: None,
                        on_submit: None,
                        cursor: 0,
                    }))
                    .with_continuity_key("tertiary")
                    .with_layout(ansiq_core::Layout {
                        width: ansiq_core::Length::Fill,
                        height: ansiq_core::Length::Fixed(3),
                    })
                    .with_focusable(true),
                );
                Element::new(ElementKind::Box(ansiq_core::BoxProps {
                    direction: ansiq_core::Direction::Column,
                    gap: 0,
                }))
                .with_children(children)
            }
        })])
    }

    fn update(&mut self, _message: Self::Message, _handle: &RuntimeHandle<Self::Message>) {}
}

#[test]
fn dirty_subtree_replacement_uses_continuity_keys_to_preserve_input_cursor() {
    reset_reactivity_for_testing();

    let remove_first = ansiq_core::signal(false);
    let mut engine = Engine::new(KeyedContinuitySubtreeApp {
        remove_first: remove_first.clone(),
    });

    engine.render_tree();

    let secondary_cursor_before = {
        let tree = engine.tree().expect("tree should exist");
        let secondary_input = &tree.children[0].children[0].children[1];
        match &secondary_input.element.kind {
            ElementKind::Input(props) => props.cursor,
            other => panic!("expected secondary input, got {other:?}"),
        }
    };
    assert_eq!(secondary_cursor_before, 2);

    remove_first.set(true);
    engine.render_tree();

    let tree = engine.tree().expect("tree should exist");
    let remaining_input = &tree.children[0].children[0].children[0];
    match &remaining_input.element.kind {
        ElementKind::Input(props) => assert_eq!(props.cursor, secondary_cursor_before),
        other => panic!("expected remaining input, got {other:?}"),
    }
}

#[test]
fn dirty_subtree_replacement_uses_continuity_keys_to_preserve_focus_across_reordering() {
    reset_reactivity_for_testing();

    let remove_first = ansiq_core::signal(false);
    let mut engine = Engine::new(KeyedContinuitySubtreeApp {
        remove_first: remove_first.clone(),
    });

    engine.render_tree();
    assert!(!engine.handle_input(Key::Tab));
    engine.render_tree();

    let focused_before = engine.focused().expect("secondary input should be focused");
    let focused_before_value = {
        let tree = engine.tree().expect("tree should exist");
        let secondary_input = &tree.children[0].children[0].children[1];
        assert_eq!(secondary_input.id, focused_before);
        match &secondary_input.element.kind {
            ElementKind::Input(props) => props.value.clone(),
            other => panic!("expected secondary input, got {other:?}"),
        }
    };
    assert_eq!(focused_before_value, "secondary");

    remove_first.set(true);
    engine.render_tree();

    let focused_after = engine.focused().expect("focus should still exist");
    let tree = engine.tree().expect("tree should exist");
    let remaining_first = &tree.children[0].children[0].children[0];
    let remaining_second = &tree.children[0].children[0].children[1];
    assert_eq!(remaining_first.id, focused_after);
    match &remaining_first.element.kind {
        ElementKind::Input(props) => assert_eq!(props.value, "secondary"),
        other => panic!("expected secondary input, got {other:?}"),
    }
    match &remaining_second.element.kind {
        ElementKind::Input(props) => assert_eq!(props.value, "tertiary"),
        other => panic!("expected tertiary input, got {other:?}"),
    }
}

struct FocusFallbackSubtreeApp {
    show_nested_input: ansiq_core::Signal<bool>,
}

impl App for FocusFallbackSubtreeApp {
    type Message = ();

    fn render(&mut self, _cx: &mut ViewCtx<'_, Self::Message>) -> Element<Self::Message> {
        Element::new(ElementKind::Box(ansiq_core::BoxProps {
            direction: ansiq_core::Direction::Column,
            gap: 0,
        }))
        .with_children(vec![
            Element::new(ElementKind::Input(ansiq_core::InputProps {
                value: String::from("root"),
                placeholder: String::new(),
                on_change: None,
                on_submit: None,
                cursor: 0,
            }))
            .with_layout(ansiq_core::Layout {
                width: ansiq_core::Length::Fill,
                height: ansiq_core::Length::Fixed(3),
            })
            .with_focusable(true),
            ansiq_core::component("maybe_editor", {
                let show_nested_input = self.show_nested_input.clone();
                move || {
                    if show_nested_input.get() {
                        Element::new(ElementKind::Input(ansiq_core::InputProps {
                            value: String::from("nested"),
                            placeholder: String::new(),
                            on_change: None,
                            on_submit: None,
                            cursor: 0,
                        }))
                        .with_layout(ansiq_core::Layout {
                            width: ansiq_core::Length::Fill,
                            height: ansiq_core::Length::Fixed(3),
                        })
                        .with_focusable(true)
                    } else {
                        Element::new(ElementKind::Text(TextProps {
                            content: String::from("done"),
                        }))
                    }
                }
            }),
        ])
    }

    fn update(&mut self, _message: Self::Message, _handle: &RuntimeHandle<Self::Message>) {}
}

#[test]
fn focus_fallback_after_subtree_replacement_invalidates_the_newly_focused_region() {
    reset_reactivity_for_testing();

    let show_nested_input = ansiq_core::signal(true);
    let mut engine = Engine::new(FocusFallbackSubtreeApp {
        show_nested_input: show_nested_input.clone(),
    });

    engine.render_tree();
    assert!(!engine.handle_input(Key::Tab));
    engine.render_tree();

    let root_input_rect = {
        let tree = engine.tree().expect("tree should exist");
        tree.children[0].rect
    };

    show_nested_input.set(false);
    engine.render_tree();

    let tree = engine.tree().expect("tree should exist");
    assert_eq!(engine.focused(), Some(tree.children[0].id));

    let redraw_regions = engine
        .redraw_regions()
        .expect("partial rerender should expose redraw regions");
    assert!(
        redraw_regions
            .iter()
            .any(|rect| rect.intersection(root_input_rect) == Some(root_input_rect)),
        "newly focused sibling must be redrawn when focus falls back after subtree replacement"
    );
}

struct KeyedRootRerenderApp {
    remove_first: bool,
}

impl App for KeyedRootRerenderApp {
    type Message = bool;

    fn render(&mut self, _cx: &mut ViewCtx<'_, Self::Message>) -> Element<Self::Message> {
        let mut children = Vec::new();
        if !self.remove_first {
            children.push(
                Element::new(ElementKind::Input(ansiq_core::InputProps {
                    value: String::from("primary"),
                    placeholder: String::new(),
                    on_change: None,
                    on_submit: None,
                    cursor: 0,
                }))
                .with_continuity_key("primary")
                .with_layout(ansiq_core::Layout {
                    width: ansiq_core::Length::Fill,
                    height: ansiq_core::Length::Fixed(3),
                })
                .with_focusable(true),
            );
        }
        children.push(
            Element::new(ElementKind::Input(ansiq_core::InputProps {
                value: String::from("secondary"),
                placeholder: String::new(),
                on_change: None,
                on_submit: None,
                cursor: if self.remove_first { 0 } else { 2 },
            }))
            .with_continuity_key("secondary")
            .with_layout(ansiq_core::Layout {
                width: ansiq_core::Length::Fill,
                height: ansiq_core::Length::Fixed(3),
            })
            .with_focusable(true),
        );
        children.push(
            Element::new(ElementKind::Input(ansiq_core::InputProps {
                value: String::from("tertiary"),
                placeholder: String::new(),
                on_change: None,
                on_submit: None,
                cursor: 0,
            }))
            .with_continuity_key("tertiary")
            .with_layout(ansiq_core::Layout {
                width: ansiq_core::Length::Fill,
                height: ansiq_core::Length::Fixed(3),
            })
            .with_focusable(true),
        );

        Element::new(ElementKind::Box(ansiq_core::BoxProps {
            direction: ansiq_core::Direction::Column,
            gap: 0,
        }))
        .with_children(children)
    }

    fn update(&mut self, message: Self::Message, _handle: &RuntimeHandle<Self::Message>) {
        self.remove_first = message;
    }
}

#[test]
fn full_root_rerender_uses_continuity_keys_to_preserve_focus_and_cursor() {
    reset_reactivity_for_testing();

    let mut engine = Engine::new(KeyedRootRerenderApp {
        remove_first: false,
    });

    engine.render_tree();
    assert!(!engine.handle_input(Key::Tab));

    let focused_before = engine.focused().expect("secondary input should be focused");
    let secondary_cursor_before = {
        let tree = engine.tree().expect("tree should exist");
        let secondary_input = &tree.children[1];
        assert_eq!(secondary_input.id, focused_before);
        match &secondary_input.element.kind {
            ElementKind::Input(props) => props.cursor,
            other => panic!("expected secondary input, got {other:?}"),
        }
    };
    assert_eq!(secondary_cursor_before, 2);

    engine
        .handle()
        .emit(true)
        .expect("message send should succeed");
    engine.drain_requests();
    engine.render_tree();

    let focused_after = engine.focused().expect("focus should still exist");
    let tree = engine.tree().expect("tree should exist");
    let remaining_first = &tree.children[0];
    let remaining_second = &tree.children[1];
    assert_eq!(remaining_first.id, focused_after);
    match &remaining_first.element.kind {
        ElementKind::Input(props) => {
            assert_eq!(props.value, "secondary");
            assert_eq!(props.cursor, secondary_cursor_before);
        }
        other => panic!("expected secondary input, got {other:?}"),
    }
    match &remaining_second.element.kind {
        ElementKind::Input(props) => assert_eq!(props.value, "tertiary"),
        other => panic!("expected tertiary input, got {other:?}"),
    }
}

struct UncontrolledListStateApp {
    tick: ansiq_core::Signal<u16>,
}

impl App for UncontrolledListStateApp {
    type Message = ();

    fn render(&mut self, _cx: &mut ViewCtx<'_, Self::Message>) -> Element<Self::Message> {
        Element::new(ElementKind::Box(ansiq_core::BoxProps {
            direction: ansiq_core::Direction::Column,
            gap: 0,
        }))
        .with_children(vec![ansiq_core::component("list_panel", {
            let tick = self.tick.clone();
            move || {
                Element::new(ElementKind::Box(ansiq_core::BoxProps {
                    direction: ansiq_core::Direction::Column,
                    gap: 0,
                }))
                .with_children(vec![
                    Element::new(ElementKind::List(ListProps {
                        block: None,
                        items: vec![
                            ListItem::new("one"),
                            ListItem::new("two"),
                            ListItem::new("three"),
                        ],
                        state: ListState::default(),
                        highlight_symbol: Some(Line::from("> ")),
                        highlight_style: ansiq_core::Style::default(),
                        highlight_spacing: ansiq_core::HighlightSpacing::WhenSelected,
                        repeat_highlight_symbol: false,
                        direction: ansiq_core::ListDirection::TopToBottom,
                        scroll_padding: 0,
                        on_select: None,
                    }))
                    .with_layout(ansiq_core::Layout {
                        width: ansiq_core::Length::Fill,
                        height: ansiq_core::Length::Fixed(3),
                    })
                    .with_focusable(true),
                    Element::new(ElementKind::Text(TextProps {
                        content: format!("tick {}", tick.get()),
                    })),
                ])
            }
        })])
    }

    fn update(&mut self, _message: Self::Message, _handle: &RuntimeHandle<Self::Message>) {}
}

#[test]
fn dirty_subtree_replacement_preserves_uncontrolled_list_selection() {
    reset_reactivity_for_testing();

    let tick = ansiq_core::signal(0);
    let mut engine = Engine::new(UncontrolledListStateApp { tick: tick.clone() });

    engine.render_tree();
    assert!(!engine.handle_input(Key::Down));
    assert!(!engine.handle_input(Key::Down));
    engine.render_tree();

    let selected_before = {
        let tree = engine.tree().expect("tree should exist");
        let list = find_first_list(tree).expect("list should exist");
        match &list.element.kind {
            ElementKind::List(props) => props.state.selected(),
            other => panic!("expected list, got {other:?}"),
        }
    };
    assert_eq!(selected_before, Some(2));

    tick.set(1);
    engine.render_tree();

    let tree = engine.tree().expect("tree should exist");
    let list = find_first_list(tree).expect("list should exist");
    match &list.element.kind {
        ElementKind::List(props) => assert_eq!(props.state.selected(), selected_before),
        other => panic!("expected list, got {other:?}"),
    }
}

struct UncontrolledTabsStateApp {
    tick: ansiq_core::Signal<u16>,
}

impl App for UncontrolledTabsStateApp {
    type Message = ();

    fn render(&mut self, _cx: &mut ViewCtx<'_, Self::Message>) -> Element<Self::Message> {
        Element::new(ElementKind::Box(ansiq_core::BoxProps {
            direction: ansiq_core::Direction::Column,
            gap: 0,
        }))
        .with_children(vec![ansiq_core::component("tabs_panel", {
            let tick = self.tick.clone();
            move || {
                Element::new(ElementKind::Box(ansiq_core::BoxProps {
                    direction: ansiq_core::Direction::Column,
                    gap: 0,
                }))
                .with_children(vec![
                    Element::new(ElementKind::Tabs(TabsProps {
                        block: None,
                        titles: vec!["CPU".into(), "Memory".into(), "Disk".into()],
                        selected: Some(0),
                        selection_explicit: false,
                        highlight_style: ansiq_core::Style::default(),
                        divider: "|".into(),
                        padding_left: " ".into(),
                        padding_right: " ".into(),
                        on_select: None,
                    }))
                    .with_layout(ansiq_core::Layout {
                        width: ansiq_core::Length::Fill,
                        height: ansiq_core::Length::Fixed(1),
                    })
                    .with_focusable(true),
                    Element::new(ElementKind::Text(TextProps {
                        content: format!("tick {}", tick.get()),
                    })),
                ])
            }
        })])
    }

    fn update(&mut self, _message: Self::Message, _handle: &RuntimeHandle<Self::Message>) {}
}

#[test]
fn dirty_subtree_replacement_preserves_uncontrolled_tabs_selection() {
    reset_reactivity_for_testing();

    let tick = ansiq_core::signal(0);
    let mut engine = Engine::new(UncontrolledTabsStateApp { tick: tick.clone() });

    engine.render_tree();
    assert!(!engine.handle_input(Key::Right));
    assert!(!engine.handle_input(Key::Right));
    engine.render_tree();

    let selected_before = {
        let tree = engine.tree().expect("tree should exist");
        let tabs = find_first_tabs(tree).expect("tabs should exist");
        match &tabs.element.kind {
            ElementKind::Tabs(props) => props.selected,
            other => panic!("expected tabs, got {other:?}"),
        }
    };
    assert_eq!(selected_before, Some(2));

    tick.set(1);
    engine.render_tree();

    let tree = engine.tree().expect("tree should exist");
    let tabs = find_first_tabs(tree).expect("tabs should exist");
    match &tabs.element.kind {
        ElementKind::Tabs(props) => assert_eq!(props.selected, selected_before),
        other => panic!("expected tabs, got {other:?}"),
    }
}

struct UncontrolledScrollViewStateApp {
    tick: ansiq_core::Signal<u16>,
}

impl App for UncontrolledScrollViewStateApp {
    type Message = ();

    fn render(&mut self, _cx: &mut ViewCtx<'_, Self::Message>) -> Element<Self::Message> {
        Element::new(ElementKind::Box(ansiq_core::BoxProps {
            direction: ansiq_core::Direction::Column,
            gap: 0,
        }))
        .with_children(vec![ansiq_core::component("scroll_panel", {
            let tick = self.tick.clone();
            move || {
                Element::new(ElementKind::Box(ansiq_core::BoxProps {
                    direction: ansiq_core::Direction::Column,
                    gap: 0,
                }))
                .with_children(vec![
                    Element::new(ElementKind::ScrollView(ScrollViewProps {
                        follow_bottom: false,
                        offset: None,
                        on_scroll: None,
                    }))
                    .with_layout(ansiq_core::Layout {
                        width: ansiq_core::Length::Fill,
                        height: ansiq_core::Length::Fixed(2),
                    })
                    .with_focusable(true)
                    .with_children(vec![Element::new(ElementKind::Text(TextProps {
                        content: String::from("one\ntwo\nthree\nfour"),
                    }))]),
                    Element::new(ElementKind::Text(TextProps {
                        content: format!("tick {}", tick.get()),
                    })),
                ])
            }
        })])
    }

    fn update(&mut self, _message: Self::Message, _handle: &RuntimeHandle<Self::Message>) {}
}

#[test]
fn dirty_subtree_replacement_preserves_uncontrolled_scroll_offset() {
    reset_reactivity_for_testing();

    let tick = ansiq_core::signal(0);
    let mut engine = Engine::new(UncontrolledScrollViewStateApp { tick: tick.clone() });

    engine.render_tree();
    assert!(!engine.handle_input(Key::Down));
    assert!(!engine.handle_input(Key::Down));
    engine.render_tree();

    let offset_before = {
        let tree = engine.tree().expect("tree should exist");
        let scroll_view = find_first_scroll_view(tree).expect("scroll view should exist");
        match &scroll_view.element.kind {
            ElementKind::ScrollView(props) => props.offset,
            other => panic!("expected scroll view, got {other:?}"),
        }
    };
    assert_eq!(offset_before, Some(2));

    tick.set(1);
    engine.render_tree();

    let tree = engine.tree().expect("tree should exist");
    let scroll_view = find_first_scroll_view(tree).expect("scroll view should exist");
    match &scroll_view.element.kind {
        ElementKind::ScrollView(props) => assert_eq!(props.offset, Some(2)),
        other => panic!("expected scroll view, got {other:?}"),
    }
}

fn collect_node_ids<Message>(node: &ansiq_core::Node<Message>, output: &mut Vec<usize>) {
    output.push(node.id);
    for child in &node.children {
        collect_node_ids(child, output);
    }
}

fn find_first_list<Message>(
    node: &ansiq_core::Node<Message>,
) -> Option<&ansiq_core::Node<Message>> {
    if matches!(node.element.kind, ElementKind::List(_)) {
        return Some(node);
    }
    for child in &node.children {
        if let Some(found) = find_first_list(child) {
            return Some(found);
        }
    }
    None
}

fn find_first_scroll_view<Message>(
    node: &ansiq_core::Node<Message>,
) -> Option<&ansiq_core::Node<Message>> {
    if matches!(node.element.kind, ElementKind::ScrollView(_)) {
        return Some(node);
    }
    for child in &node.children {
        if let Some(found) = find_first_scroll_view(child) {
            return Some(found);
        }
    }
    None
}

fn find_first_tabs<Message>(
    node: &ansiq_core::Node<Message>,
) -> Option<&ansiq_core::Node<Message>> {
    if matches!(node.element.kind, ElementKind::Tabs(_)) {
        return Some(node);
    }
    for child in &node.children {
        if let Some(found) = find_first_tabs(child) {
            return Some(found);
        }
    }
    None
}
