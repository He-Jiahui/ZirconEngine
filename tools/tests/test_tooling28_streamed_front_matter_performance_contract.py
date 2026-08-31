import tempfile
import time
import unittest
from pathlib import Path

from tools.check_conventions import FRONT_MATTER_LIST_FIELDS, _front_matter_path_fields


TOOLING28_WARMUP_PAIRS = 4
TOOLING28_SAMPLE_PAIRS = 21


def _legacy_front_matter_path_fields(document: Path) -> dict[str, list[str]] | None:
    lines = document.read_text(encoding="utf-8-sig").splitlines()
    if not lines or lines[0].strip() != "---":
        return None

    fields = {field: [] for field in FRONT_MATTER_LIST_FIELDS}
    active_field: str | None = None
    for line in lines[1:]:
        if line.strip() == "---":
            return fields
        if line and not line[0].isspace():
            key, separator, _ = line.partition(":")
            active_field = key if separator and key in fields else None
            continue
        stripped = line.strip()
        if active_field is None or not stripped.startswith("- "):
            continue
        value = stripped[2:].strip().strip("'\"").strip("`")
        if value:
            fields[active_field].append(value)
    return None


def _nearest_rank(samples: list[int], percentile: int) -> int:
    ordered = sorted(samples)
    rank = (len(ordered) * percentile + 99) // 100
    return ordered[max(rank - 1, 0)]


def _measure_parser(document: Path, parser) -> tuple[int, int]:
    started = time.perf_counter_ns()
    parsed = parser(document)
    elapsed_ns = max(time.perf_counter_ns() - started, 1)
    assert parsed is not None
    checksum = sum(len(value) for values in parsed.values() for value in values)
    return elapsed_ns, checksum


class _GuardedDocumentStream:
    def __init__(self, lines: tuple[str, ...], read_limit: int):
        self._lines = lines
        self._read_limit = read_limit
        self._index = 0

    def __enter__(self):
        return self

    def __exit__(self, _exception_type, _exception, _traceback) -> None:
        return None

    def __iter__(self):
        return self

    def __next__(self) -> str:
        if self._index >= self._read_limit:
            raise AssertionError("front matter parser read document body")
        if self._index >= len(self._lines):
            raise StopIteration
        line = self._lines[self._index]
        self._index += 1
        return line


class _GuardedDocument:
    def __init__(self, lines: tuple[str, ...], read_limit: int):
        self._lines = lines
        self._read_limit = read_limit

    def read_text(self, **_kwargs) -> str:
        raise AssertionError("front matter parser materialized the full document")

    def open(self, mode: str, *, encoding: str):
        if mode != "r" or encoding != "utf-8-sig":
            raise AssertionError("front matter parser changed its text decoding contract")
        return _GuardedDocumentStream(self._lines, self._read_limit)


class StreamedFrontMatterPerformanceContract(unittest.TestCase):
    def test_parser_stops_at_the_closing_front_matter_delimiter(self) -> None:
        document = _GuardedDocument(
            (
                "---\n",
                "implementation_files:\n",
                "  - tools/check_conventions.py\n",
                "tests:\n",
                "  - tools/tests/test_check_conventions.py::DocumentAuditTests\n",
                "---\n",
                "# Document body must stay unread.\n",
            ),
            read_limit=6,
        )

        fields = _front_matter_path_fields(document)

        self.assertEqual(
            {
                "implementation_files": ["tools/check_conventions.py"],
                "related_code": [],
                "tests": [
                    "tools/tests/test_check_conventions.py::DocumentAuditTests"
                ],
            },
            fields,
        )

    def test_non_front_matter_document_reads_only_the_first_line(self) -> None:
        document = _GuardedDocument(
            ("# Ordinary document\n", "body must stay unread\n"),
            read_limit=1,
        )

        self.assertIsNone(_front_matter_path_fields(document))

    def test_actual_parser_latency_and_bytes_acceptance(self) -> None:
        front_matter = (
            "---\n"
            "implementation_files:\n"
            "  - tools/check_conventions.py\n"
            "tests:\n"
            "  - tools/tests/test_check_conventions.py::DocumentAuditTests\n"
            "---\n"
        )
        body = "# ignored documentation body payload\n" * 262_144

        with tempfile.TemporaryDirectory() as directory:
            document = Path(directory) / "large-plan.md"
            document.write_text(front_matter + body, encoding="utf-8")
            expected = _legacy_front_matter_path_fields(document)
            self.assertEqual(expected, _front_matter_path_fields(document))

            for pair_index in range(TOOLING28_WARMUP_PAIRS):
                if pair_index % 2 == 0:
                    _, legacy_checksum = _measure_parser(
                        document, _legacy_front_matter_path_fields
                    )
                    _, optimized_checksum = _measure_parser(
                        document, _front_matter_path_fields
                    )
                else:
                    _, optimized_checksum = _measure_parser(
                        document, _front_matter_path_fields
                    )
                    _, legacy_checksum = _measure_parser(
                        document, _legacy_front_matter_path_fields
                    )
                self.assertEqual(legacy_checksum, optimized_checksum)

            legacy_samples_ns: list[int] = []
            optimized_samples_ns: list[int] = []
            legacy_checksum = 0
            optimized_checksum = 0
            for sample_index in range(TOOLING28_SAMPLE_PAIRS):
                if sample_index % 2 == 0:
                    legacy_ns, legacy_checksum = _measure_parser(
                        document, _legacy_front_matter_path_fields
                    )
                    optimized_ns, optimized_checksum = _measure_parser(
                        document, _front_matter_path_fields
                    )
                else:
                    optimized_ns, optimized_checksum = _measure_parser(
                        document, _front_matter_path_fields
                    )
                    legacy_ns, legacy_checksum = _measure_parser(
                        document, _legacy_front_matter_path_fields
                    )
                self.assertEqual(legacy_checksum, optimized_checksum)
                legacy_samples_ns.append(legacy_ns)
                optimized_samples_ns.append(optimized_ns)

            legacy_p50_ns = _nearest_rank(legacy_samples_ns, 50)
            legacy_p95_ns = _nearest_rank(legacy_samples_ns, 95)
            optimized_p50_ns = _nearest_rank(optimized_samples_ns, 50)
            optimized_p95_ns = _nearest_rank(optimized_samples_ns, 95)
            legacy_bytes = document.stat().st_size
            optimized_consumed_bytes = len(front_matter.encode("utf-8"))

        print(
            "TOOLING28_STREAMED_FRONT_MATTER_PERF "
            f"warmup_pairs={TOOLING28_WARMUP_PAIRS} "
            f"sample_pairs={TOOLING28_SAMPLE_PAIRS} "
            "pair_order=alternating_legacy_even "
            f"legacy_bytes={legacy_bytes} "
            f"optimized_consumed_bytes={optimized_consumed_bytes} "
            f"legacy_samples_ns={legacy_samples_ns} "
            f"optimized_samples_ns={optimized_samples_ns} "
            f"legacy_p50_ns={legacy_p50_ns} "
            f"optimized_p50_ns={optimized_p50_ns} "
            f"legacy_p95_ns={legacy_p95_ns} "
            f"optimized_p95_ns={optimized_p95_ns} "
            f"checksum={optimized_checksum}"
        )

        self.assertEqual(legacy_checksum, optimized_checksum)
        self.assertLessEqual(optimized_consumed_bytes * 5, legacy_bytes)
        self.assertLessEqual(optimized_p50_ns * 100, legacy_p50_ns * 15)
        self.assertLessEqual(optimized_p95_ns * 100, legacy_p95_ns * 15)


if __name__ == "__main__":
    unittest.main()
