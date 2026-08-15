use std::sync::{Arc, Mutex};

use zircon_editor::core::runtime_event_consumer::{
    EditorRuntimeEventConsumerManifest, EditorRuntimeEventConsumerRegistration,
    EditorRuntimeEventConsumerState,
};
use zircon_runtime::core::framework::navigation::{
    NavAgentTickReport, NavigationAgentDebugState, NavigationGizmoSnapshot,
};

use crate::NAVIGATION_GIZMOS_CAPABILITY;

pub use zircon_plugin_navigation_runtime::{
    NavigationOverlayFrame, NAVIGATION_OVERLAY_FRAME_EVENT_ID,
    NAVIGATION_OVERLAY_FRAME_PAYLOAD_SCHEMA,
};

pub const NAVIGATION_OVERLAY_CONSUMER_ID: &str = "navigation.editor.overlay_frame";

#[derive(Clone, Debug, PartialEq)]
pub struct NavigationPieFrame {
    pub play_session_id: u64,
    pub sequence: u64,
    pub owner_generation: u64,
    pub nav_mesh: NavigationGizmoSnapshot,
    pub tick_report: NavAgentTickReport,
}

impl NavigationPieFrame {
    pub fn new(play_session_id: u64, sequence: u64, overlay_frame: NavigationOverlayFrame) -> Self {
        Self {
            play_session_id,
            sequence,
            owner_generation: overlay_frame.owner_generation,
            nav_mesh: overlay_frame.nav_mesh,
            tick_report: overlay_frame.tick_report,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationPieMirrorApply {
    Applied,
    WrongSession,
    Stale,
    StaleOwnerGeneration,
}

#[derive(Clone, Copy, Debug, thiserror::Error, PartialEq, Eq)]
pub enum NavigationPieMirrorError {
    #[error("navigation PIE mirror received a delivery for the wrong play session")]
    WrongSession,
    #[error("navigation PIE mirror received a stale delivery sequence")]
    Stale,
    #[error("navigation PIE mirror received a stale owner generation")]
    StaleOwnerGeneration,
}

#[derive(Clone, Debug, Default)]
pub struct NavigationPieMirror {
    play_session_id: Option<u64>,
    frame: Option<NavigationPieFrame>,
}

impl NavigationPieMirror {
    pub fn begin_session(&mut self, play_session_id: u64) {
        self.play_session_id = Some(play_session_id);
        self.frame = None;
    }

    pub fn end_session(&mut self, play_session_id: u64) -> bool {
        if self.play_session_id != Some(play_session_id) {
            return false;
        }
        *self = Self::default();
        true
    }

    pub fn apply_frame(&mut self, frame: NavigationPieFrame) -> NavigationPieMirrorApply {
        if self.play_session_id != Some(frame.play_session_id) {
            return NavigationPieMirrorApply::WrongSession;
        }
        if self
            .frame
            .as_ref()
            .is_some_and(|current| frame.sequence <= current.sequence)
        {
            return NavigationPieMirrorApply::Stale;
        }
        if self
            .frame
            .as_ref()
            .is_some_and(|current| frame.owner_generation < current.owner_generation)
        {
            return NavigationPieMirrorApply::StaleOwnerGeneration;
        }
        self.frame = Some(frame);
        NavigationPieMirrorApply::Applied
    }

    pub fn apply_overlay_frame(
        &mut self,
        play_session_id: u64,
        sequence: u64,
        frame: NavigationOverlayFrame,
    ) -> NavigationPieMirrorApply {
        self.apply_frame(NavigationPieFrame::new(play_session_id, sequence, frame))
    }

    pub fn play_session_id(&self) -> Option<u64> {
        self.play_session_id
    }

    pub fn sequence(&self) -> Option<u64> {
        self.frame.as_ref().map(|frame| frame.sequence)
    }

    pub fn owner_generation(&self) -> Option<u64> {
        self.frame.as_ref().map(|frame| frame.owner_generation)
    }

    pub fn frame(&self) -> Option<&NavigationPieFrame> {
        self.frame.as_ref()
    }

    pub fn nav_mesh(&self) -> Option<&NavigationGizmoSnapshot> {
        self.frame.as_ref().map(|frame| &frame.nav_mesh)
    }

    pub fn tick_report(&self) -> Option<&NavAgentTickReport> {
        self.frame.as_ref().map(|frame| &frame.tick_report)
    }

    pub fn agent(&self, entity: u64) -> Option<&NavigationAgentDebugState> {
        self.tick_report()?
            .debug_agents
            .iter()
            .find(|agent| agent.entity == entity)
    }

    pub fn agents(&self) -> impl ExactSizeIterator<Item = &NavigationAgentDebugState> {
        self.tick_report()
            .map(|report| report.debug_agents.as_slice())
            .unwrap_or_default()
            .iter()
    }
}

impl EditorRuntimeEventConsumerState for NavigationPieMirror {
    type Payload = NavigationOverlayFrame;
    type Error = NavigationPieMirrorError;

    fn begin_session(&mut self, play_session_id: u64) {
        NavigationPieMirror::begin_session(self, play_session_id);
    }

    fn consume(
        &mut self,
        play_session_id: u64,
        sequence: u64,
        payload: Self::Payload,
    ) -> Result<(), Self::Error> {
        match self.apply_overlay_frame(play_session_id, sequence, payload) {
            NavigationPieMirrorApply::Applied => Ok(()),
            NavigationPieMirrorApply::WrongSession => Err(NavigationPieMirrorError::WrongSession),
            NavigationPieMirrorApply::Stale => Err(NavigationPieMirrorError::Stale),
            NavigationPieMirrorApply::StaleOwnerGeneration => {
                Err(NavigationPieMirrorError::StaleOwnerGeneration)
            }
        }
    }

    fn end_session(&mut self, play_session_id: u64) {
        let _ = NavigationPieMirror::end_session(self, play_session_id);
    }
}

pub(crate) fn navigation_runtime_event_consumers_with_mirror(
    mirror: Arc<Mutex<NavigationPieMirror>>,
) -> Vec<EditorRuntimeEventConsumerRegistration> {
    let manifest = EditorRuntimeEventConsumerManifest::new(
        NAVIGATION_OVERLAY_CONSUMER_ID,
        NAVIGATION_OVERLAY_FRAME_EVENT_ID,
        NAVIGATION_OVERLAY_FRAME_PAYLOAD_SCHEMA,
    )
    .with_required_capability(NAVIGATION_GIZMOS_CAPABILITY);
    vec![EditorRuntimeEventConsumerRegistration::typed(
        manifest, mirror,
    )]
}
