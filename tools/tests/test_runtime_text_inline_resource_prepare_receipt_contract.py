import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
UI_TEXTURE = ROOT / "zircon_runtime/src/graphics/scene/resources/ui_texture.rs"
RECEIPT = (
    ROOT
    / "zircon_runtime/src/graphics/scene/resources/ui_texture/prepare_receipt.rs"
)
STREAMER = (
    ROOT
    / "zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer.rs"
)
RESOURCES_MOD = ROOT / "zircon_runtime/src/graphics/scene/resources/mod.rs"
SCENE_PREPARE = (
    ROOT
    / "zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs"
)
TEXTURE_PREPARE = (
    ROOT
    / "zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_texture.rs"
)
IMAGE = ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/image.rs"


class RuntimeTextInlineResourcePrepareReceiptContractTests(unittest.TestCase):
    def test_prepare_receipt_owns_typed_outcomes_and_exact_generations(self):
        source = RECEIPT.read_text(encoding="utf-8")

        for token in (
            "enum UiTexturePrepareOutcome",
            "UnresolvedIdentity",
            "NotReady",
            "LoadFailed",
            "InvalidResourceKind",
            "InvalidDescriptor",
            "GenerationChanged",
            "UploadFailed",
            "Ready",
            "ResourceManagementGenerationIdentity",
            "ResourceReadinessGenerationIdentity",
            "prepared_revision: Option<u64>",
            "ready_texture_id",
        ):
            self.assertIn(token, source)

    def test_scene_prepare_publishes_receipt_instead_of_discarding_option_result(self):
        ui_texture = UI_TEXTURE.read_text(encoding="utf-8")
        scene_prepare = SCENE_PREPARE.read_text(encoding="utf-8")

        self.assertNotIn("fn ui_texture_id_for_upload", ui_texture)
        self.assertNotIn("ui_texture_id_for_upload", scene_prepare)
        self.assertNotIn(
            "ui_texture_id_for_upload",
            RESOURCES_MOD.read_text(encoding="utf-8"),
        )
        self.assertIn("prepare_ui_textures_for_frame", scene_prepare)
        self.assertIn("last_ui_texture_prepare_receipt", STREAMER.read_text(encoding="utf-8"))

    def test_ui_prepare_uses_one_atomic_snapshot_and_reuses_snapshot_for_gpu_prepare(self):
        receipt = RECEIPT.read_text(encoding="utf-8")
        texture_prepare = TEXTURE_PREPARE.read_text(encoding="utf-8")

        self.assertEqual(receipt.count("load_texture_asset_snapshot("), 1)
        self.assertNotIn("load_texture_asset(", receipt)
        self.assertIn("ensure_texture_snapshot_for_frame", receipt)
        self.assertIn("ResourceSnapshot<TextureAsset>", texture_prepare)
        self.assertIn(
            "TextureSnapshotFramePrepareError::GpuArtifact) =>",
            receipt,
        )
        self.assertIn(
            "TextureSnapshotFramePrepareError::Submission(error)) => return Err(error)",
            receipt,
        )
        self.assertNotIn("Err(_) => UiTexturePrepareOutcome::UploadFailed", receipt)

    def test_image_binding_consumes_ready_receipt_without_re_resolving_it(self):
        source = IMAGE.read_text(encoding="utf-8")

        self.assertIn("last_ui_texture_prepare_receipt", source)
        self.assertIn("prepared_ui_texture_id", source)

    def test_prepare_accepts_only_the_owned_distinct_dependency_set(self):
        ui_texture = UI_TEXTURE.read_text(encoding="utf-8")
        receipt = RECEIPT.read_text(encoding="utf-8")

        self.assertIn("struct UiTextureDependencies", ui_texture)
        self.assertIn(") -> UiTextureDependencies", ui_texture)
        self.assertIn("requested_ids: &UiTextureDependencies", receipt)
        self.assertNotIn("requested_ids: &[ResourceId]", receipt)
        self.assertIn(
            "pub(in crate::graphics::scene) struct UiTexturePrepareReceipt",
            receipt,
        )
        self.assertIn("pub(super) enum UiTexturePrepareOutcome", receipt)
        self.assertIn("pub(super) struct UiTexturePrepareRow", receipt)
        self.assertIn("pub(super) fn new", receipt)

    def test_prepare_profiles_fixed_low_cardinality_outcome_counts(self):
        source = RECEIPT.read_text(encoding="utf-8")

        for counter in (
            "ui.ui_texture_prepare.requested_count",
            "ui.ui_texture_prepare.ready_count",
            "ui.ui_texture_prepare.unresolved_count",
            "ui.ui_texture_prepare.not_ready_count",
            "ui.ui_texture_prepare.load_failed_count",
            "ui.ui_texture_prepare.invalid_descriptor_count",
            "ui.ui_texture_prepare.upload_failed_count",
            "ui.ui_texture_prepare.resolution_scan_row_visit_count",
            "ui.ui_texture_prepare.snapshot_load_count",
            "ui.ui_texture_prepare.prepared_reuse_count",
            "ui.ui_texture_prepare.upload_attempt_count",
        ):
            self.assertIn(counter, source)

    def test_touched_production_owners_remain_bounded(self):
        for path in (
            UI_TEXTURE,
            RECEIPT,
            RESOURCES_MOD,
            STREAMER,
            SCENE_PREPARE,
            TEXTURE_PREPARE,
            IMAGE,
        ):
            with self.subTest(path=path):
                self.assertLessEqual(
                    len(path.read_text(encoding="utf-8").splitlines()),
                    800,
                    f"{path} must stay within the production owner budget",
                )


if __name__ == "__main__":
    unittest.main()
