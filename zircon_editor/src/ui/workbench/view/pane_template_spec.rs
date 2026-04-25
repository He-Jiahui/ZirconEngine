use serde::{Deserialize, Serialize};

use super::{PaneInteractionMode, PanePayloadKind, PaneRouteNamespace};

const DEFAULT_PANE_SHELL_DOCUMENT_ID: &str = "pane.surface_controls";
const DEFAULT_PANE_SHELL_COMPONENT_ID: &str = "PaneSurface";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaneTemplateSpec {
    pub shell: PaneShellSpec,
    pub body: PaneBodySpec,
}

impl PaneTemplateSpec {
    pub fn new(body: PaneBodySpec) -> Self {
        Self {
            shell: PaneShellSpec::default(),
            body,
        }
    }

    pub fn with_shell(mut self, shell: PaneShellSpec) -> Self {
        self.shell = shell;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaneShellSpec {
    pub document_id: String,
    pub component_id: String,
}

impl PaneShellSpec {
    pub fn new(document_id: impl Into<String>, component_id: impl Into<String>) -> Self {
        Self {
            document_id: document_id.into(),
            component_id: component_id.into(),
        }
    }

    pub fn pane_surface() -> Self {
        Self::new(
            DEFAULT_PANE_SHELL_DOCUMENT_ID,
            DEFAULT_PANE_SHELL_COMPONENT_ID,
        )
    }
}

impl Default for PaneShellSpec {
    fn default() -> Self {
        Self::pane_surface()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaneBodySpec {
    pub document_id: String,
    pub payload_kind: PanePayloadKind,
    pub route_namespace: PaneRouteNamespace,
    pub interaction_mode: PaneInteractionMode,
}

impl PaneBodySpec {
    pub fn new(
        document_id: impl Into<String>,
        payload_kind: PanePayloadKind,
        route_namespace: PaneRouteNamespace,
        interaction_mode: PaneInteractionMode,
    ) -> Self {
        Self {
            document_id: document_id.into(),
            payload_kind,
            route_namespace,
            interaction_mode,
        }
    }
}
