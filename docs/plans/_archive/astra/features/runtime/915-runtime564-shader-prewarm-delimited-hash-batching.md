---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-runtime564-shader-prewarm-delimited-hash-batching.md
related_records:
  - docs/plans/astra/features/runtime/914-runtime563-bake-artifact-hash-header-batch.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/revision.rs
tests:
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/revision.rs
---

# Runtime915 Runtime564 Shader-Prewarm Delimited Hash Batching

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Shader-prewarm revision hashing | Copy short digest values and their zero separator into one 65-byte stack buffer before a single BLAKE3 update; values at or above capacity retain the exact two-update fallback. | Behavior contracts cover empty, short, 64-byte, 65-byte, long, and base-revision forms. |
| 性能门禁 | Normal 64-byte include revisions reduce content hash updates from `2N` to `N`, with the same reduction for base revisions; managed wall-clock evidence remains pending. | marker `RUNTIME564_SHADER_PREWARM_DELIMITED_HASH_BENCH_V1`; managed Release receipt remains pending. |

- `revision.rs` contains the Runtime564 behavior/source contracts and marker.
- The ignored marker harness compares the legacy and batched content/base revision paths over
  alternating samples and reports `legacy_updates_per_iteration=2N` versus
  `optimized_updates_per_iteration=N`; the managed Release wall-clock receipt remains the
  acceptance authority.
- No tooling changes; operation-count evidence is not substituted for managed timing.

### Grouped validation submission (2026-09-25)

Runtime564 is included in the shared `20260830e` batch covering Runtime552–564:
Runtime development PTY `48614`, Editor development PTY `47311`, Runtime02
Release PTY `82681`, and Editor Release PTY `81946`. All wrappers remain
intentionally unpolled and managed compiler/P95 receipts are pending.
