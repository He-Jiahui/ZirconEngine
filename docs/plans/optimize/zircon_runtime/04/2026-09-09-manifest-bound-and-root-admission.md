---
title: Runtime04 Manifest Byte Bound and Root Admission
category: zircon_runtime
report_id: Runtime04-manifest-bound-root-admission-2026-09-09
date: 2026-09-09
session_id: astra-optimize-20260909-root
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime04 Manifest Byte Bound and Root Admission

## Scope

This slice hardens the Runtime project-manifest boundary while preserving the v3 wire format and
existing error ordering. In-memory and file-backed loads now enforce the shared byte budget before
TOML/JSON materialization. Asset-root validation uses the interface admission limit and a borrowed
path-boundary index; UI-root uniqueness uses the canonical path without formatting a temporary URI.

## Implementation

- `ProjectManifest::from_toml_str` rejects documents larger than
  `MAX_PROJECT_MANIFEST_BYTES` before parsing.
- `load_with_report` checks file metadata, reads at most limit plus one byte, and reports the
  actual bounded size. Exact-limit UTF-8 and invalid-UTF-8 behavior remain typed.
- `ProjectManifest::validate` rejects more than `MAX_PROJECT_ASSET_ROOTS`, detects duplicate
  roots before overlap, and probes only real slash boundaries while retaining input-order error
  precedence. UI duplicate detection borrows `AssetUri::path()`.
- Template-receipt identity checks remain part of the same validation boundary; save-side byte
  admission continues to protect the existing atomic file.

## Local evidence

- Runtime85 and Astra benchmark-scope contracts passed in one batch: `7/7`.
- The six explicitly invoked Python modules passed `py_compile`; manifest production owners and
  the new validation test owners passed Rustfmt/parse checks. Scoped `git diff --check` passed
  with only the repository's normal LF/CRLF notices.
- The ignored release harness remains a helper microbenchmark and reports nearest-rank p50/p95/p99
  only after managed execution; no product CPU, allocation, RSS, or input-to-present result is
  inferred here.

## Validation boundary

Managed Cargo and release evidence remain pending because the external `E:/Git/zr_vm` checkout is
dirty. No coordinator state was queried or polled in this slice. The current-source fingerprints
are recorded below so a later immutable batch cannot silently use an older manifest contract.

| Owner | SHA-256 |
|---|---|
| `zircon_runtime/src/asset/project/manifest/load.rs` | `3B406D030D29D2DB0E273822CCB1E620A842FEEF147DE061F3D1EE9451EFA841` |
| `zircon_runtime/src/asset/project/manifest/project_manifest.rs` | `862ECCBA6CA7F434D371065C86290F69C6AEFFA613F2AA2B824AEFFDAFEC7464` |
| `zircon_runtime/src/asset/project/manifest/validation.rs` | `1CC8C8552D43D4F2237DF92C70B871DCDDBD6209C1F25417432A1DC5558AE6B3` |
| `zircon_runtime/src/asset/project/manifest/error.rs` | `D394E4AB51BAA584AAF75B89B5055E494CD48B211B9F7A762ABA205FE9C9C727` |
| `zircon_runtime/src/asset/project/manifest/save.rs` | `8A251474650C8ADD87FD927E13F567FBB29E2DE947B59CC6F1C86617DAA85967` |
| `zircon_runtime/src/asset/project/manifest/save/borrowed_serialization_tests.rs` | `A392BD1868FB99E04D77BE594FA18DA7309D2D4A87656DE411786B458A7CB9EF` |
| `zircon_runtime/src/asset/project/manifest/validation/astra_root_tests.rs` | `1D9CEF99AD2AB4AC3DFF1D867331003D10645DBBD59E2D914CCF885B6D474EA1` |
| `zircon_runtime/src/asset/project/manifest/validation/optimization_batch_ir_runtime629_tests.rs` | `C59FA854F39E27F6A7872B4D69DF0C826102D1C78920567E680CD52955914010` |

## Remaining parent-plan work

Runtime04 still owns the broader typed artifact, asynchronous residency, semantic streaming,
signed package, and product-scale asset qualification work. This record only closes the bounded
manifest/root admission slice at implementation level.
