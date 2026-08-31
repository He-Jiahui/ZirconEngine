use std::ops::Range;

use crate::core::framework::render::RenderGraphExecutionBatchReport;
use crate::render_graph::{
    CompiledRenderGraph, QueueLane, RenderGraphResourceAccessId, RenderPassId,
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RenderGraphExecutionPass {
    pub(crate) graph_pass_index: usize,
    pub(crate) stage: RenderPassStage,
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

/// Monotonic frame-local position in the immutable compiled pass sequence.
///
/// Stage routing may select services for a pass, but it may not reorder, repeat,
/// or omit live compiled passes. Culled passes are skipped by the packet itself.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RenderGraphExecutionCursor {
    next_graph_pass_index: usize,
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

        let passes_by_graph_index = passes_by_graph_index
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

    pub(crate) const fn begin_execution(&self) -> RenderGraphExecutionCursor {
        RenderGraphExecutionCursor {
            next_graph_pass_index: 0,
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
        if actual.culled {
            return Err(format!(
                "render graph execution cursor cannot execute culled compiled graph pass `{}` at index {graph_pass_index}",
                actual.name
            ));
        }

        let Some(expected_index) = self.next_live_pass_index(cursor.next_graph_pass_index) else {
            return Err(format!(
                "render graph execution cursor has no remaining live pass but received `{}` at index {graph_pass_index}",
                actual.name
            ));
        };
        if expected_index != graph_pass_index {
            let expected = &self.graph.passes()[expected_index];
            return Err(format!(
                "render graph execution cursor expected compiled graph pass `{}` at index {expected_index}, but stage routing selected `{}` at index {graph_pass_index}",
                expected.name, actual.name
            ));
        }

        cursor.next_graph_pass_index = graph_pass_index + 1;
        Ok(())
    }

    pub(crate) fn finish_execution(
        &self,
        cursor: RenderGraphExecutionCursor,
    ) -> Result<(), String> {
        let Some(missing_index) = self.next_live_pass_index(cursor.next_graph_pass_index) else {
            return Ok(());
        };
        let missing = &self.graph.passes()[missing_index];
        Err(format!(
            "render graph execution did not execute compiled graph pass `{}` at index {missing_index}",
            missing.name
        ))
    }

    fn next_live_pass_index(&self, start: usize) -> Option<usize> {
        self.graph
            .passes()
            .iter()
            .enumerate()
            .skip(start)
            .find_map(|(index, pass)| (!pass.culled).then_some(index))
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
