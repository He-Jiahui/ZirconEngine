---
related_code:
  - zircon_runtime/src/ui/template/asset/compiler/cache/persistent.rs
  - zircon_runtime/src/ui/template/asset/compiler/cache/mod.rs
  - zircon_runtime/src/ui/template/asset/compiler/mod.rs
  - zircon_runtime/src/ui/template/asset/mod.rs
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/v2/file_cache.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/artifact.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/cache/cache_key.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/package/header.rs
  - tools/zircon_build.py
  - zircon_runtime/src/ui/tests/asset_compile_cache.rs
  - zircon_runtime/src/ui/tests/v2_asset.rs
implementation_files:
  - zircon_runtime/src/ui/template/asset/compiler/cache/persistent.rs
  - zircon_runtime/src/ui/template/asset/compiler/cache/mod.rs
  - zircon_runtime/src/ui/template/asset/compiler/mod.rs
  - zircon_runtime/src/ui/template/asset/mod.rs
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/v2/file_cache.rs
  - tools/zircon_build.py
  - zircon_runtime/src/ui/tests/asset_compile_cache.rs
  - zircon_runtime/src/ui/tests/v2_asset.rs
plan_sources:
  - user: 2026-06-12 implement editor UI architecture plan code
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
tests:
  - zircon_runtime/src/ui/tests/asset_compile_cache.rs
  - zircon_runtime/src/ui/tests/v2_asset.rs
  - rustfmt --edition 2021 --check zircon_runtime\src\ui\template\asset\compiler\cache\persistent.rs zircon_runtime\src\ui\template\asset\compiler\cache\mod.rs zircon_runtime\src\ui\template\asset\compiler\mod.rs zircon_runtime\src\ui\template\asset\mod.rs zircon_runtime\src\ui\template\mod.rs zircon_runtime\src\ui\tests\asset_compile_cache.rs
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-persistent-cache-0612-coremin-long --message-format short --color never (2026-06-12: passed, existing warnings only)
  - cargo test -p zircon_runtime --lib persistent_cache --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-persistent-cache-0612-coremin-long --message-format short --color never -- --test-threads=1 --nocapture (2026-06-12: timed out while compiling/linking runtime lib-test target, no Rust diagnostics returned)
  - rustfmt --edition 2021 --check zircon_runtime\src\ui\template\asset\compiler\cache\persistent.rs zircon_runtime\src\ui\v2\cache.rs zircon_runtime\src\ui\v2\file_cache.rs zircon_runtime\src\ui\tests\v2_asset.rs
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-persistent-cache-0612-coremin-long --message-format short --color never (2026-06-12 M5.S2 v2 persistent file cache: passed, existing warnings only)
  - python inline import of tools/zircon_build.py stage_ui_compiled_artifacts with temp .zuiart/.zuicache inputs (2026-06-12 M5.S3: passed)
doc_type: module-detail
---

# UI Persistent Compiled Artifact Store

`UiCompiledArtifactStore` is the M5 persistent-cache layer for compiled UI assets. It belongs under the existing template asset compiler cache because it stores compiled package artifacts after the compiler has already produced and validated them. It does not replace `UiAssetCompileCache`: the in-memory cache still serves same-process compilation reuse, while this store is the disk-backed surface for cold start, staged build, and loader integration.

## Stored Artifact

The store persists the existing `UiRuntimeCompiledAssetArtifact` binary envelope from `package/artifact.rs`. That artifact already contains:

- the compiled `UiTemplateInstance`;
- the package validation report;
- the compiled asset header;
- the full `UiCompileCacheKey` used by the compiler.

The store wraps those artifact bytes in a small bincode disk record with a store record version and `UiCompiledArtifactKey`. The wrapper is intentionally separate from the artifact envelope so the store can reject stale records before exposing bytes to loader code.

The store also exposes `store_payload_bytes(...)` and `load_payload_bytes(...)` for consumers that cannot use the template artifact envelope. The v2 file-cache cold-start path uses this generic payload lane because `UiV2CompiledDocument` is a different DTO from the legacy template compiler artifact. Generic payloads still get the same stable path layout, store record version, key equality check, and corrupt-record miss behavior; consumer modules own their inner payload schema and semantic validation.

## Cache Key

`UiCompiledArtifactKey` follows the plan shape:

- `asset_id`: the compiled asset id;
- `fingerprint`: a stable `u64` fingerprint derived from the full `UiCompileCacheKey`;
- `schema_version`: the binary compiled-artifact schema version;
- `compiler_version`: the compiled-asset compiler schema version.

`UiCompiledArtifactKey::from_compile_cache_key(...)` and `UiCompiledArtifactKey::from_artifact(...)` are the normal constructors. They preserve the public M5 key shape while avoiding call-site duplication of the cache-key fingerprint algorithm.

The fingerprint covers root document fingerprint, widget/style import fingerprints, declared import revisions, descriptor registry revision, component contract revision, and resource dependency revision. Any source, import, component contract, resource reference, schema, or compiler version change therefore lands at a different persistent-cache path.

## Path Layout

`artifact_path(key)` and the generic payload path are deterministic and Windows-safe:

```text
<root>/
  schema-<schema_version>/
    compiler-<compiler_version>/
      <fingerprint>/
        <sanitized_asset_id>-<asset_id_hash>.zuiart
        <sanitized_asset_id>-<asset_id_hash>.zuicache
```

The path is only an index. A template artifact file is a cache hit only after the disk record deserializes, the record key equals the requested key, the artifact envelope decodes, and the artifact header matches the requested asset id, schema version, compiler version, package version, and full compile-key fingerprint. A generic payload file is a cache hit only after the wrapper record deserializes and the wrapper key equals the requested key; the caller then validates its own payload schema.

## Failure Policy

Reads are no-panic and cache-oriented:

- missing files return `Ok(None)`;
- corrupt disk records return `Ok(None)`;
- corrupt artifact envelopes return `Ok(None)`;
- key/header mismatches return `Ok(None)`;
- filesystem errors other than not-found are returned as `io::Error`.

Writes are stricter. `store(...)` and `store_bytes(...)` reject artifact bytes that do not match the supplied `UiCompiledArtifactKey`; a stale or unrelated artifact must not be written under a valid key.

## Eviction

`remove(key)` deletes one persistent artifact if present. `evict_asset(asset_id)` recursively scans the store root and removes every `.zuiart` and `.zuicache` record whose embedded key has the same asset id. That mirrors the in-memory compile-cache eviction path used by hot reload, while preserving versioned artifacts for unrelated UI assets.

## Current Boundary

M5.S1 implemented validated template artifact storage. M5.S2 wires the v2 file-cache cold-start path through the generic payload lane.

M5.S3 wires staged-build payload copying in `tools/zircon_build.py`. The script copies compiled cache files from `.zircon/ui/compiled_artifacts` by default, or from `ZIRCON_UI_COMPILED_ARTIFACT_CACHE` when that environment variable is set, into `ZirconEngine/assets/ui/compiled_artifacts`. Only `.zuiart` and `.zuicache` files are staged; unrelated cache notes or temporary files are skipped. The build script does not generate compiled artifacts itself yet, because there is no standalone UI asset compiler CLI in this workspace. Generation remains owned by the runtime compiler/file-cache paths, and staging packages whatever validated cache payloads they have already produced.
