from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
STATUS_RS = ROOT / "zircon_plugins/sound/runtime/src/ray_tracing/status.rs"
PROVIDER_RS = ROOT / "zircon_plugins/sound/runtime/src/ray_tracing/provider.rs"
IMPULSE_RS = ROOT / "zircon_plugins/sound/runtime/src/service_types/impulse_responses.rs"


class SoundImpulseResponseBorrowContractTests(unittest.TestCase):
    def test_state_refresh_is_centralized_for_disjoint_field_borrows(self) -> None:
        status = STATUS_RS.read_text(encoding="utf-8")
        provider = PROVIDER_RS.read_text(encoding="utf-8")
        impulse = IMPULSE_RS.read_text(encoding="utf-8")

        self.assertIn("pub(crate) fn refresh_ray_tracing_status_for_state", status)
        self.assertIn("refresh_ray_tracing_status_for_state(state)", provider)
        self.assertIn("refresh_ray_tracing_status_for_state(&mut *state)", impulse)
        self.assertNotIn(
            "refresh_ray_tracing_status(&mut state.ray_tracing, &state.ray_traced_impulse_responses)",
            provider + impulse,
        )


if __name__ == "__main__":
    unittest.main()
