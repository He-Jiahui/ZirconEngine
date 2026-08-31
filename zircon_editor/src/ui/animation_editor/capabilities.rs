#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimationEditorCapabilityDescriptor {
    id: &'static str,
    available: bool,
}

impl AnimationEditorCapabilityDescriptor {
    pub const fn id(self) -> &'static str {
        self.id
    }

    pub const fn available(self) -> bool {
        self.available
    }
}

const ANIMATION_EDITOR_CAPABILITY_TABLE: [AnimationEditorCapabilityDescriptor; 10] = [
    AnimationEditorCapabilityDescriptor {
        id: "animation.document.open",
        available: true,
    },
    AnimationEditorCapabilityDescriptor {
        id: "animation.sequence.timeline_edit",
        available: true,
    },
    AnimationEditorCapabilityDescriptor {
        id: "animation.graph.node.output",
        available: true,
    },
    AnimationEditorCapabilityDescriptor {
        id: "animation.graph.node.blend",
        available: true,
    },
    AnimationEditorCapabilityDescriptor {
        id: "animation.graph.node.clip",
        available: false,
    },
    AnimationEditorCapabilityDescriptor {
        id: "animation.graph.node.additive",
        available: false,
    },
    AnimationEditorCapabilityDescriptor {
        id: "animation.graph.node.mask",
        available: false,
    },
    AnimationEditorCapabilityDescriptor {
        id: "animation.state_machine.basic_edit",
        available: true,
    },
    AnimationEditorCapabilityDescriptor {
        id: "animation.compiler.semantic",
        available: false,
    },
    AnimationEditorCapabilityDescriptor {
        id: "animation.preview.runtime",
        available: false,
    },
];

use crate::core::editing::animation_document::AnimationGraphNodeKind;
use zircon_runtime::core::framework::animation::compiler::AnimationGraphNodeSchemaKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationEditorCommandRejectionReason {
    UnknownGraphNodeKind,
    UnavailableGraphNodeKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnimationEditorCommandDiagnostic {
    reason: AnimationEditorCommandRejectionReason,
    requested_node_kind: String,
}

impl AnimationEditorCommandDiagnostic {
    fn unknown_graph_node_kind(requested_node_kind: &str) -> Self {
        Self {
            reason: AnimationEditorCommandRejectionReason::UnknownGraphNodeKind,
            requested_node_kind: requested_node_kind.to_string(),
        }
    }

    fn unavailable_graph_node_kind(requested_node_kind: &str) -> Self {
        Self {
            reason: AnimationEditorCommandRejectionReason::UnavailableGraphNodeKind,
            requested_node_kind: requested_node_kind.to_string(),
        }
    }

    pub const fn code(&self) -> &'static str {
        match self.reason {
            AnimationEditorCommandRejectionReason::UnknownGraphNodeKind => "ZR-ANIM-CMD-001",
            AnimationEditorCommandRejectionReason::UnavailableGraphNodeKind => "ZR-ANIM-CMD-002",
        }
    }

    pub const fn reason(&self) -> AnimationEditorCommandRejectionReason {
        self.reason
    }

    pub fn requested_node_kind(&self) -> &str {
        &self.requested_node_kind
    }
}

impl std::fmt::Display for AnimationEditorCommandDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.reason {
            AnimationEditorCommandRejectionReason::UnknownGraphNodeKind => write!(
                f,
                "[{}] unknown animation graph node kind `{}`",
                self.code(),
                self.requested_node_kind
            ),
            AnimationEditorCommandRejectionReason::UnavailableGraphNodeKind => write!(
                f,
                "[{}] animation graph node kind `{}` is declared but not implemented",
                self.code(),
                self.requested_node_kind
            ),
        }
    }
}

pub fn animation_editor_capability_table() -> &'static [AnimationEditorCapabilityDescriptor] {
    &ANIMATION_EDITOR_CAPABILITY_TABLE
}

pub(crate) fn resolve_animation_graph_node_kind(
    value: &str,
) -> Result<AnimationGraphNodeKind, AnimationEditorCommandDiagnostic> {
    let canonical = value.trim().to_ascii_lowercase();
    let Some(schema_kind) = AnimationGraphNodeSchemaKind::from_id(&canonical) else {
        return Err(AnimationEditorCommandDiagnostic::unknown_graph_node_kind(
            value,
        ));
    };
    let capability_id = match schema_kind {
        AnimationGraphNodeSchemaKind::Clip => "animation.graph.node.clip",
        AnimationGraphNodeSchemaKind::Blend => "animation.graph.node.blend",
        AnimationGraphNodeSchemaKind::Additive => "animation.graph.node.additive",
        AnimationGraphNodeSchemaKind::Mask => "animation.graph.node.mask",
        AnimationGraphNodeSchemaKind::Output => "animation.graph.node.output",
    };
    if !capability_available(capability_id) {
        return Err(AnimationEditorCommandDiagnostic::unavailable_graph_node_kind(value));
    }
    match schema_kind {
        AnimationGraphNodeSchemaKind::Output => Ok(AnimationGraphNodeKind::Output),
        AnimationGraphNodeSchemaKind::Blend => Ok(AnimationGraphNodeKind::Blend),
        AnimationGraphNodeSchemaKind::Clip
        | AnimationGraphNodeSchemaKind::Additive
        | AnimationGraphNodeSchemaKind::Mask => {
            Err(AnimationEditorCommandDiagnostic::unavailable_graph_node_kind(value))
        }
    }
}

fn capability_available(capability_id: &str) -> bool {
    animation_editor_capability_table()
        .iter()
        .find(|descriptor| descriptor.id == capability_id)
        .is_some_and(|descriptor| descriptor.available)
}

#[cfg(test)]
mod tests {
    use super::{
        AnimationEditorCommandRejectionReason, animation_editor_capability_table,
        resolve_animation_graph_node_kind,
    };
    use crate::core::editing::animation_document::AnimationGraphNodeKind;

    #[test]
    fn capability_table_exposes_current_animation_authoring_truth() {
        let table = animation_editor_capability_table();
        let available = table
            .iter()
            .filter(|descriptor| descriptor.available())
            .map(|descriptor| descriptor.id())
            .collect::<Vec<_>>();

        assert!(available.contains(&"animation.document.open"));
        assert!(available.contains(&"animation.graph.node.output"));
        assert!(available.contains(&"animation.graph.node.blend"));
        assert!(!available.contains(&"animation.graph.node.clip"));
        assert!(!available.contains(&"animation.compiler.semantic"));
        assert!(!available.contains(&"animation.preview.runtime"));
    }

    #[test]
    fn graph_node_resolution_uses_the_capability_table_for_typed_rejections() {
        assert_eq!(
            resolve_animation_graph_node_kind("BLEND"),
            Ok(AnimationGraphNodeKind::Blend)
        );

        let unavailable = resolve_animation_graph_node_kind("clip")
            .expect_err("declared but unavailable node kinds must be rejected");
        assert_eq!(unavailable.code(), "ZR-ANIM-CMD-002");
        assert_eq!(
            unavailable.reason(),
            AnimationEditorCommandRejectionReason::UnavailableGraphNodeKind
        );

        let unknown = resolve_animation_graph_node_kind("pose_cache")
            .expect_err("unknown node kinds must be rejected");
        assert_eq!(unknown.code(), "ZR-ANIM-CMD-001");
        assert_eq!(
            unknown.reason(),
            AnimationEditorCommandRejectionReason::UnknownGraphNodeKind
        );
    }
}
