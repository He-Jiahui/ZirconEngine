mod bus;
mod ids;
mod inbox;
mod message;
mod refresh_report;
mod retention;
mod shared;
mod subscriber;
mod topic;
mod topics;
mod view_dirty_set;

pub(crate) use bus::EditorMessageBus;
pub use bus::{
    EditorMessageBusError, EditorMessageDispatchError, EditorMessageDispatchReport,
    EditorRequestHandler,
};
pub use ids::{DocumentId, PlayStateKind, SceneModeId, SelectionDomain};
pub use inbox::{EditorMessageInboxLimits, EditorMessageInboxStats};
pub use message::{
    DocumentMessage, EditorMessage, EditorMessageDelivery, EditorMessagePayload,
    EditorMessageProtocol, EditorMessageRequest, EditorMessageResponse, EditorViewDirtyMark,
    FocusMessage, ModeMessage, SceneInspectionFieldsDelta, SceneInspectionMessage,
    SceneInspectionPropertyPath, ToolMessage, TransactionMessage,
};
pub use refresh_report::EditorViewRefreshReport;
pub use shared::SharedEditorMessageBus;
pub use subscriber::EditorSubscriberId;
pub use topic::{EditorTopic, EditorTopicError};
pub use topics::{
    TOPIC_DOCUMENT, TOPIC_FOCUS, TOPIC_I18N, TOPIC_JOB, TOPIC_LOG, TOPIC_MODE,
    TOPIC_SCENE_INSPECTION, TOPIC_TOOL, TOPIC_TRANSACTION,
};
pub use view_dirty_set::{EditorViewInvalidationMask, ViewDirtySet};
