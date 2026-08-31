import pathlib
import unittest


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
RESOLUTION = REPO_ROOT / (
    "zircon_runtime/src/core/framework/render/environment/"
    "ibl_bake_artifact_resolution.rs"
)
DISPATCH = REPO_ROOT / (
    "zircon_runtime/src/asset/artifact/ibl_bake_artifact_runtime_dispatch.rs"
)


def production_source(path: pathlib.Path) -> str:
    return path.read_text(encoding="utf-8").split("#[cfg(test)]", 1)[0]


class BorrowedIblArtifactDispatchPerformanceContract(unittest.TestCase):
    def test_resolution_exposes_borrowed_blob_source_path(self) -> None:
        source = production_source(RESOLUTION)
        self.assertIn("fn resolve_borrowed_blob_sources(", source)
        self.assertIn("asset_derived_blobs: &[IblBakeArtifactBlob]", source)
        self.assertIn("runtime_cache_blob: Option<&IblBakeArtifactBlob>", source)

    def test_borrowed_path_projects_only_descriptors_before_selection(self) -> None:
        source = production_source(RESOLUTION)
        self.assertIn("fn resolve_borrowed_blob_sources(", source)
        body = source.split("fn resolve_borrowed_blob_sources(", 1)[1]
        self.assertIn("blob.descriptor()", body)
        self.assertIn("select_ibl_bake_artifact(request", body)
        self.assertNotIn("asset_derived_blobs.iter().cloned()", body)

    def test_runtime_dispatch_does_not_clone_candidate_blobs(self) -> None:
        source = production_source(DISPATCH)
        body = source.split("pub fn resolve_ibl_bake_artifact_runtime_dispatch(", 1)[1]
        body = body.split("pub enum IblBakeArtifactRuntimeDispatchReadbackStatus", 1)[0]
        self.assertIn("resolve_borrowed_blob_sources", body)
        self.assertIn("IblBakeArtifactCacheRead::Missing", body)
        self.assertIn("IblBakeArtifactCacheRead::Rejected(_)", body)
        self.assertNotIn("IblBakeArtifactBlobCandidate", body)
        self.assertNotIn(".iter()\n        .cloned()", body)
        self.assertNotIn("blob.clone()", body)


if __name__ == "__main__":
    unittest.main()
