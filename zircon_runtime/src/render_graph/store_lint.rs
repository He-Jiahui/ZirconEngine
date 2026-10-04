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

/// Builds the store-lint report once while a graph is compiled.
// 编译期快照供帧统计复用，避免 steady frame 再扫描 pass/resource 生命周期。
pub(crate) fn build_store_lint_report(graph: &CompiledRenderGraph) -> RenderGraphStoreLintReport {
    let mut rows = Vec::new();
    for (pass_index, pass) in graph.passes().iter().enumerate() {
        if pass.culled {
            continue;
        }
        for access in &pass.resources {
            let Some(ops) = access.attachment_ops else {
                continue;
            };
            let Some(lifetime) = graph.resource_lifetime_by_name(&access.name) else {
                continue;
            };

            if ops.load == RenderGraphAttachmentLoadOp::Load
                && !lifetime.imported
                && !has_prior_write(graph, pass_index, &access.name)
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
                && !has_future_read_before_overwrite(graph, pass_index, &access.name)
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

/// Builds the attachment bandwidth ledger once while a graph is compiled.
// ledger 按图内 Texture descriptor 估算 live attachment 的整面 load/store 字节；
// External 分支在此跳过，即使另带 typed external descriptor 也不计入。
pub(crate) fn build_attachment_bandwidth_ledger(
    graph: &CompiledRenderGraph,
) -> RenderGraphAttachmentBandwidthLedger {
    let mut rows = BTreeMap::<String, RenderGraphAttachmentBandwidthRow>::new();
    for pass in graph.passes().iter().filter(|pass| !pass.culled) {
        for access in &pass.resources {
            let Some(ops) = access.attachment_ops else {
                continue;
            };
            let Some(lifetime) = graph.resource_lifetime_by_name(&access.name) else {
                continue;
            };
            let RenderGraphResourceDesc::Texture(desc) = &lifetime.desc else {
                continue;
            };
            let bytes_per_pixel = desc.format.bytes_per_pixel();
            let surface_bytes = u64::from(desc.width)
                .saturating_mul(u64::from(desc.height))
                .saturating_mul(u64::from(desc.depth_or_array_layers()))
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
                row.read_bytes_per_frame = row.read_bytes_per_frame.saturating_add(surface_bytes);
            }
            if ops.store == RenderGraphAttachmentStoreOp::Store {
                row.store_count = row.store_count.saturating_add(1);
                row.write_bytes_per_frame = row.write_bytes_per_frame.saturating_add(surface_bytes);
            }
        }
    }
    RenderGraphAttachmentBandwidthLedger {
        rows: rows.into_values().collect(),
    }
}

impl CompiledRenderGraph {
    /// Returns the compile-time store-lint artifact without rescanning passes.
    pub fn store_lint_report(&self) -> RenderGraphStoreLintReport {
        self.store_lint_report.clone()
    }

    /// Returns the compile-time lint count for steady-frame diagnostics.
    pub(crate) fn store_lint_count(&self) -> usize {
        self.store_lint_report.count()
    }

    /// Returns the compile-time attachment bandwidth artifact without rescanning passes.
    pub fn attachment_bandwidth_ledger(&self) -> RenderGraphAttachmentBandwidthLedger {
        self.attachment_bandwidth_ledger.clone()
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
#[path = "tests/store_lint.rs"]
mod tests;
