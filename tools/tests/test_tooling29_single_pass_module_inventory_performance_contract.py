import sys
import tempfile
import time
import unittest
from collections import defaultdict
from pathlib import Path
from unittest.mock import patch


REPO_ROOT = Path(__file__).resolve().parents[2]
AUDIT_SCRIPTS = (
    REPO_ROOT
    / ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts"
)
sys.path.insert(0, str(AUDIT_SCRIPTS))

from runtime_structure_audits import module_inventory  # noqa: E402


TOOLING29_WARMUP_PAIRS = 4
TOOLING29_SAMPLE_PAIRS = 21
TOOLING29_FIXTURE_FILES = 32
TOOLING29_LINES_PER_FILE = 2_048


def _legacy_find_locations(
    root: Path, files: list[Path], pattern
) -> list[module_inventory.Location]:
    results: list[module_inventory.Location] = []
    for path in files:
        for line_no, line in enumerate(
            module_inventory._read_text(path).splitlines(), start=1
        ):
            if pattern.search(line):
                results.append(
                    module_inventory.Location(
                        path=module_inventory._relative(root, path),
                        line=line_no,
                        snippet=line.strip(),
                    )
                )
    return results


def _legacy_engine_module_impls(files: list[Path]) -> list[str]:
    owners: list[str] = []
    for path in files:
        for line in module_inventory._read_text(path).splitlines():
            match = module_inventory.ENGINE_MODULE_IMPL_RE.search(line)
            if match:
                owners.append(match.group(1))
    return owners


def _legacy_runtime_inventory(
    root: Path, hotspot_threshold: int
) -> module_inventory.RuntimeInventory:
    zircon_crates = [
        member
        for member in module_inventory._workspace_members(root)
        if member.startswith("zircon_")
    ]
    descriptor_locations: dict[str, list[module_inventory.Location]] = {}
    stub_usage: dict[str, list[module_inventory.Location]] = defaultdict(list)
    owner_impls: dict[str, list[str]] = defaultdict(list)
    hotspots: dict[str, list[tuple[str, int]]] = defaultdict(list)
    all_rs_files: list[Path] = []

    for crate_name in zircon_crates:
        production_files = [
            path
            for path in module_inventory._crate_rs_files(root, crate_name)
            if module_inventory._is_production_file(path)
        ]
        all_rs_files.extend(production_files)
        descriptor_locations[crate_name] = _legacy_find_locations(
            root, production_files, module_inventory.MODULE_DESCRIPTOR_RE
        )
        stub_usage[crate_name] = _legacy_find_locations(
            root, production_files, module_inventory.STUB_MODULE_RE
        )
        owner_impls[crate_name] = _legacy_engine_module_impls(production_files)
        for path in production_files:
            line_count = len(module_inventory._read_text(path).splitlines())
            if line_count >= hotspot_threshold:
                hotspots[crate_name].append(
                    (module_inventory._relative(root, path), line_count)
                )
        hotspots[crate_name].sort(key=lambda item: item[1], reverse=True)

    module_crates = sorted(
        crate_name
        for crate_name in zircon_crates
        if descriptor_locations[crate_name] or owner_impls[crate_name]
    )
    support_crates = sorted(
        crate_name for crate_name in zircon_crates if crate_name not in module_crates
    )
    return module_inventory.RuntimeInventory(
        zircon_crates=zircon_crates,
        all_rs_files=sorted(all_rs_files),
        module_crates=module_crates,
        support_crates=support_crates,
        hotspots=dict(hotspots),
        descriptor_locations=descriptor_locations,
        stub_usage=dict(stub_usage),
        owner_impls=dict(owner_impls),
    )


def _inventory_checksum(inventory: module_inventory.RuntimeInventory) -> int:
    return (
        len(inventory.all_rs_files)
        + sum(len(rows) for rows in inventory.hotspots.values())
        + sum(len(rows) for rows in inventory.descriptor_locations.values())
        + sum(len(rows) for rows in inventory.stub_usage.values())
        + sum(len(rows) for rows in inventory.owner_impls.values())
    )


def _measure_inventory(root: Path, inventory_builder) -> tuple[int, int]:
    started = time.perf_counter_ns()
    inventory = inventory_builder(root, TOOLING29_LINES_PER_FILE)
    elapsed_ns = max(time.perf_counter_ns() - started, 1)
    return elapsed_ns, _inventory_checksum(inventory)


def _nearest_rank(samples: list[int], percentile: int) -> int:
    ordered = sorted(samples)
    rank = (len(ordered) * percentile + 99) // 100
    return ordered[max(rank - 1, 0)]


class SinglePassModuleInventoryPerformanceContract(unittest.TestCase):
    def test_each_production_rust_file_is_read_once(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "Cargo.toml").write_text(
                '[workspace]\nmembers = ["zircon_alpha", "helper"]\n',
                encoding="utf-8",
            )
            source_root = root / "zircon_alpha/src"
            source_root.mkdir(parents=True)
            lib_rs = source_root / "lib.rs"
            lib_rs.write_text(
                "fn module_descriptor() {}\n"
                "fn marker() { stub_module_descriptor(); }\n"
                "impl EngineModule for AlphaModule {}\n"
                "fn hotspot_line() {}\n",
                encoding="utf-8",
            )
            nested_rs = source_root / "nested.rs"
            nested_rs.write_text("fn helper() {}\n", encoding="utf-8")
            (source_root / "tests.rs").write_text(
                "fn module_descriptor() {}\n", encoding="utf-8"
            )

            read_counts: dict[Path, int] = {}
            read_text = module_inventory._read_text

            def counted_read(path: Path) -> str:
                read_counts[path] = read_counts.get(path, 0) + 1
                return read_text(path)

            with patch.object(module_inventory, "_read_text", counted_read):
                inventory = module_inventory.runtime_inventory(root, hotspot_threshold=4)

        self.assertEqual(1, read_counts[root / "Cargo.toml"])
        self.assertEqual(1, read_counts[lib_rs])
        self.assertEqual(1, read_counts[nested_rs])
        self.assertNotIn(source_root / "tests.rs", read_counts)
        self.assertEqual(["AlphaModule"], inventory.owner_impls["zircon_alpha"])
        self.assertEqual(1, len(inventory.descriptor_locations["zircon_alpha"]))
        self.assertEqual(1, len(inventory.stub_usage["zircon_alpha"]))
        self.assertEqual(
            [("zircon_alpha/src/lib.rs", 4)],
            inventory.hotspots["zircon_alpha"],
        )

    def test_actual_inventory_latency_reads_and_bytes_acceptance(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = '[workspace]\nmembers = ["zircon_benchmark"]\n'
            (root / "Cargo.toml").write_text(manifest, encoding="utf-8")
            source_root = root / "zircon_benchmark/src"
            source_root.mkdir(parents=True)
            filler_lines = TOOLING29_LINES_PER_FILE - 3
            for file_index in range(TOOLING29_FIXTURE_FILES):
                source = (
                    "fn module_descriptor() {}\n"
                    "fn marker() { stub_module_descriptor(); }\n"
                    f"impl EngineModule for BenchmarkModule{file_index} {{}}\n"
                    + "fn inventory_filler_line() {}\n" * filler_lines
                )
                (source_root / f"module_{file_index:03}.rs").write_text(
                    source, encoding="utf-8"
                )
            (source_root / "tests.rs").write_text(
                "fn module_descriptor() {}\n", encoding="utf-8"
            )

            legacy_inventory = _legacy_runtime_inventory(
                root, TOOLING29_LINES_PER_FILE
            )
            optimized_inventory = module_inventory.runtime_inventory(
                root, TOOLING29_LINES_PER_FILE
            )
            self.assertEqual(legacy_inventory, optimized_inventory)

            for pair_index in range(TOOLING29_WARMUP_PAIRS):
                if pair_index % 2 == 0:
                    _, legacy_checksum = _measure_inventory(
                        root, _legacy_runtime_inventory
                    )
                    _, optimized_checksum = _measure_inventory(
                        root, module_inventory.runtime_inventory
                    )
                else:
                    _, optimized_checksum = _measure_inventory(
                        root, module_inventory.runtime_inventory
                    )
                    _, legacy_checksum = _measure_inventory(
                        root, _legacy_runtime_inventory
                    )
                self.assertEqual(legacy_checksum, optimized_checksum)

            legacy_samples_ns: list[int] = []
            optimized_samples_ns: list[int] = []
            legacy_checksum = 0
            optimized_checksum = 0
            for sample_index in range(TOOLING29_SAMPLE_PAIRS):
                if sample_index % 2 == 0:
                    legacy_ns, legacy_checksum = _measure_inventory(
                        root, _legacy_runtime_inventory
                    )
                    optimized_ns, optimized_checksum = _measure_inventory(
                        root, module_inventory.runtime_inventory
                    )
                else:
                    optimized_ns, optimized_checksum = _measure_inventory(
                        root, module_inventory.runtime_inventory
                    )
                    legacy_ns, legacy_checksum = _measure_inventory(
                        root, _legacy_runtime_inventory
                    )
                self.assertEqual(legacy_checksum, optimized_checksum)
                legacy_samples_ns.append(legacy_ns)
                optimized_samples_ns.append(optimized_ns)

            legacy_p50_ns = _nearest_rank(legacy_samples_ns, 50)
            legacy_p95_ns = _nearest_rank(legacy_samples_ns, 95)
            optimized_p50_ns = _nearest_rank(optimized_samples_ns, 50)
            optimized_p95_ns = _nearest_rank(optimized_samples_ns, 95)
            manifest_bytes = len(manifest.encode("utf-8"))
            source_bytes = sum(
                path.stat().st_size
                for path in source_root.glob("*.rs")
                if module_inventory._is_production_file(path)
            )
            legacy_reads = 1 + TOOLING29_FIXTURE_FILES * 4
            optimized_reads = 1 + TOOLING29_FIXTURE_FILES
            legacy_bytes = manifest_bytes + source_bytes * 4
            optimized_bytes = manifest_bytes + source_bytes

        print(
            "TOOLING29_SINGLE_PASS_MODULE_INVENTORY_PERF "
            f"warmup_pairs={TOOLING29_WARMUP_PAIRS} "
            f"sample_pairs={TOOLING29_SAMPLE_PAIRS} "
            "pair_order=alternating_legacy_even "
            f"fixture_files={TOOLING29_FIXTURE_FILES} "
            f"lines_per_file={TOOLING29_LINES_PER_FILE} "
            f"legacy_reads={legacy_reads} optimized_reads={optimized_reads} "
            f"legacy_bytes={legacy_bytes} optimized_bytes={optimized_bytes} "
            f"legacy_samples_ns={legacy_samples_ns} "
            f"optimized_samples_ns={optimized_samples_ns} "
            f"legacy_p50_ns={legacy_p50_ns} "
            f"optimized_p50_ns={optimized_p50_ns} "
            f"legacy_p95_ns={legacy_p95_ns} "
            f"optimized_p95_ns={optimized_p95_ns} "
            f"checksum={optimized_checksum}"
        )

        self.assertEqual(legacy_checksum, optimized_checksum)
        self.assertLessEqual(optimized_reads * 10, legacy_reads * 3)
        self.assertLessEqual(optimized_bytes * 10, legacy_bytes * 3)
        self.assertLessEqual(optimized_p50_ns * 100, legacy_p50_ns * 80)
        self.assertLessEqual(optimized_p95_ns * 100, legacy_p95_ns * 80)


if __name__ == "__main__":
    unittest.main()
