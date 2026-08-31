use std::num::NonZeroU32;

use crate::core::framework::window::{NativeWindowId, WindowRegistryId};
use zircon_runtime_interface::ZrRuntimeViewportHandle;

use super::{WindowParentKind, WindowRegistry, WindowRegistryError};

fn registry() -> WindowRegistry {
    WindowRegistry::new(WindowRegistryId::new(7).expect("registry test identity is nonzero"))
}

fn native_window(raw: u64) -> NativeWindowId {
    NativeWindowId::new(raw).expect("native test identity is nonzero")
}

fn viewport(raw: u64) -> ZrRuntimeViewportHandle {
    ZrRuntimeViewportHandle::new(raw)
}

#[test]
fn register_keeps_engine_and_native_window_lookups_consistent() {
    let mut registry = registry();
    let native_first = native_window(1);
    let native_second = native_window(2);

    let first = registry
        .register(native_first)
        .expect("first native window registers");
    let second = registry
        .register(native_second)
        .expect("second native window registers");

    assert_eq!(registry.registry_id().raw(), 7);
    assert_eq!(registry.resolve_native(native_first), Ok(first));
    assert_eq!(registry.resolve_native(native_second), Ok(second));
    assert_eq!(registry.native_for(first), Ok(native_first));
    assert_eq!(registry.native_for(second), Ok(native_second));
    assert_eq!(registry.len(), 2);
}

#[test]
fn duplicate_native_registration_preserves_the_existing_window_mapping() {
    let mut registry = registry();
    let native = native_window(1);
    let window = registry.register(native).expect("native window registers");

    assert_eq!(
        registry.register(native),
        Err(WindowRegistryError::DuplicateNativeWindow {
            native_window: native,
        })
    );
    assert_eq!(registry.resolve_native(native), Ok(window));
    assert_eq!(registry.len(), 1);
}

#[test]
fn inconsistent_native_mapping_is_rejected_before_close_mutates_registry_state() {
    let mut registry = registry();
    let native = native_window(1);
    let window = registry.register(native).expect("native window registers");
    registry
        .native_to_slot
        .insert(native, window.slot().saturating_add(1));

    assert_eq!(
        registry.begin_close(window),
        Err(WindowRegistryError::InconsistentNativeWindowMapping {
            native_window: native,
            slot: window.slot(),
        })
    );
    assert_eq!(registry.primary_window(), None);
    assert_eq!(registry.native_for(window), Ok(native));
}

#[test]
fn inconsistent_parent_membership_is_rejected_before_detaching_child_state() {
    let mut registry = registry();
    let parent = registry
        .register(native_window(1))
        .expect("parent native window registers");
    let child = registry
        .register(native_window(2))
        .expect("child native window registers");
    registry
        .set_parent(child, parent, WindowParentKind::Transient)
        .expect("child attaches to parent");
    registry.slots[parent.slot() as usize].children.clear();

    assert_eq!(
        registry.clear_parent(child),
        Err(WindowRegistryError::InconsistentWindowRelationship { parent, child })
    );
    assert_eq!(
        registry.parent_of(child),
        Ok(Some((parent, WindowParentKind::Transient)))
    );
}

#[test]
fn closing_then_destroying_retires_old_generation_and_removes_both_mappings() {
    let mut registry = registry();
    let first_native = native_window(1);
    let first = registry
        .register(first_native)
        .expect("native window registers");

    let closing = registry
        .begin_close(first)
        .expect("window begins its closing transaction");
    assert_eq!(closing.native_window(), first_native);
    assert_eq!(closing.primary_role_change(), None);
    assert_eq!(
        registry.resolve_native(first_native),
        Err(WindowRegistryError::ClosingWindow { window: first })
    );
    assert_eq!(
        registry.native_for(first),
        Err(WindowRegistryError::ClosingWindow { window: first })
    );
    assert_eq!(registry.finish_destroy(first), Ok(first_native));
    assert_eq!(
        registry.resolve_native(first_native),
        Err(WindowRegistryError::UnknownNativeWindow {
            native_window: first_native,
        })
    );
    assert_eq!(
        registry.native_for(first),
        Err(WindowRegistryError::StaleWindow { window: first })
    );

    let second_native = native_window(2);
    let second = registry
        .register(second_native)
        .expect("replacement native window registers");
    assert_eq!(second.slot(), first.slot());
    assert_ne!(second.generation(), first.generation());
    assert_eq!(registry.resolve_native(second_native), Ok(second));
}

#[test]
fn destroy_requires_the_closing_phase_to_preserve_external_teardown_order() {
    let mut registry = registry();
    let native = native_window(1);
    let window = registry.register(native).expect("native window registers");

    assert_eq!(
        registry.finish_destroy(window),
        Err(WindowRegistryError::WindowNotClosing { window })
    );
    assert_eq!(registry.native_for(window), Ok(native));
}

#[test]
fn generation_exhaustion_retires_the_slot_without_revalidating_a_stale_window() {
    let mut registry = registry();
    let first = registry
        .register(native_window(1))
        .expect("native window registers");
    registry.slots[first.slot() as usize].generation =
        NonZeroU32::new(u32::MAX).expect("maximum generation is nonzero");
    let exhausted = crate::core::framework::window::WindowId::new(
        registry.registry_id(),
        first.slot(),
        NonZeroU32::new(u32::MAX).expect("maximum generation is nonzero"),
    );

    let closing = registry
        .begin_close(exhausted)
        .expect("exhausted-generation window begins its closing transaction");
    assert_eq!(closing.native_window(), native_window(1));
    assert_eq!(closing.primary_role_change(), None);
    assert_eq!(registry.finish_destroy(exhausted), Ok(native_window(1)));

    let replacement = registry
        .register(native_window(2))
        .expect("registry skips permanently retired slot");
    assert_ne!(replacement.slot(), exhausted.slot());
    assert_eq!(
        registry.native_for(exhausted),
        Err(WindowRegistryError::StaleWindow { window: exhausted })
    );
}

#[test]
fn primary_role_replacement_advances_its_own_generation_without_reusing_window_identity() {
    let mut registry = registry();
    let first = registry
        .register(native_window(1))
        .expect("first native window registers");
    let second = registry
        .register(native_window(2))
        .expect("second native window registers");

    assert_eq!(registry.primary_window(), None);
    assert_eq!(registry.primary_generation(), 0);

    let first_selection = registry
        .set_primary(first)
        .expect("live first window becomes primary")
        .expect("new primary selection publishes a change");
    assert_eq!(first_selection.previous(), None);
    assert_eq!(first_selection.current(), Some(first));
    assert_eq!(first_selection.generation(), 1);
    assert_eq!(registry.primary_window(), Some(first));
    assert_eq!(registry.primary_generation(), 1);

    assert_eq!(
        registry
            .set_primary(first)
            .expect("re-selecting the primary remains valid"),
        None
    );
    assert_eq!(registry.primary_generation(), 1);

    let replacement = registry
        .set_primary(second)
        .expect("live second window replaces the primary")
        .expect("replacement publishes a change");
    assert_eq!(replacement.previous(), Some(first));
    assert_eq!(replacement.current(), Some(second));
    assert_eq!(replacement.generation(), 2);
    assert_eq!(registry.primary_window(), Some(second));
    assert_eq!(registry.primary_generation(), 2);
}

#[test]
fn closing_the_primary_invalidates_its_role_before_external_teardown() {
    let mut registry = registry();
    let primary = registry
        .register(native_window(1))
        .expect("primary native window registers");
    let tool = registry
        .register(native_window(2))
        .expect("tool native window registers");
    registry
        .set_primary(primary)
        .expect("primary selection succeeds");

    let closing = registry
        .begin_close(primary)
        .expect("primary begins its closing transaction");
    assert_eq!(closing.native_window(), native_window(1));
    let invalidation = closing
        .primary_role_change()
        .expect("primary close invalidates the live primary role");
    assert_eq!(invalidation.previous(), Some(primary));
    assert_eq!(invalidation.current(), None);
    assert_eq!(invalidation.generation(), 2);
    assert_eq!(registry.primary_window(), None);
    assert_eq!(registry.primary_generation(), 2);
    assert_eq!(registry.native_for(tool), Ok(native_window(2)));
    assert_eq!(
        registry.set_primary(primary),
        Err(WindowRegistryError::ClosingWindow { window: primary })
    );
    assert_eq!(registry.finish_destroy(primary), Ok(native_window(1)));

    let registry_source = include_str!("registry.rs");
    assert!(
        !registry_source.contains("WindowExitCondition")
            && !registry_source.contains("WindowLifecyclePolicy"),
        "primary role selection must remain independent from application exit policy"
    );
}

#[test]
fn closing_a_nonprimary_window_keeps_the_primary_role_stable() {
    let mut registry = registry();
    let primary = registry
        .register(native_window(1))
        .expect("primary native window registers");
    let tool = registry
        .register(native_window(2))
        .expect("tool native window registers");
    registry
        .set_primary(primary)
        .expect("primary selection succeeds");

    let closing = registry
        .begin_close(tool)
        .expect("tool window begins its closing transaction");
    assert_eq!(closing.native_window(), native_window(2));
    assert_eq!(closing.primary_role_change(), None);
    assert_eq!(registry.primary_window(), Some(primary));
    assert_eq!(registry.primary_generation(), 1);
}

#[test]
fn primary_role_generation_exhaustion_preserves_the_previous_selection() {
    let mut registry = registry();
    let first = registry
        .register(native_window(1))
        .expect("first native window registers");
    let second = registry
        .register(native_window(2))
        .expect("second native window registers");
    registry
        .set_primary(first)
        .expect("first primary selection succeeds");
    registry.primary_generation = u64::MAX;

    assert_eq!(
        registry.set_primary(second),
        Err(WindowRegistryError::PrimaryRoleGenerationExhausted)
    );
    assert_eq!(registry.primary_window(), Some(first));
    assert_eq!(registry.primary_generation(), u64::MAX);
}

#[test]
fn parent_relationships_keep_bidirectional_identity_and_reject_cycles() {
    let mut registry = registry();
    let root = registry
        .register(native_window(1))
        .expect("root native window registers");
    let child = registry
        .register(native_window(2))
        .expect("child native window registers");
    let grandchild = registry
        .register(native_window(3))
        .expect("grandchild native window registers");

    registry
        .set_parent(child, root, WindowParentKind::Transient)
        .expect("child attaches to root");
    registry
        .set_parent(grandchild, child, WindowParentKind::Modal)
        .expect("grandchild attaches to child");

    assert_eq!(
        registry.parent_of(child),
        Ok(Some((root, WindowParentKind::Transient)))
    );
    assert_eq!(
        registry.parent_of(grandchild),
        Ok(Some((child, WindowParentKind::Modal)))
    );
    assert_eq!(registry.children_of(root), Ok(vec![child]));
    assert_eq!(registry.children_of(child), Ok(vec![grandchild]));
    assert_eq!(
        registry.set_parent(root, grandchild, WindowParentKind::OwnerShutdown),
        Err(WindowRegistryError::WindowRelationshipCycle {
            child: root,
            parent: grandchild,
        })
    );
    assert_eq!(
        registry.parent_of(child),
        Ok(Some((root, WindowParentKind::Transient)))
    );
}

#[test]
fn relationship_subtree_close_is_child_first_and_invalidates_primary_once() {
    let mut registry = registry();
    let root = registry
        .register(native_window(1))
        .expect("root native window registers");
    let child = registry
        .register(native_window(2))
        .expect("child native window registers");
    let grandchild = registry
        .register(native_window(3))
        .expect("grandchild native window registers");
    let sibling = registry
        .register(native_window(4))
        .expect("sibling native window registers");
    registry
        .set_parent(child, root, WindowParentKind::Transient)
        .expect("child attaches to root");
    registry
        .set_parent(grandchild, child, WindowParentKind::Modal)
        .expect("grandchild attaches to child");
    registry
        .set_parent(sibling, root, WindowParentKind::OwnerShutdown)
        .expect("sibling attaches to root");
    registry
        .set_primary(root)
        .expect("root primary selection succeeds");

    let closing = registry
        .begin_close_tree(root)
        .expect("the entire live relationship subtree closes transactionally");
    assert_eq!(
        closing
            .iter()
            .map(|entry| entry.window())
            .collect::<Vec<_>>(),
        vec![grandchild, child, sibling, root]
    );
    assert_eq!(
        closing
            .iter()
            .filter_map(|entry| entry.primary_role_change())
            .collect::<Vec<_>>(),
        vec![closing
            .last()
            .expect("root close receipt exists")
            .primary_role_change()
            .expect("primary root close publishes one invalidation")]
    );
    assert_eq!(registry.primary_window(), None);
    assert_eq!(registry.primary_generation(), 2);

    for entry in closing {
        assert_eq!(
            registry.finish_destroy(entry.window()),
            Ok(entry.native_window())
        );
    }
    assert!(registry.is_empty());
}

#[test]
fn parent_requires_children_to_finish_destroy_before_it_can_retire() {
    let mut registry = registry();
    let parent = registry
        .register(native_window(1))
        .expect("parent native window registers");
    let child = registry
        .register(native_window(2))
        .expect("child native window registers");
    registry
        .set_parent(child, parent, WindowParentKind::Transient)
        .expect("child attaches to parent");

    assert_eq!(
        registry.begin_close(parent),
        Err(WindowRegistryError::WindowHasLiveChildren {
            window: parent,
            child_count: 1,
        })
    );
    assert_eq!(registry.native_for(parent), Ok(native_window(1)));

    registry
        .clear_parent(child)
        .expect("live child may detach from its parent");
    let closing = registry
        .begin_close(parent)
        .expect("detached parent can close independently");
    assert_eq!(closing.native_window(), native_window(1));
    assert_eq!(registry.finish_destroy(parent), Ok(native_window(1)));
}

#[test]
fn viewport_bindings_support_one_to_many_and_unbound_tool_windows() {
    let mut registry = registry();
    let render_window = registry
        .register(native_window(1))
        .expect("render window registers");
    let tool_window = registry
        .register(native_window(2))
        .expect("tool window registers");

    assert_eq!(registry.viewports_for(tool_window), Ok(Vec::new()));
    registry
        .bind_viewport(render_window, viewport(7))
        .expect("first viewport binds to render window");
    registry
        .bind_viewport(render_window, viewport(8))
        .expect("second viewport binds to render window");

    assert_eq!(
        registry.viewports_for(render_window),
        Ok(vec![viewport(7), viewport(8)])
    );
    assert_eq!(registry.window_for_viewport(viewport(7)), Ok(render_window));
    assert_eq!(registry.window_for_viewport(viewport(8)), Ok(render_window));
    assert_eq!(
        registry.bind_viewport(tool_window, viewport(7)),
        Err(WindowRegistryError::ViewportAlreadyBound {
            viewport: viewport(7),
            window: render_window,
        })
    );

    registry
        .unbind_viewport(render_window, viewport(7))
        .expect("bound viewport detaches");
    assert_eq!(registry.viewports_for(render_window), Ok(vec![viewport(8)]));
    assert_eq!(
        registry.window_for_viewport(viewport(7)),
        Err(WindowRegistryError::UnknownViewportBinding {
            viewport: viewport(7),
        })
    );
}

#[test]
fn subtree_close_revokes_viewport_bindings_before_native_destruction() {
    let mut registry = registry();
    let parent = registry
        .register(native_window(1))
        .expect("parent native window registers");
    let child = registry
        .register(native_window(2))
        .expect("child native window registers");
    registry
        .set_parent(child, parent, WindowParentKind::Transient)
        .expect("child attaches to parent");
    registry
        .bind_viewport(parent, viewport(7))
        .expect("parent viewport binds");
    registry
        .bind_viewport(child, viewport(8))
        .expect("child viewport binds");

    let closing = registry
        .begin_close_tree(parent)
        .expect("relationship subtree starts one close transaction");
    assert_eq!(closing.len(), 2);
    assert_eq!(closing[0].window(), child);
    assert_eq!(closing[0].viewports(), &[viewport(8)]);
    assert_eq!(closing[1].window(), parent);
    assert_eq!(closing[1].viewports(), &[viewport(7)]);
    assert_eq!(
        registry.window_for_viewport(viewport(7)),
        Err(WindowRegistryError::UnknownViewportBinding {
            viewport: viewport(7),
        })
    );
    assert_eq!(
        registry.window_for_viewport(viewport(8)),
        Err(WindowRegistryError::UnknownViewportBinding {
            viewport: viewport(8),
        })
    );

    for entry in closing {
        assert_eq!(
            registry.finish_destroy(entry.window()),
            Ok(entry.native_window())
        );
    }
}

#[test]
fn close_preflight_preserves_a_live_primary_when_viewport_indexes_disagree() {
    let mut registry = registry();
    let window = registry
        .register(native_window(1))
        .expect("native window registers");
    registry
        .set_primary(window)
        .expect("primary selection succeeds");
    registry
        .bind_viewport(window, viewport(7))
        .expect("viewport binding succeeds");
    registry.viewport_to_window.remove(&viewport(7));

    assert_eq!(
        registry.begin_close(window),
        Err(WindowRegistryError::InconsistentViewportBinding {
            window,
            viewport: viewport(7),
        })
    );
    assert_eq!(registry.primary_window(), Some(window));
    assert_eq!(registry.primary_generation(), 1);
    assert_eq!(registry.native_for(window), Ok(native_window(1)));
}

#[test]
fn close_preflight_rejects_a_reverse_only_viewport_binding_before_mutation() {
    let mut registry = registry();
    let window = registry
        .register(native_window(1))
        .expect("native window registers");
    registry
        .set_primary(window)
        .expect("primary selection succeeds");
    registry.viewport_to_window.insert(viewport(7), window);

    assert_eq!(
        registry.begin_close(window),
        Err(WindowRegistryError::InconsistentViewportBinding {
            window,
            viewport: viewport(7),
        })
    );
    assert_eq!(registry.primary_window(), Some(window));
    assert_eq!(registry.primary_generation(), 1);
    assert_eq!(registry.native_for(window), Ok(native_window(1)));
    assert_eq!(registry.viewport_to_window.get(&viewport(7)), Some(&window));
}
