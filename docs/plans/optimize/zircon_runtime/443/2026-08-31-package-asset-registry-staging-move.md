---
title: Runtime443 Package Asset Registry Staging Move
category: zircon_runtime
report_id: Runtime443-package-asset-registry-staging-move-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime443 Package Asset Registry Staging Move

Package asset root registration now temporarily moves the registry out of `ProjectManager`, applies
the validated registration, publishes catalog metadata, and restores the registry. The former path
deep-cloned every existing package id and resolved root before each single-root or package-root
registration.

Current registry validation resolves and validates every fallible input before inserting a root.
Both wrapper error branches restore the staged registry before returning; success branches publish
the same metadata and restore the updated registry. Package resolution, replacement behavior,
catalog generation, and returned errors remain unchanged. Regression coverage requires both methods
to stage with `mem::take`, restore on success and error, and reject restoration of the full clone.

The ignored Windows Release benchmark emits `RUNTIME443_PACKAGE_ASSET_REGISTRY_STAGING_MOVE_BENCH_V1`
over 17 alternating paired samples. Each sample stages a 256-root registry 256 times. The legacy
model copies 65,536 registry entries per sample; the optimized model transfers ownership and copies
none. The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.20`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime443 is prepared with Editor371 under request
`runtime443-editor371-performance-batch-20260831ei-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
