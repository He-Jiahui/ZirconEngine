//! Backend-neutral diagnostic query planning and aggregation.
//!
//! The graph compiler owns labels and chooses which physical passes request
//! diagnostics. This module owns only dense identifiers, bounded query ranges,
//! and decoded numeric data. Native query objects, resolve buffers, and map
//! callbacks remain backend implementation details.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::diagnostic_readback::DiagnosticReadbackBudget;

const TIMESTAMP_VALUES_PER_SCOPE: usize = 2;
/// WGPU resolves one pipeline-statistics query into these five counters.
/// This is result payload width, not query-set index width.
pub const PIPELINE_STATISTIC_COUNTERS_PER_QUERY: usize = 5;
const QUERY_VALUE_BYTES: usize = size_of::<u64>();

/// Compiler-owned dense index for a logical graph pass.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PassDiagnosticId(u32);

impl PassDiagnosticId {
    const fn from_index(index: usize) -> Self {
        Self(index as u32)
    }

    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// A pair of timestamp-query slots assigned to one physical pass scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimestampScope {
    pass: PassDiagnosticId,
    begin_query: u32,
    end_query: u32,
}

impl TimestampScope {
    pub const fn pass(self) -> PassDiagnosticId {
        self.pass
    }

    pub const fn begin_query(self) -> u32 {
        self.begin_query
    }

    pub const fn end_query(self) -> u32 {
        self.end_query
    }
}

/// One native pipeline-statistics query slot for one physical pass.
///
/// Resolving this query produces [`PIPELINE_STATISTIC_COUNTERS_PER_QUERY`]
/// consecutive `u64` counter values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipelineStatisticsScope {
    pass: PassDiagnosticId,
    query_index: u32,
}

impl PipelineStatisticsScope {
    pub const fn pass(self) -> PassDiagnosticId {
        self.pass
    }

    pub const fn query_index(self) -> u32 {
        self.query_index
    }
}

/// Diagnostics associated with one render or compute pass boundary.
///
/// The same plan can reserve either query kind independently. When both are
/// present they must belong to the same graph pass.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticPassQueryScope {
    timestamp: Option<TimestampScope>,
    pipeline_statistics: Option<PipelineStatisticsScope>,
}

impl DiagnosticPassQueryScope {
    pub const fn new(
        timestamp: Option<TimestampScope>,
        pipeline_statistics: Option<PipelineStatisticsScope>,
    ) -> Self {
        Self {
            timestamp,
            pipeline_statistics,
        }
    }

    pub const fn timestamp(self) -> Option<TimestampScope> {
        self.timestamp
    }

    pub const fn pipeline_statistics(self) -> Option<PipelineStatisticsScope> {
        self.pipeline_statistics
    }

    pub const fn is_empty(self) -> bool {
        self.timestamp.is_none() && self.pipeline_statistics.is_none()
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DiagnosticQueryPlanError {
    #[error("diagnostic pass count {current_passes} exceeds configured limit {limit}")]
    PassLimitExceeded { current_passes: usize, limit: usize },
    #[error("diagnostic pass identifier space is exhausted")]
    PassIdentifierExhausted,
    #[error("diagnostic pass {pass:?} is not registered in this query plan")]
    UnknownPass { pass: PassDiagnosticId },
    #[error("timestamp scope count {current_scopes} exceeds configured limit {limit}")]
    TimestampScopeLimitExceeded { current_scopes: usize, limit: usize },
    #[error("pipeline-statistics scope count {current_scopes} exceeds configured limit {limit}")]
    PipelineStatisticsScopeLimitExceeded { current_scopes: usize, limit: usize },
    #[error("diagnostic query index space is exhausted")]
    QueryIndexExhausted,
    #[error("a diagnostic pass scope combines query ranges from different passes")]
    ScopePassMismatch,
    #[error("diagnostic pass scope is not owned by this query plan")]
    UnknownScope,
    #[error("a diagnostic query scope was recorded more than once")]
    DuplicateScope,
    #[error("a reserved diagnostic query scope was not recorded by the submission packet")]
    UnusedScope,
}

/// Preallocated query ranges for one diagnostic graph frame.
///
/// A plan is built before native command encoding, so every range is bounded
/// before a backend allocates query sets or staging buffers. `for_frame` binds
/// the plan to a graph frame for submission-qualified native delivery.
#[derive(Clone)]
pub struct DiagnosticQueryPlan {
    frame_index: Option<u64>,
    budget: DiagnosticReadbackBudget,
    pass_count: usize,
    timestamp_scopes: Vec<TimestampScope>,
    pipeline_statistics_scopes: Vec<PipelineStatisticsScope>,
}

impl DiagnosticQueryPlan {
    pub fn new(budget: DiagnosticReadbackBudget) -> Self {
        Self::with_optional_frame_index(None, budget)
    }

    pub fn for_frame(frame_index: u64, budget: DiagnosticReadbackBudget) -> Self {
        Self::with_optional_frame_index(Some(frame_index), budget)
    }

    pub const fn frame_index(&self) -> Option<u64> {
        self.frame_index
    }

    pub fn register_pass(&mut self) -> Result<PassDiagnosticId, DiagnosticQueryPlanError> {
        if self.pass_count >= self.budget.max_diagnostic_passes() {
            return Err(DiagnosticQueryPlanError::PassLimitExceeded {
                current_passes: self.pass_count,
                limit: self.budget.max_diagnostic_passes(),
            });
        }
        let index = u32::try_from(self.pass_count)
            .map_err(|_| DiagnosticQueryPlanError::PassIdentifierExhausted)?;
        self.pass_count = self.pass_count.saturating_add(1);
        Ok(PassDiagnosticId(index))
    }

    pub fn reserve_timestamp_scope(
        &mut self,
        pass: PassDiagnosticId,
    ) -> Result<TimestampScope, DiagnosticQueryPlanError> {
        self.validate_pass(pass)?;
        if self.timestamp_scopes.len() >= self.budget.max_timestamp_scopes() {
            return Err(DiagnosticQueryPlanError::TimestampScopeLimitExceeded {
                current_scopes: self.timestamp_scopes.len(),
                limit: self.budget.max_timestamp_scopes(),
            });
        }
        let begin_query = query_index(self.timestamp_scopes.len(), TIMESTAMP_VALUES_PER_SCOPE)?;
        let end_query = begin_query
            .checked_add(1)
            .ok_or(DiagnosticQueryPlanError::QueryIndexExhausted)?;
        let scope = TimestampScope {
            pass,
            begin_query,
            end_query,
        };
        self.timestamp_scopes.push(scope);
        Ok(scope)
    }

    pub fn reserve_pipeline_statistics_scope(
        &mut self,
        pass: PassDiagnosticId,
    ) -> Result<PipelineStatisticsScope, DiagnosticQueryPlanError> {
        self.validate_pass(pass)?;
        if self.pipeline_statistics_scopes.len() >= self.budget.max_pipeline_statistics_scopes() {
            return Err(
                DiagnosticQueryPlanError::PipelineStatisticsScopeLimitExceeded {
                    current_scopes: self.pipeline_statistics_scopes.len(),
                    limit: self.budget.max_pipeline_statistics_scopes(),
                },
            );
        }
        let query_index = query_index(self.pipeline_statistics_scopes.len(), 1)?;
        let scope = PipelineStatisticsScope { pass, query_index };
        self.pipeline_statistics_scopes.push(scope);
        Ok(scope)
    }

    pub fn pass_scope(
        &self,
        timestamp: Option<TimestampScope>,
        pipeline_statistics: Option<PipelineStatisticsScope>,
    ) -> Result<DiagnosticPassQueryScope, DiagnosticQueryPlanError> {
        let scope = DiagnosticPassQueryScope::new(timestamp, pipeline_statistics);
        self.validate_pass_scope(scope)?;
        Ok(scope)
    }

    pub const fn pass_count(&self) -> usize {
        self.pass_count
    }

    pub fn timestamp_scopes(&self) -> &[TimestampScope] {
        &self.timestamp_scopes
    }

    pub fn pipeline_statistics_scopes(&self) -> &[PipelineStatisticsScope] {
        &self.pipeline_statistics_scopes
    }

    pub fn timestamp_query_count(&self) -> u32 {
        (self.timestamp_scopes.len() * TIMESTAMP_VALUES_PER_SCOPE) as u32
    }

    pub fn pipeline_statistics_query_count(&self) -> u32 {
        self.pipeline_statistics_scopes.len() as u32
    }

    /// Number of resolved `u64` values, including every counter of each
    /// pipeline-statistics query.
    pub fn pipeline_statistics_result_value_count(&self) -> usize {
        self.pipeline_statistics_scopes
            .len()
            .saturating_mul(PIPELINE_STATISTIC_COUNTERS_PER_QUERY)
    }

    pub fn is_empty(&self) -> bool {
        self.timestamp_scopes.is_empty() && self.pipeline_statistics_scopes.is_empty()
    }

    pub fn validate_pass_scope(
        &self,
        scope: DiagnosticPassQueryScope,
    ) -> Result<(), DiagnosticQueryPlanError> {
        let timestamp_pass = scope.timestamp().map(TimestampScope::pass);
        let statistics_pass = scope
            .pipeline_statistics()
            .map(PipelineStatisticsScope::pass);
        if timestamp_pass.is_some()
            && timestamp_pass != statistics_pass
            && statistics_pass.is_some()
        {
            return Err(DiagnosticQueryPlanError::ScopePassMismatch);
        }
        if let Some(timestamp) = scope.timestamp() {
            if !self.timestamp_scope_is_registered(timestamp) {
                return Err(DiagnosticQueryPlanError::UnknownScope);
            }
        }
        if let Some(statistics) = scope.pipeline_statistics() {
            if !self.pipeline_statistics_scope_is_registered(statistics) {
                return Err(DiagnosticQueryPlanError::UnknownScope);
            }
        }
        Ok(())
    }

    /// Verifies that a packet consumes every reserved native query range once.
    /// This is intentionally performed during packet construction, before a
    /// backend creates any query set or command encoder.
    pub fn validate_submission_scopes(
        &self,
        scopes: &[DiagnosticPassQueryScope],
    ) -> Result<(), DiagnosticQueryPlanError> {
        let mut timestamps = vec![false; self.timestamp_scopes.len()];
        let mut pipeline_statistics = vec![false; self.pipeline_statistics_scopes.len()];
        for scope in scopes {
            self.validate_pass_scope(*scope)?;
            if let Some(timestamp) = scope.timestamp() {
                let index = (timestamp.begin_query() / TIMESTAMP_VALUES_PER_SCOPE as u32) as usize;
                if std::mem::replace(&mut timestamps[index], true) {
                    return Err(DiagnosticQueryPlanError::DuplicateScope);
                }
            }
            if let Some(statistics) = scope.pipeline_statistics() {
                let index = statistics.query_index() as usize;
                if std::mem::replace(&mut pipeline_statistics[index], true) {
                    return Err(DiagnosticQueryPlanError::DuplicateScope);
                }
            }
        }
        if timestamps.iter().any(|used| !used) || pipeline_statistics.iter().any(|used| !used) {
            return Err(DiagnosticQueryPlanError::UnusedScope);
        }
        Ok(())
    }

    fn with_optional_frame_index(
        frame_index: Option<u64>,
        budget: DiagnosticReadbackBudget,
    ) -> Self {
        Self {
            frame_index,
            budget,
            pass_count: 0,
            timestamp_scopes: Vec::new(),
            pipeline_statistics_scopes: Vec::new(),
        }
    }

    fn validate_pass(&self, pass: PassDiagnosticId) -> Result<(), DiagnosticQueryPlanError> {
        if pass.index() >= self.pass_count {
            return Err(DiagnosticQueryPlanError::UnknownPass { pass });
        }
        Ok(())
    }

    fn timestamp_scope_is_registered(&self, scope: TimestampScope) -> bool {
        let index = (scope.begin_query() / TIMESTAMP_VALUES_PER_SCOPE as u32) as usize;
        self.timestamp_scopes
            .get(index)
            .is_some_and(|known| *known == scope)
    }

    fn pipeline_statistics_scope_is_registered(&self, scope: PipelineStatisticsScope) -> bool {
        let index = scope.query_index() as usize;
        self.pipeline_statistics_scopes
            .get(index)
            .is_some_and(|known| *known == scope)
    }
}

/// Aggregated pipeline statistics for one logical graph pass.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticPipelineStatistics {
    pub vertex_shader_invocations: u64,
    pub clipper_invocations: u64,
    pub clipper_primitives_out: u64,
    pub fragment_shader_invocations: u64,
    pub compute_shader_invocations: u64,
}

impl DiagnosticPipelineStatistics {
    fn saturating_add_assign(&mut self, other: Self) {
        self.vertex_shader_invocations = self
            .vertex_shader_invocations
            .saturating_add(other.vertex_shader_invocations);
        self.clipper_invocations = self
            .clipper_invocations
            .saturating_add(other.clipper_invocations);
        self.clipper_primitives_out = self
            .clipper_primitives_out
            .saturating_add(other.clipper_primitives_out);
        self.fragment_shader_invocations = self
            .fragment_shader_invocations
            .saturating_add(other.fragment_shader_invocations);
        self.compute_shader_invocations = self
            .compute_shader_invocations
            .saturating_add(other.compute_shader_invocations);
    }
}

/// Numeric results for one logical graph pass, indexed by `PassDiagnosticId`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticPassResult {
    pub pass: PassDiagnosticId,
    pub timestamp_ticks: u64,
    pub pipeline_statistics: DiagnosticPipelineStatistics,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DiagnosticQueryDecodeError {
    #[error("timestamp result bytes are {actual_bytes}, expected {expected_bytes}")]
    TimestampByteLengthMismatch {
        expected_bytes: usize,
        actual_bytes: usize,
    },
    #[error("pipeline-statistics result bytes are {actual_bytes}, expected {expected_bytes}")]
    PipelineStatisticsByteLengthMismatch {
        expected_bytes: usize,
        actual_bytes: usize,
    },
    #[error("diagnostic aggregation could not reserve {requested} pass results")]
    ResultAllocationFailed { requested: usize },
}

/// Decodes resolved query data and aggregates every physical scope in O(N).
pub fn aggregate_diagnostic_query_results(
    plan: &DiagnosticQueryPlan,
    timestamp_bytes: &[u8],
    pipeline_statistics_bytes: &[u8],
) -> Result<Vec<DiagnosticPassResult>, DiagnosticQueryDecodeError> {
    validate_byte_len(
        timestamp_bytes,
        plan.timestamp_query_count() as usize,
        DiagnosticQueryKind::Timestamp,
    )?;
    validate_byte_len(
        pipeline_statistics_bytes,
        plan.pipeline_statistics_result_value_count(),
        DiagnosticQueryKind::PipelineStatistics,
    )?;

    let mut results = Vec::new();
    results.try_reserve_exact(plan.pass_count()).map_err(|_| {
        DiagnosticQueryDecodeError::ResultAllocationFailed {
            requested: plan.pass_count(),
        }
    })?;
    for index in 0..plan.pass_count() {
        results.push(DiagnosticPassResult {
            pass: PassDiagnosticId::from_index(index),
            timestamp_ticks: 0,
            pipeline_statistics: DiagnosticPipelineStatistics::default(),
        });
    }

    for scope in plan.timestamp_scopes() {
        let start = read_u64(timestamp_bytes, scope.begin_query() as usize);
        let end = read_u64(timestamp_bytes, scope.end_query() as usize);
        results[scope.pass().index()].timestamp_ticks = results[scope.pass().index()]
            .timestamp_ticks
            .saturating_add(end.saturating_sub(start));
    }
    for scope in plan.pipeline_statistics_scopes() {
        let first = scope.query_index() as usize * PIPELINE_STATISTIC_COUNTERS_PER_QUERY;
        let values = DiagnosticPipelineStatistics {
            vertex_shader_invocations: read_u64(pipeline_statistics_bytes, first),
            clipper_invocations: read_u64(pipeline_statistics_bytes, first + 1),
            clipper_primitives_out: read_u64(pipeline_statistics_bytes, first + 2),
            fragment_shader_invocations: read_u64(pipeline_statistics_bytes, first + 3),
            compute_shader_invocations: read_u64(pipeline_statistics_bytes, first + 4),
        };
        results[scope.pass().index()]
            .pipeline_statistics
            .saturating_add_assign(values);
    }
    Ok(results)
}

#[derive(Clone, Copy)]
enum DiagnosticQueryKind {
    Timestamp,
    PipelineStatistics,
}

fn query_index(
    scope_index: usize,
    values_per_scope: usize,
) -> Result<u32, DiagnosticQueryPlanError> {
    let index = scope_index
        .checked_mul(values_per_scope)
        .ok_or(DiagnosticQueryPlanError::QueryIndexExhausted)?;
    u32::try_from(index).map_err(|_| DiagnosticQueryPlanError::QueryIndexExhausted)
}

fn validate_byte_len(
    bytes: &[u8],
    query_count: usize,
    kind: DiagnosticQueryKind,
) -> Result<(), DiagnosticQueryDecodeError> {
    let expected_bytes = query_count.saturating_mul(QUERY_VALUE_BYTES);
    if bytes.len() == expected_bytes {
        return Ok(());
    }
    match kind {
        DiagnosticQueryKind::Timestamp => {
            Err(DiagnosticQueryDecodeError::TimestampByteLengthMismatch {
                expected_bytes,
                actual_bytes: bytes.len(),
            })
        }
        DiagnosticQueryKind::PipelineStatistics => Err(
            DiagnosticQueryDecodeError::PipelineStatisticsByteLengthMismatch {
                expected_bytes,
                actual_bytes: bytes.len(),
            },
        ),
    }
}

fn read_u64(bytes: &[u8], index: usize) -> u64 {
    let offset = index * QUERY_VALUE_BYTES;
    let mut raw = [0_u8; QUERY_VALUE_BYTES];
    raw.copy_from_slice(&bytes[offset..offset + QUERY_VALUE_BYTES]);
    u64::from_le_bytes(raw)
}
