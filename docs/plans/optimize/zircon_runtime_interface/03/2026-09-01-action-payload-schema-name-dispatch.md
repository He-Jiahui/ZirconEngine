record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/template/asset/binding/schema.rs
related_tests:
  - tools/tests/test_runtime_interface03_schema_name_dispatch_performance_contract.py
  - zircon_runtime_interface/src/ui/template/asset/binding/schema.rs::action_payload_schema_name_dispatch_release_benchmark
---

# Direct action-payload schema-name dispatch

## Scope

`UiActionPayloadFieldName::from_schema_name` is used by asset-binding validation and previously
scanned all 23 stable payload fields for each key. Later fields and unknown extension keys paid the
full linear comparison cost during validation.

The decoder now uses an exhaustive allocation-free string `match`. Every stable snake-case schema
name maps to the same field, matching remains case-sensitive, and unknown values still return
`None`. The ordered `ALL` array remains available for iteration and compatibility tests without
serving as the lookup implementation.

## Verification

- TDD RED: the static contract found the old `Self::ALL.into_iter().find(...)` implementation.
- Focused static contract after implementation: `2/2` passed for the paired schema-name batch.
- Full stable-name round trip plus camel-case/unknown rejection: added as a Rust unit test.
- `python -m compileall` for the paired guard: passed.
- Scoped Rust 1.94.1 `rustfmt --check`: passed.
- Scoped `git diff --check`: passed.
- Managed Windows Rust 1.94.1 crate tests and release benchmark: pending asynchronous batched
  coordinator validation; no terminal performance number is claimed yet.

Managed batch submission:

- snapshot: `2651`
- snapshot request: `be9fc9333fb64ace8ba278e1e0ecefe7`
- submit request: `dcde3218798b4cae83a8e31a2b16c943`
- coordinator request: `750e6258df3e454dafd38338aad6c1d3`
- validation ticket: `dc9fdb6dfe834239aa6edcc001624971`
- source manifest: `46bc839a6b51f08a9fd74d6a86e8292931f38c44bc956983e5165b3a29e261fe`
- command: `cargo +1.94.1 test -p zircon_runtime_interface --locked --release --jobs 1 schema_name -- --include-ignored --nocapture`
- submitted state: `queued` (asynchronous; intentionally not polled)

## Performance contract

The ignored release benchmark compares the prior linear scan with direct dispatch over all 23
stable names plus one unknown input, using 200,000 lookups per sample and 11 alternating samples.
The P95 gate requires direct dispatch to be at least 20% faster. Terminal P50/P95 nanosecond
values must come from the managed Windows receipt before integration, push, or WeCom reporting.
