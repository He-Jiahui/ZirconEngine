use crate::rhi::{TextureDimension, TextureFormat, TextureResidency};

use super::graph::{CompiledRenderGraph, CompiledRenderGraphStats};
use super::types::{
    QueueLane, RenderGraphAttachmentOps, RenderGraphComputeWorkload, RenderGraphPassResourceAccess,
    RenderGraphResourceAccessKind, RenderGraphResourceDeclaration, RenderGraphResourceDesc,
    RenderGraphResourceKind, RenderGraphResourceLifetime, RenderGraphResourceUsageFlags,
    RenderPassId,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderGraphDump {
    pub graph_name: String,
    pub stats: CompiledRenderGraphStats,
    pub pass_rows: Vec<RenderGraphDumpPassRow>,
    pub resource_rows: Vec<RenderGraphDumpResourceRow>,
    pub transient_slot_rows: Vec<RenderGraphDumpTransientSlotRow>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderGraphDumpPassRow {
    pub order: usize,
    pub id: RenderPassId,
    pub name: String,
    pub declared_queue: QueueLane,
    pub queue: QueueLane,
    pub queue_fallback: bool,
    pub culled: bool,
    pub allow_culling: bool,
    pub has_side_effects: bool,
    pub dependencies: Vec<RenderPassId>,
    pub executor_id: Option<String>,
    pub compute_workload: Option<RenderGraphComputeWorkload>,
    pub resources: Vec<RenderGraphDumpPassResourceRow>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderGraphDumpPassResourceRow {
    pub name: String,
    pub kind: RenderGraphResourceKind,
    pub access: RenderGraphResourceAccessKind,
    pub attachment_ops: Option<RenderGraphAttachmentOps>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderGraphDumpResourceRow {
    pub name: String,
    pub kind: RenderGraphResourceKind,
    pub desc: RenderGraphDumpResourceDesc,
    pub imported: bool,
    pub usage: RenderGraphResourceUsageFlags,
    pub live: bool,
    pub first_pass: Option<usize>,
    pub last_pass: Option<usize>,
    pub transient_slot: Option<usize>,
    pub size_bytes: Option<u64>,
    pub slot_reserved_bytes: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderGraphDumpResourceDesc {
    Texture {
        width: u32,
        height: u32,
        depth: u32,
        mip_levels: u32,
        sample_count: u32,
        format: TextureFormat,
        usage_bits: u32,
        dimension: TextureDimension,
        residency: TextureResidency,
    },
    Buffer {
        size_bytes: u64,
        usage_bits: u32,
    },
    External,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderGraphDumpTransientSlotRow {
    pub kind: RenderGraphResourceKind,
    pub slot: usize,
    pub bucket_key_hash: u64,
    pub bytes_reserved: u64,
}

impl RenderGraphDump {
    pub fn from_graph(graph: &CompiledRenderGraph) -> Self {
        let allocation_plan = graph.transient_allocation_plan();
        let pass_rows = graph
            .passes()
            .iter()
            .enumerate()
            .map(|(order, pass)| RenderGraphDumpPassRow {
                order,
                id: pass.id,
                name: pass.name.clone(),
                declared_queue: pass.declared_queue,
                queue: pass.queue,
                queue_fallback: pass.declared_queue != pass.queue,
                culled: pass.culled,
                allow_culling: pass.flags.allow_culling,
                has_side_effects: pass.flags.has_side_effects,
                dependencies: pass.dependencies.clone(),
                executor_id: pass.executor_id.clone(),
                compute_workload: pass.compute_workload.clone(),
                resources: pass.resources.iter().map(pass_resource_row).collect(),
            })
            .collect();
        let resource_rows = graph
            .resource_declarations()
            .iter()
            .map(|declaration| {
                let lifetime = graph.resource_lifetime(declaration.resource);
                let allocation = allocation_plan
                    .allocations
                    .iter()
                    .find(|allocation| allocation.resource_name == declaration.name);
                let slot_reserved_bytes = allocation.and_then(|allocation| {
                    allocation_plan.slot_bytes_for_bucket(
                        allocation.kind,
                        allocation.slot,
                        allocation.bucket_key_hash,
                    )
                });
                resource_row(declaration, lifetime, allocation, slot_reserved_bytes)
            })
            .collect();
        let transient_slot_rows = allocation_plan
            .slot_reservations
            .iter()
            .map(|reservation| RenderGraphDumpTransientSlotRow {
                kind: reservation.kind,
                slot: reservation.slot,
                bucket_key_hash: reservation.bucket_key_hash,
                bytes_reserved: reservation.bytes_reserved,
            })
            .collect();

        Self {
            graph_name: graph.name().to_owned(),
            stats: graph.stats(),
            pass_rows,
            resource_rows,
            transient_slot_rows,
        }
    }

    pub fn to_text(&self) -> String {
        let mut text = String::new();
        text.push_str(&format!(
            "render_graph name={} passes={} executable={} culled={} resources={}\n",
            self.graph_name,
            self.stats.total_pass_count,
            self.stats.executable_pass_count,
            self.stats.culled_pass_count,
            self.stats.resource_lifetime_count
        ));
        text.push_str("passes:\n");
        for pass in &self.pass_rows {
            text.push_str(&format!(
                "  pass[{}] id={} name={} queue={} declared_queue={} fallback={} culled={} executor={} deps={} resources={}\n",
                pass.order,
                pass.id.index(),
                pass.name,
                queue_lane_label(pass.queue),
                queue_lane_label(pass.declared_queue),
                pass.queue_fallback,
                pass.culled,
                pass.executor_id.as_deref().unwrap_or("-"),
                dependency_text(&pass.dependencies),
                pass.resources.len()
            ));
            for resource in &pass.resources {
                text.push_str(&format!(
                    "    {} {} kind={} ops={}\n",
                    resource_access_label(resource.access),
                    resource.name,
                    resource_kind_label(resource.kind),
                    attachment_ops_text(resource.attachment_ops)
                ));
            }
            if let Some(workload) = &pass.compute_workload {
                text.push_str(&format!(
                    "    compute label={} workgroup={:?} extent={:?}\n",
                    workload.pipeline_label, workload.workgroup_size, workload.dispatch_extent
                ));
            }
        }
        text.push_str("resources:\n");
        for resource in &self.resource_rows {
            text.push_str(&format!(
                "  resource name={} kind={} imported={} usage={} live={} lifetime={} slot={} size_bytes={} slot_reserved_bytes={} desc={}\n",
                resource.name,
                resource_kind_label(resource.kind),
                resource.imported,
                resource_usage_text(resource.usage),
                resource.live,
                lifetime_text(resource.first_pass, resource.last_pass),
                optional_usize_text(resource.transient_slot),
                optional_u64_text(resource.size_bytes),
                optional_u64_text(resource.slot_reserved_bytes),
                resource_desc_text(&resource.desc)
            ));
        }
        text.push_str("transient_slots:\n");
        for slot in &self.transient_slot_rows {
            text.push_str(&format!(
                "  slot kind={} index={} bucket={} bytes_reserved={}\n",
                resource_kind_label(slot.kind),
                slot.slot,
                slot.bucket_key_hash,
                slot.bytes_reserved
            ));
        }
        text
    }
}

fn pass_resource_row(access: &RenderGraphPassResourceAccess) -> RenderGraphDumpPassResourceRow {
    RenderGraphDumpPassResourceRow {
        name: access.name.clone(),
        kind: access.kind,
        access: access.access,
        attachment_ops: access.attachment_ops,
    }
}

fn resource_row(
    declaration: &RenderGraphResourceDeclaration,
    lifetime: Option<&RenderGraphResourceLifetime>,
    allocation: Option<&super::graph::CompiledRenderGraphTransientAllocation>,
    slot_reserved_bytes: Option<u64>,
) -> RenderGraphDumpResourceRow {
    RenderGraphDumpResourceRow {
        name: declaration.name.clone(),
        kind: declaration.kind,
        desc: resource_desc(&declaration.desc),
        imported: declaration.imported,
        usage: declaration.usage,
        live: lifetime.is_some(),
        first_pass: lifetime.map(|lifetime| lifetime.first_pass),
        last_pass: lifetime.map(|lifetime| lifetime.last_pass),
        transient_slot: allocation.map(|allocation| allocation.slot),
        size_bytes: allocation.map(|allocation| allocation.size_bytes),
        slot_reserved_bytes,
    }
}

fn resource_desc(desc: &RenderGraphResourceDesc) -> RenderGraphDumpResourceDesc {
    match desc {
        RenderGraphResourceDesc::Texture(desc) => RenderGraphDumpResourceDesc::Texture {
            width: desc.width,
            height: desc.height,
            depth: desc.depth,
            mip_levels: desc.mip_levels,
            sample_count: desc.sample_count,
            format: desc.format,
            usage_bits: desc.usage.bits(),
            dimension: desc.dimension,
            residency: desc.residency,
        },
        RenderGraphResourceDesc::Buffer(desc) => RenderGraphDumpResourceDesc::Buffer {
            size_bytes: desc.size_bytes,
            usage_bits: desc.usage.bits(),
        },
        RenderGraphResourceDesc::External => RenderGraphDumpResourceDesc::External,
    }
}

const fn queue_lane_label(queue: QueueLane) -> &'static str {
    match queue {
        QueueLane::Graphics => "Graphics",
        QueueLane::AsyncCompute => "AsyncCompute",
        QueueLane::AsyncCopy => "AsyncCopy",
    }
}

const fn resource_kind_label(kind: RenderGraphResourceKind) -> &'static str {
    match kind {
        RenderGraphResourceKind::TransientTexture => "TransientTexture",
        RenderGraphResourceKind::TransientBuffer => "TransientBuffer",
        RenderGraphResourceKind::External => "External",
    }
}

fn resource_usage_text(usage: RenderGraphResourceUsageFlags) -> String {
    let mut labels = Vec::new();
    if usage.present {
        labels.push("present");
    }
    if usage.readback {
        labels.push("readback");
    }
    if usage.persistent {
        labels.push("persistent");
    }
    if labels.is_empty() {
        "-".to_owned()
    } else {
        labels.join("|")
    }
}

const fn resource_access_label(access: RenderGraphResourceAccessKind) -> &'static str {
    match access {
        RenderGraphResourceAccessKind::Read => "read",
        RenderGraphResourceAccessKind::Write => "write",
    }
}

fn dependency_text(dependencies: &[RenderPassId]) -> String {
    if dependencies.is_empty() {
        "-".to_owned()
    } else {
        dependencies
            .iter()
            .map(|dependency| dependency.index().to_string())
            .collect::<Vec<_>>()
            .join(",")
    }
}

fn attachment_ops_text(ops: Option<RenderGraphAttachmentOps>) -> String {
    ops.map_or_else(
        || "-".to_owned(),
        |ops| format!("{:?}/{:?}", ops.load, ops.store),
    )
}

fn lifetime_text(first_pass: Option<usize>, last_pass: Option<usize>) -> String {
    match (first_pass, last_pass) {
        (Some(first), Some(last)) => format!("{first}..{last}"),
        _ => "-".to_owned(),
    }
}

fn optional_usize_text(value: Option<usize>) -> String {
    value.map_or_else(|| "-".to_owned(), |value| value.to_string())
}

fn optional_u64_text(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_owned(), |value| value.to_string())
}

fn resource_desc_text(desc: &RenderGraphDumpResourceDesc) -> String {
    match desc {
        RenderGraphDumpResourceDesc::Texture {
            width,
            height,
            depth,
            mip_levels,
            sample_count,
            format,
            usage_bits,
            dimension,
            residency,
        } => format!(
            "texture:{}x{}x{} mips={} samples={} format={:?} usage=0x{:x} dimension={:?} residency={:?}",
            width, height, depth, mip_levels, sample_count, format, usage_bits, dimension, residency
        ),
        RenderGraphDumpResourceDesc::Buffer {
            size_bytes,
            usage_bits,
        } => format!("buffer:size_bytes={} usage=0x{:x}", size_bytes, usage_bits),
        RenderGraphDumpResourceDesc::External => "external".to_owned(),
    }
}
