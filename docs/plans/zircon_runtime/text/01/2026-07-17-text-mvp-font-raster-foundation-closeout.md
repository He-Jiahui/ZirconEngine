---
related_code:
  - zircon_runtime/Cargo.toml
  - Cargo.lock
  - zircon_plugins/Cargo.lock
  - zircon_runtime/src/asset/assets/font_source.rs
  - zircon_runtime/src/asset/tests/assets/font.rs
  - zircon_runtime/src/text/font/database.rs
  - zircon_runtime/src/text/font/database/asset_lifecycle.rs
  - zircon_runtime/src/text/font/database/equivalence.rs
  - zircon_runtime/src/text/font/database/error.rs
  - zircon_runtime/src/text/font/database/face_access.rs
  - zircon_runtime/src/text/font/database/face_matching.rs
  - zircon_runtime/src/text/font/database/fallback_queries.rs
  - zircon_runtime/src/text/font/database/system_fonts.rs
  - zircon_runtime/src/text/font/database/tests.rs
  - zircon_runtime/src/text/font/database/tests/asset_lifecycle.rs
  - zircon_runtime/src/text/font/database/tests/composite.rs
  - zircon_runtime/src/text/font/database/tests/fallback.rs
  - zircon_runtime/src/text/font/database/tests/matching.rs
  - zircon_runtime/src/text/font/database/tests/sources.rs
  - zircon_runtime/src/text/font/database/tests/system_policy.rs
  - zircon_runtime/src/text/font/database/tests/variations.rs
  - zircon_runtime/src/text/font/face_metadata.rs
  - zircon_runtime/src/text/font/fallback_cache.rs
  - zircon_runtime/src/text/font/fallback_cache/tests.rs
  - zircon_runtime/src/text/font/instance.rs
  - zircon_runtime/src/text/font/shared.rs
  - zircon_runtime/src/text/font/shared/tests.rs
  - zircon_runtime/src/text/font/vertical_metrics.rs
  - zircon_runtime/src/text/language.rs
  - zircon_runtime/src/text/mod.rs
  - zircon_runtime/src/text/render_state.rs
  - zircon_runtime/src/text/shaping/cosmic/font_system_cache.rs
  - zircon_runtime/src/text/shaping/fallback_spans.rs
  - zircon_runtime/src/text/sdf/font_bake.rs
  - zircon_runtime/src/text/sdf/font_bake/tests/offline.rs
  - zircon_runtime/src/text/parallel/raster_pool.rs
  - zircon_runtime/src/text/parallel/tests.rs
  - zircon_runtime/src/text/native_bitmap_atlas.rs
  - zircon_runtime/src/text/native_bitmap_atlas/raster_key.rs
  - zircon_runtime/src/text/native_bitmap_atlas/raster_key/tests.rs
  - zircon_runtime/src/text/native_bitmap_atlas/source_cache.rs
  - zircon_runtime/src/text/native_bitmap_atlas/tests/source_cache.rs
  - zircon_runtime/src/text/atlas/bitmap_run/tests/dirty_upload.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/font_assets.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/font_id_report.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/resolved_batches.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/tests.rs
plan_sources:
  - docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
  - docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md
  - docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - cargo +1.94.1 metadata --locked --format-version 1 --no-deps
  - cargo +1.94.1 metadata --manifest-path zircon_plugins/Cargo.toml --locked --format-version 1 --no-deps
  - python -m unittest -v tools.tests.test_frameworks_05_text_boundary tools.tests.test_text_01_composite_activation
  - managed Windows exact shared-face owner-mapping and UI asset lifecycle tests
  - managed Windows zircon_runtime focused Text font/raster/bitmap-upload tests
  - managed Windows zircon_runtime default/UI lib-test batch
  - managed Windows graphics-only upward gate
  - managed Windows exact ignored runtime Text product framebuffer exporter
doc_type: milestone-closeout
status: validation_pending
---

# Text MVP font/raster foundation closeout

Plan: `docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md`

Milestone: Text MVP foundation F1

Status: `validation_pending`

## Completed implementation

- System-font discovery is idempotent per `FontDatabase`. Repeated renderer construction no longer
  asks `fontdb` to append another operating-system catalog.
- `TextRenderState` constructs cosmic `FontSystem` directly from the shared backend database and a
  normalized system locale. The discarded temporary `FontSystem::new()` system scan is gone.
- `sys-locale` is optional and reachable only through the existing Runtime `text` feature.
- Raster worker request-channel loss fails closed before an in-flight id is recorded.
- Raster work is a face-owned CPU source. The fixed `page_generation=0` target and worker stale-page
  telemetry are removed; real page generations remain in atlas allocation/staging/upload guards.
- `render_perf_text_async_upload_merges_per_page` locks the existing one-upload-per-page contract
  through dirty-rect union, staging, and texture-request planning.
- Primary fallback resolution and SDF font lookup no longer use production `expect` paths for
  optional state that can fail closed at the owning layer.
- Each font face generation now owns one parsed `FontFaceMetadata` projection. Variation axes,
  coverage/glyph mapping, vertical/decoration metrics, and stable source identity are shared by
  shaping, fallback, native raster, and SDF consumers instead of reparsing SFNT tables per glyph.
- Effective variation and fallback candidate/resolution state use bounded, observable caches;
  complete-cluster identity, script/language, composite identity, and font generation participate
  in the fallback keys, so hot repeated text avoids family normalization/sort rebuilds without
  crossing a reload generation.
- Native bitmap glyphs keep persistent atlas slots across frames. The deterministic 256-glyph
  upload proxy produces 1.75 MiB of dirty-page traffic and stays below the 2 MiB/frame MVP budget;
  real workbench calibration remains a separate acceptance item.
- Cosmic `CacheKey.font_size_bits` is treated as an already-physical size at the native atlas
  boundary, so 12 px and 24 px inputs select distinct buckets without applying surface scale twice.
- Production font mutation now has one authoritative shared `FontDatabase`. Replace/remove/default/
  composite operations update it under the Text-owned write lock; renderer instances no longer
  publish cloned databases that can overwrite a newer owner topology.
- Font assets use stable asset-reference owners. Database render-input changes and per-owner asset
  mapping changes are reported separately, so a shared physical face can remain resident while a
  `Missing -> Ready` or `Ready -> Missing` owner transition still invalidates UI shaping, bitmap,
  and SDF consumers.
- Graphics resolves backend ids through the narrow Text-owned `font_face_id` query. Constructor
  publication, face-count inference, and production access to the complete CPU font database are
  removed.
- TTC standalone-face extraction writes each source table directly into the final SFNT buffer and
  clears `head.checkSumAdjustment` in place. The checksum read now completes before the directory
  record is mutated, preserving the zero-per-table-scratch path without overlapping borrows.
- Current-source default/UI fixtures now follow the production contracts: family-less imported
  manifests publish their parsed source family, stable asset-reference owners are explicitly retired
  between shared-database tests before exact face-count assertions, and the offline-SDF late-manifest
  fixture resolves through the actual project asset root instead of `project_root/fonts`.

## Structure and performance evidence

- No compatibility field, wrapper, renderer-root reconstruction, or duplicate atlas merge policy
  was introduced.
- System-policy tests are in `font/database/tests/system_policy.rs`; per-page upload behavior is in
  `atlas/bitmap_run/tests/dirty_upload.rs`.
- System-font policy, discovery, and backend-face registration are isolated in
  `font/database/system_fonts.rs`; the database root no longer owns this platform behavior family
  and is reduced from 941 to 502 lines. `font/database/{error,face_access,face_matching,
  fallback_queries,instances,system_fonts}.rs` separately own their contracts instead of mixing
  them with orchestration.
- The pre-split working root `font/database/tests.rs` was 903 lines (939 lines in base HEAD before
  system-policy extraction) and is now a 20-line structural root. Matching, variations, source
  materialization, CompositeFont, fallback, and system-policy tests are owned by folder-backed
  modules; the largest child is 344 lines.
- Shared publish comparison remains on the low-frequency write path; shaping/raster hot paths add no
  font-byte scan or global lock.
- The current `zircon_runtime/src/text` tree contains 224 Rust files, has no file above the
  800-line soft budget, and tops out at 789 lines. New private-helper coverage remains folder-backed
  (`native_bitmap_atlas/raster_key/tests.rs`) rather than returning to a root test monolith.
- Rust 1.94.1 scoped rustfmt, manifest parsing, obsolete-symbol scan, and owned `git diff --check`
  pass on current source.

## Validation state

- On current source after the shared-owner/UI invalidation and three fixture-contract repairs,
  `python -m unittest -v tools.tests.test_frameworks_05_text_boundary
  tools.tests.test_text_01_composite_activation` passed 19/19 in 603.405 seconds. Before the fixture
  repair, snapshot `946` matched its 115 coordinator-attributed paths with zero mismatch and a clean
  index; this static boundary evidence does not substitute for the required fresh post-fix Cargo
  manifests.
- Runtime07 current-source managed job `835ae0a9ff4b46fba734b09c7c63e60e` reached the shared
  Runtime production graph and exposed one Text-owned E0502 in `asset/assets/font_source.rs`; the
  lowest owner now separates table checksum reading from directory mutation. Rust 1.94.1 rustfmt
  and scoped diff-check are green; fresh managed compile and TTC behavior gates remain pending.
- The root and plugin `sys-locale` lock closure is complete in managed commit
  `a7607a306f9f00e37004f6d668aa6cea82d76876`; its coordinator action suite passed 34/34 and the
  cross-plan failure return is fixed.
- Managed graphics-only focused job `fa55a334a71343e1a661f5773355ba50`, run
  `e113ea2407ab4fbdbe98994ffc7f6b4f`, passed `78 / 0 / 2` with `6919` filtered on immutable
  snapshot `946`. The original graphics-only package upward job
  `a1698add813845d68b7627dbf318bab3`, run `6cd7e0a3df34411593f06d13e3f2e125`,
  compiled the Text/graphics production library and then stopped before tests on foreign Plugins09
  E0382 in `bin/zircon_export_validate/run.rs`; lifecycle node `722179` remains the rerun dependency.
- Managed default/UI lib-test job `f9f5581fb83b40c2a3cc81aa15f5bcaa`, run
  `b98dc769094b4bd9b96fc445fd8a1332`, naturally released exit `101` with no live PIDs after
  `776 passed / 7 failed / 2 ignored / 8083 filtered` in 133.29 seconds. Four failures were foreign
  substring-filter hits and now have canonical open handoffs: Render01 node `732714`, Runtime07 node
  `732788`, and Runtime15 nodes `732833`/`732834`. The three Text failures were stale imported-family
  assertion, a leaked stable owner between shared-database tests, and an offline-SDF asset-root path;
  all three fixtures are fixed
  and require fresh exact current-source reruns. This red batch is retained as diagnostic evidence,
  not acceptance.
- Independent source review is `0 Critical / 0 Important / 0 Minor`, `READY` after the
  system-font and generation-owned metadata/cache extraction. Follow-up independent reviews also
  closed the shared-database owner topology and multi-owner UI asset-mapping lifecycle at
  `0 Critical / 0 Important / 0 Minor`. Five registered Text01 failure
  lifecycles are `implementation_complete / review_green / managed_validation_pending`; they remain
  open until fresh post-fix focused, original-reproduction, broad, and upward gates pass.
  Post-fix default/UI focused tests, ignored metadata/fallback scale tests, the graphics-only upward
  rerun after Plugins09, the real WGPU product framebuffer, fresh image inspection, failure returns,
  review and managed milestone commit remain incomplete. This record must not be promoted to
  `accepted` until all those items have current-source evidence.

## Visual evidence policy

The final acceptance image must come from the real WGPU Text product framebuffer and must be copied
to `docs/tests/runtime/text`. A plan/status image or an artifact left under any `target` directory is
not acceptable.
