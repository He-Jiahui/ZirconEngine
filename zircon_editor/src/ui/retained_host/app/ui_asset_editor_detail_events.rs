use super::*;

mod binding;
mod collection;
mod component_adapter;
mod detail_dispatch;
mod palette;
mod preview;
mod source;
mod structure;
mod style;
mod widget;

#[cfg(test)]
mod tests {
    macro_rules! detail_event_sources {
        ($($path:literal),+ $(,)?) => {
            &[$(($path, include_str!(concat!("ui_asset_editor_detail_events/", $path)))),+]
        };
    }

    const DETAIL_EVENT_SOURCES: &[(&str, &str)] = detail_event_sources![
        "binding/entry/lifecycle.rs",
        "binding/payload.rs",
        "binding/suggestions/action.rs",
        "binding/suggestions/payload.rs",
        "binding/suggestions/route.rs",
        "collection.rs",
        "component_adapter.rs",
        "palette.rs",
        "preview/nested.rs",
        "preview/suggestions.rs",
        "preview/value.rs",
        "source.rs",
        "structure/layout/semantic.rs",
        "structure/slot/semantic.rs",
        "style/class.rs",
        "style/rules/declaration.rs",
        "style/rules/rule.rs",
        "style/theme_source.rs",
        "style/tokens.rs",
        "widget/promote.rs",
    ];

    #[test]
    fn instance_scoped_detail_events_do_not_widen_presentation_invalidation() {
        for (path, source) in DETAIL_EVENT_SOURCES {
            let production = source.split("#[cfg(test)]").next().unwrap_or(source);
            assert!(
                production.contains("mark_presentation_dirty_for_view"),
                "{path} must retain its UiAssetEditor instance as the invalidation target"
            );
            if *path == "component_adapter.rs" {
                assert!(
                    production.contains("dispatch_ui_component_adapter_event(&envelope)"),
                    "the component adapter must dispatch the same envelope that owns the target"
                );
                assert!(
                    production.contains("envelope.target.subject.take()"),
                    "the component adapter must reuse the dispatched target identity"
                );
                assert!(
                    production.contains("mark_presentation_dirty_for_view(&view_instance_id)"),
                    "the component adapter must invalidate the recovered target identity"
                );
            } else {
                assert!(
                    production.contains("mark_presentation_dirty_for_view(&instance_id)")
                        || production.contains("mark_presentation_dirty_for_view(instance_id)"),
                    "{path} must invalidate the instance passed to its detail-event handler"
                );
            }
            assert!(
                !production.contains("mark_presentation_dirty()"),
                "{path} must not promote an instance-local detail edit to global presentation"
            );
        }
    }
}
