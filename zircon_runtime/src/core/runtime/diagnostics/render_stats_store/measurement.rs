use super::DiagnosticStore;

pub(super) fn record_count(
    store: &mut DiagnosticStore,
    path: &'static str,
    frame_index: u64,
    value: usize,
    subsystem_tags: &[&str],
) {
    store.record_static(
        path,
        frame_index,
        value as f64,
        Some("count"),
        subsystem_tags,
    );
}

pub(super) fn record_bytes(
    store: &mut DiagnosticStore,
    path: &'static str,
    frame_index: u64,
    value: u64,
    subsystem_tags: &[&str],
) {
    store.record_static(
        path,
        frame_index,
        value as f64,
        Some("bytes"),
        subsystem_tags,
    );
}

pub(super) fn record_microseconds(
    store: &mut DiagnosticStore,
    path: &'static str,
    frame_index: u64,
    value: u64,
    subsystem_tags: &[&str],
) {
    store.record_static(
        path,
        frame_index,
        value as f64,
        Some("microseconds"),
        subsystem_tags,
    );
}

pub(super) fn record_bool(
    store: &mut DiagnosticStore,
    path: &'static str,
    frame_index: u64,
    value: bool,
    subsystem_tags: &[&str],
) {
    store.record_static(
        path,
        frame_index,
        u8::from(value) as f64,
        Some("bool"),
        subsystem_tags,
    );
}
