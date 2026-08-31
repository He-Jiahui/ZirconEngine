//! Lifecycle declarations for editor-plugin SDK consumers.

use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorPluginLifecycleStage {
    Loaded,
    Enabled,
    Disabled,
    Unloaded,
    HotReloaded,
    EnteredPlayMode,
    ExitedPlayMode,
    SceneChanged,
    AssetChanged,
    UiMessage,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorPluginLifecycleEvent {
    stage: EditorPluginLifecycleStage,
    subject: Option<String>,
}

impl EditorPluginLifecycleEvent {
    pub fn new(stage: EditorPluginLifecycleStage) -> Self {
        Self {
            stage,
            subject: None,
        }
    }

    pub fn with_subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    pub fn stage(&self) -> &EditorPluginLifecycleStage {
        &self.stage
    }

    pub fn subject(&self) -> Option<&str> {
        self.subject.as_deref()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorPluginLifecycleRecord {
    package_id: String,
    event: EditorPluginLifecycleEvent,
}

impl EditorPluginLifecycleRecord {
    pub fn new(package_id: impl Into<String>, event: EditorPluginLifecycleEvent) -> Self {
        Self {
            package_id: package_id.into(),
            event,
        }
    }

    pub fn package_id(&self) -> &str {
        &self.package_id
    }

    pub fn event(&self) -> &EditorPluginLifecycleEvent {
        &self.event
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EditorPluginLifecycleReport {
    records: Vec<EditorPluginLifecycleRecord>,
    diagnostics: Vec<String>,
}

impl EditorPluginLifecycleReport {
    pub fn record(&mut self, record: EditorPluginLifecycleRecord) {
        self.records.push(record);
    }

    pub fn extend(&mut self, report: EditorPluginLifecycleReport) {
        append_or_adopt(&mut self.records, report.records);
        append_or_adopt(&mut self.diagnostics, report.diagnostics);
    }

    pub fn push_diagnostic(&mut self, diagnostic: impl Into<String>) {
        self.diagnostics.push(diagnostic.into());
    }

    pub fn records(&self) -> &[EditorPluginLifecycleRecord] {
        &self.records
    }

    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }

    pub fn is_success(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

fn append_or_adopt<T>(target: &mut Vec<T>, mut incoming: Vec<T>) {
    if target.is_empty() && target.capacity() == 0 {
        *target = incoming;
    } else {
        target.append(&mut incoming);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorPluginLifecycleError {
    stage: EditorPluginLifecycleStage,
    message: String,
}

impl EditorPluginLifecycleError {
    pub fn new(stage: EditorPluginLifecycleStage, message: impl Into<String>) -> Self {
        Self {
            stage,
            message: message.into(),
        }
    }

    pub fn stage(&self) -> &EditorPluginLifecycleStage {
        &self.stage
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for EditorPluginLifecycleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "editor plugin lifecycle {:?} failed: {}",
            self.stage, self.message
        )
    }
}

impl std::error::Error for EditorPluginLifecycleError {}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::{
        append_or_adopt, EditorPluginLifecycleEvent, EditorPluginLifecycleRecord,
        EditorPluginLifecycleReport, EditorPluginLifecycleStage,
    };

    #[test]
    fn optimization_batch_dm_empty_lifecycle_report_adopts_both_source_buffers() {
        let mut incoming = EditorPluginLifecycleReport::default();
        incoming.record(EditorPluginLifecycleRecord::new(
            "weather",
            EditorPluginLifecycleEvent::new(EditorPluginLifecycleStage::Loaded),
        ));
        incoming.push_diagnostic("ready");
        let records_ptr = incoming.records.as_ptr();
        let diagnostics_ptr = incoming.diagnostics.as_ptr();

        let mut output = EditorPluginLifecycleReport::default();
        output.extend(incoming);

        assert_eq!(output.records.as_ptr(), records_ptr);
        assert_eq!(output.diagnostics.as_ptr(), diagnostics_ptr);
        assert_eq!(output.records()[0].package_id(), "weather");
        assert_eq!(output.diagnostics(), &["ready"]);
    }

    #[test]
    fn optimization_batch_dm_lifecycle_report_append_preserves_existing_order() {
        let mut output = EditorPluginLifecycleReport::default();
        output.push_diagnostic("first");
        let mut incoming = EditorPluginLifecycleReport::default();
        incoming.push_diagnostic("second");
        incoming.push_diagnostic("third");

        output.extend(incoming);

        assert_eq!(output.diagnostics(), &["first", "second", "third"]);
    }

    #[test]
    fn optimization_batch_dm_empty_reserved_target_reuses_its_buffer() {
        let mut output = Vec::with_capacity(8);
        let output_ptr = output.as_ptr();

        append_or_adopt(&mut output, vec![1_u64, 2, 3]);

        assert_eq!(output.as_ptr(), output_ptr);
        assert_eq!(output.as_slice(), &[1, 2, 3]);
    }

    #[test]
    #[ignore = "release-only alternating p95 performance gate"]
    fn optimization_batch_dm_adopt_first_lifecycle_report_buffers_p95() {
        const SAMPLE_PAIRS: usize = 17;
        const MERGES_PER_SAMPLE: usize = 32_768;
        const VALUES_PER_BUFFER: usize = 32;

        let template = (0..VALUES_PER_BUFFER as u64).collect::<Vec<_>>();
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_report_buffer_adoption(
                    &template,
                    MERGES_PER_SAMPLE,
                    true,
                ));
                optimized_samples.push(measure_report_buffer_adoption(
                    &template,
                    MERGES_PER_SAMPLE,
                    false,
                ));
            } else {
                optimized_samples.push(measure_report_buffer_adoption(
                    &template,
                    MERGES_PER_SAMPLE,
                    false,
                ));
                legacy_samples.push(measure_report_buffer_adoption(
                    &template,
                    MERGES_PER_SAMPLE,
                    true,
                ));
            }
        }

        let legacy_p95 = p95(&mut legacy_samples);
        let optimized_p95 = p95(&mut optimized_samples);
        println!(
            "EDITOR349_ADOPT_FIRST_LIFECYCLE_REPORT_BUFFERS_BENCH_V1 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
            optimized_p95 as f64 / legacy_p95.max(1) as f64
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
            "adopted lifecycle report buffers p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
        );
    }

    fn measure_report_buffer_adoption(template: &[u64], merge_count: usize, legacy: bool) -> u128 {
        let started_at = Instant::now();
        let mut checksum = 0_u64;
        for _ in 0..merge_count {
            let incoming = black_box(template).to_vec();
            let output = if legacy {
                let mut output = Vec::new();
                output.extend(incoming);
                output
            } else {
                let mut output = Vec::new();
                append_or_adopt(&mut output, incoming);
                output
            };
            checksum = checksum.wrapping_add(black_box(output.len()) as u64);
        }
        black_box(checksum);
        started_at.elapsed().as_nanos()
    }

    fn p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
    }
}
