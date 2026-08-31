---
title: Runtime Native Candidate Kind Bitset 546
category: zircon_runtime
report_id: Runtime546-native-candidate-kind-bitset-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime Native Candidate Kind Bitset 546

Native plugin candidate filtering previously called `slice::contains` for every package module and
embedded feature module against the same requested module-kind slice. The loader now converts the
closed Runtime/Editor/Native/Vm request set to a local four-bit value once per load operation and
uses constant-time membership tests while scanning candidates. Per-library entry gating retains its
original library-specific slice after measurement showed rebuilding a bitset there was slower.

The ignored Release evidence `RUNTIME546_NATIVE_CANDIDATE_KIND_BITSET_BENCH_V1` models 65,536
Editor-kind candidates against `[Runtime, Editor]`. Membership work falls from 131,072 enum
comparisons to 65,536 bit tests, a 50% reduction. A standalone Rust 1.94.1 `opt-level=3` check used
eight million mixed candidate checks per sample; the 11-sample median changed from 16.14 ms to
11.19 ms, a 30.66% improvement on this machine. Manifest parsing and library I/O are excluded.

## Static evidence

- TDD RED: the production source had no request bitset and candidate matching retained slice
  membership checks.
- TDD GREEN: one load-level `RequestedModuleKinds` is shared by direct and embedded module scans.
- Focused behavior covers every `PluginModuleKind` bit and the empty request set.
- The rejected per-library entry-gate bitset measured a 99.46% regression and was removed; no
  Runtime547 entry marker or altered entry contract remains.
- `rustfmt 1.94.1 --edition 2021` and scoped `git diff --check` pass.
- `zircon_runtime/src/plugin/native_plugin_loader/load_discovered.rs` SHA-256:
  `39f71656644deefeb9ef5847135fb666c9e108d190b9512621d83f62848a3ee2`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Runtime tests pass.
2. Direct and embedded modules preserve requested-kind selection and per-library entry scoping.
3. Ignored evidence emits the Runtime546 marker and reports the 2-to-1 membership gate.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
