from __future__ import annotations

import unittest
from copy import deepcopy
from pathlib import Path

from tools.zircon_export.tests.platform_bundle_template_resolution_schema_test_support import (
    PlatformBundleTemplateResolutionReportAssertions,
)


class PlatformBundleTemplateResolutionCandidateSchemaTests(
    PlatformBundleTemplateResolutionReportAssertions,
    unittest.TestCase,
):
    def test_report_rejects_template_resolution_candidate_dir_outside_root(
        self,
    ) -> None:
        def move_selected_candidate_outside_root(resolution: dict[str, object]) -> None:
            template_dir = str(
                Path(str(resolution["template_root"])).parent / "outside-template"
            )
            resolution["template_dir"] = template_dir
            resolution["candidates"][0]["template_dir"] = template_dir

        self._assert_template_resolution_diagnostic(
            move_selected_candidate_outside_root,
            "PlatformBundle report template_resolution candidates[0].template_dir "
            "must be inside template_root",
        )

    def test_report_rejects_template_resolution_skipped_candidate_dir_outside_root(
        self,
    ) -> None:
        self._assert_template_resolution_diagnostic(
            lambda resolution: resolution["skipped_candidates"][0].__setitem__(
                "template_dir",
                str(Path(str(resolution["template_root"])).parent / "broken-template"),
            ),
            "PlatformBundle report template_resolution skipped_candidates[0].template_dir "
            "must be inside template_root",
        )

    def test_report_rejects_template_resolution_candidate_dir_not_direct_child_of_root(
        self,
    ) -> None:
        def move_selected_candidate_to_nested_dir(
            resolution: dict[str, object],
        ) -> None:
            template_dir = str(
                Path(str(resolution["template_root"])) / "nested" / "windows-template"
            )
            resolution["template_dir"] = template_dir
            resolution["candidates"][0]["template_dir"] = template_dir

        self._assert_template_resolution_diagnostic(
            move_selected_candidate_to_nested_dir,
            "PlatformBundle report template_resolution candidates[0].template_dir "
            "must be a direct child of template_root",
        )

    def test_report_rejects_template_resolution_skipped_candidate_dir_not_direct_child_of_root(
        self,
    ) -> None:
        self._assert_template_resolution_diagnostic(
            lambda resolution: resolution["skipped_candidates"][0].__setitem__(
                "template_dir",
                str(
                    Path(str(resolution["template_root"]))
                    / "nested"
                    / "broken-template"
                ),
            ),
            "PlatformBundle report template_resolution skipped_candidates[0].template_dir "
            "must be a direct child of template_root",
        )

    def test_report_rejects_template_resolution_skipped_candidate_without_diagnostics(
        self,
    ) -> None:
        self._assert_template_resolution_diagnostic(
            lambda resolution: resolution["skipped_candidates"][0].__setitem__(
                "diagnostics",
                [],
            ),
            "PlatformBundle report template_resolution skipped_candidates[0].diagnostics "
            "must include diagnostics",
        )

    def test_report_rejects_template_resolution_skipped_candidate_missing_required_field(
        self,
    ) -> None:
        for field in ("diagnostics", "template_dir"):
            with self.subTest(field=field, mutation="missing"):
                self._assert_template_resolution_diagnostic(
                    lambda resolution, field=field: resolution["skipped_candidates"][
                        0
                    ].pop(field),
                    f"PlatformBundle report template_resolution skipped_candidates[0].{field} is required",
                )
            with self.subTest(field=field, mutation="null"):
                self._assert_template_resolution_diagnostic(
                    lambda resolution, field=field: resolution["skipped_candidates"][
                        0
                    ].__setitem__(field, None),
                    f"PlatformBundle report template_resolution skipped_candidates[0].{field} is required",
                )

    def test_report_rejects_template_resolution_candidate_bundle_format_unknown(
        self,
    ) -> None:
        self._assert_template_resolution_diagnostic(
            lambda resolution: resolution["candidates"][0].__setitem__(
                "bundle_format",
                "unknown_format",
            ),
            "PlatformBundle report template_resolution candidates[0].bundle_format="
            "'unknown_format' is not one of app_bundle, directory, web_static, zip",
        )

    def test_report_rejects_template_resolution_candidate_host_artifact_unknown(
        self,
    ) -> None:
        self._assert_template_resolution_diagnostic(
            lambda resolution: resolution["candidates"][0].__setitem__(
                "host_artifact",
                "generated",
            ),
            "PlatformBundle report template_resolution candidates[0].host_artifact="
            "'generated' is not one of placeholder, precompiled",
        )

    def test_report_rejects_template_resolution_candidate_missing_required_field(
        self,
    ) -> None:
        for field in (
            "bundle_format",
            "compatible_profiles",
            "engine_version",
            "host_artifact",
            "target_platform",
            "template_dir",
            "template_id",
        ):
            with self.subTest(field=field, mutation="missing"):
                self._assert_template_resolution_diagnostic(
                    lambda resolution, field=field: resolution["candidates"][
                        0
                    ].pop(field),
                    f"PlatformBundle report template_resolution candidates[0].{field} is required",
                )
            with self.subTest(field=field, mutation="null"):
                self._assert_template_resolution_diagnostic(
                    lambda resolution, field=field: resolution["candidates"][
                        0
                    ].__setitem__(field, None),
                    f"PlatformBundle report template_resolution candidates[0].{field} is required",
                )

    def test_report_rejects_template_resolution_candidate_identity_mismatch(
        self,
    ) -> None:
        cases = (
            (
                "engine_version",
                "9.9.9",
                "PlatformBundle report template_resolution candidates[0].engine_version "
                "9.9.9 does not match expected_engine_version 0.1.0",
            ),
            (
                "target_platform",
                "linux-x86_64",
                "PlatformBundle report template_resolution candidates[0].target_platform "
                "linux-x86_64 does not match expected_target_platform windows-x86_64",
            ),
        )
        for field, value, expected_diagnostic in cases:
            with self.subTest(field=field):
                self._assert_template_resolution_diagnostic(
                    lambda resolution, field=field, value=value: resolution[
                        "candidates"
                    ][0].__setitem__(
                        field,
                        value,
                    ),
                    expected_diagnostic,
                )

    def test_report_rejects_template_resolution_candidate_missing_profile_membership(
        self,
    ) -> None:
        self._assert_template_resolution_diagnostic(
            lambda resolution: resolution["candidates"][0].__setitem__(
                "compatible_profiles",
                ["linux-release"],
            ),
            "PlatformBundle report template_resolution candidates[0].compatible_profiles "
            "does not include profile windows-release",
        )

    def test_report_rejects_template_resolution_duplicate_candidate_template_dir(
        self,
    ) -> None:
        def duplicate_candidate_template_dir(resolution: dict[str, object]) -> None:
            candidates = resolution["candidates"]
            self.assertIsInstance(candidates, list)
            candidates.append(deepcopy(candidates[0]))

        self._assert_template_resolution_diagnostic(
            duplicate_candidate_template_dir,
            "PlatformBundle report template_resolution candidates[1].template_dir "
            "duplicates candidates[0].template_dir",
        )

    def test_report_rejects_template_resolution_duplicate_skipped_candidate_template_dir(
        self,
    ) -> None:
        def duplicate_skipped_candidate_template_dir(
            resolution: dict[str, object],
        ) -> None:
            skipped_candidates = resolution["skipped_candidates"]
            self.assertIsInstance(skipped_candidates, list)
            skipped_candidates.append(deepcopy(skipped_candidates[0]))

        self._assert_template_resolution_diagnostic(
            duplicate_skipped_candidate_template_dir,
            "PlatformBundle report template_resolution skipped_candidates[1].template_dir "
            "duplicates skipped_candidates[0].template_dir",
        )

    def test_report_rejects_template_resolution_candidate_also_skipped(
        self,
    ) -> None:
        def reuse_candidate_as_skipped_candidate(
            resolution: dict[str, object],
        ) -> None:
            candidates = resolution["candidates"]
            skipped_candidates = resolution["skipped_candidates"]
            self.assertIsInstance(candidates, list)
            self.assertIsInstance(skipped_candidates, list)
            candidate = candidates[0]
            skipped_candidate = skipped_candidates[0]
            self.assertIsInstance(candidate, dict)
            self.assertIsInstance(skipped_candidate, dict)
            skipped_candidate["template_dir"] = candidate["template_dir"]

        self._assert_template_resolution_diagnostic(
            reuse_candidate_as_skipped_candidate,
            "PlatformBundle report template_resolution skipped_candidates[0].template_dir "
            "duplicates candidates[0].template_dir",
        )

    def test_report_rejects_template_resolution_candidate_blank_profile_entry(
        self,
    ) -> None:
        self._assert_template_resolution_diagnostic(
            lambda resolution: resolution["candidates"][0].__setitem__(
                "compatible_profiles",
                ["windows-release", ""],
            ),
            "PlatformBundle report template_resolution candidates[0].compatible_profiles "
            "must not contain blank entries",
        )

    def test_report_rejects_template_resolution_candidate_duplicate_profile_entry(
        self,
    ) -> None:
        self._assert_template_resolution_diagnostic(
            lambda resolution: resolution["candidates"][0].__setitem__(
                "compatible_profiles",
                ["windows-release", "windows-release"],
            ),
            "PlatformBundle report template_resolution candidates[0].compatible_profiles "
            "duplicate entry windows-release",
        )

    def test_report_rejects_template_resolution_candidate_string_field_blank(
        self,
    ) -> None:
        for field in (
            "bundle_format",
            "engine_version",
            "host_artifact",
            "target_platform",
            "template_dir",
            "template_id",
        ):
            with self.subTest(field=field):
                self._assert_template_resolution_diagnostic(
                    lambda resolution, field=field: resolution["candidates"][
                        0
                    ].__setitem__(field, " "),
                    f"PlatformBundle report template_resolution candidates[0].{field} must be a non-empty string",
                )

    def test_report_rejects_template_resolution_candidate_padded_string_field(
        self,
    ) -> None:
        for field in (
            "bundle_format",
            "engine_version",
            "host_artifact",
            "target_platform",
            "template_dir",
            "template_id",
        ):
            with self.subTest(field=field):
                self._assert_template_resolution_diagnostic(
                    lambda resolution, field=field: resolution["candidates"][
                        0
                    ].__setitem__(
                        field,
                        f" {resolution['candidates'][0][field]} ",
                    ),
                    f"PlatformBundle report template_resolution candidates[0].{field} must be a non-empty trimmed string",
                )

    def test_report_rejects_template_resolution_candidate_padded_profile_entry(
        self,
    ) -> None:
        self._assert_template_resolution_diagnostic(
            lambda resolution: resolution["candidates"][0].__setitem__(
                "compatible_profiles",
                [" windows-release "],
            ),
            "PlatformBundle report template_resolution candidates[0].compatible_profiles[0] "
            "must be a non-empty trimmed string",
        )

    def test_report_rejects_template_resolution_skipped_candidate_string_field_blank(
        self,
    ) -> None:
        self._assert_template_resolution_diagnostic(
            lambda resolution: resolution["skipped_candidates"][0].__setitem__(
                "template_dir",
                " ",
            ),
            "PlatformBundle report template_resolution skipped_candidates[0].template_dir "
            "must be a non-empty string",
        )

    def test_report_rejects_template_resolution_skipped_candidate_padded_template_dir(
        self,
    ) -> None:
        self._assert_template_resolution_diagnostic(
            lambda resolution: resolution["skipped_candidates"][0].__setitem__(
                "template_dir",
                f" {resolution['skipped_candidates'][0]['template_dir']} ",
            ),
            "PlatformBundle report template_resolution skipped_candidates[0].template_dir "
            "must be a non-empty trimmed string",
        )


if __name__ == "__main__":
    unittest.main()
