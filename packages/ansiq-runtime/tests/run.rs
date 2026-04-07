use ansiq_core::Rect;
use ansiq_runtime::{Viewport, ViewportPolicy, exit_row_for_content, viewport_bounds};

#[test]
fn runtime_reexports_surface_viewport_types() {
    let viewport = Viewport {
        width: 120,
        height: 20,
        origin_y: 4,
    };

    assert_eq!(viewport.origin_y, 4);
    assert!(matches!(
        ViewportPolicy::PreserveVisible,
        ViewportPolicy::PreserveVisible
    ));
}

#[test]
fn viewport_bounds_use_surface_dimensions_without_recomputing_origin() {
    assert_eq!(
        viewport_bounds(Viewport {
            width: 80,
            height: 18,
            origin_y: 10,
        }),
        Rect::new(0, 0, 80, 18)
    );
}

#[test]
fn exit_row_tracks_rendered_content_instead_of_reserved_viewport_height() {
    assert_eq!(
        exit_row_for_content(
            Viewport {
                width: 80,
                height: 18,
                origin_y: 10,
            },
            6,
        ),
        15
    );
}

#[test]
fn exit_row_is_clamped_to_the_viewport_when_content_overflows() {
    assert_eq!(
        exit_row_for_content(
            Viewport {
                width: 80,
                height: 18,
                origin_y: 10,
            },
            40,
        ),
        27
    );
}
