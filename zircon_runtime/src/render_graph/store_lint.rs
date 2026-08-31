use std::collections::BTreeMap;

use super::{
    CompiledRenderGraph, RenderGraphAttachmentLoadOp, RenderGraphAttachmentStoreOp,
    RenderGraphResourceAccessKind, RenderGraphResourceDesc,
};
use crate::rhi::TextureFormat;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderGraphStoreLintKind {
    NeedlessLoad,
    DeadStore,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderGraphStoreLintRow {
    pub pass_name: String,
    pub resource_name: String,
    pub kind: RenderGraphStoreLintKind,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderGraphStoreLintReport {
    pub rows: Vec<RenderGraphStoreLintRow>,
}

impl RenderGraphStoreLintReport {
    pub fn count(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderGraphAttachmentBandwidthRow {
    pub resource_name: String,
    pub format: TextureFormat,
    pub bytes_per_pixel: u32,
    pub load_count: u32,
    pub store_count: u32,
    pub read_bytes_per_frame: u64,
    pub write_bytes_per_frame: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderGraphAttachmentBandwidthLedger {
    pub rows: Vec<RenderGraphAttachmentBandwidthRow>,
}

impl RenderGraphAttachmentBandwidthLedger {
    pub fn total_bytes_per_frame(&self) -> u64 {
        self.rows.iter().fold(0_u64, |total, row| {
            total
                .saturating_add(row.read_bytes_per_frame)
                .saturating_add(row.write_bytes_per_frame)
        })
    }
}

impl CompiledRenderGraph {
    pub fn store_lint_report(&self) -> RenderGraphStoreLintReport {
        let mut rows = Vec::new();
        for (pass_index, pass) in self.passes().iter().enumerate() {
            if pass.culled {
                continue;
            }
            for access in &pass.resources {
                let Some(ops) = access.attachment_ops else {
                    continue;
                };
                let Some(lifetime) = self.resource_lifetime_by_name(&access.name) else {
                    continue;
                };

                if ops.load == RenderGraphAttachmentLoadOp::Load
                    && !lifetime.imported
                    && !has_prior_write(self, pass_index, &access.name)
                {
                    rows.push(RenderGraphStoreLintRow {
                        pass_name: pass.name.clone(),
                        resource_name: access.name.clone(),
                        kind: RenderGraphStoreLintKind::NeedlessLoad,
                    });
                }

                if ops.store == RenderGraphAttachmentStoreOp::Store
                    && !lifetime.imported
                    && !lifetime.usage.is_cull_root()
                    && !has_future_read_before_overwrite(self, pass_index, &access.name)
                {
                    rows.push(RenderGraphStoreLintRow {
                        pass_name: pass.name.clone(),
                        resource_name: access.name.clone(),
                        kind: RenderGraphStoreLintKind::DeadStore,
                    });
                }
            }
        }
        RenderGraphStoreLintReport { rows }
    }

    pub fn attachment_bandwidth_ledger(&self) -> RenderGraphAttachmentBandwidthLedger {
        let mut rows = BTreeMap::<String, RenderGraphAttachmentBandwidthRow>::new();
        for pass in self.passes().iter().filter(|pass| !pass.culled) {
            for access in &pass.resources {
                let Some(ops) = access.attachment_ops else {
                    continue;
                };
                let Some(lifetime) = self.resource_lifetime_by_name(&access.name) else {
                    continue;
                };
                let RenderGraphResourceDesc::Texture(desc) = &lifetime.desc else {
                    continue;
                };
                let bytes_per_pixel = desc.format.bytes_per_pixel();
                let surface_bytes = u64::from(desc.width)
                    .saturating_mul(u64::from(desc.height))
                    .saturating_mul(u64::from(desc.depth))
                    .saturating_mul(u64::from(desc.sample_count))
                    .saturating_mul(u64::from(bytes_per_pixel));
                let row = rows.entry(access.name.clone()).or_insert_with(|| {
                    RenderGraphAttachmentBandwidthRow {
                        resource_name: access.name.clone(),
                        format: desc.format,
                        bytes_per_pixel,
                        load_count: 0,
                        store_count: 0,
                        read_bytes_per_frame: 0,
                        write_bytes_per_frame: 0,
                    }
                });
                if ops.load == RenderGraphAttachmentLoadOp::Load {
                    row.load_count = row.load_count.saturating_add(1);
                    row.read_bytes_per_frame =
                        row.read_bytes_per_frame.saturating_add(surface_bytes);
                }
                if ops.store == RenderGraphAttachmentStoreOp::Store {
                    row.store_count = row.store_count.saturating_add(1);
                    row.write_bytes_per_frame =
                        row.write_bytes_per_frame.saturating_add(surface_bytes);
                }
            }
        }
        RenderGraphAttachmentBandwidthLedger {
            rows: rows.into_values().collect(),
        }
    }
}

fn has_prior_write(graph: &CompiledRenderGraph, pass_index: usize, resource_name: &str) -> bool {
    graph.passes().iter().take(pass_index).any(|pass| {
        !pass.culled
            && pass.resources.iter().any(|access| {
                access.name == resource_name
                    && access.access == RenderGraphResourceAccessKind::Write
            })
    })
}

fn has_future_read_before_overwrite(
    graph: &CompiledRenderGraph,
    pass_index: usize,
    resource_name: &str,
) -> bool {
    for pass in graph.passes().iter().skip(pass_index.saturating_add(1)) {
        if pass.culled {
            continue;
        }
        for access in &pass.resources {
            if access.name != resource_name {
                continue;
            }
            match access.access {
                RenderGraphResourceAccessKind::Read => return true,
                RenderGraphResourceAccessKind::Write => return false,
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use crate::render_graph::{
        PassFlags, QueueLane, RenderGraphAttachmentOps, RenderGraphBuilder,
        RenderGraphStoreLintKind,
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
}
