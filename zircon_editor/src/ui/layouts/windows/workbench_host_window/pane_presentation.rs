use crate::ui::animation_editor::AnimationEditorPanePresentation;
use crate::ui::layouts::views::SceneViewportChromeData;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;
use crate::ui::workbench::view::{
    PaneBodySpec, PaneInteractionMode, PanePayloadKind, PaneRouteNamespace,
};

use super::pane_payload::PanePayload;
use super::pane_payload_builders::build_payload;
use super::{BuildExportPaneViewData, ModulePluginsPaneViewData};

#[derive(Clone)]
pub(crate) struct PanePayloadBuildContext<'a> {
    pub chrome: &'a EditorChromeSnapshot,
    pub animation_pane: Option<&'a AnimationEditorPanePresentation>,
    pub runtime_diagnostics:
        Option<&'a zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot>,
    pub module_plugins: Option<&'a ModulePluginsPaneViewData>,
    pub build_export: Option<&'a BuildExportPaneViewData>,
}

impl<'a> PanePayloadBuildContext<'a> {
    pub fn new(chrome: &'a EditorChromeSnapshot) -> Self {
        Self {
            chrome,
            animation_pane: None,
            runtime_diagnostics: None,
            module_plugins: None,
            build_export: None,
        }
    }

    pub fn with_animation_pane(
        mut self,
        animation_pane: &'a AnimationEditorPanePresentation,
    ) -> Self {
        self.animation_pane = Some(animation_pane);
        self
    }

    pub fn with_runtime_diagnostics(
        mut self,
        runtime_diagnostics: &'a zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot,
    ) -> Self {
        self.runtime_diagnostics = Some(runtime_diagnostics);
        self
    }

    pub fn with_module_plugins(mut self, module_plugins: &'a ModulePluginsPaneViewData) -> Self {
        self.module_plugins = Some(module_plugins);
        self
    }

    pub fn with_build_export(mut self, build_export: &'a BuildExportPaneViewData) -> Self {
        self.build_export = Some(build_export);
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PaneBodyPresentation {
    pub document_id: String,
    pub payload_kind: PanePayloadKind,
    pub route_namespace: PaneRouteNamespace,
    pub interaction_mode: PaneInteractionMode,
    pub payload: PanePayload,
}

#[allow(dead_code)]
#[derive(Clone)]
pub(crate) struct PaneShellPresentation {
    pub title: String,
    pub icon_key: String,
    pub subtitle: String,
    pub info: String,
    pub empty_state: Option<PaneEmptyStatePresentation>,
    pub show_toolbar: bool,
    pub viewport: SceneViewportChromeData,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PaneActionPresentation {
    pub label: String,
    pub action_id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PaneEmptyStatePresentation {
    pub title: String,
    pub body: String,
    pub primary_action: Option<PaneActionPresentation>,
    pub secondary_action: Option<PaneActionPresentation>,
    pub secondary_hint: String,
}

impl PaneShellPresentation {
    pub fn new(
        title: impl Into<String>,
        icon_key: impl Into<String>,
        subtitle: impl Into<String>,
        info: impl Into<String>,
        empty_state: Option<PaneEmptyStatePresentation>,
        show_toolbar: bool,
        viewport: SceneViewportChromeData,
    ) -> Self {
        Self {
            title: title.into(),
            icon_key: icon_key.into(),
            subtitle: subtitle.into(),
            info: info.into(),
            empty_state,
            show_toolbar,
            viewport,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub(crate) struct PanePresentation {
    pub shell: PaneShellPresentation,
    pub body: PaneBodyPresentation,
}

impl PanePresentation {
    pub fn new(shell: PaneShellPresentation, body: PaneBodyPresentation) -> Self {
        Self { shell, body }
    }
}

pub(crate) fn build_pane_body_presentation(
    spec: &PaneBodySpec,
    context: &PanePayloadBuildContext<'_>,
) -> PaneBodyPresentation {
    PaneBodyPresentation {
        document_id: spec.document_id.clone(),
        payload_kind: spec.payload_kind,
        route_namespace: spec.route_namespace,
        interaction_mode: spec.interaction_mode,
        payload: build_payload(spec.payload_kind, context),
    }
}
