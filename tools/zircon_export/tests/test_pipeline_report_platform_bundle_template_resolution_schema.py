from __future__ import annotations

import tempfile
import unittest
from copy import deepcopy
from pathlib import Path
from typing import Callable

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.platform_bundle_report_test_support import (
    _read_stage_report,
    _write_bundle_manifest_from_platform_report,
    _write_platform_bundle_fixture,
    _write_stage_report,
)


class PlatformBundleTemplateResolutionSchemaTests(unittest.TestCase):
    def test_report_rejects_template_resolution_non_fatal_with_diagnostics(
        self,
    ) -> None:
        self._assert_template_resolution_diagnostic(
            lambda resolution: resolution.__setitem__(
                "diagnostics",
                ["multiple export templates matched profile=windows-release"],
            ),
            "PlatformBundle report template_resolution non-fatal resolution "
            "must not include diagnostics",
        )

    def test_report_rejects_template_resolution_non_fatal_without_template_dir(
        self,
    ) -> None:
        self._assert_template_resolution_diagnostic(
            lambda resolution: resolution.__setitem__("template_dir", None),
            "PlatformBundle report template_resolution non-fatal resolution "
            "must select template_dir",
        )

    def test_report_rejects_template_resolution_fatal_without_diagnostics(
        self,
    ) -> None:
        def make_fatal_without_diagnostics(resolution: dict[str, object]) -> None:
            resolution["fatal"] = True
            resolution["template_dir"] = None
            resolution["diagnostics"] = []

        self._assert_template_resolution_diagnostic(
            make_fatal_without_diagnostics,
            "PlatformBundle report template_resolution fatal resolution "
            "must include diagnostics",
        )

    def test_report_rejects_template_resolution_fatal_selected_template_dir(
        self,
    ) -> None:
        self._assert_template_resolution_diagnostic(
            lambda resolution: resolution.__setitem__("fatal", True),
            "PlatformBundle report template_resolution fatal resolution "
            "must not select template_dir",
        )

    def test_report_rejects_template_resolution_fatal_single_candidate(
        self,
    ) -> None:
        def make_fatal_with_single_candidate(resolution: dict[str, object]) -> None:
            resolution["fatal"] = True
            resolution["template_dir"] = None
            resolution["diagnostics"] = ["forced fatal"]

        self._assert_template_resolution_diagnostic(
            make_fatal_with_single_candidate,
            "PlatformBundle report template_resolution fatal resolution "
            "must not contain exactly one candidate",
        )

    def test_report_rejects_template_resolution_fatal_multiple_candidates_without_multiple_match_diagnostic(
        self,
    ) -> None:
        def make_fatal_with_unexplained_multiple_candidates(
            resolution: dict[str, object],
        ) -> None:
            candidates = resolution["candidates"]
            self.assertIsInstance(candidates, list)
            second_candidate = dict(candidates[0])
            second_candidate["template_dir"] = str(
                Path(str(resolution["template_root"])) / "second-template"
            )
            candidates.append(second_candidate)
            resolution["fatal"] = True
            resolution["template_dir"] = None
            resolution["diagnostics"] = ["forced fatal"]

        self._assert_template_resolution_diagnostic(
            make_fatal_with_unexplained_multiple_candidates,
            "PlatformBundle report template_resolution fatal resolution "
            "with multiple candidates must include multiple-match diagnostics",
        )

    def test_report_rejects_template_resolution_fatal_multiple_candidates_wrong_profile_diagnostic(
        self,
    ) -> None:
        def make_fatal_with_wrong_profile_multiple_candidate_diagnostic(
            resolution: dict[str, object],
        ) -> None:
            candidates = resolution["candidates"]
            self.assertIsInstance(candidates, list)
            second_candidate = dict(candidates[0])
            second_candidate["template_dir"] = str(
                Path(str(resolution["template_root"])) / "second-template"
            )
            candidates.append(second_candidate)
            resolution["fatal"] = True
            resolution["template_dir"] = None
            resolution["diagnostics"] = [
                "multiple export templates matched profile=other-profile: "
                + ", ".join(
                    str(candidate["template_dir"])
                    for candidate in candidates
                    if isinstance(candidate, dict)
                )
            ]

        self._assert_template_resolution_diagnostic(
            make_fatal_with_wrong_profile_multiple_candidate_diagnostic,
            "PlatformBundle report template_resolution fatal resolution "
            "with multiple candidates must include multiple-match diagnostics "
            "for profile windows-release",
        )

    def test_report_rejects_template_resolution_fatal_multiple_candidates_incomplete_candidate_diagnostic(
        self,
    ) -> None:
        def make_fatal_with_incomplete_multiple_candidate_diagnostic(
            resolution: dict[str, object],
        ) -> None:
            candidates = resolution["candidates"]
            self.assertIsInstance(candidates, list)
            second_candidate = dict(candidates[0])
            second_candidate["template_dir"] = str(
                Path(str(resolution["template_root"])) / "second-template"
            )
            candidates.append(second_candidate)
            resolution["fatal"] = True
            resolution["template_dir"] = None
            resolution["diagnostics"] = [
                "multiple export templates matched profile=windows-release: "
                f"{candidates[0]['template_dir']}"
            ]

        self._assert_template_resolution_diagnostic(
            make_fatal_with_incomplete_multiple_candidate_diagnostic,
            "PlatformBundle report template_resolution fatal multiple-match "
            "diagnostics must include all candidate template_dir values",
        )

    def test_report_rejects_template_resolution_fatal_multiple_candidates_with_no_match_diagnostic(
        self,
    ) -> None:
        def make_fatal_with_contradictory_multiple_candidate_diagnostic(
            resolution: dict[str, object],
        ) -> None:
            candidates = resolution["candidates"]
            self.assertIsInstance(candidates, list)
            second_candidate = dict(candidates[0])
            second_candidate["template_dir"] = str(
                Path(str(resolution["template_root"])) / "second-template"
            )
            candidates.append(second_candidate)
            resolution["fatal"] = True
            resolution["template_dir"] = None
            resolution["diagnostics"] = [
                "multiple export templates matched profile=windows-release: "
                + ", ".join(
                    str(candidate["template_dir"])
                    for candidate in candidates
                    if isinstance(candidate, dict)
                ),
                "no export template under "
                f"{resolution['template_root']} matched profile=windows-release "
                "target_platform=windows-x86_64 engine_version=0.1.0",
            ]

        self._assert_template_resolution_diagnostic(
            make_fatal_with_contradictory_multiple_candidate_diagnostic,
            "PlatformBundle report template_resolution fatal resolution "
            "with multiple candidates must not include root-failure or "
            "no-match diagnostics",
        )

    def test_report_rejects_template_resolution_fatal_no_candidates_without_failure_diagnostic(
        self,
    ) -> None:
        def make_fatal_with_unexplained_no_candidates(
            resolution: dict[str, object],
        ) -> None:
            resolution["candidates"] = []
            resolution["fatal"] = True
            resolution["template_dir"] = None
            resolution["diagnostics"] = ["forced fatal"]

        self._assert_template_resolution_diagnostic(
            make_fatal_with_unexplained_no_candidates,
            "PlatformBundle report template_resolution fatal resolution "
            "with no candidates must include root-failure or no-match diagnostics",
        )

    def test_report_rejects_template_resolution_fatal_no_candidates_with_multiple_match_diagnostic(
        self,
    ) -> None:
        def make_fatal_with_contradictory_no_candidate_diagnostic(
            resolution: dict[str, object],
        ) -> None:
            resolution["candidates"] = []
            resolution["fatal"] = True
            resolution["template_dir"] = None
            resolution["diagnostics"] = [
                "no export template under "
                f"{resolution['template_root']} matched profile=windows-release "
                "target_platform=windows-x86_64 engine_version=0.1.0",
                "multiple export templates matched profile=windows-release: "
                f"{Path(str(resolution['template_root'])) / 'first-template'}, "
                f"{Path(str(resolution['template_root'])) / 'second-template'}",
            ]

        self._assert_template_resolution_diagnostic(
            make_fatal_with_contradictory_no_candidate_diagnostic,
            "PlatformBundle report template_resolution fatal resolution "
            "with no candidates must not include multiple-match diagnostics",
        )

    def test_report_rejects_template_resolution_fatal_no_candidates_with_root_failure_and_no_match(
        self,
    ) -> None:
        def make_fatal_with_two_zero_candidate_failure_families(
            resolution: dict[str, object],
        ) -> None:
            resolution["candidates"] = []
            resolution["skipped_candidates"] = []
            resolution["fatal"] = True
            resolution["template_dir"] = None
            resolution["diagnostics"] = [
                f"export template root {resolution['template_root']} does not exist",
                "no export template under "
                f"{resolution['template_root']} matched profile=windows-release "
                "target_platform=windows-x86_64 engine_version=0.1.0",
            ]

        self._assert_template_resolution_diagnostic(
            make_fatal_with_two_zero_candidate_failure_families,
            "PlatformBundle report template_resolution fatal resolution "
            "with no candidates must not mix root-failure and no-match diagnostics",
        )

    def test_report_rejects_template_resolution_fatal_no_candidates_wrong_profile_no_match_diagnostic(
        self,
    ) -> None:
        def make_fatal_with_wrong_profile_no_match_diagnostic(
            resolution: dict[str, object],
        ) -> None:
            resolution["candidates"] = []
            resolution["skipped_candidates"] = []
            resolution["fatal"] = True
            resolution["template_dir"] = None
            resolution["diagnostics"] = [
                f"no export template under {resolution['template_root']} "
                "matched profile=other-profile target_platform=windows-x86_64 "
                "engine_version=0.1.0"
            ]

        self._assert_template_resolution_diagnostic(
            make_fatal_with_wrong_profile_no_match_diagnostic,
            "PlatformBundle report template_resolution fatal no-match diagnostics "
            "must include profile windows-release",
        )

    def test_report_rejects_template_resolution_fatal_no_candidates_wrong_identity_no_match_diagnostic(
        self,
    ) -> None:
        cases = (
            (
                "target_platform",
                "linux-x86_64",
                "PlatformBundle report template_resolution fatal no-match diagnostics "
                "must include target_platform windows-x86_64",
            ),
            (
                "engine_version",
                "9.9.9",
                "PlatformBundle report template_resolution fatal no-match diagnostics "
                "must include engine_version 0.1.0",
            ),
        )
        for field, value, expected_diagnostic in cases:
            with self.subTest(field=field):
                def mutate(resolution: dict[str, object], field=field, value=value) -> None:
                    target_platform = "windows-x86_64"
                    engine_version = "0.1.0"
                    if field == "target_platform":
                        target_platform = value
                    else:
                        engine_version = value
                    resolution["candidates"] = []
                    resolution["skipped_candidates"] = []
                    resolution["fatal"] = True
                    resolution["template_dir"] = None
                    resolution["diagnostics"] = [
                        f"no export template under {resolution['template_root']} "
                        "matched profile=windows-release "
                        f"target_platform={target_platform} "
                        f"engine_version={engine_version}"
                    ]

                    self._assert_template_resolution_diagnostic(
                        mutate,
                        expected_diagnostic,
                    )

    def test_report_rejects_template_resolution_fatal_no_candidates_wrong_unresolved_identity_marker(
        self,
    ) -> None:
        cases = (
            (
                "expected_target_platform",
                "linux-x86_64",
                "target_platform <any>",
            ),
            (
                "expected_engine_version",
                "9.9.9",
                "engine_version <unresolved>",
            ),
        )
        for field, wrong_value, expected_suffix in cases:
            with self.subTest(field=field):
                def mutate(
                    resolution: dict[str, object],
                    field=field,
                    wrong_value=wrong_value,
                ) -> None:
                    resolution["candidates"] = []
                    resolution["skipped_candidates"] = []
                    resolution["fatal"] = True
                    resolution["template_dir"] = None
                    resolution[field] = None
                    target_platform = (
                        wrong_value
                        if field == "expected_target_platform"
                        else "windows-x86_64"
                    )
                    engine_version = (
                        wrong_value
                        if field == "expected_engine_version"
                        else "0.1.0"
                    )
                    resolution["diagnostics"] = [
                        f"no export template under {resolution['template_root']} "
                        "matched profile=windows-release "
                        f"target_platform={target_platform} "
                        f"engine_version={engine_version}"
                    ]

                self._assert_template_resolution_diagnostic(
                    mutate,
                    "PlatformBundle report template_resolution fatal no-match diagnostics "
                    f"must include {expected_suffix}",
                )

    def test_report_rejects_template_resolution_fatal_no_candidates_wrong_root_no_match_diagnostic(
        self,
    ) -> None:
        def make_fatal_with_wrong_root_no_match_diagnostic(
            resolution: dict[str, object],
        ) -> None:
            resolution["candidates"] = []
            resolution["skipped_candidates"] = []
            resolution["fatal"] = True
            resolution["template_dir"] = None
            wrong_root = Path(str(resolution["template_root"])).parent / "other-templates"
            resolution["diagnostics"] = [
                f"no export template under {wrong_root} "
                "matched profile=windows-release "
                "target_platform=windows-x86_64 "
                "engine_version=0.1.0"
            ]

        self._assert_template_resolution_diagnostic(
            make_fatal_with_wrong_root_no_match_diagnostic,
            "PlatformBundle report template_resolution fatal no-match diagnostics "
            "must include template_root",
        )

    def test_report_rejects_template_resolution_root_failure_with_skipped_candidates(
        self,
    ) -> None:
        def make_root_failure_with_skipped_candidate(
            resolution: dict[str, object],
        ) -> None:
            resolution["candidates"] = []
            resolution["fatal"] = True
            resolution["template_dir"] = None
            resolution["diagnostics"] = [
                f"export template root {resolution['template_root']} does not exist"
            ]

        self._assert_template_resolution_diagnostic(
            make_root_failure_with_skipped_candidate,
            "PlatformBundle report template_resolution root-failure resolution "
            "must not include candidate rows",
        )

    def test_report_rejects_template_resolution_root_failure_wrong_root_diagnostic(
        self,
    ) -> None:
        def make_root_failure_with_wrong_root_diagnostic(
            resolution: dict[str, object],
        ) -> None:
            resolution["candidates"] = []
            resolution["skipped_candidates"] = []
            resolution["fatal"] = True
            resolution["template_dir"] = None
            wrong_root = Path(str(resolution["template_root"])).parent / "other-templates"
            resolution["diagnostics"] = [
                f"export template root {wrong_root} does not exist"
            ]

        self._assert_template_resolution_diagnostic(
            make_root_failure_with_wrong_root_diagnostic,
            "PlatformBundle report template_resolution root-failure diagnostics "
            "must include template_root",
        )

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

    def test_report_rejects_template_resolution_missing_required_field(self) -> None:
        for field in (
            "candidates",
            "diagnostics",
            "expected_engine_version",
            "expected_target_platform",
            "fatal",
            "profile",
            "skipped_candidates",
            "template_dir",
            "template_root",
        ):
            with self.subTest(field=field):
                self._assert_template_resolution_diagnostic(
                    lambda resolution, field=field: resolution.pop(field),
                    f"PlatformBundle report template_resolution.{field} is required",
                )

    def test_report_rejects_template_resolution_required_field_null(self) -> None:
        for field in (
            "candidates",
            "diagnostics",
            "fatal",
            "profile",
            "skipped_candidates",
            "template_root",
        ):
            with self.subTest(field=field):
                self._assert_template_resolution_diagnostic(
                    lambda resolution, field=field: resolution.__setitem__(
                        field,
                        None,
                    ),
                    f"PlatformBundle report template_resolution.{field} is required",
                )

    def test_report_rejects_template_resolution_non_fatal_null_expected_identity(
        self,
    ) -> None:
        for field in ("expected_engine_version", "expected_target_platform"):
            with self.subTest(field=field):
                self._assert_template_resolution_diagnostic(
                    lambda resolution, field=field: resolution.__setitem__(
                        field,
                        None,
                    ),
                    "PlatformBundle report template_resolution non-fatal "
                    f"resolution must include {field}",
                )

    def test_report_rejects_template_resolution_profile_mismatch(self) -> None:
        def rewrite_resolution_profile(resolution: dict[str, object]) -> None:
            resolution["profile"] = "other-profile"
            candidate = resolution["candidates"][0]
            self.assertIsInstance(candidate, dict)
            candidate["compatible_profiles"] = ["other-profile"]

        self._assert_template_resolution_diagnostic(
            rewrite_resolution_profile,
            "PlatformBundle report template_resolution.profile "
            "must match PlatformBundle report profile windows-release",
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

    def test_report_rejects_template_resolution_non_fatal_multiple_candidates(
        self,
    ) -> None:
        def add_candidate(resolution: dict[str, object]) -> None:
            candidates = resolution["candidates"]
            self.assertIsInstance(candidates, list)
            second_candidate = dict(candidates[0])
            second_candidate["template_dir"] = str(
                Path(str(resolution["template_root"])) / "second-template"
            )
            candidates.append(second_candidate)

        self._assert_template_resolution_diagnostic(
            add_candidate,
            "PlatformBundle report template_resolution non-fatal resolution "
            "must contain exactly one candidate",
        )

    def test_report_rejects_template_resolution_selected_candidate_mismatch(
        self,
    ) -> None:
        self._assert_template_resolution_diagnostic(
            lambda resolution: resolution.__setitem__(
                "template_dir",
                str(Path(str(resolution["template_root"])) / "other-template"),
            ),
            "PlatformBundle report template_resolution.template_dir "
            "must match exactly one candidates[].template_dir",
        )

    def test_report_rejects_template_resolution_template_dir_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out, with_template_file=True)
            platform_report = _read_stage_report(out, "platform_bundle")
            resolution = _template_resolution(out)
            platform_report["template_resolution"] = resolution
            platform_report["template_files"] = []
            template = platform_report["template"]
            self.assertIsInstance(template, dict)
            alternate_template_dir = out / "alternate-template"
            alternate_template_dir.mkdir(parents=True)
            (alternate_template_dir / "Info.plist").write_text(
                "<plist>zircon</plist>",
                encoding="utf-8",
            )
            template["template_dir"] = str(alternate_template_dir)
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report template_resolution.template_dir "
                    "must match template.template_dir" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
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

    def test_report_rejects_template_resolution_string_field_blank(self) -> None:
        for field in (
            "expected_engine_version",
            "expected_target_platform",
            "profile",
            "template_dir",
            "template_root",
        ):
            with self.subTest(field=field):
                self._assert_template_resolution_diagnostic(
                    lambda resolution, field=field: resolution.__setitem__(field, " "),
                    f"PlatformBundle report template_resolution.{field} must be a non-empty string",
                )

    def test_report_rejects_template_resolution_padded_string_field(self) -> None:
        for field in (
            "expected_engine_version",
            "expected_target_platform",
            "profile",
            "template_dir",
            "template_root",
        ):
            with self.subTest(field=field):
                self._assert_template_resolution_diagnostic(
                    lambda resolution, field=field: resolution.__setitem__(
                        field,
                        f" {resolution[field]} ",
                    ),
                    f"PlatformBundle report template_resolution.{field} must be a non-empty trimmed string",
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

    def _assert_template_resolution_diagnostic(
        self,
        mutate_resolution: Callable[[dict[str, object]], None],
        expected_diagnostic: str,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out, with_template_file=True)
            platform_report = _read_stage_report(out, "platform_bundle")
            resolution = _template_resolution(out)
            mutate_resolution(resolution)
            platform_report["template_resolution"] = resolution
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    expected_diagnostic in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


def _template_resolution(out: Path) -> dict[str, object]:
    template_root = out
    template_dir = out / "template"
    candidate = {
        "template_dir": str(template_dir),
        "template_id": "windows-template",
        "engine_version": "0.1.0",
        "target_platform": "windows-x86_64",
        "compatible_profiles": ["windows-release"],
        "host_artifact": "precompiled",
        "bundle_format": "directory",
    }
    return {
        "template_root": str(template_root),
        "profile": "windows-release",
        "expected_engine_version": "0.1.0",
        "expected_target_platform": "windows-x86_64",
        "fatal": False,
        "diagnostics": [],
        "candidates": [deepcopy(candidate)],
        "skipped_candidates": [
            {
                "template_dir": str(out / "broken-template"),
                "diagnostics": ["template format_version 999 is not supported"],
            }
        ],
        "template_dir": str(template_dir),
    }
