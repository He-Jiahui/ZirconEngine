use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 12 input recording/replay",
        [
            "InputRecording",
            "InputReplayCursor",
            "input_recording_captures_drainable_event_records_by_frame",
            "180s timeout no result",
        ],
    ),
    (
        "Runtime 12 cursor host requests",
        [
            "ZrRuntimeHostRequestV1::Cursor",
            "apply_runtime_cursor_host_request",
            "platform.cursor_options",
            "missing_cursor_host_request_anchors = []",
        ],
    ),
];
