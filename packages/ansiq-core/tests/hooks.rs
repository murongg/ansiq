use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use ansiq_core::{Cx, HookStore, ViewCtx, flush_reactivity, reset_reactivity_for_testing, signal};

#[test]
fn cx_alias_exposes_the_signal_first_component_api() {
    let mut store = HookStore::default();

    store.begin_render();
    let mut cx: Cx<'_, ()> = ViewCtx::new(&mut store);
    let state = cx.signal(|| String::from("ansiq"));
    cx.effect({
        let state = state.clone();
        move || {
            let _ = state.get();
        }
    });
    let derived = cx.computed({
        let state = state.clone();
        move || format!("{}!", state.get())
    });
    store.finish_render();

    assert_eq!(state.get(), "ansiq");
    assert_eq!(derived.get(), "ansiq!");
}

#[test]
fn cx_signal_returns_a_stable_local_signal_handle_across_rerenders() {
    reset_reactivity_for_testing();
    let mut store = HookStore::default();

    store.begin_render();
    let first = {
        let mut cx: ViewCtx<'_, ()> = ViewCtx::new(&mut store);
        cx.signal(|| String::from("idle"))
    };
    store.finish_render();

    first.set(String::from("ready"));

    store.begin_render();
    let second = {
        let mut cx: ViewCtx<'_, ()> = ViewCtx::new(&mut store);
        cx.signal(|| String::from("ignored"))
    };
    store.finish_render();

    assert_eq!(second.get(), "ready");
}

#[test]
fn cx_effect_reacts_to_signal_changes_without_stacking_duplicate_effects() {
    reset_reactivity_for_testing();
    let observed = signal(String::from("idle"));
    let seen = Rc::new(RefCell::new(Vec::new()));
    let mut store = HookStore::default();

    store.begin_render();
    {
        let mut cx: ViewCtx<'_, ()> = ViewCtx::new(&mut store);
        cx.effect({
            let observed = observed.clone();
            let seen = seen.clone();
            move || seen.borrow_mut().push(observed.get())
        });
    }
    store.finish_render();

    assert_eq!(seen.borrow().as_slice(), &["idle"]);

    observed.set(String::from("streaming"));
    flush_reactivity();
    assert_eq!(seen.borrow().as_slice(), &["idle", "streaming"]);

    store.begin_render();
    {
        let mut cx: ViewCtx<'_, ()> = ViewCtx::new(&mut store);
        cx.effect({
            let observed = observed.clone();
            let seen = seen.clone();
            move || seen.borrow_mut().push(observed.get())
        });
    }
    store.finish_render();

    observed.set(String::from("ready"));
    flush_reactivity();
    assert_eq!(seen.borrow().as_slice(), &["idle", "streaming", "ready"]);
}

#[test]
fn dropped_effect_slots_stop_running_after_the_component_stops_using_them() {
    reset_reactivity_for_testing();
    let observed = signal(0);
    let seen = Rc::new(RefCell::new(Vec::new()));
    let mut store = HookStore::default();

    store.begin_render();
    {
        let mut cx: ViewCtx<'_, ()> = ViewCtx::new(&mut store);
        cx.effect({
            let observed = observed.clone();
            let seen = seen.clone();
            move || seen.borrow_mut().push(observed.get())
        });
    }
    store.finish_render();
    assert_eq!(seen.borrow().as_slice(), &[0]);

    store.begin_render();
    drop(ViewCtx::<()>::new(&mut store));
    store.finish_render();

    observed.set(1);
    flush_reactivity();
    assert_eq!(seen.borrow().as_slice(), &[0]);
}

#[test]
fn cx_computed_reuses_a_stable_cached_handle_across_rerenders() {
    reset_reactivity_for_testing();
    let source = signal(1);
    let runs = Rc::new(Cell::new(0));
    let mut store = HookStore::default();

    store.begin_render();
    let first = {
        let mut cx: ViewCtx<'_, ()> = ViewCtx::new(&mut store);
        cx.computed({
            let source = source.clone();
            let runs = runs.clone();
            move || {
                runs.set(runs.get() + 1);
                source.get() * 2
            }
        })
    };
    store.finish_render();

    assert_eq!(first.get(), 2);
    assert_eq!(first.get(), 2);
    assert_eq!(runs.get(), 1);

    store.begin_render();
    let second = {
        let mut cx: ViewCtx<'_, ()> = ViewCtx::new(&mut store);
        cx.computed({
            let source = source.clone();
            let runs = runs.clone();
            move || {
                runs.set(runs.get() + 1);
                source.get() * 2
            }
        })
    };
    store.finish_render();

    assert_eq!(second.get(), 2);
    assert_eq!(runs.get(), 1);

    source.set(2);
    assert_eq!(second.get(), 4);
    assert_eq!(runs.get(), 2);
}
