use super::fixtures::{
    button_metadata, rich_text_metadata, text_layout_command_count, text_metadata,
    vertical_text_metadata, visible_text_state,
};
#[cfg(feature = "profiling")]
use crate::core::runtime::diagnostics::profiling::{
    ProfileCaptureConfig, reset_capture, snapshot, start_capture, test_capture_lock,
};
use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::UiFrame,
    surface::{UiRenderCommandKind, UiTextWritingMode},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn render_extract_automatically_prewarms_visible_owner_text_before_layout() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.text.prewarm.surface"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 260.0, 72.0))
            .with_state_flags(visible_text_state(true)),
    );
    for (node_id, path, frame, text, visible) in [
        (
            UiNodeId::new(2),
            "root/first",
            UiFrame::new(0.0, 0.0, 220.0, 16.0),
            "editor base.zui",
            true,
        ),
        (
            UiNodeId::new(3),
            "root/second",
            UiFrame::new(0.0, 16.0, 220.0, 16.0),
            "folder-open-outline.svg",
            true,
        ),
        (
            UiNodeId::new(4),
            "root/duplicate",
            UiFrame::new(0.0, 32.0, 220.0, 16.0),
            "editor base.zui",
            true,
        ),
        (
            UiNodeId::new(5),
            "root/hidden",
            UiFrame::new(0.0, 48.0, 220.0, 16.0),
            "hidden-row-should-not-prewarm",
            false,
        ),
    ] {
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                UiTreeNode::new(node_id, UiNodePath::new(path))
                    .with_frame(frame)
                    .with_state_flags(visible_text_state(visible))
                    .with_template_metadata(text_metadata(text)),
            )
            .expect("text child should be inserted");
    }

    surface.rebuild();

    let prewarm_report = surface.text_measure_cache.frame_shape_prewarm_report();
    let shaped_report = surface.text_measure_cache.frame_shaped_run_report();

    assert_eq!(text_layout_command_count(&surface), 3);
    assert_eq!(
        prewarm_report.requested_count, 3,
        "only visible owner text should be collected for automatic prewarm"
    );
    assert_eq!(prewarm_report.cache_hit_count, 0);
    assert_eq!(
        prewarm_report.cache_miss_count, 2,
        "duplicate visible labels should share one pending shape miss"
    );
    assert_eq!(prewarm_report.batch_duplicate_count, 1);
    assert_eq!(prewarm_report.shaped_count, 2);
    assert_eq!(prewarm_report.inserted_count, 2);
    assert_eq!(
        shaped_report
            .miss_count
            .saturating_sub(prewarm_report.inserted_count as u64),
        1,
        "layout should only add the shared metrics run after visible source text prewarm"
    );
    assert!(
        shaped_report.hit_count >= prewarm_report.requested_count as u64,
        "layout should consume the source runs inserted by automatic prewarm"
    );
}

#[test]
fn owner_prewarm_overlap_keeps_component_text_layouts_on_the_shared_cache_path() {
    #[cfg(feature = "profiling")]
    let _capture_guard = test_capture_lock();
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.text.prewarm.text-field-cache"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 320.0, 220.0))
            .with_state_flags(visible_text_state(true)),
    );
    for index in 0..8 {
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                UiTreeNode::new(
                    UiNodeId::new(index + 2),
                    UiNodePath::new(format!("root/label-{index}")),
                )
                .with_frame(UiFrame::new(0.0, index as f32 * 16.0, 220.0, 16.0))
                .with_state_flags(visible_text_state(true))
                .with_template_metadata(text_metadata(&format!("owner label {index}"))),
            )
            .expect("owner label should be inserted");
    }
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(10), UiNodePath::new("root/input"))
                .with_frame(UiFrame::new(0.0, 144.0, 260.0, 32.0))
                .with_state_flags(visible_text_state(true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "InputField".to_string(),
                    attributes: toml::from_str(
                        r#"
content = "cached input"
font_size = 12.0
line_height = 16.0
"#,
                    )
                    .expect("text field metadata should parse"),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .expect("text field should be inserted");
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(11), UiNodePath::new("root/context-menu"))
                .with_frame(UiFrame::new(0.0, 180.0, 160.0, 32.0))
                .with_state_flags(visible_text_state(true))
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "ContextActionMenu".to_string(),
                    attributes: toml::from_str(
                        r#"
popup_open = true
menu_items = ["open|label=Open"]
"#,
                    )
                    .expect("popup menu metadata should parse"),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .expect("popup menu should be inserted");

    #[cfg(feature = "profiling")]
    {
        let mut config = ProfileCaptureConfig::default();
        config.session_id = "ui-text-component-prewarm-frame".to_string();
        config.max_frames = 2;
        config.max_spans = 128;
        config.max_counters = 256;
        start_capture(config);
        {
            crate::profile_frame!("runtime", "component_prewarm_test_frame");
            surface.rebuild();
        }
    }
    #[cfg(not(feature = "profiling"))]
    surface.rebuild();

    #[cfg(feature = "profiling")]
    let first_frame_profile = {
        let profile = snapshot();
        reset_capture();
        profile
    };

    surface.rebuild();

    let layout_report = surface.text_measure_cache.frame_layout_report();
    assert!(
        layout_report.hit_count >= 10,
        "owners, text field, and popup label must reuse cached layouts on the stable frame: {layout_report:#?}"
    );
    assert!(surface.render_extract.list.commands.iter().any(|command| {
        command.style.painter_family
            == zircon_runtime_interface::ui::style::UiPainterFamily::TextField
            && command.text.as_deref() == Some("cached input")
            && command.text_layout.is_some()
    }));
    assert!(surface.render_extract.list.commands.iter().any(|command| {
        command.text.as_deref() == Some("Open") && command.text_layout.is_some()
    }));

    #[cfg(feature = "profiling")]
    {
        for (counter_name, expected_value) in [
            ("ui_text.prewarm.owner_overlap_joins", Some(0.0)),
            ("ui_text.prewarm.requested", None),
            ("text.shape_batch.requested", None),
        ] {
            let samples = first_frame_profile
                .counters
                .iter()
                .filter(|counter| counter.name == counter_name)
                .collect::<Vec<_>>();
            assert_eq!(
                samples.len(),
                1,
                "mixed component frame must publish one counter sample: {counter_name}"
            );
            assert_eq!(samples[0].frame_index, Some(0));
            if let Some(expected_value) = expected_value {
                assert_eq!(samples[0].value, expected_value);
            } else {
                assert!(
                    samples[0].value > 0.0,
                    "mixed component frame must perform one non-empty batch: {counter_name}"
                );
            }
        }
        for (category, name) in [
            ("text.shape_batch", "shape_paragraphs_with_cache"),
            ("ui_text.prewarm", "render_command_text"),
        ] {
            let spans = first_frame_profile
                .spans
                .iter()
                .filter(|span| span.category == category && span.name == name)
                .collect::<Vec<_>>();
            assert_eq!(
                spans.len(),
                1,
                "mixed component frame must publish one span: {category}:{name}"
            );
            assert_eq!(spans[0].frame_index, Some(0));
        }
    }
}

#[cfg(feature = "profiling")]
#[test]
fn owner_prewarm_overlap_keeps_worker_samples_on_the_calling_frame_once() {
    let _capture_guard = test_capture_lock();
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.text.prewarm.profile-overlap"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 320.0, 180.0))
            .with_state_flags(visible_text_state(true)),
    );
    for index in 0..8 {
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                UiTreeNode::new(
                    UiNodeId::new(index + 2),
                    UiNodePath::new(format!("root/profile-label-{index}")),
                )
                .with_frame(UiFrame::new(0.0, index as f32 * 16.0, 220.0, 16.0))
                .with_state_flags(visible_text_state(true))
                .with_template_metadata(text_metadata(&format!("profile owner {index}"))),
            )
            .expect("profile owner label should be inserted");
    }
    let mut config = ProfileCaptureConfig::default();
    config.session_id = "ui-text-owner-overlap-frame".to_string();
    config.max_frames = 2;
    config.max_spans = 64;
    config.max_counters = 128;
    start_capture(config);

    {
        crate::profile_frame!("runtime", "owner_overlap_test_frame");
        surface.rebuild();
    }

    let profile = snapshot();
    reset_capture();
    for (counter_name, expected_value) in [
        ("text.shape_batch.requested", None),
        ("ui_text.extract.commands", None),
        ("ui_text.prewarm.requested", None),
        ("ui_text.rich_cache.parses", None),
        ("ui_text.prewarm.owner_overlap_requests", Some(8.0)),
        ("ui_text.prewarm.owner_overlap_joins", Some(1.0)),
    ] {
        let samples = profile
            .counters
            .iter()
            .filter(|counter| counter.name == counter_name)
            .collect::<Vec<_>>();
        assert_eq!(
            samples.len(),
            1,
            "counter must publish once: {counter_name}"
        );
        assert_eq!(samples[0].frame_index, Some(0));
        if let Some(expected_value) = expected_value {
            assert_eq!(samples[0].value, expected_value);
        }
    }
    for (category, name) in [
        ("text.shape_batch", "shape_paragraphs_with_cache"),
        ("ui_text.extract", "owner_prewarm_request_collection"),
        ("ui_text.extract", "owner_prewarm_overlap_admission"),
        ("ui_text.extract", "render_command_collection"),
        ("ui_text.prewarm", "render_command_text"),
    ] {
        let spans = profile
            .spans
            .iter()
            .filter(|span| span.category == category && span.name == name)
            .collect::<Vec<_>>();
        assert_eq!(spans.len(), 1, "span must publish once: {category}:{name}");
        assert_eq!(spans[0].frame_index, Some(0));
    }
}

#[test]
fn render_extract_prewarms_owner_text_without_losing_clipped_viewport() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.text.prewarm.clipped-owner"));
    let node_id = UiNodeId::new(1);
    surface.tree.insert_root(
        UiTreeNode::new(node_id, UiNodePath::new("root/text"))
            .with_frame(UiFrame::new(0.0, 0.0, 40.0, 48.0))
            .with_state_flags(visible_text_state(true))
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "Label".to_string(),
                attributes: toml::from_str(
                    r#"
text = "Alpha Beta Gamma"
font_size = 10.0
line_height = 12.0
wrap = "glyph"
"#,
                )
                .expect("clipped owner text metadata should parse"),
                ..UiTemplateNodeMetadata::default()
            }),
    );
    surface
        .tree
        .node_mut(node_id)
        .expect("owner text node should exist")
        .layout_cache
        .clip_frame = Some(UiFrame::new(0.0, 0.0, 40.0, 12.0));

    surface.rebuild();

    let command = surface
        .render_extract
        .list
        .commands
        .first()
        .expect("clipped owner command should be extracted");
    let layout = command
        .text_layout
        .as_ref()
        .expect("clipped owner command should be resolved after prewarm");
    let prewarm_report = surface.text_measure_cache.frame_shape_prewarm_report();

    assert_eq!(prewarm_report.requested_count, 1);
    assert_eq!(prewarm_report.cache_miss_count, 1);
    assert_eq!(prewarm_report.shaped_count, 1);
    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, "Alpha Be");
    assert!(layout.overflow_clipped);
}

#[test]
fn render_extract_viewported_owner_defers_full_document_prewarm() {
    let text = (0..10_000)
        .map(|index| format!("r{index:04}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.text.prewarm.huge-log"));
    let node_id = UiNodeId::new(1);
    surface.tree.insert_root(
        UiTreeNode::new(node_id, UiNodePath::new("root/log"))
            .with_frame(UiFrame::new(0.0, 0.0, 240.0, 8.0))
            .with_state_flags(visible_text_state(true))
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "Text".to_string(),
                attributes: toml::from_str(&format!(
                    r#"
text = '''{text}'''
font_size = 10.0
line_height = 20.0
wrap = "none"
text_overflow = "clip"
"#
                ))
                .expect("huge log text metadata should parse"),
                ..UiTemplateNodeMetadata::default()
            }),
    );
    surface
        .tree
        .node_mut(node_id)
        .expect("huge log owner should exist")
        .layout_cache
        .clip_frame = Some(UiFrame::new(0.0, 0.0, 240.0, 8.0));

    surface.rebuild();

    let command = surface
        .render_extract
        .list
        .commands
        .first()
        .expect("huge log owner command should be extracted");
    let layout = command
        .text_layout
        .as_ref()
        .expect("huge log owner must resolve through the viewport path");
    let prewarm_report = surface.text_measure_cache.frame_shape_prewarm_report();
    let shaped_report = surface.text_measure_cache.frame_shaped_run_report();

    assert_eq!(prewarm_report.requested_count, 0);
    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, "r0000");
    assert_eq!(layout.measured_height, 200_000.0);
    assert_eq!(shaped_report.miss_count, 4);
    assert_eq!(shaped_report.insert_count, 4);
}

#[test]
fn render_extract_prewarms_and_layouts_component_text_commands() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.text.prewarm.component-commands"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 320.0, 72.0))
            .with_state_flags(visible_text_state(true)),
    );
    for (node_id, path, frame, text, visible) in [
        (
            UiNodeId::new(2),
            "root/first_button",
            UiFrame::new(0.0, 0.0, 150.0, 22.0),
            "editor base.zui",
            true,
        ),
        (
            UiNodeId::new(3),
            "root/second_button",
            UiFrame::new(160.0, 0.0, 150.0, 22.0),
            "folder-open-outline.svg",
            true,
        ),
        (
            UiNodeId::new(4),
            "root/duplicate_button",
            UiFrame::new(0.0, 28.0, 150.0, 22.0),
            "editor base.zui",
            true,
        ),
        (
            UiNodeId::new(5),
            "root/hidden_button",
            UiFrame::new(160.0, 28.0, 150.0, 22.0),
            "hidden-row-should-not-prewarm",
            false,
        ),
    ] {
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                UiTreeNode::new(node_id, UiNodePath::new(path))
                    .with_frame(frame)
                    .with_state_flags(visible_text_state(visible))
                    .with_template_metadata(button_metadata(text)),
            )
            .expect("button child should be inserted");
    }

    surface.rebuild();

    let text_commands = surface
        .render_extract
        .list
        .commands
        .iter()
        .filter(|command| {
            matches!(command.kind, UiRenderCommandKind::Text)
                && command.text.as_ref().is_some_and(|text| !text.is_empty())
        })
        .collect::<Vec<_>>();
    let prewarm_report = surface.text_measure_cache.frame_shape_prewarm_report();

    assert_eq!(text_commands.len(), 3);
    assert!(
        text_commands
            .iter()
            .all(|command| command.text_layout.is_some()),
        "component generated text commands should be resolved before retained-host fallback"
    );
    assert_eq!(
        prewarm_report.requested_count, 3,
        "only visible component text commands should be collected for automatic prewarm"
    );
    assert_eq!(prewarm_report.cache_hit_count, 0);
    assert_eq!(prewarm_report.cache_miss_count, 2);
    assert_eq!(prewarm_report.batch_duplicate_count, 1);
    assert_eq!(prewarm_report.shaped_count, 2);
    assert_eq!(prewarm_report.inserted_count, 2);
}

#[test]
fn render_extract_prewarms_rich_and_vertical_owner_text_before_layout() {
    #[cfg(feature = "profiling")]
    let _capture_guard = test_capture_lock();
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.text.prewarm.rich-vertical"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 260.0, 80.0))
            .with_state_flags(visible_text_state(true)),
    );
    for (node_id, path, frame, metadata) in [
        (
            UiNodeId::new(2),
            "root/rich_first",
            UiFrame::new(0.0, 0.0, 220.0, 16.0),
            rich_text_metadata("m0-profile-marker **sample**"),
        ),
        (
            UiNodeId::new(3),
            "root/vertical",
            UiFrame::new(0.0, 18.0, 48.0, 48.0),
            vertical_text_metadata("folder-open-outline.svg"),
        ),
        (
            UiNodeId::new(4),
            "root/rich_duplicate",
            UiFrame::new(0.0, 66.0, 220.0, 16.0),
            rich_text_metadata("m0-profile-marker **sample**"),
        ),
    ] {
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                UiTreeNode::new(node_id, UiNodePath::new(path))
                    .with_frame(frame)
                    .with_state_flags(visible_text_state(true))
                    .with_template_metadata(metadata),
            )
            .expect("text child should be inserted");
    }

    #[cfg(feature = "profiling")]
    let rich_cache_profile = {
        let mut config = ProfileCaptureConfig::default();
        config.session_id = "ui-text-rich-cache-frame".to_string();
        config.max_frames = 2;
        config.max_spans = 32;
        config.max_counters = 128;
        start_capture(config);
        {
            crate::profile_frame!("runtime", "rich_cache_test_frame");
            surface.rebuild();
        }
        let profile = snapshot();
        reset_capture();
        profile
    };
    #[cfg(not(feature = "profiling"))]
    surface.rebuild();

    let prewarm_report = surface.text_measure_cache.frame_shape_prewarm_report();
    let shaped_report = surface.text_measure_cache.frame_shaped_run_report();
    let rich_command = surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == UiNodeId::new(2))
        .expect("rich text command should be present");
    let vertical_command = surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == UiNodeId::new(3))
        .expect("vertical text command should be present");

    assert_eq!(text_layout_command_count(&surface), 3);
    assert_eq!(
        rich_command
            .text_layout
            .as_ref()
            .and_then(|layout| layout.lines.first())
            .map(|line| line.text.as_str()),
        Some("m0-profile-marker sample"),
        "rich-text prewarm must use the same visible text that layout resolves"
    );
    assert_eq!(
        vertical_command
            .text_layout
            .as_ref()
            .map(|layout| layout.writing_mode),
        Some(UiTextWritingMode::VerticalRl)
    );
    assert_eq!(prewarm_report.requested_count, 3);
    assert_eq!(prewarm_report.cache_miss_count, 2);
    assert_eq!(prewarm_report.batch_duplicate_count, 1);
    assert_eq!(prewarm_report.shaped_count, 2);
    assert_eq!(prewarm_report.inserted_count, 2);
    assert!(
        shaped_report.hit_count >= prewarm_report.requested_count as u64,
        "rich and vertical layout should consume the prewarmed shaped runs"
    );
    #[cfg(feature = "profiling")]
    {
        let parses = rich_cache_profile
            .counters
            .iter()
            .filter(|counter| counter.name == "ui_text.rich_cache.parses")
            .collect::<Vec<_>>();
        assert_eq!(
            parses.len(),
            1,
            "rich-cache counters publish once per caller frame"
        );
        assert_eq!(parses[0].frame_index, Some(0));
        assert!(
            parses[0].value >= 1.0,
            "Markdown source must produce a compiled-rich parse in its caller frame"
        );
    }
}
