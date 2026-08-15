use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use zircon_runtime_interface::world_sync::{
    InvalidationBatch, WatchKey, WatchRegistration, WatchToken,
};

use crate::core::editor_event::ViewInstanceId;
use crate::core::editor_message::{EditorViewInvalidationMask, ViewDirtySet};

#[cfg(test)]
mod tests;

/// One editor-owned projection rule for a runtime-issued watch token.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldWatchBinding {
    token: WatchToken,
    view: ViewInstanceId,
    mask: EditorViewInvalidationMask,
    depends_on: Vec<WatchKey>,
}

impl WorldWatchBinding {
    /// Returns the runtime-issued token owned by this binding.
    pub fn token(&self) -> WatchToken {
        self.token
    }

    /// Returns the editor view receiving invalidation marks.
    pub fn view(&self) -> &ViewInstanceId {
        &self.view
    }

    /// Returns the invalidation mask projected for the target view.
    pub fn mask(&self) -> EditorViewInvalidationMask {
        self.mask
    }

    /// Returns the explicit runtime facts that make this view binding dirty.
    ///
    /// The registration remains editor-owned metadata after the opaque token has crossed the
    /// runtime boundary. Consumers can therefore explain or replace a dirty binding without
    /// recovering a second source of truth from the runtime subscription table.
    pub fn depends_on(&self) -> &[WatchKey] {
        &self.depends_on
    }
}

/// Rejected registration that leaves both watch-map indexes unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldWatchMapError {
    /// Runtime tokens use zero as the invalid sentinel.
    InvalidToken,
    /// A binding without an invalidation category cannot affect a view.
    EmptyInvalidationMask,
}

impl fmt::Display for WorldWatchMapError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidToken => formatter.write_str("runtime watch token must be non-zero"),
            Self::EmptyInvalidationMask => {
                formatter.write_str("world watch binding requires a non-empty invalidation mask")
            }
        }
    }
}

impl std::error::Error for WorldWatchMapError {}

/// Deterministic projection of one runtime invalidation batch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldWatchProjection {
    generation: u64,
    dirty: ViewDirtySet,
    matched_tokens: usize,
    used_canonical_fast_path: bool,
    duplicate_tokens: Vec<WatchToken>,
    unknown_tokens: Vec<WatchToken>,
}

impl WorldWatchProjection {
    /// Runtime world generation that produced this projection.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Coalesced dirty-view state, with diagnostics retained on this projection.
    pub fn dirty(&self) -> &ViewDirtySet {
        &self.dirty
    }

    /// Consumes the projection and returns only its dirty-view state.
    ///
    /// Discards duplicate and unknown-token diagnostics; inspect them before calling when
    /// diagnostics are required.
    pub fn into_dirty(self) -> ViewDirtySet {
        self.dirty
    }

    /// Number of unique dirty tokens that matched a live binding.
    pub fn matched_tokens(&self) -> usize {
        self.matched_tokens
    }

    /// Returns true when the runtime batch was already strictly sorted and unique.
    pub fn used_canonical_fast_path(&self) -> bool {
        self.used_canonical_fast_path
    }

    /// Sorted tokens repeated in the runtime batch.
    pub fn duplicate_tokens(&self) -> &[WatchToken] {
        &self.duplicate_tokens
    }

    /// Sorted unique tokens that no longer have an editor binding.
    pub fn unknown_tokens(&self) -> &[WatchToken] {
        &self.unknown_tokens
    }
}

/// Session-scoped editor ownership for runtime watch tokens and target views.
#[derive(Clone, Debug, Default)]
pub struct WorldWatchMap {
    by_token: BTreeMap<WatchToken, WorldWatchBinding>,
    by_view: BTreeMap<ViewInstanceId, BTreeSet<WatchToken>>,
}

impl WorldWatchMap {
    /// Returns true when the session owns no runtime watch token.
    pub fn is_empty(&self) -> bool {
        self.by_token.is_empty()
    }

    /// Returns the number of runtime token bindings.
    pub fn len(&self) -> usize {
        self.by_token.len()
    }

    /// Looks up the current editor binding for a runtime token.
    pub fn binding(&self, token: WatchToken) -> Option<&WorldWatchBinding> {
        self.by_token.get(&token)
    }

    /// Returns an already-owned token for the exact view dependency declaration.
    ///
    /// This makes view registration idempotent without collapsing distinct dependencies owned by
    /// the same view.
    pub fn token_for(
        &self,
        view: &ViewInstanceId,
        registration: &WatchRegistration,
        mask: EditorViewInvalidationMask,
    ) -> Option<WatchToken> {
        self.by_view.get(view)?.iter().copied().find(|token| {
            self.by_token.get(token).is_some_and(|binding| {
                binding.mask() == mask
                    && binding.depends_on() == std::slice::from_ref(&registration.key)
            })
        })
    }

    /// Iterates tokens for one view in sorted token order.
    pub fn tokens_for_view(&self, view: &ViewInstanceId) -> impl Iterator<Item = WatchToken> + '_ {
        self.by_view
            .get(view)
            .into_iter()
            .flat_map(|tokens| tokens.iter().copied())
    }

    /// Atomically binds a token to a view and invalidation mask.
    ///
    /// Returns the replaced binding when the token was already owned. Validation happens before
    /// either index is changed, so a rejected rebind preserves the previous relation.
    pub fn bind(
        &mut self,
        token: WatchToken,
        registration: WatchRegistration,
        view: ViewInstanceId,
        mask: EditorViewInvalidationMask,
    ) -> Result<Option<WorldWatchBinding>, WorldWatchMapError> {
        if token.value() == 0 {
            return Err(WorldWatchMapError::InvalidToken);
        }
        if mask.is_empty() {
            return Err(WorldWatchMapError::EmptyInvalidationMask);
        }

        let previous = self.remove_token(token);
        self.by_view.entry(view.clone()).or_default().insert(token);
        self.by_token.insert(
            token,
            WorldWatchBinding {
                token,
                view,
                mask,
                depends_on: vec![registration.key],
            },
        );
        Ok(previous)
    }

    /// Removes one token from both indexes and returns its former binding.
    ///
    /// Unknown tokens are a no-op.
    pub fn unbind_token(&mut self, token: WatchToken) -> Option<WorldWatchBinding> {
        self.remove_token(token)
    }

    /// Removes every binding for a view and returns runtime unwatch tokens in sorted token order.
    pub fn unbind_view(&mut self, view: &ViewInstanceId) -> Vec<WatchToken> {
        let Some(tokens) = self.by_view.remove(view) else {
            return Vec::new();
        };
        let tokens = tokens.into_iter().collect::<Vec<_>>();
        for token in &tokens {
            let removed = self.by_token.remove(token);
            debug_assert!(removed
                .as_ref()
                .is_some_and(|binding| binding.view() == view));
        }
        tokens
    }

    /// Returns every token in sorted token order for runtime unwatch and clears session state.
    pub fn drain_tokens(&mut self) -> Vec<WatchToken> {
        let tokens = self.by_token.keys().copied().collect::<Vec<_>>();
        self.by_token.clear();
        self.by_view.clear();
        tokens
    }

    /// Projects only dirty tokens from the batch; registered watches are never scanned.
    pub fn project(&self, batch: &InvalidationBatch) -> WorldWatchProjection {
        if batch.has_canonical_dirty_tokens() {
            return self.project_canonical_dirty_tokens(batch);
        }

        let mut dirty = ViewDirtySet::default();
        let mut seen = BTreeSet::new();
        let mut duplicates = BTreeSet::new();
        let mut unknown = BTreeSet::new();
        let mut matched_tokens = 0;

        for token in batch.dirty.iter().copied() {
            if !seen.insert(token) {
                duplicates.insert(token);
                continue;
            }
            if let Some(binding) = self.by_token.get(&token) {
                dirty.mark_ref(binding.view(), binding.mask);
                matched_tokens += 1;
            } else {
                unknown.insert(token);
            }
        }

        WorldWatchProjection {
            generation: batch.generation,
            dirty,
            matched_tokens,
            used_canonical_fast_path: false,
            duplicate_tokens: duplicates.into_iter().collect(),
            unknown_tokens: unknown.into_iter().collect(),
        }
    }

    fn project_canonical_dirty_tokens(&self, batch: &InvalidationBatch) -> WorldWatchProjection {
        let mut dirty = ViewDirtySet::default();
        let mut unknown_tokens = Vec::new();
        let mut matched_tokens = 0;

        for token in batch.dirty.iter().copied() {
            if let Some(binding) = self.by_token.get(&token) {
                dirty.mark_ref(binding.view(), binding.mask);
                matched_tokens += 1;
            } else {
                unknown_tokens.push(token);
            }
        }

        WorldWatchProjection {
            generation: batch.generation,
            dirty,
            matched_tokens,
            used_canonical_fast_path: true,
            duplicate_tokens: Vec::new(),
            unknown_tokens,
        }
    }

    fn remove_token(&mut self, token: WatchToken) -> Option<WorldWatchBinding> {
        let binding = self.by_token.remove(&token)?;
        let remove_view = self.by_view.get_mut(binding.view()).is_some_and(|tokens| {
            tokens.remove(&token);
            tokens.is_empty()
        });
        if remove_view {
            self.by_view.remove(binding.view());
        }
        Some(binding)
    }
}
