---
title: Runtime Registry Persistence Borrow 534
category: zircon_runtime
report_id: Runtime534-registry-persistence-borrow-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime Registry Persistence Borrow 534

Asset-registry persistence previously cloned every sorted `AssetRegistryEntry`, including owned
paths, locators, importer metadata, and dependency vectors, into a temporary persistence document
before JSON serialization. The write path now serializes a borrowed entry DTO while retaining the
existing owned DTO for deserialization. Format version, deterministic path ordering, JSON shape,
atomic publication, and recovery behavior are unchanged.

The ignored Release evidence `RUNTIME534_REGISTRY_PERSISTENCE_BORROW_BENCH_V1` models 65,536
registry entries. The legacy path performs 65,536 entry deep clones; the borrowed path performs
zero, a 100% reduction. This is an exact ownership-operation model, not elapsed-time or peak-memory
evidence. The sorted vector of entry references remains required for deterministic persistence.

## Static evidence

- TDD RED: `prepare_persistence` used `self.entries().into_iter().cloned().collect()`.
- TDD GREEN: `PersistedAssetRegistryRef<'a>` stores `Vec<&'a AssetRegistryEntry>` and serializes
  the already sorted references directly.
- The owned `PersistedAssetRegistry` remains the only decode DTO.
- `rustfmt 1.94.1 --edition 2024 --check` passes.
- `git diff --check` passes (PowerShell reports only the repository LF/CRLF notice).
- Source SHA-256:
  `888466ebcd6e6631c7aa0d66c3c1609001735b8a895508db2a32eae2492c3845`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Runtime tests pass.
2. Registry persistence and load/rebuild regressions preserve the version-1 JSON contract.
3. The ignored evidence emits the Runtime534 marker with zero optimized entry deep clones.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
