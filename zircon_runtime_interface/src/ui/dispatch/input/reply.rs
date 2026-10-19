use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;

use super::UiDispatchEffect;

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
/// 单个处理回复关联的派发阶段；hit_path_sequence 明确捕获、目标、冒泡的顺序。
pub enum UiDispatchPhase {
    #[default]
    Preprocess,
    PreviewTunnel,
    Direct,
    /// Delivers the event to the deepest node selected by the hit path.
    Target,
    /// Traverses the authoritative hit path from target back to root after target handling.
    Bubble,
    DefaultAction,
    /// Traverses the authoritative hit path from root to target before target handling.
    ///
    /// This is declared last to preserve the serialized and ordered representation of existing phases.
    Capture,
}

impl UiDispatchPhase {
    /// 返回该阶段的稳定小写标记。
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Preprocess => "preprocess",
            Self::PreviewTunnel => "preview_tunnel",
            Self::Capture => "capture",
            Self::Direct => "direct",
            Self::Target => "target",
            Self::Bubble => "bubble",
            Self::DefaultAction => "default_action",
        }
    }

    /// 判断阶段是否属于 Capture、Target 或 Bubble。
    pub const fn is_hit_path_phase(self) -> bool {
        matches!(self, Self::Capture | Self::Target | Self::Bubble)
    }

    /// Returns the authoritative hit-path delivery sequence.
    /// 返回命中路径的交付顺序 Capture → Target → Bubble。
    pub const fn hit_path_sequence() -> [Self; 3] {
        [Self::Capture, Self::Target, Self::Bubble]
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// 单个处理器对当前派发的处理结论；只有 Handled 与 Blocked 会停止回复合并。
pub enum UiDispatchDisposition {
    #[default]
    Unhandled,
    Handled,
    Blocked,
    Passthrough,
}

/// Replies are per-dispatch transient commands; durable widget state belongs to the tree/runtime.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
/// 单个派发步骤的瞬态回复；effects 由 Runtime 消费，持久状态仍归 tree/runtime。
pub struct UiDispatchReply {
    pub disposition: UiDispatchDisposition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handler: Option<UiNodeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<UiDispatchPhase>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub effects: Vec<UiDispatchEffect>,
}

impl UiDispatchReply {
    pub const fn unhandled() -> Self {
        Self {
            disposition: UiDispatchDisposition::Unhandled,
            handler: None,
            phase: None,
            effects: Vec::new(),
        }
    }

    pub const fn handled() -> Self {
        Self {
            disposition: UiDispatchDisposition::Handled,
            handler: None,
            phase: None,
            effects: Vec::new(),
        }
    }

    pub const fn blocked() -> Self {
        Self {
            disposition: UiDispatchDisposition::Blocked,
            handler: None,
            phase: None,
            effects: Vec::new(),
        }
    }

    pub const fn passthrough() -> Self {
        Self {
            disposition: UiDispatchDisposition::Passthrough,
            handler: None,
            phase: None,
            effects: Vec::new(),
        }
    }

    /// 记录处理该步骤的节点，并返回更新后的回复。
    pub fn from_handler(mut self, handler: UiNodeId) -> Self {
        self.handler = Some(handler);
        self
    }

    /// 记录回复所属阶段，并返回更新后的回复。
    pub fn in_phase(mut self, phase: UiDispatchPhase) -> Self {
        self.phase = Some(phase);
        self
    }

    /// 在回复末尾追加一个瞬态 effect。
    pub fn with_effect(mut self, effect: UiDispatchEffect) -> Self {
        self.effects.push(effect);
        self
    }

    /// 按迭代顺序追加一组瞬态 effects。
    pub fn with_effects(mut self, effects: impl IntoIterator<Item = UiDispatchEffect>) -> Self {
        self.effects.extend(effects);
        self
    }

    /// 仅 Handled 或 Blocked 返回 true。
    pub fn stops_propagation(&self) -> bool {
        matches!(
            self.disposition,
            UiDispatchDisposition::Handled | UiDispatchDisposition::Blocked
        )
    }

    /// 按 steps 的输入顺序合并处理结果；本方法不构造路由，也不执行 effects。
    pub fn merge_route(
        steps: impl IntoIterator<Item = UiDispatchReplyStep>,
    ) -> UiDispatchReplyMergeReport {
        let steps = steps.into_iter();
        let (lower_bound, upper_bound) = steps.size_hint();
        let route_trace_capacity = if upper_bound == Some(lower_bound) {
            lower_bound
        } else {
            0
        };
        let mut merged = UiDispatchReply::unhandled();
        let mut step_count = 0usize;
        let mut stopped = false;
        let mut stopped_at = None;
        let mut stopped_phase = None;
        let mut routed_steps = Vec::with_capacity(route_trace_capacity);

        for mut step in steps {
            step_count += 1;
            if step.reply.handler.is_none() {
                step.reply.handler = step.target;
            }
            if step.reply.phase.is_none() {
                step.reply.phase = Some(step.phase);
            }

            // Unhandled 丢弃本步 effects；Passthrough 累积 effects 后继续，并由首个非 Unhandled 回复占据汇总 handler/phase；
            // Handled/Blocked 累积本步 effects、覆盖汇总归属并终止路由。
            match step.reply.disposition {
                UiDispatchDisposition::Unhandled => {
                    let ignored_effect_count = step.reply.effects.len();
                    routed_steps.push(UiDispatchReplyStepTrace {
                        phase: step.phase,
                        target: step.target,
                        handler: step.reply.handler,
                        disposition: step.reply.disposition,
                        effect_start: merged.effects.len(),
                        effect_count: 0,
                        ignored_effect_count,
                        stopped: false,
                    });
                }
                UiDispatchDisposition::Passthrough => {
                    let effect_start = merged.effects.len();
                    let effect_count = step.reply.effects.len();
                    merged.effects.extend(step.reply.effects);
                    if merged.disposition == UiDispatchDisposition::Unhandled {
                        merged.disposition = UiDispatchDisposition::Passthrough;
                        merged.handler = step.reply.handler;
                        merged.phase = step.reply.phase;
                    }
                    routed_steps.push(UiDispatchReplyStepTrace {
                        phase: step.phase,
                        target: step.target,
                        handler: step.reply.handler,
                        disposition: step.reply.disposition,
                        effect_start,
                        effect_count,
                        ignored_effect_count: 0,
                        stopped: false,
                    });
                }
                UiDispatchDisposition::Handled | UiDispatchDisposition::Blocked => {
                    let effect_start = merged.effects.len();
                    let effect_count = step.reply.effects.len();
                    merged.effects.extend(step.reply.effects);
                    merged.disposition = step.reply.disposition;
                    merged.handler = step.reply.handler;
                    merged.phase = step.reply.phase;
                    stopped = true;
                    stopped_at = step.reply.handler;
                    stopped_phase = step.reply.phase;
                    routed_steps.push(UiDispatchReplyStepTrace {
                        phase: step.phase,
                        target: step.target,
                        handler: step.reply.handler,
                        disposition: step.reply.disposition,
                        effect_start,
                        effect_count,
                        ignored_effect_count: 0,
                        stopped: true,
                    });
                    break;
                }
            }
        }

        UiDispatchReplyMergeReport {
            reply: merged,
            step_count,
            stopped,
            stopped_at,
            stopped_phase,
            steps: routed_steps,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 路由中一个处理步骤的 phase、候选 target 与处理器回复。
pub struct UiDispatchReplyStep {
    pub phase: UiDispatchPhase,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<UiNodeId>,
    pub reply: UiDispatchReply,
}

impl UiDispatchReplyStep {
    /// 组合一个回复步骤；target 在 reply 未指定 handler 时作为其回退归属。
    pub fn new(phase: UiDispatchPhase, target: Option<UiNodeId>, reply: UiDispatchReply) -> Self {
        Self {
            phase,
            target,
            reply,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// merge_route 已消费的步骤数、最终回复及停止位置和步骤轨迹。
pub struct UiDispatchReplyMergeReport {
    pub reply: UiDispatchReply,
    pub step_count: usize,
    pub stopped: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stopped_at: Option<UiNodeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stopped_phase: Option<UiDispatchPhase>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<UiDispatchReplyStepTrace>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 一个已消费步骤对汇总 effect 列表的贡献范围及忽略/停止状态。
pub struct UiDispatchReplyStepTrace {
    pub phase: UiDispatchPhase,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<UiNodeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handler: Option<UiNodeId>,
    pub disposition: UiDispatchDisposition,
    pub effect_start: usize,
    pub effect_count: usize,
    #[serde(default, skip_serializing_if = "usize_is_zero")]
    pub ignored_effect_count: usize,
    pub stopped: bool,
}

fn usize_is_zero(value: &usize) -> bool {
    *value == 0
}
