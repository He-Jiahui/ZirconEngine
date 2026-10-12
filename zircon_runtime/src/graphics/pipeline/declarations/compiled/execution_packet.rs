use std::collections::HashMap;
use std::ops::Range;

use crate::core::framework::render::RenderGraphExecutionBatchReport;
use crate::render_graph::{
    CompiledRenderGraph, CompiledRenderGraphAccessAllocationBinding,
    CompiledRenderGraphResourceStateTransition, QueueLane, RenderGraphBufferRange,
    RenderGraphResourceAccessId, RenderGraphResourceAccessRange, RenderGraphTextureAspect,
    RenderGraphTextureSubresourceRange, RenderPassId,
};

use super::super::render_pass_stage::RenderPassStage;

/// Product-authored stage metadata before it is resolved against the compiled graph.
///
/// This type exists only on the compiler side of the packet boundary. A frame
/// never consumes it: packet construction resolves each authored ID into one
/// immutable compiled-graph index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RenderGraphExecutionPassMetadata {
    pub(crate) pass_id: RenderPassId,
    pub(crate) stage: RenderPassStage,
}

impl RenderGraphExecutionPassMetadata {
    pub(crate) const fn new(pass_id: RenderPassId, stage: RenderPassStage) -> Self {
        Self { pass_id, stage }
    }
}

/// Immutable stage placement for one compiled graph pass.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RenderGraphExecutionPass {
    pub(crate) graph_pass_index: usize,
    pub(crate) stage: RenderPassStage,
    pub(crate) device_access_bindings: Box<[CompiledRenderGraphAccessAllocationBinding]>,
    pub(crate) device_transitions_before: Box<[CompiledRenderGraphResourceStateTransition]>,
}

/// One contiguous executable segment of the compiled graph.
///
/// Batches are derived from compiled graph order, never from authored stage
/// order. A batch contains only live passes on one queue lane; culling gaps and
/// queue transitions therefore become explicit boundaries for future resource
/// transition and encoder grouping work.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RenderGraphExecutionBatch {
    graph_pass_range: Range<usize>,
    queue: QueueLane,
}

impl RenderGraphExecutionBatch {
    pub(crate) fn graph_pass_range(&self) -> Range<usize> {
        self.graph_pass_range.clone()
    }

    pub(crate) const fn queue(&self) -> QueueLane {
        self.queue
    }
}

/// Frame-local admission state for the immutable compiled pass sequence.
///
/// Stage routing may select services in a different order than the compiler's
/// topological list when passes are independent.  Admission still rejects
/// duplicate passes and a pass whose compiled dependencies have not run; the
/// frame tail check rejects omitted live passes.  The only intentional
/// exception is an explicitly accounted-for primary-surface `Present` pass
/// when an offscreen frame has no surface target.  This keeps dependency
/// safety without conflating topological tie-breaking with the renderer's
/// stage service order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct RenderGraphExecutionCursor {
    executed_graph_passes: Vec<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RenderGraphExecutionPacket {
    graph: CompiledRenderGraph,
    passes_by_graph_index: Vec<RenderGraphExecutionPass>,
    access_ids_by_graph_index: Vec<Box<[RenderGraphResourceAccessId]>>,
    stage_pass_indices: Vec<usize>,
    stage_pass_ranges: [Range<usize>; RenderPassStage::COUNT],
    execution_batches: Vec<RenderGraphExecutionBatch>,
    batch_index_by_graph_pass: Vec<Option<usize>>,
    stage_batch_indices: [Box<[usize]>; RenderPassStage::COUNT],
    stage_order: Box<[RenderPassStage]>,
    execution_batch_report: RenderGraphExecutionBatchReport,
}

impl RenderGraphExecutionPacket {
    pub(crate) fn new(
        graph: CompiledRenderGraph,
        execution_pass_metadata: Vec<RenderGraphExecutionPassMetadata>,
    ) -> Result<Self, String> {
        let mut passes_by_graph_index = vec![None; graph.passes().len()];
        for metadata in execution_pass_metadata {
            let Some((graph_pass_index, _)) = graph.indexed_pass(metadata.pass_id) else {
                return Err(format!(
                    "render graph execution packet references missing compiled pass identity {:?}",
                    metadata.pass_id
                ));
            };
            if passes_by_graph_index[graph_pass_index].is_some() {
                return Err(format!(
                    "render graph execution packet duplicates compiled pass identity {:?}",
                    metadata.pass_id
                ));
            }
            passes_by_graph_index[graph_pass_index] = Some(RenderGraphExecutionPass {
                graph_pass_index,
                stage: metadata.stage,
                device_access_bindings: Box::default(),
                device_transitions_before: Box::default(),
            });
        }

        let missing_pass_indices = passes_by_graph_index
            .iter()
            .enumerate()
            .filter_map(|(index, pass)| pass.is_none().then_some(index))
            .collect::<Vec<_>>();
        if !missing_pass_indices.is_empty() {
            return Err(format!(
                "render graph execution packet is missing stage metadata for {} compiled pass(es): {:?}",
                missing_pass_indices.len(),
                missing_pass_indices
            ));
        }

        let mut passes_by_graph_index = passes_by_graph_index
            .into_iter()
            .enumerate()
            .map(|(graph_pass_index, pass)| {
                pass.ok_or_else(|| {
                    format!(
                        "render graph execution packet is missing stage metadata for compiled pass index {graph_pass_index}"
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let access_ids_by_graph_index = graph
            .passes()
            .iter()
            .map(|pass| {
                pass.resources
                    .iter()
                    .enumerate()
                    .map(|(access_index, _)| {
                        graph.access_id_at(pass.id, access_index).ok_or_else(|| {
                            format!(
                                "render graph execution packet is missing compiled access identity for pass {:?} at access ordinal {access_index}",
                                pass.id
                            )
                        })
                    })
                    .collect::<Result<Box<[_]>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()?;
        let device_execution_plan = CompiledRenderGraphDeviceExecutionPlan::from_graph(&graph)?;
        for (graph_pass_index, execution_pass) in passes_by_graph_index.iter_mut().enumerate() {
            execution_pass.device_access_bindings = device_execution_plan
                .access_bindings_for_pass(graph_pass_index)
                .unwrap_or_default()
                .into();
            execution_pass.device_transitions_before = device_execution_plan
                .transitions_for_pass(graph_pass_index)
                .unwrap_or_default()
                .into();
        }
        let mut stage_counts = [0_usize; RenderPassStage::COUNT];
        for execution_pass in &passes_by_graph_index {
            stage_counts[execution_pass.stage.index()] += 1;
        }
        let mut next_range_start = 0_usize;
        let stage_pass_ranges = std::array::from_fn(|stage_index| {
            let range_end = next_range_start + stage_counts[stage_index];
            let range = next_range_start..range_end;
            next_range_start = range_end;
            range
        });
        let mut stage_next_indices: [usize; RenderPassStage::COUNT] =
            std::array::from_fn(|stage_index| stage_pass_ranges[stage_index].start);
        let mut stage_pass_indices = vec![0_usize; passes_by_graph_index.len()];
        for (graph_pass_index, execution_pass) in passes_by_graph_index.iter().enumerate() {
            let stage_index = execution_pass.stage.index();
            let write_index = stage_next_indices[stage_index];
            stage_pass_indices[write_index] = graph_pass_index;
            stage_next_indices[stage_index] += 1;
        }

        let mut execution_batches = Vec::new();
        let mut current_start = None;
        let mut current_queue = None;
        for (graph_pass_index, pass) in graph.passes().iter().enumerate() {
            let queue_changed = current_queue.is_some_and(|queue| queue != pass.queue);
            if pass.culled || queue_changed {
                if let (Some(start), Some(queue)) = (current_start.take(), current_queue.take()) {
                    execution_batches.push(RenderGraphExecutionBatch {
                        graph_pass_range: start..graph_pass_index,
                        queue,
                    });
                }
            }
            if !pass.culled && current_start.is_none() {
                current_start = Some(graph_pass_index);
                current_queue = Some(pass.queue);
            }
        }
        if let (Some(start), Some(queue)) = (current_start.take(), current_queue.take()) {
            execution_batches.push(RenderGraphExecutionBatch {
                graph_pass_range: start..graph.passes().len(),
                queue,
            });
        }
        validate_execution_batches(&graph, &execution_batches)?;
        let mut batch_index_by_graph_pass = vec![None; graph.passes().len()];
        for (batch_index, batch) in execution_batches.iter().enumerate() {
            for graph_pass_index in batch.graph_pass_range.clone() {
                if batch_index_by_graph_pass[graph_pass_index]
                    .replace(batch_index)
                    .is_some()
                {
                    return Err(format!(
                        "render graph execution batch index table overlaps at compiled graph pass index {graph_pass_index}"
                    ));
                }
            }
        }
        let mut stage_batch_indices: [Vec<usize>; RenderPassStage::COUNT] =
            std::array::from_fn(|_| Vec::new());
        for (batch_index, batch) in execution_batches.iter().enumerate() {
            let mut seen_stages = [false; RenderPassStage::COUNT];
            for execution_pass in passes_by_graph_index[batch.graph_pass_range.clone()].iter() {
                let stage_index = execution_pass.stage.index();
                if !seen_stages[stage_index] {
                    seen_stages[stage_index] = true;
                    stage_batch_indices[stage_index].push(batch_index);
                }
            }
        }
        let stage_batch_indices = stage_batch_indices.map(Vec::into_boxed_slice);
        let mut stage_order = Vec::new();
        let mut seen_stages = [false; RenderPassStage::COUNT];
        for execution_pass in &passes_by_graph_index {
            if graph.passes()[execution_pass.graph_pass_index].culled {
                continue;
            }
            let stage_index = execution_pass.stage.index();
            if !seen_stages[stage_index] {
                seen_stages[stage_index] = true;
                stage_order.push(execution_pass.stage);
            }
        }
        let execution_batch_report = execution_batch_report(&execution_batches);

        Ok(Self {
            graph,
            passes_by_graph_index,
            access_ids_by_graph_index,
            stage_pass_indices,
            stage_pass_ranges,
            execution_batches,
            batch_index_by_graph_pass,
            stage_batch_indices,
            stage_order: stage_order.into_boxed_slice(),
            execution_batch_report,
        })
    }

    pub(crate) fn graph(&self) -> &CompiledRenderGraph {
        &self.graph
    }

    pub(crate) fn execution_pass_at(
        &self,
        graph_pass_index: usize,
    ) -> Option<&RenderGraphExecutionPass> {
        self.passes_by_graph_index.get(graph_pass_index)
    }

    /// Returns the stable compiled access identities owned by one product pass.
    ///
    /// Product execution must carry these identities into its future physical
    /// binding table; resource labels remain diagnostics, not binding keys.
    pub(crate) fn access_ids_for_pass(
        &self,
        graph_pass_index: usize,
    ) -> Option<&[RenderGraphResourceAccessId]> {
        self.access_ids_by_graph_index
            .get(graph_pass_index)
            .map(Box::as_ref)
    }

    pub(crate) fn passes_for_stage(
        &self,
        stage: RenderPassStage,
    ) -> impl Iterator<Item = &RenderGraphExecutionPass> {
        self.stage_pass_indices[self.stage_pass_ranges[stage.index()].clone()]
            .iter()
            .map(|index| &self.passes_by_graph_index[*index])
    }

    pub(crate) fn execution_passes_in_graph_order(
        &self,
    ) -> impl Iterator<Item = &RenderGraphExecutionPass> {
        self.passes_by_graph_index.iter()
    }

    /// Returns live graph-order batches for queue-aware execution lowering.
    pub(crate) fn execution_batches(&self) -> impl Iterator<Item = &RenderGraphExecutionBatch> {
        self.execution_batches.iter()
    }

    /// Returns the immutable queue batch owning a live compiled graph pass.
    ///
    /// Culling is represented by `None`; callers never need to rescan batch
    /// ranges when lowering queue transitions or assigning frame services.
    pub(crate) fn execution_batch_index_for_pass(&self, graph_pass_index: usize) -> Option<usize> {
        self.batch_index_by_graph_pass
            .get(graph_pass_index)
            .copied()
            .flatten()
    }

    /// Returns only the batches that contain passes routed to `stage`.
    ///
    /// The index is compiled once with the packet, so stage-specific products
    /// do not rescan unrelated graph batches on every frame.
    pub(crate) fn execution_batches_for_stage(
        &self,
        stage: RenderPassStage,
    ) -> impl Iterator<Item = &RenderGraphExecutionBatch> {
        self.stage_batch_indices[stage.index()]
            .iter()
            .map(|batch_index| &self.execution_batches[*batch_index])
    }

    pub(crate) fn execution_batches_with_indices_for_stage(
        &self,
        stage: RenderPassStage,
    ) -> impl Iterator<Item = (usize, &RenderGraphExecutionBatch)> {
        self.stage_batch_indices[stage.index()]
            .iter()
            .copied()
            .map(|batch_index| (batch_index, &self.execution_batches[batch_index]))
    }

    /// Returns the first-seen stage order from the compiled graph.
    ///
    /// The order is cached with the packet so frame execution does not rescan
    /// batches merely to discover which late services are needed.
    pub(crate) fn execution_stages_in_graph_order(
        &self,
    ) -> impl Iterator<Item = RenderPassStage> + '_ {
        self.stage_order.iter().copied()
    }

    pub(crate) fn passes_for_batch(
        &self,
        batch: &RenderGraphExecutionBatch,
    ) -> impl Iterator<Item = &RenderGraphExecutionPass> {
        self.passes_by_graph_index
            .get(batch.graph_pass_range.clone())
            .into_iter()
            .flatten()
    }

    pub(crate) const fn execution_batch_report(&self) -> RenderGraphExecutionBatchReport {
        self.execution_batch_report
    }

    pub(crate) fn begin_execution(&self) -> RenderGraphExecutionCursor {
        RenderGraphExecutionCursor {
            executed_graph_passes: vec![false; self.graph.passes().len()],
        }
    }

    pub(crate) fn admit_execution_pass(
        &self,
        cursor: &mut RenderGraphExecutionCursor,
        graph_pass_index: usize,
    ) -> Result<(), String> {
        let actual = self.graph.passes().get(graph_pass_index).ok_or_else(|| {
            format!(
                "render graph execution cursor references missing compiled graph pass index {graph_pass_index}"
            )
        })?;
        if std::env::var_os("ZR_TRACE_RENDER_GRAPH").is_some() {
            let stage = self
                .execution_pass_at(graph_pass_index)
                .map(|execution_pass| format!("{:?}", execution_pass.stage))
                .unwrap_or_else(|| "<missing-stage>".to_owned());
            let dependencies = actual
                .dependencies
                .iter()
                .filter_map(|dependency| {
                    self.graph.indexed_pass(*dependency).map(|(index, pass)| {
                        format!("{}:{}:{}", index, pass.name, dependency.index())
                    })
                })
                .collect::<Vec<_>>();
            let executed = cursor
                .executed_graph_passes
                .iter()
                .enumerate()
                .filter_map(|(index, is_executed)| {
                    if *is_executed {
                        self.graph
                            .passes()
                            .get(index)
                            .map(|pass| format!("{}:{}", index, pass.name))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            eprintln!(
                "ZR_TRACE admit graph_index={} name={} stage={} deps={:?} executed={:?}",
                graph_pass_index, actual.name, stage, dependencies, executed
            );
        }
        if actual.culled {
            return Err(format!(
                "render graph execution cursor cannot execute culled compiled graph pass `{}` at index {graph_pass_index}",
                actual.name
            ));
        }

        let Some(executed) = cursor.executed_graph_passes.get(graph_pass_index).copied() else {
            return Err(format!(
                "render graph execution cursor references graph pass index {graph_pass_index} outside the packet"
            ));
        };
        if executed {
            return Err(format!(
                "render graph execution cursor admitted compiled graph pass `{}` at index {graph_pass_index} more than once",
                actual.name
            ));
        }

        for dependency in &actual.dependencies {
            let Some((dependency_index, dependency_pass)) = self.graph.indexed_pass(*dependency)
            else {
                return Err(format!(
                    "render graph execution pass `{}` at index {graph_pass_index} references missing dependency {:?}",
                    actual.name, dependency
                ));
            };
            if dependency_pass.culled {
                continue;
            }
            if !cursor
                .executed_graph_passes
                .get(dependency_index)
                .copied()
                .unwrap_or(false)
            {
                return Err(format!(
                    "render graph execution pass `{}` at index {graph_pass_index} depends on `{}` at index {dependency_index} which has not executed",
                    actual.name, dependency_pass.name
                ));
            }
        }

        // The immutable dependency checks above must complete before taking a
        // mutable borrow of the admission bitset.  Keeping the borrow narrow
        // also makes it impossible to accidentally inspect another dependency
        // through an aliased mutable slice in future changes.
        if let Some(executed) = cursor.executed_graph_passes.get_mut(graph_pass_index) {
            *executed = true;
        }
        Ok(())
    }

    /// Marks an unavailable primary-surface presentation pass as intentionally
    /// elided for this frame.
    ///
    /// A compiled pipeline is shared by surface-backed and offscreen frames.
    /// The `Present` pass is authored for a primary surface, but an offscreen
    /// submission has no acquired surface target to encode into.  Such a pass
    /// still participates in dependency and completion accounting so skipping
    /// it cannot hide an omitted ordinary pass or leave the execution cursor
    /// inconsistent.
    pub(crate) fn skip_surface_present_execution_pass(
        &self,
        cursor: &mut RenderGraphExecutionCursor,
        graph_pass_index: usize,
    ) -> Result<(), String> {
        let actual = self.graph.passes().get(graph_pass_index).ok_or_else(|| {
            format!(
                "render graph execution cursor references missing compiled graph pass index {graph_pass_index}"
            )
        })?;
        let Some(execution_pass) = self.execution_pass_at(graph_pass_index) else {
            return Err(format!(
                "render graph execution packet references missing stage metadata for graph pass index {graph_pass_index}"
            ));
        };
        if execution_pass.stage != RenderPassStage::Present {
            return Err(format!(
                "render graph execution cannot skip non-terminal graph pass `{}` at index {graph_pass_index}",
                actual.name
            ));
        }
        if actual.name != super::super::terminal_surface_pass::SURFACE_PRESENT_PASS_NAME {
            return Err(format!(
                "render graph execution cannot skip non-surface terminal graph pass `{}` at index {graph_pass_index}",
                actual.name
            ));
        }
        if actual.culled {
            return Err(format!(
                "render graph execution cannot skip culled compiled graph pass `{}` at index {graph_pass_index}",
                actual.name
            ));
        }

        let Some(executed) = cursor.executed_graph_passes.get(graph_pass_index).copied() else {
            return Err(format!(
                "render graph execution cursor references graph pass index {graph_pass_index} outside the packet"
            ));
        };
        if executed {
            return Err(format!(
                "render graph execution cursor admitted or skipped compiled graph pass `{}` at index {graph_pass_index} more than once",
                actual.name
            ));
        }

        for dependency in &actual.dependencies {
            let Some((dependency_index, dependency_pass)) = self.graph.indexed_pass(*dependency)
            else {
                return Err(format!(
                    "render graph execution pass `{}` at index {graph_pass_index} references missing dependency {:?}",
                    actual.name, dependency
                ));
            };
            if dependency_pass.culled {
                continue;
            }
            if !cursor
                .executed_graph_passes
                .get(dependency_index)
                .copied()
                .unwrap_or(false)
            {
                return Err(format!(
                    "render graph execution pass `{}` at index {graph_pass_index} depends on `{}` at index {dependency_index} which has not executed",
                    actual.name, dependency_pass.name
                ));
            }
        }

        // A skipped terminal pass must not feed another live pass.  This is
        // normally guaranteed by the compiler's terminal-stage contract, but
        // checking it here keeps the escape hatch safe for custom pipelines.
        for (dependent_index, dependent_pass) in self.graph.passes().iter().enumerate() {
            if dependent_pass.culled || dependent_index == graph_pass_index {
                continue;
            }
            if dependent_pass.dependencies.iter().any(|dependency| {
                self.graph
                    .indexed_pass(*dependency)
                    .is_some_and(|(index, _)| index == graph_pass_index)
            }) {
                return Err(format!(
                    "render graph execution cannot skip graph pass `{}` at index {graph_pass_index}; live pass `{}` at index {dependent_index} depends on it",
                    actual.name, dependent_pass.name
                ));
            }
        }

        if let Some(executed) = cursor.executed_graph_passes.get_mut(graph_pass_index) {
            *executed = true;
        }
        Ok(())
    }

    pub(crate) fn finish_execution(
        &self,
        cursor: &RenderGraphExecutionCursor,
    ) -> Result<(), String> {
        for (missing_index, missing) in self.graph.passes().iter().enumerate() {
            if missing.culled {
                continue;
            }
            if !cursor
                .executed_graph_passes
                .get(missing_index)
                .copied()
                .unwrap_or(false)
            {
                return Err(format!(
                    "render graph execution did not execute compiled graph pass `{}` at index {missing_index}",
                    missing.name
                ));
            }
        }
        Ok(())
    }

    pub(crate) fn stage_for_pass_name(&self, pass_name: &str) -> Option<RenderPassStage> {
        self.graph
            .passes()
            .iter()
            .position(|pass| pass.name == pass_name)
            .and_then(|index| self.execution_pass_at(index))
            .map(|execution_pass| execution_pass.stage)
    }
}

fn execution_batch_report(
    batches: &[RenderGraphExecutionBatch],
) -> RenderGraphExecutionBatchReport {
    let mut planned_live_pass_count = 0_usize;
    let mut graphics_batch_count = 0_usize;
    let mut async_compute_batch_count = 0_usize;
    let mut async_copy_batch_count = 0_usize;
    let mut max_passes_per_batch = 0_usize;
    let mut queue_transition_count = 0_usize;
    let mut previous_queue = None;
    for batch in batches {
        let pass_count = batch.graph_pass_range.len();
        planned_live_pass_count = planned_live_pass_count.saturating_add(pass_count);
        max_passes_per_batch = max_passes_per_batch.max(pass_count);
        match batch.queue {
            QueueLane::Graphics => graphics_batch_count += 1,
            QueueLane::AsyncCompute => async_compute_batch_count += 1,
            QueueLane::AsyncCopy => async_copy_batch_count += 1,
        }
        if previous_queue.is_some_and(|queue| queue != batch.queue) {
            queue_transition_count += 1;
        }
        previous_queue = Some(batch.queue);
    }
    RenderGraphExecutionBatchReport::new(
        batches.len(),
        planned_live_pass_count,
        graphics_batch_count,
        async_compute_batch_count,
        async_copy_batch_count,
        max_passes_per_batch,
        queue_transition_count,
    )
}

fn validate_execution_batches(
    graph: &CompiledRenderGraph,
    batches: &[RenderGraphExecutionBatch],
) -> Result<(), String> {
    let mut covered = vec![false; graph.passes().len()];
    for batch in batches {
        let range = &batch.graph_pass_range;
        if range.start >= range.end || range.end > graph.passes().len() {
            return Err(format!(
                "render graph execution batch has invalid graph pass range {:?}",
                range
            ));
        }
        for graph_pass_index in range.clone() {
            let pass = &graph.passes()[graph_pass_index];
            if pass.culled {
                return Err(format!(
                    "render graph execution batch includes culled compiled graph pass `{}` at index {graph_pass_index}",
                    pass.name
                ));
            }
            if pass.queue != batch.queue {
                return Err(format!(
                    "render graph execution batch queue {:?} disagrees with compiled graph pass `{}` queue {:?} at index {graph_pass_index}",
                    batch.queue, pass.name, pass.queue
                ));
            }
            if std::mem::replace(&mut covered[graph_pass_index], true) {
                return Err(format!(
                    "render graph execution batches overlap at compiled graph pass `{}` index {graph_pass_index}",
                    pass.name
                ));
            }
        }
    }
    for (graph_pass_index, pass) in graph.passes().iter().enumerate() {
        if !pass.culled && !covered[graph_pass_index] {
            return Err(format!(
                "render graph execution batches omit live compiled graph pass `{}` at index {graph_pass_index}",
                pass.name
            ));
        }
    }
    Ok(())
}

/// Exact graph-use and transition rows carried to product execution.
///
/// This is compiler output, not a backend barrier command list. On WGPU the
/// native command encoders still declare concrete usages and `wgpu_core`
/// tracks those usages and inserts the required transitions. Product execution
/// consumes these rows to validate pass order, access identity, range, state,
/// and backing lease before it records native work.
#[derive(Clone, Debug, PartialEq, Eq)]
struct CompiledRenderGraphDeviceExecutionPlan {
    access_bindings_by_graph_pass: Vec<Option<Box<[CompiledRenderGraphAccessAllocationBinding]>>>,
    transitions_by_graph_pass: Vec<Option<Box<[CompiledRenderGraphResourceStateTransition]>>>,
}

impl CompiledRenderGraphDeviceExecutionPlan {
    fn from_graph(graph: &CompiledRenderGraph) -> Result<Self, String> {
        let pass_count = graph.passes().len();
        let mut access_bindings_by_graph_pass = (0..pass_count)
            .map(|graph_pass_index| (!graph.passes()[graph_pass_index].culled).then(Vec::new))
            .collect::<Vec<_>>();
        let mut transitions_by_graph_pass = (0..pass_count)
            .map(|graph_pass_index| (!graph.passes()[graph_pass_index].culled).then(Vec::new))
            .collect::<Vec<_>>();
        let mut access_owner = HashMap::new();

        for (graph_pass_index, pass) in graph.passes().iter().enumerate() {
            if pass.culled {
                continue;
            }
            for (access_ordinal, pass_access) in pass.resources.iter().enumerate() {
                let access_id = graph
                    .access_id_at(pass.id, access_ordinal)
                    .ok_or_else(|| {
                        format!(
                            "device execution plan is missing access identity for live pass `{}` access ordinal {access_ordinal}",
                            pass.name
                        )
                    })?;
                let binding = graph.access_allocation_binding(access_id).ok_or_else(|| {
                    format!(
                        "device execution plan is missing compiled access binding for live pass `{}` access {:?}",
                        pass.name, access_id
                    )
                })?;
                let metadata = graph.access_metadata(access_id).ok_or_else(|| {
                    format!(
                        "device execution plan is missing access metadata for live pass `{}` access {:?}",
                        pass.name, access_id
                    )
                })?;
                let declaration = graph
                    .resource_declaration(binding.key.resource)
                    .ok_or_else(|| {
                        format!(
                            "device execution plan access {:?} references an undeclared resource",
                            access_id
                        )
                    })?;
                if binding.key.access_id != access_id
                    || binding.key.range != metadata.range
                    || binding.key.intent != metadata.intent
                    || binding.key.access != pass_access.access
                    || declaration.name != pass_access.name
                    || declaration.kind != pass_access.kind
                {
                    return Err(format!(
                        "device execution plan access {:?} disagrees with its compiled pass metadata",
                        access_id
                    ));
                }
                if access_owner
                    .insert(access_id, (graph_pass_index, binding))
                    .is_some()
                {
                    return Err(format!(
                        "device execution plan contains duplicate access identity {:?}",
                        access_id
                    ));
                }
                access_bindings_by_graph_pass[graph_pass_index]
                    .as_mut()
                    .expect("live passes own a plan row")
                    .push(*binding);
            }
        }

        for &transition in graph.resource_state_plan().transitions() {
            let (from_pass_index, from_binding) =
                access_owner.get(&transition.from_access).ok_or_else(|| {
                    format!(
                        "device execution transition references missing live source access {:?}",
                        transition.from_access
                    )
                })?;
            let (to_pass_index, to_binding) =
                access_owner.get(&transition.to_access).ok_or_else(|| {
                    format!(
                        "device execution transition references missing live destination access {:?}",
                        transition.to_access
                    )
                })?;
            let from_pass = &graph.passes()[*from_pass_index];
            let to_pass = &graph.passes()[*to_pass_index];
            if from_pass_index >= to_pass_index
                || transition.resource != from_binding.key.resource
                || transition.resource != to_binding.key.resource
                || transition.from_state
                    != crate::render_graph::RenderGraphResourceState::from(from_binding.key.intent)
                || transition.to_state
                    != crate::render_graph::RenderGraphResourceState::from(to_binding.key.intent)
                || transition.from_queue != from_pass.queue
                || transition.to_queue != to_pass.queue
                || !access_ranges_overlap(transition.range, from_binding.key.range)
                || !access_ranges_overlap(transition.range, to_binding.key.range)
            {
                return Err(format!(
                    "device execution transition {:?}->{:?} disagrees with its ordered access rows",
                    transition.from_access, transition.to_access
                ));
            }
            transitions_by_graph_pass[*to_pass_index]
                .as_mut()
                .expect("live transition destinations own a plan row")
                .push(transition);
        }

        Ok(Self {
            access_bindings_by_graph_pass: access_bindings_by_graph_pass
                .into_iter()
                .map(|bindings| bindings.map(Vec::into_boxed_slice))
                .collect(),
            transitions_by_graph_pass: transitions_by_graph_pass
                .into_iter()
                .map(|transitions| transitions.map(Vec::into_boxed_slice))
                .collect(),
        })
    }

    fn access_bindings_for_pass(
        &self,
        graph_pass_index: usize,
    ) -> Option<&[CompiledRenderGraphAccessAllocationBinding]> {
        self.access_bindings_by_graph_pass
            .get(graph_pass_index)
            .and_then(Option::as_deref)
    }

    fn transitions_for_pass(
        &self,
        graph_pass_index: usize,
    ) -> Option<&[CompiledRenderGraphResourceStateTransition]> {
        self.transitions_by_graph_pass
            .get(graph_pass_index)
            .and_then(Option::as_deref)
    }
}

fn access_ranges_overlap(
    transition: RenderGraphResourceAccessRange,
    access: RenderGraphResourceAccessRange,
) -> bool {
    match (transition, access) {
        (
            RenderGraphResourceAccessRange::Texture(left),
            RenderGraphResourceAccessRange::Texture(right),
        ) => texture_ranges_overlap(left, right),
        (
            RenderGraphResourceAccessRange::Buffer(left),
            RenderGraphResourceAccessRange::Buffer(right),
        ) => buffer_ranges_overlap(left, right),
        _ => false,
    }
}

fn texture_ranges_overlap(
    left: RenderGraphTextureSubresourceRange,
    right: RenderGraphTextureSubresourceRange,
) -> bool {
    let aspect_overlaps = left.aspect == right.aspect
        || left.aspect == RenderGraphTextureAspect::All
        || right.aspect == RenderGraphTextureAspect::All;
    aspect_overlaps
        && intervals_overlap(
            left.base_mip_level,
            left.mip_level_count,
            right.base_mip_level,
            right.mip_level_count,
        )
        && intervals_overlap(
            left.base_array_layer,
            left.array_layer_count,
            right.base_array_layer,
            right.array_layer_count,
        )
}

fn intervals_overlap(
    left_start: u32,
    left_count: Option<u32>,
    right_start: u32,
    right_count: Option<u32>,
) -> bool {
    let left_end = left_count
        .map(|count| u64::from(left_start) + u64::from(count))
        .unwrap_or(u64::MAX);
    let right_end = right_count
        .map(|count| u64::from(right_start) + u64::from(count))
        .unwrap_or(u64::MAX);
    u64::from(left_start) < right_end && u64::from(right_start) < left_end
}

fn buffer_ranges_overlap(left: RenderGraphBufferRange, right: RenderGraphBufferRange) -> bool {
    let left_end = left
        .size
        .and_then(|size| left.offset.checked_add(size))
        .unwrap_or(u64::MAX);
    let right_end = right
        .size
        .and_then(|size| right.offset.checked_add(size))
        .unwrap_or(u64::MAX);
    left.offset < right_end && right.offset < left_end
}
