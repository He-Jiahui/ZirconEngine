from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


class Editor04PlayHistoryContextContractTests(unittest.TestCase):
    def test_history_identity_is_partitioned_by_play_instance(self) -> None:
        history = read("zircon_editor/src/core/editing/engine/history.rs")

        self.assertIn("PlaySession(PlayInstanceId)", history)
        self.assertIn("pub const fn is_volatile", history)
        self.assertIn("WorldDomain::Play(instance)", history)

    def test_routing_rejects_cross_world_history_contexts(self) -> None:
        routing = read("zircon_editor/src/core/editing/engine/routing.rs")

        self.assertIn("world_domain: WorldDomain", routing)
        self.assertIn("HistoryContextId::PlaySession(instance)", routing)
        self.assertIn("CrossWorldHistory", routing)

    def test_volatile_history_never_enters_document_dirty_state(self) -> None:
        replay = read(
            "zircon_editor/src/core/editing/engine/transaction/replay.rs"
        )
        dirty = read(
            "zircon_editor/src/core/editing/engine/transaction/dirty_batch.rs"
        )

        self.assertIn("history.is_volatile()", replay)
        self.assertIn("!history.is_volatile()", dirty)
        self.assertIn("dirty: Option<HistoryDirtyChangeReservation>", dirty)

    def test_volatile_history_cannot_be_saved_or_journaled(self) -> None:
        save = read(
            "zircon_editor/src/core/editing/engine/transaction/save_token.rs"
        )
        journal = read(
            "zircon_editor/src/core/editing/engine/journal/transaction.rs"
        )

        self.assertIn("VolatileHistoryPersistenceUnsupported", save)
        self.assertIn("VolatileHistory", journal)

    def test_play_history_has_an_explicit_discard_owner(self) -> None:
        volatile = read(
            "zircon_editor/src/core/editing/engine/transaction/volatile.rs"
        )

        self.assertIn("pub fn discard_play_history", volatile)
        self.assertIn("HistoryContextId::PlaySession(instance)", volatile)
        self.assertIn("record.finalize(context.as_mut())", volatile)
        self.assertIn("history_generations.remove(&history)", volatile)
        self.assertIn("capture_world_route(WorldDomain::Edit)", volatile)
        self.assertIn("activate_world_route(&authoring_route)", volatile)

    def test_terminal_detach_discards_play_history_before_the_gateway(self) -> None:
        controller = read("zircon_editor/src/core/play/controller.rs")
        host = read("zircon_editor/src/ui/host/editor_host_event_controller.rs")
        shutdown = read(
            "zircon_editor/src/ui/host/editor_host_event_controller/runtime_shutdown.rs"
        )
        body = shutdown.split("fn detach_terminal_play_gateway", 1)[1].split(
            "/// Stops the project-owned Play session", 1
        )[0]

        self.assertIn("discard_play_history(instance)", body)
        self.assertLess(
            body.index("discard_play_history(instance)"),
            body.index(".detach_terminal_play_gateway("),
        )
        self.assertIn("EditorTerminalPlayDetachError", shutdown)
        self.assertNotIn("pub fn detach_play_gateway", controller)
        self.assertNotIn("pub fn detach_play_gateway", host)

    def test_terminal_detach_runs_extension_cleanup_outside_the_transition_gate(self) -> None:
        controller = read("zircon_editor/src/core/play/controller.rs")
        body = controller.split("fn detach_terminal_play_gateway", 1)[1].split(
            "pub fn attached_world_domain", 1
        )[0]

        self.assertIn("TerminalGatewayDetachReservation::acquire", body)
        self.assertIn("drop(transition)", body)
        self.assertLess(body.index("drop(transition)"), body.index("prepare(instance)"))

    def test_backend_retirement_waits_for_terminal_gateway_detachment(self) -> None:
        controller = read("zircon_editor/src/core/play/controller.rs")
        body = controller.split("pub fn retire_terminal_backend", 1)[1].split(
            "fn plugin_activation", 1
        )[0]

        self.assertIn("self.terminal_gateway_detach.load(Ordering::Acquire)", body)
        self.assertIn("self.play_domain.attached_domain().is_some()", body)

    def test_discard_retires_the_play_world_context(self) -> None:
        command = read("zircon_editor/src/core/editing/engine/command.rs")
        context = read("zircon_editor/src/core/editing/context.rs")
        volatile = read(
            "zircon_editor/src/core/editing/engine/transaction/volatile.rs"
        )

        self.assertIn("fn retire_world_route", command)
        self.assertIn("self.selections.remove(&world_domain)", context)
        self.assertIn(
            "context.retire_world_route(WorldDomain::Play(instance))", volatile
        )

    def test_world_routed_context_replaces_the_legacy_play_rejection(self) -> None:
        context = read("zircon_editor/src/core/editing/context.rs")
        scope = read("zircon_editor/src/core/editing/engine/transaction/scope.rs")
        command = read("zircon_editor/src/core/editing/engine/command.rs")
        state = read(
            "zircon_editor/src/core/editing/engine/transaction/engine_state.rs"
        )

        self.assertIn("authoring_gateway: EditorRuntimeGatewayHandle", context)
        self.assertIn("play_gateway: EditorRuntimeGatewayHandle", context)
        self.assertIn("with_world_at_identity", context)
        self.assertIn("with_world_mut_at_identity", context)
        self.assertIn("capture_world_route", scope)
        self.assertIn("activate_world_route", scope)
        self.assertIn("route: EditWorldRoute", state)
        self.assertNotIn("ensure_single_gateway_history", scope)
        self.assertNotIn("VolatileHistoryRouteUnavailable", command)

    def test_public_edit_context_can_preserve_a_runtime_route_identity(self) -> None:
        command = read("zircon_editor/src/core/editing/engine/command.rs")

        self.assertIn("pub fn runtime(", command)
        self.assertIn("pub fn gateway_identity", command)

    def test_runtime_operation_commands_use_one_identity_pinned_route(self) -> None:
        command = read(
            "zircon_plugins/navigation/editor/src/operation_command/command.rs"
        )
        context = read("zircon_editor/src/core/editing/engine/command.rs")
        route = read("zircon_editor/src/core/gateway/operation_route.rs")

        self.assertIn(".runtime_operations()", command)
        self.assertNotIn("context.runtime_gateway()", command)
        self.assertIn("fn runtime_operations", context)
        self.assertIn("origin: GatewayOrigin", route)
        self.assertIn("self.origin.gateway().submit_operation", route)
        self.assertIn("self.origin.gateway().poll_operation", route)
        self.assertIn("self.origin.gateway().harvest_operation", route)

    def test_play_route_has_one_attachment_authority_and_one_shared_handle(self) -> None:
        builder = read("zircon_editor/src/core/context/builder.rs")
        editor_context = read("zircon_editor/src/core/context/editor_context.rs")
        host = read("zircon_editor/src/ui/host/editor_host_event_controller.rs")

        self.assertIn("let play_gateway = EditorRuntimeGatewayHandle::detached()", builder)
        self.assertIn("play_gateway: EditorRuntimeGatewayHandle", editor_context)
        self.assertIn("play_gateway_handle", editor_context)
        self.assertIn("context.play_gateway_handle()", host)

    def test_replay_and_discard_activate_the_exact_history_world(self) -> None:
        history = read("zircon_editor/src/core/editing/engine/history.rs")
        replay = read(
            "zircon_editor/src/core/editing/engine/transaction/replay.rs"
        )
        volatile = read(
            "zircon_editor/src/core/editing/engine/transaction/volatile.rs"
        )

        self.assertIn("route: EditWorldRoute", history)
        self.assertIn("replay_route(undo)", replay)
        self.assertIn("activate_world_route(&route)", replay)
        self.assertNotIn("capture_world_route(history.world_domain())", replay)
        self.assertIn("world_route()", volatile)
        self.assertIn("activate_world_route(route)", volatile)
        self.assertNotIn("capture_world_route(history.world_domain())", volatile)

    def test_workbench_history_commands_follow_the_active_world_domain(self) -> None:
        binding = read(
            "zircon_editor/src/ui/workbench/state/scene_document_binding.rs"
        )
        apply_intent = read(
            "zircon_editor/src/ui/workbench/state/editor_state_apply_intent.rs"
        )
        snapshot = read(
            "zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs"
        )
        context = read("zircon_editor/src/core/editing/context.rs")
        mutation_guard = apply_intent.split("let mutates_edit_world", 1)[1].split(
            "if self.is_playing()", 1
        )[0]

        self.assertIn("WorldDomain::Play(instance)", binding)
        self.assertIn("HistoryContextId::PlaySession(instance)", binding)
        self.assertNotIn("EditorIntent::Undo", mutation_guard)
        self.assertNotIn("EditorIntent::Redo", mutation_guard)
        self.assertNotIn("(!self.is_playing())", snapshot)
        self.assertIn("bind_selection", context)

    def test_play_inspector_captures_commands_inside_the_pinned_transaction_route(self) -> None:
        scope = read(
            "zircon_editor/src/core/editing/engine/transaction/scope.rs"
        )
        selection = read(
            "zircon_editor/src/ui/workbench/state/editor_state_selection.rs"
        )
        binding = read(
            "zircon_editor/src/ui/binding_dispatch/inspector/apply.rs"
        )
        projection = read(
            "zircon_editor/src/ui/host/play_inspector_projection.rs"
        )

        self.assertIn("with_context_mut", scope)
        self.assertIn("apply_play_inspector_changes", selection)
        self.assertIn("WorldDomain::Play", binding)
        self.assertIn("context.with_scene", selection)
        self.assertIn("field.writable && field.serializable", projection)

    def test_operation_dispatch_routes_explicit_edit_targets_before_command_capture(self) -> None:
        target = read(
            "zircon_editor/src/core/editing/operation/edit_target.rs"
        )
        registration = read(
            "zircon_editor/src/core/editing/operation/registration.rs"
        )
        dispatch = read(
            "zircon_editor/src/ui/host/editor_operation_dispatch.rs"
        )
        event = read("zircon_editor/src/core/editor_event/types.rs")

        self.assertIn("pub enum EditOperationTarget", target)
        self.assertIn("edit_target: EditOperationTarget", registration)
        self.assertIn("operation_factory.edit_target()", dispatch)
        self.assertLess(
            dispatch.index(".route_edit("),
            dispatch.index("operation_factory.create("),
        )
        self.assertIn("EditQueued", event)
        self.assertFalse(
            (ROOT / "zircon_editor/src/core/play/edit_policy/target.rs").exists()
        )

    def test_operation_registration_requires_explicit_target_metadata(self) -> None:
        registration = read(
            "zircon_editor/src/core/editing/operation/registration.rs"
        )
        navigation = read(
            "zircon_plugins/navigation/editor/src/plugin/registration/operations.rs"
        )
        neural = read("zircon_plugins/neural/editor/src/plugin.rs")

        constructor = registration.split("pub fn new(", 1)[1].split(") -> Self", 1)[0]
        self.assertIn("edit_target: EditOperationTarget", constructor)
        self.assertNotIn("with_edit_target", registration)
        self.assertIn("EditOperationTarget::EditWorkspace", navigation)
        self.assertIn("EditOperationTarget::EditWorkspace", neural)


if __name__ == "__main__":
    unittest.main()
