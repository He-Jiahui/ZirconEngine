use std::sync::Arc;

use zircon_runtime_interface::ui::{
    layout::{UiFrame, UiPoint},
    surface::{UiHitTestQuery, UiSurfaceFrame, UiVirtualPointerPosition},
};

use crate::{core::math::UVec2, ui::tree::bounded_hit_grid_dimensions};

const INPUT_PUBLICATION_CELL_SIZE: f32 = 64.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct RuntimeUiInputPublicationReport {
    pub full_rebuild: bool,
    pub patched_surface_count: usize,
    pub visited_entry_count: usize,
    pub cell_membership_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct RuntimeUiInputQuery {
    generation: u64,
    cell_index: Option<u32>,
    candidate_count: usize,
    physical_point: UiPoint,
    virtual_pointer: Option<UiVirtualPointerPosition>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RuntimeUiInputQueryRejectReason {
    NonFinitePointer,
    DegenerateViewport,
    AffineProjectionOverflow,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum RuntimeUiInputQueryAdmission {
    Published(RuntimeUiInputQuery),
    Unpublished,
    Rejected(RuntimeUiInputQueryRejectReason),
}

impl RuntimeUiInputQueryAdmission {
    #[cfg(test)]
    const fn published(self) -> Option<RuntimeUiInputQuery> {
        match self {
            Self::Published(query) => Some(query),
            Self::Unpublished | Self::Rejected(_) => None,
        }
    }
}

impl RuntimeUiInputQuery {
    pub(super) const fn candidate_count(self) -> usize {
        self.candidate_count
    }

    pub(super) fn hit_test_query(self) -> UiHitTestQuery {
        let query = UiHitTestQuery::new(self.physical_point);
        match self.virtual_pointer {
            Some(virtual_pointer) => query.with_virtual_pointer(virtual_pointer),
            None => query,
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct RuntimeUiInputPublication {
    generation: u64,
    viewport_size: Option<UVec2>,
    bounds: UiFrame,
    cell_size: f32,
    columns: u32,
    rows: u32,
    cells: Vec<Vec<u32>>,
    cell_visit_stamps: Vec<u32>,
    next_cell_visit_stamp: u32,
    surface_hit_generations: Vec<u64>,
    surface_footprints: Vec<Vec<u32>>,
}

impl RuntimeUiInputPublication {
    pub(super) fn publish(
        &mut self,
        viewport_size: UVec2,
        surface_count: usize,
        frames: impl IntoIterator<Item = Arc<UiSurfaceFrame>>,
    ) -> RuntimeUiInputPublicationReport {
        let full_rebuild = self.viewport_size != Some(viewport_size)
            || self.surface_hit_generations.len() != surface_count;
        if full_rebuild {
            self.reset(viewport_size, surface_count);
        }

        let mut report = RuntimeUiInputPublicationReport {
            full_rebuild,
            ..RuntimeUiInputPublicationReport::default()
        };
        let mut observed_surface_count = 0_usize;
        for (surface_index, frame) in frames.into_iter().enumerate() {
            if surface_index >= surface_count {
                break;
            }
            observed_surface_count = observed_surface_count.saturating_add(1);
            let hit_generation = frame.domain_generations.hit_test;
            if !full_rebuild && self.surface_hit_generations[surface_index] == hit_generation {
                continue;
            }

            if !full_rebuild {
                self.remove_surface(surface_index);
            }
            let mut footprint = std::mem::take(&mut self.surface_footprints[surface_index]);
            let visited_entry_count = self.populate_surface_footprint(&frame, &mut footprint);
            report.visited_entry_count = report
                .visited_entry_count
                .saturating_add(visited_entry_count);
            report.cell_membership_count =
                report.cell_membership_count.saturating_add(footprint.len());
            report.patched_surface_count = report.patched_surface_count.saturating_add(1);
            self.insert_surface(surface_index, &footprint);
            self.surface_footprints[surface_index] = footprint;
            self.surface_hit_generations[surface_index] = hit_generation;
        }
        debug_assert_eq!(observed_surface_count, surface_count);
        self.generation = self.generation.saturating_add(1);
        report
    }

    pub(super) fn query(
        &self,
        viewport_size: UVec2,
        physical_point: UiPoint,
        previous_physical_point: UiPoint,
    ) -> RuntimeUiInputQueryAdmission {
        if !point_is_finite(physical_point) || !point_is_finite(previous_physical_point) {
            return RuntimeUiInputQueryAdmission::Rejected(
                RuntimeUiInputQueryRejectReason::NonFinitePointer,
            );
        }
        if viewport_size.x == 0 || viewport_size.y == 0 {
            return RuntimeUiInputQueryAdmission::Rejected(
                RuntimeUiInputQueryRejectReason::DegenerateViewport,
            );
        }
        let Some(published_viewport_size) = self.viewport_size else {
            return RuntimeUiInputQueryAdmission::Unpublished;
        };
        if published_viewport_size.x == 0 || published_viewport_size.y == 0 {
            return RuntimeUiInputQueryAdmission::Rejected(
                RuntimeUiInputQueryRejectReason::DegenerateViewport,
            );
        }
        let virtual_pointer = if published_viewport_size != viewport_size {
            let projected = [
                map_pointer_axis(physical_point.x, published_viewport_size.x, viewport_size.x),
                map_pointer_axis(physical_point.y, published_viewport_size.y, viewport_size.y),
                map_pointer_axis(
                    previous_physical_point.x,
                    published_viewport_size.x,
                    viewport_size.x,
                ),
                map_pointer_axis(
                    previous_physical_point.y,
                    published_viewport_size.y,
                    viewport_size.y,
                ),
            ];
            let [Some(current_x), Some(current_y), Some(previous_x), Some(previous_y)] = projected
            else {
                return RuntimeUiInputQueryAdmission::Rejected(
                    RuntimeUiInputQueryRejectReason::AffineProjectionOverflow,
                );
            };
            Some(UiVirtualPointerPosition::new(
                UiPoint::new(current_x, current_y),
                UiPoint::new(previous_x, previous_y),
            ))
        } else {
            None
        };
        let hit_point = virtual_pointer
            .map(|virtual_pointer| virtual_pointer.current)
            .unwrap_or(physical_point);
        let cell_index = self.cell_index(hit_point);
        let candidate_count = cell_index
            .and_then(|cell_index| self.cells.get(cell_index as usize))
            .map_or(0, Vec::len);
        RuntimeUiInputQueryAdmission::Published(RuntimeUiInputQuery {
            generation: self.generation,
            cell_index,
            candidate_count,
            physical_point,
            virtual_pointer,
        })
    }

    pub(super) fn candidate_surface(
        &self,
        query: RuntimeUiInputQuery,
        candidate_offset: usize,
    ) -> Option<usize> {
        if query.generation != self.generation {
            return None;
        }
        let candidates = self.cells.get(query.cell_index? as usize)?;
        let candidate_index = candidates
            .len()
            .checked_sub(candidate_offset.saturating_add(1))?;
        candidates
            .get(candidate_index)
            .copied()
            .and_then(|surface_index| usize::try_from(surface_index).ok())
    }

    fn reset(&mut self, viewport_size: UVec2, surface_count: usize) {
        let bounds = UiFrame::new(
            0.0,
            0.0,
            viewport_size.x.max(1) as f32,
            viewport_size.y.max(1) as f32,
        );
        let (columns, rows, cell_size) =
            bounded_hit_grid_dimensions(bounds, &[], INPUT_PUBLICATION_CELL_SIZE);
        let cell_count = (columns as usize)
            .checked_mul(rows as usize)
            .expect("runtime UI input publication dimensions are bounded");
        self.viewport_size = Some(viewport_size);
        self.bounds = bounds;
        self.cell_size = cell_size;
        self.columns = columns;
        self.rows = rows;
        self.cells = vec![Vec::new(); cell_count];
        self.cell_visit_stamps.clear();
        self.cell_visit_stamps.resize(cell_count, 0);
        self.next_cell_visit_stamp = 0;
        self.surface_hit_generations = vec![0; surface_count];
        self.surface_footprints = vec![Vec::new(); surface_count];
    }

    fn populate_surface_footprint(
        &mut self,
        frame: &UiSurfaceFrame,
        footprint: &mut Vec<u32>,
    ) -> usize {
        footprint.clear();
        let visit_stamp = self.begin_cell_visit();
        let mut visited_entry_count = 0_usize;
        let bounds = self.bounds;
        let columns = self.columns;
        let rows = self.rows;
        let cell_size = self.cell_size;
        let cell_visit_stamps = &mut self.cell_visit_stamps;
        for entry in frame.hit_grid.entries.iter() {
            visited_entry_count = visited_entry_count.saturating_add(1);
            let Some(clipped_frame) = entry.frame.intersection(entry.clip_frame) else {
                continue;
            };
            visit_bounded_cells(
                bounds,
                columns,
                rows,
                cell_size,
                clipped_frame,
                |cell_index| {
                    let Some(cell_stamp) = cell_visit_stamps.get_mut(cell_index) else {
                        return;
                    };
                    if *cell_stamp != visit_stamp {
                        *cell_stamp = visit_stamp;
                        footprint.push(
                            u32::try_from(cell_index)
                                .expect("runtime UI input publication cell count is bounded"),
                        );
                    }
                },
            );
        }
        visited_entry_count
    }

    fn begin_cell_visit(&mut self) -> u32 {
        self.next_cell_visit_stamp = self.next_cell_visit_stamp.wrapping_add(1);
        if self.next_cell_visit_stamp == 0 {
            self.cell_visit_stamps.fill(0);
            self.next_cell_visit_stamp = 1;
        }
        self.next_cell_visit_stamp
    }

    #[cfg(test)]
    fn patch_scratch_capacities_for_test(&self, surface_index: usize) -> (usize, usize) {
        (
            self.cell_visit_stamps.capacity(),
            self.surface_footprints
                .get(surface_index)
                .map_or(0, Vec::capacity),
        )
    }

    fn remove_surface(&mut self, surface_index: usize) {
        let surface_id = u32::try_from(surface_index)
            .expect("runtime UI input publication Surface count exceeds u32");
        let Some(footprint) = self.surface_footprints.get(surface_index) else {
            return;
        };
        for cell_index in footprint {
            let Some(candidates) = self.cells.get_mut(*cell_index as usize) else {
                continue;
            };
            if let Ok(candidate_index) = candidates.binary_search(&surface_id) {
                candidates.remove(candidate_index);
            }
        }
    }

    fn insert_surface(&mut self, surface_index: usize, footprint: &[u32]) {
        let surface_id = u32::try_from(surface_index)
            .expect("runtime UI input publication Surface count exceeds u32");
        for cell_index in footprint {
            let Some(candidates) = self.cells.get_mut(*cell_index as usize) else {
                continue;
            };
            if let Err(candidate_index) = candidates.binary_search(&surface_id) {
                candidates.insert(candidate_index, surface_id);
            }
        }
    }

    fn cell_index(&self, point: UiPoint) -> Option<u32> {
        if self.columns == 0
            || self.rows == 0
            || !self.cell_size.is_finite()
            || self.cell_size <= 0.0
            || !self.bounds.contains_point(point)
        {
            return None;
        }
        let column = ((point.x - self.bounds.x) / self.cell_size).floor() as u32;
        let row = ((point.y - self.bounds.y) / self.cell_size).floor() as u32;
        let column = column.min(self.columns - 1);
        let row = row.min(self.rows - 1);
        Some(row * self.columns + column)
    }
}

fn visit_bounded_cells(
    bounds: UiFrame,
    columns: u32,
    rows: u32,
    cell_size: f32,
    frame: UiFrame,
    mut visit: impl FnMut(usize),
) {
    if columns == 0
        || rows == 0
        || !cell_size.is_finite()
        || cell_size <= 0.0
        || !point_is_finite(UiPoint::new(frame.x, frame.y))
        || !frame.width.is_finite()
        || !frame.height.is_finite()
        || frame.width <= 0.0
        || frame.height <= 0.0
        || frame.intersection(bounds).is_none()
    {
        return;
    }
    let left = ((frame.x - bounds.x) / cell_size).floor().max(0.0) as u32;
    let top = ((frame.y - bounds.y) / cell_size).floor().max(0.0) as u32;
    let right = ((frame.right() - bounds.x) / cell_size)
        .floor()
        .max(0.0)
        .min((columns - 1) as f32) as u32;
    let bottom = ((frame.bottom() - bounds.y) / cell_size)
        .floor()
        .max(0.0)
        .min((rows - 1) as f32) as u32;
    if left > right || top > bottom {
        return;
    }
    for row in top..=bottom {
        for column in left..=right {
            visit((row * columns + column) as usize);
        }
    }
}

fn point_is_finite(point: UiPoint) -> bool {
    point.x.is_finite() && point.y.is_finite()
}

fn map_pointer_axis(point: f32, published_extent: u32, current_extent: u32) -> Option<f32> {
    if !point.is_finite() || published_extent == 0 || current_extent == 0 {
        return None;
    }
    let mapped = point * published_extent as f32 / current_extent as f32;
    mapped.is_finite().then_some(mapped)
}

#[cfg(test)]
#[path = "tests/input_publication.rs"]
mod tests;
