use crate::core::diagnostics::DiagnosticStore;

mod camera_targets;
mod mesh_gpu_scene;
mod ui;
mod visibility_hzb_light;

fn assert_series(store: &DiagnosticStore, path: &str, value: f64, unit: &str) {
    let snapshot = store.snapshot();
    let series = snapshot
        .series
        .iter()
        .find(|series| series.path.as_str() == path)
        .unwrap_or_else(|| panic!("missing diagnostic series `{path}`"));
    assert_eq!(series.current, Some(value));
    assert_eq!(series.unit.as_deref(), Some(unit));
    assert_eq!(series.history.len(), 1);
    assert_eq!(series.history[0].frame_index, 12);
    assert_eq!(series.history[0].value, value);
}
