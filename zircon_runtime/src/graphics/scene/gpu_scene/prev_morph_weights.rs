//! current 保存本次抽取的权重快照，previous 只随上一成功场景帧推进。
//! 稳定 Arc 身份让未变化实例复用快照；稳定实例键本身不描述 mesh 或 target schema 的代次。

use std::{collections::HashMap, sync::Arc};

use super::gpu_scene::GpuScene;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GpuScenePrevMorphWeightsRollReport {
    pub(crate) current_weight_state_count: usize,
    pub(crate) previous_weight_state_count: usize,
    pub(crate) removed_previous_weight_state_count: usize,
}

impl GpuScene {
    pub(crate) fn previous_morph_weights(&self, stable_instance_key: u64) -> Option<&[f32]> {
        self.previous_morph_weights
            .get(&stable_instance_key)
            .map(Arc::as_ref)
    }

    /// 非空切片（包括全零）保留为本帧历史；None 或空切片移除本帧状态。
    pub(crate) fn stage_current_morph_weights(
        &mut self,
        stable_instance_key: u64,
        weights: Option<&[f32]>,
    ) {
        if let Some(weights) = weights.filter(|weights| !weights.is_empty()) {
            if self
                .current_morph_weights
                .get(&stable_instance_key)
                .is_some_and(|current| current.as_ref() == weights)
            {
                return;
            }
            self.current_morph_weights
                .insert(stable_instance_key, Arc::from(weights));
        } else {
            self.current_morph_weights.remove(&stable_instance_key);
        }
    }

    pub(crate) fn roll_prev_morph_weights_after_success(
        &mut self,
    ) -> GpuScenePrevMorphWeightsRollReport {
        let removed_previous_weight_state_count = roll_previous_morph_weight_snapshots(
            &mut self.previous_morph_weights,
            &self.current_morph_weights,
        );

        let previous_weight_state_count = self.previous_morph_weights.len();
        GpuScenePrevMorphWeightsRollReport {
            current_weight_state_count: self.current_morph_weights.len(),
            previous_weight_state_count,
            removed_previous_weight_state_count,
        }
    }
}

fn roll_previous_morph_weight_snapshots(
    previous: &mut HashMap<u64, Arc<[f32]>>,
    current: &HashMap<u64, Arc<[f32]>>,
) -> usize {
    let mut removed = 0;
    previous.retain(|key, previous_weights| {
        let Some(current_weights) = current.get(key) else {
            removed += 1;
            return false;
        };
        if !Arc::ptr_eq(previous_weights, current_weights) {
            *previous_weights = Arc::clone(current_weights);
        }
        true
    });

    // 上面的 retain 已使 previous 的键成为 current 的子集；只有长度较小时才可能缺少新键。
    if previous.len() < current.len() {
        previous.reserve(current.len() - previous.len());
        for (key, weights) in current {
            if let std::collections::hash_map::Entry::Vacant(entry) = previous.entry(*key) {
                entry.insert(Arc::clone(weights));
            }
        }
    }
    removed
}

#[cfg(test)]
#[path = "tests/prev_morph_weights.rs"]
mod tests;
