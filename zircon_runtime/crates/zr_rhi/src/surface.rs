use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use thiserror::Error;

use crate::{
    DeviceGeneration, DeviceId, RenderNativeSurfaceTarget, SubmissionLimits, SubmissionTicket,
    SwapchainDesc, TextureDesc, TextureHandle, TextureViewHandle,
};

static NEXT_SURFACE_NAMESPACE: AtomicU64 = AtomicU64::new(1);

/// Requested native target and the requested swapchain policy for one surface session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderSurfaceDescriptor {
    pub label: Option<String>,
    pub target: RenderNativeSurfaceTarget,
    pub swapchain: SwapchainDesc,
}

impl RenderSurfaceDescriptor {
    pub fn new(
        label: impl Into<String>,
        target: RenderNativeSurfaceTarget,
        swapchain: SwapchainDesc,
    ) -> Self {
        Self {
            label: Some(label.into()),
            target,
            swapchain,
        }
    }

    pub const fn is_renderable(&self) -> bool {
        self.swapchain.width != 0 && self.swapchain.height != 0
    }
}

/// Opaque identity for one device-generation-local native surface session.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SurfaceSession(SurfaceHandleIdentity);

impl SurfaceSession {
    pub const fn device_id(self) -> DeviceId {
        self.0.device_id
    }

    pub const fn generation(self) -> DeviceGeneration {
        self.0.generation
    }

    /// Diagnostic-only identity. It cannot be used to recover or forge a session.
    pub const fn diagnostic_id(self) -> u64 {
        self.0.diagnostic_id()
    }
}

impl fmt::Debug for SurfaceSession {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SurfaceSession")
            .field("diagnostic_id", &self.diagnostic_id())
            .finish()
    }
}

/// Opaque identity for one acquired surface frame lease.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SurfaceFrameId(SurfaceHandleIdentity);

impl SurfaceFrameId {
    pub const fn device_id(self) -> DeviceId {
        self.0.device_id
    }

    pub const fn generation(self) -> DeviceGeneration {
        self.0.generation
    }

    /// Diagnostic-only identity. It cannot be used to recover or forge a frame lease.
    pub const fn diagnostic_id(self) -> u64 {
        self.0.diagnostic_id()
    }
}

impl fmt::Debug for SurfaceFrameId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SurfaceFrameId")
            .field("diagnostic_id", &self.diagnostic_id())
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum SurfaceHandleKind {
    Session,
    Frame,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct SurfaceHandleIdentity {
    namespace: u64,
    device_id: DeviceId,
    generation: DeviceGeneration,
    kind: SurfaceHandleKind,
    value: u64,
}

impl SurfaceHandleIdentity {
    const fn diagnostic_id(self) -> u64 {
        let mut value = self.namespace ^ self.device_id.raw().rotate_left(9);
        value ^= self.generation.raw().rotate_left(21);
        value ^= (self.kind as u64) << 61;
        value ^= self.value.rotate_left(37);
        value ^ (value >> 29)
    }
}

#[derive(Debug, Default)]
struct RenderSurfaceHandleAllocatorState {
    next_session: u64,
    next_frame: u64,
    active_sessions: HashSet<u64>,
    active_frames: HashSet<u64>,
}

/// Validation failure returned before a surface service uses an opaque identity.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum RenderSurfaceHandleError {
    #[error("surface handle belongs to device `{actual:?}`, expected `{expected:?}")]
    WrongDevice {
        expected: DeviceId,
        actual: DeviceId,
    },
    #[error("surface handle belongs to device generation `{actual:?}`, expected `{expected:?}")]
    WrongGeneration {
        expected: DeviceGeneration,
        actual: DeviceGeneration,
    },
    #[error("surface handle diagnostic id `{diagnostic_id}` belongs to another device registry")]
    ForeignAllocator { diagnostic_id: u64 },
    #[error("surface handle diagnostic id `{diagnostic_id}` is stale or has been released")]
    StaleHandle { diagnostic_id: u64 },
}

/// Device-owned allocator for opaque surface sessions and frame leases.
///
/// The namespace prevents another backend owner from accepting an otherwise
/// matching device/generation/value tuple. Released identities never become
/// valid again, so resize, teardown, and terminalization fail closed.
#[derive(Clone, Debug)]
pub struct RenderSurfaceHandleAllocator {
    device_id: DeviceId,
    generation: DeviceGeneration,
    namespace: u64,
    state: Arc<Mutex<RenderSurfaceHandleAllocatorState>>,
}

impl RenderSurfaceHandleAllocator {
    pub fn new(device_id: DeviceId, generation: DeviceGeneration) -> Self {
        Self {
            device_id,
            generation,
            namespace: NEXT_SURFACE_NAMESPACE.fetch_add(1, Ordering::Relaxed),
            state: Arc::new(Mutex::new(RenderSurfaceHandleAllocatorState {
                next_session: 1,
                next_frame: 1,
                ..RenderSurfaceHandleAllocatorState::default()
            })),
        }
    }

    pub const fn device_id(&self) -> DeviceId {
        self.device_id
    }

    pub const fn generation(&self) -> DeviceGeneration {
        self.generation
    }

    pub fn allocate_session(&self) -> Result<SurfaceSession, RenderSurfaceHandleError> {
        let value = self.allocate(SurfaceHandleKind::Session)?;
        Ok(SurfaceSession(
            self.identity(SurfaceHandleKind::Session, value),
        ))
    }

    pub fn allocate_frame(&self) -> Result<SurfaceFrameId, RenderSurfaceHandleError> {
        let value = self.allocate(SurfaceHandleKind::Frame)?;
        Ok(SurfaceFrameId(
            self.identity(SurfaceHandleKind::Frame, value),
        ))
    }

    pub fn validate_session(
        &self,
        session: SurfaceSession,
    ) -> Result<(), RenderSurfaceHandleError> {
        self.validate(session.0, SurfaceHandleKind::Session)
    }

    pub fn validate_frame(&self, frame: SurfaceFrameId) -> Result<(), RenderSurfaceHandleError> {
        self.validate(frame.0, SurfaceHandleKind::Frame)
    }

    pub fn release_session(&self, session: SurfaceSession) -> Result<(), RenderSurfaceHandleError> {
        self.release(session.0, SurfaceHandleKind::Session)
    }

    pub fn release_frame(&self, frame: SurfaceFrameId) -> Result<(), RenderSurfaceHandleError> {
        self.release(frame.0, SurfaceHandleKind::Frame)
    }

    fn identity(&self, kind: SurfaceHandleKind, value: u64) -> SurfaceHandleIdentity {
        SurfaceHandleIdentity {
            namespace: self.namespace,
            device_id: self.device_id,
            generation: self.generation,
            kind,
            value,
        }
    }

    fn allocate(&self, kind: SurfaceHandleKind) -> Result<u64, RenderSurfaceHandleError> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let value = match kind {
            SurfaceHandleKind::Session => state.next_session,
            SurfaceHandleKind::Frame => state.next_frame,
        };
        let next_value = value
            .checked_add(1)
            .ok_or(RenderSurfaceHandleError::StaleHandle {
                diagnostic_id: self.identity(kind, value).diagnostic_id(),
            })?;
        match kind {
            SurfaceHandleKind::Session => {
                state.next_session = next_value;
                state.active_sessions.insert(value);
            }
            SurfaceHandleKind::Frame => {
                state.next_frame = next_value;
                state.active_frames.insert(value);
            }
        }
        Ok(value)
    }

    fn validate(
        &self,
        identity: SurfaceHandleIdentity,
        expected_kind: SurfaceHandleKind,
    ) -> Result<(), RenderSurfaceHandleError> {
        self.validate_owner(identity, expected_kind)?;
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let active = match expected_kind {
            SurfaceHandleKind::Session => &state.active_sessions,
            SurfaceHandleKind::Frame => &state.active_frames,
        };
        active.contains(&identity.value).then_some(()).ok_or(
            RenderSurfaceHandleError::StaleHandle {
                diagnostic_id: identity.diagnostic_id(),
            },
        )
    }

    fn release(
        &self,
        identity: SurfaceHandleIdentity,
        expected_kind: SurfaceHandleKind,
    ) -> Result<(), RenderSurfaceHandleError> {
        self.validate_owner(identity, expected_kind)?;
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let active = match expected_kind {
            SurfaceHandleKind::Session => &mut state.active_sessions,
            SurfaceHandleKind::Frame => &mut state.active_frames,
        };
        active
            .remove(&identity.value)
            .then_some(())
            .ok_or(RenderSurfaceHandleError::StaleHandle {
                diagnostic_id: identity.diagnostic_id(),
            })
    }

    fn validate_owner(
        &self,
        identity: SurfaceHandleIdentity,
        expected_kind: SurfaceHandleKind,
    ) -> Result<(), RenderSurfaceHandleError> {
        if identity.device_id != self.device_id {
            return Err(RenderSurfaceHandleError::WrongDevice {
                expected: self.device_id,
                actual: identity.device_id,
            });
        }
        if identity.generation != self.generation {
            return Err(RenderSurfaceHandleError::WrongGeneration {
                expected: self.generation,
                actual: identity.generation,
            });
        }
        if identity.namespace != self.namespace || identity.kind != expected_kind {
            return Err(RenderSurfaceHandleError::ForeignAllocator {
                diagnostic_id: identity.diagnostic_id(),
            });
        }
        Ok(())
    }
}

/// Negotiated session receipt. The lease identity remains opaque while the
/// negotiated swapchain is available to graph and presentation planners.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurfaceSessionReceipt {
    session: SurfaceSession,
    pub swapchain: SwapchainDesc,
}

impl SurfaceSessionReceipt {
    pub fn new(session: SurfaceSession, swapchain: SwapchainDesc) -> Self {
        Self { session, swapchain }
    }

    pub const fn session(&self) -> SurfaceSession {
        self.session
    }
}

/// Surface creation or reconfiguration result. Zero extents keep a real
/// session identity but cannot acquire a frame until reconfigured.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SurfaceSessionCreateOutcome {
    Renderable(SurfaceSessionReceipt),
    NonRenderable(SurfaceSessionReceipt),
}

/// One short-lived acquired target. Its texture and default view may only be
/// used through the owning device generation until present or discard consumes
/// this lease.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurfaceFrameLease {
    frame: SurfaceFrameId,
    session: SurfaceSession,
    target: TextureHandle,
    default_view: TextureViewHandle,
    desc: TextureDesc,
}

impl SurfaceFrameLease {
    pub fn new(
        frame: SurfaceFrameId,
        session: SurfaceSession,
        target: TextureHandle,
        default_view: TextureViewHandle,
        desc: TextureDesc,
    ) -> Self {
        Self {
            frame,
            session,
            target,
            default_view,
            desc,
        }
    }

    pub const fn frame(&self) -> SurfaceFrameId {
        self.frame
    }

    pub const fn session(&self) -> SurfaceSession {
        self.session
    }

    pub const fn target(&self) -> TextureHandle {
        self.target
    }

    pub const fn default_view(&self) -> TextureViewHandle {
        self.default_view
    }

    pub const fn desc(&self) -> &TextureDesc {
        &self.desc
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceRetryReason {
    Timeout,
    Occluded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceReconfigureReason {
    Outdated,
    Lost,
    Suboptimal,
}

/// Typed acquire result. Retryable and reconfigure-required results never
/// expose a native target or a neutral texture handle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SurfaceAcquireOutcome {
    Acquired(SurfaceFrameLease),
    Retryable {
        session: SurfaceSession,
        reason: SurfaceRetryReason,
    },
    ReconfigureRequired {
        session: SurfaceSession,
        reason: SurfaceReconfigureReason,
    },
    NonRenderable {
        session: SurfaceSession,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceFrameTerminal {
    Presented,
    Discarded,
}

/// Bounded device-generation-local receipt history for recently terminalized
/// surface frames. Once a receipt ages out, the opaque handle remains stale
/// and is still rejected by its allocator.
#[derive(Debug)]
pub struct SurfaceFrameTerminalHistory {
    max_entries: usize,
    terminals: HashMap<SurfaceFrameId, SurfaceFrameTerminal>,
    terminal_order: VecDeque<SurfaceFrameId>,
}

impl SurfaceFrameTerminalHistory {
    pub fn new(max_entries: usize) -> Self {
        Self {
            max_entries,
            terminals: HashMap::new(),
            terminal_order: VecDeque::new(),
        }
    }

    pub fn terminal(&self, frame: SurfaceFrameId) -> Option<SurfaceFrameTerminal> {
        self.terminals.get(&frame).copied()
    }

    pub fn record(&mut self, frame: SurfaceFrameId, terminal: SurfaceFrameTerminal) {
        if self.terminals.insert(frame, terminal).is_none() {
            self.terminal_order.push_back(frame);
        }
        while self.terminal_order.len() > self.max_entries {
            if let Some(expired) = self.terminal_order.pop_front() {
                self.terminals.remove(&expired);
            }
        }
    }
}

impl Default for SurfaceFrameTerminalHistory {
    fn default() -> Self {
        Self::new(SubmissionLimits::default().max_terminal_statuses())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfacePresentReceipt {
    pub frame: SurfaceFrameId,
    pub submission: SubmissionTicket,
    pub terminal: SurfaceFrameTerminal,
}

#[cfg(test)]
mod tests {
    use super::{
        RenderSurfaceHandleAllocator, RenderSurfaceHandleError, SurfaceFrameTerminal,
        SurfaceFrameTerminalHistory,
    };
    use crate::{DeviceGeneration, DeviceId};

    #[test]
    fn session_and_frame_leases_are_independently_terminalized() {
        let handles =
            RenderSurfaceHandleAllocator::new(DeviceId::new(1), DeviceGeneration::initial());
        let session = handles.allocate_session().unwrap();
        let frame = handles.allocate_frame().unwrap();

        handles.release_session(session).unwrap();
        assert!(matches!(
            handles.validate_session(session),
            Err(RenderSurfaceHandleError::StaleHandle { .. })
        ));
        handles.validate_frame(frame).unwrap();

        handles.release_frame(frame).unwrap();
        assert!(matches!(
            handles.validate_frame(frame),
            Err(RenderSurfaceHandleError::StaleHandle { .. })
        ));
    }

    #[test]
    fn surface_handle_sequences_are_monotonic_and_allocator_local() {
        let handles =
            RenderSurfaceHandleAllocator::new(DeviceId::new(1), DeviceGeneration::initial());
        let first_session = handles.allocate_session().unwrap();
        let first_frame = handles.allocate_frame().unwrap();

        handles.release_session(first_session).unwrap();
        handles.release_frame(first_frame).unwrap();

        let second_session = handles.allocate_session().unwrap();
        let second_frame = handles.allocate_frame().unwrap();
        assert_eq!((first_session.0.value, second_session.0.value), (1, 2));
        assert_eq!((first_frame.0.value, second_frame.0.value), (1, 2));
        assert_ne!(second_session.diagnostic_id(), second_frame.diagnostic_id());
        handles.validate_session(second_session).unwrap();
        handles.validate_frame(second_frame).unwrap();

        let foreign =
            RenderSurfaceHandleAllocator::new(DeviceId::new(1), DeviceGeneration::initial());
        assert!(matches!(
            foreign.validate_session(second_session),
            Err(RenderSurfaceHandleError::ForeignAllocator { .. })
        ));
        assert!(matches!(
            foreign.validate_frame(second_frame),
            Err(RenderSurfaceHandleError::ForeignAllocator { .. })
        ));
    }

    #[test]
    fn surface_handle_overflow_fails_without_publishing_a_handle() {
        let handles =
            RenderSurfaceHandleAllocator::new(DeviceId::new(1), DeviceGeneration::initial());
        {
            let mut state = handles
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state.next_session = u64::MAX;
            state.next_frame = u64::MAX;
        }

        assert!(matches!(
            handles.allocate_session(),
            Err(RenderSurfaceHandleError::StaleHandle { .. })
        ));
        assert!(matches!(
            handles.allocate_frame(),
            Err(RenderSurfaceHandleError::StaleHandle { .. })
        ));

        let state = handles
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert_eq!(state.next_session, u64::MAX);
        assert_eq!(state.next_frame, u64::MAX);
        assert!(!state.active_sessions.contains(&u64::MAX));
        assert!(!state.active_frames.contains(&u64::MAX));
    }

    #[test]
    fn terminal_history_is_bounded_but_evicted_frames_remain_allocator_stale() {
        let handles =
            RenderSurfaceHandleAllocator::new(DeviceId::new(1), DeviceGeneration::initial());
        let first = handles.allocate_frame().unwrap();
        let second = handles.allocate_frame().unwrap();
        handles.release_frame(first).unwrap();
        handles.release_frame(second).unwrap();

        let mut history = SurfaceFrameTerminalHistory::new(1);
        history.record(first, SurfaceFrameTerminal::Discarded);
        history.record(second, SurfaceFrameTerminal::Presented);

        assert_eq!(history.terminal(first), None);
        assert_eq!(
            history.terminal(second),
            Some(SurfaceFrameTerminal::Presented)
        );
        assert!(handles.validate_frame(first).is_err());
    }
}
