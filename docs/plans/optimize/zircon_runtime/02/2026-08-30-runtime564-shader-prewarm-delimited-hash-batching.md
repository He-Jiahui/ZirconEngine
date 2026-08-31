---
title: Runtime Shader Prewarm Delimited Hash Batching 564
category: zircon_runtime
report_id: Runtime564-shader-prewarm-delimited-hash-batching-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime Shader Prewarm Delimited Hash Batching 564

Shader prewarm asset-scan revisions frame each include content hash with a zero separator.
The common short-digest path now copies that value and its separator into one 65-byte stack
buffer before updating BLAKE3 once. Values at or above the buffer capacity retain the exact
two-update fallback, so the framing bytes and compatibility behavior are unchanged.

For the normal 64-byte include digests, content revision hashing reduces BLAKE3 update calls
from `2N` to `N` (50%). The base-revision form reduces `2(N + 1)` calls to `N + 1` for the same
digest set. This is a measured operation-count reduction; managed Release validation must supply
the wall-clock result on the coordinator's clean-copy workspace.

## Static evidence

- TDD RED: the delimited batching helper was absent before implementation.
- TDD GREEN: focused tests compare optimized and legacy digest bytes for empty, short, 64-byte,
  65-byte, and long values, including the base-revision form.
- Short values are stack-only; long values use the original update sequence.
- Focused tests use prefix `optimization_batch_20260830ey_runtime564_`.
- Ignored evidence marker: `RUNTIME564_SHADER_PREWARM_DELIMITED_HASH_BENCH_V1`.
- `rustfmt 1.94.1 --edition 2021` and scoped `git diff --check` pass.
- Production/test source SHA-256: `14739d17114afb9ec08750ac8cbabc0f7a8626a2027fe4861a293543dad57eca`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Runtime tests pass.
2. Optimized and legacy revision values remain identical for all tested input lengths.
3. Include hash ordering, zero separators, base revision encoding, and non-zero fallback remain unchanged.
4. Coordinator records the clean-copy wall-clock performance result before commit/push and WeCom publication.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
