//! 工作台pane的模板外壳与内容声明；共用外壳承载不同资源、数据载荷和事件域。
use serde::{Deserialize, Serialize};

use super::{PaneInteractionMode, PanePayloadKind, PaneRouteNamespace};

const DEFAULT_PANE_SHELL_DOCUMENT_ID: &str = "res://ui/editor/host/pane_surface_controls.zui";
const DEFAULT_PANE_SHELL_COMPONENT_ID: &str = "PaneSurface";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 组合外壳与内容声明；构造及注册不校验资源或路由，后续投影才尝试按文档ID装载并消费载荷。
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
/// 公共pane外壳模板身份；默认资源由宿主设计资产提供，不在此处加载。
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
/// pane特定内容及载荷/路由契约；kind、namespace、mode须与宿主实际处理器一致。
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
