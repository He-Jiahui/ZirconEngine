from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
LEDGER_RS = ROOT / "zircon_editor/src/core/jobs/system/admission_ledger.rs"
RESERVATION_TESTS_RS = ROOT / (
    "zircon_editor/src/core/jobs/tests/admission_scaling_contract/reservation.rs"
)


def source(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def compact(text: str) -> str:
    return re.sub(r"\s+", "", text)


def function_body(text: str, signature: str, next_signature: str) -> str:
    return text.split(signature, 1)[1].split(next_signature, 1)[0]


class Editor09BorrowedBatchAdmissionPerformanceContract(unittest.TestCase):
    def test_reservation_preflight_borrows_requests_without_a_temporary_vec(self) -> None:
        text = source(LEDGER_RS)
        body = compact(function_body(text, "pub(super) fn reserve_batch", "pub(super) fn ensure_reservation_batch_admissible"))

        self.assertNotIn("collect::<Vec", body)
        self.assertIn(
            "ensure_reservation_batch_admissible_iter("
            "reservations.iter().map(|reservation|&reservation.request),",
            body,
        )

    def test_slice_callers_share_the_borrowed_iterator_preflight(self) -> None:
        text = compact(source(LEDGER_RS))

        self.assertIn(
            "fnensure_reservation_batch_admissible_iter<'a>("
            "&self,requests:implClone+ExactSizeIterator<"
            "Item=&'aEditorJobAdmissionRequest>,",
            text,
        )
        self.assertIn(
            "ensure_reservation_batch_admissible_iter(requests.iter().copied(),limits,now)",
            text,
        )

    def test_existing_batch_admission_behavior_oracles_remain_present(self) -> None:
        text = source(RESERVATION_TESTS_RS)

        for test_name in (
            "batch_admission_rejects_atomically_without_retaining_partial_tickets",
            "batch_admission_reservation_holds_capacity_until_commit_or_drop",
            "dropped_batch_admission_reservation_releases_its_reserved_bytes",
            "shutdown_releases_uncommitted_batch_admission_reservations",
        ):
            self.assertIn(f"fn {test_name}()", text)


if __name__ == "__main__":
    unittest.main()
