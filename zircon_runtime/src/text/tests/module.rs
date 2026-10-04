use std::sync::Arc;

use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::CoreRuntime;

use crate::text::{TextRuntimeContextLifecycleState, TextSystemFontPolicy};

use super::{module_descriptor, text_runtime_context_for_core, TextModule, TEXT_MODULE_NAME};

fn runtime_with_text_services() -> CoreRuntime {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(module_descriptor())
        .expect("text module should register");
    runtime
        .activate_module(TEXT_MODULE_NAME)
        .expect("text module should activate");
    runtime
}

#[test]
fn text_module_selects_system_font_policy_from_runtime_target() {
    assert_eq!(
        TextModule::for_target(RuntimeTargetMode::ClientRuntime).system_font_policy,
        TextSystemFontPolicy::DiscoverPlatform
    );
    assert_eq!(
        TextModule::for_target(RuntimeTargetMode::EditorHost).system_font_policy,
        TextSystemFontPolicy::DiscoverPlatform
    );
    assert_eq!(
        TextModule::for_target(RuntimeTargetMode::ServerRuntime).system_font_policy,
        TextSystemFontPolicy::PackagedOnly
    );
    assert_eq!(
        TextModule::default().system_font_policy,
        TextSystemFontPolicy::PackagedOnly
    );
}

#[test]
fn text_font_services_are_stable_within_one_runtime_and_isolated_across_runtimes() {
    let first_runtime = runtime_with_text_services();
    let second_runtime = runtime_with_text_services();
    let first_context =
        text_runtime_context_for_core(&first_runtime.handle()).expect("first runtime text context");
    let first_again_context = text_runtime_context_for_core(&first_runtime.handle())
        .expect("first runtime text context should remain resolvable");
    let second_context = text_runtime_context_for_core(&second_runtime.handle())
        .expect("second runtime text context");
    let first = first_context.font_collection();
    let first_again = first_again_context.font_collection();
    let second = second_context.font_collection();

    assert!(Arc::ptr_eq(&first, &first_again));
    assert!(!Arc::ptr_eq(&first, &second));
    assert_ne!(first.collection_id(), second.collection_id());
}

#[test]
fn text_runtime_context_is_stable_per_core_and_isolated_across_cores() {
    let first_runtime = runtime_with_text_services();
    let second_runtime = runtime_with_text_services();
    let first =
        text_runtime_context_for_core(&first_runtime.handle()).expect("first runtime text context");
    let first_again = text_runtime_context_for_core(&first_runtime.handle())
        .expect("first runtime text context should remain resolvable");
    let second = text_runtime_context_for_core(&second_runtime.handle())
        .expect("second runtime text context");

    assert!(Arc::ptr_eq(&first, &first_again));
    assert_ne!(first.id(), second.id());
    assert_ne!(
        first.font_collection_revision().collection_id(),
        second.font_collection_revision().collection_id()
    );
}

#[test]
fn text_runtime_context_module_cleanup_closes_retained_context_and_reactivation_replaces_it() {
    let runtime = runtime_with_text_services();
    let retained =
        text_runtime_context_for_core(&runtime.handle()).expect("retained runtime text context");

    runtime
        .deactivate_module(TEXT_MODULE_NAME)
        .expect("text module should deactivate");
    assert_eq!(
        retained.lifecycle_state(),
        TextRuntimeContextLifecycleState::Closed
    );
    assert!(retained.create_layout_session().is_err());

    runtime
        .activate_module(TEXT_MODULE_NAME)
        .expect("text module should reactivate");
    let replacement =
        text_runtime_context_for_core(&runtime.handle()).expect("replacement runtime text context");
    assert_ne!(retained.id(), replacement.id());
    assert_eq!(
        replacement.lifecycle_state(),
        TextRuntimeContextLifecycleState::Active
    );
}
