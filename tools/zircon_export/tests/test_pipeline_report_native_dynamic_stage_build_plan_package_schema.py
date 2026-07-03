from __future__ import annotations

import unittest

from tools.zircon_export.tests.native_dynamic_stage_schema_test_support import (
    NativeDynamicStageSchemaReportAssertions,
    _native_build_plan,
    _native_build_plan_package,
    _native_build_plan_package_with_features,
    _native_build_plan_package_without,
)


class PipelineReportNativeDynamicStageBuildPlanPackageSchemaTests(
    NativeDynamicStageSchemaReportAssertions,
    unittest.TestCase,
):
    def test_report_stage_rejects_native_dynamic_build_plan_package_unknown_field(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_plan",
            _native_build_plan(
                packages=[
                    _native_build_plan_package(
                        unsigned_sidecar="target/sidecar.bin"
                    )
                ]
            ),
            "native_dynamic report native_build_plan.packages[0] "
            "unknown field unsigned_sidecar",
        )

    def test_report_stage_rejects_native_dynamic_build_plan_package_missing_required_field(
        self,
    ) -> None:
        cases = (
            (
                "package_id",
                "native_dynamic report native_build_plan.packages[0].package_id must be a string",
            ),
            (
                "crate_name",
                "native_dynamic report native_build_plan.packages[0].crate_name must be a string",
            ),
            (
                "manifest_path",
                "native_dynamic report native_build_plan.packages[0].manifest_path must be a string",
            ),
            (
                "workspace_manifest",
                "native_dynamic report native_build_plan.packages[0].workspace_manifest must be a string",
            ),
            (
                "target_dir",
                "native_dynamic report native_build_plan.packages[0].target_dir must be a string",
            ),
            (
                "cargo_profile",
                "native_dynamic report native_build_plan.packages[0].cargo_profile must be a string",
            ),
            (
                "expected_loadable_artifact",
                "native_dynamic report native_build_plan.packages[0].expected_loadable_artifact must be a string",
            ),
            (
                "release",
                "native_dynamic report native_build_plan.packages[0].release must be a boolean",
            ),
            (
                "features",
                "native_dynamic report native_build_plan.packages[0].features must be a string array",
            ),
            (
                "command",
                "native_dynamic report native_build_plan.packages[0].command must be a string array",
            ),
        )
        for field, expected_diagnostic in cases:
            with self.subTest(field=field):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_plan",
                    _native_build_plan(
                        packages=[_native_build_plan_package_without(field)]
                    ),
                    expected_diagnostic,
                )

    def test_report_stage_rejects_native_dynamic_build_plan_package_empty_required_string_field(
        self,
    ) -> None:
        for field in (
            "package_id",
            "crate_name",
            "manifest_path",
            "workspace_manifest",
            "target_dir",
            "cargo_profile",
            "expected_loadable_artifact",
        ):
            with self.subTest(field=field):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_plan",
                    _native_build_plan(
                        packages=[_native_build_plan_package(**{field: ""})]
                    ),
                    "native_dynamic report "
                    f"native_build_plan.packages[0].{field} "
                    "must be a non-empty string",
                )

    def test_report_stage_rejects_native_dynamic_build_plan_package_blank_required_string_field(
        self,
    ) -> None:
        for field in (
            "package_id",
            "crate_name",
            "manifest_path",
            "workspace_manifest",
            "target_dir",
            "cargo_profile",
            "expected_loadable_artifact",
        ):
            with self.subTest(field=field):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_plan",
                    _native_build_plan(
                        packages=[_native_build_plan_package(**{field: "   "})]
                    ),
                    "native_dynamic report "
                    f"native_build_plan.packages[0].{field} "
                    "must be a non-empty string",
                )

    def test_report_stage_rejects_native_dynamic_build_plan_package_padded_required_string_field_before_semantics(
        self,
    ) -> None:
        cases = (
            (
                "workspace_manifest",
                " zircon_plugins/Cargo.toml ",
                "native_dynamic report native_build_plan.packages[0].command "
                "--manifest-path zircon_plugins/Cargo.toml does not match",
            ),
            (
                "target_dir",
                " target/native_dynamic ",
                "native_dynamic report native_build_plan.packages[0].command "
                "--target-dir target/native_dynamic does not match",
            ),
            (
                "crate_name",
                " zircon_plugin_animation_native ",
                "native_dynamic report native_build_plan.packages[0].command "
                "-p/--package zircon_plugin_animation_native does not match",
            ),
            (
                "cargo_profile",
                " release ",
                "native_dynamic report native_build_plan.packages[0]."
                "cargo_profile  release  does not match",
            ),
            (
                "expected_loadable_artifact",
                " target/native_dynamic/release/zircon_plugin_animation_native.dll ",
                "native_dynamic report native_build_plan.packages[0]."
                "expected_loadable_artifact  target/native_dynamic/release/"
                "zircon_plugin_animation_native.dll  does not match "
                "derived artifact",
            ),
        )
        for field, value, unexpected_diagnostic in cases:
            with self.subTest(field=field):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_plan",
                    _native_build_plan(
                        packages=[
                            _native_build_plan_package(**{field: value})
                        ]
                    ),
                    "native_dynamic report "
                    f"native_build_plan.packages[0].{field} "
                    "must be a non-empty trimmed string",
                    unexpected_diagnostic,
                )

    def test_report_stage_rejects_native_dynamic_build_plan_package_empty_command(
        self,
    ) -> None:
        for command in ([], ["cargo", ""], ["cargo", "   "]):
            with self.subTest(command=command):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_plan",
                    _native_build_plan(
                        packages=[
                            _native_build_plan_package(command=command)
                        ]
                    ),
                    "native_dynamic report native_build_plan.packages[0].command "
                    "must be a non-empty string array",
                )

    def test_report_stage_rejects_native_dynamic_build_plan_package_header_mismatch(
        self,
    ) -> None:
        cases = (
            (
                "workspace_manifest",
                "forged/Cargo.toml",
                {
                    "workspace_manifest": "forged/Cargo.toml",
                    "command": [
                        "cargo",
                        "build",
                        "--manifest-path",
                        "forged/Cargo.toml",
                        "-p",
                        "zircon_plugin_animation_native",
                        "--target-dir",
                        "target/native_dynamic",
                        "--features",
                        "v3_fixture_diagnostics",
                        "--release",
                    ],
                },
                "zircon_plugins/Cargo.toml",
            ),
            (
                "target_dir",
                "forged/target",
                {
                    "target_dir": "forged/target",
                    "command": [
                        "cargo",
                        "build",
                        "--manifest-path",
                        "zircon_plugins/Cargo.toml",
                        "-p",
                        "zircon_plugin_animation_native",
                        "--target-dir",
                        "forged/target",
                        "--features",
                        "v3_fixture_diagnostics",
                        "--release",
                    ],
                },
                "target/native_dynamic",
            ),
            (
                "cargo_profile",
                "debug",
                {
                    "cargo_profile": "debug",
                },
                "release",
            ),
            (
                "release",
                False,
                {
                    "release": False,
                    "command": [
                        "cargo",
                        "build",
                        "--manifest-path",
                        "zircon_plugins/Cargo.toml",
                        "-p",
                        "zircon_plugin_animation_native",
                        "--target-dir",
                        "target/native_dynamic",
                        "--features",
                        "v3_fixture_diagnostics",
                    ],
                },
                True,
            ),
            (
                "features",
                ["abi_v2_only"],
                {
                    "features": ["abi_v2_only"],
                    "command": [
                        "cargo",
                        "build",
                        "--manifest-path",
                        "zircon_plugins/Cargo.toml",
                        "-p",
                        "zircon_plugin_animation_native",
                        "--target-dir",
                        "target/native_dynamic",
                        "--features",
                        "abi_v2_only",
                        "--release",
                    ],
                },
                ["v3_fixture_diagnostics"],
            ),
        )
        for field, package_value, package_overrides, plan_value in cases:
            with self.subTest(field=field):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_plan",
                    _native_build_plan(
                        packages=[
                            _native_build_plan_package(**package_overrides)
                        ]
                    ),
                    "native_dynamic report native_build_plan.packages[0]."
                    f"{field} {package_value} does not match "
                    "native_dynamic report native_build_plan."
                    f"{'build_features' if field == 'features' else field} "
                    f"{plan_value}",
                )

    def test_report_stage_rejects_native_dynamic_build_plan_expected_artifact_mismatch(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_plan",
            _native_build_plan(
                packages=[
                    _native_build_plan_package(
                        expected_loadable_artifact=(
                            "target/native_dynamic/release/forged.dll"
                        )
                    )
                ]
            ),
            "native_dynamic report native_build_plan.packages[0]."
            "expected_loadable_artifact "
            "target/native_dynamic/release/forged.dll does not match "
            "derived artifact "
            "target/native_dynamic/release/zircon_plugin_animation_native.dll",
        )

    def test_report_stage_rejects_native_dynamic_build_plan_package_blank_feature_entry(
        self,
    ) -> None:
        for features in (
            [""],
            ["   "],
            ["v3_fixture_diagnostics", ""],
        ):
            with self.subTest(features=features):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_plan",
                    _native_build_plan(
                        packages=[
                            _native_build_plan_package(features=features)
                        ]
                    ),
                    "native_dynamic report native_build_plan.packages[0].features "
                    "must not contain blank entries",
                )

    def test_report_stage_rejects_native_dynamic_build_plan_package_padded_feature_entry(
        self,
    ) -> None:
        package = _native_build_plan_package_with_features(
            [" v3_fixture_diagnostics "]
        )
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_plan",
            _native_build_plan(
                build_features=[" v3_fixture_diagnostics "],
                packages=[package],
            ),
            "native_dynamic report native_build_plan.packages[0].features[0] "
            "must be a non-empty trimmed string",
        )

    def test_report_stage_rejects_native_dynamic_build_plan_package_duplicate_feature_entry(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_plan",
            _native_build_plan(
                packages=[
                    _native_build_plan_package(
                        features=[
                            "v3_fixture_diagnostics",
                            "v3_fixture_diagnostics",
                        ]
                    )
                ]
            ),
            "native_dynamic report native_build_plan.packages[0].features "
            "must not contain duplicate entries",
        )

    def test_report_stage_rejects_native_dynamic_build_plan_package_non_string_feature_entry_before_array_shape(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_plan",
            _native_build_plan(
                packages=[
                    _native_build_plan_package(
                        features=["v3_fixture_diagnostics", 42]
                    )
                ]
            ),
            "native_dynamic report native_build_plan.packages[0].features[1] "
            "must be a string",
            "native_dynamic report native_build_plan.packages[0].features "
            "must be a string array",
        )

    def test_report_stage_rejects_native_dynamic_build_plan_package_padded_duplicate_feature_before_uniqueness(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_plan",
            _native_build_plan(
                packages=[
                    _native_build_plan_package_with_features(
                        [
                            " v3_fixture_diagnostics ",
                            " v3_fixture_diagnostics ",
                        ]
                    )
                ]
            ),
            "native_dynamic report native_build_plan.packages[0].features[0] "
            "must be a non-empty trimmed string",
            (
                "native_dynamic report native_build_plan.packages[0].features "
                "must not contain duplicate entries"
            ),
        )

    def test_report_stage_rejects_native_dynamic_build_plan_package_field_types(
        self,
    ) -> None:
        cases = (
            ("package_id", 42, "must be a string"),
            ("crate_name", 42, "must be a string"),
            ("manifest_path", 42, "must be a string"),
            ("workspace_manifest", 42, "must be a string"),
            ("target_dir", 42, "must be a string"),
            ("cargo_profile", 42, "must be a string"),
            ("expected_loadable_artifact", 42, "must be a string"),
            ("release", "true", "must be a boolean"),
            ("features", 42, "must be a string array"),
            ("features[0]", [42], "must be a string"),
            ("command", 42, "must be a string array"),
            ("command[0]", [42], "must be a string"),
        )
        for expected_field, value, expected_type in cases:
            field = expected_field.split("[", maxsplit=1)[0]
            with self.subTest(field=expected_field, value=value):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_plan",
                    _native_build_plan(
                        packages=[_native_build_plan_package(**{field: value})]
                    ),
                    "native_dynamic report native_build_plan."
                    f"packages[0].{expected_field} {expected_type}",
                )


if __name__ == "__main__":
    unittest.main()
