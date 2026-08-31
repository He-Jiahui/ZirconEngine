---
related_code:
  - zircon_runtime/src/dynamic_api/bounded_json.rs
  - zircon_runtime/src/dynamic_api/bounded_json/error.rs
  - zircon_runtime/src/dynamic_api/bounded_json/deadline.rs
  - zircon_runtime/src/dynamic_api/bounded_json/preflight.rs
  - zircon_runtime/src/dynamic_api/bounded_json/writer.rs
  - zircon_runtime/src/dynamic_api/bounded_json/tests.rs
implementation_files:
  - zircon_runtime/src/dynamic_api/bounded_json.rs
  - zircon_runtime/src/dynamic_api/bounded_json/error.rs
  - zircon_runtime/src/dynamic_api/bounded_json/deadline.rs
  - zircon_runtime/src/dynamic_api/bounded_json/preflight.rs
  - zircon_runtime/src/dynamic_api/bounded_json/writer.rs
  - zircon_runtime/src/dynamic_api/bounded_json/tests.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/optimize/zircon_runtime/43-dynamic-runtime-session-registry-ffi-frame-event-extract-host-request-world-sync-ui-shader-prewarm-product-integration-review.md
  - docs/plans/optimize/zircon_runtime_interface/09-runtime-host-foreign-output-safe-owner-admission-budget-fuse-observability-current-source-review.md
  - docs/plans/performance/01/2026-08-23-runtime-dynamic-api-root-viewport-json-current-architecture-review.md
tests:
  - zircon_runtime/src/dynamic_api/bounded_json/tests.rs::decode_reports_nesting_as_a_payload_limit
  - zircon_runtime/src/dynamic_api/bounded_json/tests.rs::encode_reports_nesting_as_a_payload_limit
  - zircon_runtime/src/dynamic_api/bounded_json/tests.rs::validate_reports_limits_without_materializing_encoded_bytes
  - zircon_runtime/src/dynamic_api/bounded_json/tests.rs::nesting_tracker_ignores_delimiters_inside_split_escaped_strings
  - zircon_runtime/src/dynamic_api/bounded_json/tests.rs::decode_applies_the_item_limit_to_business_items_not_json_nodes
  - zircon_runtime/src/dynamic_api/bounded_json/tests.rs::bounded_json_facade_keeps_policy_stages_in_child_owners
  - rustfmt --edition 2021 --check
  - git diff --check
doc_type: milestone-detail
---

# Runtime 10 bounded JSON owner split

## Status and completed items

| Milestone | Slice | Status | Date | Evidence |
|---|---|---|---|---|
| M2/M4 | Bounded JSON policy-stage owner split | `runtime_10_bounded_json_owner_split_implemented_static_passed_managed_validation_deferred_algorithm_unchanged` | 2026-08-26 | Root 724 -> 104 lines; four production child owners 59/69/185/218 lines; 48/48 functions, 10/10 types, and all safety-pass counters retained. |

Completed:

- Kept foreign-slice validation and decode/encode/validate stage orchestration in the facade.
- Split the error contract, processing deadline/reader, syntax preflight, and bounded writers into independent private owners.
- Preserved original `dynamic_api` visibility for the three consumed error/writer/count symbols.
- Moved all five existing behavior tests into a test owner and added a facade/owner structure contract.
- Recorded the existing multi-pass algorithm and the profiler evidence required before changing it.

## Review basis

Local Unreal JSON code separates reader, serializer, and writer owners. Existing Zircon Runtime43, Interface09, and performance reviews agree that byte/depth/item/time defenses remain required while the inbound multi-pass path and session-lock placement need measurement-driven redesign. This slice establishes replaceable owners without weakening those defenses.

There is no compatibility module, duplicate implementation, public API expansion, safety-pass removal, algorithm replacement, new allocation, or performance claim.

## Verification

- Scoped `rustfmt --edition 2021 --check` passed for all six touched Rust files.
- Static migration comparison retained all 48 original functions and 10 original types; one source-ownership test was added.
- Production string literals are `20/20` with zero delta.
- The 4 KiB nesting scan, syntax preflight, typed decode, serde encode, deadline, nesting, and typed limit-error occurrence counts match the original file.
- Root/source contracts confirm the four private child mounts, three facade operations, owner-specific types, and a 350-line production budget.
- Production files contain no new `allow`, `unwrap`, `expect`, `panic`, `todo`, or `unimplemented` escape path.
- Managed Cargo, behavior, allocator, WPR/Tracy, and power validation were not run while bypassing the current validation blocker.

## Open scope

Runtime 10 and the full runtime architecture remain `in_progress`. DYN-P1-059/RHOST-P1-073 are not fixed: successful inbound decode still performs nesting scan, syntax preflight, typed deserialize, and business-item traversal. Profiling, single-pass budget-aware decode design, session-lock relocation, managed validation, milestone commit, coordinator integration receipt, and WeCom publication remain open.
