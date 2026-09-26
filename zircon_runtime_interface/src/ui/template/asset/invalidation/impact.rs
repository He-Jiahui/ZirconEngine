use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::ui::tree::UiDirtyFlags;

use super::UiInvalidationStage;

// TODO: [CR-UITEMPLATE-0004] 确认报告影响何时交给执行路径，避免两套映射长期分叉。
/// 失效图对后续 UI 工作的影响摘要，折叠为脏域、投影与重建标志。
/// 当前编译器把它写入报告，热重载执行仍由独立的 plan 决定。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiInvalidationImpact {
    pub dirty: UiDirtyFlags,
    pub projection_dirty: bool,
    pub rebuild_required: bool,
}

impl UiInvalidationImpact {
    pub fn from_stages(stages: &BTreeSet<UiInvalidationStage>) -> Self {
        let mut impact = Self::default();
        for stage in stages {
            impact.include_stage(*stage);
        }
        impact
    }

    pub fn include_stage(&mut self, stage: UiInvalidationStage) {
        match stage {
            UiInvalidationStage::SourceParse
            | UiInvalidationStage::DocumentShape
            | UiInvalidationStage::ImportGraph
            | UiInvalidationStage::DescriptorRegistry
            | UiInvalidationStage::ComponentContract => {
                self.rebuild_required = true;
                self.dirty.layout = true;
                self.dirty.hit_test = true;
                self.dirty.render = true;
                self.dirty.style = true;
                self.dirty.text = true;
                self.dirty.input = true;
                self.dirty.visible_range = true;
                self.projection_dirty = true;
            }
            UiInvalidationStage::ResourceDependency => {
                self.rebuild_required = true;
                self.dirty.render = true;
                self.projection_dirty = true;
            }
            UiInvalidationStage::SelectorMatch | UiInvalidationStage::StyleValue => {
                self.dirty.style = true;
                self.dirty.layout = true;
                self.dirty.hit_test = true;
                self.dirty.render = true;
                self.dirty.text = true;
            }
            UiInvalidationStage::Layout => {
                self.dirty.layout = true;
                self.dirty.hit_test = true;
                self.dirty.render = true;
            }
            UiInvalidationStage::Render => {
                self.dirty.render = true;
            }
            UiInvalidationStage::Interaction => {
                self.dirty.input = true;
            }
            UiInvalidationStage::Projection => {
                self.projection_dirty = true;
            }
        }
    }
}
