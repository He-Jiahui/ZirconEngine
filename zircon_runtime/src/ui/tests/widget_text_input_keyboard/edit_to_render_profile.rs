use std::{hint::black_box, time::Instant};

use super::*;

const MARKER: &str = "RUNTIME82_EDIT_TO_RENDER_EXTRACT_PROFILE_V1";
const WARMUPS: usize = 5;
const SAMPLES: usize = 31;

#[test]
fn retained_keyboard_edit_reaches_the_render_extract_and_undo_restores_it() {
    let mut surface = renderable_text_input_surface("a\u{0301}bcx");
    let mut manager = UiInputManager::default();

    let result = dispatch_key_with_manager(&mut manager, &mut surface, "Backspace", 8);
    let changed = edit_receipt(&result);
    assert!(changed.revision.get() > changed.previous_revision.get());
    surface.rebuild();
    assert_eq!(content(&surface), "a\u{0301}bc");
    assert_eq!(rendered_text(&surface), Some("a\u{0301}bc"));

    let undo = dispatch_key_with_manager_control(&mut manager, &mut surface, "z", 90);
    let undone = edit_receipt(&undo);
    assert_eq!(undone.document_id, changed.document_id);
    assert_eq!(
        undone.kind,
        zircon_runtime_interface::ui::text::UiTextEditKind::Undo
    );
    surface.rebuild();
    assert_eq!(content(&surface), "a\u{0301}bcx");
    assert_eq!(rendered_text(&surface), Some("a\u{0301}bcx"));
}

#[test]
#[ignore = "managed Windows Release UI edit-to-render-extract profiling"]
fn retained_keyboard_edit_to_render_extract_scale_profile() {
    assert!(!cfg!(debug_assertions), "run this profile with --release");
    for base_graphemes in [1, 100, 1_000, 10_000] {
        profile_scale(base_graphemes);
    }
}

#[test]
#[ignore = "managed Windows Release million-character editing diagnostic"]
fn retained_keyboard_edit_to_render_extract_million_character_profile() {
    assert!(!cfg!(debug_assertions), "run this profile with --release");
    profile_scale(1_000_000);
}

fn profile_scale(base_graphemes: usize) {
    let edited = "a".repeat(base_graphemes);
    let initial = format!("{edited}x");
    let mut surface = renderable_text_input_surface(&initial);
    let mut manager = UiInputManager::default();

    for _ in 0..WARMUPS {
        black_box(edit_render_undo_cycle(
            &mut manager,
            &mut surface,
            &edited,
            &initial,
        ));
    }
    let mut dispatch_ns = Vec::with_capacity(SAMPLES);
    let mut extract_ns = Vec::with_capacity(SAMPLES);
    let mut total_ns = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let sample = edit_render_undo_cycle(&mut manager, &mut surface, &edited, &initial);
        dispatch_ns.push(sample.dispatch_ns);
        extract_ns.push(sample.extract_ns);
        total_ns.push(sample.total_ns);
    }

    print_samples(base_graphemes, "dispatch", dispatch_ns);
    print_samples(base_graphemes, "surface_rebuild_to_extract", extract_ns);
    print_samples(base_graphemes, "edit_to_extract", total_ns);
}

struct EditRenderSample {
    dispatch_ns: u128,
    extract_ns: u128,
    total_ns: u128,
}

fn edit_render_undo_cycle(
    manager: &mut UiInputManager,
    surface: &mut UiSurface,
    edited: &str,
    initial: &str,
) -> EditRenderSample {
    let previous_revision = text_layout_revision(surface);
    let start = Instant::now();
    let result = dispatch_key_with_manager(manager, surface, "Backspace", 8);
    let dispatch_ns = start.elapsed().as_nanos();
    let render_start = Instant::now();
    surface.rebuild();
    let extract_ns = render_start.elapsed().as_nanos();
    let total_ns = start.elapsed().as_nanos();

    let changed = edit_receipt(&result);
    assert!(changed.revision.get() > changed.previous_revision.get());
    assert_eq!(content(surface), edited);
    assert_eq!(rendered_text(surface), Some(edited));
    assert_eq!(int_attr(surface, "caret_offset"), edited.len() as i64);
    assert!(text_layout_revision(surface) > previous_revision);

    let undone = dispatch_key_with_manager_control(manager, surface, "z", 90);
    let undone_receipt = edit_receipt(&undone);
    assert_eq!(undone_receipt.document_id, changed.document_id);
    assert_eq!(
        undone_receipt.kind,
        zircon_runtime_interface::ui::text::UiTextEditKind::Undo
    );
    surface.rebuild();
    assert_eq!(content(surface), initial);
    assert_eq!(rendered_text(surface), Some(initial));
    assert_eq!(int_attr(surface, "caret_offset"), initial.len() as i64);

    EditRenderSample {
        dispatch_ns,
        extract_ns,
        total_ns,
    }
}

fn edit_receipt(
    result: &zircon_runtime_interface::ui::dispatch::UiInputDispatchResult,
) -> &zircon_runtime_interface::ui::text::UiTextEditReceipt {
    result
        .widget_events
        .iter()
        .find_map(|event| match event {
            UiWidgetEvent::TextEditChange { receipt } => Some(receipt),
            _ => None,
        })
        .expect("retained text edit receipt")
}

fn renderable_text_input_surface(value: &str) -> UiSurface {
    let mut surface = text_input_surface(value, value.len());
    surface
        .tree
        .node_mut(UiNodeId::new(1))
        .expect("editable root")
        .state_flags
        .visible = true;
    surface
        .tree
        .node_mut(UiNodeId::new(2))
        .and_then(|node| node.template_metadata.as_mut())
        .expect("editable widget metadata")
        .component = "InputField".to_string();
    surface.rebuild();
    surface
        .focus_node(UiNodeId::new(2))
        .expect("focus editable widget");
    surface.rebuild();
    surface
}

fn content(surface: &UiSurface) -> &str {
    surface
        .tree
        .node(UiNodeId::new(2))
        .and_then(|node| node.template_metadata.as_ref())
        .and_then(|metadata| metadata.attributes.get("content"))
        .and_then(toml::Value::as_str)
        .expect("editable content")
}

fn rendered_text(surface: &UiSurface) -> Option<&str> {
    surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == UiNodeId::new(2) && command.text.is_some())
        .and_then(|command| command.text.as_deref())
}

fn print_samples(base_graphemes: usize, stage: &str, raw: Vec<u128>) {
    let mut sorted = raw.clone();
    sorted.sort_unstable();
    let rank = |percent: usize| sorted[(sorted.len() * percent).div_ceil(100) - 1];
    println!(
        "{MARKER} base_graphemes={base_graphemes} stage={stage} warmups={WARMUPS} samples={} p50_ns={} p95_ns={} p99_ns={} raw_ns={raw:?} os={} arch={} package_version={}",
        raw.len(),
        rank(50),
        rank(95),
        rank(99),
        std::env::consts::OS,
        std::env::consts::ARCH,
        env!("CARGO_PKG_VERSION")
    );
}
