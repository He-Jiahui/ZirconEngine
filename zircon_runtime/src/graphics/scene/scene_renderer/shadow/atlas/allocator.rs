use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use crate::core::framework::render::ShadowResolutionTier;

pub(crate) const SHADOW_ATLAS_DEFAULT_SIZE: u32 = 4096;
pub(crate) const SHADOW_ATLAS_DEFAULT_CSM_ROW_HEIGHT: u32 = 1024;
pub(crate) const SHADOW_ATLAS_SLOT_RETENTION_FRAMES: u32 = 8;
pub(crate) const SHADOW_ATLAS_PREEMPTION_FRAMES: u32 = 4;
pub(crate) const SHADOW_ATLAS_PREEMPTION_SCORE_MULTIPLIER: f32 = 1.25;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct ShadowSlotKey {
    pub(crate) light_id: u64,
    pub(crate) face_index: u8,
}

impl ShadowSlotKey {
    pub(crate) const fn new(light_id: u64, face_index: u8) -> Self {
        Self {
            light_id,
            face_index,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ShadowAtlasRect {
    pub(crate) x: u32,
    pub(crate) y: u32,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl ShadowAtlasRect {
    pub(crate) const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub(crate) const fn right(self) -> u32 {
        self.x + self.width
    }

    pub(crate) const fn bottom(self) -> u32 {
        self.y + self.height
    }

    pub(crate) const fn area(self) -> u64 {
        self.width as u64 * self.height as u64
    }

    pub(crate) fn intersects(self, other: Self) -> bool {
        self.x < other.right()
            && self.right() > other.x
            && self.y < other.bottom()
            && self.bottom() > other.y
    }

    pub(crate) fn contains_rect(self, other: Self) -> bool {
        self.x <= other.x
            && self.y <= other.y
            && self.right() >= other.right()
            && self.bottom() >= other.bottom()
    }

    fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }

    fn subtract(self, used: Self) -> Vec<Self> {
        if !self.intersects(used) {
            return vec![self];
        }

        let overlap_x0 = self.x.max(used.x);
        let overlap_y0 = self.y.max(used.y);
        let overlap_x1 = self.right().min(used.right());
        let overlap_y1 = self.bottom().min(used.bottom());
        let mut free = Vec::with_capacity(4);

        free.push(Self::new(
            self.x,
            self.y,
            self.width,
            overlap_y0.saturating_sub(self.y),
        ));
        free.push(Self::new(
            self.x,
            overlap_y1,
            self.width,
            self.bottom().saturating_sub(overlap_y1),
        ));
        free.push(Self::new(
            self.x,
            overlap_y0,
            overlap_x0.saturating_sub(self.x),
            overlap_y1.saturating_sub(overlap_y0),
        ));
        free.push(Self::new(
            overlap_x1,
            overlap_y0,
            self.right().saturating_sub(overlap_x1),
            overlap_y1.saturating_sub(overlap_y0),
        ));

        free.into_iter().filter(|rect| !rect.is_empty()).collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ShadowAtlasConfig {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) reserved_top_px: u32,
    pub(crate) slot_retention_frames: u32,
    pub(crate) preemption_confirmation_frames: u32,
    pub(crate) preemption_score_multiplier: f32,
}

impl ShadowAtlasConfig {
    pub(crate) const fn new_square(size: u32) -> Self {
        Self {
            width: size,
            height: size,
            reserved_top_px: 0,
            slot_retention_frames: SHADOW_ATLAS_SLOT_RETENTION_FRAMES,
            preemption_confirmation_frames: SHADOW_ATLAS_PREEMPTION_FRAMES,
            preemption_score_multiplier: SHADOW_ATLAS_PREEMPTION_SCORE_MULTIPLIER,
        }
    }

    pub(crate) const fn with_reserved_top_px(mut self, reserved_top_px: u32) -> Self {
        self.reserved_top_px = reserved_top_px;
        self
    }

    pub(crate) fn available_rect(self) -> Option<ShadowAtlasRect> {
        let available_height = self.height.checked_sub(self.reserved_top_px)?;
        if self.width == 0 || available_height == 0 {
            return None;
        }
        Some(ShadowAtlasRect::new(
            0,
            self.reserved_top_px,
            self.width,
            available_height,
        ))
    }

    fn max_slot_size(self) -> u32 {
        self.available_rect()
            .map(|rect| rect.width.min(rect.height))
            .unwrap_or(0)
    }
}

impl Default for ShadowAtlasConfig {
    fn default() -> Self {
        Self::new_square(SHADOW_ATLAS_DEFAULT_SIZE)
            .with_reserved_top_px(SHADOW_ATLAS_DEFAULT_CSM_ROW_HEIGHT)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ShadowSlotRequest {
    pub(crate) key: ShadowSlotKey,
    pub(crate) requested_tier: ShadowResolutionTier,
    pub(crate) minimum_tier: ShadowResolutionTier,
    pub(crate) priority: f32,
}

impl ShadowSlotRequest {
    pub(crate) const fn new(
        key: ShadowSlotKey,
        requested_tier: ShadowResolutionTier,
        priority: f32,
    ) -> Self {
        Self {
            key,
            requested_tier,
            minimum_tier: ShadowResolutionTier::MIN,
            priority,
        }
    }

    pub(crate) const fn with_minimum_tier(mut self, minimum_tier: ShadowResolutionTier) -> Self {
        self.minimum_tier = minimum_tier;
        self
    }

    fn priority_score(self) -> f32 {
        if self.priority.is_finite() {
            self.priority.max(0.0)
        } else {
            0.0
        }
    }

    fn normalized_minimum_tier(self) -> ShadowResolutionTier {
        if self.minimum_tier.size_px() > self.requested_tier.size_px() {
            self.requested_tier
        } else {
            self.minimum_tier
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ShadowSlotAllocation {
    pub(crate) key: ShadowSlotKey,
    pub(crate) rect: ShadowAtlasRect,
    pub(crate) requested_tier: ShadowResolutionTier,
    pub(crate) allocated_tier: ShadowResolutionTier,
    pub(crate) priority: f32,
    pub(crate) reused_previous: bool,
}

impl ShadowSlotAllocation {
    pub(crate) fn was_downgraded(self) -> bool {
        self.allocated_tier.size_px() < self.requested_tier.size_px()
    }

    pub(crate) fn atlas_scale_bias(self, atlas_width: u32, atlas_height: u32) -> [f32; 4] {
        let width = atlas_width.max(1) as f32;
        let height = atlas_height.max(1) as f32;
        [
            self.rect.width as f32 / width,
            self.rect.height as f32 / height,
            self.rect.x as f32 / width,
            self.rect.y as f32 / height,
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShadowSlotRejectionReason {
    AtlasUnavailable,
    BelowMinimumTier,
    AtlasFull,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ShadowSlotRejection {
    pub(crate) key: ShadowSlotKey,
    pub(crate) requested_tier: ShadowResolutionTier,
    pub(crate) reason: ShadowSlotRejectionReason,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ShadowAtlasFrameAllocation {
    pub(crate) frame_index: u64,
    pub(crate) scale_factor: u32,
    pub(crate) allocations: Vec<ShadowSlotAllocation>,
    pub(crate) rejected: Vec<ShadowSlotRejection>,
}

impl ShadowAtlasFrameAllocation {
    pub(crate) fn allocation_for(&self, key: ShadowSlotKey) -> Option<&ShadowSlotAllocation> {
        self.allocations
            .iter()
            .find(|allocation| allocation.key == key)
    }
}

#[derive(Clone, Copy, Debug)]
struct PlannedShadowSlot {
    request: ShadowSlotRequest,
    allocated_tier: ShadowResolutionTier,
}

#[derive(Clone, Copy, Debug)]
struct RetainedShadowSlot {
    allocation: ShadowSlotAllocation,
    last_seen_frame: u64,
}

pub(crate) struct ShadowAtlasAllocator {
    config: ShadowAtlasConfig,
    frame_index: u64,
    previous: HashMap<ShadowSlotKey, RetainedShadowSlot>,
    preemption: HashMap<(ShadowSlotKey, ShadowSlotKey), u32>,
    last_frame: ShadowAtlasFrameAllocation,
}

impl ShadowAtlasAllocator {
    pub(crate) fn new(config: ShadowAtlasConfig) -> Self {
        Self {
            config,
            frame_index: 0,
            previous: HashMap::new(),
            preemption: HashMap::new(),
            last_frame: ShadowAtlasFrameAllocation::default(),
        }
    }

    pub(crate) fn config(&self) -> ShadowAtlasConfig {
        self.config
    }

    pub(crate) fn last_frame(&self) -> &ShadowAtlasFrameAllocation {
        &self.last_frame
    }

    pub(crate) fn allocate_frame(
        &mut self,
        requests: &[ShadowSlotRequest],
    ) -> ShadowAtlasFrameAllocation {
        self.frame_index = self.frame_index.saturating_add(1);
        let mut frame = ShadowAtlasFrameAllocation {
            frame_index: self.frame_index,
            scale_factor: 1,
            allocations: Vec::new(),
            rejected: Vec::new(),
        };

        let Some(available_rect) = self.config.available_rect() else {
            frame
                .rejected
                .extend(requests.iter().map(|request| ShadowSlotRejection {
                    key: request.key,
                    requested_tier: request.requested_tier,
                    reason: ShadowSlotRejectionReason::AtlasUnavailable,
                }));
            self.last_frame = frame.clone();
            return frame;
        };

        let requests = deduplicate_requests(requests);
        frame.scale_factor = estimate_scale_factor(&requests, available_rect.area());
        let scale_steps = frame.scale_factor.trailing_zeros();
        let mut planned = Vec::with_capacity(requests.len());
        for request in requests {
            match tier_for_request(request, scale_steps, self.config) {
                Some(allocated_tier) => planned.push(PlannedShadowSlot {
                    request,
                    allocated_tier,
                }),
                None => frame.rejected.push(ShadowSlotRejection {
                    key: request.key,
                    requested_tier: request.requested_tier,
                    reason: ShadowSlotRejectionReason::BelowMinimumTier,
                }),
            }
        }

        let planned_area = planned
            .iter()
            .map(|slot| {
                let size = slot.allocated_tier.size_px() as u64;
                size * size
            })
            .sum::<u64>();
        self.update_preemption_contention(&planned, planned_area > available_rect.area());

        let mut packer = FreeRectPacker::new(available_rect);
        let mut occupied = HashSet::new();
        let planned_by_key = planned
            .iter()
            .map(|slot| (slot.request.key, *slot))
            .collect::<HashMap<_, _>>();
        let mut retained = self.previous.values().copied().collect::<Vec<_>>();
        retained.sort_by(|lhs, rhs| compare_retained_slots(lhs, rhs));

        for retained_slot in retained {
            let key = retained_slot.allocation.key;
            let Some(planned_slot) = planned_by_key.get(&key).copied() else {
                continue;
            };
            if self
                .frame_index
                .saturating_sub(retained_slot.last_seen_frame)
                > self.config.slot_retention_frames as u64
            {
                continue;
            }
            if self.should_release_for_confirmed_preemption(key) {
                continue;
            }
            if retained_slot.allocation.allocated_tier != planned_slot.allocated_tier {
                continue;
            }
            if !packer.reserve(retained_slot.allocation.rect) {
                continue;
            }

            occupied.insert(key);
            frame.allocations.push(ShadowSlotAllocation {
                key,
                rect: retained_slot.allocation.rect,
                requested_tier: planned_slot.request.requested_tier,
                allocated_tier: planned_slot.allocated_tier,
                priority: planned_slot.request.priority_score(),
                reused_previous: true,
            });
        }

        planned.sort_by(compare_planned_slots);
        for planned_slot in planned {
            if occupied.contains(&planned_slot.request.key) {
                continue;
            }
            match self.pack_planned_slot(&mut packer, planned_slot) {
                Some(allocation) => {
                    occupied.insert(allocation.key);
                    frame.allocations.push(allocation);
                }
                None => frame.rejected.push(ShadowSlotRejection {
                    key: planned_slot.request.key,
                    requested_tier: planned_slot.request.requested_tier,
                    reason: ShadowSlotRejectionReason::AtlasFull,
                }),
            }
        }

        self.previous = frame
            .allocations
            .iter()
            .copied()
            .map(|allocation| {
                (
                    allocation.key,
                    RetainedShadowSlot {
                        allocation,
                        last_seen_frame: self.frame_index,
                    },
                )
            })
            .collect();
        self.last_frame = frame.clone();
        frame
    }

    fn pack_planned_slot(
        &self,
        packer: &mut FreeRectPacker,
        planned_slot: PlannedShadowSlot,
    ) -> Option<ShadowSlotAllocation> {
        let minimum_tier = planned_slot.request.normalized_minimum_tier();
        let mut tier = planned_slot.allocated_tier;
        loop {
            if let Some(rect) = packer.allocate_square(tier.size_px()) {
                return Some(ShadowSlotAllocation {
                    key: planned_slot.request.key,
                    rect,
                    requested_tier: planned_slot.request.requested_tier,
                    allocated_tier: tier,
                    priority: planned_slot.request.priority_score(),
                    reused_previous: false,
                });
            }
            let next_tier = tier.next_lower()?;
            if next_tier.size_px() < minimum_tier.size_px() {
                return None;
            }
            tier = next_tier;
        }
    }

    fn update_preemption_contention(
        &mut self,
        planned: &[PlannedShadowSlot],
        oversubscribed: bool,
    ) {
        if !oversubscribed {
            self.preemption.clear();
            return;
        }

        let mut active_pairs = HashSet::new();
        for retained_key in self.previous.keys().copied() {
            let Some(incumbent) = planned
                .iter()
                .find(|slot| slot.request.key == retained_key)
                .copied()
            else {
                continue;
            };
            let incumbent_priority = incumbent.request.priority_score();
            let required_priority =
                incumbent_priority * self.config.preemption_score_multiplier.max(1.0);
            for challenger in planned.iter().copied() {
                if challenger.request.key == incumbent.request.key {
                    continue;
                }
                if challenger.request.priority_score() < required_priority {
                    continue;
                }
                if challenger.allocated_tier.size_px() < incumbent.allocated_tier.size_px() {
                    continue;
                }
                let pair = (challenger.request.key, incumbent.request.key);
                active_pairs.insert(pair);
                let frames = self.preemption.entry(pair).or_insert(0);
                *frames = frames.saturating_add(1);
            }
        }
        self.preemption
            .retain(|pair, _| active_pairs.contains(pair));
    }

    fn should_release_for_confirmed_preemption(&self, incumbent: ShadowSlotKey) -> bool {
        let required_frames = self.config.preemption_confirmation_frames.max(1);
        self.preemption
            .iter()
            .any(|((_, challenged), frames)| *challenged == incumbent && *frames >= required_frames)
    }
}

impl Default for ShadowAtlasAllocator {
    fn default() -> Self {
        Self::new(ShadowAtlasConfig::default())
    }
}

fn compare_retained_slots(lhs: &RetainedShadowSlot, rhs: &RetainedShadowSlot) -> Ordering {
    lhs.allocation
        .rect
        .y
        .cmp(&rhs.allocation.rect.y)
        .then_with(|| lhs.allocation.rect.x.cmp(&rhs.allocation.rect.x))
        .then_with(|| lhs.allocation.key.cmp(&rhs.allocation.key))
}

fn compare_planned_slots(lhs: &PlannedShadowSlot, rhs: &PlannedShadowSlot) -> Ordering {
    rhs.request
        .priority_score()
        .total_cmp(&lhs.request.priority_score())
        .then_with(|| {
            rhs.allocated_tier
                .size_px()
                .cmp(&lhs.allocated_tier.size_px())
        })
        .then_with(|| lhs.request.key.cmp(&rhs.request.key))
}

fn deduplicate_requests(requests: &[ShadowSlotRequest]) -> Vec<ShadowSlotRequest> {
    let mut deduped = HashMap::<ShadowSlotKey, ShadowSlotRequest>::new();
    for request in requests.iter().copied() {
        deduped
            .entry(request.key)
            .and_modify(|current| {
                if compare_requests_for_dedup(request, *current) == Ordering::Less {
                    *current = request;
                }
            })
            .or_insert(request);
    }
    let mut requests = deduped.into_values().collect::<Vec<_>>();
    requests.sort_by(|lhs, rhs| lhs.key.cmp(&rhs.key));
    requests
}

fn compare_requests_for_dedup(lhs: ShadowSlotRequest, rhs: ShadowSlotRequest) -> Ordering {
    rhs.priority_score()
        .total_cmp(&lhs.priority_score())
        .then_with(|| {
            rhs.requested_tier
                .size_px()
                .cmp(&lhs.requested_tier.size_px())
        })
        .then_with(|| lhs.key.cmp(&rhs.key))
}

fn estimate_scale_factor(requests: &[ShadowSlotRequest], available_area: u64) -> u32 {
    if available_area == 0 {
        return 1;
    }
    let requested_area = requests
        .iter()
        .map(|request| {
            let size = request.requested_tier.size_px() as u64;
            size * size
        })
        .sum::<u64>();
    let mut scale_factor = 1u32;
    while requested_area > available_area.saturating_mul(scale_factor as u64 * scale_factor as u64)
    {
        scale_factor = scale_factor.saturating_mul(2);
        if scale_factor == 0 {
            return u32::MAX;
        }
    }
    scale_factor
}

fn tier_for_request(
    request: ShadowSlotRequest,
    scale_steps: u32,
    config: ShadowAtlasConfig,
) -> Option<ShadowResolutionTier> {
    let minimum_tier = request.normalized_minimum_tier();
    if minimum_tier.size_px() > config.max_slot_size() {
        return None;
    }

    let mut tier = request.requested_tier.downgraded_by_steps(scale_steps);
    if tier.size_px() < minimum_tier.size_px() {
        tier = minimum_tier;
    }
    while tier.size_px() > config.max_slot_size() {
        tier = tier.next_lower()?;
        if tier.size_px() < minimum_tier.size_px() {
            return None;
        }
    }
    Some(tier)
}

struct FreeRectPacker {
    free_rects: Vec<ShadowAtlasRect>,
}

impl FreeRectPacker {
    fn new(available_rect: ShadowAtlasRect) -> Self {
        Self {
            free_rects: vec![available_rect],
        }
    }

    fn reserve(&mut self, rect: ShadowAtlasRect) -> bool {
        if !self
            .free_rects
            .iter()
            .any(|free_rect| free_rect.contains_rect(rect))
        {
            return false;
        }

        let mut free_rects = Vec::new();
        for free_rect in self.free_rects.drain(..) {
            free_rects.extend(free_rect.subtract(rect));
        }
        self.free_rects = compact_free_rects(free_rects);
        true
    }

    fn allocate_square(&mut self, size: u32) -> Option<ShadowAtlasRect> {
        self.free_rects.sort_by(|lhs, rhs| {
            lhs.y
                .cmp(&rhs.y)
                .then_with(|| lhs.x.cmp(&rhs.x))
                .then_with(|| lhs.height.cmp(&rhs.height))
                .then_with(|| lhs.width.cmp(&rhs.width))
        });

        let rect = self
            .free_rects
            .iter()
            .find(|free_rect| free_rect.width >= size && free_rect.height >= size)
            .map(|free_rect| ShadowAtlasRect::new(free_rect.x, free_rect.y, size, size))?;
        if self.reserve(rect) {
            Some(rect)
        } else {
            None
        }
    }
}

fn compact_free_rects(rects: Vec<ShadowAtlasRect>) -> Vec<ShadowAtlasRect> {
    let rects = rects
        .into_iter()
        .filter(|rect| !rect.is_empty())
        .collect::<Vec<_>>();
    let mut compacted = Vec::new();
    'outer: for (index, rect) in rects.iter().copied().enumerate() {
        for (other_index, other) in rects.iter().copied().enumerate() {
            if index != other_index && other != rect && other.contains_rect(rect) {
                continue 'outer;
            }
        }
        if !compacted.contains(&rect) {
            compacted.push(rect);
        }
    }
    compacted
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(light_id: u64) -> ShadowSlotKey {
        ShadowSlotKey::new(light_id, 0)
    }

    fn request(
        light_id: u64,
        tier: ShadowResolutionTier,
        minimum_tier: ShadowResolutionTier,
        priority: f32,
    ) -> ShadowSlotRequest {
        ShadowSlotRequest::new(key(light_id), tier, priority).with_minimum_tier(minimum_tier)
    }

    fn no_allocations_overlap(allocations: &[ShadowSlotAllocation]) -> bool {
        for (index, lhs) in allocations.iter().enumerate() {
            for rhs in allocations.iter().skip(index + 1) {
                if lhs.rect.intersects(rhs.rect) {
                    return false;
                }
            }
        }
        true
    }

    #[test]
    fn render_shadow_atlas_allocates_tiers_descending() {
        let mut allocator = ShadowAtlasAllocator::new(ShadowAtlasConfig::new_square(512));
        let frame = allocator.allocate_frame(&[
            request(
                1,
                ShadowResolutionTier::T256,
                ShadowResolutionTier::T128,
                1.0,
            ),
            request(
                2,
                ShadowResolutionTier::T256,
                ShadowResolutionTier::T128,
                1.0,
            ),
            request(
                3,
                ShadowResolutionTier::T256,
                ShadowResolutionTier::T128,
                1.0,
            ),
            request(
                4,
                ShadowResolutionTier::T256,
                ShadowResolutionTier::T128,
                1.0,
            ),
        ]);

        assert_eq!(frame.scale_factor, 1);
        assert_eq!(frame.allocations.len(), 4);
        assert!(frame.rejected.is_empty());
        assert!(frame
            .allocations
            .iter()
            .all(|allocation| allocation.allocated_tier == ShadowResolutionTier::T256));
        assert!(no_allocations_overlap(&frame.allocations));
    }

    #[test]
    fn render_shadow_atlas_global_downgrade_fits_pressure() {
        let mut allocator = ShadowAtlasAllocator::new(ShadowAtlasConfig::new_square(512));
        let requests = (0..5)
            .map(|index| {
                request(
                    index,
                    ShadowResolutionTier::T256,
                    ShadowResolutionTier::T128,
                    1.0,
                )
            })
            .collect::<Vec<_>>();
        let frame = allocator.allocate_frame(&requests);

        assert_eq!(frame.scale_factor, 2);
        assert_eq!(frame.allocations.len(), 5);
        assert!(frame.rejected.is_empty());
        assert!(frame.allocations.iter().all(|allocation| {
            allocation.allocated_tier == ShadowResolutionTier::T128 && allocation.was_downgraded()
        }));
        assert!(no_allocations_overlap(&frame.allocations));
    }

    #[test]
    fn render_shadow_atlas_evicts_lowest_priority_on_pressure() {
        let mut allocator = ShadowAtlasAllocator::new(ShadowAtlasConfig::new_square(256));
        let frame = allocator.allocate_frame(&[
            request(
                1,
                ShadowResolutionTier::T128,
                ShadowResolutionTier::T128,
                1.0,
            ),
            request(
                2,
                ShadowResolutionTier::T128,
                ShadowResolutionTier::T128,
                2.0,
            ),
            request(
                3,
                ShadowResolutionTier::T128,
                ShadowResolutionTier::T128,
                3.0,
            ),
            request(
                4,
                ShadowResolutionTier::T128,
                ShadowResolutionTier::T128,
                4.0,
            ),
            request(
                5,
                ShadowResolutionTier::T128,
                ShadowResolutionTier::T128,
                5.0,
            ),
        ]);

        assert_eq!(frame.allocations.len(), 4);
        assert_eq!(frame.rejected.len(), 1);
        assert!(frame.allocation_for(key(5)).is_some());
        assert!(frame.allocation_for(key(4)).is_some());
        assert!(frame.allocation_for(key(3)).is_some());
        assert!(frame.allocation_for(key(2)).is_some());
        assert_eq!(frame.rejected[0].key, key(1));
    }

    #[test]
    fn render_shadow_atlas_hysteresis_prevents_flapping() {
        let mut allocator = ShadowAtlasAllocator::new(ShadowAtlasConfig::new_square(256));
        let first = allocator.allocate_frame(&[request(
            1,
            ShadowResolutionTier::T256,
            ShadowResolutionTier::T256,
            1.0,
        )]);
        let previous_rect = first.allocation_for(key(1)).unwrap().rect;

        let second = allocator.allocate_frame(&[
            request(
                1,
                ShadowResolutionTier::T256,
                ShadowResolutionTier::T256,
                1.0,
            ),
            request(
                2,
                ShadowResolutionTier::T256,
                ShadowResolutionTier::T256,
                1.2,
            ),
        ]);

        let retained = second.allocation_for(key(1)).unwrap();
        assert_eq!(retained.rect, previous_rect);
        assert!(retained.reused_previous);
        assert!(second.allocation_for(key(2)).is_none());
    }

    #[test]
    fn render_shadow_atlas_preempts_after_confirmed_priority_margin() {
        let mut allocator = ShadowAtlasAllocator::new(ShadowAtlasConfig::new_square(256));
        allocator.allocate_frame(&[request(
            1,
            ShadowResolutionTier::T256,
            ShadowResolutionTier::T256,
            1.0,
        )]);

        for _ in 0..SHADOW_ATLAS_PREEMPTION_FRAMES {
            allocator.allocate_frame(&[
                request(
                    1,
                    ShadowResolutionTier::T256,
                    ShadowResolutionTier::T256,
                    1.0,
                ),
                request(
                    2,
                    ShadowResolutionTier::T256,
                    ShadowResolutionTier::T256,
                    1.5,
                ),
            ]);
        }
        let frame = allocator.last_frame();

        assert!(frame.allocation_for(key(1)).is_none());
        assert!(frame.allocation_for(key(2)).is_some());
    }

    #[test]
    fn render_shadow_atlas_scale_bias_matches_slice_transform() {
        let allocation = ShadowSlotAllocation {
            key: key(7),
            rect: ShadowAtlasRect::new(128, 256, 512, 512),
            requested_tier: ShadowResolutionTier::T512,
            allocated_tier: ShadowResolutionTier::T512,
            priority: 1.0,
            reused_previous: false,
        };

        assert_eq!(
            allocation.atlas_scale_bias(2048, 2048),
            [0.25, 0.25, 0.0625, 0.125]
        );
    }
}
