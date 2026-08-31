"""Static guard for Runtime87 stable-identity reference resolution."""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
RESOLVER = ROOT / "zircon_runtime/src/asset/reference_resolver.rs"
ERRORS = ROOT / "zircon_runtime/src/asset/reference_resolution_error.rs"
FULL = ROOT / "zircon_runtime/src/asset/project/manager/scan_and_import/full_generation.rs"
TARGETED = ROOT / "zircon_runtime/src/asset/project/manager/scan_and_import/targeted.rs"
CATALOG = ROOT / "zircon_runtime/src/asset/project/catalog_input_generation.rs"
EDITOR_PROJECTION = (
    ROOT
    / "zircon_editor/src/ui/host/editor_asset_manager/manager/project_sync/record_projection.rs"
)
REGISTRY_QUERY = ROOT / "zircon_runtime/src/asset/registry/query.rs"
NATIVE_IMPORTER = ROOT / "zircon_runtime/src/asset/importer/native.rs"
IMPORTER_DOCUMENTATION = ROOT / "docs/zircon_runtime/asset/importer.md"
MUTATION_DELETE_PREFLIGHT = ROOT / "zircon_runtime/src/asset/mutation/delete_preflight.rs"
MUTATION_RELOCATION_PREFLIGHT = (
    ROOT / "zircon_runtime/src/asset/mutation/relocation_preflight.rs"
)
RESOURCE_RECONCILIATION = (
    ROOT / "zircon_runtime/src/asset/pipeline/manager/resource_sync/reconcile_project_resources.rs"
)
RESOURCE_REGISTRY = ROOT / "zircon_runtime/src/core/resource/registry.rs"
ASSET_REGISTRY_RELOCATION = ROOT / "zircon_runtime/src/asset/registry/relocation.rs"
PROJECT_RELOCATION = ROOT / "zircon_runtime/src/asset/project/manager/relocation.rs"
PIPELINE_RELOCATION = (
    ROOT / "zircon_runtime/src/asset/pipeline/manager/project_asset_manager/relocation.rs"
)
EDITOR_ASSET_API = ROOT / "zircon_editor/src/ui/host/editor_asset_manager/api.rs"
EDITOR_ASSET_RELOCATION = (
    ROOT
    / "zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/project_relocation.rs"
)
EDITOR_ASSET_TRAIT_BRIDGE = (
    ROOT
    / "zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/preview_trait_bridge.rs"
)


class Runtime87ReferenceResolutionContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.resolver = RESOLVER.read_text(encoding="utf-8")
        cls.errors = ERRORS.read_text(encoding="utf-8")
        cls.full = FULL.read_text(encoding="utf-8")
        cls.targeted = TARGETED.read_text(encoding="utf-8")
        cls.catalog = CATALOG.read_text(encoding="utf-8")
        cls.editor_projection = EDITOR_PROJECTION.read_text(encoding="utf-8")
        cls.registry_query = REGISTRY_QUERY.read_text(encoding="utf-8")
        cls.native_importer = NATIVE_IMPORTER.read_text(encoding="utf-8")
        cls.importer_documentation = IMPORTER_DOCUMENTATION.read_text(encoding="utf-8")
        cls.resource_registry = RESOURCE_REGISTRY.read_text(encoding="utf-8")
        cls.asset_registry_relocation = ASSET_REGISTRY_RELOCATION.read_text(encoding="utf-8")
        cls.project_relocation = PROJECT_RELOCATION.read_text(encoding="utf-8")
        cls.pipeline_relocation = PIPELINE_RELOCATION.read_text(encoding="utf-8")
        cls.editor_asset_api = EDITOR_ASSET_API.read_text(encoding="utf-8")
        cls.editor_asset_relocation = EDITOR_ASSET_RELOCATION.read_text(encoding="utf-8")
        cls.editor_asset_trait_bridge = EDITOR_ASSET_TRAIT_BRIDGE.read_text(
            encoding="utf-8"
        )

    def test_missing_guid_path_candidate_is_explicit(self):
        self.assertIn("PathOccupiedCandidate", self.errors)
        self.assertIn("candidate_uuid: AssetUuid", self.errors)
        self.assertIn("candidate_path: ResourceLocator", self.errors)
        self.assertIn("ReferenceResolutionError::PathOccupiedCandidate", self.resolver)

    def test_resolver_has_no_semantic_guid_or_subasset_repair(self):
        self.assertNotIn("ReferenceRepairKind::Guid", self.resolver)
        self.assertNotIn("ReferenceRepairKind::Subasset", self.resolver)
        self.assertIn("kind: ReferenceRepairKind::PathHint", self.resolver)

    def test_guid_subasset_mismatch_is_a_conflict(self):
        self.assertIn("if entry.path().label() != reference.sub()", self.resolver)
        self.assertIn("ReferenceResolutionError::Conflict", self.resolver)
        self.assertIn("must never replace a stable GUID", self.resolver)

    def test_registry_reference_lookup_does_not_fall_back_to_path(self):
        self.assertNotIn(
            ".or_else(|_| self.resolve_asset_id_by_path(path))", self.registry_query
        )
        self.assertIn(
            "self.resolve_asset_id_by_uuid(uuid).map_err", self.registry_query
        )

    def test_full_and_targeted_generations_publish_repair_observations(self):
        self.assertIn("let reference_repairs = outcome.reference_repairs.clone();", self.full)
        self.assertIn("reference_repairs,", self.full)
        self.assertIn("let reference_repairs = outcome.reference_repairs.clone();", self.targeted)
        self.assertIn("root_direct_references,\n            reference_repairs,", self.targeted)

    def test_native_import_protocol_carries_required_repair_observations(self):
        self.assertIn('const RESPONSE_MAGIC: &[u8] = b"ZRIMO002\\n";', self.native_importer)
        self.assertIn("pub reference_repairs: Vec<crate::asset::ReferenceRepair>", self.native_importer)
        self.assertIn("reference_repairs: response.reference_repairs", self.native_importer)
        self.assertNotIn("#[serde(default)]\n    pub entries", self.native_importer)

    def test_current_importer_documentation_matches_the_hard_cut_protocol(self):
        self.assertIn("`ZRIMO002` response envelope", self.importer_documentation)
        self.assertIn("required typed `reference_repairs` list", self.importer_documentation)
        self.assertNotIn("`ZRIMO001` response envelope", self.importer_documentation)

    def test_current_importer_documentation_has_no_subasset_base_fallback(self):
        self.assertIn("`PathOccupiedCandidate`", self.importer_documentation)
        self.assertIn("GUID/subasset mismatch is `Conflict`", self.importer_documentation)
        self.assertNotIn("falls back to the canonical base asset", self.importer_documentation)

    def test_runtime_mutation_owns_delete_topology_preflight(self):
        source = MUTATION_DELETE_PREFLIGHT.read_text(encoding="utf-8")

        self.assertIn("pub struct AssetMutationAsset", source)
        self.assertIn("pub enum AssetMutationDeleteDisposition", source)
        self.assertIn("get_referencers_by_uuid", source)
        self.assertIn("UnsupportedSubasset", source)
        self.assertIn("referencers.sort_by", source)
        self.assertNotIn("left.uuid.cmp", source)

    def test_runtime_mutation_relocation_preflight_owns_companions_and_closure(self):
        source = MUTATION_RELOCATION_PREFLIGHT.read_text(encoding="utf-8")

        self.assertIn("pub struct AssetMutationRelocationPreflight", source)
        self.assertIn("pub enum AssetMutationRelocationDisposition", source)
        self.assertIn("source_entries", source)
        self.assertIn("TargetOccupied", source)
        self.assertIn("referencer_closure", source)
        self.assertIn("ResourceScheme::Res", source)
        self.assertNotIn("uuid().cmp", source)

    def test_generation_reconciliation_uses_explicit_resource_renames(self):
        source = RESOURCE_RECONCILIATION.read_text(encoding="utf-8")

        self.assertIn("pub(in crate::asset::pipeline::manager) fn reconcile_project_resources", source)
        self.assertIn("struct ProjectResourceIdentity", source)
        self.assertIn("project_resource_identities", source)
        self.assertIn("previous_identities", source)
        self.assertIn("candidate.registry().get", source)
        self.assertIn("batch.rename", source)
        self.assertIn("batch.remove", source)

    def test_offline_registry_staging_requires_an_explicit_rename(self):
        source = self.resource_registry

        self.assertIn("fn stage_rename_locator", source)
        self.assertIn("MissingRecordForLocator", source)
        self.assertIn("LocatorOccupied", source)
        self.assertIn("authorized_locator", source)

    def test_asset_registry_relocation_preserves_identity_and_retargets_dependents(self):
        source = self.asset_registry_relocation

        self.assertIn("pub(crate) fn prepare_source_relocation_generation", source)
        self.assertIn("source_entries(&source)", source)
        self.assertIn("replace_dependency_paths", source)
        self.assertIn("refresh_dependency_owners", source)
        self.assertIn("source_digest()", source)

    def test_project_manager_relocation_uses_one_durable_generation(self):
        source = self.project_relocation

        self.assertIn("pub(crate) fn prepare_project_source_relocation", source)
        self.assertNotIn("pub(crate) fn relocate_project_source(", source)
        self.assertIn("prepare_source_relocation_generation", source)
        self.assertIn("PreparedFileWrite::new(target_path", source)
        self.assertIn(".retiring_with_expected_digest(source_path", source)
        self.assertIn(".retiring(source_meta_path.clone())", source)
        self.assertIn("stage_rename_locator", source)
        self.assertIn("ProjectCatalogInputGeneration::publish_targeted", source)
        self.assertIn("commit_prepared_files", source)

    def test_pipeline_owns_the_public_relocation_commit_and_resource_rename(self):
        source = self.pipeline_relocation

        self.assertIn("pub fn relocate_project_source", source)
        self.assertIn("prepare_project_source_relocation", source)
        self.assertIn("prepare_project_source_relocation_resource_sync", source)
        self.assertIn("project_generation_write", source)
        self.assertIn("AssetChangeKind::Renamed", source)

    def test_editor_relocation_gateway_only_delegates_to_runtime_then_refreshes_projection(self):
        self.assertIn("fn relocate_project_source", self.editor_asset_api)
        self.assertIn("asset_manager.relocate_project_source", self.editor_asset_relocation)
        self.assertIn("self.refresh_from_runtime_project()?", self.editor_asset_relocation)
        self.assertNotIn("std::fs", self.editor_asset_relocation)
        self.assertIn("parse_uuid(uuid)?", self.editor_asset_trait_bridge)
        self.assertIn("relocate_project_source(self", self.editor_asset_trait_bridge)

    def test_catalog_and_editor_share_the_same_repair_observation(self):
        self.assertIn("reference_repairs: Arc<[ReferenceRepair]>", self.catalog)
        self.assertIn("pub fn reference_repairs(&self) -> &[ReferenceRepair]", self.catalog)
        self.assertIn("catalog_input\n            .reference_repairs()", self.editor_projection)
        self.assertIn("reference path hint requires fix-up", self.editor_projection)


if __name__ == "__main__":
    unittest.main()
