use ansiq_core::{ElementKind, Node, Rect};
use ansiq_examples::{
    activity_monitor::{
        ActivitySnapshot, ActivitySummary, MemorySummary, NetworkSummary, ProcessSample,
        ResourceTotals,
    },
    scenarios::activity_monitor::{ActivityMonitorApp, ActivityMonitorMessage},
};
use ansiq_render::FrameBuffer;
use ansiq_runtime::{Engine, RuntimeHandle, draw_tree, draw_tree_in_regions};
use ansiq_surface::Key;

fn fixture_snapshot() -> ActivitySnapshot {
    ActivitySnapshot {
        process_count: 3,
        thread_count: 81,
        cpu: ResourceTotals {
            user_percent: 17.2,
            system_percent: 7.7,
            idle_percent: 75.1,
        },
        memory: MemorySummary {
            used_bytes: 12 * 1024 * 1024 * 1024,
            app_bytes: 6 * 1024 * 1024 * 1024,
            wired_bytes: 3 * 1024 * 1024 * 1024,
            compressed_bytes: 512 * 1024 * 1024,
            cached_bytes: 2 * 1024 * 1024 * 1024,
            swap_used_bytes: 256 * 1024 * 1024,
        },
        disk: ActivitySummary {
            read_per_sec: 12_000_000,
            write_per_sec: 8_000_000,
            total_in: 0,
            total_out: 0,
        },
        network: NetworkSummary {
            recv_per_sec: 18_000_000,
            send_per_sec: 2_200_000,
            total_recv: 420_000_000,
            total_send: 96_000_000,
        },
        processes: vec![
            ProcessSample {
                name: "FClash".to_string(),
                pid: 9643,
                user: "murong".to_string(),
                kind: "Apple".to_string(),
                cpu_percent: 56.5,
                cpu_time_secs: 22_986,
                threads: 15,
                idle_wakeups: 177,
                gpu_percent: 0.0,
                gpu_time_secs: 54,
                memory_bytes: 380 * 1024 * 1024,
                compressed_bytes: 28 * 1024 * 1024,
                energy_impact: 38.0,
                energy_impact_avg: 24.0,
                disk_read_bytes: 23_000_000,
                disk_write_bytes: 8_000_000,
                network_in_bytes: 12_400_000,
                network_out_bytes: 21_000_000,
            },
            ProcessSample {
                name: "Google Chrome".to_string(),
                pid: 76868,
                user: "murong".to_string(),
                kind: "Apple".to_string(),
                cpu_percent: 55.8,
                cpu_time_secs: 1_565,
                threads: 33,
                idle_wakeups: 408,
                gpu_percent: 0.0,
                gpu_time_secs: 0,
                memory_bytes: 1_240 * 1024 * 1024,
                compressed_bytes: 64 * 1024 * 1024,
                energy_impact: 41.2,
                energy_impact_avg: 30.6,
                disk_read_bytes: 14_000_000,
                disk_write_bytes: 19_000_000,
                network_in_bytes: 210_000_000,
                network_out_bytes: 18_000_000,
            },
            ProcessSample {
                name: "WindowServer".to_string(),
                pid: 398,
                user: "_windowserver".to_string(),
                kind: "Apple".to_string(),
                cpu_percent: 55.3,
                cpu_time_secs: 44_820,
                threads: 24,
                idle_wakeups: 600,
                gpu_percent: 25.8,
                gpu_time_secs: 16_596,
                memory_bytes: 512 * 1024 * 1024,
                compressed_bytes: 12 * 1024 * 1024,
                energy_impact: 44.8,
                energy_impact_avg: 37.4,
                disk_read_bytes: 2_000_000,
                disk_write_bytes: 3_000_000,
                network_in_bytes: 0,
                network_out_bytes: 0,
            },
        ],
        warning: None,
    }
}

fn collect_text(node: &Node<ActivityMonitorMessage>, lines: &mut Vec<String>) {
    match &node.element.kind {
        ElementKind::Text(props) => lines.push(props.content.clone()),
        ElementKind::StatusBar(props) => lines.push(props.content.clone()),
        _ => {}
    }

    for child in &node.children {
        collect_text(child, lines);
    }
}

fn find_block_rect(node: &Node<ActivityMonitorMessage>, title: &str) -> Option<Rect> {
    match &node.element.kind {
        ElementKind::Block(props)
            if props
                .titles
                .iter()
                .any(|block_title| block_title.content.plain() == title) =>
        {
            Some(node.rect)
        }
        _ => node
            .children
            .iter()
            .find_map(|child| find_block_rect(child, title)),
    }
}

fn rendered_screen(engine: &Engine<ActivityMonitorApp>, width: u16, height: u16) -> String {
    let tree = engine.tree().expect("tree should exist");
    let mut buffer = FrameBuffer::new(width, height);
    draw_tree(tree, engine.focused(), &mut buffer);

    (0..height)
        .map(|y| {
            (0..width)
                .map(|x| buffer.get(x, y).symbol)
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn buffer_to_string(buffer: &FrameBuffer) -> String {
    (0..buffer.height())
        .map(|y| {
            (0..buffer.width())
                .map(|x| buffer.get(x, y).symbol)
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn activity_monitor_renders_all_processes_dashboard() {
    let mut engine = Engine::new(ActivityMonitorApp::from_snapshot(fixture_snapshot()));
    engine.render_tree();

    let tree = engine.tree().expect("tree should exist");
    let mut lines = Vec::new();
    collect_text(tree, &mut lines);
    let joined = lines.join("\n");

    assert!(joined.contains("activity monitor"));
    assert!(joined.contains("Selected process: FClash"));
    assert!(joined.contains("Tab: CPU"));

    let screen = rendered_screen(&engine, 120, 28);
    assert!(screen.contains("All Processes"));
    assert!(screen.contains("FClash"));
    assert!(screen.contains("% CPU"));
}

#[test]
fn activity_monitor_switches_tabs_and_selected_process() {
    let mut engine = Engine::new(ActivityMonitorApp::from_snapshot(fixture_snapshot()));
    engine.render_tree();

    assert!(!engine.handle_input(Key::Right));
    engine.render_tree();

    let mut lines = Vec::new();
    collect_text(engine.tree().expect("tree should exist"), &mut lines);
    assert!(lines.join("\n").contains("Tab: Memory"));
    assert!(lines.join("\n").contains("Selected process: Google Chrome"));

    assert!(!engine.handle_input(Key::Tab));
    assert!(!engine.handle_input(Key::Down));
    engine.render_tree();

    let mut lines = Vec::new();
    collect_text(engine.tree().expect("tree should exist"), &mut lines);
    assert!(lines.join("\n").contains("Selected process: WindowServer"));
}

#[test]
fn activity_monitor_applies_snapshot_messages() {
    let mut engine = Engine::new(ActivityMonitorApp::from_snapshot(fixture_snapshot()));
    engine.render_tree();

    let mut updated = fixture_snapshot();
    updated.processes[0].name = "Wave Helper".to_string();
    updated.processes[0].cpu_percent = 88.4;
    updated.processes[0].threads = 21;

    let handle: RuntimeHandle<ActivityMonitorMessage> = engine.handle();
    handle
        .emit(ActivityMonitorMessage::Snapshot(updated))
        .expect("snapshot send should succeed");
    engine.drain_requests();
    engine.render_tree();

    let screen = rendered_screen(&engine, 120, 28);
    assert!(screen.contains("Wave Helper"));
    assert!(screen.contains("88.4"));
}

#[test]
fn activity_monitor_switches_to_network_tab() {
    let mut engine = Engine::new(ActivityMonitorApp::from_snapshot(fixture_snapshot()));
    engine.render_tree();

    assert!(!engine.handle_input(Key::Right));
    assert!(!engine.handle_input(Key::Right));
    assert!(!engine.handle_input(Key::Right));
    assert!(!engine.handle_input(Key::Right));
    engine.render_tree();

    let mut lines = Vec::new();
    collect_text(engine.tree().expect("tree should exist"), &mut lines);
    assert!(lines.join("\n").contains("Tab: Network"));

    let screen = rendered_screen(&engine, 120, 28);
    assert!(screen.contains("Bytes In"));
    assert!(screen.contains("Bytes Out"));
}

#[test]
fn activity_monitor_footer_blocks_do_not_overlap_the_process_table() {
    let mut engine = Engine::new(ActivityMonitorApp::from_snapshot(fixture_snapshot()));
    engine.render_tree();

    let tree = engine.tree().expect("tree should exist");
    let table_rect = find_block_rect(tree, "All Processes").expect("process block should exist");
    let cpu_rect = find_block_rect(tree, "CPU").expect("cpu footer block should exist");
    let memory_rect = find_block_rect(tree, "Memory").expect("memory footer block should exist");
    let details_rect =
        find_block_rect(tree, "CPU details").expect("details footer block should exist");

    assert!(cpu_rect.y >= table_rect.bottom());
    assert!(memory_rect.y >= table_rect.bottom());
    assert!(details_rect.y >= table_rect.bottom());
}

#[test]
fn activity_monitor_footer_height_stays_stable_for_long_selection_text() {
    let width = 100;
    let height = 28;
    let mut engine = Engine::new(ActivityMonitorApp::from_snapshot(fixture_snapshot()));
    engine.set_bounds(Rect::new(0, 0, width, height));
    engine.render_tree();

    let initial_tree = engine.tree().expect("tree should exist");
    let initial_cpu_rect = find_block_rect(initial_tree, "CPU").expect("cpu footer block");
    let initial_memory_rect = find_block_rect(initial_tree, "Memory").expect("memory footer block");
    let initial_details_rect =
        find_block_rect(initial_tree, "CPU details").expect("details footer block");

    let mut updated = fixture_snapshot();
    updated.processes[0].name =
        "A process name long enough to wrap the selection summary in a narrow viewport".to_string();

    let handle: RuntimeHandle<ActivityMonitorMessage> = engine.handle();
    handle
        .emit(ActivityMonitorMessage::Snapshot(updated))
        .expect("snapshot send should succeed");
    engine.drain_requests();
    engine.render_tree();

    let updated_tree = engine.tree().expect("tree should exist");
    let updated_cpu_rect = find_block_rect(updated_tree, "CPU").expect("cpu footer block");
    let updated_memory_rect = find_block_rect(updated_tree, "Memory").expect("memory footer block");
    let updated_details_rect =
        find_block_rect(updated_tree, "CPU details").expect("details footer block");

    assert_eq!(updated_cpu_rect.y, initial_cpu_rect.y);
    assert_eq!(updated_memory_rect.y, initial_memory_rect.y);
    assert_eq!(updated_details_rect.y, initial_details_rect.y);
}

#[test]
fn activity_monitor_header_height_stays_stable_when_warning_clears() {
    let width = 64;
    let height = 28;
    let mut engine = Engine::new(ActivityMonitorApp::default());
    engine.set_bounds(Rect::new(0, 0, width, height));
    engine.render_tree();

    let initial_tree = engine.tree().expect("tree should exist");
    let initial_table_rect =
        find_block_rect(initial_tree, "All Processes").expect("process block should exist");
    let initial_cpu_rect = find_block_rect(initial_tree, "CPU").expect("cpu footer block");

    let handle: RuntimeHandle<ActivityMonitorMessage> = engine.handle();
    handle
        .emit(ActivityMonitorMessage::Snapshot(fixture_snapshot()))
        .expect("snapshot send should succeed");
    engine.drain_requests();
    engine.render_tree();

    let updated_tree = engine.tree().expect("tree should exist");
    let updated_table_rect =
        find_block_rect(updated_tree, "All Processes").expect("process block should exist");
    let updated_cpu_rect = find_block_rect(updated_tree, "CPU").expect("cpu footer block");

    assert_eq!(updated_table_rect.y, initial_table_rect.y);
    assert_eq!(updated_cpu_rect.y, initial_cpu_rect.y);
}

#[test]
fn activity_monitor_redraw_matches_a_fresh_full_redraw() {
    let width = 120;
    let height = 28;

    let mut engine = Engine::new(ActivityMonitorApp::from_snapshot(fixture_snapshot()));
    engine.set_bounds(Rect::new(0, 0, width, height));
    engine.render_tree();

    let mut previous = FrameBuffer::new(width, height);
    draw_tree(
        engine.tree().expect("tree should exist"),
        engine.focused(),
        &mut previous,
    );

    let mut updated = fixture_snapshot();
    updated.process_count = 4;
    updated.processes[0].name = "Wave Helper".to_string();
    updated.processes[0].cpu_percent = 88.4;
    updated.processes[0].threads = 21;
    updated.processes[1].name = "Terminal".to_string();
    updated.processes[1].cpu_percent = 12.8;

    let handle: RuntimeHandle<ActivityMonitorMessage> = engine.handle();
    handle
        .emit(ActivityMonitorMessage::Snapshot(updated))
        .expect("snapshot send should succeed");
    engine.drain_requests();
    engine.render_tree();

    let mut partial = previous.clone();
    if let Some(regions) = engine.redraw_regions() {
        draw_tree_in_regions(
            engine.tree().expect("tree should exist"),
            engine.focused(),
            &mut partial,
            regions,
        );
    } else {
        draw_tree(
            engine.tree().expect("tree should exist"),
            engine.focused(),
            &mut partial,
        );
    }

    let mut full = FrameBuffer::new(width, height);
    draw_tree(
        engine.tree().expect("tree should exist"),
        engine.focused(),
        &mut full,
    );

    assert_eq!(buffer_to_string(&partial), buffer_to_string(&full));
}

#[test]
fn activity_monitor_placeholder_redraw_matches_a_fresh_full_redraw() {
    let width = 120;
    let height = 28;

    let mut engine = Engine::new(ActivityMonitorApp::default());
    engine.set_bounds(Rect::new(0, 0, width, height));
    engine.render_tree();

    let mut previous = FrameBuffer::new(width, height);
    draw_tree(
        engine.tree().expect("tree should exist"),
        engine.focused(),
        &mut previous,
    );

    let handle: RuntimeHandle<ActivityMonitorMessage> = engine.handle();
    handle
        .emit(ActivityMonitorMessage::Snapshot(fixture_snapshot()))
        .expect("snapshot send should succeed");
    engine.drain_requests();
    engine.render_tree();

    let mut partial = previous.clone();
    if let Some(regions) = engine.redraw_regions() {
        draw_tree_in_regions(
            engine.tree().expect("tree should exist"),
            engine.focused(),
            &mut partial,
            regions,
        );
    } else {
        draw_tree(
            engine.tree().expect("tree should exist"),
            engine.focused(),
            &mut partial,
        );
    }

    let mut full = FrameBuffer::new(width, height);
    draw_tree(
        engine.tree().expect("tree should exist"),
        engine.focused(),
        &mut full,
    );

    assert_eq!(buffer_to_string(&partial), buffer_to_string(&full));
}

#[test]
fn activity_monitor_partial_redraw_matches_full_redraw_when_selection_footer_wraps() {
    let width = 72;
    let height = 28;

    let mut engine = Engine::new(ActivityMonitorApp::from_snapshot(fixture_snapshot()));
    engine.set_bounds(Rect::new(0, 0, width, height));
    engine.render_tree();

    let mut previous = FrameBuffer::new(width, height);
    draw_tree(
        engine.tree().expect("tree should exist"),
        engine.focused(),
        &mut previous,
    );

    let mut updated = fixture_snapshot();
    updated.processes[0].name =
        "FeatureAccessAgentWithAnExtremelyLongSelectionSummaryName".to_string();

    let handle: RuntimeHandle<ActivityMonitorMessage> = engine.handle();
    handle
        .emit(ActivityMonitorMessage::Snapshot(updated))
        .expect("snapshot send should succeed");
    engine.drain_requests();
    engine.render_tree();

    let mut partial = previous.clone();
    if let Some(regions) = engine.redraw_regions() {
        draw_tree_in_regions(
            engine.tree().expect("tree should exist"),
            engine.focused(),
            &mut partial,
            regions,
        );
    } else {
        draw_tree(
            engine.tree().expect("tree should exist"),
            engine.focused(),
            &mut partial,
        );
    }

    let mut full = FrameBuffer::new(width, height);
    draw_tree(
        engine.tree().expect("tree should exist"),
        engine.focused(),
        &mut full,
    );

    assert_eq!(buffer_to_string(&partial), buffer_to_string(&full));
}
