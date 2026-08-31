use std::ops::Range;

use zr_rhi::{DiagnosticQueryPlan, DiagnosticReadbackTerminal, RhiError, SubmissionStatus};

use super::{PreparedDiagnosticQueryResources, PIPELINE_STATISTICS_TYPES, QUERY_VALUE_BYTES};

pub(super) fn prepare_query_resources(
    device: &wgpu::Device,
    plan: DiagnosticQueryPlan,
    timestamp_period_ns: f32,
    timestamp_query_set: Option<wgpu::QuerySet>,
    pipeline_statistics_query_set: Option<wgpu::QuerySet>,
) -> Result<PreparedDiagnosticQueryResources, RhiError> {
    let timestamp_bytes = query_bytes(u64::from(plan.timestamp_query_count()))?;
    let pipeline_statistics_bytes =
        pipeline_statistics_query_bytes(u64::from(plan.pipeline_statistics_query_count()))?;
    let timestamp_query_set = if timestamp_bytes > 0 {
        Some(timestamp_query_set.ok_or(RhiError::DiagnosticQueryPlanRequired)?)
    } else {
        None
    };
    let pipeline_statistics_query_set = if pipeline_statistics_bytes > 0 {
        Some(pipeline_statistics_query_set.ok_or(RhiError::DiagnosticQueryPlanRequired)?)
    } else {
        None
    };
    let timestamp_staging_range = 0..timestamp_bytes;
    let pipeline_statistics_offset = align_up(timestamp_bytes, query_copy_alignment())?;
    let pipeline_statistics_end = pipeline_statistics_offset
        .checked_add(pipeline_statistics_bytes)
        .ok_or_else(|| RhiError::ReadbackUnavailable {
            reason: "diagnostic query staging layout overflowed".to_string(),
        })?;
    let pipeline_statistics_staging_range = pipeline_statistics_offset..pipeline_statistics_end;
    let timestamp_resolve = timestamp_query_set.as_ref().map(|_| {
        query_resolve_buffer(
            device,
            "zircon-diagnostic-timestamp-resolve",
            timestamp_bytes,
        )
    });
    let pipeline_statistics_resolve = pipeline_statistics_query_set.as_ref().map(|_| {
        query_resolve_buffer(
            device,
            "zircon-diagnostic-pipeline-statistics-resolve",
            pipeline_statistics_bytes,
        )
    });
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-diagnostic-query-staging"),
        size: pipeline_statistics_end,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    Ok(PreparedDiagnosticQueryResources {
        plan,
        timestamp_period_ns,
        timestamp_query_set,
        pipeline_statistics_query_set,
        timestamp_resolve,
        pipeline_statistics_resolve,
        staging,
        timestamp_staging_range,
        pipeline_statistics_staging_range,
    })
}

pub(super) fn create_timestamp_query_set(
    device: &wgpu::Device,
    count: u32,
) -> Option<wgpu::QuerySet> {
    (count > 0).then(|| {
        device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("zircon-diagnostic-timestamps"),
            ty: wgpu::QueryType::Timestamp,
            count,
        })
    })
}

pub(super) fn create_pipeline_statistics_query_set(
    device: &wgpu::Device,
    count: u32,
) -> Option<wgpu::QuerySet> {
    (count > 0).then(|| {
        device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("zircon-diagnostic-pipeline-statistics"),
            ty: wgpu::QueryType::PipelineStatistics(PIPELINE_STATISTICS_TYPES),
            count,
        })
    })
}

pub(super) fn bounded_timestamp_query_count(max_scopes: usize) -> Result<u32, RhiError> {
    let query_count = max_scopes
        .checked_mul(2)
        .ok_or_else(|| RhiError::ReadbackUnavailable {
            reason: "timestamp query count overflowed".to_string(),
        })?;
    bounded_query_count(query_count, "timestamp")
}

pub(super) fn bounded_query_count(count: usize, kind: &str) -> Result<u32, RhiError> {
    u32::try_from(count).map_err(|_| RhiError::ReadbackUnavailable {
        reason: format!("{kind} query count exceeds native u32 range"),
    })
}

pub(super) fn pipeline_statistics_query_bytes(query_count: u64) -> Result<u64, RhiError> {
    query_count
        .checked_mul(zr_rhi::PIPELINE_STATISTIC_COUNTERS_PER_QUERY as u64)
        .ok_or_else(|| RhiError::ReadbackUnavailable {
            reason: "pipeline-statistics result value count overflowed".to_string(),
        })
        .and_then(query_bytes)
}

pub(super) fn query_bytes(query_value_count: u64) -> Result<u64, RhiError> {
    query_value_count
        .checked_mul(QUERY_VALUE_BYTES)
        .ok_or_else(|| RhiError::ReadbackUnavailable {
            reason: "diagnostic query byte size overflowed".to_string(),
        })
}

fn query_resolve_buffer(device: &wgpu::Device, label: &str, byte_len: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: byte_len,
        usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    })
}

const fn query_copy_alignment() -> u64 {
    wgpu::COPY_BUFFER_ALIGNMENT
}

fn align_up(value: u64, alignment: u64) -> Result<u64, RhiError> {
    value
        .checked_add(alignment.saturating_sub(1))
        .map(|aligned| aligned / alignment * alignment)
        .ok_or_else(|| RhiError::ReadbackUnavailable {
            reason: "diagnostic query staging layout overflowed".to_string(),
        })
}

pub(super) fn range_len(range: &Range<u64>) -> u64 {
    range.end.saturating_sub(range.start)
}

pub(super) fn terminal_for_submission(
    status: SubmissionStatus,
) -> Option<DiagnosticReadbackTerminal> {
    match status {
        SubmissionStatus::Accepted | SubmissionStatus::Submitted | SubmissionStatus::Completed => {
            None
        }
        SubmissionStatus::Cancelled => Some(DiagnosticReadbackTerminal::Cancelled),
        SubmissionStatus::DeviceLost => Some(DiagnosticReadbackTerminal::DeviceLost),
        SubmissionStatus::Failed => Some(DiagnosticReadbackTerminal::MapFailed),
    }
}
