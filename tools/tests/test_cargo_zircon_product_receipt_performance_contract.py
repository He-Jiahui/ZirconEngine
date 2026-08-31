from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
RECEIPT = ROOT / "tools/cargo-zircon/src/build/receipt"
PRODUCT_BUILD = ROOT / "tools/cargo-zircon/src/build/product_build.rs"
PRODUCT_BUILD_CAPTURE = ROOT / "tools/cargo-zircon/src/build/product_build/capture.rs"
PRODUCT_BUILD_PROTOCOL = ROOT / "tools/cargo-zircon/src/build/product_build/cargo_protocol.rs"
PRODUCT_BUILD_SET = ROOT / "tools/cargo-zircon/src/build/product_build/build_set.rs"
PRODUCT_BUILD_ENVIRONMENT = ROOT / "tools/cargo-zircon/src/build/product_build/environment.rs"
PRODUCT_BUILD_BATCH = ROOT / "tools/cargo-zircon/src/build/product_build/batch.rs"
PRODUCT_BUILD_PERFORMANCE = ROOT / "tools/cargo-zircon/src/build/product_build/performance_tests.rs"
PRODUCT_BUILD_SET_TESTS = (
    PRODUCT_BUILD_SET.parent / "build_set/tests.rs",
    PRODUCT_BUILD_SET.parent / "build_set/behavior_tests.rs",
)
PRODUCT_RECEIPT_CLI = ROOT / "tools/cargo-zircon/src/product_receipt_cli"
PRODUCT_RECEIPT_CLI_RUN = PRODUCT_RECEIPT_CLI / "run.rs"
PRODUCT_RECEIPT_CLI_OPTIONS = PRODUCT_RECEIPT_CLI / "options.rs"
PRODUCT_RECEIPT_CLI_OPTIONS_PERFORMANCE = (
    PRODUCT_RECEIPT_CLI / "options/performance_tests.rs"
)
PRODUCT_RECEIPT_CLI_INPUT = PRODUCT_RECEIPT_CLI / "input.rs"
PRODUCT_RECEIPT_CLI_INPUT_PERFORMANCE = PRODUCT_RECEIPT_CLI / "input/performance_tests.rs"


def source_between(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


def build_set_test_sources() -> str:
    return "\n".join(path.read_text(encoding="utf-8") for path in PRODUCT_BUILD_SET_TESTS)


class CargoZirconProductReceiptPerformanceContractTests(unittest.TestCase):
    def test_build_set_test_modules_are_partitioned_below_the_large_file_limit(
        self,
    ) -> None:
        build_set = PRODUCT_BUILD_SET.read_text(encoding="utf-8")
        test_modules = PRODUCT_BUILD_SET_TESTS + (
            PRODUCT_BUILD_SET.parent / "build_set/performance_tests.rs",
        )

        self.assertIn("mod behavior_tests;", build_set)
        self.assertIn("mod performance_tests;", build_set)
        self.assertIn("mod tests;", build_set)
        for path in test_modules:
            self.assertLess(len(path.read_text(encoding="utf-8").splitlines()), 1_000)

    def test_cargo_linker_environment_key_uses_one_output_allocation(self) -> None:
        environment = PRODUCT_BUILD_ENVIRONMENT.read_text(encoding="utf-8")
        key = source_between(
            environment,
            "fn cargo_linker_environment_key(",
            "fn insert_forced_environment(",
        )
        behavior = (
            PRODUCT_BUILD_ENVIRONMENT.parent / "environment/tests.rs"
        ).read_text(encoding="utf-8")
        benchmark = (
            PRODUCT_BUILD_ENVIRONMENT.parent / "environment/performance_tests.rs"
        ).read_text(encoding="utf-8")

        self.assertEqual(key.count("String::with_capacity"), 1)
        self.assertNotIn("format!", key)
        self.assertNotIn("collect::<String>", key)
        self.assertIn('linker_key.push_str("CARGO_TARGET_")', key)
        self.assertIn('linker_key.push_str("_LINKER")', key)
        self.assertIn("cargo_linker_environment_key_maps_and_rejects_target_triples", behavior)
        self.assertIn("TOOLING15_SINGLE_ALLOCATION_LINKER_KEY_BENCH_V1", benchmark)
        self.assertIn("candidate P50 did not improve by at least 15%", benchmark)
        self.assertIn("candidate P95 did not improve by at least 15%", benchmark)

    def test_hex_encoding_uses_one_preallocated_output_buffer(self) -> None:
        source = (RECEIPT / "canonical.rs").read_text(encoding="utf-8")
        encode = source_between(source, "pub(crate) fn bytes_to_hex", "pub(crate) fn decode_hex")
        decode = source_between(source, "pub(crate) fn decode_hex", "#[cfg(test)]")

        self.assertIn('UPPER_HEX_DIGITS: &[u8; 16] = b"0123456789ABCDEF"', source)
        self.assertIn("String::with_capacity(bytes.len().saturating_mul(2))", encode)
        self.assertNotIn("format!", encode)
        self.assertIn("Vec::with_capacity(value.len() / 2)", decode)
        self.assertNotIn("from_str_radix", decode)
        self.assertIn("encodes_and_decodes_every_hex_nibble_boundary", source)
        self.assertIn("decodes_mixed_case_hex_and_rejects_invalid_input", source)

    def test_file_digests_reuse_hex_and_toolchain_streams_identity_hash(self) -> None:
        file_digest = (RECEIPT / "file_digest.rs").read_text(encoding="utf-8")
        toolchain_set = (RECEIPT / "toolchain_set.rs").read_text(encoding="utf-8")
        derived_id = source_between(toolchain_set, "fn derived_id", "fn normalize_digest")

        self.assertIn("canonical::bytes_to_hex", file_digest)
        self.assertIn("sha256: bytes_to_hex(&digest.sha256)", file_digest)
        self.assertIn("sha256_serialized", toolchain_set)
        self.assertIn("sha256_serialized(&payload", derived_id)
        self.assertNotIn("serde_json::to_vec", derived_id)
        self.assertNotIn("Sha256::digest", derived_id)
        self.assertNotIn('format!("{byte:02X}")', file_digest)
        self.assertNotIn('format!("{byte:02X}")', toolchain_set)

    def test_toolchain_capture_reuses_the_product_digest_buffer(self) -> None:
        file_digest = (RECEIPT / "file_digest.rs").read_text(encoding="utf-8")
        toolchain = (RECEIPT / "toolchain_set.rs").read_text(encoding="utf-8")
        capture = source_between(
            toolchain,
            "pub fn capture_from_files(",
            "pub(crate) fn normalize_and_verify_identity",
        )
        components = source_between(
            toolchain,
            "impl ToolchainComponentDigests",
            "fn normalize_digest",
        )
        behavior = (
            RECEIPT / "toolchain_set/tests.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("fn digest_open_file_with_buffer", file_digest)
        self.assertEqual(capture.count("FileDigestBuffer::new()"), 1)
        self.assertGreaterEqual(components.count("digest_open_file_handle_with_buffer"), 3)
        self.assertNotIn(".map(digest_open_file)", capture)
        self.assertIn(
            "capture_from_files_reuses_buffer_across_tool_binaries", behavior
        )

    def test_receipt_closure_capture_reuses_one_product_digest_buffer(self) -> None:
        closure = (RECEIPT / "product_receipt_closure.rs").read_text(
            encoding="utf-8"
        )
        artifact = (RECEIPT / "receipt_artifact.rs").read_text(encoding="utf-8")
        capture = source_between(
            closure,
            "impl ProductReceiptClosure",
            "#[cfg(windows)]\nfn require_immutable_capture_platform",
        )
        behavior = (
            RECEIPT / "product_receipt_closure/tests.rs"
        ).read_text(encoding="utf-8")

        self.assertEqual(capture.count("FileDigestBuffer::new()"), 1)
        self.assertIn("ToolchainSet::capture_from_files_with_buffer", capture)
        self.assertGreaterEqual(capture.count("&mut digest_buffer"), 5)
        self.assertIn("capture_from_file_with_buffer", artifact)
        self.assertIn("capture_from_file_with_buffer", closure)
        self.assertIn(
            "closure_artifact_capture_reuses_one_digest_buffer", behavior
        )

    def test_product_build_capture_reuses_one_prepared_toolchain_and_digest_buffer(self) -> None:
        build = PRODUCT_BUILD.read_text(encoding="utf-8")
        batch = PRODUCT_BUILD_BATCH.read_text(encoding="utf-8")
        capture_helpers = PRODUCT_BUILD_CAPTURE.read_text(encoding="utf-8")
        capture = source_between(
            build,
            "pub(super) fn build_product_receipt_draft_in_build_set(",
            "pub fn select_cargo_product_artifact(",
        )
        helpers = capture_helpers
        behavior = (
            PRODUCT_BUILD.parent / "product_build/capture/tests.rs"
        ).read_text(encoding="utf-8")
        benchmark = (
            PRODUCT_BUILD.parent / "product_build/capture/performance_tests.rs"
        ).read_text(encoding="utf-8")
        batch_build = source_between(
            batch,
            "pub fn build_product_receipt_draft_batch(",
            "fn validate_build_batch_request(",
        )

        self.assertIn("struct PreparedProductBuildToolchain", capture_helpers)
        self.assertEqual(capture_helpers.count("FileDigestBuffer::new()"), 1)
        self.assertEqual(build.count("PreparedProductBuildToolchain::open"), 1)
        self.assertEqual(batch_build.count("PreparedProductBuildToolchain::open"), 1)
        self.assertLess(
            batch_build.index("PreparedProductBuildToolchain::open"),
            batch_build.index("for build in request.builds"),
        )
        self.assertIn("prepared_toolchain.receipt_toolchain(environment_digest)", capture)
        self.assertIn("prepared_toolchain.digest_buffer()", capture)
        self.assertIn("ReceiptArtifact::capture_from_file_with_buffer", capture)
        self.assertGreaterEqual(capture.count("digest_buffer"), 6)
        self.assertIn("digest_open_file_handle_with_buffer", helpers)
        self.assertIn("capture_from_file_with_buffer", helpers)
        self.assertIn(
            "product_build_capture_helpers_reuse_one_digest_buffer", behavior
        )
        self.assertIn(
            "prepared_toolchain_reuses_components_and_retains_handles", behavior
        )
        self.assertIn("TOOLING15_SHARED_BATCH_TOOLCHAIN_CAPTURE_BENCH_V1", benchmark)
        self.assertIn("candidate P50 did not improve by at least 50%", benchmark)
        self.assertIn("candidate P95 did not improve by at least 50%", benchmark)

    def test_normalized_sha256_validation_uses_one_byte_pass(self) -> None:
        validation = (RECEIPT / "validation.rs").read_text(encoding="utf-8")
        normalized_sha = source_between(
            validation,
            "fn validate_sha256_if_normalized",
            "#[cfg(test)]\nfn artifacts_are_normalized",
        )
        benchmark = (
            RECEIPT / "validation/performance_tests.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("let mut normalized = true;", normalized_sha)
        self.assertIn("for byte in value.bytes()", normalized_sha)
        self.assertIn("normalized &= !byte.is_ascii_lowercase()", normalized_sha)
        self.assertNotIn("is_normalized_sha256(value)", normalized_sha)
        self.assertIn(
            "TOOLING15_SINGLE_PASS_SHA256_VALIDATION_BENCH_V1", benchmark
        )

    def test_created_utc_validation_uses_fixed_layout_byte_parsing(self) -> None:
        validation = (RECEIPT / "validation.rs").read_text(encoding="utf-8")
        created_utc = source_between(
            validation,
            "fn validate_created_utc(",
            "fn validate_artifacts(",
        )
        behavior = (
            RECEIPT / "validation/tests.rs"
        ).read_text(encoding="utf-8")
        benchmark = (
            RECEIPT / "validation/performance_tests.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("let bytes = value.as_bytes();", created_utc)
        self.assertIn("parse_decimal_component", created_utc)
        self.assertNotIn("value.is_ascii()", created_utc)
        self.assertNotIn("strip_suffix", created_utc)
        self.assertNotIn("split_once", created_utc)
        self.assertNotIn("fn decimal_value", created_utc)
        self.assertIn("accepts_receipt_timestamp_boundary_values", behavior)
        self.assertIn("rejects_malformed_receipt_timestamps_without_panicking", behavior)
        self.assertIn(
            "TOOLING15_FIXED_LAYOUT_TIMESTAMP_BENCH_V1", benchmark
        )

    def test_relative_path_validation_uses_one_platform_independent_pass(self) -> None:
        validation = (RECEIPT / "validation.rs").read_text(encoding="utf-8")
        relative_path = source_between(
            validation,
            "fn validate_relative_path",
            "fn validate_sha256(",
        )
        behavior = (RECEIPT / "validation/tests.rs").read_text(encoding="utf-8")
        benchmark = (
            RECEIPT / "validation/performance_tests.rs"
        ).read_text(encoding="utf-8")

        self.assertIn(
            "for (index, byte) in bytes.iter().copied().enumerate()", relative_path
        )
        self.assertIn("is_windows_drive_prefix(bytes)", relative_path)
        self.assertIn("is_dot_component", relative_path)
        self.assertNotIn("Path::new", relative_path)
        self.assertNotIn(".components()", relative_path)
        self.assertNotIn('.contains("//")', relative_path)
        self.assertIn(
            "validates_artifact_relative_paths_without_host_path_semantics", behavior
        )
        self.assertIn(
            "TOOLING15_SINGLE_PASS_RELATIVE_PATH_BENCH_V1", benchmark
        )
        self.assertIn("candidate P50 did not improve by at least 20%", benchmark)
        self.assertIn("candidate P95 did not improve by at least 20%", benchmark)

    def test_artifact_name_deduplication_borrows_names(self) -> None:
        validation = (RECEIPT / "validation.rs").read_text(encoding="utf-8")
        unique_names = source_between(
            validation,
            "fn validate_unique_artifact_names",
            "fn validate_relative_path",
        )

        self.assertIn("HashSet::with_capacity", unique_names)
        self.assertIn("artifact.logical_name.as_str()", unique_names)
        self.assertNotIn("artifact.logical_name.clone()", validation)

    def test_sorted_artifact_names_use_fixed_partition_merge(self) -> None:
        validation = (RECEIPT / "validation.rs").read_text(encoding="utf-8")
        artifact_validation = source_between(
            validation,
            "fn validate_artifacts(",
            "fn validate_artifacts_if_normalized",
        )
        unique_names = source_between(
            validation,
            "fn validate_unique_artifact_names",
            "fn validate_relative_path",
        )

        self.assertIn("validate_unique_sorted_artifact_names", unique_names)
        self.assertIn("let partitions = [", unique_names)
        self.assertIn("let mut offsets = [0_usize; 4]", unique_names)
        self.assertEqual(unique_names.count("HashSet::with_capacity"), 1)
        self.assertIn(
            "paths.insert(artifact.relative_path.as_str())",
            unique_names,
        )
        self.assertNotIn(
            ".iter()\n        .chain(runtime_dependencies)",
            unique_names,
        )
        self.assertIn("artifacts.sort_unstable_by", artifact_validation)
        self.assertNotIn("artifacts.sort_by", artifact_validation)
        benchmark = (
            RECEIPT / "validation/performance_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "TOOLING15_SORTED_ARTIFACT_NAME_MERGE_BENCH_V1", benchmark
        )
        self.assertIn(
            "TOOLING15_UNSTABLE_IDENTITY_RECORD_SORT_BENCH_V1", benchmark
        )
        self.assertIn(
            "TOOLING15_FUSED_ARTIFACT_UNIQUENESS_BENCH_V1", benchmark
        )

    def test_normalized_artifact_closure_fuses_field_and_uniqueness_walks(self) -> None:
        validation = (RECEIPT / "validation.rs").read_text(encoding="utf-8")
        benchmark = (
            RECEIPT / "validation/performance_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("validate_normalized_artifact_closure", validation)
        closure = source_between(
            validation,
            "fn validate_normalized_artifact_closure(",
            "fn validate_artifact_kind",
        )
        self.assertIn("let mut paths = HashSet::with_capacity(artifact_count);", closure)
        self.assertIn("let mut offsets = [0_usize; 4];", closure)
        self.assertIn("offsets[partition_index] - 1", closure)
        self.assertIn("validate_artifact_if_normalized(artifact)", closure)
        self.assertIn("paths.insert(artifact.relative_path.as_str())", closure)
        self.assertNotIn(".windows(2)", closure)
        self.assertNotIn("validate_artifacts_if_normalized(&", closure)
        self.assertIn("TOOLING15_NORMALIZED_ARTIFACT_CLOSURE_BENCH_V1", benchmark)

    def test_sorted_build_features_use_adjacent_duplicate_detection(self) -> None:
        validation = (RECEIPT / "validation.rs").read_text(encoding="utf-8")
        action = source_between(
            validation,
            "fn validate_action(action:",
            "fn validate_action_if_normalized",
        )

        self.assertIn("action.features.sort_unstable()", action)
        self.assertIn("let mut previous", action)
        self.assertIn("previous == Some(feature.as_str())", action)
        self.assertNotIn("HashSet", action)
        benchmark = (
            RECEIPT / "validation/performance_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "TOOLING15_SORTED_FEATURE_DEDUP_BENCH_V1", benchmark
        )

    def test_sorted_product_build_request_fields_use_adjacent_deduplication(self) -> None:
        product_build = PRODUCT_BUILD.read_text(encoding="utf-8")
        validation = source_between(
            product_build,
            "pub(super) fn validate_build_request",
            "fn validate_required",
        )

        self.assertGreaterEqual(validation.count(".sort_unstable_by"), 2)
        self.assertNotIn(".sort_by", validation)
        self.assertIn("action.features.sort_unstable()", validation)
        self.assertGreaterEqual(validation.count("let mut previous"), 3)
        self.assertNotIn("HashSet", validation)
        behavior = (
            PRODUCT_BUILD_BATCH.parent / "tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "request_validation_rejects_duplicate_sdk_names_after_sorting", behavior
        )
        self.assertIn(
            "request_validation_rejects_duplicate_runtime_dependency_names_after_sorting",
            behavior,
        )
        self.assertIn(
            "request_validation_rejects_duplicate_features_after_sorting", behavior
        )
        benchmark = (
            RECEIPT / "validation/performance_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("TOOLING15_SORTED_FEATURE_DEDUP_BENCH_V1", benchmark)
        self.assertIn("TOOLING15_UNSTABLE_FEATURE_SORT_BENCH_V1", benchmark)

    def test_receipt_identity_hashes_the_canonical_stream_without_a_byte_buffer(self) -> None:
        canonical = (RECEIPT / "canonical.rs").read_text(encoding="utf-8")
        receipt = (RECEIPT / "product_receipt.rs").read_text(encoding="utf-8")

        self.assertIn("struct CanonicalDigestWriter", canonical)
        self.assertIn("serde_json::to_writer(&mut writer, payload)", canonical)
        self.assertIn("streamed_digest_matches_the_canonical_json_bytes", canonical)
        self.assertIn("canonical_receipt_sha256", receipt)
        self.assertNotIn("canonical_bytes", receipt)

    def test_attestation_payloads_preallocate_their_canonical_json_buffer(self) -> None:
        canonical = (RECEIPT / "canonical.rs").read_text(encoding="utf-8")
        receipt_attestation = source_between(
            canonical,
            "pub(crate) fn attestation_bytes",
            "pub(crate) fn canonical_receipt_batch_sha256",
        )
        batch_attestation = source_between(
            canonical,
            "pub(crate) fn batch_attestation_bytes",
            "#[cfg(test)]\nfn sha256_hex",
        )
        self.assertIn("fn serialize_attestation_with_capacity", canonical)
        serializer = source_between(
            canonical,
            "fn serialize_attestation_with_capacity",
            "pub(crate) fn canonical_receipt_batch_sha256",
        )
        benchmark = (
            RECEIPT / "canonical/performance_tests.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("serialize_attestation_with_capacity", receipt_attestation)
        self.assertIn("serialize_attestation_with_capacity", batch_attestation)
        self.assertNotIn("serde_json::to_vec", receipt_attestation)
        self.assertNotIn("serde_json::to_vec", batch_attestation)
        self.assertIn("Vec::with_capacity", serializer)
        self.assertIn("serde_json::to_writer(&mut serialized, payload)", serializer)
        self.assertIn(
            "preallocated_attestation_payloads_match_serde_for_escaped_fields",
            canonical,
        )
        self.assertIn(
            "TOOLING15_PREALLOCATED_ATTESTATION_PAYLOAD_BENCH_V1",
            benchmark,
        )

    def test_attestation_signature_verification_uses_inline_decode_with_heap_fallback(
        self,
    ) -> None:
        canonical = (RECEIPT / "canonical.rs").read_text(encoding="utf-8")
        receipt = (RECEIPT / "product_receipt.rs").read_text(encoding="utf-8")
        batch = (RECEIPT / "product_receipt_batch.rs").read_text(encoding="utf-8")
        behavior = (RECEIPT / "canonical.rs").read_text(encoding="utf-8")
        benchmark = (
            RECEIPT / "product_receipt/performance_tests.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("fn decode_hex_into", canonical)
        self.assertIn("const INLINE_SIGNATURE_CAPACITY: usize = 64", canonical)
        self.assertIn(
            "let mut inline_signature = [0_u8; INLINE_SIGNATURE_CAPACITY]",
            receipt,
        )
        self.assertIn(
            "let mut inline_signature = [0_u8; INLINE_SIGNATURE_CAPACITY]",
            batch,
        )
        self.assertIn("decode_hex_into", receipt)
        self.assertIn("decode_hex_into", batch)
        self.assertIn("decode_hex(&self.attestation.signature_hex)", receipt)
        self.assertIn("decode_hex(&self.attestation.signature_hex)", batch)
        self.assertIn("inline_hex_decode_matches_allocating_decode", behavior)
        self.assertIn(
            "oversized_custom_signature_keeps_allocating_decode_fallback",
            (RECEIPT / "product_receipt/tests.rs").read_text(encoding="utf-8"),
        )
        self.assertIn("TOOLING15_INLINE_SIGNATURE_DECODE_BENCH_V1", benchmark)

    def test_canonical_receipt_integrity_avoids_deep_closure_clone(self) -> None:
        canonical = (RECEIPT / "canonical.rs").read_text(encoding="utf-8")
        receipt = (RECEIPT / "product_receipt.rs").read_text(encoding="utf-8")
        behavior = (
            RECEIPT / "product_receipt/tests.rs"
        ).read_text(encoding="utf-8")
        validation = (RECEIPT / "validation.rs").read_text(encoding="utf-8")
        verify = source_between(
            receipt,
            "pub fn verify_integrity",
            "pub fn verify_attestation",
        )

        self.assertIn("validate_receipt_if_normalized(self)?", verify)
        self.assertIn("canonical_receipt_sha256_from_receipt_matches(self", verify)
        self.assertIn("self.normalized_draft()", verify)
        self.assertIn("pub(crate) fn validate_receipt_if_normalized", validation)
        self.assertIn(
            "pub(crate) fn canonical_receipt_sha256_from_receipt_matches", canonical
        )
        self.assertIn(
            "noncanonical_receipt_keeps_legacy_normalizing_verification", behavior
        )
        benchmark = (
            RECEIPT / "product_receipt/performance_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "TOOLING15_BORROWED_RECEIPT_INTEGRITY_BENCH_V1", benchmark
        )

    def test_canonical_receipt_validation_uses_one_shape_walk(self) -> None:
        receipt = (RECEIPT / "product_receipt.rs").read_text(encoding="utf-8")
        validation = (RECEIPT / "validation.rs").read_text(encoding="utf-8")
        verify = source_between(
            receipt,
            "pub fn verify_integrity",
            "pub fn verify_attestation",
        )

        self.assertIn("validate_receipt_if_normalized(self)?", verify)
        self.assertNotIn("is_normalized_receipt(self)", verify)
        self.assertIn("pub(crate) fn validate_receipt_if_normalized", validation)
        benchmark = (
            RECEIPT / "product_receipt/performance_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "TOOLING15_SINGLE_PASS_RECEIPT_SHAPE_BENCH_V1", benchmark
        )

    def test_build_action_digest_borrows_normalized_features_with_sorted_fallback(
        self,
    ) -> None:
        canonical = (RECEIPT / "canonical.rs").read_text(encoding="utf-8")
        action_key = source_between(
            canonical,
            "pub(crate) fn canonical_build_action_key",
            "pub(crate) fn canonical_build_action_sha256",
        )
        action_digest = source_between(
            canonical,
            "pub(crate) fn canonical_build_action_sha256",
            "pub(crate) fn batch_attestation_bytes",
        )

        self.assertIn("struct CanonicalBuildActionKey", canonical)
        self.assertIn("Cow::Borrowed(action.features.as_slice())", action_key)
        self.assertIn("let mut features = action.features.clone()", action_key)
        self.assertIn("features.sort_unstable()", action_key)
        self.assertIn("Cow::Owned(features)", action_key)
        self.assertNotIn(".map(String::as_str)", action_key)
        self.assertIn("canonical_build_action_key(action)", action_digest)
        self.assertNotIn("action.clone()", action_digest)
        self.assertIn(
            "borrowed_build_action_digest_matches_the_legacy_sorted_payload", canonical
        )
        benchmark = (
            RECEIPT / "canonical/performance_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "TOOLING15_BORROWED_BUILD_ACTION_DIGEST_BENCH_V1", benchmark
        )
        self.assertIn("struct LegacyCanonicalBuildActionKey", benchmark)
        self.assertIn(".map(String::as_str)", benchmark)

    def test_batch_action_deduplication_uses_borrowed_structural_keys(self) -> None:
        canonical = (RECEIPT / "canonical.rs").read_text(encoding="utf-8")
        product_batch = PRODUCT_BUILD_BATCH.read_text(encoding="utf-8")
        receipt_batch = (
            RECEIPT / "product_receipt_batch.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("struct CanonicalBuildActionKey", canonical)
        self.assertIn("pub(crate) fn canonical_build_action_key", canonical)
        self.assertIn("structural_build_action_key_ignores_feature_order", canonical)
        self.assertGreaterEqual(product_batch.count("canonical_build_action_key"), 3)
        self.assertGreaterEqual(receipt_batch.count("canonical_build_action_key"), 2)
        self.assertNotIn("canonical_build_action_sha256", product_batch)
        self.assertNotIn("canonical_build_action_sha256", receipt_batch)
        benchmark = (
            RECEIPT / "canonical/performance_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "TOOLING15_STRUCTURAL_BUILD_ACTION_DEDUP_BENCH_V1", benchmark
        )

    def test_verified_draft_batch_handoff_consumes_the_shape_validation(self) -> None:
        cli_run = PRODUCT_RECEIPT_CLI_RUN.read_text(encoding="utf-8")
        batch = PRODUCT_BUILD_BATCH.read_text(encoding="utf-8")
        issue_entry = source_between(
            cli_run,
            "fn issue_product_receipt_draft_batch",
            "fn issue_product_receipt(",
        )
        fast_issue = source_between(
            batch,
            "fn issue_after_shape_validation",
            "fn validate_shape",
        )

        self.assertIn("struct VerifiedProductBuildDraftBatchHandoff", batch)
        self.assertIn("verify_handoff_sha256_owned", batch)
        self.assertIn(
            "ProductBuildDraftBatch::parse_and_verify_handoff_sha256(",
            issue_entry,
        )
        self.assertIn("&draft_bytes", issue_entry)
        self.assertLess(
            issue_entry.index("parse_and_verify_handoff_sha256"),
            issue_entry.index("PRODUCT_RECEIPT_PRIVATE_KEY_LIMIT"),
        )
        self.assertIn("self.issue_after_shape_validation", batch)
        self.assertNotIn("validate_shape", fast_issue)
        benchmark = (
            PRODUCT_BUILD_BATCH.parent / "batch/performance_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "TOOLING15_VERIFIED_DRAFT_BATCH_ISSUE_BENCH_V1", benchmark
        )

    def test_handoff_verification_compares_digest_without_hex_allocation(self) -> None:
        canonical = (RECEIPT / "canonical.rs").read_text(encoding="utf-8")
        product_batch = PRODUCT_BUILD_BATCH.read_text(encoding="utf-8")
        draft = (RECEIPT / "product_receipt_draft.rs").read_text(encoding="utf-8")
        batch_verify = source_between(
            product_batch,
            "pub fn verify_handoff_sha256(&self, expected: &str)",
            "pub fn verify_handoff_sha256_owned",
        )
        draft_verify = source_between(
            draft,
            "pub fn verify_handoff_sha256(&self, expected: &str)",
            "pub fn verify_handoff_sha256_owned",
        )

        self.assertIn("pub(crate) fn serialized_sha256_matches", canonical)
        self.assertIn("fn upper_hex_matches", canonical)
        self.assertIn("self.validate_shape()?", batch_verify)
        self.assertIn("serialized_sha256_matches(self, expected", batch_verify)
        self.assertNotIn("self.handoff_sha256()", batch_verify)
        self.assertIn("serialized_sha256_matches(self, expected", draft_verify)
        self.assertNotIn("self.handoff_sha256()", draft_verify)
        self.assertIn("streamed_digest_match_preserves_canonical_uppercase", canonical)

    def test_integrity_verification_compares_digests_without_hex_allocation(
        self,
    ) -> None:
        canonical = (RECEIPT / "canonical.rs").read_text(encoding="utf-8")
        receipt = (RECEIPT / "product_receipt.rs").read_text(encoding="utf-8")
        receipt_batch = (RECEIPT / "product_receipt_batch.rs").read_text(
            encoding="utf-8"
        )
        receipt_verify = source_between(
            receipt,
            "pub fn verify_integrity(&self)",
            "pub fn verify_attestation(",
        )
        batch_verify = source_between(
            receipt_batch,
            "pub fn verify_integrity(&self)",
            "pub fn verify_attestations(",
        )

        self.assertIn("fn canonical_receipt_sha256_matches", canonical)
        self.assertIn("fn canonical_receipt_sha256_from_receipt_matches", canonical)
        self.assertIn("fn canonical_receipt_batch_sha256_matches", canonical)
        self.assertIn("canonical_receipt_sha256_matches", receipt_verify)
        self.assertIn("canonical_receipt_sha256_from_receipt_matches", receipt_verify)
        self.assertNotIn("let actual =", receipt_verify)
        self.assertIn("canonical_receipt_batch_sha256_matches", batch_verify)
        self.assertNotIn("let actual =", batch_verify)

    def test_toolchain_identity_verification_compares_digest_without_hex_allocation(
        self,
    ) -> None:
        toolchain = (RECEIPT / "toolchain_set.rs").read_text(encoding="utf-8")
        benchmark = (
            RECEIPT / "product_receipt/performance_tests.rs"
        ).read_text(encoding="utf-8")
        verify = source_between(
            toolchain,
            "fn verify_declared_identity(&self)",
            "fn normalize_components(&mut self)",
        )
        impl_tail = source_between(
            toolchain,
            "fn derived_id(&self)",
            "\n}\n\nfn normalize_digest",
        )

        self.assertIn("serialized_sha256_matches", verify)
        self.assertIn("&self.toolchain_set_id", verify)
        self.assertNotIn("self.derived_id()?", verify)
        self.assertIn("fn canonical_payload(&self)", impl_tail)
        self.assertIn("sha256_serialized", impl_tail)
        self.assertIn("self.canonical_payload()", impl_tail)
        self.assertIn("TOOLING15_NORMALIZED_DRAFT_ISSUE_BENCH_V1", benchmark)

    def test_file_content_verification_compares_digest_without_hex_allocation(
        self,
    ) -> None:
        canonical = (RECEIPT / "canonical.rs").read_text(encoding="utf-8")
        file_digest = (RECEIPT / "file_digest.rs").read_text(encoding="utf-8")
        materialization = (RECEIPT / "materialization.rs").read_text(
            encoding="utf-8"
        )
        build_set = PRODUCT_BUILD_SET.read_text(encoding="utf-8")
        benchmark = (
            RECEIPT / "canonical/performance_tests.rs"
        ).read_text(encoding="utf-8")
        build_set_verify = source_between(
            build_set,
            "fn verify_file_content",
            "fn open_verified_snapshot_file",
        )

        self.assertIn("pub(crate) fn upper_hex_matches", canonical)
        self.assertIn("digest_open_file_handle_bytes", file_digest)
        self.assertIn("RawFileDigest", file_digest)
        self.assertIn("digest_open_file_handle_bytes", materialization)
        self.assertIn("upper_hex_matches(&digest.sha256", materialization)
        self.assertIn("upper_hex_matches(&actual_digest", build_set_verify)
        self.assertNotIn("hex_digest(&hasher.finalize())", build_set_verify)
        self.assertIn("TOOLING15_DIRECT_DIGEST_MATCH_BENCH_V1", benchmark)

    def test_build_set_inventory_reuses_relative_path_buffer(self) -> None:
        build_set = PRODUCT_BUILD_SET.read_text(encoding="utf-8")
        benchmark = (
            PRODUCT_BUILD_SET.parent / "build_set/performance_tests.rs"
        ).read_text(encoding="utf-8")
        collection = source_between(
            build_set,
            "fn collect_snapshot_inventory",
            "fn verify_snapshot_inventory",
        )
        verification = source_between(
            build_set,
            "fn verify_snapshot_inventory",
            "fn visit_snapshot_files",
        )
        visitor = source_between(
            build_set,
            "fn visit_snapshot_files",
            "fn derive_build_set_id",
        )

        self.assertIn("impl FnMut(&str)", visitor)
        self.assertIn("let mut relative = String::new();", visitor)
        self.assertIn("snapshot_relative_path_into(", visitor)
        self.assertIn("inventory.push(relative.to_owned())", collection)
        self.assertIn("expected_inventory.contains(relative)", verification)
        self.assertIn("TOOLING15_BUILD_SET_REUSED_PATH_BUFFER_BENCH_V1", benchmark)

    def test_materialization_inventory_reuses_relative_path_buffer(self) -> None:
        materialization = (RECEIPT / "materialization.rs").read_text(
            encoding="utf-8"
        )
        benchmark = (
            RECEIPT / "materialization/performance_tests.rs"
        ).read_text(encoding="utf-8")
        inventory = source_between(
            materialization,
            "fn inventory_materialization",
            "\n#[cfg(windows)]\nfn open_locked_directory",
        )

        self.assertIn("let mut relative = String::new();", inventory)
        self.assertIn("inventory_relative_path_into(", inventory)
        self.assertIn("pending.push((path, relative.clone()))", inventory)
        self.assertIn("fn inventory_relative_path_into", materialization)
        self.assertIn("TOOLING15_MATERIALIZATION_DIRECT_PATH_BENCH_V1", benchmark)
        self.assertIn("let mut relative = String::new();", benchmark)
        self.assertIn("inventory_relative_path_into(", benchmark)
        verification = source_between(
            materialization,
            "fn verify_windows_materialization",
            "fn open_locked_absolute_directory_chain",
        )
        self.assertIn("let mut artifact_path = PathBuf::new();", verification)
        self.assertIn("materialization_path_into(", verification)
        self.assertNotIn("artifact_root.join(&artifact.relative_path)", verification)
        self.assertIn("fn materialization_path_into", materialization)
        self.assertIn("TOOLING15_MATERIALIZATION_PATH_BUFFER_BENCH_V1", benchmark)

    def test_fresh_receipt_batch_issue_skips_redundant_child_integrity_walk(self) -> None:
        product_build_batch = PRODUCT_BUILD_BATCH.read_text(encoding="utf-8")
        receipt_batch_path = RECEIPT / "product_receipt_batch.rs"
        receipt_batch = receipt_batch_path.read_text(encoding="utf-8")
        behavior = (
            RECEIPT / "product_receipt_batch/tests.rs"
        ).read_text(encoding="utf-8")
        public_issue = source_between(
            receipt_batch,
            "pub fn issue",
            "pub(crate) fn issue_after_validated_closure",
        )
        fresh_issue = source_between(
            receipt_batch,
            "pub(crate) fn issue_after_validated_closure",
            "#[cfg(test)]",
        )

        self.assertIn("receipt.verify_integrity()?", public_issue)
        self.assertIn(
            "ProductReceiptBatch::issue_fresh_from_batch_shape_drafts",
            product_build_batch,
        )
        self.assertNotIn("verify_integrity", fresh_issue)
        self.assertIn("public_batch_issue_rejects_a_tampered_child_receipt", behavior)
        benchmark = (
            RECEIPT / "product_receipt_batch/performance_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "TOOLING15_FRESH_RECEIPT_BATCH_ISSUE_BENCH_V1", benchmark
        )

    def test_validated_draft_batch_skips_redundant_receipt_batch_closure_walk(self) -> None:
        product_build_batch = PRODUCT_BUILD_BATCH.read_text(encoding="utf-8")
        receipt_batch = (
            RECEIPT / "product_receipt_batch.rs"
        ).read_text(encoding="utf-8")
        public_issue = source_between(
            receipt_batch,
            "pub fn issue",
            "pub(crate) fn issue_after_validated_closure",
        )
        validated_issue = source_between(
            receipt_batch,
            "pub(crate) fn issue_after_validated_closure",
            "#[cfg(test)]",
        )

        self.assertIn("batch.validate_closure_shape()?", public_issue)
        self.assertIn(
            "ProductReceiptBatch::issue_fresh_from_batch_shape_drafts",
            product_build_batch,
        )
        self.assertNotIn("validate_closure_shape", validated_issue)
        benchmark = (
            RECEIPT / "product_receipt_batch/performance_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "TOOLING15_VALIDATED_BATCH_CLOSURE_BENCH_V1", benchmark
        )

    def test_verified_write_does_not_repeat_the_integrity_walk(self) -> None:
        receipt = (RECEIPT / "product_receipt.rs").read_text(encoding="utf-8")
        writer = (RECEIPT / "receipt_writer.rs").read_text(encoding="utf-8")
        write_entry = source_between(
            receipt,
            "pub fn write_new_verified",
            "fn draft(&self)",
        )

        self.assertIn("self.verify_attestation(verifier)?", write_entry)
        self.assertIn("receipt_writer::write_new_after_verification", write_entry)
        self.assertNotIn("verify_integrity()?", writer)

    def test_fresh_issued_batch_verifies_attestations_without_rehashing_closure(
        self,
    ) -> None:
        cli_run = PRODUCT_RECEIPT_CLI_RUN.read_text(encoding="utf-8")
        product_batch = PRODUCT_BUILD_BATCH.read_text(encoding="utf-8")
        receipt_batch = (RECEIPT / "product_receipt_batch.rs").read_text(
            encoding="utf-8"
        )
        behavior = (
            RECEIPT / "product_receipt_batch/tests.rs"
        ).read_text(encoding="utf-8")
        issue_entry = source_between(
            cli_run,
            "fn issue_product_receipt_draft_batch",
            "fn issue_product_receipt(",
        )
        fresh_verification = source_between(
            receipt_batch,
            "impl FreshProductReceiptBatch",
            "impl ProductReceiptBatch",
        )

        self.assertIn("pub struct VerifiedProductReceiptBatchPublication", receipt_batch)
        self.assertIn("pub(crate) struct FreshProductReceiptBatch", receipt_batch)
        self.assertIn("pub fn issue_verified", product_batch)
        self.assertIn("issue_fresh_after_shape_validation", product_batch)
        self.assertIn("verify_attestations", product_batch)
        self.assertIn(
            "verify_batch_attestation_payload_after_integrity", fresh_verification
        )
        self.assertIn(
            "receipt.verify_attestation_payload_after_integrity", fresh_verification
        )
        self.assertNotIn("verify_integrity", fresh_verification)
        self.assertNotIn("validate_closure_shape", fresh_verification)
        self.assertIn("draft_batch.issue_verified", issue_entry)
        self.assertIn("publication.write_new", issue_entry)
        self.assertNotIn("write_new_verified", issue_entry)
        self.assertLess(issue_entry.index("trust_registry"), issue_entry.index("private_key"))
        self.assertIn(
            "fresh_publication_proof_checks_batch_and_every_child_attestation_once",
            behavior,
        )
        self.assertIn(
            "fresh_publication_proof_rejects_any_untrusted_child_attestation",
            behavior,
        )
        benchmark = (
            PRODUCT_BUILD_BATCH.parent / "batch/performance_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "TOOLING15_FRESH_VERIFIED_PUBLICATION_BENCH_V1", benchmark
        )

    def test_fresh_issued_receipt_verifies_attestation_without_rehashing_identity(
        self,
    ) -> None:
        cli_run = PRODUCT_RECEIPT_CLI_RUN.read_text(encoding="utf-8")
        draft = (RECEIPT / "product_receipt_draft.rs").read_text(encoding="utf-8")
        receipt = (RECEIPT / "product_receipt.rs").read_text(encoding="utf-8")
        behavior = (RECEIPT / "product_receipt/tests.rs").read_text(
            encoding="utf-8"
        )
        issue_entry = source_between(
            cli_run,
            "fn issue_product_receipt_draft(",
            "fn issue_product_receipt_draft_batch(",
        )
        direct_issue_entry = source_between(
            cli_run,
            "fn issue_product_receipt(",
            "fn verify_product_receipt(",
        )
        fresh_verification = source_between(
            receipt,
            "impl FreshProductReceipt",
            "impl ProductReceipt",
        )

        self.assertIn("struct VerifiedProductReceiptDraftHandoff", draft)
        self.assertIn("verify_handoff_sha256_owned", draft)
        self.assertIn("draft.issue_verified", issue_entry)
        self.assertIn("publication.write_new", issue_entry)
        self.assertNotIn("write_new_verified", issue_entry)
        self.assertLess(
            issue_entry.index("trust_registry"), issue_entry.index("private_key")
        )
        self.assertIn("verify_attestation_payload_after_integrity", fresh_verification)
        self.assertNotIn("verify_integrity", fresh_verification)
        self.assertIn("ProductReceipt::issue_verified", direct_issue_entry)
        self.assertIn("publication.write_new", direct_issue_entry)
        self.assertNotIn("write_new_verified", direct_issue_entry)
        self.assertIn(
            "fresh_receipt_publication_checks_attestation_once", behavior
        )
        self.assertIn(
            "fresh_receipt_publication_rejects_an_untrusted_attestation", behavior
        )
        self.assertIn(
            "direct_fresh_receipt_publication_checks_attestation_once", behavior
        )
        benchmark = (
            RECEIPT / "product_receipt/performance_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "TOOLING15_FRESH_VERIFIED_RECEIPT_PUBLICATION_BENCH_V1", benchmark
        )

    def test_fresh_publication_reuses_raw_signatures_without_hex_decode(self) -> None:
        product_batch = PRODUCT_BUILD_BATCH.read_text(encoding="utf-8")
        receipt = (RECEIPT / "product_receipt.rs").read_text(encoding="utf-8")
        receipt_batch = (RECEIPT / "product_receipt_batch.rs").read_text(
            encoding="utf-8"
        )
        receipt_behavior = (RECEIPT / "product_receipt/tests.rs").read_text(
            encoding="utf-8"
        )
        batch_behavior = (RECEIPT / "product_receipt_batch/tests.rs").read_text(
            encoding="utf-8"
        )
        fresh_receipt = source_between(
            receipt,
            "pub(crate) struct FreshProductReceipt",
            "impl ProductReceipt",
        )
        fresh_attestation = source_between(
            receipt,
            "pub(crate) struct FreshAttestation",
            "pub(crate) struct FreshProductReceipt",
        )
        fresh_batch = source_between(
            receipt_batch,
            "pub(crate) struct FreshProductReceiptBatch",
            "impl ProductReceiptBatch",
        )

        self.assertIn("payload: Vec<u8>", fresh_attestation)
        self.assertIn("signature: Vec<u8>", fresh_attestation)
        self.assertIn("attestation: FreshAttestation", fresh_receipt)
        self.assertIn("verify_attestation_payload_after_integrity", fresh_receipt)
        self.assertNotIn("verify_attestation_after_integrity", fresh_receipt)
        self.assertIn(
            "ProductReceiptBatch::issue_fresh_from_batch_shape_drafts", product_batch
        )
        self.assertIn("ProductReceipt::issue_fresh_after_batch_shape", receipt_batch)
        self.assertIn("batch_attestation: FreshAttestation", fresh_batch)
        self.assertIn(
            "receipt_attestations: Option<Vec<FreshAttestation>>", fresh_batch
        )
        self.assertIn("verify_batch_attestation_payload_after_integrity", fresh_batch)
        self.assertIn("verify_attestation_payload_after_integrity", fresh_batch)
        self.assertIn(
            "fresh_receipt_publication_verifies_the_retained_raw_signature",
            receipt_behavior,
        )
        self.assertIn(
            "fresh_receipt_publication_reuses_the_signed_payload",
            receipt_behavior,
        )
        self.assertIn(
            "fresh_batch_publication_verifies_retained_raw_signatures",
            batch_behavior,
        )
        self.assertIn(
            "fresh_batch_publication_reuses_signed_payloads",
            batch_behavior,
        )

    def test_normalized_draft_issuance_avoids_repeat_sorting(self) -> None:
        validation = (RECEIPT / "validation.rs").read_text(encoding="utf-8")
        behavior = (RECEIPT / "product_receipt/tests.rs").read_text(
            encoding="utf-8"
        )
        benchmark = (
            RECEIPT / "product_receipt/performance_tests.rs"
        ).read_text(encoding="utf-8")
        normalize_entry = source_between(
            validation,
            "pub(crate) fn normalize_and_validate(",
            "pub(crate) fn validate_receipt_if_normalized(",
        )

        self.assertIn("validate_draft_if_normalized(draft, created_utc)?", normalize_entry)
        self.assertIn("normalize_and_validate_owned(draft, created_utc)", normalize_entry)
        self.assertLess(
            normalize_entry.index("validate_draft_if_normalized"),
            normalize_entry.index("normalize_and_validate_owned"),
        )
        self.assertIn("fn validate_draft_if_normalized(", normalize_entry)
        self.assertIn("validate_action_if_normalized", normalize_entry)
        self.assertIn("validate_artifacts_if_normalized", normalize_entry)
        self.assertIn(
            "fresh_receipt_issue_normalizes_external_unordered_draft", behavior
        )
        self.assertIn(
            "TOOLING15_NORMALIZED_DRAFT_ISSUE_BENCH_V1", benchmark
        )

    def test_product_build_batch_issuance_reuses_artifact_uniqueness_proof(
        self,
    ) -> None:
        product_batch = PRODUCT_BUILD_BATCH.read_text(encoding="utf-8")
        receipt = (RECEIPT / "product_receipt.rs").read_text(encoding="utf-8")
        receipt_batch = (RECEIPT / "product_receipt_batch.rs").read_text(
            encoding="utf-8"
        )
        validation = (RECEIPT / "validation.rs").read_text(encoding="utf-8")
        behavior = (
            ROOT / "tools/cargo-zircon/src/build/product_build/batch/tests.rs"
        ).read_text(encoding="utf-8")
        benchmark = (
            RECEIPT / "product_receipt/performance_tests.rs"
        ).read_text(encoding="utf-8")
        batch_benchmark = (
            PRODUCT_BUILD_BATCH.parent / "batch/performance_tests.rs"
        ).read_text(encoding="utf-8")
        issue_entry = source_between(
            product_batch,
            "fn issue_fresh_after_shape_validation(",
            "fn validate_shape(&self)",
        )

        self.assertIn(
            "ProductReceiptBatch::issue_fresh_from_batch_shape_drafts", issue_entry
        )
        self.assertIn("fn issue_fresh_after_batch_shape_with_signer(", receipt)
        proof_issue_entry = source_between(
            receipt_batch,
            "pub(crate) fn issue_fresh_from_batch_shape_drafts(",
            "pub(crate) fn issue_fresh_after_validated_receipts(",
        )
        self.assertIn(
            "ProductReceipt::issue_fresh_after_batch_shape_with_signer",
            proof_issue_entry,
        )
        self.assertIn("validate_created_utc_for_batch", proof_issue_entry)
        self.assertIn("ValidatedCreatedUtc", proof_issue_entry)
        self.assertIn("normalize_and_validate_after_batch_shape_with_validated_utc", receipt)
        self.assertIn(
            "pub(crate) fn normalize_and_validate_after_batch_shape(", validation
        )
        proof_validation = source_between(
            validation,
            "pub(crate) fn normalize_and_validate_after_batch_shape(",
            "#[cfg(test)]\npub(crate) fn normalize_and_validate_owned_for_benchmark(",
        )
        self.assertIn("validate_draft_fields_if_normalized", proof_validation)
        self.assertIn("normalize_and_validate_owned_fields", proof_validation)
        self.assertNotIn("validate_unique_artifact_names", proof_validation)
        self.assertIn("validate_created_utc_for_batch", proof_validation)
        self.assertIn("normalize_and_validate_owned_fields_without_timestamp", validation)
        self.assertIn(
            "product_build_batch_issue_reuses_validated_artifact_uniqueness",
            behavior,
        )
        self.assertIn(
            "product_build_batch_issue_still_validates_artifact_fields", behavior
        )
        self.assertIn(
            "product_build_batch_issue_rejects_invalid_shared_created_utc", behavior
        )
        self.assertIn("normalize_and_validate_after_batch_shape", benchmark)
        self.assertIn(
            "TOOLING15_BATCH_TIMESTAMP_VALIDATION_BENCH_V1", batch_benchmark
        )

    def test_product_build_batch_issuance_streams_fresh_drafts(self) -> None:
        product_batch = PRODUCT_BUILD_BATCH.read_text(encoding="utf-8")
        receipt_batch = (RECEIPT / "product_receipt_batch.rs").read_text(
            encoding="utf-8"
        )
        behavior = (
            ROOT / "tools/cargo-zircon/src/build/product_build/batch/tests.rs"
        ).read_text(encoding="utf-8")
        issue_entry = source_between(
            product_batch,
            "fn issue_fresh_after_shape_validation(",
            "fn validate_shape(&self)",
        )

        self.assertIn(
            "ProductReceiptBatch::issue_fresh_from_batch_shape_drafts", issue_entry
        )
        self.assertNotIn("collect::<Result<Vec<_>, _>>()", issue_entry)
        streaming_entry = source_between(
            receipt_batch,
            "pub(crate) fn issue_fresh_from_batch_shape_drafts(",
            "pub(crate) fn issue_fresh_after_validated_receipts(",
        )
        self.assertIn("Vec::with_capacity(draft_count)", streaming_entry)
        self.assertIn("std::mem::take(&mut created_utc)", streaming_entry)
        self.assertIn("receipt_attestations.push(attestation)", streaming_entry)
        self.assertIn(
            "product_build_batch_issue_reuses_validated_artifact_uniqueness",
            behavior,
        )

    def test_verified_write_flushes_before_atomic_no_overwrite_publication(self) -> None:
        writer = (RECEIPT / "receipt_writer.rs").read_text(encoding="utf-8")
        write_entry = source_between(
            writer,
            "pub(crate) fn write_new_json",
            "fn create_temporary_receipt",
        )
        flush_helper = writer[writer.index("fn write_and_flush") :]

        self.assertIn("create_new(true)", writer)
        self.assertIn("file.sync_all()?", flush_helper)
        self.assertIn("fs::hard_link(&temporary_path, output_path)", writer)
        self.assertIn("fs::remove_file(&temporary_path)", writer)
        self.assertLess(write_entry.index("write_and_flush"), write_entry.index("fs::hard_link"))
        self.assertIn("if let Err(error) = published", write_entry)
        self.assertNotIn("cleanup?", write_entry)
        publication_tail = writer[writer.index("let published") : writer.index("fn create_temporary_receipt")]
        self.assertLess(publication_tail.index("fs::hard_link"), publication_tail.rindex("drop(file)"))
        self.assertIn("share_mode(0x0000_0001)", writer)
        self.assertIn("BufWriter::with_capacity", writer)
        self.assertIn("serde_json::to_writer_pretty", writer)
        self.assertNotIn("serde_json::to_vec_pretty", writer)
        writer_tests = (
            RECEIPT / "receipt_writer/tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "streaming_pretty_json_matches_the_legacy_buffered_bytes", writer_tests
        )
        benchmark = (
            RECEIPT / "receipt_writer/performance_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("TOOLING15_STREAMING_RECEIPT_WRITE_BENCH_V1", benchmark)

    def test_receipt_closure_rejects_path_aliases_and_partition_kind_confusion(self) -> None:
        validation = (RECEIPT / "validation.rs").read_text(encoding="utf-8")

        self.assertIn("validate_artifact_kind", validation)
        self.assertIn("artifact.relative_path.as_str()", validation)
        self.assertIn("duplicate artifact relative path", validation)
        self.assertIn("ArtifactKind::Executable", validation)
        self.assertIn("ArtifactKind::DynamicLibrary | ArtifactKind::Resource", validation)
        self.assertIn("ArtifactKind::SymbolFile", validation)
        self.assertIn("ArtifactKind::Sbom", validation)

    def test_trust_registry_decodes_keys_once_and_uses_single_issuer_direct_path(
        self,
    ) -> None:
        authority = (RECEIPT / "ed25519_authority.rs").read_text(encoding="utf-8")
        verifier = source_between(
            authority,
            "impl ProductReceiptVerifier for ProductReceiptTrustRegistry",
            "fn decode_public_key",
        )

        self.assertIn("enum TrustedIssuers", authority)
        self.assertIn("Single {", authority)
        self.assertIn("Multiple(HashMap<String, TrustedIssuer>)", authority)
        self.assertIn("HashMap::with_capacity(document.issuers.len())", authority)
        self.assertIn("public_key: [u8; ED25519_PUBLIC_KEY_LENGTH]", authority)
        self.assertEqual(authority.count("decode_public_key("), 2)
        self.assertIn("issuers.entry(signer_id)", authority)
        self.assertIn("Entry::Vacant", authority)
        self.assertNotIn("issuer.signer_id.clone()", authority)
        self.assertIn("TrustedIssuers::Single", verifier)
        self.assertIn("TrustedIssuers::Multiple(issuers)", verifier)
        self.assertIn("issuers.get(signer_id)", verifier)
        self.assertNotIn("decode_public_key", verifier)
        self.assertNotIn("decode_hex", verifier)
        benchmark = (
            RECEIPT / "ed25519_authority/performance_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("TOOLING15_TRUST_REGISTRY_KEY_MOVE_BENCH_V1", benchmark)

    def test_ed25519_authority_defers_display_hex_and_borrows_registry_text(
        self,
    ) -> None:
        authority = (RECEIPT / "ed25519_authority.rs").read_text(encoding="utf-8")
        signer = source_between(
            authority,
            "pub struct Ed25519ProductReceiptSigner",
            "impl ProductReceiptSigner for Ed25519ProductReceiptSigner",
        )
        registry_documents = source_between(
            authority,
            "struct TrustRegistryDocument",
            "struct TrustedIssuer {",
        )

        self.assertIn("use std::borrow::Cow", authority)
        self.assertIn("use std::sync::OnceLock", authority)
        self.assertIn("public_key_hex: OnceLock<String>", signer)
        self.assertIn("public_key_hex: OnceLock::new()", signer)
        self.assertIn(".get_or_init(|| bytes_to_hex", signer)
        self.assertNotIn("let public_key_hex = bytes_to_hex", signer)
        self.assertGreaterEqual(registry_documents.count("Cow<'a, str>"), 3)
        self.assertIn("#[serde(borrow)]", registry_documents)
        self.assertIn("public_key_hex_is_cached_only_when_requested", authority)
        self.assertIn("borrowed_registry_text_retains_escaped_json_support", authority)

    def test_product_build_batch_captures_validated_signer_metadata_once(self) -> None:
        receipt = (RECEIPT / "product_receipt.rs").read_text(encoding="utf-8")
        receipt_batch = (RECEIPT / "product_receipt_batch.rs").read_text(
            encoding="utf-8"
        )
        behavior = (
            RECEIPT / "product_receipt_batch/tests.rs"
        ).read_text(encoding="utf-8")
        batch_issue = source_between(
            receipt_batch,
            "pub(crate) fn issue_fresh_from_batch_shape_drafts(",
            "pub(crate) fn issue_fresh_after_validated_receipts(",
        )

        self.assertIn("pub(crate) struct ValidatedProductReceiptSigner", receipt)
        self.assertIn("ValidatedProductReceiptSigner::new(signer)?", batch_issue)
        self.assertIn("issue_fresh_after_batch_shape_with_signer", batch_issue)
        self.assertIn("signer: &ValidatedProductReceiptSigner<'_>", receipt_batch)
        self.assertIn("batch_issue_reads_signer_metadata_once", behavior)

    def test_cli_bounds_receipt_inputs_and_self_verifies_before_new_write(self) -> None:
        main = (ROOT / "tools/cargo-zircon/src/main.rs").read_text(encoding="utf-8")
        cli_run = PRODUCT_RECEIPT_CLI_RUN.read_text(encoding="utf-8")
        cli_input = PRODUCT_RECEIPT_CLI_INPUT.read_text(encoding="utf-8")
        closure_source = (RECEIPT / "product_receipt_closure.rs").read_text(encoding="utf-8")
        issue = source_between(cli_run, "fn issue_product_receipt", "fn verify_product_receipt")
        bounded_read = source_between(cli_input, "pub(super) fn read_bounded", "#[cfg(test)]")

        self.assertIn("mod product_receipt_cli;", main)
        self.assertIn("return product_receipt_cli::run(arguments);", main)
        self.assertNotIn("fn issue_product_receipt", main)
        self.assertIn("PRODUCT_RECEIPT_JSON_LIMIT", cli_run)
        self.assertIn("PRODUCT_RECEIPT_TRUST_REGISTRY_LIMIT", cli_run)
        self.assertIn("PRODUCT_RECEIPT_PRIVATE_KEY_LIMIT", cli_run)
        self.assertIn("let closure: ProductReceiptClosure", issue)
        self.assertIn("let draft = closure.capture()?", issue)
        self.assertIn("contents: &'a mut Vec<u8>", bounded_read)
        self.assertIn("contents.clear()", bounded_read)
        self.assertIn("contents.reserve(limit.min(PRODUCT_RECEIPT_READ_CAPACITY))", bounded_read)
        self.assertIn("reader.take(limit as u64 + 1).read_to_end(contents)", bounded_read)
        self.assertNotIn("Vec::with_capacity", bounded_read)
        self.assertNotIn(".metadata()", bounded_read)
        self.assertIn("bounded_read_accepts_the_exact_limit", cli_input)
        self.assertIn("bounded_read_rejects_limit_plus_one", cli_input)
        self.assertIn("bounded_reads_reuse_existing_capacity", cli_input)
        self.assertIn("mod performance_tests;", cli_input)
        self.assertIn(
            "TOOLING15_REUSED_BOUNDED_INPUT_BUFFER_BENCH_V1",
            PRODUCT_RECEIPT_CLI_INPUT_PERFORMANCE.read_text(encoding="utf-8"),
        )
        batch_issue = source_between(
            cli_run,
            "fn issue_product_receipt_draft_batch",
            "fn issue_product_receipt(",
        )
        self.assertIn("let mut input = Vec::new();", batch_issue)
        self.assertEqual(batch_issue.count("&mut input"), 3)
        self.assertIn("drop(input);", batch_issue)
        self.assertIn("ProductReceipt::issue_verified", issue)
        self.assertIn("publication.write_new(&options.output)?", issue)
        self.assertIn("require_immutable_capture_platform()?", closure_source)
        self.assertIn("immutable ProductReceipt capture backend", closure_source)

    def test_cli_consumes_owned_issue_arguments_without_string_clones(self) -> None:
        cli_run = PRODUCT_RECEIPT_CLI_RUN.read_text(encoding="utf-8")
        cli_options = PRODUCT_RECEIPT_CLI_OPTIONS.read_text(encoding="utf-8")
        route = source_between(
            cli_run,
            "pub(crate) fn run",
            "fn build_product_receipt_draft_file",
        )
        batch_options = source_between(
            cli_options,
            "pub(super) fn parse_product_receipt_draft_batch_issue_options",
            "pub(super) fn parse_product_receipt_issue_options",
        )

        self.assertIn("let mut arguments = arguments.into_iter();", route)
        self.assertIn("let command = arguments.next().ok_or_else(usage_error)?;", route)
        self.assertNotIn("arguments.remove(0)", route)
        self.assertNotIn("first().cloned()", route)
        self.assertEqual(
            cli_options.count("arguments: impl IntoIterator<Item = String>"), 6
        )
        self.assertEqual(cli_options.count("let mut arguments = arguments.into_iter();"), 6)
        self.assertIn("while let Some(argument) = arguments.next()", batch_options)
        self.assertIn("let value = arguments.next().ok_or_else(usage_error)?", batch_options)
        self.assertIn("set_once(&mut expected_draft_sha256, value)?", batch_options)
        self.assertIn("set_once(&mut signer_id, value)?", batch_options)
        self.assertIn("set_once(&mut created_utc, value)?", batch_options)
        self.assertNotIn("value.clone()", batch_options)
        self.assertNotIn("arguments.get(", batch_options)
        self.assertIn("owned_batch_issue_options_preserve_values", cli_options)
        self.assertIn("owned_options_reject_duplicates_and_missing_values", cli_options)
        self.assertIn("mod performance_tests;", cli_options)
        self.assertIn(
            "TOOLING15_SINGLE_PASS_CLI_ARGUMENT_ROUTE_BENCH_V1",
            PRODUCT_RECEIPT_CLI_OPTIONS_PERFORMANCE.read_text(encoding="utf-8"),
        )

    def test_cli_verifies_the_exact_locked_artifact_materialization(self) -> None:
        cli_run = PRODUCT_RECEIPT_CLI_RUN.read_text(encoding="utf-8")
        cli_options = PRODUCT_RECEIPT_CLI_OPTIONS.read_text(encoding="utf-8")
        receipt = (RECEIPT / "product_receipt.rs").read_text(encoding="utf-8")
        materialization = (RECEIPT / "materialization.rs").read_text(encoding="utf-8")
        verify_entry = source_between(
            cli_run,
            "fn verify_product_receipt",
            "fn verify_product_receipt_batch",
        )

        self.assertIn("artifact_root: PathBuf", cli_options)
        self.assertIn(
            "receipt.verify_attestation_and_materialization(&registry, &options.artifact_root)?",
            verify_entry,
        )
        self.assertIn("self.verify_integrity()?", receipt)
        self.assertIn("inventory_materialization", materialization)
        self.assertIn("undeclared artifact", materialization)
        self.assertIn("undeclared directory", materialization)
        self.assertIn("insert_expected_artifact_directories", materialization)
        self.assertIn("open_locked_directory", materialization)
        self.assertIn("open_locked_absolute_directory_chain", materialization)
        self.assertIn("for component in path.components()", materialization)
        self.assertIn("FILE_ATTRIBUTE_REPARSE_POINT", materialization)
        self.assertIn("digest_open_file_handle", materialization)
        self.assertIn("immutable ProductReceipt materialization verifier", materialization)

    def test_materialization_inventory_carries_relative_directory_prefix(self) -> None:
        materialization = (RECEIPT / "materialization.rs").read_text(
            encoding="utf-8"
        )
        inventory = source_between(
            materialization,
            "fn inventory_materialization",
            "fn inventory_relative_path",
        )
        relative_path = source_between(
            materialization,
            "fn inventory_relative_path",
            "fn open_locked_directory",
        )
        benchmark = (
            RECEIPT / "materialization/performance_tests.rs"
        ).read_text(encoding="utf-8")
        behavior = (RECEIPT / "materialization/tests.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn(
            "let mut pending = vec![(artifact_root.to_path_buf(), String::new())]",
            inventory,
        )
        self.assertIn(
            "while let Some((directory, relative_directory)) = pending.pop()",
            inventory,
        )
        self.assertIn("inventory_relative_path_into(", inventory)
        self.assertIn("&mut relative", inventory)
        self.assertNotIn("strip_prefix", inventory)
        self.assertIn("String::with_capacity", relative_path)
        self.assertIn("TOOLING15_MATERIALIZATION_DIRECT_PATH_BENCH_V1", benchmark)
        self.assertIn("inventory_relative_path", benchmark)
        self.assertIn(
            "inventory_relative_path_appends_one_entry_to_the_carried_directory",
            behavior,
        )
        self.assertIn(
            "inventory_relative_path_rejects_non_unicode_entry_names", behavior
        )

    def test_materialization_reuses_opened_file_metadata_for_reparse_validation(
        self,
    ) -> None:
        materialization = (RECEIPT / "materialization.rs").read_text(
            encoding="utf-8"
        )
        artifact_verification = source_between(
            materialization,
            "let mut locked_files = Vec::with_capacity(artifact_count)",
            "drop(locked_files)",
        )
        behavior = (RECEIPT / "materialization/tests.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("let mut file = open_locked_file", artifact_verification)
        self.assertIn("let metadata = file.metadata()", artifact_verification)
        self.assertIn("reject_reparse_metadata(&metadata", artifact_verification)
        self.assertNotIn("reject_reparse_path", artifact_verification)
        self.assertNotIn("fn reject_reparse_path", materialization)
        self.assertIn(".custom_flags(0x0020_0000)", materialization)
        self.assertIn(
            "opened_artifact_handle_supplies_reparse_validation_metadata", behavior
        )

    def test_materialization_reuses_one_product_digest_buffer(self) -> None:
        file_digest = (RECEIPT / "file_digest.rs").read_text(encoding="utf-8")
        materialization = (RECEIPT / "materialization.rs").read_text(
            encoding="utf-8"
        )
        verification = source_between(
            materialization,
            "let mut locked_files = Vec::with_capacity(artifact_count)",
            "drop(locked_files)",
        )
        behavior = (
            RECEIPT / "file_digest/tests.rs"
        ).read_text(encoding="utf-8")
        benchmark = (
            RECEIPT / "file_digest/performance_tests.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("pub(crate) struct FileDigestBuffer", file_digest)
        self.assertIn(
            "digest_open_file_handle_bytes_with_buffer", file_digest
        )
        self.assertIn("let mut digest_buffer = FileDigestBuffer::new()", verification)
        self.assertIn(
            "digest_open_file_handle_bytes_with_buffer(&mut file, &mut digest_buffer)",
            verification,
        )
        self.assertIn("shared_digest_buffer_hashes_multiple_open_files", behavior)
        self.assertIn(
            "TOOLING15_PRODUCT_DIGEST_BUFFER_REUSE_BENCH_V1", benchmark
        )

    def test_build_set_inventory_carries_relative_directory_prefix(self) -> None:
        build_set = PRODUCT_BUILD_SET.read_text(encoding="utf-8")
        inventory = source_between(
            build_set,
            "fn visit_snapshot_files",
            "fn derive_build_set_id",
        )
        relative_path = source_between(
            build_set,
            "fn snapshot_relative_path",
            "fn ordinal_compare",
        )
        build_set_tests = build_set_test_sources()

        self.assertIn(
            "let mut pending = vec![(snapshot_root.to_path_buf(), 0_usize, String::new())]",
            inventory,
        )
        self.assertIn(
            "while let Some((directory, depth, relative_directory)) = pending.pop()",
            inventory,
        )
        self.assertIn("snapshot_relative_path_into(", inventory)
        self.assertIn("&mut relative", inventory)
        self.assertNotIn("strip_prefix(snapshot_root)", inventory)
        self.assertIn("String::with_capacity", relative_path)
        self.assertIn(
            "TOOLING15_BUILD_SET_RELATIVE_PATH_BUFFER_BENCH_V1", build_set_tests
        )
        self.assertIn("snapshot_relative_path", build_set_tests)
        self.assertIn(
            "snapshot_relative_path_appends_one_entry_to_the_carried_directory",
            build_set_tests,
        )

    def test_build_set_constructs_validated_relative_paths_directly(self) -> None:
        build_set = PRODUCT_BUILD_SET.read_text(encoding="utf-8")
        relative_path = source_between(
            build_set,
            "fn relative_path(value:",
            "fn validate_relative_path(value:",
        )
        behavior = build_set_test_sources()
        benchmark = (
            PRODUCT_BUILD_SET.parent / "build_set/performance_tests.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("PathBuf::from(value)", relative_path)
        self.assertNotIn("value.split('/').collect()", relative_path)
        self.assertIn(
            "direct_relative_path_construction_preserves_components", behavior
        )

    def test_build_set_relative_path_validation_is_platform_independent(self) -> None:
        build_set = PRODUCT_BUILD_SET.read_text(encoding="utf-8")
        validation = source_between(
            build_set,
            "fn validate_relative_path(value:",
            "fn snapshot_relative_path_into(",
        )
        behavior = build_set_test_sources()

        self.assertIn("for (index, byte) in bytes.iter().copied().enumerate()", validation)
        self.assertIn("is_windows_drive_prefix(bytes)", validation)
        self.assertIn("is_dot_component", validation)
        self.assertNotIn("Path::new(value)", validation)
        self.assertNotIn(".components()", validation)
        self.assertIn(
            "allocation_free_path_validation_preserves_path_safety_rules", behavior
        )
        benchmark = (
            PRODUCT_BUILD_SET.parent / "build_set/performance_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "TOOLING15_BUILD_SET_SINGLE_PASS_RELATIVE_PATH_BENCH_V1", benchmark
        )
        self.assertIn("candidate P50 did not improve by at least 20%", benchmark)
        self.assertIn("candidate P95 did not improve by at least 20%", benchmark)
        self.assertIn(
            "TOOLING15_BUILD_SET_DIRECT_PATHBUF_BENCH_V1", benchmark
        )

    def test_build_set_reads_manifest_from_the_retained_locked_handle(self) -> None:
        build_set = PRODUCT_BUILD_SET.read_text(encoding="utf-8")
        opening = source_between(
            build_set,
            "impl ValidatedBuildSet",
            "fn validate_manifest_authority",
        )
        bounded_read = source_between(
            build_set,
            "fn read_bounded_file(",
            "#[cfg(windows)]\nfn is_reparse_or_symlink",
        )
        behavior = build_set_test_sources()
        benchmark = (
            PRODUCT_BUILD_SET.parent / "build_set/performance_tests.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("let mut manifest_file", opening)
        self.assertIn("read_bounded_file(\n            &mut manifest_file", opening)
        self.assertIn("file: &mut File", bounded_read)
        self.assertNotIn("file.try_clone()", bounded_read)
        self.assertIn("bounded_manifest_read_retains_the_locked_handle", behavior)
        self.assertIn(
            "TOOLING15_BUILD_SET_HANDLE_CLONE_ELISION_BENCH_V1", benchmark
        )

    def test_build_set_reuses_one_hash_buffer_across_snapshot_files(self) -> None:
        build_set = PRODUCT_BUILD_SET.read_text(encoding="utf-8")
        opening = source_between(
            build_set,
            "impl ValidatedBuildSet",
            "fn validate_manifest_authority",
        )
        verification = source_between(
            build_set,
            "fn verify_file_content(",
            "fn capture_prefix(",
        )
        behavior = build_set_test_sources()
        benchmark = (
            PRODUCT_BUILD_SET.parent / "build_set/performance_tests.rs"
        ).read_text(encoding="utf-8")

        self.assertIn(
            "let mut hash_buffer = [0_u8; BUILD_SET_HASH_BUFFER_BYTES]", opening
        )
        self.assertIn("&mut hash_buffer", opening)
        self.assertIn(
            "buffer: &mut [u8; BUILD_SET_HASH_BUFFER_BYTES]", verification
        )
        self.assertNotIn("let mut buffer =", verification)
        self.assertIn(
            "shared_hash_buffer_verifies_multiple_snapshot_files", behavior
        )
        self.assertIn(
            "TOOLING15_BUILD_SET_HASH_BUFFER_REUSE_BENCH_V1", benchmark
        )

    def test_product_build_batch_elides_the_redundant_terminal_inventory_walk(self) -> None:
        product_batch = PRODUCT_BUILD_BATCH.read_text(encoding="utf-8")
        batch_build = source_between(
            product_batch,
            "pub fn build_product_receipt_draft_batch(",
            "fn validate_build_batch_request(",
        )
        benchmark = (
            PRODUCT_BUILD_SET.parent / "build_set/performance_tests.rs"
        ).read_text(encoding="utf-8")

        self.assertNotIn("build_set.verify_inventory()?", batch_build)
        self.assertIn("let build_set_id = build_set.build_set_id;", batch_build)
        self.assertIn("build_set_id,", batch_build)
        self.assertNotIn("build_set.build_set_id.clone()", batch_build)
        self.assertIn(
            "TOOLING15_FOUR_PRODUCT_TERMINAL_INVENTORY_ELISION_BENCH_V1",
            benchmark,
        )

    def test_local_cargo_package_reuses_canonical_manifest_path(self) -> None:
        protocol = PRODUCT_BUILD_PROTOCOL.read_text(encoding="utf-8")
        benchmark = (
            PRODUCT_BUILD_PROTOCOL.parent / "cargo_protocol/performance_tests.rs"
        ).read_text(encoding="utf-8")
        canonical_graph = source_between(
            protocol, "fn canonical_cargo_graph_digest", "fn canonical_package_id"
        )
        canonical_package = source_between(
            protocol, "fn canonical_package_id", "fn canonical_snapshot_path"
        )
        package_id_benchmark = source_between(
            benchmark,
            "fn unchanged_package_id_reuse_performance_evidence",
            "fn external_package_index_reuse_performance_evidence",
        )
        package_id_measurement = source_between(
            benchmark,
            "fn measure_package_id_references",
            "fn measure_graph_paths",
        )

        self.assertIn(
            "let (canonical_id, canonical_manifest_path) =", canonical_graph
        )
        self.assertIn("package.manifest_path = canonical_manifest_path;", canonical_graph)
        self.assertEqual(canonical_package.count("canonical_snapshot_path("), 1)
        self.assertNotIn("fn canonical_manifest_path", protocol)
        self.assertIn(
            "TOOLING15_LOCAL_MANIFEST_PATH_REUSE_BENCH_V1", benchmark
        )
        self.assertIn("baseline_package_ids", package_id_benchmark)
        self.assertIn("candidate_package_ids", package_id_benchmark)
        self.assertIn(
            "baseline_package_ids: &HashMap<String, String>", package_id_measurement
        )
        self.assertIn(
            "candidate_package_ids: &HashMap<String, Option<String>>",
            package_id_measurement,
        )

    def test_build_owner_runs_bounded_cargo_and_issues_from_selected_outputs(self) -> None:
        cli_run = PRODUCT_RECEIPT_CLI_RUN.read_text(encoding="utf-8")
        build = PRODUCT_BUILD.read_text(encoding="utf-8")
        capture = PRODUCT_BUILD_CAPTURE.read_text(encoding="utf-8")
        protocol = PRODUCT_BUILD_PROTOCOL.read_text(encoding="utf-8")
        build_set = PRODUCT_BUILD_SET.read_text(encoding="utf-8")
        environment = PRODUCT_BUILD_ENVIRONMENT.read_text(encoding="utf-8")
        batch = PRODUCT_BUILD_BATCH.read_text(encoding="utf-8")
        receipt_batch = (RECEIPT / "product_receipt_batch.rs").read_text(
            encoding="utf-8"
        )
        materialization = (RECEIPT / "materialization.rs").read_text(encoding="utf-8")
        draft = (RECEIPT / "product_receipt_draft.rs").read_text(encoding="utf-8")
        build_entry = source_between(
            cli_run,
            "fn build_product_receipt_draft_file",
            "fn issue_product_receipt_draft",
        )
        batch_build_entry = source_between(
            cli_run,
            "fn build_product_receipt_draft_batch_file",
            "fn issue_product_receipt_draft(",
        )
        draft_issue_entry = source_between(
            cli_run,
            "fn issue_product_receipt_draft",
            "fn issue_product_receipt(",
        )
        batch_issue_entry = source_between(
            cli_run,
            "fn issue_product_receipt_draft_batch(",
            "fn issue_product_receipt(",
        )

        self.assertIn('"build" => {', cli_run)
        self.assertIn("build_product_receipt_draft_file(parse_product_receipt_build_options", cli_run)
        self.assertIn('"issue-draft" => {', cli_run)
        self.assertIn("issue_product_receipt_draft(parse_product_receipt_draft_issue_options", cli_run)
        self.assertIn("build_product_receipt_draft", cli_run)
        self.assertIn(".env_clear()", build)
        self.assertIn("CARGO_METADATA_OUTPUT_LIMIT", build)
        self.assertIn("mod cargo_protocol;", build)
        self.assertIn("mod build_set;", build)
        self.assertIn("mod environment;", build)
        self.assertIn("mod batch;", build)
        request_declaration = source_between(
            build,
            "pub struct ProductBuildRequest",
            "pub struct CargoProductArtifact",
        )
        self.assertIn("build_set_manifest_path: PathBuf", request_declaration)
        self.assertIn("environment_policy: String", request_declaration)
        self.assertNotIn("environment_allowlist", request_declaration)
        self.assertNotIn("build_set_id", request_declaration)
        self.assertNotIn("snapshot_root", request_declaration)
        self.assertIn("ValidatedBuildSet::open", build)
        self.assertIn("build_set.verify_inventory()?", build)
        public_build = source_between(
            build,
            "pub fn build_product_receipt_draft(",
            "pub(super) fn build_product_receipt_draft_in_build_set(",
        )
        self.assertNotIn("build_set.verify_inventory()?", public_build)
        self.assertIn("create_owned_target_directory", build)
        self.assertIn("WINDOWS_MSVC_ENVIRONMENT_NAMES", environment)
        self.assertIn("unknown product build environment policy", environment)
        self.assertIn("Cargo target directory must not already exist", build)
        self.assertEqual(build.count('OsStr::new("--frozen")'), 2)
        self.assertNotIn('"--locked"', build)
        self.assertIn("deny_unknown_fields", build_set)
        self.assertIn("FILE_FLAG_OPEN_REPARSE_POINT", build_set)
        self.assertIn("update_length_framed", build_set)
        self.assertIn("bytes.len() as i64", build_set)
        self.assertIn("expected_inventory: HashSet<String>", build_set)
        self.assertIn(
            "verify_snapshot_inventory(&self.snapshot_root, &self.expected_inventory)",
            build_set,
        )
        inventory_verification = source_between(
            build_set, "fn verify_snapshot_inventory", "fn visit_snapshot_files"
        )
        self.assertIn("expected_inventory.contains(relative)", inventory_verification)
        self.assertNotIn("sort_by", inventory_verification)
        self.assertIn(
            "inventory_snapshot_with_directory_leases(&snapshot_root, manifest.files.len())",
            build_set,
        )
        snapshot_collection = source_between(
            build_set,
            "fn collect_snapshot_inventory",
            "fn verify_snapshot_inventory",
        )
        self.assertIn("let mut inventory = Vec::with_capacity(file_capacity);", snapshot_collection)
        self.assertIn("_locked_directories", build_set)
        self.assertIn("open_locked_directory", build_set)
        self.assertIn("open_locked_directory_with_metadata", build_set)
        self.assertIn("let (directory_lease, metadata)", build_set)
        self.assertNotIn(
            "locked_directories.push(open_locked_directory(&directory)?)", build_set
        )
        self.assertIn("FILE_FLAG_BACKUP_SEMANTICS", build_set)
        self.assertIn("left.is_ascii() && right.is_ascii()", build_set)
        self.assertIn("left.as_bytes().cmp(right.as_bytes())", build_set)
        self.assertIn("let mut prefix = [0_u8; GIT_LFS_PREFIX.len() + 2]", build_set)
        self.assertNotIn("Vec::with_capacity(GIT_LFS_PREFIX.len() + 2)", build_set)
        self.assertIn("update_length_framed_u64(&mut hasher, file.byte_length)", build_set)
        self.assertNotIn("file.byte_length.to_string()", build_set)
        normalized_build_set_path = source_between(
            build_set, "fn snapshot_relative_path", "fn ordinal_compare"
        )
        self.assertIn("String::with_capacity", normalized_build_set_path)
        self.assertIn(".saturating_add(separator_length)", normalized_build_set_path)
        self.assertIn(".saturating_add(file_name.len())", normalized_build_set_path)
        build_set_tests = build_set_test_sources()
        self.assertIn(
            "TOOLING15_BUILD_SET_RELATIVE_PATH_BUFFER_BENCH_V1",
            build_set_tests,
        )
        manifest_validation = source_between(
            build_set, "fn validate_manifest_files", "fn verify_file_content"
        )
        self.assertIn("validate_relative_path(&file.relative_path)?", manifest_validation)
        self.assertIn("open_verified_snapshot_file", build_set)
        self.assertIn("open_locked_file_with_metadata", build_set)
        self.assertIn("initial_metadata: &fs::Metadata", build_set)
        self.assertNotIn("SeekFrom::Start(0)", build_set)
        self.assertIn("read_bounded_message_line", protocol)
        self.assertIn("select_build_artifacts", protocol)
        self.assertIn("struct CargoMessageHeader<'a>", protocol)
        self.assertIn("reason: &'a str", protocol)
        self.assertIn("package_id: Option<&'a str>", protocol)
        self.assertIn("is_binary: bool", protocol)
        self.assertIn("deserialize_cargo_target_kind", protocol)
        self.assertNotIn("kind: Vec<&'a str>", protocol)
        self.assertIn("struct CargoArtifactPayload", protocol)
        self.assertIn("parse_cargo_artifact_payload(message_bytes)?", protocol)
        self.assertNotIn("struct CargoMessage {", protocol)
        self.assertIn("HashMap::with_capacity(resolution.runtime_dependencies.len())", protocol)
        self.assertIn("reachable_packages", protocol)
        self.assertIn("CARGO_METADATA_RESOLVE_EDGE_LIMIT", protocol)
        self.assertIn("VecDeque::with_capacity(nodes.len())", protocol)
        self.assertNotIn("sha256_bytes(&metadata_bytes)", build)
        self.assertIn("canonical_cargo_graph_digest", protocol)
        self.assertIn("canonical_snapshot_path", protocol)
        self.assertIn("cargo_graph_digest", protocol)
        canonical_graph = source_between(
            protocol, "fn canonical_cargo_graph_digest", "fn canonical_package_id"
        )
        canonical_path = source_between(
            protocol, "fn canonical_relative_path", "fn canonicalize_package_ids"
        )
        self.assertIn(
            "std::mem::replace(&mut package.id, canonical_id)", canonical_graph
        )
        self.assertNotIn("let raw_id = package.id.clone()", canonical_graph)
        self.assertNotIn("map(Path::to_path_buf)", canonical_graph)
        self.assertIn(
            "String::with_capacity(path.as_os_str().len())", canonical_path
        )
        cargo_protocol_bench = (
            PRODUCT_BUILD_PROTOCOL.parent / "cargo_protocol/performance_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "TOOLING15_CARGO_GRAPH_PATH_BUFFER_BENCH_V1", cargo_protocol_bench
        )
        canonical_ids = source_between(
            protocol,
            "fn canonicalize_package_ids",
            "fn reachable_packages",
        )
        self.assertIn("canonicalize_package_id_reference_in_place", canonical_ids)
        self.assertIn("HashMap<String, Option<String>>", canonical_ids)
        self.assertIn("Some(canonical)", canonical_ids)
        self.assertIn("canonical.as_str() != id.as_str()", canonical_ids)
        self.assertIn("id.clone_from(canonical)", canonical_ids)
        self.assertNotIn(
            "*id = canonical_package_id_reference(id, package_ids)?", canonical_ids
        )
        self.assertIn(
            "TOOLING15_UNCHANGED_PACKAGE_ID_REUSE_BENCH_V1", cargo_protocol_bench
        )
        self.assertIn(
            "TOOLING15_EXTERNAL_PACKAGE_INDEX_REUSE_BENCH_V1", cargo_protocol_bench
        )
        self.assertIn("Ok((None, PathBuf::from(file_name)))", protocol)
        self.assertIn("package_ids.insert(package.id.clone(), None)", canonical_graph)
        self.assertGreaterEqual(canonical_graph.count("sort_unstable()"), 5)
        self.assertGreaterEqual(canonical_graph.count("sort_unstable_by"), 3)
        self.assertIn("package.targets.sort_unstable_by", canonical_graph)
        self.assertIn("left.crate_types.cmp(&right.crate_types)", canonical_graph)
        self.assertIn(
            "left.required_features.cmp(&right.required_features)", canonical_graph
        )
        self.assertIn("left.edition.cmp(&right.edition)", canonical_graph)
        self.assertIn("left.src_path.cmp(&right.src_path)", canonical_graph)
        self.assertIn("node.deps.sort_unstable_by", canonical_graph)
        self.assertIn("left.dep_kinds.cmp(&right.dep_kinds)", canonical_graph)
        self.assertIn(
            "cargo_graph_digest_is_stable_across_target_order",
            (PRODUCT_BUILD_PROTOCOL.parent / "cargo_protocol/tests.rs").read_text(
                encoding="utf-8"
            ),
        )
        self.assertIn("ids.sort_unstable()", canonical_ids)
        self.assertNotIn("values.sort();", canonical_graph)
        self.assertIn(
            "TOOLING15_UNSTABLE_CARGO_GRAPH_SORT_BENCH_V1", cargo_protocol_bench
        )
        self.assertIn("selected_package_index", protocol)
        self.assertIn("HashSet::with_capacity(runtime_dependencies.len().saturating_add(1))", protocol)
        self.assertIn("if reachable.insert(dependency.as_str())", protocol)
        self.assertNotIn("fn select_package", protocol)
        product_artifact = source_between(
            protocol,
            "fn product_artifact(",
            "fn require_successful_finish(",
        )
        self.assertEqual(product_artifact.count("for path in payload.filenames"), 1)
        self.assertIn("let mut executable_found = false;", product_artifact)
        self.assertNotIn("payload.filenames.iter().any", product_artifact)
        self.assertIn("symbol_files.sort_unstable();", product_artifact)
        self.assertNotIn("symbol_files.sort();", product_artifact)
        self.assertIn(
            "selected_artifact_fuses_executable_check_and_symbol_collection",
            (PRODUCT_BUILD_PROTOCOL.parent / "cargo_protocol/tests.rs").read_text(
                encoding="utf-8"
            ),
        )
        self.assertIn(
            "TOOLING15_FUSED_CARGO_ARTIFACT_SELECTION_BENCH_V1",
            cargo_protocol_bench,
        )
        self.assertIn(
            "candidate P50 did not improve by at least 10%", cargo_protocol_bench
        )
        self.assertIn(
            "candidate P95 did not improve by at least 10%", cargo_protocol_bench
        )
        self.assertIn("open_locked_source", capture)
        self.assertIn("PreparedProductBuildToolchain::open", build)
        self.assertIn("sdk_files: Vec<ProductBuildSdkSource>", build)
        self.assertIn("capture_sdk_fingerprint", capture)
        self.assertNotIn("pub sdk_fingerprint", build)
        self.assertNotIn("Command::output", build)
        runtime_declaration = source_between(
            build,
            "pub struct CargoRuntimeDependencyDeclaration",
            "pub struct ProductBuildRequest",
        )
        self.assertIn("artifact_file_name: String", runtime_declaration)
        self.assertNotIn("source_path", runtime_declaration)
        producer_declaration = source_between(
            build,
            "pub struct ProductBuildProducer",
            "pub struct CargoRuntimeDependencyDeclaration",
        )
        self.assertIn("worker_id: String", producer_declaration)
        self.assertIn("operation_id: String", producer_declaration)
        self.assertNotIn("tool:", producer_declaration)
        self.assertNotIn("tool_version:", producer_declaration)
        self.assertIn('tool: "cargo-zircon".to_string()', build)
        self.assertIn('tool_version: env!("CARGO_PKG_VERSION").to_string()', build)
        self.assertIn("draft.write_new_with_handoff_sha256(&options.output)?", build_entry)
        self.assertNotIn("let handoff_sha256 = draft.handoff_sha256()?", build_entry)
        self.assertIn(
            "batch.write_new_with_handoff_sha256(&options.output)?", batch_build_entry
        )
        self.assertNotIn("batch.handoff_sha256()?", batch_build_entry)
        self.assertNotIn("batch.write_new(&options.output)?", batch_build_entry)
        self.assertNotIn("private_key", build_entry)
        self.assertNotIn("signer", build_entry)
        self.assertNotIn("trust_registry", build_entry)
        self.assertIn("ProductReceiptTrustRegistry::from_json", draft_issue_entry)
        self.assertIn(
            "ProductReceiptDraft::parse_and_verify_handoff_sha256(",
            draft_issue_entry,
        )
        self.assertIn("&draft_bytes", draft_issue_entry)
        self.assertIn(
            "ProductBuildDraftBatch::parse_and_verify_handoff_sha256(",
            batch_issue_entry,
        )
        self.assertIn("&draft_bytes", batch_issue_entry)
        self.assertNotIn(
            "draft_batch.verify_handoff_sha256_owned(", batch_issue_entry
        )
        self.assertIn(
            "draft.issue_verified(options.created_utc, &signer, &registry)?",
            draft_issue_entry,
        )
        self.assertIn("publication.write_new(&options.output)?", draft_issue_entry)
        self.assertIn("build-owner handoff digest", draft)
        self.assertIn("build_product_receipt_draft_batch", batch)
        self.assertEqual(batch.count("ValidatedBuildSet::open"), 1)
        self.assertIn("for build in request.builds", batch)
        self.assertIn("build_product_receipt_draft_in_build_set", batch)
        self.assertIn("product build batch contains a duplicate build action", batch)
        self.assertIn("product build batch must use one BuildSet manifest", batch)
        self.assertIn("let artifact_count =", batch)
        self.assertGreaterEqual(
            batch.count("HashSet::with_capacity(artifact_count)"), 2
        )
        self.assertIn("ProductReceiptBatch::issue", batch)
        self.assertIn("canonical_build_action_key", batch)
        self.assertIn("verify_handoff_sha256", batch)
        combined_batch_write = source_between(
            batch,
            "pub fn write_new_with_handoff_sha256",
            "pub fn verify_handoff_sha256",
        )
        self.assertEqual(combined_batch_write.count("self.validate_shape()?"), 1)
        self.assertIn(
            "receipt_writer::write_new_canonical_json_with_sha256",
            combined_batch_write,
        )
        self.assertNotIn("sha256_serialized", combined_batch_write)
        self.assertNotIn("receipt_writer::write_new_json", combined_batch_write)
        self.assertNotIn("self.handoff_sha256()", combined_batch_write)
        writer = (RECEIPT / "receipt_writer.rs").read_text(encoding="utf-8")
        canonical_batch_writer = source_between(
            writer,
            "pub(crate) fn write_new_canonical_json_with_sha256",
            "fn write_new_json_with",
        )
        self.assertIn("write_canonical_json_with_sha256", canonical_batch_writer)
        self.assertIn("file.sync_all()?", canonical_batch_writer)
        self.assertIn("struct Sha256Writer", writer)
        self.assertIn("serde_json::to_writer(&mut writer, value)", writer)
        self.assertIn(
            "draft_batch_write_returns_the_validated_handoff_digest",
            (PRODUCT_BUILD_BATCH.parent / "batch/tests.rs").read_text(
                encoding="utf-8"
            ),
        )
        batch_write_benchmark = (
            PRODUCT_BUILD_BATCH.parent / "batch/performance_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "TOOLING15_SINGLE_VALIDATION_DRAFT_BATCH_WRITE_BENCH_V1",
            batch_write_benchmark,
        )
        self.assertIn(
            "candidate P50 did not improve by at least 35%", batch_write_benchmark
        )
        self.assertIn(
            "TOOLING15_RAW_DRAFT_HANDOFF_VERIFICATION_BENCH_V1",
            batch_write_benchmark,
        )
        receipt_benchmark = (
            RECEIPT / "product_receipt/performance_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "TOOLING15_SINGLE_RAW_DRAFT_HANDOFF_VERIFICATION_BENCH_V1",
            receipt_benchmark,
        )
        self.assertIn(
            "candidate P50 did not improve by at least 30%", batch_write_benchmark
        )
        self.assertIn(
            "candidate P95 did not improve by at least 30%", batch_write_benchmark
        )
        self.assertIn(
            "candidate P95 did not improve by at least 35%", batch_write_benchmark
        )
        self.assertIn("canonical_receipt_batch_sha256", receipt_batch)
        batch_issue = source_between(
            receipt_batch, "    pub fn issue(", "    pub fn verify_integrity"
        )
        self.assertNotIn("batch.verify_integrity()?", batch_issue)
        self.assertIn("let artifact_count =", receipt_batch)
        self.assertGreaterEqual(
            receipt_batch.count("HashSet::with_capacity(artifact_count)"), 2
        )
        self.assertIn("duplicate canonical build action", receipt_batch)
        self.assertIn("verify_batch_attestation_after_integrity", receipt_batch)
        self.assertIn("declared receipt set", receipt_batch)
        self.assertIn("materialization::verify_receipts(&self.receipts", receipt_batch)
        self.assertNotIn("self.receipts.iter().collect::<Vec<_>>()", receipt_batch)
        self.assertIn("-> impl Iterator<Item = &ReceiptArtifact>", materialization)
        self.assertIn(".saturating_add(separator_length)", materialization)
        self.assertIn(".saturating_add(file_name.len())", materialization)
        self.assertNotIn("let mut components = Vec::new()", materialization)
        self.assertIn("collections::HashSet", materialization)
        self.assertIn("expected_paths.remove(relative.as_str())", materialization)
        self.assertIn("expected_directories.remove(relative.as_str())", materialization)
        materialization_verify = source_between(
            materialization,
            "fn verify_windows_materialization",
            "fn open_locked_absolute_directory_chain",
        )
        self.assertIn("HashSet::with_capacity(artifact_count)", materialization_verify)
        self.assertIn("receipt.build_products.len()", materialization_verify)
        self.assertIn("receipt.runtime_dependencies.len()", materialization_verify)
        self.assertIn("receipt.symbols.len()", materialization_verify)
        self.assertNotIn("receipt_artifacts(receipt).count()", materialization_verify)
        self.assertGreaterEqual(
            materialization_verify.count(
                "receipts.iter().flat_map(receipt_artifacts)"
            ),
            2,
        )
        self.assertNotIn("collect::<Vec<_>>()", materialization_verify)
        self.assertNotIn("collections::BTreeSet", materialization)
        self.assertNotIn("Result<(BTreeSet<String>, BTreeSet<String>)", materialization)
        self.assertNotIn("path.display().to_string()", materialization)
        materialization_bench = (
            RECEIPT / "materialization/performance_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "TOOLING15_MATERIALIZATION_CONSUMING_INVENTORY_BENCH_V1",
            materialization_bench,
        )
        batch_bench = (
            RECEIPT / "product_receipt_batch/performance_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "TOOLING15_BATCH_ISSUE_SINGLE_VALIDATION_BENCH_V1", batch_bench
        )
        self.assertIn('"build-batch" => {', cli_run)
        self.assertIn('"issue-draft-batch" =>', cli_run)
        self.assertIn('"verify-batch" => {', cli_run)

    def test_product_build_borrows_snapshot_root_for_cargo_invocation(self) -> None:
        build = PRODUCT_BUILD.read_text(encoding="utf-8")
        entry = source_between(
            build,
            "pub(super) fn build_product_receipt_draft_in_build_set",
            "fn create_owned_target_directory",
        )

        self.assertIn("let snapshot_root = build_set.snapshot_root.as_path();", entry)
        self.assertNotIn("build_set.snapshot_root.clone()", entry)

    def test_product_build_borrows_binary_name_during_cargo_resolution(self) -> None:
        build = PRODUCT_BUILD.read_text(encoding="utf-8")
        protocol = PRODUCT_BUILD_PROTOCOL.read_text(encoding="utf-8")
        entry = source_between(
            build,
            "pub(super) fn build_product_receipt_draft_in_build_set",
            "pub fn select_cargo_product_artifact",
        )
        resolution = source_between(
            protocol,
            "pub(super) struct CargoBuildResolution",
            "pub(super) struct ResolvedRuntimeDependency",
        )
        resolve_build = source_between(
            protocol,
            "pub(super) fn resolve_build",
            "fn canonical_cargo_graph_digest",
        )

        self.assertIn("request.action.bin.as_deref()", entry)
        self.assertNotIn("request.action.bin.clone()", entry)
        self.assertIn("struct CargoBuildResolution<'a>", resolution)
        self.assertIn("product_binary: &'a str", resolution)
        self.assertIn("product_binary,", resolve_build)
        self.assertNotIn("product_binary: product_binary.to_string()", resolve_build)

    def test_product_build_moves_product_name_after_symbol_capture(self) -> None:
        build = PRODUCT_BUILD.read_text(encoding="utf-8")
        entry = source_between(
            build,
            "pub(super) fn build_product_receipt_draft_in_build_set",
            "pub fn select_cargo_product_artifact",
        )

        symbols = entry.index("let symbols = capture_symbol_artifacts")
        product = entry.index("let build_product = ReceiptArtifact::capture_from_file")
        self.assertLess(symbols, product)
        self.assertIn("request.product.logical_name,", entry)
        self.assertNotIn("request.product.logical_name.clone()", entry)

    def test_cargo_runtime_capture_borrows_dependency_declaration(self) -> None:
        build = PRODUCT_BUILD.read_text(encoding="utf-8")
        capture = PRODUCT_BUILD_CAPTURE.read_text(encoding="utf-8")
        protocol = PRODUCT_BUILD_PROTOCOL.read_text(encoding="utf-8")
        benchmark = (
            PRODUCT_BUILD_PROTOCOL.parent / "cargo_protocol/performance_tests.rs"
        ).read_text(encoding="utf-8")
        runtime_artifact = source_between(
            build,
            "pub(super) struct CargoRuntimeArtifact",
            "pub fn build_product_receipt_draft",
        )
        selection = source_between(
            protocol,
            "pub(super) fn select_build_artifacts",
            "fn selected_package_index",
        )
        opening = source_between(
            capture,
            "pub(super) fn open_cargo_runtime_dependency",
            "fn canonical_build_output",
        )

        self.assertIn("struct CargoRuntimeArtifact<'a>", runtime_artifact)
        self.assertIn(
            "declaration: &'a CargoRuntimeDependencyDeclaration", runtime_artifact
        )
        self.assertIn("declaration,", selection)
        self.assertNotIn("declaration: declaration.clone()", selection)
        self.assertIn("artifact.declaration.logical_name.clone()", opening)
        self.assertIn("artifact.declaration.relative_path.clone()", opening)
        marker = "TOOLING15_BORROWED_RUNTIME_DECLARATION_BENCH_V1"
        self.assertIn(marker, benchmark)
        self.assertIn("candidate P50 did not improve by at least 25%", benchmark)
        self.assertIn("candidate P95 did not improve by at least 25%", benchmark)

    def test_bounded_cargo_output_uses_capped_initial_buffer(self) -> None:
        build = PRODUCT_BUILD.read_text(encoding="utf-8")
        benchmark = PRODUCT_BUILD_PERFORMANCE.read_text(encoding="utf-8")
        bounded_read = source_between(
            build,
            "fn run_bounded_cargo_output(",
            "fn spawn_cargo(",
        )

        self.assertIn("bounded_output_buffer(limit)", bounded_read)
        self.assertIn(".take(limit as u64 + 1)", bounded_read)
        self.assertIn("const CARGO_OUTPUT_INITIAL_CAPACITY: usize = 1024 * 1024", build)
        self.assertIn("fn bounded_output_buffer(limit: usize) -> Vec<u8>", build)
        self.assertIn(
            "Vec::with_capacity(limit.min(CARGO_OUTPUT_INITIAL_CAPACITY))", build
        )
        marker = "TOOLING15_BOUNDED_CARGO_OUTPUT_BUFFER_BENCH_V1"
        self.assertIn(marker, benchmark)
        self.assertIn("candidate P50 did not improve by at least 10%", benchmark)
        self.assertIn("candidate P95 did not improve by at least 10%", benchmark)

    def test_sdk_fingerprint_digests_files_without_transient_artifacts(self) -> None:
        build = PRODUCT_BUILD_CAPTURE.read_text(encoding="utf-8")
        capture = source_between(
            build,
            "pub(super) fn capture_sdk_fingerprint",
            "pub(super) fn open_declared_artifact",
        )

        self.assertIn(
            "let digest = digest_open_file_handle_with_buffer(&mut source.file, digest_buffer)?;",
            capture,
        )
        self.assertIn("sources: &mut [OpenedSdkSource]", capture)
        self.assertIn("sha256: digest.sha256", capture)
        self.assertIn("byte_length: digest.byte_length", capture)
        self.assertNotIn("ReceiptArtifact::capture_from_file", capture)
        self.assertNotIn("source.logical_name.clone()", capture)
        self.assertNotIn('format!("sdk/{}", source.logical_name)', capture)

    def test_cargo_argument_vectors_borrow_stable_values(self) -> None:
        build = PRODUCT_BUILD.read_text(encoding="utf-8")
        arguments = source_between(
            build,
            "fn metadata_arguments",
            "fn run_bounded_cargo_output",
        )

        self.assertIn("Vec<Cow<'a, OsStr>>", arguments)
        self.assertIn("fn push_features<'a>", arguments)
        self.assertIn("Cow::Owned(features.join(\",\").into())", arguments)
        self.assertNotIn("manifest_path.as_os_str().to_owned()", arguments)
        self.assertNotIn("request.target.target_triple.clone().into()", arguments)
        self.assertNotIn("request.action.package.clone().into()", arguments)
        self.assertNotIn("request.target.cargo_profile.clone().into()", arguments)
        self.assertNotIn("target_directory.as_os_str().to_owned()", arguments)

    def test_borrowed_cargo_arguments_have_release_performance_evidence(self) -> None:
        benchmark = PRODUCT_BUILD_PERFORMANCE.read_text(encoding="utf-8")

        self.assertIn("TOOLING15_BORROWED_CARGO_ARGUMENTS_BENCH_V1", benchmark)
        self.assertIn("legacy_build_arguments", benchmark)
        self.assertIn("super::build_arguments", benchmark)
        self.assertEqual(
            benchmark.count("candidate P50 did not improve by at least 20%"), 1
        )
        self.assertEqual(
            benchmark.count("candidate P95 did not improve by at least 20%"), 1
        )

    def test_receipt_batch_identity_streams_ids_without_a_pointer_vector(self) -> None:
        canonical = (RECEIPT / "canonical.rs").read_text(encoding="utf-8")
        batch_identity = source_between(
            canonical,
            "struct CanonicalReceiptBatch",
            "pub(crate) fn canonical_build_action_key",
        )

        self.assertIn("struct ReceiptIds<'a>(&'a [ProductReceipt]);", canonical)
        self.assertIn("receipt_ids: ReceiptIds<'a>", batch_identity)
        self.assertIn("receipt_ids: ReceiptIds(receipts)", batch_identity)
        self.assertIn("serializer.serialize_seq(Some(self.0.len()))?", canonical)
        self.assertNotIn(".collect()", batch_identity)

    def test_canonical_build_action_borrows_normalized_features(self) -> None:
        canonical = (RECEIPT / "canonical.rs").read_text(encoding="utf-8")
        action_key = source_between(
            canonical,
            "pub(crate) fn canonical_build_action_key",
            "#[cfg(test)]\npub(crate) fn canonical_receipt_batch_sha256_with_collected_ids",
        )

        self.assertIn("use std::borrow::Cow;", canonical)
        self.assertIn("features: Cow<'a, [String]>", canonical)
        self.assertIn("Cow::Borrowed(action.features.as_slice())", action_key)
        self.assertIn("Cow::Owned(features)", action_key)
        self.assertIn("pair[0] <= pair[1]", action_key)
        self.assertNotIn(".map(String::as_str)", action_key)
        self.assertNotIn("collect::<Vec", action_key)

    def test_m1_build_set_toolchain_and_shared_product_acceptance_has_behavior_coverage(
        self,
    ) -> None:
        build_set_tests = build_set_test_sources()
        batch_tests = (PRODUCT_BUILD_BATCH.parent / "batch/tests.rs").read_text(
            encoding="utf-8"
        )
        toolchain_tests = (RECEIPT / "toolchain_set/tests.rs").read_text(
            encoding="utf-8"
        )
        product_build_owner_tests = "\n".join(
            path.read_text(encoding="utf-8")
            for path in (
                ROOT / "tools/cargo-zircon/tests/product_build_owner.rs",
                ROOT / "tools/cargo-zircon/tests/product_build_owner/batch.rs",
            )
        )
        build_set_pester = (
            ROOT / "tools/tests/mvp-build-set.Tests.ps1"
        ).read_text(encoding="utf-8")

        self.assertIn(
            "locks_the_snapshot_namespace_against_absent_input_and_a_b_a_mutation",
            build_set_tests,
        )
        self.assertIn(
            "retains captured bytes when the active source returns from B to HEAD A",
            build_set_pester,
        )
        self.assertIn("rejects a Git submodule", build_set_pester)
        self.assertIn("rejects an unmaterialized Git LFS pointer", build_set_pester)
        self.assertIn("excluding untracked files", build_set_pester)
        self.assertIn(
            "every_component_change_derives_a_distinct_toolchain_set_identity",
            toolchain_tests,
        )
        self.assertIn(
            "component_mutation_rejects_a_stale_toolchain_set_identity",
            toolchain_tests,
        )
        self.assertIn(
            "four_product_batch_accepts_unique_actions_on_one_build_set",
            batch_tests,
        )
        self.assertIn(
            "batch_rejects_a_duplicate_build_action_before_running_cargo",
            batch_tests,
        )
        self.assertIn(
            "public_four_product_batch_builds_each_action_once",
            product_build_owner_tests,
        )
        for product, binary, feature in (
            ("runtime", "zircon_runtime", "target-client"),
            ("editor", "zircon_editor", "target-editor-host"),
            ("hub", "zircon_hub", "target-hub"),
            ("workbench", "zircon_workbench", "target-workbench"),
        ):
            self.assertIn(
                f'("{product}", "{binary}", "{feature}")',
                product_build_owner_tests,
            )
        self.assertIn('format!("{product}/{bin}.exe")', product_build_owner_tests)
        self.assertIn('"build-batch"', product_build_owner_tests)
        self.assertIn("sha256_bytes(&output_bytes)", product_build_owner_tests)
        self.assertIn("assert_eq!(batch.drafts.len(), 4)", product_build_owner_tests)
        self.assertIn("invocations.lines().count(), 4", product_build_owner_tests)
        self.assertEqual(
            PRODUCT_BUILD_BATCH.read_text(encoding="utf-8").count(
                "ValidatedBuildSet::open"
            ),
            1,
        )


if __name__ == "__main__":
    unittest.main()
