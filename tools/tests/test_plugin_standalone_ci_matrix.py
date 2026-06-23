import re
import unittest
from pathlib import Path

from tools.plugin_structure_audits.dependency_boundary import (
    audit_plugin_dependency_boundary,
)


class PluginStandaloneCiMatrixTests(unittest.TestCase):
    def test_plugin_standalone_dist_ci_matrix_covers_dist_capable_plugins(self):
        repo_root = Path(__file__).resolve().parents[2]
        audit = audit_plugin_dependency_boundary(repo_root).to_json()
        expected_entries = {
            (entry["plugin_id"], entry["package"])
            for entry in audit["dist_build_matrix_entries"]
        }
        workflow_text = (repo_root / ".github" / "workflows" / "ci.yml").read_text(
            encoding="utf-8"
        )

        self.assertEqual(
            expected_entries,
            standalone_dist_ci_matrix_entries(workflow_text),
        )


def standalone_dist_ci_matrix_entries(workflow_text: str) -> set[tuple[str, str]]:
    job_block = standalone_dist_ci_job_block(workflow_text)
    entries: set[tuple[str, str]] = set()
    current_plugin_id: str | None = None
    for line in job_block.splitlines():
        plugin_match = re.match(r"\s*-\s*plugin_id:\s*([A-Za-z0-9_.-]+)\s*$", line)
        if plugin_match:
            current_plugin_id = plugin_match.group(1)
            continue
        package_match = re.match(r"\s*package:\s*([A-Za-z0-9_.-]+)\s*$", line)
        if current_plugin_id is not None and package_match:
            entries.add((current_plugin_id, package_match.group(1)))
            current_plugin_id = None
    return entries


def standalone_dist_ci_job_block(workflow_text: str) -> str:
    start_match = re.search(r"^  plugin-standalone-dist:\s*$", workflow_text, re.M)
    if start_match is None:
        return ""
    next_job_match = re.search(
        r"^  [A-Za-z0-9_-]+:\s*$",
        workflow_text[start_match.end() :],
        re.M,
    )
    if next_job_match is None:
        return workflow_text[start_match.start() :]
    return workflow_text[
        start_match.start() : start_match.end() + next_job_match.start()
    ]


if __name__ == "__main__":
    unittest.main()
