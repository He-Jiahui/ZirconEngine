from __future__ import annotations

import unittest

from tools.zircon_export.tests.native_dynamic_stage_schema_test_support import (
    NativeDynamicStageSchemaReportAssertions,
    _native_build_execution,
    _native_build_execution_package_for_default_report,
    _native_build_plan,
    _native_build_plan_package,
    _native_build_plan_package_with_features,
    _native_build_plan_without,
)


class PipelineReportNativeDynamicStageBuildPlanSchemaTests(
    NativeDynamicStageSchemaReportAssertions,
    unittest.TestCase,
):
    def test_report_stage_rejects_native_dynamic_build_plan_unknown_field(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_plan",
            _native_build_plan(unsigned_sidecar="target/sidecar.bin"),
            "native_dynamic report native_build_plan unknown field unsigned_sidecar",
        )

    def test_report_stage_rejects_native_dynamic_build_plan_missing_release_evidence_field(
        self,
    ) -> None:
        cases = (
            (
                "workspace_manifest",
                "native_dynamic report native_build_plan.workspace_manifest must be a string",
            ),
            (
                "target_dir",
                "native_dynamic report native_build_plan.target_dir must be a string",
            ),
            (
                "cargo_profile",
                "native_dynamic report native_build_plan.cargo_profile must be a string",
            ),
            (
                "release",
                "native_dynamic report native_build_plan.release must be a boolean",
            ),
            (
                "build_features",
                "native_dynamic report native_build_plan.build_features must be a string array",
            ),
            (
                "package_count",
                "native_dynamic report native_build_plan.package_count must be an integer",
            ),
            (
                "diagnostics",
                "native_dynamic report native_build_plan.diagnostics must be a string array",
            ),
            (
                "packages",
                "native_dynamic report native_build_plan.packages must be an object array",
            ),
            (
                "fatal",
                "native_dynamic report native_build_plan.fatal must be a boolean",
            ),
        )
        for field, expected_diagnostic in cases:
            with self.subTest(field=field):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_plan",
                    _native_build_plan_without(field),
                    expected_diagnostic,
                )

    def test_report_stage_rejects_native_dynamic_build_plan_empty_required_string_release_evidence_field(
        self,
    ) -> None:
        for field in ("workspace_manifest", "target_dir", "cargo_profile"):
            with self.subTest(field=field):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_plan",
                    _native_build_plan(**{field: ""}),
                    "native_dynamic report "
                    f"native_build_plan.{field} must be a non-empty string",
                )

    def test_report_stage_rejects_native_dynamic_build_plan_blank_required_string_release_evidence_field(
        self,
    ) -> None:
        for field in ("workspace_manifest", "target_dir", "cargo_profile"):
            with self.subTest(field=field):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_plan",
                    _native_build_plan(**{field: "   "}),
                    "native_dynamic report "
                    f"native_build_plan.{field} must be a non-empty string",
                )

    def test_report_stage_rejects_native_dynamic_build_plan_padded_required_string_release_evidence_field_before_semantics(
        self,
    ) -> None:
        cases = (
            (
                "workspace_manifest",
                " zircon_plugins/Cargo.toml ",
                "native_dynamic report native_build_plan.packages[0]."
                "workspace_manifest zircon_plugins/Cargo.toml does not match "
                "native_dynamic report native_build_plan.workspace_manifest "
                " zircon_plugins/Cargo.toml ",
            ),
            (
                "target_dir",
                " target/native_dynamic ",
                "native_dynamic report native_build_plan.packages[0]."
                "target_dir target/native_dynamic does not match "
                "native_dynamic report native_build_plan.target_dir "
                " target/native_dynamic ",
            ),
            (
                "cargo_profile",
                " release ",
                "native_dynamic report native_build_plan.cargo_profile "
                "must be debug or release",
            ),
        )
        for field, value, unexpected_diagnostic in cases:
            with self.subTest(field=field):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_plan",
                    _native_build_plan(**{field: value}),
                    "native_dynamic report "
                    f"native_build_plan.{field} "
                    "must be a non-empty trimmed string",
                    unexpected_diagnostic,
                )

    def test_report_stage_rejects_native_dynamic_build_plan_field_types(
        self,
    ) -> None:
        cases = (
            ("fatal", "false", "must be a boolean"),
            ("diagnostics", 42, "must be a string array"),
            ("diagnostics[0]", [42], "must be a string"),
            ("workspace_manifest", 42, "must be a string"),
            ("target_dir", 42, "must be a string"),
            ("cargo_profile", 42, "must be a string"),
            ("release", "true", "must be a boolean"),
            ("build_features", 42, "must be a string array"),
            ("build_features[0]", [42], "must be a string"),
            ("package_count", "1", "must be an integer"),
            ("packages", "not-an-array", "must be an object array"),
            ("packages[0]", [42], "must be an object"),
        )
        for expected_field, value, expected_type in cases:
            field = expected_field.split("[", maxsplit=1)[0]
            with self.subTest(field=expected_field, value=value):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_plan",
                    _native_build_plan(**{field: value}),
                    "native_dynamic report "
                    f"native_build_plan.{expected_field} {expected_type}",
                )

    def test_report_stage_rejects_native_dynamic_build_plan_negative_count_fields(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_plan",
            _native_build_plan(package_count=-1),
            "native_dynamic report native_build_plan.package_count must be non-negative",
        )

    def test_report_stage_rejects_native_dynamic_build_plan_blank_build_feature_entry(
        self,
    ) -> None:
        for build_features in (
            [""],
            ["   "],
            ["v3_fixture_diagnostics", ""],
        ):
            with self.subTest(build_features=build_features):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_plan",
                    _native_build_plan(build_features=build_features),
                    "native_dynamic report native_build_plan.build_features "
                    "must not contain blank entries",
                )

    def test_report_stage_rejects_native_dynamic_build_plan_padded_build_feature_entry(
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
            "native_dynamic report native_build_plan.build_features[0] "
            "must be a non-empty trimmed string",
        )

    def test_report_stage_rejects_native_dynamic_build_plan_duplicate_build_feature_entry(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_plan",
            _native_build_plan(
                build_features=[
                    "v3_fixture_diagnostics",
                    "v3_fixture_diagnostics",
                ]
            ),
            "native_dynamic report native_build_plan.build_features "
            "must not contain duplicate entries",
        )

    def test_report_stage_rejects_native_dynamic_build_plan_non_string_build_feature_entry_before_array_shape(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_plan",
            _native_build_plan(
                build_features=["v3_fixture_diagnostics", 42],
                packages=[
                    _native_build_plan_package(
                        features=["v3_fixture_diagnostics", 42]
                    )
                ],
            ),
            "native_dynamic report native_build_plan.build_features[1] "
            "must be a string",
            "native_dynamic report native_build_plan.build_features "
            "must be a string array",
        )

    def test_report_stage_rejects_native_dynamic_build_plan_padded_duplicate_build_feature_before_uniqueness(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_plan",
            _native_build_plan(
                build_features=[
                    " v3_fixture_diagnostics ",
                    " v3_fixture_diagnostics ",
                ],
                package_count=0,
                packages=[],
            ),
            "native_dynamic report native_build_plan.build_features[0] "
            "must be a non-empty trimmed string",
            (
                "native_dynamic report native_build_plan.build_features "
                "must not contain duplicate entries"
            ),
        )

    def test_report_stage_rejects_native_dynamic_build_plan_profile_release_contract(
        self,
    ) -> None:
        cases = (
            (
                "invalid_profile",
                _native_build_plan(
                    cargo_profile="shipping",
                    packages=[
                        _native_build_plan_package(cargo_profile="shipping")
                    ],
                ),
                "native_dynamic report native_build_plan.cargo_profile "
                "must be debug or release",
            ),
            (
                "release_true_debug_profile",
                _native_build_plan(
                    cargo_profile="debug",
                    release=True,
                    packages=[
                        _native_build_plan_package(cargo_profile="debug")
                    ],
                ),
                "native_dynamic report native_build_plan.release "
                "must match cargo_profile",
            ),
            (
                "release_false_release_profile",
                _native_build_plan(
                    cargo_profile="release",
                    release=False,
                    packages=[
                        _native_build_plan_package(
                            release=False,
                            command=[
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
                        )
                    ],
                ),
                "native_dynamic report native_build_plan.release "
                "must match cargo_profile",
            ),
        )
        for case_name, build_plan, expected_diagnostic in cases:
            with self.subTest(case=case_name):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_plan",
                    build_plan,
                    expected_diagnostic,
                )

    def test_report_stage_rejects_native_dynamic_build_plan_fatal_without_diagnostics(
        self,
    ) -> None:
        def mutate_report(native_report: dict[str, object]) -> None:
            native_report["fatal"] = True
            native_report["diagnostics"] = ["native build plan failed"]
            native_build_plan = native_report["native_build_plan"]
            self.assertIsInstance(native_build_plan, dict)
            native_build_plan["fatal"] = True
            native_build_plan["diagnostics"] = []

        self._assert_native_dynamic_report_mutation_diagnostic(
            mutate_report,
            "native_dynamic report native_build_plan fatal report "
            "must include diagnostics",
        )

    def test_report_stage_rejects_native_dynamic_build_plan_non_fatal_with_diagnostics(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_plan",
            _native_build_plan(diagnostics=["native build plan warning"]),
            "native_dynamic report native_build_plan.diagnostics "
            "must be empty when fatal is False",
        )

    def test_report_stage_rejects_native_dynamic_build_plan_non_string_diagnostic_entry_before_array_shape(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_plan",
            _native_build_plan(diagnostics=[42]),
            "native_dynamic report native_build_plan.diagnostics[0] "
            "must be a string",
            "native_dynamic report native_build_plan.diagnostics "
            "must be a string array",
        )

    def test_report_stage_rejects_native_dynamic_build_audit_package_count_mismatch(
        self,
    ) -> None:
        cases = (
            (
                "native_build_plan",
                "native_build_plan",
                "native_dynamic report native_build_plan.package_count 2 "
                "does not match native_dynamic report "
                "native_build_plan.packages 1",
            ),
            (
                "native_build_execution",
                "native_build_execution",
                "native_dynamic report native_build_execution.package_count 2 "
                "does not match native_dynamic report "
                "native_build_execution.packages 1",
            ),
        )
        for case_name, field, expected_diagnostic in cases:
            with self.subTest(case=case_name):

                def mutate_report(native_report: dict[str, object]) -> None:
                    native_report["fatal"] = True
                    native_report["diagnostics"] = ["native signing failed"]
                    build_audit = native_report[field]
                    self.assertIsInstance(build_audit, dict)
                    if field == "native_build_execution":
                        build_audit["enabled"] = True
                        build_audit["packages"] = [
                            _native_build_execution_package_for_default_report()
                        ]
                    build_audit["package_count"] = 2

                self._assert_native_dynamic_report_mutation_diagnostic(
                    mutate_report,
                    expected_diagnostic,
                )

    def test_report_stage_rejects_native_dynamic_build_audit_duplicate_package_id(
        self,
    ) -> None:
        cases = (
            (
                "native_build_plan",
                "native_build_plan",
                _native_build_plan_package,
                "native_dynamic report native_build_plan.packages "
                "package_id animation must be unique",
            ),
            (
                "native_build_execution",
                "native_build_execution",
                _native_build_execution_package_for_default_report,
                "native_dynamic report native_build_execution.packages "
                "package_id animation must be unique",
            ),
        )
        for case_name, field, package_factory, expected_diagnostic in cases:
            with self.subTest(case=case_name):

                def mutate_report(native_report: dict[str, object]) -> None:
                    native_report["fatal"] = True
                    native_report["diagnostics"] = ["native signing failed"]
                    build_audit = native_report[field]
                    self.assertIsInstance(build_audit, dict)
                    if field == "native_build_execution":
                        build_audit["enabled"] = True
                    build_audit["package_count"] = 2
                    build_audit["packages"] = [
                        package_factory(),
                        package_factory(),
                    ]

                self._assert_native_dynamic_report_mutation_diagnostic(
                    mutate_report,
                    expected_diagnostic,
                )

    def test_report_stage_rejects_native_dynamic_build_audit_padded_duplicate_package_id_before_uniqueness(
        self,
    ) -> None:
        cases = (
            (
                "native_build_plan",
                _native_build_plan(
                    package_count=2,
                    packages=[
                        _native_build_plan_package(package_id=" animation "),
                        _native_build_plan_package(package_id=" animation "),
                    ],
                ),
                "native_dynamic report native_build_plan.packages[0]."
                "package_id must be a non-empty trimmed string",
                "native_dynamic report native_build_plan.packages "
                "package_id animation must be unique",
            ),
            (
                "native_build_execution",
                _native_build_execution(
                    package_count=2,
                    packages=[
                        _native_build_execution_package_for_default_report(
                            package_id=" animation "
                        ),
                        _native_build_execution_package_for_default_report(
                            package_id=" animation "
                        ),
                    ],
                ),
                "native_dynamic report native_build_execution.packages[0]."
                "package_id must be a non-empty trimmed string",
                "native_dynamic report native_build_execution.packages "
                "package_id animation must be unique",
            ),
        )
        for field, value, expected_diagnostic, unexpected_diagnostic in cases:
            with self.subTest(field=field):
                self._assert_native_dynamic_report_field_diagnostic(
                    field,
                    value,
                    expected_diagnostic,
                    unexpected_diagnostic,
                )

    def test_report_stage_rejects_native_dynamic_build_plan_blank_diagnostic_entry(
        self,
    ) -> None:
        for diagnostics in ([""], ["   "], ["build failed", ""]):
            with self.subTest(diagnostics=diagnostics):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_plan",
                    _native_build_plan(diagnostics=diagnostics),
                    "native_dynamic report native_build_plan.diagnostics "
                    "must not contain blank entries",
                )

    def test_report_stage_rejects_native_dynamic_build_plan_padded_diagnostic_entry(
        self,
    ) -> None:
        def mutate_report(native_report: dict[str, object]) -> None:
            native_report["fatal"] = True
            native_report["diagnostics"] = ["native build plan failed"]
            native_build_plan = native_report["native_build_plan"]
            self.assertIsInstance(native_build_plan, dict)
            native_build_plan["fatal"] = True
            native_build_plan["diagnostics"] = [" workspace manifest missing "]

        self._assert_native_dynamic_report_mutation_diagnostic(
            mutate_report,
            "native_dynamic report native_build_plan.diagnostics[0] "
            "must be a non-empty trimmed string",
        )

    def test_report_stage_rejects_native_dynamic_build_plan_fatal_enabled_execution_mismatch(
        self,
    ) -> None:
        def mutate_report(native_report: dict[str, object]) -> None:
            native_build_plan = native_report["native_build_plan"]
            self.assertIsInstance(native_build_plan, dict)
            native_build_plan["fatal"] = True
            native_build_plan["diagnostics"] = ["workspace manifest missing"]
            native_report["native_build_execution"] = _native_build_execution(
                enabled=True,
                package_count=1,
                packages=[_native_build_execution_package_for_default_report()],
            )

        self._assert_native_dynamic_report_mutation_diagnostic(
            mutate_report,
            "native_dynamic report native_build_plan.fatal must be False "
            "when native_build_execution.enabled is True and "
            "NativeDynamic report fatal is False",
        )


if __name__ == "__main__":
    unittest.main()
