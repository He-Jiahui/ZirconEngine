use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use super::super::host_template_node;
use super::attributes::normalized_tone;
use super::parse::{parse_count, reset_parse_count};
use super::{
    projected_notification_center_metadata, projected_notification_center_options,
    projected_notification_center_structured_options, projected_notification_center_value_text,
};
use crate::ui::template_runtime::RetainedUiHostNodeProjection;
use toml::Value;
use zircon_runtime_interface::ui::layout::UiFrame;

#[test]
fn runtime_component_projection_projects_notification_center_rows_for_native_painter() {
    let node = host_template_node(projected_node(
        "NotificationCenter",
        [
            ("open", Value::Boolean(true)),
            ("title", Value::String("Notifications".into())),
            ("empty_text", Value::String("No notifications".into())),
            ("notification_generation", Value::Integer(7)),
            ("unread_count", Value::Integer(2)),
            ("overflow_count", Value::Integer(3)),
            ("selected_notification_id", Value::String("build".into())),
            ("focused_index", Value::Integer(1)),
            ("visible_limit", Value::Integer(2)),
            ("placement", Value::String("bottom-end".into())),
            ("popup_anchor_x", Value::Float(400.0)),
            ("popup_anchor_y", Value::Float(40.0)),
            ("popup_anchor_width", Value::Float(64.0)),
            ("popup_anchor_height", Value::Float(16.0)),
            ("popup_offset_y", Value::Float(8.0)),
            (
                "notifications",
                Value::Array(vec![
                    notification_entry([
                        ("id", Value::String("build".into())),
                        ("title", Value::String("Build failed".into())),
                        ("message", Value::String("Shader compile error".into())),
                        ("severity", Value::String("error".into())),
                        ("unread", Value::Boolean(true)),
                    ]),
                    notification_entry([
                        ("id", Value::String("asset".into())),
                        ("title", Value::String("Asset import complete".into())),
                        ("description", Value::String("StoneWall.mesh ready".into())),
                        ("severity", Value::String("success".into())),
                        ("new", Value::Boolean(true)),
                    ]),
                    notification_entry([
                        ("id", Value::String("source".into())),
                        ("title", Value::String("Source control synced".into())),
                    ]),
                ]),
            ),
        ],
    ))
    .expect("NotificationCenter should project into the host contract");

    assert_eq!(node.component_role.as_str(), "notification-center");
    assert!(node.popup_open);
    assert_eq!(node.text.as_str(), "Notifications");
    assert_eq!(node.value_text.as_str(), "No notifications");
    assert_eq!(node.notification_generation, 7);
    assert_eq!(node.notification_unread_count, 2);
    assert_eq!(node.notification_overflow_count, 3);
    assert_eq!(node.notification_selected_id.as_str(), "build");
    assert_eq!(node.notification_focused_index, 1);
    assert_eq!(node.notification_visible_limit, 2);
    assert_eq!(node.options.row_count(), 2);
    assert_eq!(node.options.row_data(0).as_deref(), Some("Build failed"));
    assert_eq!(
        node.options_text.as_str(),
        "Build failed, Asset import complete"
    );
    assert_eq!(node.frame.x, 144.0);
    assert_eq!(node.frame.y, 64.0);

    let first = node
        .structured_options
        .row_data(0)
        .expect("first notification row should be projected");
    assert_eq!(first.id.as_str(), "build");
    assert_eq!(first.label.as_str(), "Build failed");
    assert_eq!(first.description.as_str(), "Shader compile error");
    assert_eq!(first.tone.as_str(), "error");
    assert!(first.selected);
    assert!(first.unread);
    assert!(first.special);

    let second = node
        .structured_options
        .row_data(1)
        .expect("second notification row should be projected");
    assert_eq!(second.id.as_str(), "asset");
    assert_eq!(second.description.as_str(), "StoneWall.mesh ready");
    assert_eq!(second.tone.as_str(), "success");
    assert!(second.focused);
    assert!(second.unread);
}

#[test]
fn visible_limit_stops_notification_parsing_before_offscreen_rows() {
    let notifications = (0..64)
        .map(|index| Value::String(format!("row-{index}|title=Notification {index}")))
        .collect();
    let attributes = notification_attributes([
        ("visible_limit", Value::Integer(2)),
        ("notifications", Value::Array(notifications)),
    ]);

    reset_parse_count();
    let rows = projected_notification_center_structured_options("notification-center", &attributes)
        .expect("notification rows should project");

    assert_eq!(rows.len(), 2);
    assert_eq!(parse_count(), 2);
}

#[test]
fn pipe_string_notifications_project_row_state() {
    let attributes = notification_attributes([
        ("empty_text", Value::String("Nothing pending".into())),
        ("selected_notification_id", Value::String("sync".into())),
        ("focused_index", Value::Integer(0)),
        (
            "notifications",
            Value::Array(vec![
                Value::String(
                    "sync|title=Sync paused|message=Remote offline|tone=warn|unread=yes|enabled=no"
                        .into(),
                ),
                Value::String("ok|label=Import complete|body=Mesh ready|kind=done".into()),
            ]),
        ),
    ]);

    assert_eq!(
        projected_notification_center_value_text("notification-center", &attributes).as_deref(),
        Some("Nothing pending")
    );
    assert_eq!(
        projected_notification_center_options("notification-center", &attributes),
        Some(vec![
            "Sync paused".to_string(),
            "Import complete".to_string()
        ])
    );

    let rows = projected_notification_center_structured_options("notification-center", &attributes)
        .expect("notification rows should project");
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].id.as_str(), "sync");
    assert_eq!(rows[0].description.as_str(), "Remote offline");
    assert_eq!(rows[0].tone.as_str(), "warning");
    assert!(rows[0].selected);
    assert!(rows[0].focused);
    assert!(rows[0].unread);
    assert!(rows[0].disabled);
    assert_eq!(rows[1].tone.as_str(), "success");
}

#[test]
fn pipe_string_title_fallback_distinguishes_missing_from_explicit_empty() {
    let attributes = notification_attributes([(
        "notifications",
        Value::Array(vec![
            Value::String("missing|severity=INFO".into()),
            Value::String("empty|title=|severity=WARNING".into()),
        ]),
    )]);

    let rows = projected_notification_center_structured_options("notification-center", &attributes)
        .expect("notification rows should project");

    assert_eq!(rows[0].label.as_str(), "missing");
    assert_eq!(rows[0].tone.as_str(), "info");
    assert_eq!(rows[1].label.as_str(), "");
    assert_eq!(rows[1].tone.as_str(), "warning");
}

#[test]
fn pipe_string_severity_is_not_overwritten_by_notification_kind() {
    let attributes = notification_attributes([(
        "notifications",
        Value::Array(vec![
            Value::String("error|severity=error|kind=toast".into()),
            Value::String("warning|kind=toast|severity=warning".into()),
            Value::String("success|tone=success|kind=toast".into()),
            Value::String("legacy-kind|kind=done".into()),
        ]),
    )]);

    let rows = projected_notification_center_structured_options("notification-center", &attributes)
        .expect("notification rows should project");

    assert_eq!(rows[0].tone.as_str(), "error");
    assert_eq!(rows[1].tone.as_str(), "warning");
    assert_eq!(rows[2].tone.as_str(), "success");
    assert_eq!(rows[3].tone.as_str(), "success");
}

#[test]
fn tone_normalization_avoids_allocating_a_lowercase_copy() {
    let source = include_str!("attributes.rs");

    assert!(source.contains("eq_ignore_ascii_case"));
    assert!(!source.contains("to_ascii_lowercase"));
}

#[test]
#[ignore = "managed Editor10 performance evidence"]
fn editor10_notification_tone_normalization_evidence() {
    const ROUNDS: usize = 100_000;
    const VALUES: [&str; 4] = ["SUCCESS", "Warning", "ERROR", "toast"];
    const MAX_ELAPSED: Duration = Duration::from_millis(250);

    let started = Instant::now();
    for _ in 0..ROUNDS {
        for value in VALUES {
            std::hint::black_box(normalized_tone(std::hint::black_box(value)));
        }
    }
    let elapsed = started.elapsed();
    let normalizations = ROUNDS * VALUES.len();
    let normalizations_per_second = normalizations as f64 / elapsed.as_secs_f64();

    assert!(elapsed <= MAX_ELAPSED);
    println!(
        "EDITOR_NOTIFICATION_BENCH_V1 kind=tone_normalization normalizations={} lowercase_allocations_before={} lowercase_allocations_after=0 allocation_reduction_percent=100.0000 elapsed_ns={} target_ns={} normalizations_per_second={:.2}",
        normalizations,
        normalizations,
        elapsed.as_nanos(),
        MAX_ELAPSED.as_nanos(),
        normalizations_per_second,
    );
}

#[test]
fn non_notification_roles_do_not_claim_options() {
    let attributes = notification_attributes([("empty_text", Value::String("Hidden".into()))]);

    assert_eq!(
        projected_notification_center_value_text("command-palette", &attributes),
        None
    );
    assert_eq!(
        projected_notification_center_options("command-palette", &attributes),
        None
    );
    assert_eq!(
        projected_notification_center_structured_options("command-palette", &attributes),
        None
    );
    assert_eq!(
        projected_notification_center_metadata("command-palette", &attributes),
        None
    );
}

#[test]
fn workbench_projection_builds_notification_option_pairs_once() {
    let source = include_str!("../../../workbench_window_projection.rs");
    let paired_projection = "projected_notification_center_option_rows(";
    let duplicate_options_projection = ["projected_notification_center_", "options("].concat();
    let duplicate_structured_projection =
        ["projected_notification_center_", "structured_options("].concat();

    assert!(source.contains(paired_projection));
    assert!(!source.contains(&duplicate_options_projection));
    assert!(!source.contains(&duplicate_structured_projection));
}

fn projected_node(
    component: &str,
    attributes: impl IntoIterator<Item = (&'static str, Value)>,
) -> RetainedUiHostNodeProjection {
    RetainedUiHostNodeProjection {
        node_id: format!("{component}Node"),
        surface_node_id: None,
        has_workbench_icon_tooltip: false,
        parent_id: None,
        component: component.to_owned(),
        control_id: Some(format!("{component}Control")),
        frame: UiFrame::new(0.0, 0.0, 320.0, 240.0),
        clip_frame: None,
        z_index: 0,
        attributes: notification_attributes(attributes),
        style_overrides: BTreeMap::new(),
        style_tokens: BTreeMap::new(),
        bindings: Vec::new(),
    }
}

fn notification_attributes(
    attributes: impl IntoIterator<Item = (&'static str, Value)>,
) -> BTreeMap<String, Value> {
    attributes
        .into_iter()
        .map(|(name, value)| (name.to_owned(), value))
        .collect()
}

fn notification_entry(values: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    let mut table = toml::map::Map::new();
    for (name, value) in values {
        table.insert(name.to_owned(), value);
    }
    Value::Table(table)
}
