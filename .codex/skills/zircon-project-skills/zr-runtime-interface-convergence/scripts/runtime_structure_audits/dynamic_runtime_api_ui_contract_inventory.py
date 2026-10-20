from __future__ import annotations

from runtime_structure_audits.dynamic_runtime_api_archive_inventory import (
    RUNTIME_09_UI_OUTPUT_ARCHIVE,
    RUNTIME_10_DYNAMIC_API_OUTPUT_ARCHIVE,
    RUNTIME_15_RUNTIME_INDEX_OUTPUT_ARCHIVE,
)


UI_PENDING_GATE_ANCHORS = (
    (
        RUNTIME_15_RUNTIME_INDEX_OUTPUT_ARCHIVE,
        "runtime_10_ui_contract_m2_gate_stays_pending_until_runtime_09_owner_handoff",
    ),
    (
        RUNTIME_10_DYNAMIC_API_OUTPUT_ARCHIVE,
        "runtime_10_ui_contract_m2_gate_stays_pending_until_runtime_09_owner_handoff",
    ),
    (
        "docs/architecture/runtime-interface-convergence.md",
        "Runtime 10 UI Contract M2 Gate",
    ),
    (
        "docs/architecture/runtime-architecture-review-m0.md",
        "Runtime 10 UI Contract M2 Gate",
    ),
    (
        "docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md",
        "cargo test -p zircon_runtime_interface --locked",
    ),
    (
        "docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md",
        "cargo test -p zircon_runtime --lib ui --locked",
    ),
    (
        "docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md",
        "cargo check -p zircon_editor --lib --locked",
    ),
)


UI_CONTRACT_SINGLE_SOURCE_ANCHORS = (
    (
        "zircon_runtime/src/tests/contracts/dynamic_api_session/ui_contract.rs",
        "runtime_10_ui_contract_types_have_single_definition_across_interface_and_runtime",
    ),
    (
        "zircon_runtime/src/tests/contracts/dynamic_api_session/ui_contract.rs",
        "UiBindingCodec",
    ),
    (
        "zircon_runtime/src/tests/contracts/dynamic_api_session/ui_contract.rs",
        "UiAssetSchemaVersionPolicy",
    ),
    (
        RUNTIME_15_RUNTIME_INDEX_OUTPUT_ARCHIVE,
        "runtime_10_m2_1_ui_contract_duplicate_public_types_removed_static_passed_cargo_pending",
    ),
    (
        RUNTIME_10_DYNAMIC_API_OUTPUT_ARCHIVE,
        "runtime_10_m2_1_ui_contract_duplicate_public_types_removed_static_passed_cargo_pending",
    ),
    (
        "docs/architecture/runtime-interface-convergence.md",
        "runtime_10_m2_1_ui_contract_duplicate_public_types_removed_static_passed_cargo_pending",
    ),
    (
        "docs/architecture/runtime-architecture-review-m0.md",
        "runtime_10_m2_1_ui_contract_duplicate_public_types_removed_static_passed_cargo_pending",
    ),
)


UI_V2_CONTRACT_SYNC_ANCHORS = (
    (
        "zircon_runtime/src/tests/contracts/dynamic_api_session/v2_contract.rs",
        "runtime_10_ui_v2_contract_sync_matches_runtime_09_verdict_and_interface_owner",
    ),
    (
        "zircon_runtime_interface/src/tests/ui_v2_contracts.rs",
        "ui_component_api_version_mismatch_is_rejected_with_parse_error",
    ),
    (
        "zircon_runtime_interface/src/ui/template/asset/component_contract/api_version.rs",
        "pub struct UiComponentApiVersion",
    ),
    (
        "zircon_runtime/src/ui/template/asset/component_contract/validation.rs",
        "actual.is_compatible_with(required)",
    ),
    (
        RUNTIME_09_UI_OUTPUT_ARCHIVE,
        "v2-replacement-mainline",
    ),
    (
        "docs/crates/zircon_runtime/ui/architecture.md",
        "v2-replacement-mainline",
    ),
    (
        RUNTIME_15_RUNTIME_INDEX_OUTPUT_ARCHIVE,
        "runtime_10_m2_2_ui_v2_contract_sync_static_passed_cargo_pending",
    ),
    (
        RUNTIME_10_DYNAMIC_API_OUTPUT_ARCHIVE,
        "runtime_10_m2_2_ui_v2_contract_sync_static_passed_cargo_pending",
    ),
    (
        "docs/crates/zircon_runtime_interface/ui/mod.md",
        "runtime_10_m2_2_ui_v2_contract_sync_static_passed_cargo_pending",
    ),
)
