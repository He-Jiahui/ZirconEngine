use crate::scene::modes::SceneModeActivation;
use crate::scene::selection::SelectionMutation;
use crate::scene::viewport::{
    DisplayMode, GridMode, PivotMode, ProjectionMode, TransformSpace, ViewOrientation,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use zircon_runtime::core::framework::animation::AnimationTrackPath;

use super::{
    EditorHierarchyEvent, InspectorFieldChange, LayoutCommand, MenuAction, SelectionHostEvent,
    ViewInstanceId,
};

// 事件身份与序号均为强类型包装，避免派发入口、日志和监听器把两个全局编号互换。
macro_rules! define_id {
    ($name:ident) => {
        #[derive(
            Clone,
            Copy,
            Debug,
            Default,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Serialize,
            Deserialize,
        )]
        pub struct $name(pub u64);

        impl $name {
            pub const fn new(value: u64) -> Self {
                Self(value)
            }
        }
    };
}

define_id!(EditorEventId);
define_id!(EditorEventSequence);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 派发来源写入日志供诊断与监听器筛选；Replay 是再次执行的来源，而非恢复旧执行状态。
pub enum EditorEventSource {
    RetainedHost,
    Headless,
    Cli,
    Mcp,
    Replay,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorAssetSurface {
    Activity,
    Browser,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorAssetViewMode {
    List,
    Thumbnail,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorAssetUtilityTab {
    Preview,
    References,
    Metadata,
    Plugins,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 资产面板与资产操作的语义请求；资产定位、删除和导入的权限与副作用由宿主执行层决定。
pub enum EditorAssetEvent {
    OpenAsset {
        asset_locator: String,
    },
    SelectFolder {
        folder_id: String,
    },
    SelectItem {
        asset_uuid: String,
    },
    ActivateReference {
        asset_uuid: String,
    },
    SetSearchQuery {
        query: String,
    },
    SetKindFilter {
        kind: Option<String>,
    },
    SetViewMode {
        surface: EditorAssetSurface,
        view_mode: EditorAssetViewMode,
    },
    SetUtilityTab {
        surface: EditorAssetSurface,
        tab: EditorAssetUtilityTab,
    },
    RelocateAsset {
        asset_uuid: String,
        target_locator: String,
    },
    DeleteAsset {
        asset_uuid: String,
    },
    OpenAssetBrowser,
    LocateSelectedAsset,
    ImportModel,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditorInspectorEvent {
    pub subject_path: String,
    pub changes: Vec<InspectorFieldChange>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EditorDraftEvent {
    SetInspectorField {
        subject_path: String,
        field_id: String,
        value: String,
    },
    SetMeshImportPath {
        value: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 操作系统结果进入统一事件流的投影；交易标识只在对应操作提交后提供给记录。
pub enum EditorOperationEvent {
    ControlFailure {
        operation_id: String,
        error: String,
    },
    CommandExecuted {
        operation_id: String,
        transaction_id: u64,
        group_open: bool,
    },
    NativeCommandExecuted {
        operation_id: String,
        status_code: u32,
    },
    EditQueued {
        operation_id: String,
        pending_edit_id: u64,
        coalesced: bool,
        evicted_pending_edit_ids: Vec<u64>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 动画时间线、图和状态机的编辑意图；事件只携带目标标识，执行层负责定位与验证。
pub enum EditorAnimationEvent {
    AddKey {
        track_path: AnimationTrackPath,
        frame: u32,
    },
    RemoveKey {
        track_path: AnimationTrackPath,
        frame: u32,
    },
    CreateTrack {
        track_path: AnimationTrackPath,
    },
    RemoveTrack {
        track_path: AnimationTrackPath,
    },
    RebindTrack {
        from_track_path: AnimationTrackPath,
        to_track_path: AnimationTrackPath,
    },
    ScrubTimeline {
        frame: u32,
    },
    SetTimelineRange {
        start_frame: u32,
        end_frame: u32,
    },
    SelectTimelineSpan {
        track_path: AnimationTrackPath,
        start_frame: u32,
        end_frame: u32,
    },
    SetPlayback {
        playing: bool,
        looping: bool,
        speed: f32,
    },
    AddGraphNode {
        graph_locator: String,
        node_id: String,
        node_kind: String,
    },
    RemoveGraphNode {
        graph_locator: String,
        node_id: String,
    },
    ConnectGraphNodes {
        graph_locator: String,
        from_node_id: String,
        to_node_id: String,
    },
    DisconnectGraphNodes {
        graph_locator: String,
        from_node_id: String,
        to_node_id: String,
    },
    SetGraphParameter {
        graph_locator: String,
        parameter_name: String,
        value_literal: String,
    },
    CreateState {
        state_machine_locator: String,
        state_name: String,
        graph_locator: String,
    },
    RemoveState {
        state_machine_locator: String,
        state_name: String,
    },
    SetEntryState {
        state_machine_locator: String,
        state_name: String,
    },
    CreateTransition {
        state_machine_locator: String,
        from_state: String,
        to_state: String,
        duration_frames: u32,
    },
    RemoveTransition {
        state_machine_locator: String,
        from_state: String,
        to_state: String,
    },
    SetTransitionCondition {
        state_machine_locator: String,
        from_state: String,
        to_state: String,
        parameter_name: String,
        operator: String,
        value_literal: String,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 视口交互及设置的语义输入；执行层转换为 ViewportCommand，并根据反馈决定渲染与界面刷新。
pub enum EditorViewportEvent {
    /// A viewport command bound to one committed Scene leaf.
    ///
    /// The ordinary variants remain the wire-compatible event form used by
    /// keyboard and legacy command sources.  Retained toolbar input wraps the
    /// command here only after resolving its surface to a live view instance;
    /// execution must reject a retired target instead of falling back to the
    /// active viewport session.
    ForView {
        view_id: crate::core::editor_event::ViewInstanceId,
        event: Box<EditorViewportEvent>,
    },
    PointerMoved {
        x: f32,
        y: f32,
    },
    LeftPressed {
        x: f32,
        y: f32,
        selection_mutation: SelectionMutation,
    },
    LeftReleased,
    CancelInteraction,
    RightPressed {
        x: f32,
        y: f32,
    },
    RightReleased,
    MiddlePressed {
        x: f32,
        y: f32,
    },
    MiddleReleased,
    Scrolled {
        delta: f32,
    },
    Resized {
        width: u32,
        height: u32,
    },
    ActivateSceneMode {
        mode: SceneModeActivation,
    },
    SetTransformSpace {
        space: TransformSpace,
    },
    SetPivotMode {
        mode: PivotMode,
    },
    SetProjectionMode {
        mode: ProjectionMode,
    },
    AlignView {
        orientation: ViewOrientation,
    },
    SetDisplayMode {
        mode: DisplayMode,
    },
    SetGridMode {
        mode: GridMode,
    },
    SetTranslateSnap {
        step: f32,
    },
    SetRotateSnapDegrees {
        step: f32,
    },
    SetScaleSnap {
        step: f32,
    },
    SetPreviewLighting {
        enabled: bool,
    },
    SetPreviewSkybox {
        enabled: bool,
    },
    SetGizmosEnabled {
        enabled: bool,
    },
    ToggleOverlayProvider {
        provider_id: String,
    },
    FrameSelection,
}

impl EditorViewportEvent {
    // 标记只需视口控件投影更新的设置类输入，供执行层避免把每次设置都当成整个宿主展示重建。
    pub(crate) fn changes_chrome_projection(&self) -> bool {
        if let Self::ForView { event, .. } = self {
            return event.changes_chrome_projection();
        }
        matches!(
            self,
            Self::ActivateSceneMode { .. }
                | Self::SetTransformSpace { .. }
                | Self::SetPivotMode { .. }
                | Self::SetProjectionMode { .. }
                | Self::AlignView { .. }
                | Self::SetDisplayMode { .. }
                | Self::SetGridMode { .. }
                | Self::SetTranslateSnap { .. }
                | Self::SetRotateSnapDegrees { .. }
                | Self::SetScaleSnap { .. }
                | Self::SetPreviewLighting { .. }
                | Self::SetPreviewSkybox { .. }
                | Self::SetGizmosEnabled { .. }
                | Self::ToggleOverlayProvider { .. }
        )
    }
}

#[cfg(test)]
#[path = "tests/types_target_serialization_tests.rs"]
mod target_serialization_tests;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 短时 UI 交互意图；是否入保留日志由事件服务的记录策略决定，不能按变体名称推断持久性。
pub enum EditorEventTransient {
    HoverNode { node_path: String, hovered: bool },
    FocusNode { node_path: String },
    PressNode { node_path: String, pressed: bool },
    SetDrawerResizing { drawer_id: String, resizing: bool },
    BeginViewDrag { instance_id: String },
    EndViewDrag,
    OpenCommandPalette,
    OpenSettingsWindow,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 所有入口共享的语义事件联合类型；绑定、回放与无界面调用先汇合到此类型，再由宿主执行。
pub enum EditorEvent {
    WorkbenchMenu(MenuAction),
    Layout(LayoutCommand),
    Selection(SelectionHostEvent),
    Hierarchy(EditorHierarchyEvent),
    Asset(EditorAssetEvent),
    Draft(EditorDraftEvent),
    Animation(EditorAnimationEvent),
    Inspector(EditorInspectorEvent),
    Viewport(EditorViewportEvent),
    Operation(EditorOperationEvent),
    Transient(EditorEventTransient),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 语义事件及其来源；调用方应选择真实入口来源，以便监听器筛选和日志溯源。
pub struct EditorEventEnvelope {
    pub source: EditorEventSource,
    pub event: EditorEvent,
}

impl EditorEventEnvelope {
    pub fn new(source: EditorEventSource, event: EditorEvent) -> Self {
        Self { source, event }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 执行层交给宿主刷新的请求集合；效果表示需要采取的后续动作，不代表持久化或渲染已经完成。
pub enum EditorEventEffect {
    PresentationChanged,
    LayoutChanged,
    RenderChanged,
    ReflectionChanged,
    PresentWelcomeRequested,
    ProjectOpenRequested,
    ProjectSaveRequested,
    DocumentSaveAllRequested,
    ProjectCloseRequested,
    AssetDetailsRefreshRequested,
    AssetPreviewRefreshRequested,
    ImportModelRequested,
    AssetRelocationRequested {
        asset_uuid: String,
        target_locator: String,
    },
    AssetDeletionRequested {
        asset_uuid: String,
    },
    CommandPaletteOpenRequested,
    SettingsWindowOpenRequested,
    OpenScenePickerRequested,
    CreateScenePickerRequested,
    /// Explicitly opened view identity; the UI host decides whether its current
    /// authored drawer is reachable without changing layout on mere resize.
    OpenedViewVisibilityRequested {
        instance_id: ViewInstanceId,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
/// 派发可序列化结果；错误文本被回放用作失败预期，因此变更文案会影响已有失败日志的回放比较。
pub struct EditorEventResult {
    pub value: Option<Value>,
    pub error: Option<String>,
}

impl EditorEventResult {
    pub fn success(value: Value) -> Self {
        Self {
            value: Some(value),
            error: None,
        }
    }

    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            value: None,
            error: Some(error.into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 记录对撤销责任的声明；实际交易与撤销由编辑事务引擎管理，事件日志本身不构成逆向操作。
pub enum EditorEventUndoPolicy {
    NonUndoable,
    DelegatedToTransactionEngine,
    FutureInverseEvent,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 单次派发的诊断记录，串联来源、语义事件、操作元数据、效果与结果。
/// 编号和修订在执行前分配；失败记录的修订范围不证明状态提交，回放也只按语义重新执行。
pub struct EditorEventRecord {
    pub event_id: EditorEventId,
    pub sequence: EditorEventSequence,
    pub source: EditorEventSource,
    pub event: EditorEvent,
    /// Stable UI event path only when normal binding dispatch originated this record.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_arguments: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_group: Option<String>,
    /// Transaction committed by an authoring event, when that event creates one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<u64>,
    /// Generation captured by a successful project save after persistence completes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub save_generation: Option<u64>,
    pub effects: Vec<EditorEventEffect>,
    pub undo_policy: EditorEventUndoPolicy,
    pub before_revision: u64,
    pub after_revision: u64,
    pub result: EditorEventResult,
}
