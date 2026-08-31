import pathlib
import unittest


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
LOADER_ROOT = REPO_ROOT / "zircon_runtime/src/plugin/native_plugin_loader"


def _section(source: str, start: str, end: str) -> str:
    return source.split(start, 1)[1].split(end, 1)[0]


class Runtime11NativePluginDiscoveryContractTests(unittest.TestCase):
    def test_canonical_namespace_exposes_prepared_root_ticket_and_snapshot(self) -> None:
        source = (REPO_ROOT / "zircon_runtime/src/plugin/native/discovery.rs").read_text(
            encoding="utf-8"
        )
        for symbol in (
            "resolve_native_plugin_discovery_root",
            "request_native_plugin_discovery_refresh",
            "latest_native_plugin_discovery_snapshot",
            "NativePluginDiscoveryRefreshTicket",
            "NativePluginDiscoverySnapshot",
        ):
            self.assertIn(symbol, source)

    def test_request_facade_accepts_prepared_root_without_filesystem_or_wait(self) -> None:
        source = (LOADER_ROOT / "discover.rs").read_text(encoding="utf-8")
        request = _section(
            source,
            "pub fn request_discovery_refresh",
            "pub fn latest_discovery_snapshot",
        )
        self.assertIn("root: &NativePluginDiscoveryRoot", request)
        for forbidden in (
            "AsRef<Path>",
            "root_identity",
            "canonicalize",
            "wait_terminal",
            "load_candidates",
            "NativePluginLoadReport",
        ):
            self.assertNotIn(forbidden, request)

    def test_latest_snapshot_is_a_read_only_prepared_root_projection(self) -> None:
        source = (LOADER_ROOT / "discover/authority.rs").read_text(encoding="utf-8")
        latest = _section(source, "pub(super) fn latest_snapshot", "pub(in crate::")
        self.assertIn("root: &NativePluginDiscoveryRoot", latest)
        self.assertIn("self.refresh.snapshot(root)", latest)
        for forbidden in ("root_identity", "canonicalize", "submit", "ticket_for"):
            self.assertNotIn(forbidden, latest)

    def test_root_resolution_is_an_explicit_setup_operation(self) -> None:
        facade = (LOADER_ROOT / "discover.rs").read_text(encoding="utf-8")
        resolver = _section(
            facade,
            "pub fn resolve_discovery_root",
            "pub fn request_discovery_refresh",
        )
        identity = (LOADER_ROOT / "discovery_refresh/mod.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("discovery_authority().resolve_root", resolver)
        self.assertIn("root resolution may query the filesystem", facade.lower())
        self.assertIn("canonicalize()", identity)

    def test_async_admission_reclaims_terminal_authority_entries(self) -> None:
        source = (LOADER_ROOT / "discover/authority.rs").read_text(encoding="utf-8")
        admission = _section(source, "fn ticket_for", "fn clear_terminal_ticket")
        self.assertIn(
            "in_flight.retain(|_, existing| !existing.ticket.is_complete())",
            admission,
        )

    def test_blocking_wait_stays_out_of_the_public_ticket_surface(self) -> None:
        source = (LOADER_ROOT / "discovery_refresh/ticket.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("pub(crate) fn wait_terminal", source)
        self.assertNotIn("pub fn wait_terminal", source)


if __name__ == "__main__":
    unittest.main()
