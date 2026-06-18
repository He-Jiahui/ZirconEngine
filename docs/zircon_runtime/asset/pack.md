---
related_code:
  - zircon_runtime/src/asset/pack/mod.rs
  - zircon_runtime/src/asset/pack/manifest.rs
  - zircon_runtime/src/asset/pack/dedup.rs
  - zircon_runtime/src/asset/pack/delta.rs
  - zircon_runtime/src/asset/pack/install/mod.rs
  - zircon_runtime/src/asset/pack/install/error.rs
  - zircon_runtime/src/asset/pack/install/file_io.rs
  - zircon_runtime/src/asset/pack/install/installer.rs
  - zircon_runtime/src/asset/pack/install/promotion.rs
  - zircon_runtime/src/asset/pack/install/promotion_report.rs
  - zircon_runtime/src/asset/pack/install/receipt.rs
  - zircon_runtime/src/asset/pack/install/receipt_io.rs
  - zircon_runtime/src/asset/pack/install/staging.rs
  - zircon_runtime/src/asset/pack/install/staging_report.rs
  - zircon_runtime/src/asset/pack/writer.rs
  - zircon_runtime/src/asset/pack/reader.rs
  - zircon_runtime/src/asset/pack/trim.rs
  - zircon_runtime/src/bin/zircon_export_pack/pack.rs
  - zircon_runtime/src/bin/zircon_export_pack/manifest.rs
  - zircon_runtime/src/bin/zircon_export_pack/run.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/tests/pack.rs
  - zircon_runtime/src/asset/tests/mod.rs
  - zircon_runtime/src/core/framework/net/download.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/hot_update_application.rs
implementation_files:
  - zircon_runtime/src/asset/pack/mod.rs
  - zircon_runtime/src/asset/pack/manifest.rs
  - zircon_runtime/src/asset/pack/dedup.rs
  - zircon_runtime/src/asset/pack/delta.rs
  - zircon_runtime/src/asset/pack/install/mod.rs
  - zircon_runtime/src/asset/pack/install/error.rs
  - zircon_runtime/src/asset/pack/install/file_io.rs
  - zircon_runtime/src/asset/pack/install/installer.rs
  - zircon_runtime/src/asset/pack/install/promotion.rs
  - zircon_runtime/src/asset/pack/install/promotion_report.rs
  - zircon_runtime/src/asset/pack/install/receipt.rs
  - zircon_runtime/src/asset/pack/install/receipt_io.rs
  - zircon_runtime/src/asset/pack/install/staging.rs
  - zircon_runtime/src/asset/pack/install/staging_report.rs
  - zircon_runtime/src/asset/pack/writer.rs
  - zircon_runtime/src/asset/pack/reader.rs
  - zircon_runtime/src/asset/pack/trim.rs
  - zircon_runtime/src/bin/zircon_export_pack/pack.rs
  - zircon_runtime/src/bin/zircon_export_pack/manifest.rs
  - zircon_runtime/src/bin/zircon_export_pack/run.rs
  - zircon_runtime/src/asset/tests/pack.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/hot_update_application.rs
plan_sources:
  - docs/plans/zircon_plugins/09-export-publishing.md
  - docs/plans/zircon_plugins/07-net.md
tests:
  - pack_round_trip
  - duplicate_content_stored_once
  - deterministic_pack_double_run_byte_identical
  - unreferenced_asset_trimmed_and_reported
  - asset_filter_trim_is_reported
  - duplicate_trim_input_path_is_reported
  - run_reports_missing_asset_source_without_writing_pack
  - delta_pack_contains_only_changed_chunks
  - delta_pack_applies_to_base_pack
  - delta_pack_rejects_wrong_base_manifest
  - delta_installer_rebuilds_target_pack_to_staging
  - delta_installer_rejects_wrong_base_without_staging
  - delta_installer_promotes_staged_pack_with_backup
  - delta_installer_copies_staged_pack_when_promotion_rename_fails
  - delta_installer_rejects_invalid_staged_pack_without_replacing_installed
  - delta_installer_writes_install_receipt_from_staging_and_promotion
  - delta_installer_receipt_records_copy_fallback_promotion_method
  - delta_installer_rejects_receipt_for_mismatched_reports
  - native_runtime_hot_update_uses_export_load_manifest_package_set
  - native_runtime_hot_update_reports_non_runtime_manifest_entries_as_skipped
  - native_runtime_hot_update_accepts_runtime_feature_extension_modules
doc_type: module-detail
---

# Asset Pack

`zircon_runtime::asset::pack` owns the M2 `zrpack` writer/reader and trim-plan foundation for export
packages. It reuses the shared `core::framework::net::download::{ZrPackManifest, ZrChunkEntry}` DTO
so release packages and Net content-download manifests describe chunks with the same neutral data
shape.

## Format

The current binary layout is:

```text
header:
  magic "ZRPK"
  format_version: u32
  manifest_offset: u64
  manifest_size: u64
chunks:
  deduplicated content bytes
manifest:
  JSON ZrPackDocumentManifest
```

`ZrPackDocumentManifest` wraps the shared `ZrPackManifest` and adds asset entries:

- `path`: package-local asset path.
- `chunk_hash`: the content-addressed chunk id.
- `size`: original asset byte size.

The writer sorts assets by path, rejects duplicate asset paths, writes each unique chunk once, and
places the JSON manifest at the end of the file. The reader validates magic/version/header ranges,
parses the manifest, validates every asset's chunk range, and can read an asset by package path.

## Delta Format

`delta.rs` owns the M5-T2 hot-update package foundation. A delta file uses the same 24-byte header
layout as a full pack, but the magic is `ZRPD`. Its manifest is `ZrPackDeltaDocumentManifest`:

- `base`: the previous full `ZrPackDocumentManifest`.
- `target`: the new full `ZrPackDocumentManifest`.
- `chunks`: chunk byte ranges physically stored in the delta file.
- `changed_assets`: target asset entries whose chunk hash is absent from the base manifest.
- `removed_assets`: base asset paths that are no longer present in the target manifest.

`ZrPackDeltaWriter` compares old and new full-pack manifests by chunk hash. Chunks already present in
the base pack are not rewritten into the delta, even if a new asset path aliases the same bytes.
`ZrPackDeltaReader` validates the delta header/manifest/chunk ranges and can read changed assets
from the delta payload. It can also apply the delta to a `ZrPackReader` for the installed base pack:
the base manifest must exactly match the delta manifest's `base` field, changed chunks come from the
delta payload, reused chunks come from the base pack by hash, removed assets are omitted because the
output is rebuilt from the target manifest, and the rebuilt manifest must equal the declared target
manifest before bytes are returned.

`install/` owns the runtime-facing pre-install boundary. The folder is split by behavior:
`staging.rs` rebuilds a target full pack from an installed base `.zrpack` and downloaded `.zrpd`,
`promotion.rs` promotes a staged full pack into the installed path, `receipt_io.rs` writes and reads
the persistent install receipt, `file_io.rs` centralizes path-qualified I/O and rename errors, and
the report/error/receipt files keep declarations out of behavior modules.
`ZrPackDeltaInstaller::rebuild_to_staging` applies the delta through the same
manifest-checked primitive, creates the staging parent directory, writes the rebuilt target full pack
to the requested staging path, and returns `ZrPackDeltaInstallReport` with the base/delta/staged
paths, target manifest, staged size, and `delta_apply_verified=true`. Pack-format failures are
reported as `ZrPackDeltaInstallError::Pack`, while file reads and staging writes are reported with
their concrete paths.

`ZrPackDeltaInstaller::promote_staged_pack` validates the staged pack with `ZrPackReader`, records
its manifest and byte size, optionally moves the previous installed pack to a caller-provided backup
path, and then renames the staged pack into the installed path. The promotion report records
`ZrPackPromotionMethod::Renamed` for the fast path. If the final staged-to-installed rename fails
after a backup move and the installed path is no longer occupied, the installer falls back to
copying staged bytes into the installed path, validates the copied pack's manifest and byte size,
then deletes the staged file and reports `CopiedAfterRenameFailure`. If promotion still fails after a
backup move, the installer removes any partial installed file and attempts to restore the backup to
the installed path before returning the original error. This gives Hub/runtime a deterministic
"downloaded patch is staged and can be promoted" call across same-volume and cross-filesystem staging
layouts. It is still a filesystem boundary only: asset-manager reload and in-process asset handle
invalidation remain later slices. NativeDynamic now has separate manifest-driven live-host
application/report entries (`NativePluginLiveHost::hot_reload_runtime_plugins_from_export_root` and
`hot_reload_runtime_plugins_from_export_root_with_bridge_lifecycle`), but those entries are not part
of `ZrPackDeltaInstaller` promotion and do not yet claim a real cdylib success matrix or Hub/editor
end-to-end invocation.

`ZrPackDeltaInstaller::write_install_receipt` persists the evidence that staging and promotion
belong to the same target. It writes a v2 `ZrPackInstallReceipt` JSON file with format version,
base, delta, staged, installed and optional backup paths, target and installed manifests, staged and
installed byte sizes, `delta_apply_verified`, `promotion_method`, and `promoted`. Receipt writing
first checks that the staging report verified delta apply, both reports name the same staged pack,
and the staged target manifest equals the installed manifest. `promotion_method` is copied from
`ZrPackPromotionReport`, so audit consumers can distinguish `Renamed` from
`CopiedAfterRenameFailure` after a cross-filesystem fallback. `ZrPackDeltaInstaller::read_install_receipt`
decodes the same JSON and rejects unsupported receipt format versions. This gives Hub/runtime a
stable audit artifact for "patch applied and promoted" without yet claiming live asset reload. The
runtime native plugin hot-update entry consumes the promoted export root separately through the
native load manifest.

`zircon_export_pack` uses the same delta apply primitive as a writer self-check. When
`--previous-pack` and `--delta-pack` are supplied, the binary writes the `ZRPD` file, reads it back,
applies it to the previous full pack, and compares the rebuilt bytes with the just-written target
pack. The report field `delta_apply_verified` is true only when that reconstruction is
byte-identical; requested delta output with a false verification result is fatal.

## Deduplication

`ZrPackWriter` hashes each asset payload and stores only one copy of identical content. The
`ZrPackWriteReport` returns `deduplicated_assets` so export reporting can state which asset paths
were collapsed onto an existing chunk instead of silently hiding the optimization.

The 09 plan names blake3 as the final chunk id algorithm. M2-T2 deliberately does not add a new
dependency or change lockfiles; `zrpack_content_hash` currently uses a stable 32-byte FNV-derived
content hash as the no-lockfile placeholder. Replacing the hash implementation with blake3 later
should not change the writer/reader ownership shape.

## Dependency Closure Trim

`ZrPackTrimPlanner` is a pure planning layer for the M2-T3 asset-cooking step. It does not read files,
invoke importers, or write pack bytes. Instead it consumes `ZrPackTrimInputAsset` records produced by
the future CookAssets stage:

- `path`: package-local asset path.
- `dependencies`: package-local direct dependency paths emitted by importer or authoring metadata.
- `labels`: profile-facing labels such as `shipping`.

`ZrPackTrimConfig` supplies root entry assets plus the optional export-profile `asset_filter`. The
planner first computes the reachable dependency closure from the roots, then applies the label filter.
That order is intentional: the resulting `ZrPackTrimReport` can distinguish assets that were never
referenced from assets that were reachable but excluded by `asset_filter`. The report contains:

- `included_assets`: sorted package paths that should be passed to `ZrPackWriter`.
- `trimmed_assets`: sorted package paths plus a `ZrPackTrimReason`.
- `missing_dependencies`: root or asset edges that reference a missing path.
- `duplicate_assets`: duplicate package paths that appeared more than once in trim input.
- `diagnostics`: human-readable lines suitable for export reports.

This keeps the 09 plan's "no silent trim" rule enforceable before the CLI owns the full CookAssets
stage. The current API stays independent from the existing asset manager so importer coverage gaps,
such as glTF sub-assets or material texture references, can be surfaced as missing dependency edges
instead of hidden inside pack writing. Duplicate input paths are also structured report data, so the
Pack binary can stop before `ZrPackWriter` without depending on human diagnostic wording.
The Pack binary also treats included asset source materialization failures as publication preflight
errors. Missing manifest entries, missing `source` fields, and source read failures are copied into
the Pack report diagnostics and recorded internally as `asset_source_errors`; when that list is not
empty, the binary writes a fatal report with `manifest=null` and zero asset/chunk counts without
writing full or delta pack bytes.

## Test Coverage

`pack_round_trip` writes two assets, reads the generated bytes, and verifies asset recovery plus
complete byte-plan coverage. `duplicate_content_stored_once` writes two paths with identical bytes
and one unique path, verifies there are three asset entries but only two chunk entries, and checks
both duplicate asset paths read back correctly. `unreferenced_asset_trimmed_and_reported` verifies
that a root scene pulls in its texture dependency and reports an unused texture as trimmed.
`asset_filter_trim_is_reported` verifies that reachable assets without the profile label are reported
with an explicit `AssetFilterMismatch` reason. `duplicate_trim_input_path_is_reported` verifies that
the trim input cannot silently overwrite duplicate package paths and records those paths in
`duplicate_assets`. `run_reports_missing_asset_source_without_writing_pack` covers the binary
boundary for an included asset whose `source` cannot be read: the report still records
`included_assets`, carries the read diagnostic, returns exit code 2, and leaves no `assets.zrpack`.
`deterministic_pack_double_run_byte_identical` proves the writer emits identical bytes when the same
logical assets arrive in a different order.
`delta_pack_contains_only_changed_chunks` verifies that a delta contains only target chunks missing
from the base pack, records removed and reused asset paths, and can read changed asset bytes from the
delta payload. `delta_pack_applies_to_base_pack` verifies that applying the delta to the matching
base pack reconstructs target bytes, preserves reused-chunk aliases, and omits removed assets.
`delta_pack_rejects_wrong_base_manifest` verifies that a delta cannot be applied to a different base
pack manifest. `delta_installer_rebuilds_target_pack_to_staging` verifies that the runtime install
boundary reads base and delta files, stages a rebuilt target pack, reports the staged size, and keeps
target assets readable. `delta_installer_rejects_wrong_base_without_staging` verifies that a base
manifest mismatch is surfaced as a pack error and leaves the staging path absent.
`delta_installer_promotes_staged_pack_with_backup` verifies that promotion validates the staged pack,
backs up the previous installed pack, moves staged bytes into the installed path, reports the target
manifest and size, reports the `Renamed` promotion method, and removes the staged file by rename.
`delta_installer_copies_staged_pack_when_promotion_rename_fails` verifies that a staged-to-installed
rename failure can fall back to copy+delete after backup, still installs the new pack, preserves the
old pack backup, removes the staged file, and reports `CopiedAfterRenameFailure`.
`delta_installer_rejects_invalid_staged_pack_without_replacing_installed` verifies that invalid staged
bytes return a pack error before moving the installed pack or writing a backup.
`delta_installer_writes_install_receipt_from_staging_and_promotion` verifies that a staged and
promoted delta writes a readable JSON receipt containing the expected paths, manifests, byte sizes,
verification flag, `Renamed` promotion method, promotion flag, and format version.
`delta_installer_receipt_records_copy_fallback_promotion_method` verifies that the receipt records
`CopiedAfterRenameFailure` when promotion used the copy fallback after a staged rename failure.
`delta_installer_rejects_receipt_for_mismatched_reports` verifies that inconsistent
staging/promotion reports are rejected before any receipt file is written.

2026-06-14 validation: `rustfmt --edition 2021 --check`, conflict scan, and `git diff --check`
passed for the pack files after formatting. `cargo check -p zircon_runtime --bin
zircon_export_validate --locked --offline --jobs 1 --target-dir
D:\cargo-targets\zircon-export-m1-validate-0614` passed with existing runtime warnings, proving the
new asset pack module type-checks in the runtime crate. The real CLI Pack smoke also passed through
`python -m zircon_export --profile windows-release --out D:\zircon-export-m2-smoke --stage pack
--asset-manifest D:\zircon-export-m2-smoke\assets\assets.json --determinism-check --offline
--target-dir D:\cargo-targets\zircon-export-m1-validate-0614 --pretty`; it wrote `assets.zrpack`,
returned `fatal=false`, included two assets, trimmed one unused/editor-only asset, and confirmed
`deterministic_double_run=true`. Focused `zircon_runtime` lib tests are still blocked by unrelated UI
test compile drift in `table_pointer_routes.rs`.

2026-06-14 M5-T2 validation: `rustfmt --edition 2021 --check`, `python -m py_compile`,
conflict-marker/trailing-whitespace scans, and `git diff --check` passed for the delta and pack CLI
files. `cargo check -p zircon_runtime --bin zircon_export_pack --locked --offline --jobs 1
--target-dir D:\cargo-targets\zircon-export-m5-native-dynamic-0614` passed with existing warnings.
The focused `cargo test -p zircon_runtime --lib delta_pack_contains_only_changed_chunks` command
timed out after 304 seconds during lib-test compilation before executing the target test.

2026-06-15 M5-T2 delta apply validation: `ZrPackDeltaReader::apply_to_base` now reconstructs the
target full pack from a matching base reader plus delta payload and rejects mismatched base
manifests with `DeltaBaseManifestMismatch`. `rustfmt --edition 2021 --check` passed for the touched
pack files. `cargo check -p zircon_runtime --bin zircon_export_pack --locked --offline --jobs 1
--target-dir D:\cargo-targets\zircon-export-m5-native-dynamic-0614 --message-format short --color
never` passed with existing warning noise. Focused `cargo test -p zircon_runtime --lib delta_pack
--locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m5-native-dynamic-0614`
timed out after 604 seconds during lib-test compilation and did not produce target test results.

2026-06-15 M5-T2 packer delta self-check validation: `zircon_export_pack` now verifies every written
delta by applying it to the previous pack and comparing the rebuilt bytes with the target pack before
reporting success. `cargo check -p zircon_runtime --bin zircon_export_pack --locked --offline --jobs
1 --target-dir D:\cargo-targets\zircon-export-m5-native-dynamic-0614 --message-format short --color
never` passed with existing warning noise. A real `python -m zircon_export --stage pack` smoke using
the prebuilt packer wrote `assets.delta.zrpd` and reported `fatal=false`, `delta_apply_verified=true`,
`delta_asset_count=2`, `delta_chunk_count=2`, and `delta_reused_assets=keep.bin`. An earlier smoke
without `--packer` timed out while waiting on `cargo run`; the base pack had already been produced,
and the follow-up prebuilt-packer run completed the target delta path.

2026-06-15 M5-T2 runtime install receipt validation: `ZrPackDeltaInstaller` now writes and reads a
persistent install receipt after a delta has been staged and promoted. `rustfmt --edition 2021` was
applied to the install subtree and pack tests. `cargo check -p zircon_runtime --lib
--no-default-features --features core-min --locked --offline --jobs 1 --target-dir
D:\cargo-targets\zircon-export-m5-delta-install-coremin-0615 --message-format short --color never`
passed with existing warning noise. `cargo test -p zircon_runtime --lib delta_installer
--no-default-features --features core-min --locked --offline --jobs 1 --target-dir
D:\cargo-targets\zircon-export-m5-delta-install-coremin-0615 --message-format short --color never
-- --test-threads=1 --nocapture` passed 6 focused tests.

2026-06-15 M5-T2 runtime promotion copy-fallback validation: promotion now records the promotion
method and falls back from staged-to-installed rename failure to copy+validate+delete when the backup
move has already vacated the installed path. `cargo check -p zircon_runtime --lib
--no-default-features --features core-min --locked --offline --jobs 1 --target-dir
D:\cargo-targets\zircon-export-m5-delta-install-coremin-0615 --message-format short --color never`
passed with existing warning noise. `cargo test -p zircon_runtime --lib delta_installer
--no-default-features --features core-min --locked --offline --jobs 1 --target-dir
D:\cargo-targets\zircon-export-m5-delta-install-coremin-0615 --message-format short --color never
-- --test-threads=1 --nocapture` passed 7 focused tests.

2026-06-15 M5-T2 runtime install receipt promotion-method validation:
`ZrPackInstallReceipt` is now format version 2 and stores the promotion method copied from
`ZrPackPromotionReport`. `ZRPACK_INSTALL_RECEIPT_FORMAT_VERSION` is re-exported from
`zircon_runtime::asset::pack` so callers and tests can check the active receipt schema. `rustfmt
--edition 2021 --check` passed for the touched pack/install files and pack tests. Conflict-marker
and trailing-whitespace scans passed for the touched pack docs/code files. Focused `cargo test -p
zircon_runtime --lib delta_installer_receipt_records_copy_fallback_promotion_method
--no-default-features --features core-min --locked --offline --jobs 1 --target-dir
D:\cargo-targets\zircon-export-m5-delta-install-coremin-0615 --message-format short --color never
-- --test-threads=1 --nocapture` is currently blocked during `zircon_runtime` lib-test compilation
by unrelated render post-process worktree drift:
`post_process/constants/mod.rs` re-exports a private `EXPOSURE_HISTOGRAM_WORKGROUP_SIZE`, and
`post_process/resources/new/construct/new.rs` is out of sync with `create_pipeline_bundle` and
`ScenePostProcessResources` fields. Those files were already dirty outside this pack/install lane,
so this blocked Cargo run is not used as acceptance evidence for the receipt change.

2026-06-15 M5-T2 runtime delta staging validation: `ZrPackDeltaInstaller` now provides the
runtime-facing staging and promotion boundary for downloaded `.zrpd` files. `rustfmt --edition 2021
--check` passed for `zircon_runtime/src/asset/pack/install/*`,
`zircon_runtime/src/asset/pack/mod.rs`, and `zircon_runtime/src/asset/tests/pack.rs`. `cargo check
-p zircon_runtime --lib --no-default-features --features core-min --locked --offline --jobs 1
--target-dir D:\cargo-targets\zircon-export-m5-delta-install-coremin-0615 --message-format short
--color never` passed with existing warning noise, proving the new pack API type-checks in the
minimal runtime configuration. `cargo test -p zircon_runtime --lib delta_installer
--no-default-features --features core-min --locked --offline --jobs 1 --target-dir
D:\cargo-targets\zircon-export-m5-delta-install-coremin-0615 --message-format short --color never
-- --test-threads=1 --nocapture` passed 4 focused tests:
`delta_installer_rebuilds_target_pack_to_staging`,
`delta_installer_rejects_wrong_base_without_staging`,
`delta_installer_promotes_staged_pack_with_backup`, and
`delta_installer_rejects_invalid_staged_pack_without_replacing_installed`. A default-feature packer
check had previously reached `zircon_runtime` lib compilation but was blocked at the time by
unrelated render post-process volume API drift; that blocked run is not used as acceptance evidence
for the installer.
