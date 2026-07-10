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
  - zircon_runtime/src/bin/zircon_export_pack/error.rs
  - zircon_runtime/src/bin/zircon_export_pack/manifest.rs
  - zircon_runtime/src/bin/zircon_export_pack/run.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/tests/pack.rs
  - zircon_runtime/src/asset/tests/mod.rs
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
  - zircon_runtime/src/bin/zircon_export_pack/error.rs
  - zircon_runtime/src/bin/zircon_export_pack/manifest.rs
  - zircon_runtime/src/bin/zircon_export_pack/run.rs
  - zircon_runtime/src/asset/tests/pack.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/hot_update_application.rs
plan_sources:
  - docs/plans/zircon_plugins/09-export-publishing.md
  - docs/plans/zircon_plugins/07-net.md
  - docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
tests:
  - pack_round_trip
  - pack_manifest_chunk_plan_round_trips_from_asset_owner
  - duplicate_content_stored_once
  - deterministic_pack_double_run_byte_identical
  - pack_writer_rejects_unsafe_asset_paths
  - pack_writer_rejects_unnormalized_asset_paths
  - pack_reader_rejects_manifest_asset_path_schema
  - pack_reader_rejects_duplicate_manifest_asset_paths
  - pack_reader_rejects_unsorted_manifest_asset_paths
  - pack_reader_rejects_manifest_pack_version_mismatch
  - pack_reader_rejects_manifest_chunk_table_shape
  - pack_reader_rejects_manifest_total_size_mismatch
  - pack_reader_rejects_manifest_asset_chunk_mismatch
  - pack_reader_rejects_manifest_extra_unreferenced_chunks
  - pack_reader_rejects_chunk_payload_hash_mismatch
  - pack_reader_rejects_payload_manifest_gap
  - pack_reader_rejects_manifest_trailing_bytes
  - delta_reader_rejects_nested_pack_manifest_asset_path_schema
  - delta_reader_rejects_changed_asset_path_schema
  - delta_reader_rejects_removed_asset_path_schema
  - delta_reader_rejects_duplicate_changed_and_removed_asset_paths
  - delta_reader_rejects_unsorted_changed_and_removed_asset_paths
  - delta_reader_rejects_delta_manifest_format_version_mismatch
  - delta_reader_rejects_removed_asset_set_mismatch
  - delta_reader_rejects_changed_asset_set_mismatch
  - delta_reader_rejects_delta_chunk_table_mismatch
  - delta_reader_rejects_changed_chunk_payload_hash_mismatch
  - delta_reader_rejects_payload_manifest_gap
  - delta_reader_rejects_manifest_trailing_bytes
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
  - native_runtime_delta_hot_update_installs_pack_then_runs_manifest_hot_reload
  - native_runtime_hot_update_uses_export_load_manifest_package_set
  - native_runtime_hot_update_reports_non_runtime_manifest_entries_as_skipped
  - native_runtime_hot_update_accepts_runtime_feature_extension_modules
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/asset_tests/pack.rs::runtime_15_asset_pack_header_readers_are_panic_free
doc_type: module-detail
---

# Asset Pack

`zircon_runtime::asset::pack` owns the M2 `zrpack` format, manifest/chunk DTOs, writer/reader, and
trim-plan foundation for export packages. `ZrPackManifest` and `ZrChunkEntry` are format-owned asset
protocol data; they are not network transport contracts. Download providers may transfer pack bytes,
but `core::framework::net` does not define or re-export the pack format.

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

`ZrPackDocumentManifest` wraps the asset-owned `ZrPackManifest` and adds asset entries:

- `path`: package-local asset path.
- `chunk_hash`: the content-addressed chunk id.
- `size`: original asset byte size.

The writer first validates each input asset path as a package-local safe relative asset path in
normalized forward-slash form. Empty paths, absolute paths, drive-letter paths, `.`, `..`, empty
segments, padded strings, and backslash-separated paths are rejected before sorting or hashing. The
writer then sorts assets by path, rejects duplicate asset paths, writes each unique chunk once, and
places the JSON manifest at the end of the file. The reader validates magic/version/header ranges and
requires the manifest range to consume the rest of the file before parsing the manifest. It then
applies the same safe normalized asset path gate to the decoded manifest,
rejects duplicate or non-sorted asset rows, validates every asset's chunk range, and can read an
asset by package path. This keeps externally supplied, downloaded, or hand-written pack manifests on
the same canonical package-path contract as bytes produced by `ZrPackWriter`. The decoded pack
document also keeps the writer's chunk-table contract before byte ranges are trusted: `pack.version`
must match the active format, chunk hashes must be unique and sorted by hash, `pack.total_size` must
equal the sum of chunk sizes, every asset must reference a chunk with the same byte size, and the
chunk table may not contain unreferenced chunk rows. The reader also derives the contiguous payload
end from the chunk offsets and sizes, then requires the binary header's `manifest_offset` to match
that end before exposing bytes. After a chunk range is bounded, the reader recomputes
`zrpack_content_hash` over the physical payload bytes and rejects any chunk whose bytes no longer
match the manifest hash.

Runtime 15 M3 asset pack panic-free header readers
(`runtime_15_asset_pack_header_readers_panic_free_static_passed_cargo_deferred`) keeps full-pack and
delta header parsing on typed read helpers instead of panic-based slice conversion. `reader.rs` owns
`read_header_u32(...)`, `read_header_u64(...)`, and the fixed-byte helper; `delta.rs` reuses those
helpers for the matching ZRPD header. Header underflow and offset overflow continue to report
`ZrPackError::HeaderTooSmall`, while unsupported versions, manifest bounds, trailing bytes, payload
extent, and content-hash checks keep the existing `ZrPackError` variants and public pack/delta API.

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
`ZrPackDeltaReader` validates the delta header, decodes the manifest, then applies the same manifest
identity boundary as full-pack reads before chunk ranges are trusted: embedded `base` and `target`
pack documents must have safe normalized, unique, path-sorted asset rows; `changed_assets[]` must
carry the same asset-entry path contract; and `removed_assets[]` must be a safe normalized, unique,
path-sorted path list. It can read changed assets from the delta payload. It can also apply the delta
to a `ZrPackReader` for the installed base pack. The decoded delta manifest's own `format_version`
must match the active pack format version; `removed_assets[]` must equal the path difference
`base.assets - target.assets`; `changed_assets[]` must equal the target asset entries whose chunk
hash is absent from the base chunk table; and the delta `chunks[]` table must contain exactly the
unique changed chunk hashes in sorted order. Only after those checks do changed chunks come from the
delta payload and reused chunks from the base pack by hash. Delta payload chunks use the same
payload-extent and content-hash verification as full packs before changed asset bytes are exposed:
`manifest_offset` must point to the end of the contiguous changed-chunk payload, and no changed chunk
range can cross into the embedded manifest. Removed assets are omitted because the output is rebuilt
from the target manifest, and the rebuilt manifest must equal the declared target manifest before
bytes are returned.

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
layouts. It is still a filesystem boundary only for asset-manager reload and in-process asset handle
invalidation; those remain later slices. NativeDynamic runtime hot update now has a composition
entry in the native live host: `NativePluginLiveHost::hot_reload_runtime_plugins_after_delta_pack_install(...)`
delegates all pack mutation to `ZrPackDeltaInstaller`, optionally writes the install receipt, and
then runs the export-root manifest hot-update pass. The pack layer still owns staging, promotion,
backup, receipt validation, and manifest evidence; the native loader owns runtime plugin reload
diagnostics. This does not yet claim a real cdylib success matrix or Hub/editor end-to-end
invocation.

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

The export binary compiles the same asset pack source modules through explicit `#[path]` declarations.
It re-exports `ZrPackManifest` and `ZrChunkEntry` from the shared manifest owner; the former fake
`core::framework::net` module and duplicate DTO definitions were hard-deleted. Library and tool
builds therefore have one source of truth for the serialized format.

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
`pack_manifest_chunk_plan_round_trips_from_asset_owner` locks the asset-owned manifest/chunk helper
behavior and serde shape that previously lived in the Net contract test.

Frameworks 03 ownership validation on 2026-07-10 passed the WSL nightly locked/offline
`zircon_export_pack` check in 3m17s and the binary test target in 9m31s with 3/3 tests. A separate
`zircon_runtime --lib --features core-min` pack-filter attempt did not execute tests because the
global lib-test tree still has 84 un-gated optional-domain references; that command is recorded as
pending M1 test-support work, not as a pack regression or pass.
`pack_writer_rejects_unsafe_asset_paths` keeps unsafe package paths out of writer manifests, and
`pack_writer_rejects_unnormalized_asset_paths` keeps padded or backslash-separated paths from being
silently normalized at the pack byte boundary.
`pack_reader_rejects_manifest_asset_path_schema`,
`pack_reader_rejects_duplicate_manifest_asset_paths`, and
`pack_reader_rejects_unsorted_manifest_asset_paths` apply the same manifest identity boundary on
read: unsafe or unnormalized paths, duplicate asset rows, and non-canonical asset order are rejected
before chunk range validation trusts the decoded manifest.
`pack_reader_rejects_manifest_pack_version_mismatch`,
`pack_reader_rejects_manifest_chunk_table_shape`,
`pack_reader_rejects_manifest_total_size_mismatch`,
`pack_reader_rejects_manifest_asset_chunk_mismatch`, and
`pack_reader_rejects_manifest_extra_unreferenced_chunks` keep decoded full-pack documents on the
writer's chunk-table contract: pack format version, chunk hash uniqueness/order, total byte size,
asset chunk references, asset byte sizes, and the absence of unreferenced chunks must all be coherent
before `ZrPackReader` trusts the manifest.
`pack_reader_rejects_chunk_payload_hash_mismatch`,
`pack_reader_rejects_payload_manifest_gap`, and
`pack_reader_rejects_manifest_trailing_bytes` keep the physical bytes tied to that manifest after the
shape checks pass: a full pack is rejected if chunk payload bytes no longer hash to the declared
chunk id, if the header's `manifest_offset` leaves undeclared bytes between the payload extent and
the embedded manifest, or if the embedded manifest is followed by trailing bytes.
`delta_reader_rejects_nested_pack_manifest_asset_path_schema`,
`delta_reader_rejects_changed_asset_path_schema`,
`delta_reader_rejects_removed_asset_path_schema`,
`delta_reader_rejects_duplicate_changed_and_removed_asset_paths`, and
`delta_reader_rejects_unsorted_changed_and_removed_asset_paths` extend that boundary to downloaded
ZRPD manifests: embedded base/target pack documents, changed asset entries, and removed asset path
lists must all use safe normalized, unique, canonical asset identities before changed chunk ranges are
accepted.
`delta_reader_rejects_delta_manifest_format_version_mismatch`,
`delta_reader_rejects_removed_asset_set_mismatch`,
`delta_reader_rejects_changed_asset_set_mismatch`, and
`delta_reader_rejects_delta_chunk_table_mismatch` cover the next semantic gate: delta manifests must
use the current format version, and their removed assets, changed asset entries, and chunk table must
be derivable from the embedded base/target manifests before a downloaded patch is trusted.
`delta_reader_rejects_changed_chunk_payload_hash_mismatch`,
`delta_reader_rejects_payload_manifest_gap`, and
`delta_reader_rejects_manifest_trailing_bytes` apply the same physical-byte checks to ZRPD payloads.
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
`native_runtime_delta_hot_update_installs_pack_then_runs_manifest_hot_reload` verifies the M5/T2
composition boundary: a staged delta rebuild promotes into the installed pack path with backup,
writes a v2 receipt, then runs manifest-driven NativeDynamic runtime hot update and reports the
runtime plugin manifest diagnostics separately from pack install evidence.

`runtime_15_asset_pack_header_readers_are_panic_free` locks the typed header reader helper ownership
and rejects reintroducing `expect("header ... bytes")` or `try_into().unwrap()` header conversions
in `asset/pack/{reader,delta}.rs`.

2026-06-14 validation: `rustfmt --edition 2021 --check`, conflict scan, and `git diff --check`
passed for the pack files after formatting. `cargo check -p zircon_runtime --bin
zircon_export_validate --locked --offline --jobs 1 --target-dir
D:\cargo-targets\zircon-export-m1-validate-0614` passed with existing runtime warnings, proving the
new asset pack module type-checks in the runtime crate. The real CLI Pack smoke also passed through
`python -m tools.zircon_export --profile windows-release --out D:\zircon-export-m2-smoke --stage pack
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

2026-07-01 M5/T2 NativeDynamic delta hot-update validation: the runtime composition entry now calls
`ZrPackDeltaInstaller::rebuild_to_staging`, `promote_staged_pack`, and optional
`write_install_receipt` before invoking manifest-driven runtime plugin hot update. Scoped
`rustfmt --edition 2021 --check` passed for the touched pack/native-loader files, and
`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1
--target-dir D:\cargo-targets\zircon-plugin-delta-hot-update-0701-check --message-format short
--color never` passed with existing warning noise. The focused lib-test command
`cargo test -p zircon_runtime --lib --no-default-features --features core-min
native_runtime_delta_hot_update_installs_pack_then_runs_manifest_hot_reload --locked --jobs 1
--target-dir D:\cargo-targets\zircon-plugin-delta-hot-update-0701-green --message-format short
--color never -- --test-threads=1 --nocapture` passed 1/1 after the existing Runtime 07
`performance_hotspots/owner_budget.rs` child-owner split was re-exposed through explicit
`#[path = "owner_budget/..."]` module mounts.

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
never` passed with existing warning noise. A real `python -m tools.zircon_export --stage pack` smoke using
the prebuilt packer wrote `assets.delta.zrpd` and reported `fatal=false`, `delta_apply_verified=true`,
`delta_asset_count=2`, `delta_chunk_count=2`, and `delta_reused_assets=keep.bin`. An earlier smoke
without `--packer` timed out while waiting on `cargo run`; the base pack had already been produced,
and the follow-up prebuilt-packer run completed the target delta path.

2026-06-27 Runtime 15 F5 export CLI typed errors: `zircon_export_pack/error.rs` now owns
`ExportPackError` / `ExportPackResult` for the packer binary. Pack argument usage, asset manifest
read/decode, pack/report file IO, previous/full/delta ZRPK/ZRPD read-write-verify failures, delta
apply mismatch, and deterministic comparison failures stay typed until the CLI `main.rs` display
boundary. `manifest.rs::pack_inputs(...)` is non-fallible because source materialization failures are
reported in the Pack JSON report as fatal preflight diagnostics; that report schema remains a string
diagnostic surface. `review_f5_export_cli_uses_typed_errors_before_cli_boundary` locks the no
`Result<_, String>` rollback for the packer owner; Cargo remains deferred under active compile lanes.

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

2026-06-21 writer path schema validation: `ZrPackWriter` now rejects unsafe or non-normalized asset
paths before sorting, hashing, or manifest serialization. `rustfmt --check` passed for
`manifest.rs`, `writer.rs`, and `asset/tests/pack.rs`; conflict-marker and `git diff --check` scans
passed with only LF/CRLF warnings. Focused `cargo test -p zircon_runtime --locked --lib
pack_writer_rejects` and the lighter `cargo test -p zircon_runtime --locked --no-default-features
--features core-min --lib pack_writer_rejects` both timed out during compile before producing test
results, and the leftover Cargo/rustc processes were stopped. This slice therefore records the new
tests and static checks, but does not claim a Rust test pass.

2026-06-21 reader manifest schema validation: `ZrPackReader` now validates decoded full-pack
manifest asset paths with the same shared helper as `ZrPackWriter`, then rejects duplicate asset
paths and non-sorted `assets[]` rows before validating referenced chunk ranges. `rustfmt --check`,
conflict-marker scan, and `git diff --check` passed for `manifest.rs`, `reader.rs`, `writer.rs`, and
`asset/tests/pack.rs`; `git diff --check` only reported LF/CRLF warnings. Focused
`cargo test -p zircon_runtime --locked --no-default-features --features core-min --lib
pack_reader_rejects --no-run --jobs 1` timed out after 10 minutes during compilation before
producing test-build results, and leftover Cargo/rustc processes were stopped. This slice records the
new reader tests and static checks, but does not claim a Rust test pass.

2026-06-21 delta reader manifest schema validation: `ZrPackDeltaReader` now validates decoded ZRPD
manifest asset identities before changed chunk range validation. The validation reuses full-pack
document checks for embedded `base` and `target`, and applies the same safe normalized, unique,
sorted path contract to `changed_assets[]` and `removed_assets[]`. `rustfmt --check`,
conflict-marker scan, and `git diff --check` passed for `manifest.rs`, `delta.rs`, `reader.rs`,
`writer.rs`, and `asset/tests/pack.rs`; `git diff --check` only reported LF/CRLF warnings. Focused
`cargo test -p zircon_runtime --locked --no-default-features --features core-min --lib
delta_reader_rejects --no-run --jobs 1` timed out after 10 minutes during compilation before
producing test-build results, and leftover Cargo/rustc processes were stopped. This slice records the
new delta reader tests and static checks, but does not claim a Rust test pass.

2026-06-21 delta reader semantic validation: `ZrPackDeltaReader` now validates the decoded ZRPD
manifest's own format version plus the semantic relationship between embedded base/target manifests,
`removed_assets[]`, `changed_assets[]`, and the physical delta chunk table before reading changed
chunk ranges. `rustfmt --check`, conflict-marker scan, and `git diff --check` passed for
`manifest.rs`, `delta.rs`, `reader.rs`, `writer.rs`, and `asset/tests/pack.rs`; `git diff --check`
only reported LF/CRLF warnings. The first focused `cargo test -p zircon_runtime --locked
--no-default-features --features core-min --lib delta_reader_rejects --no-run --jobs 1` exposed two
test-fixture type errors in this slice, both fixed by passing borrowed payload bytes to
`extend_from_slice`. The second focused run cleared those local errors and is currently blocked by
pre-existing lib-test compile drift in
`zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs:415`
(`struct takes 0 lifetime arguments but 1 lifetime argument was supplied`). This slice records the
new semantic tests and static checks, but does not claim a Rust test pass.

2026-06-21 full pack document chunk-table validation: `ZrPackReader` now validates decoded full-pack
document chunk semantics before trusting manifest byte ranges, and the same helper also covers
embedded base/target documents inside ZRPD manifests. `rustfmt --check`, conflict-marker scan, and
`git diff --check` passed for `manifest.rs`, `delta.rs`, `reader.rs`, `writer.rs`, and
`asset/tests/pack.rs`; `git diff --check` only reported LF/CRLF warnings. Focused
`cargo test -p zircon_runtime --locked --no-default-features --features core-min --lib
pack_reader_rejects --no-run --jobs 1` produced no local pack-reader diagnostics and is blocked by
the same pre-existing lib-test compile drift in
`zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs:415`
(`struct takes 0 lifetime arguments but 1 lifetime argument was supplied`). This slice records the
new pack-reader tests and static checks, but does not claim a Rust test pass.

2026-06-21 reader payload extent validation: `ZrPackReader` and `ZrPackDeltaReader` now reject full
pack or ZRPD bytes whose header `manifest_offset` does not match the contiguous payload end derived
from their decoded chunk tables. This closes the runtime load path for hand-written artifacts that
keep valid chunk hashes but insert undeclared bytes before the embedded manifest. New coverage:
`pack_reader_rejects_payload_manifest_gap` and `delta_reader_rejects_payload_manifest_gap`, both
starting from writer-produced valid bytes and inserting only a manifest gap. `rustfmt --check`
passed for `manifest.rs`, `reader.rs`, `delta.rs`, and `asset/tests/pack.rs`. Focused
`cargo test -p zircon_runtime --locked --no-default-features --features core-min --lib
payload_manifest_gap --no-run --jobs 1` did not produce a target test build result: the first run
stopped after dependency compilation with no captured Rust diagnostic, and a second longer run also
exited non-zero during dependency compilation without leaving Cargo/rustc processes. This slice
records static checks and test coverage, but does not claim a Rust test pass.

2026-06-21 reader manifest trailing-bytes validation: `ZrPackReader` and `ZrPackDeltaReader` now
reject full pack or ZRPD bytes whose embedded manifest does not end at the artifact boundary. This
closes the runtime load path for hand-written artifacts that keep valid payload extents and chunk
hashes but append undeclared bytes after the manifest. New coverage:
`pack_reader_rejects_manifest_trailing_bytes` and `delta_reader_rejects_manifest_trailing_bytes`,
both starting from writer-produced valid bytes and appending only `trail`. `rustfmt --check` passed
for `manifest.rs`, `reader.rs`, `delta.rs`, and `asset/tests/pack.rs`. Focused
`cargo test -p zircon_runtime --locked --no-default-features --features core-min --lib
manifest_trailing_bytes --no-run --jobs 1` exited non-zero during dependency/local crate
compilation without captured Rust diagnostics. The closeout process audit found no remaining
`manifest_trailing_bytes` command, but did find unrelated `ecs_query` and
`render_product_multi_spot_shadow_atlas_darkens_receivers_capture` Cargo/rustc validation processes
still running, so they were not cleaned by this slice. This slice records static checks and test
coverage, but does not claim a Rust test pass.
