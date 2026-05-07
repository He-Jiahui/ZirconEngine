use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;

use super::UiDispatchEffect;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiDispatchPhase {
    #[default]
    Preprocess,
    PreviewTunnel,
    Direct,
    Target,
    Bubble,
    DefaultAction,
}

impl UiDispatchPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Preprocess => "preprocess",
            Self::PreviewTunnel => "preview_tunnel",
            Self::Direct => "direct",
            Self::Target => "target",
            Self::Bubble => "bubble",
            Self::DefaultAction => "default_action",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiDispatchDisposition {
    #[default]
    Unhandled,
    Handled,
    Blocked,
    Passthrough,
}

/// Replies are per-dispatch transient commands; durable widget state belongs to the tree/runtime.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
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

    pub fn from_handler(mut self, handler: UiNodeId) -> Self {
        self.handler = Some(handler);
        self
    }

    pub fn in_phase(mut self, phase: UiDispatchPhase) -> Self {
        self.phase = Some(phase);
        self
    }

    pub fn with_effect(mut self, effect: UiDispatchEffect) -> Self {
        self.effects.push(effect);
        self
    }

    pub fn with_effects(mut self, effects: impl IntoIterator<Item = UiDispatchEffect>) -> Self {
        self.effects.extend(effects);
        self
    }

    pub fn stops_propagation(&self) -> bool {
        matches!(
            self.disposition,
            UiDispatchDisposition::Handled | UiDispatchDisposition::Blocked
        )
    }

    pub fn merge_route(
        steps: impl IntoIterator<Item = UiDispatchReplyStep>,
    ) -> UiDispatchReplyMergeReport {
        let mut merged = UiDispatchReply::unhandled();
        let mut step_count = 0usize;
        let mut stopped = false;
        let mut stopped_at = None;
        let mut stopped_phase = None;

        for mut step in steps {
            step_count += 1;
            if step.reply.handler.is_none() {
                step.reply.handler = step.target;
            }
            if step.reply.phase.is_none() {
                step.reply.phase = Some(step.phase);
            }

            merged.effects.extend(step.reply.effects);

            match step.reply.disposition {
                UiDispatchDisposition::Unhandled => {}
                UiDispatchDisposition::Passthrough => {
                    if merged.disposition == UiDispatchDisposition::Unhandled {
                        merged.disposition = UiDispatchDisposition::Passthrough;
                        merged.handler = step.reply.handler;
                        merged.phase = step.reply.phase;
                    }
                }
                UiDispatchDisposition::Handled | UiDispatchDisposition::Blocked => {
                    merged.disposition = step.reply.disposition;
                    merged.handler = step.reply.handler;
                    merged.phase = step.reply.phase;
                    stopped = true;
                    stopped_at = step.reply.handler;
                    stopped_phase = step.reply.phase;
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
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiDispatchReplyStep {
    pub phase: UiDispatchPhase,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<UiNodeId>,
    pub reply: UiDispatchReply,
}

impl UiDispatchReplyStep {
    pub fn new(phase: UiDispatchPhase, target: Option<UiNodeId>, reply: UiDispatchReply) -> Self {
        Self {
            phase,
            target,
            reply,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiDispatchReplyMergeReport {
    pub reply: UiDispatchReply,
    pub step_count: usize,
    pub stopped: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stopped_at: Option<UiNodeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stopped_phase: Option<UiDispatchPhase>,
}
