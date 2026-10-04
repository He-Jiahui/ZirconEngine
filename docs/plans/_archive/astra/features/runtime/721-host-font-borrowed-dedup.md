---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/00-ui-architecture-performance-reassessment-2026-09-02.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md
related_records:
  - docs/plans/astra/features/runtime/37-runtime-ui-static-contract-alignment.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/surface/host_font_assets.rs
tests:
  - zircon_runtime/src/ui/surface/host_font_assets.rs
---

# Runtime host-font borrowed deduplication

Host font admission now deduplicates the borrowed `&str` input slice with
`sort_unstable`/`dedup` before taking ownership. Cache hits therefore avoid the
old per-input `Arc<str>` allocation and temporary `BTreeSet`; a cache miss still
creates one owned `Arc<str>` for the existing runtime loader contract. Sorted
admission order, resident lifetime, readiness reporting, and failure handling
remain unchanged.

## Plan completion list

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Runtime11A / Runtime11B / RUI-721 | Deduplicate host-font references while borrowed and allocate owned path data only on registry miss. | implemented_pending_validation | RED/GREEN source probes, host-font source guard, Rustfmt, and scoped diff checks pass; managed Runtime Cargo and Release font allocation/latency evidence remain pending. |

## Complexity and allocation boundary

Admission remains `O(K log K + U)` for `K` input references and `U` unique
references, with deterministic lexical order. The previous owned-Arc set plus
vector is removed; one borrowed vector is retained for sorting, and misses
allocate only the Arc/String data required by the existing claim and registry.
No font collection authority, generation, or resident teardown semantics change.

## Local evidence

- A RED source probe confirmed the borrowed sort/dedup path was absent; the
  GREEN probe and in-file source guard confirm the new ownership boundary.
- Existing duplicate-load lifecycle coverage remains in the same test module;
  the focused hierarchy/editor contract batch passed `57/57`, and the final
  non-tooling Runtime/Editor batch passed `1320/1320` across 343 modules in
  `94.188s`; the subsequent Runtime723 follow-up reran the same loader and
  passed `1320/1320` in `5.353s`.
- Rustfmt, `git diff --check`, and optimization-record whitespace checks pass.
- The current combined non-tooling Runtime/Editor batch covers 347 modules and
  passes `1331/1331` tests in `6.361s`.
- These checks are source/contract evidence only and do not claim product CPU,
  allocator, RSS, or p50/p95/p99 acceptance.

## Managed acceptance gate

This slice joins the existing asynchronous Runtime/Editor admission recorded in
`696`; no new coordinator request or status query was issued. The row remains
`implemented_pending_validation` until an owner-attributed Windows Release run
measures cold/warm host-font admission allocations and latency.
