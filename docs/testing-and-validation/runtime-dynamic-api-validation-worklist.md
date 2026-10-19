---
status: in_progress
---

# Runtime Dynamic Api Validation Worklist

## Dynamic API and ownership obligations

Runtime ABI versions are independent of native-plugin and host callback version domains. ZrRuntimeApiV3 / zircon_runtime_get_api_v3 / ZR_RUNTIME_GET_API_SYMBOL_V3 are dated V3 evidence; the later ZrRuntimeApiV8 contract is a separately dated cutover record. Neither archived version number overrides the actual export/table owner or permits two current runtime tables. Runtime entry points contain panic and map wrapper failures to ZrStatusCode::Panic; obtaining a table during panic returns null.

Session destruction rejects missing or stale handles. The empty drain_host_requests fast path returns canonical empty owned bytes; nonempty batches retain schema and ownership/free rules. selected_node must not survive as a second editor selection authority. Shared UI DTO/version negotiation includes UiComponentApiVersion mismatch behavior.

session/events.rs routes keyboard_ime.rs and gamepad.rs without a second event state. Foreign capture_frame copies, nonempty page allocation and PERF-MVP-023 remain independent performance obligations. dynamic_runtime_api_boundary reviews ABI layout and forbidden types; it does not establish atomic cross-product migration.

### Validation owners and provenance

Keep exact session, UI-version, event-child and shared-contract owner identifiers below. The 2026-07-17 empty-batch result and 2026-07-23 ABI review are limited dated evidence, not full-runtime acceptance.

This worklist retains its declared status independently of module policy, the runtime index and archive evidence. No new test/product acceptance is recorded.

### Retained source status identities

| Contract | Responsibility / evidence boundary |
|---|---|
| `runtime_10_12_15_dynamic_event_keyboard_ime_gamepad_owner_split_static_passed_cargo_deferred` | Keyboard/IME and gamepad packaging remain children of the same session. |

### Source and guard ownership

The finite source, guard and provenance inventory for these responsibilities is:

- `docs/crates/zircon_runtime/dynamic_api/session.md`, `zircon_runtime/src/dynamic_api/exports.rs`, `zircon_runtime_interface/src/ui`.
- `zircon_runtime/src/ui`.

### Validation interfaces

Keep these existing validation and domain interfaces in the gate:

- `session_destroy_reports_explicit_not_found_after_headless_destroy`, `ui_component_api_version_mismatch_is_rejected_with_parse_error`, `destroyed_headless_session_entry_points_reject_old_handle`, `destroy_session`.

### Clause provenance

These content identities preserve the source of this transfer. They are not fresh measurements, active-file dependencies or acceptance receipts.

- `10-dynamic-api-and-interface-convergence.md`, SHA-256 `40b5e38edb1a96e9bec303a24747476ec00669a8c412483bbf990e2d81f23dec`, selected source lines 97-271.
