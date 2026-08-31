from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
EXTENSION_VIEWS = ROOT / "zircon_editor/src/ui/host/editor_extension_views.rs"


def function_region(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class Editor06BorrowedExtensionViewValidationPerformanceContractTests(
    unittest.TestCase
):
    def test_pending_descriptor_ids_are_borrowed_from_the_candidate_slice(self) -> None:
        source = EXTENSION_VIEWS.read_text(encoding="utf-8")
        validation = function_region(
            source,
            "fn validate_extension_view_descriptors(",
            "#[cfg(test)]",
        )
        compact = " ".join(validation.split())

        self.assertIn(
            "let mut pending = "
            "std::collections::HashSet::<&ViewDescriptorId>::with_capacity(views.len());",
            compact,
        )
        self.assertIn("pending.insert(&view.descriptor_id)", validation)
        self.assertNotIn("view.descriptor_id.clone()", validation)

    def test_registry_check_still_precedes_batch_duplicate_admission(self) -> None:
        source = EXTENSION_VIEWS.read_text(encoding="utf-8")
        validation = function_region(
            source,
            "fn validate_extension_view_descriptors(",
            "#[cfg(test)]",
        )

        registered = validation.index("registry.descriptor(&view.descriptor_id).is_some()")
        pending = validation.index("pending.insert(&view.descriptor_id)")
        self.assertLess(registered, pending)

    def test_unique_and_duplicate_batches_are_covered_by_rust(self) -> None:
        source = EXTENSION_VIEWS.read_text(encoding="utf-8")

        self.assertIn("fn borrowed_view_validation_accepts_unique_ids()", source)
        self.assertIn("fn borrowed_view_validation_rejects_a_batch_duplicate()", source)


if __name__ == "__main__":
    unittest.main()
