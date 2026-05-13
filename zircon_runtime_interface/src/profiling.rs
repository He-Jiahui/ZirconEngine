use serde::{Deserialize, Serialize};

use crate::buffer::{ZrByteSlice, ZrOwnedByteBuffer};
use crate::handles::ZrRuntimeSessionHandle;
use crate::status::ZrStatus;

pub type ZrRuntimeProfileControlFnV1 =
    unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrByteSlice, *mut ZrOwnedByteBuffer) -> ZrStatus;

pub const PROFILE_TIMELINE_NATIVE_FILE: &str = "timeline.zrtrace.json";
pub const PROFILE_TIMELINE_PERFETTO_FILE: &str = "timeline.perfetto.json";
pub const PROFILE_HOTSPOTS_FILE: &str = "hotspots.json";
pub const PROFILE_SUMMARY_FILE: &str = "summary.md";
pub const PROFILE_DEFAULT_OUTPUT_ROOT: &str = "target/zircon-profiles";
pub const PROFILE_DEFAULT_SESSION_ID: &str = "local";
pub const PROFILE_DEFAULT_FRAME_BUDGET_MS: f64 = 16.67;
pub const PROFILE_DEFAULT_MAX_FRAMES: usize = 512;
pub const PROFILE_DEFAULT_MAX_SPANS: usize = 16_384;
pub const PROFILE_DEFAULT_MAX_COUNTERS: usize = 4_096;

/// Capture options shared by the in-process recorder and the dynamic-runtime ABI.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProfileCaptureConfig {
    pub session_id: String,
    pub output_root: String,
    pub max_frames: usize,
    pub max_spans: usize,
    pub max_counters: usize,
    pub frame_budget_ms: f64,
    pub include_perfetto: bool,
}

impl Default for ProfileCaptureConfig {
    fn default() -> Self {
        Self {
            session_id: PROFILE_DEFAULT_SESSION_ID.to_string(),
            output_root: PROFILE_DEFAULT_OUTPUT_ROOT.to_string(),
            max_frames: PROFILE_DEFAULT_MAX_FRAMES,
            max_spans: PROFILE_DEFAULT_MAX_SPANS,
            max_counters: PROFILE_DEFAULT_MAX_COUNTERS,
            frame_budget_ms: PROFILE_DEFAULT_FRAME_BUDGET_MS,
            include_perfetto: true,
        }
    }
}

impl ProfileCaptureConfig {
    pub fn normalized(mut self) -> Self {
        if self.session_id.trim().is_empty() {
            self.session_id = PROFILE_DEFAULT_SESSION_ID.to_string();
        }
        if self.output_root.trim().is_empty() {
            self.output_root = PROFILE_DEFAULT_OUTPUT_ROOT.to_string();
        }
        if self.max_frames == 0 {
            self.max_frames = PROFILE_DEFAULT_MAX_FRAMES;
        }
        if self.max_spans == 0 {
            self.max_spans = PROFILE_DEFAULT_MAX_SPANS;
        }
        if self.max_counters == 0 {
            self.max_counters = PROFILE_DEFAULT_MAX_COUNTERS;
        }
        if self.frame_budget_ms <= 0.0 {
            self.frame_budget_ms = PROFILE_DEFAULT_FRAME_BUDGET_MS;
        }
        self
    }
}

/// A transport-safe timeline snapshot containing frame, span, and counter samples.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProfileSnapshot {
    pub session_id: String,
    pub output_root: String,
    pub active: bool,
    pub feature_enabled: bool,
    pub frame_budget_ms: f64,
    pub frames: Vec<ProfileFrameSnapshot>,
    pub spans: Vec<ProfileSpanSnapshot>,
    pub counters: Vec<ProfileCounterSnapshot>,
}

impl Default for ProfileSnapshot {
    fn default() -> Self {
        Self {
            session_id: PROFILE_DEFAULT_SESSION_ID.to_string(),
            output_root: PROFILE_DEFAULT_OUTPUT_ROOT.to_string(),
            active: false,
            feature_enabled: false,
            frame_budget_ms: PROFILE_DEFAULT_FRAME_BUDGET_MS,
            frames: Vec::new(),
            spans: Vec::new(),
            counters: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProfileFrameSnapshot {
    pub stream: String,
    pub name: String,
    pub frame_index: u64,
    pub start_us: u64,
    pub duration_us: u64,
    pub budget_ms: f64,
    pub over_budget: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProfileSpanSnapshot {
    pub id: u64,
    pub parent_id: Option<u64>,
    pub frame_index: Option<u64>,
    pub stream: String,
    pub category: String,
    pub name: String,
    pub path: String,
    pub start_us: u64,
    pub duration_us: u64,
    pub depth: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProfileCounterSnapshot {
    pub stream: String,
    pub name: String,
    pub value: f64,
    pub timestamp_us: u64,
    pub frame_index: Option<u64>,
}

/// Aggregated span cost report generated from a `ProfileSnapshot` export.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HotspotReport {
    pub session_id: String,
    pub frame_budget_ms: f64,
    pub generated_from_span_count: usize,
    pub hotspots: Vec<HotspotEntry>,
    pub hints: Vec<String>,
}

impl Default for HotspotReport {
    fn default() -> Self {
        Self {
            session_id: PROFILE_DEFAULT_SESSION_ID.to_string(),
            frame_budget_ms: PROFILE_DEFAULT_FRAME_BUDGET_MS,
            generated_from_span_count: 0,
            hotspots: Vec::new(),
            hints: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HotspotEntry {
    pub stream: String,
    pub category: String,
    pub name: String,
    pub path: String,
    pub total_us: u64,
    pub avg_us: u64,
    pub p95_us: u64,
    pub max_us: u64,
    pub count: u64,
    pub frame_count: u64,
    pub over_budget_count: u64,
}

/// JSON command carried through `ZrRuntimeProfileControlFnV1`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProfileControlCommand {
    StartCapture,
    StopCapture,
    Snapshot,
    ExportReport,
    Reset,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProfileControlRequest {
    pub command: ProfileControlCommand,
    #[serde(default)]
    pub config: Option<ProfileCaptureConfig>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProfileControlResponse {
    pub status: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<ProfileSnapshot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hotspot_report: Option<HotspotReport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub export_dir: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
}

impl ProfileControlResponse {
    pub fn ok(message: impl Into<String>) -> Self {
        Self {
            status: "ok".to_string(),
            message: message.into(),
            snapshot: None,
            hotspot_report: None,
            export_dir: None,
            files: Vec::new(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            status: "error".to_string(),
            message: message.into(),
            snapshot: None,
            hotspot_report: None,
            export_dir: None,
            files: Vec::new(),
        }
    }
}
