use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use thiserror::Error;

use super::font::{FontCollectionRevision, FontCollectionService, SystemFontPolicy};
use super::{compiled_unicode_data_snapshot, SharedTextLayoutSession, UnicodeDataSnapshot};

mod health;

pub use health::TextRuntimeContextHealthSnapshot;

static NEXT_TEXT_RUNTIME_CONTEXT_ID: AtomicU64 = AtomicU64::new(1);

const TEXT_CONTEXT_LIFECYCLE_STATE_BITS: u32 = 2;
const TEXT_CONTEXT_LIFECYCLE_STATE_MASK: u64 = (1 << TEXT_CONTEXT_LIFECYCLE_STATE_BITS) - 1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextSystemFontPolicy {
    #[default]
    PackagedOnly,
    DiscoverPlatform,
}

impl TextSystemFontPolicy {
    const fn database_policy(self) -> SystemFontPolicy {
        match self {
            Self::PackagedOnly => SystemFontPolicy::Disabled,
            Self::DiscoverPlatform => SystemFontPolicy::Discover,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextRuntimeContextId(u64);

impl TextRuntimeContextId {
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// Identifies one context-admitted layout-session family.
///
/// Snapshot clones share this identity and lifecycle lease. A UI surface or parser cache has its own
/// owner identity and must be correlated separately.
pub struct TextSessionId {
    context: TextRuntimeContextId,
    sequence: u64,
}

impl TextSessionId {
    pub const fn context(self) -> TextRuntimeContextId {
        self.context
    }

    pub const fn sequence(self) -> u64 {
        self.sequence
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum TextRuntimeContextLifecycleState {
    Active = 0,
    Draining = 1,
    Closed = 2,
    Faulted = 3,
}

impl TextRuntimeContextLifecycleState {
    fn from_raw(value: u8) -> Self {
        match value {
            value if value == Self::Active as u8 => Self::Active,
            value if value == Self::Draining as u8 => Self::Draining,
            value if value == Self::Closed as u8 => Self::Closed,
            _ => Self::Faulted,
        }
    }
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum TextRuntimeContextAccessError {
    #[error("text runtime context {context:?} does not admit work while {state:?}")]
    Unavailable {
        context: TextRuntimeContextId,
        state: TextRuntimeContextLifecycleState,
    },
    #[error("text runtime context {context:?} exhausted its layout-session admission counter")]
    LayoutSessionAdmissionExhausted { context: TextRuntimeContextId },
    #[error("text runtime context {context:?} exhausted its active layout-session family counter")]
    ActiveLayoutSessionFamilyExhausted { context: TextRuntimeContextId },
    #[error("text runtime context {context:?} exhausted its layout-session identity space")]
    LayoutSessionIdentityExhausted { context: TextRuntimeContextId },
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub(crate) enum TextRuntimeContextCreateError {
    #[error("text runtime context identity space is exhausted")]
    IdentityExhausted,
}

#[derive(Debug)]
struct TextRuntimeContextLifecycle {
    state_and_active_families: AtomicU64,
}

impl TextRuntimeContextLifecycle {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            state_and_active_families: AtomicU64::new(Self::pack(
                TextRuntimeContextLifecycleState::Active,
                0,
            )),
        })
    }

    fn state(&self) -> TextRuntimeContextLifecycleState {
        self.snapshot().0
    }

    fn snapshot(&self) -> (TextRuntimeContextLifecycleState, u64) {
        Self::unpack(self.state_and_active_families.load(Ordering::Acquire))
    }

    fn begin_draining(&self) -> TextRuntimeContextLifecycleState {
        match self.state_and_active_families.fetch_update(
            Ordering::AcqRel,
            Ordering::Acquire,
            |word| {
                (Self::unpack(word).0 == TextRuntimeContextLifecycleState::Active).then_some(
                    Self::with_state(word, TextRuntimeContextLifecycleState::Draining),
                )
            },
        ) {
            Ok(_) => TextRuntimeContextLifecycleState::Draining,
            Err(word) => Self::unpack(word).0,
        }
    }

    fn close(&self) -> TextRuntimeContextLifecycleState {
        let state = self.begin_draining();
        if matches!(
            state,
            TextRuntimeContextLifecycleState::Closed | TextRuntimeContextLifecycleState::Faulted
        ) {
            return state;
        }
        match self.state_and_active_families.fetch_update(
            Ordering::AcqRel,
            Ordering::Acquire,
            |word| {
                let (state, active_families) = Self::unpack(word);
                (state == TextRuntimeContextLifecycleState::Draining && active_families == 0)
                    .then_some(Self::with_state(
                        word,
                        TextRuntimeContextLifecycleState::Closed,
                    ))
            },
        ) {
            Ok(_) => TextRuntimeContextLifecycleState::Closed,
            Err(word) => Self::unpack(word).0,
        }
    }

    fn admit_layout_session_family(
        &self,
        context: TextRuntimeContextId,
    ) -> Result<(), TextRuntimeContextAccessError> {
        self.state_and_active_families
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |word| {
                let (state, active_families) = Self::unpack(word);
                if state != TextRuntimeContextLifecycleState::Active {
                    return None;
                }
                active_families
                    .checked_add(1)
                    .and_then(|active_families| Self::try_pack(state, active_families))
            })
            .map(|_| ())
            .map_err(|word| {
                let state = Self::unpack(word).0;
                if state == TextRuntimeContextLifecycleState::Active {
                    TextRuntimeContextAccessError::ActiveLayoutSessionFamilyExhausted { context }
                } else {
                    TextRuntimeContextAccessError::Unavailable { context, state }
                }
            })
    }

    fn release_layout_session_family(&self) {
        let _ = self.state_and_active_families.fetch_update(
            Ordering::AcqRel,
            Ordering::Acquire,
            |word| {
                let (state, active_families) = Self::unpack(word);
                let Some(active_families) = active_families.checked_sub(1) else {
                    return Some(Self::with_state(
                        word,
                        TextRuntimeContextLifecycleState::Faulted,
                    ));
                };
                let state = if state == TextRuntimeContextLifecycleState::Draining
                    && active_families == 0
                {
                    TextRuntimeContextLifecycleState::Closed
                } else {
                    state
                };
                Some(Self::pack(state, active_families))
            },
        );
    }

    const fn pack(state: TextRuntimeContextLifecycleState, active_families: u64) -> u64 {
        (active_families << TEXT_CONTEXT_LIFECYCLE_STATE_BITS) | state as u64
    }

    const fn try_pack(
        state: TextRuntimeContextLifecycleState,
        active_families: u64,
    ) -> Option<u64> {
        if active_families <= (u64::MAX >> TEXT_CONTEXT_LIFECYCLE_STATE_BITS) {
            Some(Self::pack(state, active_families))
        } else {
            None
        }
    }

    const fn with_state(word: u64, state: TextRuntimeContextLifecycleState) -> u64 {
        (word & !TEXT_CONTEXT_LIFECYCLE_STATE_MASK) | state as u64
    }

    fn unpack(word: u64) -> (TextRuntimeContextLifecycleState, u64) {
        (
            TextRuntimeContextLifecycleState::from_raw(
                (word & TEXT_CONTEXT_LIFECYCLE_STATE_MASK) as u8,
            ),
            word >> TEXT_CONTEXT_LIFECYCLE_STATE_BITS,
        )
    }
}

#[derive(Debug)]
struct TextRuntimeSessionLeaseFamily {
    id: TextSessionId,
    lifecycle: Arc<TextRuntimeContextLifecycle>,
}

impl Drop for TextRuntimeSessionLeaseFamily {
    fn drop(&mut self) {
        self.lifecycle.release_layout_session_family();
    }
}

#[derive(Clone, Debug)]
pub(super) struct TextRuntimeSessionLease {
    _family: Arc<TextRuntimeSessionLeaseFamily>,
}

impl TextRuntimeSessionLease {
    fn admit(
        id: TextSessionId,
        lifecycle: Arc<TextRuntimeContextLifecycle>,
    ) -> Result<Self, TextRuntimeContextAccessError> {
        lifecycle.admit_layout_session_family(id.context())?;
        Ok(Self {
            _family: Arc::new(TextRuntimeSessionLeaseFamily { id, lifecycle }),
        })
    }

    pub(super) fn id(&self) -> TextSessionId {
        self._family.id
    }
}

pub struct TextRuntimeContext {
    id: TextRuntimeContextId,
    font_collection: Arc<FontCollectionService>,
    system_font_policy: TextSystemFontPolicy,
    discovered_system_face_count: usize,
    unicode_data: UnicodeDataSnapshot,
    lifecycle: Arc<TextRuntimeContextLifecycle>,
    next_layout_session_sequence: AtomicU64,
    admitted_layout_session_total: AtomicU64,
}

impl fmt::Debug for TextRuntimeContext {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TextRuntimeContext")
            .field("id", &self.id)
            .field("font_collection", &self.font_collection.collection_id())
            .field("font_generation", &self.font_collection.generation())
            .field("system_font_policy", &self.system_font_policy)
            .field(
                "discovered_system_face_count",
                &self.discovered_system_face_count,
            )
            .field("unicode_data", &self.unicode_data.id())
            .field("lifecycle", &self.lifecycle_state())
            .finish()
    }
}

impl TextRuntimeContext {
    pub(crate) fn new() -> Result<Arc<Self>, TextRuntimeContextCreateError> {
        Self::new_with_system_font_policy(TextSystemFontPolicy::PackagedOnly)
    }

    pub(crate) fn new_with_system_font_policy(
        system_font_policy: TextSystemFontPolicy,
    ) -> Result<Arc<Self>, TextRuntimeContextCreateError> {
        let (font_collection, discovered_system_face_count) =
            FontCollectionService::new_with_system_font_policy(
                system_font_policy.database_policy(),
            );
        Self::new_with_font_collection_and_policy(
            font_collection,
            system_font_policy,
            discovered_system_face_count,
        )
    }

    pub(crate) fn new_with_font_collection(
        font_collection: Arc<FontCollectionService>,
    ) -> Result<Arc<Self>, TextRuntimeContextCreateError> {
        Self::new_with_font_collection_and_policy(
            font_collection,
            TextSystemFontPolicy::PackagedOnly,
            0,
        )
    }

    fn new_with_font_collection_and_policy(
        font_collection: Arc<FontCollectionService>,
        system_font_policy: TextSystemFontPolicy,
        discovered_system_face_count: usize,
    ) -> Result<Arc<Self>, TextRuntimeContextCreateError> {
        let id = NEXT_TEXT_RUNTIME_CONTEXT_ID
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                current.checked_add(1)
            })
            .map(TextRuntimeContextId)
            .map_err(|_| TextRuntimeContextCreateError::IdentityExhausted)?;
        Ok(Arc::new(Self {
            id,
            font_collection,
            system_font_policy,
            discovered_system_face_count,
            unicode_data: compiled_unicode_data_snapshot(),
            lifecycle: TextRuntimeContextLifecycle::new(),
            next_layout_session_sequence: AtomicU64::new(1),
            admitted_layout_session_total: AtomicU64::new(0),
        }))
    }

    pub const fn id(&self) -> TextRuntimeContextId {
        self.id
    }

    pub const fn unicode_data_snapshot(&self) -> UnicodeDataSnapshot {
        self.unicode_data
    }

    pub fn lifecycle_state(&self) -> TextRuntimeContextLifecycleState {
        self.lifecycle.state()
    }

    pub(crate) fn font_collection(&self) -> Arc<FontCollectionService> {
        Arc::clone(&self.font_collection)
    }

    pub(crate) fn font_collection_revision(&self) -> FontCollectionRevision {
        self.font_collection.revision()
    }

    pub(crate) fn create_layout_session(
        &self,
    ) -> Result<SharedTextLayoutSession, TextRuntimeContextAccessError> {
        self.require_active()?;
        let session_id = self.allocate_layout_session_id()?;
        let lease = TextRuntimeSessionLease::admit(session_id, Arc::clone(&self.lifecycle))?;
        let session = SharedTextLayoutSession::new_with_font_collection_and_context_lease(
            self.font_collection(),
            lease,
        );
        self.require_active()?;
        self.record_layout_session_admission()?;
        Ok(session)
    }

    pub(crate) fn begin_draining(&self) -> TextRuntimeContextLifecycleState {
        self.lifecycle.begin_draining()
    }

    pub(crate) fn close(&self) -> TextRuntimeContextLifecycleState {
        self.lifecycle.close()
    }

    fn require_active(&self) -> Result<(), TextRuntimeContextAccessError> {
        let state = self.lifecycle_state();
        if state == TextRuntimeContextLifecycleState::Active {
            Ok(())
        } else {
            Err(TextRuntimeContextAccessError::Unavailable {
                context: self.id,
                state,
            })
        }
    }

    fn record_layout_session_admission(&self) -> Result<(), TextRuntimeContextAccessError> {
        self.admitted_layout_session_total
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                current.checked_add(1)
            })
            .map(|_| ())
            .map_err(
                |_| TextRuntimeContextAccessError::LayoutSessionAdmissionExhausted {
                    context: self.id,
                },
            )
    }

    fn allocate_layout_session_id(&self) -> Result<TextSessionId, TextRuntimeContextAccessError> {
        self.next_layout_session_sequence
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                current.checked_add(1)
            })
            .map(|sequence| TextSessionId {
                context: self.id,
                sequence,
            })
            .map_err(
                |_| TextRuntimeContextAccessError::LayoutSessionIdentityExhausted {
                    context: self.id,
                },
            )
    }
}

#[cfg(test)]
#[path = "tests/context.rs"]
mod tests;
