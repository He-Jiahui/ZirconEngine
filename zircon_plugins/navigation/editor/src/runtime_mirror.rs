use std::sync::{Arc, Mutex};
use zircon_runtime::core::framework::navigation::{NavAgentTickReport, NavigationAgentDebugState};

use zircon_editor::core::runtime_event_consumer::{
    EditorRuntimeEventConsumerManifest, EditorRuntimeEventConsumerRegistration,
    EditorRuntimeEventConsumerState,
};

use crate::NAVIGATION_GIZMOS_CAPABILITY;

pub const NAVIGATION_TICK_CONSUMER_ID: &str = "navigation.editor.agent_tick";
pub const NAVIGATION_TICK_EVENT_ID: &str = "navigation.events.agent_tick_completed";
pub const NAVIGATION_TICK_PAYLOAD_SCHEMA: &str = "navigation.events.nav_agent_tick_report.v1";

#[derive(Clone, Debug, PartialEq)]
pub struct NavigationPieFrame {
    pub play_session_id: u64,
    pub sequence: u64,
    pub tick_report: NavAgentTickReport,
}

impl NavigationPieFrame {
    pub fn new(play_session_id: u64, sequence: u64, tick_report: NavAgentTickReport) -> Self {
        Self {
            play_session_id,
            sequence,
            tick_report,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationPieMirrorApply {
    Applied,
    WrongSession,
    Stale,
}

#[derive(Clone, Copy, Debug, thiserror::Error, PartialEq, Eq)]
pub enum NavigationPieMirrorError {
    #[error("navigation PIE mirror received a delivery for the wrong play session")]
    WrongSession,
    #[error("navigation PIE mirror received a stale delivery sequence")]
    Stale,
}

#[derive(Clone, Debug, Default)]
pub struct NavigationPieMirror {
    play_session_id: Option<u64>,
    sequence: Option<u64>,
    tick_report: Option<NavAgentTickReport>,
}

impl NavigationPieMirror {
    pub fn begin_session(&mut self, play_session_id: u64) {
        self.play_session_id = Some(play_session_id);
        self.sequence = None;
        self.tick_report = None;
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
            .sequence
            .is_some_and(|sequence| frame.sequence <= sequence)
        {
            return NavigationPieMirrorApply::Stale;
        }
        self.sequence = Some(frame.sequence);
        self.tick_report = Some(frame.tick_report);
        NavigationPieMirrorApply::Applied
    }

    /// Consumes the shared `NavAgentTickReport` event registered by the runtime plugin.
    pub fn apply_tick_report(
        &mut self,
        play_session_id: u64,
        sequence: u64,
        report: NavAgentTickReport,
    ) -> NavigationPieMirrorApply {
        self.apply_frame(NavigationPieFrame::new(play_session_id, sequence, report))
    }

    pub fn play_session_id(&self) -> Option<u64> {
        self.play_session_id
    }

    pub fn sequence(&self) -> Option<u64> {
        self.sequence
    }

    pub fn tick_report(&self) -> Option<&NavAgentTickReport> {
        self.tick_report.as_ref()
    }

    pub fn agent(&self, entity: u64) -> Option<&NavigationAgentDebugState> {
        self.tick_report
            .as_ref()?
            .debug_agents
            .iter()
            .find(|agent| agent.entity == entity)
    }

    pub fn agents(&self) -> impl ExactSizeIterator<Item = &NavigationAgentDebugState> {
        self.tick_report
            .as_ref()
            .map(|report| report.debug_agents.as_slice())
            .unwrap_or_default()
            .iter()
    }
}

impl EditorRuntimeEventConsumerState for NavigationPieMirror {
    type Payload = NavAgentTickReport;
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
        match self.apply_tick_report(play_session_id, sequence, payload) {
            NavigationPieMirrorApply::Applied => Ok(()),
            NavigationPieMirrorApply::WrongSession => Err(NavigationPieMirrorError::WrongSession),
            NavigationPieMirrorApply::Stale => Err(NavigationPieMirrorError::Stale),
        }
    }

    fn end_session(&mut self, play_session_id: u64) {
        let _ = NavigationPieMirror::end_session(self, play_session_id);
    }
}

pub fn navigation_runtime_event_consumers() -> Vec<EditorRuntimeEventConsumerRegistration> {
    let manifest = EditorRuntimeEventConsumerManifest::new(
        NAVIGATION_TICK_CONSUMER_ID,
        NAVIGATION_TICK_EVENT_ID,
        NAVIGATION_TICK_PAYLOAD_SCHEMA,
    )
    .with_required_capability(NAVIGATION_GIZMOS_CAPABILITY);
    vec![EditorRuntimeEventConsumerRegistration::typed(
        manifest,
        Arc::new(Mutex::new(NavigationPieMirror::default())),
    )]
}
