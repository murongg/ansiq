use std::cell::{Cell, RefCell};
use std::rc::Rc;

use ansiq_core::{
    computed, dispose_component_scope, effect, flush_reactivity, render_in_component_scope,
    reset_reactivity_for_testing, signal, take_dirty_component_scopes,
};

#[test]
fn signal_returns_the_latest_value_after_set_and_update() {
    reset_reactivity_for_testing();
    let count = signal(0);

    assert_eq!(count.get(), 0);
    count.set(1);
    count.update(|value| *value += 1);
    flush_reactivity();

    assert_eq!(count.get(), 2);
}

#[test]
fn computed_caches_until_a_dependency_changes() {
    reset_reactivity_for_testing();
    let count = signal(1);
    let runs = Rc::new(Cell::new(0));

    let doubled = computed({
        let count = count.clone();
        let runs = runs.clone();
        move || {
            runs.set(runs.get() + 1);
            count.get() * 2
        }
    });

    assert_eq!(doubled.get(), 2);
    assert_eq!(doubled.get(), 2);
    assert_eq!(runs.get(), 1);

    count.set(2);

    assert_eq!(doubled.get(), 4);
    assert_eq!(runs.get(), 2);
}

#[test]
fn effect_reruns_on_flush_after_its_dependency_changes() {
    reset_reactivity_for_testing();
    let query = signal(String::from("a"));
    let seen = Rc::new(RefCell::new(Vec::new()));

    let _effect = effect({
        let query = query.clone();
        let seen = seen.clone();
        move || seen.borrow_mut().push(query.get())
    });

    assert_eq!(seen.borrow().as_slice(), &["a"]);

    query.set(String::from("b"));
    assert_eq!(seen.borrow().as_slice(), &["a"]);

    flush_reactivity();
    assert_eq!(seen.borrow().as_slice(), &["a", "b"]);
}

#[test]
fn computed_recollects_branching_dependencies() {
    reset_reactivity_for_testing();
    let use_left = signal(true);
    let left = signal(1);
    let right = signal(10);
    let runs = Rc::new(Cell::new(0));

    let value = computed({
        let use_left = use_left.clone();
        let left = left.clone();
        let right = right.clone();
        let runs = runs.clone();
        move || {
            runs.set(runs.get() + 1);
            if use_left.get() {
                left.get()
            } else {
                right.get()
            }
        }
    });

    assert_eq!(value.get(), 1);
    use_left.set(false);
    assert_eq!(value.get(), 10);
    left.set(2);
    assert_eq!(value.get(), 10);
    right.set(11);
    assert_eq!(value.get(), 11);
}

#[test]
fn chained_computed_values_invalidate_downstream_caches() {
    reset_reactivity_for_testing();
    let base = signal(2);
    let first_runs = Rc::new(Cell::new(0));
    let second_runs = Rc::new(Cell::new(0));

    let doubled = computed({
        let base = base.clone();
        let first_runs = first_runs.clone();
        move || {
            first_runs.set(first_runs.get() + 1);
            base.get() * 2
        }
    });

    let quadrupled = computed({
        let doubled = doubled.clone();
        let second_runs = second_runs.clone();
        move || {
            second_runs.set(second_runs.get() + 1);
            doubled.get() * 2
        }
    });

    assert_eq!(quadrupled.get(), 8);
    assert_eq!(quadrupled.get(), 8);
    assert_eq!(first_runs.get(), 1);
    assert_eq!(second_runs.get(), 1);

    base.set(3);

    assert_eq!(quadrupled.get(), 12);
    assert_eq!(first_runs.get(), 2);
    assert_eq!(second_runs.get(), 2);
}

#[test]
fn stopped_effect_does_not_run_again() {
    reset_reactivity_for_testing();
    let value = signal(1);
    let seen = Rc::new(RefCell::new(Vec::new()));

    let effect_handle = effect({
        let value = value.clone();
        let seen = seen.clone();
        move || seen.borrow_mut().push(value.get())
    });

    effect_handle.stop();
    value.set(2);
    flush_reactivity();

    assert_eq!(seen.borrow().as_slice(), &[1]);
}

#[test]
fn component_scope_is_marked_dirty_when_a_dependency_changes() {
    reset_reactivity_for_testing();
    let value = signal(String::from("idle"));

    let value_for_first = value.clone();
    let (scope, first) = render_in_component_scope(None, move |_| value_for_first.get());
    assert_eq!(first, "idle");
    assert!(take_dirty_component_scopes().is_empty());

    value.set(String::from("ready"));

    assert_eq!(take_dirty_component_scopes(), vec![scope]);

    let value_for_second = value.clone();
    let (_scope, second) = render_in_component_scope(Some(scope), move |_| value_for_second.get());
    assert_eq!(second, "ready");
    assert!(take_dirty_component_scopes().is_empty());
}

#[test]
fn disposed_component_scopes_stop_receiving_dirty_notifications() {
    reset_reactivity_for_testing();
    let value = signal(String::from("idle"));

    let value_for_render = value.clone();
    let (scope, first) = render_in_component_scope(None, move |_| value_for_render.get());
    assert_eq!(first, "idle");

    dispose_component_scope(scope);
    value.set(String::from("ready"));

    assert!(take_dirty_component_scopes().is_empty());
}

#[test]
fn dirty_component_queue_reports_each_scope_once_without_scanning_all_scopes() {
    reset_reactivity_for_testing();
    let value = signal(0usize);

    let value_for_a = value.clone();
    let (scope_a, _) = render_in_component_scope(None, move |_| value_for_a.get());
    let value_for_b = value.clone();
    let (scope_b, _) = render_in_component_scope(None, move |_| value_for_b.get());

    assert!(take_dirty_component_scopes().is_empty());

    value.set(1);

    let dirty = take_dirty_component_scopes();
    assert_eq!(dirty, vec![scope_a, scope_b]);
    assert!(take_dirty_component_scopes().is_empty());

    value.set(2);
    value.set(3);

    let dirty_again = take_dirty_component_scopes();
    assert_eq!(dirty_again, vec![scope_a, scope_b]);
    assert!(take_dirty_component_scopes().is_empty());
}

#[test]
fn set_if_changed_skips_dirty_component_propagation_for_identical_values() {
    reset_reactivity_for_testing();
    let value = signal(1usize);

    let value_for_render = value.clone();
    let (scope, first) = render_in_component_scope(None, move |_| value_for_render.get());
    assert_eq!(first, 1);
    assert!(take_dirty_component_scopes().is_empty());

    value.set_if_changed(1);
    assert!(take_dirty_component_scopes().is_empty());

    value.set_if_changed(2);
    assert_eq!(take_dirty_component_scopes(), vec![scope]);
}

#[test]
fn set_if_changed_does_not_rerun_effect_for_identical_values() {
    reset_reactivity_for_testing();
    let value = signal(String::from("same"));
    let seen = Rc::new(RefCell::new(Vec::new()));

    let _effect = effect({
        let value = value.clone();
        let seen = seen.clone();
        move || seen.borrow_mut().push(value.get())
    });

    assert_eq!(seen.borrow().as_slice(), &["same"]);

    value.set_if_changed(String::from("same"));
    flush_reactivity();
    assert_eq!(seen.borrow().as_slice(), &["same"]);

    value.set_if_changed(String::from("changed"));
    flush_reactivity();
    assert_eq!(seen.borrow().as_slice(), &["same", "changed"]);
}
