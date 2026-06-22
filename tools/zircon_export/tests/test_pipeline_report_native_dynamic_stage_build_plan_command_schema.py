from __future__ import annotations

import unittest

from tools.zircon_export.tests.native_dynamic_stage_schema_test_support import (
    NativeDynamicStageSchemaReportAssertions,
    _native_build_plan,
    _native_build_plan_package,
)


class PipelineReportNativeDynamicStageBuildPlanCommandSchemaTests(
    NativeDynamicStageSchemaReportAssertions,
    unittest.TestCase,
):

    def test_report_stage_rejects_native_dynamic_build_plan_command_manifest_mismatch(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_plan",
            _native_build_plan(
                packages=[
                    _native_build_plan_package(
                        command=[
                            "cargo",
                            "build",
                            "--manifest-path",
                            "forged/Cargo.toml",
                        ]
                    )
                ]
            ),
            "native_dynamic report native_build_plan.packages[0].command "
            "--manifest-path forged/Cargo.toml does not match "
            "native_dynamic report native_build_plan.packages[0]."
            "workspace_manifest zircon_plugins/Cargo.toml",
        )

    def test_report_stage_rejects_native_dynamic_build_plan_command_package_mismatch(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_plan",
            _native_build_plan(
                packages=[
                    _native_build_plan_package(
                        command=[
                            "cargo",
                            "build",
                            "--manifest-path",
                            "zircon_plugins/Cargo.toml",
                            "-p",
                            "forged_plugin_native",
                        ]
                    )
                ]
            ),
            "native_dynamic report native_build_plan.packages[0].command "
            "-p/--package forged_plugin_native does not match "
            "native_dynamic report native_build_plan.packages[0]."
            "crate_name zircon_plugin_animation_native",
        )

    def test_report_stage_rejects_native_dynamic_build_plan_command_target_dir_mismatch(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_plan",
            _native_build_plan(
                packages=[
                    _native_build_plan_package(
                        command=[
                            "cargo",
                            "build",
                            "--manifest-path",
                            "zircon_plugins/Cargo.toml",
                            "-p",
                            "zircon_plugin_animation_native",
                            "--target-dir",
                            "forged/target",
                        ]
                    )
                ]
            ),
            "native_dynamic report native_build_plan.packages[0].command "
            "--target-dir forged/target does not match "
            "native_dynamic report native_build_plan.packages[0]."
            "target_dir target/native_dynamic",
        )

    def test_report_stage_rejects_native_dynamic_build_plan_command_missing_release_flag(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_plan",
            _native_build_plan(
                packages=[
                    _native_build_plan_package(
                        command=[
                            "cargo",
                            "build",
                            "--manifest-path",
                            "zircon_plugins/Cargo.toml",
                            "-p",
                            "zircon_plugin_animation_native",
                            "--target-dir",
                            "target/native_dynamic",
                        ]
                    )
                ]
            ),
            "native_dynamic report native_build_plan.packages[0].command "
            "must include --release",
        )

    def test_report_stage_rejects_native_dynamic_build_plan_command_missing_features(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_plan",
            _native_build_plan(
                packages=[
                    _native_build_plan_package(
                        command=[
                            "cargo",
                            "build",
                            "--manifest-path",
                            "zircon_plugins/Cargo.toml",
                            "-p",
                            "zircon_plugin_animation_native",
                            "--target-dir",
                            "target/native_dynamic",
                            "--release",
                        ]
                    )
                ]
            ),
            "native_dynamic report native_build_plan.packages[0].command "
            "must include --features",
        )

    def test_report_stage_rejects_native_dynamic_build_plan_command_feature_broadening(
        self,
    ) -> None:
        package = _native_build_plan_package()
        command = [*package["command"], "--all-features"]
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_plan",
            _native_build_plan(
                packages=[
                    _native_build_plan_package(command=command),
                ]
            ),
            "native_dynamic report native_build_plan.packages[0].command "
            "must not include --all-features because "
            "native_build_plan.packages[0].features owns feature selection",
        )

    def test_report_stage_rejects_native_dynamic_build_plan_command_no_default_features_override(
        self,
    ) -> None:
        package = _native_build_plan_package()
        command = [*package["command"], "--no-default-features"]
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_plan",
            _native_build_plan(
                packages=[
                    _native_build_plan_package(command=command),
                ]
            ),
            "native_dynamic report native_build_plan.packages[0].command "
            "must not include --no-default-features because "
            "native_build_plan.packages[0].features owns feature selection",
        )

    def test_report_stage_rejects_native_dynamic_build_plan_command_target_broadening(
        self,
    ) -> None:
        cases = (
            (
                "--all-targets",
                "native_dynamic report native_build_plan.packages[0].command "
                "must not include --all-targets because "
                "native_build_plan.packages[0].crate_name owns the single "
                "native build target",
            ),
            (
                "--bins",
                "native_dynamic report native_build_plan.packages[0].command "
                "must not include --bins because "
                "native_build_plan.packages[0].crate_name owns the single "
                "native build target",
            ),
        )
        package = _native_build_plan_package()
        for flag, expected_diagnostic in cases:
            with self.subTest(flag=flag):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_plan",
                    _native_build_plan(
                        packages=[
                            _native_build_plan_package(
                                command=[*package["command"], flag]
                            ),
                        ]
                    ),
                    expected_diagnostic,
                )

    def test_report_stage_rejects_native_dynamic_build_plan_command_package_broadening(
        self,
    ) -> None:
        cases = (
            "--workspace",
            "--all",
            "--exclude",
        )
        package = _native_build_plan_package()
        for flag in cases:
            with self.subTest(flag=flag):
                command = [*package["command"], flag]
                if flag == "--exclude":
                    command.append("zircon_plugin_physics_native")
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_plan",
                    _native_build_plan(
                        packages=[
                            _native_build_plan_package(command=command),
                        ]
                    ),
                    "native_dynamic report native_build_plan.packages[0].command "
                    f"must not include {flag} because "
                    "native_build_plan.packages[0].crate_name owns package "
                    "selection",
                )

    def test_report_stage_rejects_native_dynamic_build_plan_command_profile_override(
        self,
    ) -> None:
        package = _native_build_plan_package()
        for command in (
            [*package["command"], "--profile", "shipping"],
            [*package["command"], "--profile=shipping"],
        ):
            with self.subTest(command=command):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_plan",
                    _native_build_plan(
                        packages=[
                            _native_build_plan_package(command=command),
                        ]
                    ),
                    "native_dynamic report native_build_plan.packages[0].command "
                    "must not include --profile because "
                    "native_build_plan.packages[0].cargo_profile/release "
                    "owns profile selection",
                )

    def test_report_stage_rejects_native_dynamic_build_plan_command_non_cargo(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_plan",
            _native_build_plan(
                packages=[
                    _native_build_plan_package(
                        command=[
                            "python",
                            "build",
                            "--manifest-path",
                            "zircon_plugins/Cargo.toml",
                            "-p",
                            "zircon_plugin_animation_native",
                            "--target-dir",
                            "target/native_dynamic",
                            "--features",
                            "v3_fixture_diagnostics",
                            "--release",
                        ]
                    )
                ]
            ),
            "native_dynamic report native_build_plan.packages[0].command[0] "
            "must be cargo",
        )

    def test_report_stage_rejects_native_dynamic_build_plan_command_non_string_entry_before_array_shape(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_plan",
            _native_build_plan(
                packages=[
                    _native_build_plan_package(
                        command=[
                            "cargo",
                            42,
                            "--manifest-path",
                            "zircon_plugins/Cargo.toml",
                            "-p",
                            "zircon_plugin_animation_native",
                            "--target-dir",
                            "target/native_dynamic",
                            "--features",
                            "v3_fixture_diagnostics",
                            "--release",
                        ]
                    )
                ]
            ),
            "native_dynamic report native_build_plan.packages[0].command[1] "
            "must be a string",
            "native_dynamic report native_build_plan.packages[0].command "
            "must be a string array",
        )

    def test_report_stage_rejects_native_dynamic_build_plan_command_shape_before_semantics(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_plan",
            _native_build_plan(
                packages=[
                    _native_build_plan_package(
                        command=[
                            " cargo ",
                            "build",
                            "--manifest-path",
                            "zircon_plugins/Cargo.toml",
                            "-p",
                            "zircon_plugin_animation_native",
                            "--target-dir",
                            "target/native_dynamic",
                            "--features",
                            "v3_fixture_diagnostics",
                            "--release",
                        ]
                    )
                ]
            ),
            "native_dynamic report native_build_plan.packages[0].command[0] "
            "must be a non-empty trimmed string",
            "native_dynamic report native_build_plan.packages[0].command[0] "
            "must be cargo",
        )


if __name__ == "__main__":
    unittest.main()
