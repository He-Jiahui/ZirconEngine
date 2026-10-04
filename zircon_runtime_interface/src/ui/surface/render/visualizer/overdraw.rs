use std::collections::HashMap;

use crate::ui::{event_ui::UiNodeId, layout::UiFrame};

use super::{UiPaintElement, UiPaintPayload, UiRenderVisualizerOverdrawRegion};

pub(super) fn overdraw_regions(
    elements: &[UiPaintElement],
) -> Vec<UiRenderVisualizerOverdrawRegion> {
    let visible_elements = elements
        .iter()
        .filter_map(|element| {
            if element.effects.opacity <= 0.0 || element.payload == UiPaintPayload::Empty {
                return None;
            }
            let frame = visible_paint_frame(element)?;
            Some((element.node_id, frame))
        })
        .collect::<Vec<_>>();
    overdraw_regions_from_visible(&visible_elements)
}

fn overdraw_regions_from_visible(
    visible_elements: &[(UiNodeId, UiFrame)],
) -> Vec<UiRenderVisualizerOverdrawRegion> {
    let mut regions = Vec::new();
    let mut frame_index = OverdrawFrameIndex::default();
    for left_index in 0..visible_elements.len() {
        for right_index in left_index + 1..visible_elements.len() {
            let (left_node_id, left_frame) = visible_elements[left_index];
            let (_, right_frame) = visible_elements[right_index];
            let Some(frame) = left_frame.intersection(right_frame) else {
                continue;
            };
            if !frame_index.insert_if_absent(frame, &regions) {
                continue;
            }

            let mut node_ids = vec![left_node_id];
            for (node_id, candidate_frame) in visible_elements {
                if candidate_frame.intersection(frame).is_some() {
                    node_ids.push(*node_id);
                }
            }
            // 一个节点可产生多个绘制项；热区的 paint_count 按不同 node_id 计数。
            node_ids.sort();
            node_ids.dedup();
            let paint_count = node_ids.len();
            regions.push(UiRenderVisualizerOverdrawRegion {
                frame,
                paint_count,
                node_ids,
                heat: paint_count as f32,
            });
        }
    }
    regions
}

fn visible_paint_frame(element: &UiPaintElement) -> Option<UiFrame> {
    let frame = element.geometry.render_bounds;
    if frame.width <= 0.0 || frame.height <= 0.0 {
        return None;
    }
    if let Some(clip) = element.clip.as_ref() {
        return frame.intersection(clip.frame);
    }
    if let Some(clip) = element.geometry.clip_frame {
        return frame.intersection(clip);
    }
    Some(frame)
}

#[derive(Default)]
struct OverdrawFrameIndex {
    region_indices_by_frame_bits: HashMap<[u32; 4], Vec<usize>>,
}

impl OverdrawFrameIndex {
    fn insert_if_absent(
        &mut self,
        frame: UiFrame,
        regions: &[UiRenderVisualizerOverdrawRegion],
    ) -> bool {
        let candidates = self
            .region_indices_by_frame_bits
            .entry(frame_bits(frame))
            .or_default();
        if candidates
            .iter()
            .any(|&region_index| regions[region_index].frame == frame)
        {
            return false;
        }
        candidates.push(regions.len());
        true
    }
}

fn frame_bits(frame: UiFrame) -> [u32; 4] {
    [
        canonical_component_bits(frame.x),
        canonical_component_bits(frame.y),
        canonical_component_bits(frame.width),
        canonical_component_bits(frame.height),
    ]
}

// 哈希桶先按位筛候选，正负零归一化后仍以 UiFrame 相等比较确认合并。
fn canonical_component_bits(value: f32) -> u32 {
    if value == 0.0 {
        0.0f32.to_bits()
    } else {
        value.to_bits()
    }
}

#[cfg(test)]
#[path = "tests/overdraw.rs"]
mod tests;
