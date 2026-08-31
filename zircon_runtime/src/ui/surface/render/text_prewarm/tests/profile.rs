use crate::core::runtime::diagnostics::profiling::{
    ProfileCaptureConfig, ProfileSnapshot, reset_capture, snapshot, start_capture,
    test_capture_lock,
};
use crate::text::TextDocumentKey;
use crate::ui::surface::UiSurface;
use crate::ui::text::{UiTextMeasureCache, UiTextViewport};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    surface::{UiRenderCommand, UiRenderCommandKind, UiResolvedStyle, UiRichTextFormat},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

use super::{PendingOwnerTextLayouts, resolve_missing_render_command_text_layouts};

#[test]
fn render_command_profile_records_fixed_extract_prewarm_and_layout_stages() {
    let _capture_guard = test_capture_lock();
    let mut config = ProfileCaptureConfig::default();
    config.session_id = "ui-text-render-command-profile".to_string();
    config.max_spans = 8;
    config.max_counters = 160;
    start_capture(config);

    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.text.profile"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root/text"))
            .with_frame(UiFrame::new(0.0, 0.0, 180.0, 24.0))
            .with_state_flags(UiStateFlags {
                visible: true,
                enabled: true,
                ..UiStateFlags::default()
            })
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "Text".to_string(),
                attributes: toml::from_str(
                    r#"
text = "profiled layout"
font_size = 10.0
line_height = 12.0
wrap = "None"
"#,
                )
                .expect("profile text metadata should parse"),
                ..UiTemplateNodeMetadata::default()
            }),
    );
    surface.rebuild();
    let profile = snapshot();
    reset_capture();

    for category in [
        "ui_text.extract",
        "ui_text.prewarm",
        "ui_text.layout_resolve",
    ] {
        assert!(
            profile.spans.iter().any(|span| span.category == category),
            "{category} scope must be visible to the shared frame profiler"
        );
    }
    assert_eq!(counter_value(&profile, "ui_text.extract.commands"), 1.0);
    assert_eq!(counter_value(&profile, "ui_text.extract.owner_text"), 1.0);
    assert_eq!(counter_value(&profile, "ui_text.prewarm.requested"), 1.0);
    assert_eq!(counter_value(&profile, "ui_text.prewarm.cache_misses"), 1.0);
    assert_eq!(counter_value(&profile, "ui_text.prewarm.shaped"), 1.0);
    assert_eq!(counter_value(&profile, "ui_text.prewarm.inserted"), 1.0);
    assert!(
        profile
            .counters
            .iter()
            .any(|counter| counter.name == "ui_text.layout_resolve.cache_misses"),
        "layout cache counters must be projected after the layout scope closes"
    );
    assert_eq!(
        counter_value(&profile, "ui_text.layout_resolve.cache_hits"),
        0.0,
        "the first complete Plain owner layout must not claim a persistent layout-cache hit"
    );
    assert_eq!(
        counter_value(&profile, "ui_text.layout_resolve.cache_misses"),
        1.0,
        "a complete Plain owner layout must populate the persistent layout cache"
    );
    assert_eq!(
        counter_value(&profile, "ui_text.layout_resolve.cache_lookup_candidates"),
        0.0,
        "a cold persistent-cache miss must not claim a bucket candidate probe"
    );
    assert_eq!(
        counter_value(&profile, "ui_text.layout_resolve.cache_eviction_scans"),
        0.0,
        "a first layout below capacity must not evict a persistent entry"
    );
    assert_eq!(
        counter_value(&profile, "ui_text.layout_resolve.cache_entry_moves"),
        0.0,
        "a first layout below capacity must not move a persistent cache slot"
    );
    assert_eq!(
        counter_value(&profile, "ui_text.layout_resolve.cache_evictions"),
        0.0,
        "a first layout below capacity must not evict a persistent entry"
    );
    assert_eq!(
        counter_value(
            &profile,
            "ui_text.layout_resolve.uncached_document_resolves"
        ),
        0.0,
        "a complete viewport must not be reported as a virtualized-document cache bypass"
    );
    assert!(
        counter_value(&profile, "ui_text.layout_resolve.shape_cache_hits") >= 1.0,
        "layout must report consuming the source run inserted by prewarm"
    );
    assert!(
        counter_value(&profile, "ui_text.layout_resolve.shape_cache_misses") >= 1.0,
        "layout must report the fixed metrics run it shapes after prewarm"
    );
    assert!(
        counter_value(
            &profile,
            "ui_text.layout_resolve.shape_cache_lookup_candidates"
        ) >= 1.0,
        "layout must expose exact shaped-run cache lookup candidates"
    );
    assert!(
        counter_value(&profile, "ui_text.layout_resolve.shape_cache_inserts") >= 1.0,
        "layout must report shaped runs inserted after prewarm"
    );
    assert!(
        counter_value(
            &profile,
            "ui_text.layout_resolve.shape_cache_owned_key_allocation_bytes"
        ) >= 1.0,
        "layout must expose owned shaped-cache keys when a miss creates one"
    );
    assert_eq!(
        counter_value(
            &profile,
            "ui_text.layout_resolve.shape_cache_eviction_scans"
        ),
        0.0,
        "the first shaped runs must not evict an entry below capacity"
    );
    assert_eq!(
        counter_value(&profile, "ui_text.layout_resolve.shape_cache_entry_moves"),
        0.0,
        "the first shaped runs must not move a cache slot"
    );
    assert_eq!(
        counter_value(&profile, "ui_text.layout_resolve.shape_cache_evictions"),
        0.0,
        "the first shaped runs must not evict an entry below capacity"
    );
    assert!(
        counter_value(&profile, "text_analysis_request_count") >= 1.0,
        "the complete UI path must retain shape-request analysis construction counts"
    );
    assert!(
        counter_value(&profile, "text_analysis_bidi_build_count") >= 1.0,
        "the complete UI path must expose Bidi construction work"
    );
    assert!(
        counter_value(&profile, "text_analysis_script_emoji_build_count") >= 1.0,
        "the complete UI path must expose script/emoji construction work"
    );
    assert!(
        counter_value(&profile, "text_analysis_line_break_build_count") >= 1.0,
        "the complete UI path must expose line-break construction work"
    );
    assert!(
        counter_value(
            &profile,
            "text_font_fallback_cache_state_lock_acquire_count"
        ) >= 0.0,
        "fallback cache lock work must retain a fixed counter even on a primary fast path"
    );
}

#[test]
fn rich_owner_profile_does_not_report_plain_document_layout_bypass() {
    let _capture_guard = test_capture_lock();
    let mut config = ProfileCaptureConfig::default();
    config.session_id = "ui-text-rich-owner-layout-profile".to_string();
    config.max_spans = 4;
    config.max_counters = 160;
    start_capture(config);

    let mut commands = vec![UiRenderCommand {
        node_id: UiNodeId::new(1),
        kind: UiRenderCommandKind::Text,
        frame: UiFrame::new(0.0, 0.0, 180.0, 24.0),
        clip_frame: None,
        z_index: 0,
        style: UiResolvedStyle {
            rich_text_format: UiRichTextFormat::MarkdownInlineV1,
            font_size: 10.0,
            line_height: 12.0,
            ..UiResolvedStyle::default()
        },
        text_layout: None,
        text: Some("**cached rich owner**".to_string()),
        image: None,
        opacity: 1.0,
    }];
    let mut pending = PendingOwnerTextLayouts::default();
    pending.push(
        0,
        Some(TextDocumentKey::new(7, 1)),
        UiTextViewport::new(0.0, 24.0, 2),
        None,
    );
    let mut cache = UiTextMeasureCache::default();
    cache.begin_frame();
    resolve_missing_render_command_text_layouts(&mut commands, &pending, &mut cache);
    let profile = snapshot();
    reset_capture();

    assert!(commands[0].text_layout.is_some());
    assert_eq!(
        counter_value(
            &profile,
            "ui_text.layout_resolve.uncached_document_resolves"
        ),
        0.0,
        "rich owner layout must not enter the Plain retained-document bypass"
    );
    assert_eq!(
        counter_value(&profile, "ui_text.layout_resolve.cache_hits"),
        0.0
    );
    assert_eq!(
        counter_value(&profile, "ui_text.layout_resolve.cache_misses"),
        1.0,
        "the non-bypass owner must exercise the persistent layout cache"
    );
}

fn counter_value(profile: &ProfileSnapshot, name: &str) -> f64 {
    profile
        .counters
        .iter()
        .find(|counter| counter.stream == "runtime" && counter.name == name)
        .map(|counter| counter.value)
        .unwrap_or_else(|| panic!("missing profile counter: {name}"))
}
