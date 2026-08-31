# Rich inline resource outcome owner review

Date: 2026-08-30

Status: `RRT-P1-029_frame_qualified_prepare_receipt_static_implemented /
single_snapshot_gpu_prepare_and_exact_revision_binding_static_implemented /
distinct_dependency_owner_invariant_static_implemented /
managed_rust_profile_and_product_validation_pending`

## Scope

This review locates readiness, fallback, and failure ownership for authored-size rich inline images
and image-backed icons. It follows the typed `ImageTexture`/`IconAsset` dependency cut and does not
change layout, resource loading, caching, or renderer behavior. Intrinsic-size layout, font-backed
icons, widget children, WGPU pixels, RSS, power, and product acceptance remain separate work.

## Current-source finding

The compiled artifact correctly owns immutable author intent: requested resource identity, authored
size, baseline, alternative text, and tooltip. `ui_texture_ids` discovers and deduplicates the typed
dependencies across the complete UI submission before scene-resource preparation. Its result is now
a module-owned `UiTextureDependencies` rather than a raw `Vec<ResourceId>`; only that constructor can
publish the sorted, distinct dependency slice accepted by frame preparation. The receipt's one-row-
per-distinct-dependency contract therefore no longer depends on an unchecked caller convention.
`UiTexturePrepareReceipt` remains scene-readable, while its outcome/row types and constructor are
private to the `ui_texture` owner so sibling scene modules cannot synthesize a receipt.

The former failure contract was lossy: `ui_texture_id_for_upload` returned `Option<ResourceId>` after
one full asset load, and the caller discarded the result of a second snapshot-based GPU prepare.
Unresolved identity, readiness/load failure, descriptor rejection, and upload failure therefore
collapsed into absence. The static implementation below removes that boundary while preserving the
existing shared fallback and authored geometry.

The renderer now consumes the frame receipt when it exists. A row is bindable only when its typed
outcome is `Ready`, both resolved ID and prepared revision are present, and the streamer's current GPU
texture still has that exact revision and a one-layer 2D descriptor. Every other case selects the
shared fallback without recompiling or relaying out authored-size text. The legacy generation-scoped
resolution route remains only for frames without a receipt.

## Unreal reference boundary

Local Unreal `FSlateImageRun` owns a brush shared by `Measure` and `OnPaint`. Static null brushes are
replaced with `FStyleDefaults::GetNoBrush`; dynamic image construction asks the Slate renderer to
generate the resource and retains a `FSlateDynamicImageBrush`, releasing it in the run destructor.
`FSlateBrush::ImageSize` remains the layout size while resource object/name, dynamic-load state,
UV region, draw type, and tint remain brush/render data.

Zircon should preserve the same separation rather than copy Unreal object ownership. Authored size
belongs to the compiled run. Mutable load/upload/fallback state belongs to the frame's resource and
renderer owners. A missing texture must not mutate source identity or force rich-text recompilation.

## Required owner contract

1. Keep `CompiledRichText` immutable and generation-independent for authored-size inline resources.
   It retains only the typed request and semantic fallback.
2. Replace the lossy UI upload `Option` boundary with a typed, content-free prepare outcome. At
   minimum it must distinguish unresolved identity, load failure, invalid dimension/layer count,
   upload failure/not-ready, and ready.
3. Qualify every published outcome by requested ID, resolved ID when present, exact resource
   management generation, readiness generation, and prepared texture revision when ready. An
   unqualified `ResourceId -> bool` cache is forbidden.
4. Publish the outcome from the existing scene-resource preparation pass. The image renderer may
   consume that receipt when binding real/fallback resources, but it must not re-query the asset
   registry or repeat texture loading.
5. Keep fallback drawing deterministic. Missing or non-ready resources use the existing shared 2D
   fallback and the authored geometry; later readiness changes binding only.
6. A future accessibility/product projection must use composite artifact identity plus exact inline
   source range and the frame-qualified receipt. It must not store mutable readiness in the compiled
   semantic text or infer failure by comparing pixels/texture pointers.
7. Intrinsic-size syntax, if introduced, is a separate contract: the resolved texture metric revision
   must enter layout identity and invalidate the affected paragraph. It cannot reuse authored-size
   behavior silently.

## Algorithm and performance gate

The target frame algorithm is `O(D + B)` for distinct UI texture dependencies and image batches.
Resolution, descriptor admission, and upload must execute at most once per distinct dependency in the
existing prepare pass. Outcome publication may add one bounded row per dependency and fixed
low-cardinality counters; it may not add a run scan, registry scan from paint, per-frame string, or
second resource cache.

Before changing resolution or cache algorithms, collect an E-drive managed release profile with 1,
16, 128, and 512 inline resources across shared-ID and distinct-ID lanes, cold/warm registry state,
ready/not-ready/missing/wrong-dimension cases, and stable versus replaced management/readiness
generations. Record p50/p95/p99 prepare time, registry candidate visits, load snapshot calls, upload
attempts, dependency dedupe, real/fallback bindings, allocations, working-set delta, and package
power. Compare the same authored-size workload with Unreal Slate; no parity or optimality claim is
allowed before matched evidence.

The current static implementation intentionally does not claim the target bound yet. Direct resource
IDs use immutable-generation `O(1)` lookup, but locator-derived request IDs still require a compatibility
scan of the captured management generation because the current projection has no locator-identity
secondary index. With `D` distinct dependencies and `R` resource rows, that lane remains worst-case
`O(D * R + B)`. The receipt records fixed
`ui.ui_texture_prepare.resolution_scan_row_visit_count`, `snapshot_load_count`, `prepared_reuse_count`,
and `upload_attempt_count` counters so the required 1/16/128/512 profile can determine whether this is
the material bottleneck. No persistent lookup cache or resource-foundation index was added before that
evidence. The distinct-set type reuses the collector's existing `HashSet` and sorted `Vec`; preparation
only borrows its slice, so this invariant adds no second dedupe pass, registry scan, or allocation.

## Implementation order

1. Add typed UI texture admission/prepare reasons at the resource-streamer boundary without changing
   fallback behavior.
2. Retain one bounded, generation-qualified frame receipt and fixed counters in the existing render
   resource owner.
3. Feed the receipt to image prepare; add source-kind/range qualification only when a real product or
   accessibility consumer is ready.
4. Run managed Rust behavior tests for every reason and generation replacement, then real WGPU/PNG
   captures under `docs/tests/runtime/text` using actual inline image/icon assets.
5. Only after the profile identifies registry resolution, upload, binding, or batch rebuild as the
   bottleneck may that owner be optimized.

## Evidence and remaining gates

Current-source and local Unreal review are complete. The production scene-resource pass now publishes
one sorted, frame-epoch-qualified `UiTexturePrepareReceipt` containing exact management/readiness
generation identities and one typed row per distinct dependency. Outcomes distinguish unresolved,
not-ready, load failure, wrong resource kind, invalid descriptor, generation replacement, upload
failure, and ready. Texture preparation reuses one `ResourceSnapshot<TextureAsset>` for descriptor
admission and GPU publication; the previous preflight load plus second snapshot load is removed.
GPU artifact construction failure becomes the content-free per-dependency `UploadFailed` outcome,
while backend submission or ticket-ledger failure remains frame-fatal and propagates to the existing
transaction settlement path instead of being misreported as a fallback resource.

The image renderer consumes the receipt without registry or asset-load repetition and rechecks the
exact prepared revision before selecting the real texture. A 2026-08-31 owner review also rejected two
incorrect follow-up changes: comparing a frame snapshot to the latest resource generation at bind time
would break snapshot consistency, while treating CPU payload readiness as a mandatory gate would reject
a still-valid exact-revision GPU resident texture. Instead, the review closed the actual unchecked
boundary by making frame preparation accept only `UiTextureDependencies`.

Failing-first focused static contracts passed 7/7; the complete Runtime Text static suite passed
106/106 in 2.172 s. Rust regressions for unresolved/wrong-kind
candidate admission and qualified-ready lookup are source-present, and rustfmt parsed all touched Rust
owners. The largest touched production owner is 740 lines, below the 800-line warning boundary.
The docs convention gate still reports the shared-tree baseline of 1536 violations across 416 of 4636
documents, with zero violations attributed to this review or the Text index. The repository structure
gate timed out after 64 seconds without a result; it is not reported as passing and was not retried.

Managed Cargo did not run for this slice because the already accepted request
`9df75274da66456d974c3e89b2d19f58` produced no terminal Cargo result and is not polled or duplicated.
Managed Rust behavior execution, 1/16/128/512 timing/allocation/RSS profile, WGPU/PNG under
`docs/tests/runtime/text`, matched Unreal experience, package power, milestone commit, and WeCom remain
open. This is static implementation evidence, not runtime or performance acceptance.
