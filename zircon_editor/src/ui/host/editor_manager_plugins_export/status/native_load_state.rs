use zircon_runtime::plugin::native::NativePluginLoadProjection;

pub(super) fn native_load_state(
    projection: &NativePluginLoadProjection,
    plugin_id: &str,
    diagnostics: &[String],
) -> String {
    let is_loaded = projection.is_loaded(plugin_id);
    native_load_state_label(
        is_loaded,
        is_loaded && projection.has_descriptor(plugin_id),
        diagnostics,
    )
    .to_string()
}

fn native_load_state_label<'a>(
    is_loaded: bool,
    has_descriptor: bool,
    diagnostics: impl IntoIterator<Item = &'a String>,
) -> &'static str {
    let mut has_diagnostics = false;
    let mut has_load_failure = false;
    for diagnostic in diagnostics {
        has_diagnostics = true;
        if is_loaded {
            if diagnostic.contains(" entry failed:") {
                return "entry failed";
            }
        } else if diagnostic.contains("library is missing") {
            return "missing library";
        } else if diagnostic.contains("failed to load") {
            has_load_failure = true;
        }
    }

    if is_loaded {
        if !has_descriptor {
            "loaded without descriptor"
        } else if has_diagnostics {
            "loaded with diagnostics"
        } else {
            "loaded"
        }
    } else if has_load_failure {
        "load failed"
    } else {
        "manifest only"
    }
}

#[cfg(test)]
#[path = "tests/native_load_state_performance_tests.rs"]
mod performance_tests;

#[cfg(test)]
#[path = "native_load_state/tests/optimization_tests.rs"]
mod optimization_tests;
