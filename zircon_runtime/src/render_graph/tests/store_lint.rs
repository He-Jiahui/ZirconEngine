use crate::render_graph::{
    PassFlags, QueueLane, RenderGraphAttachmentOps, RenderGraphBuilder, RenderGraphStoreLintKind,
};
use crate::rhi::{TextureDesc, TextureFormat, TextureUsage};

#[test]
fn render_perf_store_lint_detects_dead_store() {
    let mut builder = RenderGraphBuilder::new("dead-store");
    let color = builder.create_texture(TextureDesc::new(
        "unused-color",
        64,
        32,
        TextureFormat::Rgba16Float,
        TextureUsage::RENDER_ATTACHMENT,
    ));
    let terminal = builder.add_pass("terminal-write", QueueLane::Graphics);
    builder
        .set_pass_flags(
            terminal,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();
    builder
        .write_texture_with_ops(terminal, color, RenderGraphAttachmentOps::clear_store())
        .unwrap();

    let graph = builder.compile().unwrap();
    let report = graph.store_lint_report();

    assert_eq!(report.rows.len(), 1);
    assert_eq!(report.rows[0].kind, RenderGraphStoreLintKind::DeadStore);
    assert_eq!(report.rows[0].pass_name, "terminal-write");
    assert_eq!(report.rows[0].resource_name, "unused-color");
    assert_eq!(graph.store_lint_count(), report.count());
}

#[test]
fn render_perf_store_lint_and_bandwidth_reads_use_compiled_artifacts() {
    let normalized_store_lint_source = include_str!("../store_lint.rs").replace("\r\n", "\n");
    let production = normalized_store_lint_source
        .split("#[cfg(test)]")
        .next()
        .expect("store-lint production source");
    assert!(production.contains(
        "pub fn store_lint_report(&self) -> RenderGraphStoreLintReport {\n        self.store_lint_report.clone()"
    ));
    assert!(production.contains(
        "pub(crate) fn store_lint_count(&self) -> usize {\n        self.store_lint_report.count()"
    ));
    assert!(production.contains(
        "pub fn attachment_bandwidth_ledger(&self) -> RenderGraphAttachmentBandwidthLedger {\n        self.attachment_bandwidth_ledger.clone()"
    ));

    let report_accessor = production
        .split("pub fn store_lint_report")
        .nth(1)
        .expect("compiled store-lint report accessor")
        .split("pub(crate) fn store_lint_count")
        .next()
        .expect("store-lint count boundary");
    assert!(!report_accessor.contains("self.passes()"));

    let ledger_accessor = production
        .split("pub fn attachment_bandwidth_ledger")
        .nth(1)
        .expect("compiled attachment ledger accessor")
        .split("}\n")
        .next()
        .expect("attachment ledger accessor body");
    assert!(!ledger_accessor.contains("self.passes()"));

    let graph_source = include_str!("../graph.rs");
    assert!(graph_source.contains("store_lint_report: RenderGraphStoreLintReport"));
    assert!(
        graph_source.contains("attachment_bandwidth_ledger: RenderGraphAttachmentBandwidthLedger")
    );
    assert!(graph_source.contains("let store_lint_report = build_store_lint_report(&graph);"));
    assert!(graph_source
        .contains("let attachment_bandwidth_ledger = build_attachment_bandwidth_ledger(&graph);"));

    let update_stats_source = include_str!(
        "../../graphics/runtime/render_framework/submit_frame_extract/update_stats/update.rs"
    );
    assert!(update_stats_source.contains(".store_lint_count()"));
    assert!(!update_stats_source.contains(".store_lint_report().count()"));
}

#[test]
fn render_perf_attachment_bandwidth_ledger_uses_format_and_attachment_ops() {
    let mut builder = RenderGraphBuilder::new("bandwidth-ledger");
    let color = builder.create_texture(TextureDesc::new(
        "hdr-color",
        64,
        32,
        TextureFormat::Rgba16Float,
        TextureUsage::RENDER_ATTACHMENT,
    ));
    let output = builder.import_present_external_resource("output");
    let draw = builder.add_pass("draw", QueueLane::Graphics);
    let present = builder.add_pass("present", QueueLane::Graphics);
    builder
        .write_texture_with_ops(draw, color, RenderGraphAttachmentOps::clear_store())
        .unwrap();
    builder.read_texture(present, color).unwrap();
    builder.write_external(present, output).unwrap();

    let graph = builder.compile().unwrap();
    let ledger = graph.attachment_bandwidth_ledger();

    assert_eq!(ledger.rows.len(), 1);
    assert_eq!(ledger.rows[0].resource_name, "hdr-color");
    assert_eq!(ledger.rows[0].bytes_per_pixel, 8);
    assert_eq!(ledger.rows[0].load_count, 0);
    assert_eq!(ledger.rows[0].store_count, 1);
    assert_eq!(ledger.rows[0].write_bytes_per_frame, 64 * 32 * 8);
    assert_eq!(ledger.total_bytes_per_frame(), 64 * 32 * 8);
}
