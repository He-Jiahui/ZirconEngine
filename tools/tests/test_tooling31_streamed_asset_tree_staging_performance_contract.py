import tempfile
import time
import types
import unittest
from pathlib import Path
from unittest.mock import patch

from tools import zircon_build_asset_staging as asset_staging


TOOLING31_WARMUP_PAIRS = 4
TOOLING31_SAMPLE_PAIRS = 21
TOOLING31_ROOT_DIRECTORIES = 128
TOOLING31_FILES_PER_DIRECTORY = 32


def _legacy_tree_entries(source_root: Path) -> list[tuple[Path, bool]]:
    entries: list[tuple[Path, bool]] = []
    for source in sorted(source_root.rglob("*")):
        if source.is_dir():
            entries.append((source, True))
        elif source.is_file():
            entries.append((source, False))
    return entries


def _optimized_tree_entries(source_root: Path) -> list[tuple[Path, bool]]:
    return list(asset_staging._iter_tree_entries(source_root))


def _entry_checksum(entries: list[tuple[Path, bool]]) -> int:
    return sum(len(path.name) + int(is_directory) for path, is_directory in entries)


def _measure_entries(source_root: Path, builder) -> tuple[int, int]:
    started = time.perf_counter_ns()
    entries = builder(source_root)
    elapsed_ns = max(time.perf_counter_ns() - started, 1)
    return elapsed_ns, _entry_checksum(entries)


def _nearest_rank(samples: list[int], percentile: int) -> int:
    ordered = sorted(samples)
    rank = (len(ordered) * percentile + 99) // 100
    return ordered[max(rank - 1, 0)]


class StreamedAssetTreeStagingPerformanceContract(unittest.TestCase):
    def test_asset_tree_is_streamed_without_materializing_rglob(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source_root = root / "source"
            destination_root = root / "destination"
            (source_root / "a-dir" / "nested").mkdir(parents=True)
            (source_root / "a-dir" / "a.txt").write_text("a", encoding="utf-8")
            (source_root / "a-dir" / "nested" / "z.txt").write_text(
                "z", encoding="utf-8"
            )
            (source_root / "b.txt").write_text("b", encoding="utf-8")
            config = types.SimpleNamespace(dry_run=True)
            copied: list[str] = []

            def record_copy(source: Path, _destination: Path, _config: object) -> None:
                copied.append(source.relative_to(source_root).as_posix())

            with (
                patch.object(
                    Path,
                    "rglob",
                    side_effect=AssertionError("asset staging materialized the full tree"),
                ),
                patch.object(asset_staging, "copy_asset_file", record_copy),
                patch("builtins.print"),
            ):
                skipped = asset_staging.copy_tree_contents(
                    source_root, destination_root, config
                )

        self.assertEqual(0, skipped)
        self.assertEqual(
            ["a-dir/a.txt", "a-dir/nested/z.txt", "b.txt"],
            copied,
        )

    def test_actual_streamed_traversal_latency_and_frontier_acceptance(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            source_root = Path(directory) / "source"
            source_root.mkdir()
            for directory_index in range(TOOLING31_ROOT_DIRECTORIES):
                asset_directory = source_root / f"asset_{directory_index:03}"
                asset_directory.mkdir()
                for file_index in range(TOOLING31_FILES_PER_DIRECTORY):
                    (asset_directory / f"payload_{file_index:03}.zasset").write_bytes(
                        b"zircon-asset-staging-fixture"
                    )

            legacy_entries = _legacy_tree_entries(source_root)
            optimized_entries = _optimized_tree_entries(source_root)
            self.assertEqual(legacy_entries, optimized_entries)

            for pair_index in range(TOOLING31_WARMUP_PAIRS):
                if pair_index % 2 == 0:
                    _, legacy_checksum = _measure_entries(
                        source_root, _legacy_tree_entries
                    )
                    _, optimized_checksum = _measure_entries(
                        source_root, _optimized_tree_entries
                    )
                else:
                    _, optimized_checksum = _measure_entries(
                        source_root, _optimized_tree_entries
                    )
                    _, legacy_checksum = _measure_entries(
                        source_root, _legacy_tree_entries
                    )
                self.assertEqual(legacy_checksum, optimized_checksum)

            legacy_samples_ns: list[int] = []
            optimized_samples_ns: list[int] = []
            legacy_checksum = 0
            optimized_checksum = 0
            for sample_index in range(TOOLING31_SAMPLE_PAIRS):
                if sample_index % 2 == 0:
                    legacy_ns, legacy_checksum = _measure_entries(
                        source_root, _legacy_tree_entries
                    )
                    optimized_ns, optimized_checksum = _measure_entries(
                        source_root, _optimized_tree_entries
                    )
                else:
                    optimized_ns, optimized_checksum = _measure_entries(
                        source_root, _optimized_tree_entries
                    )
                    legacy_ns, legacy_checksum = _measure_entries(
                        source_root, _legacy_tree_entries
                    )
                self.assertEqual(legacy_checksum, optimized_checksum)
                legacy_samples_ns.append(legacy_ns)
                optimized_samples_ns.append(optimized_ns)

            legacy_p50_ns = _nearest_rank(legacy_samples_ns, 50)
            legacy_p95_ns = _nearest_rank(legacy_samples_ns, 95)
            optimized_p50_ns = _nearest_rank(optimized_samples_ns, 50)
            optimized_p95_ns = _nearest_rank(optimized_samples_ns, 95)
            legacy_peak_buffered_entries = len(legacy_entries)
            optimized_peak_frontier_entries = (
                TOOLING31_ROOT_DIRECTORIES + TOOLING31_FILES_PER_DIRECTORY
            )

        print(
            "TOOLING31_STREAMED_ASSET_TREE_STAGING_PERF "
            f"warmup_pairs={TOOLING31_WARMUP_PAIRS} "
            f"sample_pairs={TOOLING31_SAMPLE_PAIRS} "
            "pair_order=alternating_legacy_even "
            f"root_directories={TOOLING31_ROOT_DIRECTORIES} "
            f"files_per_directory={TOOLING31_FILES_PER_DIRECTORY} "
            f"legacy_peak_buffered_entries={legacy_peak_buffered_entries} "
            f"optimized_peak_frontier_entries={optimized_peak_frontier_entries} "
            f"legacy_samples_ns={legacy_samples_ns} "
            f"optimized_samples_ns={optimized_samples_ns} "
            f"legacy_p50_ns={legacy_p50_ns} "
            f"optimized_p50_ns={optimized_p50_ns} "
            f"legacy_p95_ns={legacy_p95_ns} "
            f"optimized_p95_ns={optimized_p95_ns} "
            f"checksum={optimized_checksum}"
        )

        self.assertEqual(legacy_checksum, optimized_checksum)
        self.assertLessEqual(
            optimized_peak_frontier_entries * 10,
            legacy_peak_buffered_entries,
        )
        self.assertLessEqual(optimized_p50_ns * 100, legacy_p50_ns * 90)
        self.assertLessEqual(optimized_p95_ns * 100, legacy_p95_ns * 95)


if __name__ == "__main__":
    unittest.main()
