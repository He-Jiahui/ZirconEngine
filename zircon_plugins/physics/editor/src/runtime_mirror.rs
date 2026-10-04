use std::fmt;
use std::sync::{Arc, Mutex};

use zircon_editor::core::runtime_event_consumer::{
    EditorRuntimeEventConsumerManifest, EditorRuntimeEventConsumerRegistration,
    EditorRuntimeEventConsumerState,
};

use crate::PHYSICS_AUTHORING_CAPABILITY;

pub use zircon_plugin_physics_runtime::{
    PhysicsOverlayFrame, PHYSICS_OVERLAY_FRAME_EVENT_ID, PHYSICS_OVERLAY_FRAME_PAYLOAD_SCHEMA,
};

pub const PHYSICS_OVERLAY_CONSUMER_ID: &str = "physics.editor.overlay_frame";

#[derive(Clone, Debug, PartialEq)]
pub struct PhysicsPieFrame {
    pub play_session_id: u64,
    pub sequence: u64,
    pub overlay_frame: PhysicsOverlayFrame,
}

impl PhysicsPieFrame {
    pub fn new(play_session_id: u64, sequence: u64, overlay_frame: PhysicsOverlayFrame) -> Self {
        Self {
            play_session_id,
            sequence,
            overlay_frame,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhysicsPieMirrorApply {
    Applied,
    WrongSession,
    Stale,
    StaleOwnerGeneration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhysicsPieMirrorError {
    WrongSession,
    Stale,
    StaleOwnerGeneration,
}

impl fmt::Display for PhysicsPieMirrorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::WrongSession => {
                "physics PIE mirror received a delivery for the wrong play session"
            }
            Self::Stale => "physics PIE mirror received a stale delivery sequence",
            Self::StaleOwnerGeneration => "physics PIE mirror received a stale owner generation",
        })
    }
}

impl std::error::Error for PhysicsPieMirrorError {}

#[derive(Clone, Debug, Default)]
pub struct PhysicsPieMirror {
    play_session_id: Option<u64>,
    frame: Option<PhysicsPieFrame>,
}

impl PhysicsPieMirror {
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

    pub fn apply_frame(&mut self, frame: PhysicsPieFrame) -> PhysicsPieMirrorApply {
        if self.play_session_id != Some(frame.play_session_id) {
            return PhysicsPieMirrorApply::WrongSession;
        }
        if self
            .frame
            .as_ref()
            .is_some_and(|current| frame.sequence <= current.sequence)
        {
            return PhysicsPieMirrorApply::Stale;
        }
        if self.frame.as_ref().is_some_and(|current| {
            frame.overlay_frame.owner_generation < current.overlay_frame.owner_generation
        }) {
            return PhysicsPieMirrorApply::StaleOwnerGeneration;
        }
        self.frame = Some(frame);
        PhysicsPieMirrorApply::Applied
    }

    pub fn apply_overlay_frame(
        &mut self,
        play_session_id: u64,
        sequence: u64,
        frame: PhysicsOverlayFrame,
    ) -> PhysicsPieMirrorApply {
        self.apply_frame(PhysicsPieFrame::new(play_session_id, sequence, frame))
    }

    pub fn play_session_id(&self) -> Option<u64> {
        self.play_session_id
    }

    pub fn sequence(&self) -> Option<u64> {
        self.frame.as_ref().map(|frame| frame.sequence)
    }

    pub fn owner_generation(&self) -> Option<u64> {
        self.frame
            .as_ref()
            .map(|frame| frame.overlay_frame.owner_generation)
    }

    pub fn frame(&self) -> Option<&PhysicsPieFrame> {
        self.frame.as_ref()
    }
}

impl EditorRuntimeEventConsumerState for PhysicsPieMirror {
    type Payload = PhysicsOverlayFrame;
    type Error = PhysicsPieMirrorError;

    fn begin_session(&mut self, play_session_id: u64) {
        PhysicsPieMirror::begin_session(self, play_session_id);
    }

    fn consume(
        &mut self,
        play_session_id: u64,
        sequence: u64,
        payload: Self::Payload,
    ) -> Result<(), Self::Error> {
        match self.apply_overlay_frame(play_session_id, sequence, payload) {
            PhysicsPieMirrorApply::Applied => Ok(()),
            PhysicsPieMirrorApply::WrongSession => Err(PhysicsPieMirrorError::WrongSession),
            PhysicsPieMirrorApply::Stale => Err(PhysicsPieMirrorError::Stale),
            PhysicsPieMirrorApply::StaleOwnerGeneration => {
                Err(PhysicsPieMirrorError::StaleOwnerGeneration)
            }
        }
    }

    fn end_session(&mut self, play_session_id: u64) {
        let _ = PhysicsPieMirror::end_session(self, play_session_id);
    }
}

pub(crate) fn physics_runtime_event_consumers_with_mirror(
    mirror: Arc<Mutex<PhysicsPieMirror>>,
) -> Vec<EditorRuntimeEventConsumerRegistration> {
    let manifest = EditorRuntimeEventConsumerManifest::new(
        PHYSICS_OVERLAY_CONSUMER_ID,
        PHYSICS_OVERLAY_FRAME_EVENT_ID,
        PHYSICS_OVERLAY_FRAME_PAYLOAD_SCHEMA,
    )
    .with_required_capability(PHYSICS_AUTHORING_CAPABILITY);
    vec![EditorRuntimeEventConsumerRegistration::typed(
        manifest, mirror,
    )]
}
