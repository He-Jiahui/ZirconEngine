from __future__ import annotations

import unittest

from tools.zircon_export.tests.native_dynamic_stage_schema_test_support import (
    NativeDynamicStageSchemaReportAssertions,
    _native_build_execution,
    _native_build_execution_package,
    _native_build_execution_package_for_default_report,
    _native_build_execution_package_without,
    _native_build_execution_without,
)


class PipelineReportNativeDynamicStageBuildExecutionSchemaTests(
    NativeDynamicStageSchemaReportAssertions,
    unittest.TestCase,
):

    def test_report_stage_rejects_native_dynamic_build_execution_unknown_field(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_execution",
            _native_build_execution(unsigned_sidecar="target/sidecar.bin"),
            "native_dynamic report native_build_execution unknown field unsigned_sidecar",
        )

    def test_report_stage_rejects_native_dynamic_build_execution_missing_release_evidence_field(
        self,
    ) -> None:
        cases = (
            (
                "enabled",
                "native_dynamic report native_build_execution.enabled must be a boolean",
            ),
            (
                "fatal",
                "native_dynamic report native_build_execution.fatal must be a boolean",
            ),
            (
                "skipped",
                "native_dynamic report native_build_execution.skipped must be a boolean",
            ),
            (
                "diagnostics",
                "native_dynamic report native_build_execution.diagnostics must be a string array",
            ),
            (
                "package_count",
                "native_dynamic report native_build_execution.package_count must be an integer",
            ),
            (
                "packages",
                "native_dynamic report native_build_execution.packages must be an object array",
            ),
        )
        for field, expected_diagnostic in cases:
            with self.subTest(field=field):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_execution",
                    _native_build_execution_without(field),
                    expected_diagnostic,
                )

    def test_report_stage_rejects_native_dynamic_build_execution_field_types(
        self,
    ) -> None:
        cases = (
            ("enabled", "true", "must be a boolean"),
            ("fatal", "false", "must be a boolean"),
            ("skipped", "true", "must be a boolean"),
            ("skip_reason", 42, "must be a string"),
            ("diagnostics", 42, "must be a string array"),
            ("diagnostics[0]", [42], "must be a string"),
            ("package_count", "1", "must be an integer"),
            ("packages", "not-an-array", "must be an object array"),
            ("packages[0]", [42], "must be an object"),
        )
        for expected_field, value, expected_type in cases:
            field = expected_field.split("[", maxsplit=1)[0]
            with self.subTest(field=expected_field, value=value):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_execution",
                    _native_build_execution(**{field: value}),
                    "native_dynamic report "
                    f"native_build_execution.{expected_field} {expected_type}",
                )

    def test_report_stage_rejects_native_dynamic_build_execution_blank_skip_reason(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_execution",
            _native_build_execution(skip_reason="   "),
            "native_dynamic report native_build_execution.skip_reason "
            "must be a non-empty string",
        )

    def test_report_stage_rejects_native_dynamic_build_execution_padded_skip_reason(
        self,
    ) -> None:
        def mutate_report(native_report: dict[str, object]) -> None:
            native_report["fatal"] = True
            native_report["diagnostics"] = ["materialization failed"]
            native_build_execution = native_report["native_build_execution"]
            self.assertIsInstance(native_build_execution, dict)
            native_build_execution["enabled"] = True
            native_build_execution["fatal"] = False
            native_build_execution["skipped"] = True
            native_build_execution["skip_reason"] = " materialization diagnostics "
            native_build_execution["package_count"] = 0
            native_build_execution["packages"] = []

        self._assert_native_dynamic_report_mutation_diagnostic(
            mutate_report,
            "native_dynamic report native_build_execution.skip_reason "
            "must be a non-empty trimmed string",
        )

    def test_report_stage_rejects_native_dynamic_build_execution_skip_reason_non_skipped_mismatch(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_execution",
            _native_build_execution(skip_reason="not requested"),
            "native_dynamic report native_build_execution.skip_reason "
            "must be absent when skipped is False",
        )

    def test_report_stage_rejects_native_dynamic_build_execution_skipped_without_reason(
        self,
    ) -> None:
        def mutate_report(native_report: dict[str, object]) -> None:
            native_report["fatal"] = True
            native_report["diagnostics"] = ["materialization failed"]
            native_build_execution = native_report["native_build_execution"]
            self.assertIsInstance(native_build_execution, dict)
            native_build_execution["skipped"] = True
            native_build_execution.pop("skip_reason", None)

        self._assert_native_dynamic_report_mutation_diagnostic(
            mutate_report,
            "native_dynamic report native_build_execution.skip_reason "
            "must be a non-empty string when skipped is True",
        )

    def test_report_stage_rejects_native_dynamic_build_execution_skipped_non_empty_table(
        self,
    ) -> None:
        def mutate_report(native_report: dict[str, object]) -> None:
            native_report["fatal"] = True
            native_report["diagnostics"] = ["materialization failed"]
            native_build_execution = native_report["native_build_execution"]
            self.assertIsInstance(native_build_execution, dict)
            native_build_execution["enabled"] = True
            native_build_execution["fatal"] = False
            native_build_execution["skipped"] = True
            native_build_execution["skip_reason"] = "materialization diagnostics"
            native_build_execution["package_count"] = 1
            native_build_execution["packages"] = [
                _native_build_execution_package_for_default_report()
            ]

        self._assert_native_dynamic_report_mutation_diagnostic(
            mutate_report,
            (
                "native_dynamic report native_build_execution.package_count "
                "must be 0 when skipped is True",
                "native_dynamic report native_build_execution.packages "
                "must be empty when skipped is True",
            ),
        )

    def test_report_stage_rejects_native_dynamic_build_execution_skipped_state_mismatch(
        self,
    ) -> None:
        cases = (
            (
                "disabled",
                {"enabled": False},
                "native_dynamic report native_build_execution.enabled "
                "must be True when skipped is True",
            ),
            (
                "fatal",
                {"fatal": True, "diagnostics": ["cargo build failed"]},
                "native_dynamic report native_build_execution.fatal "
                "must be False when skipped is True",
            ),
        )
        for case_name, execution_overrides, expected_diagnostic in cases:
            with self.subTest(case=case_name):

                def mutate_report(native_report: dict[str, object]) -> None:
                    native_report["fatal"] = True
                    native_report["diagnostics"] = ["materialization failed"]
                    native_build_execution = native_report["native_build_execution"]
                    self.assertIsInstance(native_build_execution, dict)
                    native_build_execution["enabled"] = True
                    native_build_execution["fatal"] = False
                    native_build_execution["skipped"] = True
                    native_build_execution["skip_reason"] = (
                        "materialization diagnostics"
                    )
                    native_build_execution["package_count"] = 0
                    native_build_execution["packages"] = []
                    native_build_execution.update(execution_overrides)

                self._assert_native_dynamic_report_mutation_diagnostic(
                    mutate_report,
                    expected_diagnostic,
                )

    def test_report_stage_rejects_native_dynamic_build_execution_negative_count_fields(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_execution",
            _native_build_execution(package_count=-1),
            "native_dynamic report native_build_execution.package_count "
            "must be non-negative",
        )

    def test_report_stage_rejects_native_dynamic_build_execution_disabled_non_empty_table(
        self,
    ) -> None:
        execution = _native_build_execution(
            enabled=False,
            package_count=1,
            packages=[_native_build_execution_package_for_default_report()],
        )
        cases = (
            (
                "package_count",
                "native_dynamic report native_build_execution.package_count "
                "must be 0 when enabled is False",
            ),
            (
                "packages",
                "native_dynamic report native_build_execution.packages "
                "must be empty when enabled is False",
            ),
        )
        for case_name, expected_diagnostic in cases:
            with self.subTest(case=case_name):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_execution",
                    execution,
                    expected_diagnostic,
                )

    def test_report_stage_rejects_native_dynamic_build_execution_fatal_success_report_mismatch(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_execution",
            _native_build_execution(
                fatal=True,
                diagnostics=["cargo build failed"],
                packages=[
                    _native_build_execution_package_for_default_report(
                        exit_code=1
                    )
                ],
            ),
            "native_dynamic report native_build_execution.fatal "
            "must be False when NativeDynamic report fatal is False",
        )

    def test_report_stage_rejects_native_dynamic_build_execution_fatal_without_diagnostics(
        self,
    ) -> None:
        def mutate_report(native_report: dict[str, object]) -> None:
            native_report["fatal"] = True
            native_report["diagnostics"] = ["native build execution failed"]
            native_build_execution = native_report["native_build_execution"]
            self.assertIsInstance(native_build_execution, dict)
            native_build_execution["fatal"] = True
            native_build_execution["diagnostics"] = []

        self._assert_native_dynamic_report_mutation_diagnostic(
            mutate_report,
            "native_dynamic report native_build_execution fatal report "
            "must include diagnostics",
        )

    def test_report_stage_rejects_native_dynamic_build_execution_non_fatal_with_diagnostics(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_execution",
            _native_build_execution(
                diagnostics=["cargo build warning"],
                packages=[_native_build_execution_package_for_default_report()],
            ),
            "native_dynamic report native_build_execution.diagnostics "
            "must be empty when fatal is False",
        )

    def test_report_stage_rejects_native_dynamic_build_execution_non_string_diagnostic_entry_before_array_shape(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_execution",
            _native_build_execution(
                diagnostics=[42],
                packages=[_native_build_execution_package_for_default_report()],
            ),
            "native_dynamic report native_build_execution.diagnostics[0] "
            "must be a string",
            "native_dynamic report native_build_execution.diagnostics "
            "must be a string array",
        )

    def test_report_stage_rejects_native_dynamic_build_execution_skipped_success_report_mismatch(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_execution",
            _native_build_execution(
                enabled=False,
                package_count=0,
                packages=[],
                skipped=True,
                skip_reason="not requested",
            ),
            "native_dynamic report native_build_execution.skipped "
            "must be False when NativeDynamic report fatal is False",
        )

    def test_report_stage_rejects_native_dynamic_build_execution_blank_diagnostic_entry(
        self,
    ) -> None:
        for diagnostics in ([""], ["   "], ["build failed", ""]):
            with self.subTest(diagnostics=diagnostics):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_execution",
                    _native_build_execution(diagnostics=diagnostics),
                    "native_dynamic report native_build_execution.diagnostics "
                    "must not contain blank entries",
                )

    def test_report_stage_rejects_native_dynamic_build_execution_padded_diagnostic_entry(
        self,
    ) -> None:
        def mutate_report(native_report: dict[str, object]) -> None:
            native_report["fatal"] = True
            native_report["diagnostics"] = ["native build execution failed"]
            native_build_execution = native_report["native_build_execution"]
            self.assertIsInstance(native_build_execution, dict)
            native_build_execution["fatal"] = True
            native_build_execution["diagnostics"] = [" cargo build failed "]

        self._assert_native_dynamic_report_mutation_diagnostic(
            mutate_report,
            "native_dynamic report native_build_execution.diagnostics[0] "
            "must be a non-empty trimmed string",
        )

    def test_report_stage_rejects_native_dynamic_build_execution_package_unknown_field(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_execution",
            _native_build_execution(
                packages=[
                    _native_build_execution_package(
                        unsigned_sidecar="target/sidecar.bin"
                    )
                ]
            ),
            "native_dynamic report native_build_execution.packages[0] "
            "unknown field unsigned_sidecar",
        )

    def test_report_stage_rejects_native_dynamic_build_execution_package_missing_required_field(
        self,
    ) -> None:
        cases = (
            (
                "package_id",
                "native_dynamic report native_build_execution.packages[0].package_id must be a string",
            ),
            (
                "crate_name",
                "native_dynamic report native_build_execution.packages[0].crate_name must be a string",
            ),
            (
                "command",
                "native_dynamic report native_build_execution.packages[0].command must be a string array",
            ),
            (
                "exit_code",
                "native_dynamic report native_build_execution.packages[0].exit_code must be an integer",
            ),
            (
                "stdout",
                "native_dynamic report native_build_execution.packages[0].stdout must be a string",
            ),
            (
                "stderr",
                "native_dynamic report native_build_execution.packages[0].stderr must be a string",
            ),
            (
                "expected_loadable_artifact",
                "native_dynamic report native_build_execution.packages[0].expected_loadable_artifact must be a string",
            ),
            (
                "copied_loadable_artifact",
                "native_dynamic report native_build_execution.packages[0].copied_loadable_artifact must be a string",
            ),
            (
                "copied_sidecars",
                "native_dynamic report native_build_execution.packages[0].copied_sidecars must be a string array",
            ),
        )
        for field, expected_diagnostic in cases:
            with self.subTest(field=field):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_execution",
                    _native_build_execution(
                        packages=[_native_build_execution_package_without(field)]
                    ),
                    expected_diagnostic,
                )

    def test_report_stage_rejects_native_dynamic_build_execution_package_empty_required_string_field(
        self,
    ) -> None:
        for field in (
            "package_id",
            "crate_name",
            "expected_loadable_artifact",
            "copied_loadable_artifact",
        ):
            with self.subTest(field=field):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_execution",
                    _native_build_execution(
                        packages=[
                            _native_build_execution_package(**{field: ""})
                        ]
                    ),
                    "native_dynamic report "
                    f"native_build_execution.packages[0].{field} "
                    "must be a non-empty string",
                )

    def test_report_stage_rejects_native_dynamic_build_execution_package_blank_required_string_field(
        self,
    ) -> None:
        for field in (
            "package_id",
            "crate_name",
            "expected_loadable_artifact",
            "copied_loadable_artifact",
        ):
            with self.subTest(field=field):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_execution",
                    _native_build_execution(
                        packages=[
                            _native_build_execution_package(
                                **{field: "   "}
                            )
                        ]
                    ),
                    "native_dynamic report "
                    f"native_build_execution.packages[0].{field} "
                    "must be a non-empty string",
                )

    def test_report_stage_rejects_native_dynamic_build_execution_package_empty_command(
        self,
    ) -> None:
        for command in ([], ["cargo", ""], ["cargo", "   "]):
            with self.subTest(command=command):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_execution",
                    _native_build_execution(
                        packages=[
                            _native_build_execution_package(command=command)
                        ]
                    ),
                    "native_dynamic report "
                    "native_build_execution.packages[0].command "
                    "must be a non-empty string array",
                )

    def test_report_stage_rejects_native_dynamic_build_execution_package_command_non_string_entry_before_array_shape(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_execution",
            _native_build_execution(
                packages=[
                    _native_build_execution_package(
                        command=[
                            "cargo",
                            42,
                            "--manifest-path",
                            "zircon_plugins/Cargo.toml",
                            "-p",
                            "zircon_plugin_animation_native",
                            "--target-dir",
                            "target/native_dynamic",
                        ],
                        expected_loadable_artifact=(
                            "target/native_dynamic/debug/"
                            "zircon_plugin_animation_native.dll"
                        ),
                        copied_loadable_artifact=(
                            "plugins/animation/native/"
                            "zircon_plugin_animation.dll"
                        ),
                        copied_sidecars=[],
                    )
                ]
            ),
            "native_dynamic report native_build_execution.packages[0].command[1] "
            "must be a string",
            "native_dynamic report native_build_execution.packages[0].command "
            "must be a string array",
        )

    def test_report_stage_rejects_native_dynamic_build_execution_command_plan_mismatch(
        self,
    ) -> None:
        def mutate_report(native_report: dict[str, object]) -> None:
            native_report["native_build_execution"] = _native_build_execution(
                packages=[
                    _native_build_execution_package(
                        command=[
                            "cargo",
                            "build",
                            "--manifest-path",
                            "zircon_plugins/Cargo.toml",
                            "-p",
                            "zircon_plugin_animation_native",
                            "--target-dir",
                            "target/native_dynamic_forged",
                            "--features",
                            "v3_fixture_diagnostics",
                            "--release",
                        ]
                    )
                ]
            )

        self._assert_native_dynamic_report_mutation_diagnostic(
            mutate_report,
            "native_dynamic report "
            "native_build_execution package animation command",
        )

    def test_report_stage_rejects_native_dynamic_build_execution_plan_field_shape_before_plan_semantics(
        self,
    ) -> None:
        cases = (
            (
                "command",
                _native_build_execution_package(
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
                ),
                "native_dynamic report native_build_execution."
                "packages[0].command[0] must be a non-empty trimmed string",
                "native_dynamic report native_build_execution package "
                "animation command",
            ),
            (
                "crate_name",
                _native_build_execution_package(
                    crate_name=" zircon_plugin_animation_native "
                ),
                "native_dynamic report native_build_execution."
                "packages[0].crate_name must be a non-empty trimmed string",
                "native_dynamic report native_build_execution package "
                "animation crate_name",
            ),
            (
                "expected_loadable_artifact",
                _native_build_execution_package(
                    expected_loadable_artifact=(
                        " target/native_dynamic/release/"
                        "zircon_plugin_animation_native.dll "
                    )
                ),
                "native_dynamic report native_build_execution."
                "packages[0].expected_loadable_artifact "
                "must be a non-empty trimmed string",
                "native_dynamic report native_build_execution package "
                "animation expected_loadable_artifact",
            ),
        )
        for case_name, package, expected_diagnostic, unexpected_diagnostic in cases:
            with self.subTest(case=case_name):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_execution",
                    _native_build_execution(packages=[package]),
                    expected_diagnostic,
                    unexpected_diagnostic,
                )

    def test_report_stage_rejects_native_dynamic_build_execution_package_blank_copied_sidecar_entry(
        self,
    ) -> None:
        for copied_sidecars in (
            [""],
            ["   "],
            ["plugins/animation/native/plugin.pdb", ""],
        ):
            with self.subTest(copied_sidecars=copied_sidecars):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_execution",
                    _native_build_execution(
                        packages=[
                            _native_build_execution_package(
                                copied_sidecars=copied_sidecars
                            )
                        ]
                    ),
                    "native_dynamic report "
                    "native_build_execution.packages[0].copied_sidecars "
                    "must not contain blank entries",
                )

    def test_report_stage_rejects_native_dynamic_build_execution_package_padded_copied_sidecar_entry(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_execution",
            _native_build_execution(
                packages=[
                    _native_build_execution_package(
                        copied_sidecars=[
                            " plugins/animation/native/plugin.pdb ",
                        ]
                    )
                ]
            ),
            "native_dynamic report "
            "native_build_execution.packages[0].copied_sidecars[0] "
            "must be a non-empty trimmed string",
        )

    def test_report_stage_rejects_native_dynamic_build_execution_package_non_string_copied_sidecar_entry_before_array_shape(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_execution",
            _native_build_execution(
                packages=[
                    _native_build_execution_package(
                        command=[
                            "cargo",
                            "build",
                            "--manifest-path",
                            "zircon_plugins/Cargo.toml",
                            "-p",
                            "zircon_plugin_animation_native",
                            "--target-dir",
                            "target/native_dynamic",
                        ],
                        expected_loadable_artifact=(
                            "target/native_dynamic/debug/"
                            "zircon_plugin_animation_native.dll"
                        ),
                        copied_loadable_artifact=(
                            "plugins/animation/native/"
                            "zircon_plugin_animation.dll"
                        ),
                        copied_sidecars=[42],
                    )
                ]
            ),
            "native_dynamic report "
            "native_build_execution.packages[0].copied_sidecars[0] "
            "must be a string",
            "native_dynamic report "
            "native_build_execution.packages[0].copied_sidecars "
            "must be a string array",
        )

    def test_report_stage_rejects_native_dynamic_build_execution_copied_artifact_shape_before_path_semantics(
        self,
    ) -> None:
        cases = (
            (
                "copied_loadable_artifact",
                _native_build_execution_package(
                    copied_loadable_artifact=(
                        " plugins/physics/native/plugin.dll "
                    )
                ),
                (
                    "native_dynamic report "
                    "native_build_execution.packages[0]."
                    "copied_loadable_artifact must be a non-empty trimmed string"
                ),
                "must be inside plugins/animation/",
            ),
            (
                "copied_sidecars",
                _native_build_execution_package(
                    copied_sidecars=[
                        " plugins/physics/native/plugin.pdb ",
                    ]
                ),
                (
                    "native_dynamic report "
                    "native_build_execution.packages[0].copied_sidecars[0] "
                    "must be a non-empty trimmed string"
                ),
                "must be inside plugins/animation/",
            ),
        )
        for label, package, expected_diagnostic, unexpected_diagnostic in cases:
            with self.subTest(label=label):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_execution",
                    _native_build_execution(packages=[package]),
                    expected_diagnostic,
                    unexpected_diagnostic,
                )

    def test_report_stage_rejects_native_dynamic_build_execution_package_unsafe_copied_loadable_path(
        self,
    ) -> None:
        for copied_loadable_artifact in (
            "../animation/native/plugin.dll",
            "C:/zircon/plugins/animation/native/plugin.dll",
        ):
            with self.subTest(copied_loadable_artifact=copied_loadable_artifact):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_execution",
                    _native_build_execution(
                        packages=[
                            _native_build_execution_package(
                                copied_loadable_artifact=copied_loadable_artifact
                            )
                        ]
                    ),
                    "native_dynamic report "
                    "native_build_execution.packages[0].copied_loadable_artifact "
                    "must be a safe relative path",
                )

    def test_report_stage_rejects_native_dynamic_build_execution_package_unsafe_copied_sidecar_path(
        self,
    ) -> None:
        for copied_sidecars in (
            ["../animation/native/plugin.pdb"],
            ["plugins/animation/../plugin.pdb"],
            ["C:/zircon/plugins/animation/native/plugin.pdb"],
        ):
            with self.subTest(copied_sidecars=copied_sidecars):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_execution",
                    _native_build_execution(
                        packages=[
                            _native_build_execution_package(
                                copied_sidecars=copied_sidecars
                            )
                        ]
                    ),
                    "native_dynamic report "
                    "native_build_execution.packages[0].copied_sidecars[0] "
                    "must be a safe relative path",
                )

    def test_report_stage_rejects_native_dynamic_build_execution_package_copied_loadable_outside_package_dir(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_execution",
            _native_build_execution(
                packages=[
                    _native_build_execution_package(
                        copied_loadable_artifact=(
                            "plugins/physics/native/plugin.dll"
                        )
                    )
                ]
            ),
            "native_dynamic report "
            "native_build_execution.packages[0].copied_loadable_artifact "
            "must be inside plugins/animation/",
        )

    def test_report_stage_rejects_native_dynamic_build_execution_package_copied_sidecar_outside_package_dir(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_execution",
            _native_build_execution(
                packages=[
                    _native_build_execution_package(
                        copied_sidecars=[
                            "plugins/physics/native/plugin.pdb",
                        ]
                    )
                ]
            ),
            "native_dynamic report "
            "native_build_execution.packages[0].copied_sidecars[0] "
            "must be inside plugins/animation/",
        )

    def test_report_stage_rejects_native_dynamic_build_execution_package_duplicate_copied_sidecar_entry(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_execution",
            _native_build_execution(
                packages=[
                    _native_build_execution_package(
                        copied_sidecars=[
                            "plugins/animation/native/plugin.pdb",
                            "plugins/animation/native/plugin.pdb",
                        ]
                    )
                ]
            ),
            "native_dynamic report "
            "native_build_execution.packages[0].copied_sidecars "
            "must not contain duplicate entries",
        )

    def test_report_stage_rejects_native_dynamic_build_execution_package_field_types(
        self,
    ) -> None:
        cases = (
            ("package_id", 42, "must be a string"),
            ("crate_name", 42, "must be a string"),
            ("command", 42, "must be a string array"),
            ("command[0]", [42], "must be a string"),
            ("exit_code", "0", "must be an integer"),
            ("stdout", 42, "must be a string"),
            ("stderr", 42, "must be a string"),
            ("expected_loadable_artifact", 42, "must be a string"),
            ("copied_loadable_artifact", 42, "must be a string"),
            ("copied_sidecars", 42, "must be a string array"),
            ("copied_sidecars[0]", [42], "must be a string"),
        )
        for expected_field, value, expected_type in cases:
            field = expected_field.split("[", maxsplit=1)[0]
            with self.subTest(field=expected_field, value=value):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_execution",
                    _native_build_execution(
                        packages=[
                            _native_build_execution_package(**{field: value})
                        ]
                    ),
                    "native_dynamic report native_build_execution."
                    f"packages[0].{expected_field} {expected_type}",
                )

    def test_report_stage_rejects_native_dynamic_build_execution_package_negative_exit_code(
        self,
    ) -> None:
        def mutate_report(native_report: dict[str, object]) -> None:
            native_report["fatal"] = True
            native_report["diagnostics"] = ["cargo build failed"]
            native_report["native_build_execution"] = _native_build_execution(
                fatal=True,
                diagnostics=["cargo build failed"],
                packages=[
                    _native_build_execution_package(
                        exit_code=-1,
                    )
                ],
            )

        self._assert_native_dynamic_report_mutation_diagnostic(
            mutate_report,
            "native_dynamic report "
            "native_build_execution.packages[0].exit_code must be non-negative",
        )

    def test_report_stage_rejects_native_dynamic_build_execution_package_nonzero_exit_code(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_execution",
            _native_build_execution(
                packages=[
                    _native_build_execution_package(exit_code=1),
                ],
            ),
            "native_dynamic report native_build_execution.packages[0].exit_code "
            "must be 0 for non-fatal build execution",
        )


if __name__ == "__main__":
    unittest.main()
