import unittest
from pathlib import Path


class RuntimeAssetEventOwnerStructureTests(unittest.TestCase):
    def setUp(self) -> None:
        self.repo_root = Path(__file__).resolve().parents[2]
        self.owner = self.repo_root / "zircon_runtime/src/asset/facade/event.rs"
        self.owner_dir = self.owner.with_suffix("")

    def test_asset_event_contract_uses_focused_folder_backed_owners(self) -> None:
        owner_source = self.owner.read_text(encoding="utf-8")
        production_lines = [
            line
            for line in owner_source.splitlines()
            if line.strip() and not line.lstrip().startswith("//")
        ]

        self.assertLessEqual(len(production_lines), 20)
        for declaration in (
            "mod declaration;",
            "mod projection;",
            "mod receiver;",
            '#[cfg(test)]\nmod tests;',
        ):
            self.assertIn(declaration, owner_source)

        for public_reexport in (
            "pub use declaration::{AssetEvent, AssetEventKind};",
            "pub use receiver::AssetEventReceiver;",
            "pub(crate) use receiver::{typed_event_receiver, AssetEventPoll};",
        ):
            self.assertIn(public_reexport, owner_source)

        expected_children = {
            "declaration.rs": (
                "pub enum AssetEventKind",
                "pub enum AssetEvent<TAsset: Asset>",
            ),
            "projection.rs": (
                "impl<TAsset: Asset> AssetEvent<TAsset>",
                "pub fn from_resource_event",
                "pub fn previous_locator",
            ),
            "receiver.rs": (
                "pub(crate) enum AssetEventPoll",
                "pub struct AssetEventReceiver",
                "pub(crate) fn typed_event_receiver",
            ),
            "tests.rs": (
                "typed_asset_events_roundtrip_for_tooling_snapshots",
                "typed_asset_receiver_skips_other_resource_kinds_without_a_filter_thread",
            ),
        }
        for child_name, anchors in expected_children.items():
            child = self.owner_dir / child_name
            self.assertTrue(child.is_file(), child)
            child_source = child.read_text(encoding="utf-8")
            for anchor in anchors:
                self.assertIn(anchor, child_source)

        for forbidden in (
            "pub enum AssetEventKind",
            "pub enum AssetEvent<TAsset: Asset>",
            "pub struct AssetEventReceiver",
            "impl<TAsset: Asset> AssetEvent<TAsset>",
        ):
            self.assertNotIn(forbidden, owner_source)

        facade_source = (self.owner.parent / "mod.rs").read_text(encoding="utf-8")
        self.assertIn(
            "pub(crate) use event::{typed_event_receiver, AssetEventPoll};",
            facade_source,
        )
        self.assertIn(
            "pub use event::{AssetEvent, AssetEventKind, AssetEventReceiver};",
            facade_source,
        )


if __name__ == "__main__":
    unittest.main()
