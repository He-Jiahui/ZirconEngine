from __future__ import annotations

import json
import math
import re
import subprocess
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
BASELINE = "8e56165c4c789416c328898d3d8937d934b52efa"
CONTRACT = ROOT / "examples/woc/contracts/m4_abilities.json"
GENERATOR = ROOT / "examples/woc/tools/m4_ability_zr_codegen.mjs"
CATALOG = (
    ROOT / "examples/woc/scripts/woc_game/src/generated/m4_ability_catalog.zr"
)
CATALOG_TEST = (
    ROOT / "examples/woc/scripts/woc_game/src/content/m4_ability_catalog_test_main.zr"
)
METRIC_BUCKET_SIZE = 8


def zr_function_body(source: str, name: str) -> str:
    match = re.search(
        rf"(?m)^(?:pub\s+)?{re.escape(name)}\s*\([^)]*\)\s*:[^{{]+{{",
        source,
    )
    if match is None:
        raise AssertionError(f"missing Zr function {name}")
    depth = 1
    index = match.end()
    while index < len(source) and depth:
        depth += source[index] == "{"
        depth -= source[index] == "}"
        index += 1
    if depth:
        raise AssertionError(f"unterminated Zr function {name}")
    return source[match.end() : index - 1]


def zr_index_branch(source: str, index_value: int) -> str:
    marker = f"    if (index == {index_value}) {{"
    start = source.find(marker)
    if start < 0:
        raise AssertionError(f"missing metric branch for index {index_value}")
    depth = 0
    index = start
    opened = False
    while index < len(source):
        if source[index] == "{":
            depth += 1
            opened = True
        elif source[index] == "}":
            depth -= 1
            if opened and depth == 0:
                return source[start : index + 1]
        index += 1
    raise AssertionError(f"unterminated metric branch for index {index_value}")


class Runtime18M4AbilityMetricPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.document = json.loads(CONTRACT.read_text(encoding="utf-8"))
        cls.generator = GENERATOR.read_text(encoding="utf-8")
        cls.catalog = CATALOG.read_text(encoding="utf-8")
        cls.catalog_test = CATALOG_TEST.read_text(encoding="utf-8")
        cls.entry_count = len(cls.document["entries"])
        cls.bucket_count = math.ceil(cls.entry_count / METRIC_BUCKET_SIZE)
        baseline = subprocess.run(
            [
                "git",
                "show",
                f"{BASELINE}:{CATALOG.relative_to(ROOT).as_posix()}",
            ],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        if baseline.returncode != 0:
            raise AssertionError(baseline.stdout + baseline.stderr)
        cls.baseline_catalog = baseline.stdout

    def test_generated_projection_is_current(self) -> None:
        result = subprocess.run(
            ["node", str(GENERATOR), "--check"],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_zr_contract_tracks_the_current_catalog_identity(self) -> None:
        digest = self.document["catalog_sha256"]
        self.assertIn(f'"{digest}"', self.catalog_test)
        self.assertIn(f"catalog.count() != {self.entry_count}", self.catalog_test)

    def test_metric_routes_through_bounded_generated_buckets(self) -> None:
        body = zr_function_body(self.catalog, "metric")
        self.assertIn(f"var bucket = index / {METRIC_BUCKET_SIZE};", body)
        self.assertNotIn("if (index ==", body)
        for bucket in range(self.bucket_count):
            self.assertIn(f"metricBucket{bucket}(index, rank, field)", body)

    def test_metric_buckets_cover_every_index_once(self) -> None:
        covered: list[int] = []
        for bucket in range(self.bucket_count):
            body = zr_function_body(self.catalog, f"metricBucket{bucket}")
            indexes = [int(value) for value in re.findall(r"if \(index == (\d+)\)", body)]
            self.assertLessEqual(len(indexes), METRIC_BUCKET_SIZE)
            self.assertGreater(len(indexes), 0)
            covered.extend(indexes)
        self.assertEqual(covered, list(range(self.entry_count)))

    def test_every_metric_result_branch_matches_the_baseline(self) -> None:
        legacy = zr_function_body(self.baseline_catalog, "metric")
        optimized = "\n".join(
            zr_function_body(self.catalog, f"metricBucket{bucket}")
            for bucket in range(self.bucket_count)
        )
        for index in range(self.entry_count):
            self.assertEqual(
                zr_index_branch(optimized, index),
                zr_index_branch(legacy, index),
                f"metric result logic changed for index {index}",
            )

    def test_worst_case_index_comparisons_drop_by_at_least_eighty_percent(self) -> None:
        legacy_worst = self.entry_count
        bucketed_worst = self.bucket_count + METRIC_BUCKET_SIZE
        reduction_percent = (legacy_worst - bucketed_worst) * 100.0 / legacy_worst
        self.assertGreaterEqual(reduction_percent, 80.0)


if __name__ == "__main__":
    unittest.main()
