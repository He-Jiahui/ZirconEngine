//! 发现服务与权威收集器之间的预算契约和不可变发布载荷。
//! 准入凭据在候选、诊断或读取内容形成之前取得；这些凭据约束本次工作，并非自动管理分配器。
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use super::super::NativePluginCandidate;
use super::manifest_index::NativePluginDiscoveryManifestIndex;
use super::metrics::NativePluginDiscoveryRefreshMetrics;
use super::ticket::NativePluginDiscoveryRefreshCancellation;
use super::work::NativePluginDiscoveryRefreshWork;

/// A stable discovery key supplied by the discovery authority after it has canonicalized a root.
/// Constructing this identity performs no filesystem work, so UI and watcher callbacks can submit
/// it without turning admission into a synchronous scan or stat operation.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
/// 身份由权威准备阶段建立；缺失或不可访问的路径可能保留词法形式，不能据此断言磁盘路径存在。
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
    LoadManifest { export_root: Arc<PathBuf> },
}

impl NativePluginDiscoveryRefreshInput {
    pub(in crate::plugin::native_plugin_loader) fn root_scan() -> Self {
        Self::RootScan
    }

    pub(in crate::plugin::native_plugin_loader) fn load_manifest(export_root: PathBuf) -> Self {
        Self::LoadManifest {
            export_root: Arc::new(export_root),
        }
    }
}

#[cfg(test)]
#[path = "tests/contract_refresh_input_tests.rs"]
mod refresh_input_tests;

/// Non-empty collector-owned identity for the exact inputs represented by one publication.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
// TODO: [CR-PLUGIN-NATIVE-0204] 确认输入身份是否需要区分清单内容变化；当前权威只记录根、工作种类和计数，
// 相同规模的内容修改可能产生相同身份；下一步明确公开消费者能否把它用于缓存或相等性判断。
/// 当前权威提供来源描述，不提供清单字节指纹；内容版本应结合发布代际判断。
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
/// 根预算针对根与输入模式组合，资源预算针对单次收集；归一化后零配置会成为最小非零额度。
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
    work: NativePluginDiscoveryRefreshWork,
    base_snapshot: Option<Arc<NativePluginDiscoverySnapshot>>,
    generation: u64,
    budget: NativePluginDiscoveryRefreshBudget,
    cancellation: NativePluginDiscoveryRefreshCancellation,
}

impl NativePluginDiscoveryRefreshRequest {
    pub(super) fn new(
        root: NativePluginDiscoveryRoot,
        input: NativePluginDiscoveryRefreshInput,
        work: NativePluginDiscoveryRefreshWork,
        base_snapshot: Option<Arc<NativePluginDiscoverySnapshot>>,
        generation: u64,
        budget: NativePluginDiscoveryRefreshBudget,
        cancellation: NativePluginDiscoveryRefreshCancellation,
    ) -> Self {
        Self {
            root,
            input,
            work,
            base_snapshot,
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

    /// Work belongs to the active ticket, keeping `(root, input)` as the stable selection key.
    pub(in crate::plugin::native_plugin_loader) fn work(
        &self,
    ) -> &NativePluginDiscoveryRefreshWork {
        &self.work
    }

    /// Incremental collection reads from an immutable last-good snapshot and never mutates it.
    pub(crate) fn base_snapshot(&self) -> Option<&Arc<NativePluginDiscoverySnapshot>> {
        self.base_snapshot.as_ref()
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
    metrics: NativePluginDiscoveryRefreshMetrics,
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
    metrics: NativePluginDiscoveryRefreshMetrics,
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
            metrics: NativePluginDiscoveryRefreshMetrics::default(),
        }
    }

    /// 候选解析前取得一次性额度；后续解析失败或选择校验丢弃候选也会消耗本次尝试的额度。
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

    /// 在构造诊断字符串前准入；调用者应将延迟构造闭包留到取得额度之后。
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

    /// 每个读取单元预留上界，再用实际读取字节提交；失败路径也须归还该单元未使用的额度。
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

    /// 参数是本次工作需要的总暂存上界，不是额外增量；不会因令牌离开作用域而自动降低峰值。
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

    /// 在既有准入峰值上累加估算，用于源缓冲和解析存储；保守额度会保留到本代际结束。
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

    pub(in crate::plugin::native_plugin_loader) fn record_traversal(
        &mut self,
        enumerated_directories: u64,
        inspected_entries: u64,
    ) {
        self.metrics
            .record_traversal(enumerated_directories, inspected_entries);
    }

    pub(in crate::plugin::native_plugin_loader) fn record_manifest_read(&mut self) {
        self.metrics.record_manifest_read();
    }

    pub(in crate::plugin::native_plugin_loader) fn record_manifest_parse(&mut self) {
        self.metrics.record_manifest_parse();
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
            metrics: self.metrics,
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
    /// 凭据须提交给创建它的同一收集槽；令牌消费防止重复归还，但类型本身没有记录槽身份。
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
/// 候选与诊断以共享切片对外只读；读取字节和暂存峰值描述生成此快照的工作，而非全树累计量。
pub struct NativePluginDiscoverySnapshot {
    root: NativePluginDiscoveryRoot,
    input: NativePluginDiscoveryRefreshInput,
    generation: u64,
    manifest_index: NativePluginDiscoveryManifestIndex,
    candidates: Arc<[NativePluginCandidate]>,
    diagnostics: Arc<[String]>,
    collector_diagnostics: Arc<[String]>,
    input_identity: NativePluginDiscoveryInputIdentity,
    read_bytes: u64,
    peak_scratch_bytes: u64,
    metrics: NativePluginDiscoveryRefreshMetrics,
}

impl NativePluginDiscoverySnapshot {
    pub(super) fn from_payload(
        root: NativePluginDiscoveryRoot,
        input: NativePluginDiscoveryRefreshInput,
        generation: u64,
        payload: NativePluginDiscoveryRefreshPayload,
    ) -> Self {
        let NativePluginDiscoveryRefreshPayload {
            candidates,
            diagnostics,
            input_identity,
            read_bytes,
            peak_scratch_bytes,
            metrics,
        } = payload;
        let index = NativePluginDiscoveryManifestIndex::from_candidates(candidates);
        Self::from_index(
            root,
            input,
            generation,
            index,
            diagnostics,
            input_identity,
            read_bytes,
            peak_scratch_bytes,
            metrics,
        )
    }

    pub(super) fn from_incremental_payload(
        root: NativePluginDiscoveryRoot,
        input: NativePluginDiscoveryRefreshInput,
        generation: u64,
        base: &NativePluginDiscoverySnapshot,
        work: &NativePluginDiscoveryRefreshWork,
        payload: NativePluginDiscoveryRefreshPayload,
        max_candidates: usize,
    ) -> Result<Self, NativePluginDiscoveryRefreshError> {
        let NativePluginDiscoveryRefreshPayload {
            candidates,
            diagnostics,
            input_identity,
            read_bytes,
            peak_scratch_bytes,
            metrics,
        } = payload;
        // TODO: [CR-PLUGIN-NATIVE-0208] 明确暂存预算是否覆盖发布阶段的索引克隆与候选投影；
        // 此处及后续投影在收集槽之外复制持久候选，现有预算测试只检查收集准入；下一步核算大基准快照。
        let mut index = base.manifest_index.clone();
        index.apply_incremental(work, &candidates, max_candidates)?;
        // BUG: [CR-PLUGIN-NATIVE-0203] 全扫曾记录某清单解析失败后，即使该清单经增量刷新修复或删除，
        // 旧收集诊断仍被整体继承并展示；证据：全扫访客转诊断，本处没有按通知路径失效旧诊断。
        let mut collector_diagnostics = base.collector_diagnostics.to_vec();
        collector_diagnostics.extend(diagnostics);
        Ok(Self::from_index(
            root,
            input,
            generation,
            index,
            collector_diagnostics,
            input_identity,
            read_bytes,
            peak_scratch_bytes,
            metrics,
        ))
    }

    fn from_index(
        root: NativePluginDiscoveryRoot,
        input: NativePluginDiscoveryRefreshInput,
        generation: u64,
        manifest_index: NativePluginDiscoveryManifestIndex,
        collector_diagnostics: Vec<String>,
        input_identity: NativePluginDiscoveryInputIdentity,
        read_bytes: u64,
        peak_scratch_bytes: u64,
        metrics: NativePluginDiscoveryRefreshMetrics,
    ) -> Self {
        let (candidates, duplicate_diagnostics) = manifest_index.project();
        let mut diagnostics = collector_diagnostics.clone();
        diagnostics.extend(duplicate_diagnostics);
        Self {
            root,
            input,
            generation,
            manifest_index,
            candidates: Arc::from(candidates),
            diagnostics: Arc::from(diagnostics),
            collector_diagnostics: Arc::from(collector_diagnostics),
            input_identity,
            read_bytes,
            peak_scratch_bytes,
            metrics,
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

    /// 按清单路径确定性选择重复包的唯一赢家；这只是发现候选，尚未执行动态库。
    pub fn candidates(&self) -> &[NativePluginCandidate] {
        &self.candidates
    }

    /// 包含收集诊断和当前索引重算的重复包诊断；消费者应与同一快照的候选一起投影。
    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }

    pub fn input_identity(&self) -> &NativePluginDiscoveryInputIdentity {
        &self.input_identity
    }

    /// 本代际实际读取量；删除增量可以为零，即便发布快照仍包含其他包。
    pub fn read_bytes(&self) -> u64 {
        self.read_bytes
    }

    pub fn peak_scratch_bytes(&self) -> u64 {
        self.peak_scratch_bytes
    }

    #[cfg(test)]
    pub(crate) fn metrics(&self) -> NativePluginDiscoveryRefreshMetrics {
        self.metrics
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
/// 收集失败与预算/期限拒绝由票据发布；服务保留既有快照，调用端决定如何展示失败。
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
