use ansiq_examples::scenarios::{
    activity_monitor::VIEWPORT_POLICY as ACTIVITY_POLICY,
    list_navigation::VIEWPORT_POLICY as LIST_POLICY, scroll_sync::VIEWPORT_POLICY as SCROLL_POLICY,
    table_interaction::VIEWPORT_POLICY as TABLE_POLICY,
};
use ansiq_runtime::ViewportPolicy;

#[test]
fn scenarios_use_policies_sized_for_their_own_layouts() {
    assert_eq!(
        ACTIVITY_POLICY,
        ViewportPolicy::ReserveFitContent { min: 28, max: 28 }
    );
    assert_eq!(
        LIST_POLICY,
        ViewportPolicy::ReserveFitContent { min: 5, max: 8 }
    );
    assert_eq!(
        SCROLL_POLICY,
        ViewportPolicy::ReserveFitContent { min: 5, max: 8 }
    );
    assert_eq!(
        TABLE_POLICY,
        ViewportPolicy::ReserveFitContent { min: 6, max: 10 }
    );
}
