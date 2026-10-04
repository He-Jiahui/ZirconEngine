//! 稠密图节点、参数槽及剪辑贡献的只读契约；输出携带播放速度、混合模式和骨架掩码。
use std::sync::Arc;

use zircon_runtime::asset::AssetReference;
use zircon_runtime::core::framework::animation::{
    AnimationGraphBlendMode, AnimationParameterValue,
};
use zircon_runtime::core::math::Real;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct GraphNodeSlot(u32);

impl GraphNodeSlot {
    pub(super) fn new(index: usize) -> Option<Self> {
        u32::try_from(index).ok().map(Self)
    }

    pub(super) fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ParameterSlot(u32);

impl ParameterSlot {
    pub(super) fn new(index: usize) -> Option<Self> {
        u32::try_from(index).ok().map(Self)
    }

    pub(super) fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Debug)]
pub(super) struct CompiledParameter {
    pub(super) name: String,
    pub(super) default_value: AnimationParameterValue,
}

#[derive(Clone, Debug)]
pub(super) enum CompiledGraphNode {
    Clip {
        clip: AssetReference,
        playback_speed: Real,
        looping: bool,
    },
    Blend {
        inputs: Box<[GraphNodeSlot]>,
        weight_parameter: Option<ParameterSlot>,
    },
    Additive {
        base: GraphNodeSlot,
        additive: GraphNodeSlot,
        weight_parameter: Option<ParameterSlot>,
    },
    Mask {
        input: GraphNodeSlot,
        target_mask: Arc<[bool]>,
    },
}

#[derive(Clone, Debug)]
/// 编译时固定节点、参数默认值和骨架掩码，后续源资产变更不修改此对象。
pub struct CompiledAnimationGraph {
    pub(super) parameters: Box<[CompiledParameter]>,
    pub(super) nodes: Box<[CompiledGraphNode]>,
    pub(super) output: GraphNodeSlot,
    pub(super) evaluation_order: Box<[GraphNodeSlot]>,
}

impl CompiledAnimationGraph {
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn parameter_count(&self) -> usize {
        self.parameters.len()
    }
}

#[derive(Clone, Debug)]
/// 一条剪辑贡献的播放与混合约束；空掩码视图表示未限定目标。
pub struct CompiledGraphClipInstance {
    pub(super) clip: AssetReference,
    pub(super) playback_speed: Real,
    pub(super) looping: bool,
    pub(super) weight: Real,
    pub(super) blend_mode: AnimationGraphBlendMode,
    pub(super) target_mask: Option<Arc<[bool]>>,
}

impl CompiledGraphClipInstance {
    pub fn clip(&self) -> &AssetReference {
        &self.clip
    }

    pub fn playback_speed(&self) -> Real {
        self.playback_speed
    }

    pub fn looping(&self) -> bool {
        self.looping
    }

    pub fn weight(&self) -> Real {
        self.weight
    }

    pub fn blend_mode(&self) -> AnimationGraphBlendMode {
        self.blend_mode
    }

    pub fn target_mask(&self) -> &[bool] {
        self.target_mask.as_deref().unwrap_or(&[])
    }

    pub(crate) fn target_mask_owner(&self) -> Option<Arc<[bool]>> {
        self.target_mask.clone()
    }
}

#[derive(Clone, Debug, Default)]
/// 所有消费者共享的求值快照；剪辑加载、事件准入和姿态合成由帧管线负责。
pub struct CompiledAnimationGraphEvaluation {
    pub(super) clips: Vec<CompiledGraphClipInstance>,
}

impl CompiledAnimationGraphEvaluation {
    pub fn clips(&self) -> &[CompiledGraphClipInstance] {
        &self.clips
    }
}
