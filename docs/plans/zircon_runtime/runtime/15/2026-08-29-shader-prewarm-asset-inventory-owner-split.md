---
title: Shader Prewarm Asset Inventory Owner Split
doc_type: implementation-record
status: source_complete_validation_pending
implementation_status: complete
validation_status: rustfmt_static_guard_isolated_metadata_green_managed_cargo_pending
owner: Runtime15
related_code:
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/asset_inventory.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/asset_inventory/snapshot.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/asset_inventory/traversal.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/shader_prewarm_asset_inventory.rs
references:
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Private/AssetDataGatherer.h
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Public/AssetRegistry/AssetRegistryState.h
---

# Shader Prewarm Asset Inventory Owner Split

## Decision

The former 781-line asset inventory file mixed three independent responsibilities: bounded asset
payload collection, safe deterministic directory traversal, and persisted warm-snapshot/index
lifecycle. Runtime15 keeps one inventory state and hard-splits only implementation ownership:

- the 224-line root owns inventory state, collection orchestration, payload budget and accessors;
- the 431-line `snapshot.rs` owns schema v4, index/payload validation, changed-path comparison and
  atomic payload-before-index publication;
- the 159-line `traversal.rs` owns sorted directory discovery, excluded-root identity, link/reparse
  rejection and source-kind error mapping.

Unreal's `FAssetDataGatherer` versus `FAssetRegistryState` is the primary owner reference. Zircon
retains its stricter relative-path and reparse-point checks; this split does not copy Unreal's cache
format or asynchronous scheduling model.

## Completed Items

- added a Runtime structure guard with `320 / 500 / 220` root/snapshot/traversal line budgets;
- preserved schema version, serialized fields, payload-before-readiness ordering, scan sorting,
  text budget, path validation and all error variants;
- retained the existing test helper paths through the parent test module;
- avoided a second inventory, cache, traversal algorithm, facade or compatibility route.

## Evidence And Remaining Gate

The guard failed before the child owners existed and passes `1/1` after the split. Rust 1.94.1
isolated metadata compilation of the actual three production files passes using existing F-drive
dependency artifacts. The moved source has `3,784 / 3,784` normalized tokens and SHA-256
`388F004C4302DD2021AA92C865714EF078091F8C9041BC9E0B94E1E39F764DB1`; normalization removes only
module visibility/qualification and formatting-only trailing commas. Scoped Rust formatting,
whitespace and diff checks pass.

Managed Cargo, the existing shader-prewarm behavior suite, product execution and profiling remain
pending behind the shared validation lane. No performance gain, Runtime15 acceptance, milestone
commit or WeCom completion message is claimed.

Status token:
`runtime_15_shader_prewarm_asset_inventory_owner_split_static_metadata_passed_cargo_product_deferred`.
