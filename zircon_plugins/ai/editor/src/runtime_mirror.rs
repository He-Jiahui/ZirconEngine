use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use zircon_editor::core::runtime_event_consumer::{
    EditorRuntimeEventConsumerManifest, EditorRuntimeEventConsumerRegistration,
    EditorRuntimeEventConsumerState,
};
use zircon_runtime::core::framework::ai::{
    AiBehaviorDebugFrame, AiBehaviorDebugSnapshot, BtNodeResultEvent,
};
use zircon_runtime::core::framework::scene::WorldHandle;

use crate::capability::AI_DEBUG_CAPABILITY;
use crate::extension_ids::{
    AI_BEHAVIOR_DEBUG_CONSUMER_ID, AI_BT_NODE_RESULT_CONSUMER_ID,
    AI_BT_NODE_RESULT_SNAPSHOT_PRUNE_CONSUMER_ID,
};

pub use zircon_plugin_ai_runtime::{
    AI_BEHAVIOR_DEBUG_SNAPSHOT_EVENT_ID, AI_BEHAVIOR_DEBUG_SNAPSHOT_PAYLOAD_SCHEMA,
    BT_NODE_RESULT_EVENT_ID, BT_NODE_RESULT_PAYLOAD_SCHEMA,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiPieMirrorApply {
    Applied,
    WrongSession,
    Stale,
    InvalidSnapshotWorld,
}

#[derive(Clone, Copy, Debug, thiserror::Error, PartialEq, Eq)]
pub enum AiPieMirrorError {
    #[error("AI PIE mirror received a delivery for the wrong play session")]
    WrongSession,
    #[error("AI PIE mirror received a stale delivery sequence")]
    Stale,
    #[error("AI PIE mirror received a debug snapshot containing a frame from another world")]
    InvalidSnapshotWorld,
}

#[derive(Clone, Debug, Default)]
pub struct AiPieMirror {
    play_session_id: Option<u64>,
    sequence: Option<u64>,
    agents: BTreeMap<(u64, u64), AiBehaviorDebugFrame>,
}

impl AiPieMirror {
    pub fn begin_session(&mut self, play_session_id: u64) {
        self.play_session_id = Some(play_session_id);
        self.sequence = None;
        self.agents.clear();
    }

    pub fn end_session(&mut self, play_session_id: u64) -> bool {
        if self.play_session_id != Some(play_session_id) {
            return false;
        }
        *self = Self::default();
        true
    }

    pub fn apply_debug_snapshot(
        &mut self,
        play_session_id: u64,
        sequence: u64,
        snapshot: AiBehaviorDebugSnapshot,
    ) -> AiPieMirrorApply {
        if self.play_session_id != Some(play_session_id) {
            return AiPieMirrorApply::WrongSession;
        }
        if snapshot
            .frames
            .iter()
            .any(|frame| frame.report.world != snapshot.world)
        {
            return AiPieMirrorApply::InvalidSnapshotWorld;
        }
        if self
            .sequence
            .is_some_and(|previous_sequence| sequence <= previous_sequence)
        {
            return AiPieMirrorApply::Stale;
        }
        self.sequence = Some(sequence);
        let world = snapshot.world.get();
        self.agents
            .retain(|(frame_world, _), _| *frame_world != world);
        for frame in snapshot.frames {
            self.agents.insert((world, frame.report.entity), frame);
        }
        AiPieMirrorApply::Applied
    }

    pub fn play_session_id(&self) -> Option<u64> {
        self.play_session_id
    }

    pub fn sequence(&self) -> Option<u64> {
        self.sequence
    }

    pub fn agent(&self, entity: u64) -> Option<&AiBehaviorDebugFrame> {
        let mut agents = self
            .agents
            .values()
            .filter(|frame| frame.report.entity == entity);
        let agent = agents.next()?;
        agents.next().is_none().then_some(agent)
    }

    pub fn agent_in_world(
        &self,
        world: &WorldHandle,
        entity: u64,
    ) -> Option<&AiBehaviorDebugFrame> {
        self.agents.get(&(world.get(), entity))
    }

    pub fn agents(&self) -> impl ExactSizeIterator<Item = &AiBehaviorDebugFrame> {
        self.agents.values()
    }

    pub fn agents_in_world(
        &self,
        world: &WorldHandle,
    ) -> impl Iterator<Item = &AiBehaviorDebugFrame> {
        let world = world.get();
        self.agents
            .iter()
            .filter(move |((frame_world, _), _)| *frame_world == world)
            .map(|(_, frame)| frame)
    }
}

impl EditorRuntimeEventConsumerState for AiPieMirror {
    type Payload = AiBehaviorDebugSnapshot;
    type Error = AiPieMirrorError;

    fn begin_session(&mut self, play_session_id: u64) {
        AiPieMirror::begin_session(self, play_session_id);
    }

    fn consume(
        &mut self,
        play_session_id: u64,
        sequence: u64,
        payload: Self::Payload,
    ) -> Result<(), Self::Error> {
        match self.apply_debug_snapshot(play_session_id, sequence, payload) {
            AiPieMirrorApply::Applied => Ok(()),
            AiPieMirrorApply::WrongSession => Err(AiPieMirrorError::WrongSession),
            AiPieMirrorApply::Stale => Err(AiPieMirrorError::Stale),
            AiPieMirrorApply::InvalidSnapshotWorld => Err(AiPieMirrorError::InvalidSnapshotWorld),
        }
    }

    fn end_session(&mut self, play_session_id: u64) {
        let _ = AiPieMirror::end_session(self, play_session_id);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiBtNodeResultMirrorApply {
    Applied,
    WrongSession,
    Stale,
    InvalidSnapshotWorld,
}

#[derive(Clone, Copy, Debug, thiserror::Error, PartialEq, Eq)]
pub enum AiBtNodeResultMirrorError {
    #[error("AI behavior-tree node mirror received a delivery for the wrong play session")]
    WrongSession,
    #[error("AI behavior-tree node mirror received a stale delivery sequence")]
    Stale,
    #[error(
        "AI behavior-tree node mirror received a debug snapshot containing a frame from another world"
    )]
    InvalidSnapshotWorld,
}

#[derive(Clone, Debug, Default)]
pub struct AiBtNodeResultMirror {
    play_session_id: Option<u64>,
    node_result_sequence: Option<u64>,
    snapshot_sequence: Option<u64>,
    results: BTreeMap<(u64, u64, String), BtNodeResultEvent>,
}

impl AiBtNodeResultMirror {
    pub fn begin_session(&mut self, play_session_id: u64) {
        self.play_session_id = Some(play_session_id);
        self.node_result_sequence = None;
        self.snapshot_sequence = None;
        self.results.clear();
    }

    pub fn end_session(&mut self, play_session_id: u64) -> bool {
        if self.play_session_id != Some(play_session_id) {
            return false;
        }
        *self = Self::default();
        true
    }

    pub fn apply_node_result(
        &mut self,
        play_session_id: u64,
        sequence: u64,
        event: BtNodeResultEvent,
    ) -> AiBtNodeResultMirrorApply {
        if self.play_session_id != Some(play_session_id) {
            return AiBtNodeResultMirrorApply::WrongSession;
        }
        if self
            .node_result_sequence
            .is_some_and(|previous_sequence| sequence <= previous_sequence)
        {
            return AiBtNodeResultMirrorApply::Stale;
        }
        self.node_result_sequence = Some(sequence);
        let key = (event.world.get(), event.entity, event.node_id.clone());
        self.results.insert(key, event);
        AiBtNodeResultMirrorApply::Applied
    }

    pub fn node_result(
        &self,
        world: &WorldHandle,
        entity: u64,
        node_id: &str,
    ) -> Option<&BtNodeResultEvent> {
        self.results.get(&(world.get(), entity, node_id.to_owned()))
    }

    pub fn apply_debug_snapshot(
        &mut self,
        play_session_id: u64,
        sequence: u64,
        snapshot: AiBehaviorDebugSnapshot,
    ) -> AiBtNodeResultMirrorApply {
        if self.play_session_id != Some(play_session_id) {
            return AiBtNodeResultMirrorApply::WrongSession;
        }
        if snapshot
            .frames
            .iter()
            .any(|frame| frame.report.world != snapshot.world)
        {
            return AiBtNodeResultMirrorApply::InvalidSnapshotWorld;
        }
        if self
            .snapshot_sequence
            .is_some_and(|previous_sequence| sequence <= previous_sequence)
        {
            return AiBtNodeResultMirrorApply::Stale;
        }
        self.snapshot_sequence = Some(sequence);
        let world = snapshot.world.get();
        let active_nodes = snapshot
            .frames
            .into_iter()
            .filter_map(|frame| {
                frame
                    .report
                    .active_node
                    .map(|node_id| (world, frame.report.entity, node_id))
            })
            .collect::<std::collections::BTreeSet<_>>();
        self.results.retain(|(result_world, entity, node_id), _| {
            *result_world != world
                || active_nodes.contains(&(*result_world, *entity, node_id.clone()))
        });
        AiBtNodeResultMirrorApply::Applied
    }
}

impl EditorRuntimeEventConsumerState for AiBtNodeResultMirror {
    type Payload = BtNodeResultEvent;
    type Error = AiBtNodeResultMirrorError;

    fn begin_session(&mut self, play_session_id: u64) {
        AiBtNodeResultMirror::begin_session(self, play_session_id);
    }

    fn consume(
        &mut self,
        play_session_id: u64,
        sequence: u64,
        payload: Self::Payload,
    ) -> Result<(), Self::Error> {
        match self.apply_node_result(play_session_id, sequence, payload) {
            AiBtNodeResultMirrorApply::Applied => Ok(()),
            AiBtNodeResultMirrorApply::WrongSession => Err(AiBtNodeResultMirrorError::WrongSession),
            AiBtNodeResultMirrorApply::Stale => Err(AiBtNodeResultMirrorError::Stale),
            AiBtNodeResultMirrorApply::InvalidSnapshotWorld => {
                Err(AiBtNodeResultMirrorError::InvalidSnapshotWorld)
            }
        }
    }

    fn end_session(&mut self, play_session_id: u64) {
        let _ = AiBtNodeResultMirror::end_session(self, play_session_id);
    }
}

struct AiBtNodeResultSnapshotPruner {
    node_results: Arc<Mutex<AiBtNodeResultMirror>>,
}

impl AiBtNodeResultSnapshotPruner {
    fn new(node_results: Arc<Mutex<AiBtNodeResultMirror>>) -> Self {
        Self { node_results }
    }
}

impl EditorRuntimeEventConsumerState for AiBtNodeResultSnapshotPruner {
    type Payload = AiBehaviorDebugSnapshot;
    type Error = AiBtNodeResultMirrorError;

    fn begin_session(&mut self, _play_session_id: u64) {}

    fn consume(
        &mut self,
        play_session_id: u64,
        sequence: u64,
        payload: Self::Payload,
    ) -> Result<(), Self::Error> {
        match self
            .node_results
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .apply_debug_snapshot(play_session_id, sequence, payload)
        {
            AiBtNodeResultMirrorApply::Applied => Ok(()),
            AiBtNodeResultMirrorApply::WrongSession => Err(AiBtNodeResultMirrorError::WrongSession),
            AiBtNodeResultMirrorApply::Stale => Err(AiBtNodeResultMirrorError::Stale),
            AiBtNodeResultMirrorApply::InvalidSnapshotWorld => {
                Err(AiBtNodeResultMirrorError::InvalidSnapshotWorld)
            }
        }
    }

    fn end_session(&mut self, _play_session_id: u64) {}
}

pub fn ai_runtime_event_consumers() -> Vec<EditorRuntimeEventConsumerRegistration> {
    let debug_manifest = EditorRuntimeEventConsumerManifest::new(
        AI_BEHAVIOR_DEBUG_CONSUMER_ID,
        AI_BEHAVIOR_DEBUG_SNAPSHOT_EVENT_ID,
        AI_BEHAVIOR_DEBUG_SNAPSHOT_PAYLOAD_SCHEMA,
    )
    .with_required_capability(AI_DEBUG_CAPABILITY);
    let node_result_manifest = EditorRuntimeEventConsumerManifest::new(
        AI_BT_NODE_RESULT_CONSUMER_ID,
        BT_NODE_RESULT_EVENT_ID,
        BT_NODE_RESULT_PAYLOAD_SCHEMA,
    )
    .with_required_capability(AI_DEBUG_CAPABILITY);
    let node_result_snapshot_prune_manifest = EditorRuntimeEventConsumerManifest::new(
        AI_BT_NODE_RESULT_SNAPSHOT_PRUNE_CONSUMER_ID,
        AI_BEHAVIOR_DEBUG_SNAPSHOT_EVENT_ID,
        AI_BEHAVIOR_DEBUG_SNAPSHOT_PAYLOAD_SCHEMA,
    )
    .with_required_capability(AI_DEBUG_CAPABILITY);
    let node_results = Arc::new(Mutex::new(AiBtNodeResultMirror::default()));
    vec![
        EditorRuntimeEventConsumerRegistration::typed(
            debug_manifest,
            Arc::new(Mutex::new(AiPieMirror::default())),
        ),
        EditorRuntimeEventConsumerRegistration::typed(node_result_manifest, node_results.clone()),
        EditorRuntimeEventConsumerRegistration::typed(
            node_result_snapshot_prune_manifest,
            Arc::new(Mutex::new(AiBtNodeResultSnapshotPruner::new(node_results))),
        ),
    ]
}
