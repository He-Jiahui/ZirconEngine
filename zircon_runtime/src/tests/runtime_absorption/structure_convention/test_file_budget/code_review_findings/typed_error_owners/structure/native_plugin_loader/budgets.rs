use super::super::super::super::super::*;
use super::*;

pub(super) fn assert_typed_error_native_plugin_loader_structure_budgets_are_focused(
    sources: &TypedErrorNativePluginLoaderSources,
) {
    for (path, source) in [
        (
            TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD,
            sources.structure_assertions_parent.as_str(),
        ),
        (
            TYPED_ERROR_NATIVE_STRUCTURE_CHILD,
            sources.native_structure_child.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
    for (path, source) in typed_error_native_plugin_loader_child_sources() {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 native plugin loader child budget; got {line_count} lines"
        );
    }
    for (path, source) in routes::typed_error_native_plugin_loader_route_child_sources() {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 native plugin loader route child budget; got {line_count} lines"
        );
    }
}
