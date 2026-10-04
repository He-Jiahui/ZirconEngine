//! 剪辑评估错误先缓存在评估器中，再由帧管线按事件队列准入结果发布；延期实体必须保留待发布事件。
use std::collections::BTreeSet;

use zircon_runtime::scene::EntityId;

use crate::AnimationEvaluationDiagnostic;

use super::{AnimationAssetRevision, AnimationClipEvaluator, AnimationEvaluationError};

impl AnimationClipEvaluator {
    // TODO: [CR-PLUGIN-ANIMATION-0001] 确认资源级去重是否允许抑制其他实体的诊断；去重键没有实体，而发布按实体准入；下一步覆盖同资源两个实体且首实体延期的测试。
    pub fn record_diagnostic(
        &mut self,
        entity: EntityId,
        skeleton: AnimationAssetRevision,
        clip: AnimationAssetRevision,
        error: AnimationEvaluationError,
    ) {
        let key = (
            skeleton.id(),
            skeleton.revision(),
            clip.id(),
            clip.revision(),
            error.to_string(),
        );
        if self.reported_diagnostics.insert(key.clone()) {
            self.diagnostic_order.push_back(key);
            self.pending_diagnostics
                .push(AnimationEvaluationDiagnostic {
                    entity,
                    skeleton,
                    clip,
                    error,
                });
            self.enforce_diagnostic_limit();
        }
    }

    pub fn drain_diagnostics(&mut self) -> Vec<AnimationEvaluationDiagnostic> {
        std::mem::take(&mut self.pending_diagnostics)
    }

    pub fn drain_diagnostics_excluding(
        &mut self,
        deferred_entities: &BTreeSet<EntityId>,
    ) -> Vec<AnimationEvaluationDiagnostic> {
        if deferred_entities.is_empty() {
            return self.drain_diagnostics();
        }
        let (retained, admitted) = std::mem::take(&mut self.pending_diagnostics)
            .into_iter()
            .partition(|diagnostic| deferred_entities.contains(&diagnostic.entity));
        self.pending_diagnostics = retained;
        admitted
    }

    pub(crate) fn reset_diagnostics(&mut self) {
        self.pending_diagnostics.clear();
        self.reported_diagnostics.clear();
        self.diagnostic_order.clear();
    }
}

#[cfg(test)]
#[path = "tests/diagnostics.rs"]
mod tests;
