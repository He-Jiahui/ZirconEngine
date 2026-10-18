import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
TAURI = ROOT / "zircon_hub/src/tauri_app"
RUNTIME = TAURI / "runtime_state.rs"
COMMANDS = TAURI / "commands.rs"
TASKS = TAURI / "runtime_state/action_tasks.rs"
VIEW_MODEL = TAURI / "view_model.rs"
RUNTIME_TESTS = TAURI / "runtime_state/tests/cases.rs"


class HubRustStatePublicationContractTests(unittest.TestCase):
    def test_session_owns_epoch_and_increments_revision_only_on_publication(self) -> None:
        source = RUNTIME.read_text(encoding="utf-8")

        for anchor in (
            "use std::",
            "sync::atomic::{AtomicU64, Ordering}",
            "static BACKEND_EPOCH_COUNTER: AtomicU64 = AtomicU64::new(1);",
            "backend_epoch: String,",
            "state_revision: u64,",
            "BACKEND_EPOCH_COUNTER.fetch_add(1, Ordering::Relaxed)",
            "state_revision: 0,",
            "pub(super) fn view_model(&self) -> HubViewModel",
            "pub(super) fn publish_view_model(&mut self) -> HubViewModel",
            "checked_add(1)",
            "view_model.backend_epoch = self.backend_epoch.clone();",
            "view_model.state_revision = self.state_revision.to_string();",
        ):
            self.assertIn(anchor, source)

        publish = source.split("pub(super) fn publish_view_model", 1)[1]
        self.assertLess(publish.index("checked_add(1)"), publish.index("self.view_model()"))
        self.assertNotIn("state_revision +=", publish)

    def test_view_model_wire_shape_has_decimal_chronology_and_snapshot_fallback(self) -> None:
        source = VIEW_MODEL.read_text(encoding="utf-8")

        for anchor in (
            "pub backend_epoch: String,",
            "pub state_revision: String,",
            "backend_epoch: \"snapshot\".to_owned(),",
            "state_revision: \"0\".to_string(),",
            "pub(crate) fn from_snapshot_with_availability(",
        ):
            self.assertIn(anchor, source)

        self.assertNotIn("pub state_revision: u64", source)
        self.assertNotIn("pub backend_epoch: u64", source)

    def test_tauri_commands_publish_every_returned_or_emitted_state(self) -> None:
        source = COMMANDS.read_text(encoding="utf-8")

        for anchor in (
            "pub(super) fn hub_state(",
            "Ok(session.publish_view_model())",
            "pub(super) fn hub_action(",
            "let view_model = session.publish_view_model();",
            "let view_model = session.apply_action(request)?;",
            'app.emit("hub-state-changed", &view_model)',
            'app.emit("hub-state-changed", view_model)',
            "should_emit.then(|| session.publish_view_model())",
        ):
            self.assertIn(anchor, source)

        hub_state = source.split("pub(super) fn hub_state", 1)[1].split("pub(super) fn hub_action", 1)[0]
        self.assertNotIn("session.view_model()", hub_state)
        action = source.split("pub(super) fn hub_action", 1)[1]
        self.assertLess(action.index("session.publish_view_model()"), action.index("app.emit"))

    def test_background_worker_never_emits_a_nonpublished_snapshot(self) -> None:
        source = TASKS.read_text(encoding="utf-8")

        self.assertNotIn("session.view_model()", source)
        self.assertGreaterEqual(source.count("session.publish_view_model()"), 8)
        self.assertGreaterEqual(source.count("emit_state(&view_model)"), 8)
        for anchor in (
            "let view_model = session.publish_view_model();",
            "drop(session);",
            "emit_state(&view_model);",
            "let started_view_model = next_request.as_ref().map(|_| session.publish_view_model());",
            "session.record_background_worker_panic(&request, &detail);",
        ):
            self.assertIn(anchor, source)

        # Every owned model is released from the mutex before crossing into the emitter.
        for block in source.split("emit_state(&view_model);")[:-1]:
            tail = block[-360:]
            self.assertTrue(
                "drop(session);" in tail or "started_view_model" in tail and "};" in tail,
                "an emitted model must leave the session lock before emission",
            )

    def test_rust_chronology_regressions_cover_epoch_revision_and_reads(self) -> None:
        source = RUNTIME_TESTS.read_text(encoding="utf-8")

        for anchor in (
            "published_view_models_share_an_epoch_and_advance_lossless_revisions",
            "separate_runtime_sessions_use_distinct_backend_epochs",
            "let repeated_read = session.view_model();",
            'assert_eq!(first.state_revision, "1");',
            'assert_eq!(repeated_read.state_revision, first.state_revision);',
            'assert_eq!(second.state_revision, "2");',
            "assert_ne!(first.backend_epoch, second.backend_epoch);",
        ):
            self.assertIn(anchor, source)


if __name__ == "__main__":
    unittest.main()
