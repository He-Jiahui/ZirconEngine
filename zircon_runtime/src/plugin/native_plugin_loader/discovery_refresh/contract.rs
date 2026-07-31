use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use super::super::NativePluginCandidate;
use super::ticket::NativePluginDiscoveryRefreshCancellation;

/// A stable discovery key supplied by the discovery authority after it has canonicalized a root.
/// Constructing this identity performs no filesystem work, so UI and watcher callbacks can submit
/// it without turning admission into a synchronous scan or stat operation.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NativePluginDiscoveryRoot {
    canonical_path: Arc<PathBuf>,
}

impl NativePluginDiscoveryRoot {
    pub(super) fn from_canonical_path(path: impl Into<PathBuf>) -> Self {
        Self {
            canonical_path: Arc::new(path.into()),
        }
    }

    pub fn as_path(&self) -> &Path {
        self.canonical_path.as_path()
    }
}

/// The authority-owned collector mode represented by a single refresh generation. This remains
/// internal so the public loader facade cannot bypass ticketing, admission, or publication.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum NativePluginDiscoveryRefreshInput {
    RootScan,
    LoadManifest { export_root: PathBuf },
}

impl NativePluginDiscoveryRefreshInput {
    pub(in crate::plugin::native_plugin_loader) fn root_scan() -> Self {
        Self::RootScan
    }

    pub(in crate::plugin::native_plugin_loader) fn load_manifest(export_root: PathBuf) -> Self {
        Self::LoadManifest { export_root }
    }
}

/// Non-empty collector-owned identity for the exact inputs represented by one publication.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct NativePluginDiscoveryInputIdentity(Arc<str>);

impl NativePluginDiscoveryInputIdentity {
    pub fn new(value: impl Into<Arc<str>>) -> Result<Self, NativePluginDiscoveryRefreshError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(NativePluginDiscoveryRefreshError::InvalidInputIdentity);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Admission and work ceilings for native-plugin discovery refreshes.
/// A deadline that cannot be represented by the current platform `Instant` is rejected at
/// admission instead of wrapping or becoming an implicit immediate cancellation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativePluginDiscoveryRefreshBudget {
    pub max_roots: usize,
    pub max_candidates: usize,
    pub max_diagnostics: usize,
    pub max_read_bytes: u64,
    pub max_scratch_bytes: u64,
    pub deadline: Duration,
    pub max_terminal_observers: usize,
}

impl Default for NativePluginDiscoveryRefreshBudget {
    fn default() -> Self {
        Self {
            max_roots: 16,
            max_candidates: 4_096,
            max_diagnostics: 128,
            max_read_bytes: 128 * 1024 * 1024,
            max_scratch_bytes: 64 * 1024 * 1024,
            deadline: Duration::from_secs(10),
            max_terminal_observers: 32,
        }
    }
}

impl NativePluginDiscoveryRefreshBudget {
    pub(super) fn normalized(mut self) -> Self {
        self.max_roots = self.max_roots.max(1);
        self.max_candidates = self.max_candidates.max(1);
        self.max_diagnostics = self.max_diagnostics.max(1);
        self.max_read_bytes = self.max_read_bytes.max(1);
        self.max_scratch_bytes = self.max_scratch_bytes.max(1);
        self.deadline = self.deadline.max(Duration::from_millis(1));
        self.max_terminal_observers = self.max_terminal_observers.max(1);
        self
    }
}

/// Immutable request context passed to the Frameworks04-owned collector.
#[derive(Clone, Debug)]
pub(crate) struct NativePluginDiscoveryRefreshRequest {
    root: NativePluginDiscoveryRoot,
    input: NativePluginDiscoveryRefreshInput,
    generation: u64,
    budget: NativePluginDiscoveryRefreshBudget,
    cancellation: NativePluginDiscoveryRefreshCancellation,
}

impl NativePluginDiscoveryRefreshRequest {
    pub(super) fn new(
        root: NativePluginDiscoveryRoot,
        input: NativePluginDiscoveryRefreshInput,
        generation: u64,
        budget: NativePluginDiscoveryRefreshBudget,
        cancellation: NativePluginDiscoveryRefreshCancellation,
    ) -> Self {
        Self {
            root,
            input,
            generation,
            budget,
            cancellation,
        }
    }

    pub(crate) fn root(&self) -> &NativePluginDiscoveryRoot {
        &self.root
    }

    pub(crate) fn input(&self) -> &NativePluginDiscoveryRefreshInput {
        &self.input
    }

    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }

    pub(crate) fn budget(&self) -> &NativePluginDiscoveryRefreshBudget {
        &self.budget
    }

    /// Collectors must check this between directory, manifest, and parse units.
    pub(crate) fn is_cancelled(&self) -> bool {
        self.cancellation.is_cancelled()
    }

    pub(crate) fn check_active(&self) -> Result<(), NativePluginDiscoveryRefreshError> {
        self.cancellation.check_active()
    }
}

/// Internal, already-metered collection output consumed only by Runtime11 publication.
#[derive(Debug)]
pub(super) struct NativePluginDiscoveryRefreshPayload {
    candidates: Vec<NativePluginCandidate>,
    diagnostics: Vec<String>,
    /// Stable collector-owned identity for the exact inputs represented by this payload.
    input_identity: NativePluginDiscoveryInputIdentity,
    read_bytes: u64,
    peak_scratch_bytes: u64,
}

/// Runtime-owned bounded collector output. A Frameworks04 collector must reserve through this
/// sink before it builds a candidate or diagnostic, starts a manifest read, or requests scratch.
pub(crate) struct NativePluginDiscoveryRefreshSink {
    budget: NativePluginDiscoveryRefreshBudget,
    candidates: Vec<NativePluginCandidate>,
    diagnostics: Vec<String>,
    candidate_admissions: usize,
    diagnostic_admissions: usize,
    read_bytes: u64,
    peak_scratch_bytes: u64,
}

impl NativePluginDiscoveryRefreshSink {
    pub(super) fn new(budget: NativePluginDiscoveryRefreshBudget) -> Self {
        Self {
            budget,
            candidates: Vec::new(),
            diagnostics: Vec::new(),
            candidate_admissions: 0,
            diagnostic_admissions: 0,
            read_bytes: 0,
            peak_scratch_bytes: 0,
        }
    }

    pub(crate) fn reserve_candidate(
        &mut self,
        request: &NativePluginDiscoveryRefreshRequest,
    ) -> Result<NativePluginDiscoveryRefreshCandidateReservation, NativePluginDiscoveryRefreshError>
    {
        request.check_active()?;
        validate_accounting(
            &self.budget,
            self.candidate_admissions.saturating_add(1),
            self.diagnostic_admissions,
            self.read_bytes,
            self.peak_scratch_bytes,
        )?;
        self.candidate_admissions = self.candidate_admissions.saturating_add(1);
        Ok(NativePluginDiscoveryRefreshCandidateReservation {})
    }

    pub(crate) fn reserve_diagnostic(
        &mut self,
        request: &NativePluginDiscoveryRefreshRequest,
    ) -> Result<NativePluginDiscoveryRefreshDiagnosticReservation, NativePluginDiscoveryRefreshError>
    {
        request.check_active()?;
        validate_accounting(
            &self.budget,
            self.candidate_admissions,
            self.diagnostic_admissions.saturating_add(1),
            self.read_bytes,
            self.peak_scratch_bytes,
        )?;
        self.diagnostic_admissions = self.diagnostic_admissions.saturating_add(1);
        Ok(NativePluginDiscoveryRefreshDiagnosticReservation {})
    }

    pub(crate) fn reserve_read_bytes(
        &mut self,
        request: &NativePluginDiscoveryRefreshRequest,
        requested_bytes: u64,
    ) -> Result<NativePluginDiscoveryRefreshReadReservation, NativePluginDiscoveryRefreshError>
    {
        request.check_active()?;
        let read_bytes = self
            .read_bytes
            .checked_add(requested_bytes)
            .ok_or_else(|| {
                NativePluginDiscoveryRefreshError::budget_exceeded(
                    NativePluginDiscoveryRefreshBudgetKind::ReadBytes,
                    u64::MAX,
                    self.budget.max_read_bytes,
                )
            })?;
        validate_accounting(
            &self.budget,
            self.candidate_admissions,
            self.diagnostic_admissions,
            read_bytes,
            self.peak_scratch_bytes,
        )?;
        self.read_bytes = read_bytes;
        Ok(NativePluginDiscoveryRefreshReadReservation { requested_bytes })
    }

    pub(crate) fn remaining_read_bytes(&self) -> u64 {
        self.budget.max_read_bytes.saturating_sub(self.read_bytes)
    }

    pub(crate) fn reserve_scratch_bytes(
        &mut self,
        request: &NativePluginDiscoveryRefreshRequest,
        required_bytes: u64,
    ) -> Result<NativePluginDiscoveryRefreshScratchReservation, NativePluginDiscoveryRefreshError>
    {
        request.check_active()?;
        let peak_scratch_bytes = self.peak_scratch_bytes.max(required_bytes);
        validate_accounting(
            &self.budget,
            self.candidate_admissions,
            self.diagnostic_admissions,
            self.read_bytes,
            peak_scratch_bytes,
        )?;
        self.peak_scratch_bytes = peak_scratch_bytes;
        Ok(NativePluginDiscoveryRefreshScratchReservation {})
    }

    pub(crate) fn reserve_additional_scratch_bytes(
        &mut self,
        request: &NativePluginDiscoveryRefreshRequest,
        additional_bytes: u64,
    ) -> Result<NativePluginDiscoveryRefreshScratchReservation, NativePluginDiscoveryRefreshError>
    {
        let required_bytes = self
            .peak_scratch_bytes
            .checked_add(additional_bytes)
            .ok_or_else(|| {
                NativePluginDiscoveryRefreshError::budget_exceeded(
                    NativePluginDiscoveryRefreshBudgetKind::ScratchBytes,
                    u64::MAX,
                    self.budget.max_scratch_bytes,
                )
            })?;
        self.reserve_scratch_bytes(request, required_bytes)
    }

    pub(crate) fn admitted_scratch_bytes(&self) -> u64 {
        self.peak_scratch_bytes
    }

    /// Selection collectors keep the first accepted package without creating an unmetered
    /// duplicate-id side index beside the published candidate set.
    pub(crate) fn contains_candidate_id(&self, plugin_id: &str) -> bool {
        self.candidates
            .iter()
            .any(|candidate| candidate.package_manifest.id == plugin_id)
    }

    pub(super) fn into_payload(
        self,
        input_identity: NativePluginDiscoveryInputIdentity,
    ) -> NativePluginDiscoveryRefreshPayload {
        NativePluginDiscoveryRefreshPayload {
            candidates: self.candidates,
            diagnostics: self.diagnostics,
            input_identity,
            read_bytes: self.read_bytes,
            peak_scratch_bytes: self.peak_scratch_bytes,
        }
    }
}

/// A single candidate slot admitted before the collector constructs the candidate value.
pub(crate) struct NativePluginDiscoveryRefreshCandidateReservation {}

impl NativePluginDiscoveryRefreshCandidateReservation {
    pub(crate) fn insert(
        self,
        sink: &mut NativePluginDiscoveryRefreshSink,
        candidate: NativePluginCandidate,
    ) {
        sink.candidates.push(candidate);
    }
}

/// A single diagnostic slot admitted before the collector constructs the diagnostic string.
pub(crate) struct NativePluginDiscoveryRefreshDiagnosticReservation {}

impl NativePluginDiscoveryRefreshDiagnosticReservation {
    pub(crate) fn insert(self, sink: &mut NativePluginDiscoveryRefreshSink, diagnostic: String) {
        sink.diagnostics.push(diagnostic);
    }
}

/// Read work may start only after reserving an upper bound. The caller records the actual byte
/// count so unused capacity is returned before the next read unit is admitted.
#[must_use = "record the actual admitted read byte count"]
pub(crate) struct NativePluginDiscoveryRefreshReadReservation {
    requested_bytes: u64,
}

impl NativePluginDiscoveryRefreshReadReservation {
    pub(crate) fn commit(
        self,
        sink: &mut NativePluginDiscoveryRefreshSink,
        actual_bytes: u64,
    ) -> Result<(), NativePluginDiscoveryRefreshError> {
        if actual_bytes > self.requested_bytes {
            return Err(NativePluginDiscoveryRefreshError::collector(
                "native plugin discovery read exceeded its admitted byte reservation",
            ));
        }
        sink.read_bytes = sink
            .read_bytes
            .saturating_sub(self.requested_bytes - actual_bytes);
        Ok(())
    }
}

/// Scratch work may start only after this admission token has been acquired.
#[must_use = "hold this token through the admitted scratch allocation"]
pub(crate) struct NativePluginDiscoveryRefreshScratchReservation {}

/// Immutable last-good publication consumed by editor and plugin-management code.
#[derive(Clone, Debug)]
pub struct NativePluginDiscoverySnapshot {
    root: NativePluginDiscoveryRoot,
    input: NativePluginDiscoveryRefreshInput,
    generation: u64,
    candidates: Arc<[NativePluginCandidate]>,
    diagnostics: Arc<[String]>,
    input_identity: NativePluginDiscoveryInputIdentity,
    read_bytes: u64,
    peak_scratch_bytes: u64,
}

impl NativePluginDiscoverySnapshot {
    pub(super) fn from_payload(
        root: NativePluginDiscoveryRoot,
        input: NativePluginDiscoveryRefreshInput,
        generation: u64,
        payload: NativePluginDiscoveryRefreshPayload,
    ) -> Self {
        Self {
            root,
            input,
            generation,
            candidates: Arc::from(payload.candidates),
            diagnostics: Arc::from(payload.diagnostics),
            input_identity: payload.input_identity,
            read_bytes: payload.read_bytes,
            peak_scratch_bytes: payload.peak_scratch_bytes,
        }
    }

    pub fn root(&self) -> &NativePluginDiscoveryRoot {
        &self.root
    }

    pub(crate) fn input(&self) -> &NativePluginDiscoveryRefreshInput {
        &self.input
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn candidates(&self) -> &[NativePluginCandidate] {
        &self.candidates
    }

    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }

    pub fn input_identity(&self) -> &NativePluginDiscoveryInputIdentity {
        &self.input_identity
    }

    pub fn read_bytes(&self) -> u64 {
        self.read_bytes
    }

    pub fn peak_scratch_bytes(&self) -> u64 {
        self.peak_scratch_bytes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativePluginDiscoveryRefreshBudgetKind {
    CandidateCount,
    DiagnosticCount,
    ReadBytes,
    ScratchBytes,
}

impl fmt::Display for NativePluginDiscoveryRefreshBudgetKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::CandidateCount => "candidate entry",
            Self::DiagnosticCount => "diagnostic entry",
            Self::ReadBytes => "read byte",
            Self::ScratchBytes => "scratch byte",
        };
        formatter.write_str(label)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativePluginDiscoveryRefreshError {
    Collector {
        message: Arc<str>,
    },
    Cancelled,
    DeadlineExceeded,
    BudgetExceeded {
        kind: NativePluginDiscoveryRefreshBudgetKind,
        actual: u64,
        limit: u64,
    },
    InvalidInputIdentity,
}

impl NativePluginDiscoveryRefreshError {
    pub fn collector(message: impl Into<Arc<str>>) -> Self {
        Self::Collector {
            message: message.into(),
        }
    }

    pub fn cancelled() -> Self {
        Self::Cancelled
    }

    pub(super) fn deadline_exceeded() -> Self {
        Self::DeadlineExceeded
    }

    pub(super) fn budget_exceeded(
        kind: NativePluginDiscoveryRefreshBudgetKind,
        actual: u64,
        limit: u64,
    ) -> Self {
        Self::BudgetExceeded {
            kind,
            actual,
            limit,
        }
    }
}

pub(super) fn validate_accounting(
    budget: &NativePluginDiscoveryRefreshBudget,
    candidate_count: usize,
    diagnostic_count: usize,
    read_bytes: u64,
    peak_scratch_bytes: u64,
) -> Result<(), NativePluginDiscoveryRefreshError> {
    if candidate_count > budget.max_candidates {
        return Err(NativePluginDiscoveryRefreshError::budget_exceeded(
            NativePluginDiscoveryRefreshBudgetKind::CandidateCount,
            candidate_count as u64,
            budget.max_candidates as u64,
        ));
    }
    if diagnostic_count > budget.max_diagnostics {
        return Err(NativePluginDiscoveryRefreshError::budget_exceeded(
            NativePluginDiscoveryRefreshBudgetKind::DiagnosticCount,
            diagnostic_count as u64,
            budget.max_diagnostics as u64,
        ));
    }
    if read_bytes > budget.max_read_bytes {
        return Err(NativePluginDiscoveryRefreshError::budget_exceeded(
            NativePluginDiscoveryRefreshBudgetKind::ReadBytes,
            read_bytes,
            budget.max_read_bytes,
        ));
    }
    if peak_scratch_bytes > budget.max_scratch_bytes {
        return Err(NativePluginDiscoveryRefreshError::budget_exceeded(
            NativePluginDiscoveryRefreshBudgetKind::ScratchBytes,
            peak_scratch_bytes,
            budget.max_scratch_bytes,
        ));
    }
    Ok(())
}

impl fmt::Display for NativePluginDiscoveryRefreshError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Collector { message } => formatter.write_str(message),
            Self::Cancelled => formatter.write_str("native plugin discovery refresh cancelled"),
            Self::DeadlineExceeded => {
                formatter.write_str("native plugin discovery refresh deadline exceeded")
            }
            Self::BudgetExceeded {
                kind,
                actual,
                limit,
            } => write!(
                formatter,
                "native plugin discovery refresh {kind} budget exceeded: {actual} > {limit}"
            ),
            Self::InvalidInputIdentity => {
                formatter.write_str("native plugin discovery refresh input identity is empty")
            }
        }
    }
}

impl std::error::Error for NativePluginDiscoveryRefreshError {}
