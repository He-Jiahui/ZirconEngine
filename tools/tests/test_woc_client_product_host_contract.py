import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
CLIENT = ROOT / "examples" / "woc" / "native" / "apps" / "woc_client"


class WocClientProductHostContractTests(unittest.TestCase):
    def test_client_binary_enters_engine_owned_host(self):
        main = (CLIENT / "src" / "main.rs").read_text(encoding="utf-8")
        self.assertIn("ClientProductHost", main)
        self.assertNotIn("identity_report_json", main)

        host = (CLIENT / "src" / "application.rs").read_text(encoding="utf-8")
        self.assertIn("load_engine_project_vm", host)
        self.assertIn("vm.activate()", host)
        self.assertIn("vm.deactivate()", host)
        self.assertIn("new_with_authority_recovery", host)
        self.assertIn("into_authority", host)

    def test_client_manifest_enables_real_engine_host_by_default(self):
        manifest = (CLIENT / "Cargo.toml").read_text(encoding="utf-8")
        self.assertIn('[features]\ndefault = ["engine-host"]', manifest)
        self.assertIn('engine-host = ["woc_runtime/engine-host"]', manifest)
        self.assertIn('required-features = ["engine-host"]', manifest)

    def test_client_defaults_to_the_authored_woc_game_project(self):
        options = (CLIENT / "src" / "main.rs").read_text(encoding="utf-8")
        # The engine-host loader receives the project root, which owns
        # zircon-project.toml; it resolves scripts/woc_game itself.
        self.assertIn('join("../../..")', options)
        self.assertNotIn('join("../../../scripts/woc_game")', options)

    def test_state_only_path_is_explicitly_non_visual(self):
        authority = (CLIENT / "src" / "presentation" / "authority.rs").read_text(
            encoding="utf-8"
        )
        host = (CLIENT / "src" / "application.rs").read_text(encoding="utf-8")
        self.assertIn("StateOnlyClientAuthority", authority)
        self.assertIn("ClientStateProjection", authority)
        state_only_start = authority.index(
            "impl<V> ClientAuthority<ClientStateProjection>"
        )
        state_only = authority[state_only_start:]
        self.assertIn("PresentationSnapshot<ClientStateProjection>", state_only)
        self.assertIn("tick_with_movement", state_only)
        self.assertNotIn("ClientPresentationProjection::decode_json", state_only)
        self.assertIn('"stateOnly": false', host)
        self.assertIn(
            '"presentationReady": metrics.sampled_presentation_frames > 0', host
        )
        self.assertIn('"nativeWindowReady": false', host)
        self.assertIn('"renderedFrames": 0', host)

    def test_offline_preparation_has_an_explicit_retry_rollback(self):
        offline = (CLIENT / "src" / "shell" / "offline_flow.rs").read_text(
            encoding="utf-8"
        )
        shell = (CLIENT / "src" / "shell" / "woc_shell.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("RejectPreparedSession", offline)
        self.assertIn("reject_prepared_session", offline)
        self.assertIn("reject_offline_preparation", shell)
        self.assertIn("OfflineShellState::OfflinePicker", offline)

    def test_auth_credentials_use_a_redacted_zeroing_wrapper(self):
        auth = (CLIENT / "src" / "shell" / "auth_flow.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("pub struct AuthSecret", auth)
        self.assertIn("impl Drop for AuthSecret", auth)
        self.assertIn("impl fmt::Debug for AuthSecret", auth)
        self.assertIn("impl fmt::Display for AuthSecret", auth)
        self.assertIn("*byte = 0", auth)
        self.assertIn("InvalidResetToken", auth)

    def test_realm_probe_updates_are_generation_bound(self):
        realms = (CLIENT / "src" / "shell" / "realm_directory.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("generation(&self)", realms)
        self.assertIn("set_status_for_generation", realms)
        self.assertIn("finish_status_refresh_for_generation", realms)
        self.assertIn("StaleGeneration", realms)

    def test_input_and_clock_boundaries_fail_closed(self):
        math = (CLIENT / "src" / "input" / "gamepad" / "math.rs").read_text(
            encoding="utf-8"
        )
        frame = (CLIENT / "src" / "presentation" / "frame_driver.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("valid_deadzone", math)
        self.assertIn("is_finite", math)
        self.assertIn("MAX_FRAME_DELTA_NS", frame)
        self.assertIn("ElapsedTooLarge", frame)


if __name__ == "__main__":
    unittest.main()
