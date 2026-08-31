---
title: Runtime Compiled Capability Short Circuit 547
category: zircon_runtime
report_id: Runtime547-compiled-capability-short-circuit-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime Compiled Capability Short Circuit 547

Compiled render feature flags are monotonic, but every enabled feature previously resolved Hybrid
GI and Virtual Geometry capability requirements even after both flags were already true. Each
capability now continues resolving until its first match, then short-circuits independently for the
remaining features. Lists where a capability appears late or never appears still scan fully.

The ignored Release evidence `RUNTIME547_COMPILED_CAPABILITY_SHORT_CIRCUIT_BENCH_V1` models 65,536
features whose first item declares both capabilities. Capability resolutions fall from 131,072 to
2, a 99.998% reduction. A standalone Rust 1.94.1 `opt-level=3` check over the same list changed from
0.483 ms to 0.079 ms at the 11-sample median, an 83.60% improvement on this machine. Full render
pipeline compilation and feature descriptor construction are outside this elapsed result.

## Static evidence

- TDD RED: both capability resolutions were unconditional inside the feature loop.
- TDD GREEN: Hybrid GI and Virtual Geometry each have an independent monotonic guard.
- Focused behavior proves an ordinary first feature does not prevent a later advanced feature from
  enabling both flags.
- `rustfmt 1.94.1 --edition 2021` and scoped `git diff --check` pass.
- `zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline/runtime_feature_flags.rs`
  SHA-256: `8dda4d4bce7c21263d3635c3fe30926c1f8ee32947da38b1b5486c51ebddca77`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Runtime tests pass.
2. First, late, absent, and repeated capability declarations preserve compiled flags.
3. Ignored evidence emits the Runtime547 marker and reports the 131,072-to-2 resolution gate.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
