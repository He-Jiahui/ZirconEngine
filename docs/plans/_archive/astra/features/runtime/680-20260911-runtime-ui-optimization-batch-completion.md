---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/76-runtime-ui-layout-box-model-measure-arrange-flex-grid-overflow-scroll-virtualization-dpi-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/11c-gpu-ui-renderer-atlas-sdf-batch-clip-submit-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_runtime/200/2026-09-19-focus-hovered-retain-capacity.md
  - docs/plans/optimize/zircon_runtime/200/2026-09-19-focus-pointer-drag-retain-capacity.md
  - docs/plans/optimize/zircon_runtime/77/2026-09-19-input-timer-drain-capacity.md
  - docs/plans/optimize/zircon_runtime/74/2026-09-19-dependency-cascade-target-capacity.md
  - docs/plans/optimize/zircon_runtime/200/2026-09-19-surface-frame-patch-range-capacity.md
  - docs/plans/optimize/zircon_runtime/49/2026-09-19-virtual-geometry-overlay-capacity.md
  - docs/plans/optimize/zircon_runtime/73/2026-09-19-runtime-pseudo-state-capacity.md
  - docs/plans/optimize/zircon_runtime/75/2026-09-20-tree-view-metadata-collection-capacity.md
  - docs/plans/optimize/zircon_runtime/11a/2026-09-20-ui-node-resource-registration-capacity.md
  - docs/plans/optimize/zircon_runtime/03/2026-09-20-style-plan-rule-capacity.md
  - docs/plans/optimize/zircon_runtime/169/2026-09-20-navigation-projection-capacity.md
  - docs/plans/optimize/zircon_runtime/169/2026-09-20-navigation-path-dedup-in-place.md
  - docs/plans/optimize/zircon_runtime/169/2026-09-20-navigation-vertex-projection-capacity.md
  - docs/plans/optimize/zircon_runtime/26/2026-09-20-particle-extract-output-capacity.md
  - docs/plans/optimize/zircon_runtime/03/2026-09-20-style-import-resolution-capacity.md
  - docs/plans/optimize/zircon_runtime/73/2026-09-20-v2-style-rule-filter-retain.md
  - docs/plans/optimize/zircon_runtime/level_system/2026-09-20-animation-drain-output-capacity.md
  - docs/plans/optimize/zircon_runtime/world/2026-09-20-render-post-process-output-capacity.md
  - docs/plans/optimize/zircon_runtime/11c/2026-09-20-logical-text-batch-capacity.md
  - docs/plans/optimize/zircon_runtime/85/2026-09-21-resolved-dependency-output-capacity.md
related_records:
  - docs/plans/astra/features/runtime/661-text-image-owner-and-contract-repair.md
  - docs/plans/astra/features/runtime/662-grapheme-projection-monotonic-cursor.md
  - docs/plans/astra/features/runtime/663-kinsoku-fixed-set-dispatch.md
  - docs/plans/astra/features/runtime/664-streaming-tab-layout.md
  - docs/plans/astra/features/runtime/665-navigation-rebuild-candidate-stream.md
  - docs/plans/astra/features/runtime/666-navigation-group-candidate-single-pass.md
  - docs/plans/astra/features/runtime/667-popup-projection-stack-capacity.md
  - docs/plans/astra/features/runtime/668-incremental-layout-report-capacity.md
  - docs/plans/astra/features/runtime/669-render-extract-command-capacity.md
  - docs/plans/astra/features/runtime/670-fixed-control-command-capacity.md
  - docs/plans/astra/features/runtime/671-render-cache-popup-transient-allocation.md
  - docs/plans/astra/features/runtime/672-single-node-pseudo-state-fast-path.md
  - docs/plans/astra/features/runtime/673-modal-restore-scratch-streaming.md
  - docs/plans/astra/features/runtime/674-feedback-overlay-dialog-command-capacity.md
  - docs/plans/astra/features/runtime/675-layout-constraint-workspace-capacity.md
  - docs/plans/astra/features/runtime/676-incremental-layout-root-capacity.md
  - docs/plans/astra/features/runtime/677-pointer-component-event-route-capacity.md
  - docs/plans/astra/features/runtime/678-navigation-dispatch-invocation-capacity.md
  - docs/plans/astra/features/runtime/681-hit-test-stacked-output-capacity.md
  - docs/plans/astra/features/runtime/686-control-index-hash-directories.md
  - docs/plans/astra/features/runtime/687-hit-grid-cell-batch-compile-repair.md
  - docs/plans/astra/features/runtime/688-runtime-ui-test-import-api-compile-repair.md
  - docs/plans/astra/features/runtime/689-runtime-ui-text-test-helper-compile-repair.md
  - docs/plans/astra/features/runtime/690-runtime-performance-benchmark-compile-repair.md
  - docs/plans/astra/features/runtime/691-runtime-ui-text-test-api-ownership-compile-repair.md
  - docs/plans/astra/features/runtime/692-runtime-accessibility-test-contract-compile-repair.md
  - docs/plans/astra/features/runtime/693-runtime-ui-test-dto-and-assertion-compile-repair.md
  - docs/plans/astra/features/runtime/697-runtime-ui-layout-slot-accessor-compile-repair.md
  - docs/plans/astra/features/runtime/698-runtime-ui-submission-segment-test-compile-repair.md
  - docs/plans/astra/features/runtime/699-runtime-ui-layout-report-arc-assertion-compile-repair.md
  - docs/plans/astra/features/runtime/700-runtime-ui-route-sharing-hover-completion.md
  - docs/plans/astra/features/runtime/701-runtime-text-wrapping-outcome-test-repair.md
  - docs/plans/astra/features/runtime/702-runtime-ui-residual-test-api-compile-repairs.md
  - docs/plans/astra/features/runtime/703-runtime-render-text-test-api-compile-repairs.md
  - docs/plans/astra/features/runtime/705-hit-route-publication-scratch-reuse.md
  - docs/plans/astra/features/runtime/706-accessibility-visibility-detached-scratch-reuse.md
  - docs/plans/astra/features/runtime/707-notification-keyboard-filter-borrowing.md
  - docs/plans/astra/features/runtime/708-accessibility-relation-target-retain.md
  - docs/plans/astra/features/runtime/709-accessibility-resolution-key-scratch-reuse.md
  - docs/plans/astra/features/runtime/710-accessibility-description-reference-borrow.md
  - docs/plans/astra/features/runtime/711-accessibility-filter-map-membership.md
  - docs/plans/astra/features/runtime/712-ui-tree-dirty-index-ownership.md
  - docs/plans/astra/features/runtime/714-text-decoration-range-fallback.md
  - docs/plans/astra/features/runtime/715-rich-table-shrink-and-track-metrics.md
  - docs/plans/astra/features/runtime/717-inline-widget-arrangement-capacity.md
  - docs/plans/astra/features/runtime/720-ecs-projection-node-capacity.md
  - docs/plans/astra/features/runtime/721-host-font-borrowed-dedup.md
  - docs/plans/astra/features/runtime/723-runtime-index-output-capacity.md
  - docs/plans/astra/features/runtime/725-runtime-visibility-single-pass-publication.md
  - docs/plans/astra/features/runtime/726-runtime-hit-route-parent-index-reuse.md
  - docs/plans/astra/features/runtime/727-runtime-hit-grid-entry-capacity.md
  - docs/plans/astra/features/runtime/728-runtime-pointer-state-single-node-accumulator.md
  - docs/plans/astra/features/runtime/729-runtime-hit-grid-cell-iterator.md
  - docs/plans/astra/features/runtime/730-runtime-hit-grid-reverse-map-reuse.md
  - docs/plans/astra/features/runtime/731-navigation-position-map-reuse.md
  - docs/plans/astra/features/runtime/732-navigation-group-key-reuse.md
  - docs/plans/astra/features/runtime/733-navigation-first-group-candidate.md
  - docs/plans/astra/features/runtime/734-navigation-candidate-bucket-reuse.md
  - docs/plans/astra/features/runtime/735-navigation-first-group-key-retention.md
  - docs/plans/astra/features/runtime/736-virtual-list-reconciliation-scratch-reuse.md
  - docs/plans/astra/features/runtime/737-node-pool-owned-key-lookup.md
  - docs/plans/astra/features/runtime/738-reflection-node-index-hash-lookup.md
  - docs/plans/astra/features/runtime/739-dispatch-handler-hash-lookup.md
  - docs/plans/astra/features/runtime/740-active-pointer-table-index.md
  - docs/plans/astra/features/runtime/741-registry-iterator-streaming.md
  - docs/plans/astra/features/runtime/742-dynamic-pointer-state-hash.md
  - docs/plans/astra/features/runtime/743-runtime206-contract-repair.md
  - docs/plans/astra/features/runtime/744-runtime206-secondary-query-postings.md
  - docs/plans/astra/features/runtime/745-runtime206-referencer-binary-sort.md
  - docs/plans/astra/features/runtime/746-runtime206-registry-build-capacity.md
  - docs/plans/astra/features/runtime/747-runtime-naming-boundary-contract-repair.md
  - docs/plans/astra/features/runtime/748-runtime-hover-diff-membership-scratch.md
  - docs/plans/astra/features/runtime/749-runtime-primary-pointer-source-counter.md
  - docs/plans/astra/features/runtime/752-input-effect-result-capacity.md
  - docs/plans/astra/features/runtime/753-pointer-reply-capacity.md
  - docs/plans/astra/features/runtime/754-secure-text-presentation-capacity.md
  - docs/plans/astra/features/runtime/755-command-palette-filtered-capacity.md
  - docs/plans/astra/features/runtime/756-command-palette-entry-streaming.md
  - docs/plans/astra/features/runtime/757-tree-id-collection-capacity.md
  - docs/plans/astra/features/runtime/758-keyboard-option-entry-streaming.md
  - docs/plans/astra/features/runtime/759-keyboard-option-id-streaming.md
  - docs/plans/astra/features/runtime/763-menu-search-projection-capacity.md
  - docs/plans/astra/features/runtime/764-menu-search-streaming.md
  - docs/plans/astra/features/runtime/765-menu-typeahead-option-id-borrow.md
  - docs/plans/astra/features/runtime/766-keyboard-indexed-entry-streaming.md
  - docs/plans/astra/features/runtime/767-text-input-property-borrow.md
  - docs/plans/astra/features/runtime/768-text-input-timing-normalization.md
  - docs/plans/astra/features/runtime/769-selection-flags-capacity.md
  - docs/plans/astra/features/runtime/770-collection-single-resolution.md
  - docs/plans/astra/features/runtime/771-table-borrowed-sort-setting.md
  - docs/plans/astra/features/runtime/772-toast-borrowed-setting.md
  - docs/plans/astra/features/runtime/773-text-search-streaming-prefix.md
  - docs/plans/astra/features/runtime/774-text-search-streaming-contains.md
  - docs/plans/astra/features/runtime/775-menu-typeahead-text-normalization.md
  - docs/plans/astra/features/runtime/776-menu-typeahead-append-reuse.md
  - docs/plans/astra/features/runtime/777-menu-typeahead-search-projection.md
  - docs/plans/astra/features/runtime/778-menu-search-query-borrow.md
  - docs/plans/astra/features/runtime/779-menu-search-filter-accumulator.md
  - docs/plans/astra/features/runtime/780-menu-label-borrow.md
  - docs/plans/astra/features/runtime/781-menu-child-values-iterator.md
  - docs/plans/astra/features/runtime/782-surface-tree-interaction-shared-catalog.md
  - docs/plans/astra/features/runtime/783-overlay-static-key-update.md
  - docs/plans/astra/features/runtime/784-notification-static-key-update.md
  - docs/plans/astra/features/runtime/785-v2-style-rule-capacity.md
  - docs/plans/astra/features/runtime/786-runtime206-incremental-publication-capacity.md
  - docs/plans/astra/features/runtime/788-tree-selection-output-capacity.md
  - docs/plans/astra/features/runtime/790-runtime75-shared-component-catalog-view.md
  - docs/plans/astra/features/runtime/791-focus-navigation-capacity.md
  - docs/plans/astra/features/runtime/792-surface-index-target-capacity.md
  - docs/plans/astra/features/runtime/793-hot-reload-eviction-capacity.md
  - docs/plans/astra/features/runtime/794-hot-reload-template-assets-capacity.md
  - docs/plans/astra/features/runtime/795-ui-resource-reference-streaming-visitor.md
  - docs/plans/astra/features/runtime/796-compile-cache-hash-eviction.md
  - docs/plans/astra/features/runtime/797-compile-cache-eviction-key-capacity.md
  - docs/plans/astra/features/runtime/799-random-selector-compiled-weight-table.md
  - docs/plans/astra/features/runtime/800-package-feature-definition-streaming.md
  - docs/plans/astra/features/runtime/845-style-plan-rule-capacity.md
  - docs/plans/astra/features/runtime/801-terminal-selector-single-candidate.md
  - docs/plans/astra/features/runtime/802-bridge-dependency-scratch-capacity.md
  - docs/plans/astra/features/runtime/803-runtime74-benchmark-evidence-hardening.md
  - docs/plans/astra/features/runtime/804-dispatch-output-empty-fast-path.md
  - docs/plans/astra/features/runtime/807-action-revocation-scratch-capacity.md
  - docs/plans/astra/features/runtime/809-accesskit-tree-projection-capacity.md
  - docs/plans/astra/features/runtime/810-v2-pseudo-state-capacity.md
  - docs/plans/astra/features/runtime/819-accessibility-diagnostic-index-dedup.md
  - docs/plans/astra/features/runtime/820-dispatch-host-request-capacity.md
  - docs/plans/astra/features/runtime/821-focus-hovered-retain-capacity.md
  - docs/plans/astra/features/runtime/822-focus-pointer-drag-retain-capacity.md
  - docs/plans/astra/features/runtime/824-input-timer-drain-capacity.md
  - docs/plans/astra/features/runtime/829-dependency-cascade-target-capacity.md
  - docs/plans/astra/features/runtime/833-surface-frame-patch-range-capacity.md
  - docs/plans/astra/features/runtime/835-virtual-geometry-overlay-capacity.md
  - docs/plans/astra/features/runtime/841-runtime-pseudo-state-capacity.md
  - docs/plans/astra/features/runtime/842-tree-view-metadata-collection-capacity.md
  - docs/plans/astra/features/runtime/843-ui-node-resource-registration-capacity.md
  - docs/plans/astra/features/runtime/846-navigation-projection-capacity.md
  - docs/plans/astra/features/runtime/847-particle-extract-output-capacity.md
  - docs/plans/astra/features/runtime/851-style-import-resolution-capacity.md
  - docs/plans/astra/features/runtime/853-v2-style-rule-filter-retain.md
  - docs/plans/astra/features/runtime/860-selector-candidate-capacity.md
  - docs/plans/astra/features/runtime/861-navigation-path-dedup-in-place.md
  - docs/plans/astra/features/runtime/862-navigation-vertex-projection-capacity.md
  - docs/plans/astra/features/runtime/863-ui-v2-direct-reference-convergence.md
  - docs/plans/astra/features/runtime/864-handwritten-dependency-move-append.md
  - docs/plans/astra/features/runtime/865-borrowed-handwritten-meta-dependency-index.md
  - docs/plans/astra/features/runtime/866-resolved-dependency-output-capacity.md
---

# Runtime UI Optimization Batch Completion List

This ledger groups the current Runtime UI optimization slices selected from
the optimize plans. The latest Runtime206 postings/build follow-ups and the
Runtime naming-boundary repair are included alongside the earlier UI slices.
They are deliberately local ownership/capacity changes:
navigation and popup projection avoid redundant work, layout/render/input
paths reuse known bounds or scratch, and text helpers keep their bounded fast
paths. Semantics, ordering, fallback behavior, and public DTO contracts were
left unchanged.

## Latest completion entries

| Work | Status | Local evidence |
| --- | --- | --- |
| Runtime11A/Runtime81 inline-widget arrangement capacity admission plus Runtime11A/Runtime200 route/visibility publication, pointer-state fast path, hit-grid cell/reverse-map reuse, navigation position-map/key/candidate reuse, virtual-list reconciliation scratch reuse, node-pool owned-key lookup, reflection-index hash lookup, and pointer/navigation handler hash lookup | implemented_pending_validation | Direct-child, managed-binding, and preorder scratch containers reserve their known lower bounds; route/visibility publication reserves authoritative output counts, projects the visibility map in one pass, reuses the full-build parent index during route composition, reserves/reuses the draw-order hit-entry upper bound/index, keeps single-node pointer-state bookkeeping scalar until a real batch is needed, lazily streams bounded cell memberships during full grid rebuilds, retains stable reverse-map vectors, uses reusable lookup-only `HashMap` buckets for tab positions while sorted candidates remain authoritative, keeps per-owner virtual-list candidate slot/key/generation buffers across warm scroll reconciliation, moves desired node-pool key fields into a temporary lookup key instead of cloning them on retained-child insertion while report residency uses one bucket walk, uses a capacity-retaining `HashMap` for lookup-only reflection node paths while ordered tree/node traversal preserves duplicate-path precedence, and uses exact-key `HashMap` handler tables while preserving per-key registration order. Stable navigation groups now probe existing buckets before cloning their owned `String` key, first group targets are reduced in the primary node stream without a temporary candidate map, and Runtime UI prototype preparation streams the canonical registry iterator without an intermediate entry vector. The latest UI loader loaded 173 modules and passed `776/776` tests in `1.962s`; the broader non-tooling Runtime/Editor performance-plus-pressure batch loaded 548 modules and passed `2057/2057` in `15.174s`; the focused Runtime UI node-pool/reflection/layout batch passed `30/30`; the latest Runtime dispatch batch passed `492/492` across 113 modules and the Editor asset/reference batch passed `89/89` across 18 modules; the current combined Runtime UI plus Editor asset contract batch passed `620/620` in `15.422s`, with the focused ten-module slice passing `37/37` in `0.039s`; the broader Runtime/Editor performance-contract and pressure discovery passes `1358/1358` in `9.465s`; scoped py_compile, rustfmt, diff checks, and wiki validation also pass. Managed Cargo and Release allocation/latency evidence remain pending. |
| Runtime752–755 capacity follow-ups | implemented_pending_validation | Effect-result, pointer-reply, secure-text presentation, and command-palette filtered-ID projections now reserve known bounds. Runtime755's adjacent Runtime/Editor command-palette and capacity invocation passes `34/34` in `0.146s`; the prior broad `1852/1852` receipt predates Runtime755. Managed Cargo/Release allocation and product percentile evidence remain pending. |
| Runtime755–756 command-palette parse/filter follow-ups | implemented_pending_validation | Filtered IDs reserve the parsed-entry bound, and nested declaration parsing streams leaves into one root output vector instead of recursive temporary vectors. The combined adjacent Runtime/Editor command-palette and capacity invocation passes `38/38` in `0.427s`; the prior broad `1852/1852` receipt predates both slices. Managed Cargo/Release allocation and product percentile evidence remain pending. |
| Runtime757 TreeView ID collection capacity | implemented_pending_validation | Ordered node, borrowed/owned string, and disabled-option ID collectors reserve each known direct array/flags bound before recursion, without a second pre-scan. The adjacent Runtime/Editor contract batch passes `42/42` in `0.314s`; managed Cargo/Release allocation and product percentile evidence remain pending. |
| Runtime758 keyboard option-entry streaming | implemented_pending_validation | Keyboard option declarations flatten through one capacity-hinted output vector, avoiding recursive array `Vec<OptionEntry>` temporaries while retaining map/string/enum semantics. The adjacent Runtime/Editor contract batch passes `46/46` in `0.266s`; managed Cargo/Release allocation and product percentile evidence remain pending. |
| Runtime759 keyboard option-ID streaming | implemented_pending_validation | Keyboard map/array IDs flatten through one capacity-hinted collector with direct-array reservations, preserving `id`/`value` precedence and empty rules. The adjacent Runtime/Editor contract batch passes `50/50` in `0.131s`; managed Cargo/Release allocation and product percentile evidence remain pending. |
| Runtime763–764 menu-search capacity and streaming | implemented_pending_validation | Filtered-ID and preorder projections reserve direct bounds, while recursive top-level/descendant option construction streams into retained owner vectors without temporary `Vec<MenuSearchOption>` chains. The adjacent Runtime/Editor contract batch passes `58/58` in `0.110s`; managed Cargo/Release allocation and product percentile evidence remain pending. |
| Runtime765 menu typeahead option-ID borrow | implemented_pending_validation | Typeahead resolves the current index against the parsed `OptionEntry` slice instead of cloning every ID into a transient vector. The adjacent Runtime/Editor contract batch passes `62/62` in `0.061s`; managed Cargo/Release allocation and product percentile evidence remain pending. |
| Runtime766 indexed keyboard-entry streaming | implemented_pending_validation | Indexed keyboard navigation moves nonempty IDs directly into one retained vector instead of materializing and filtering a complete intermediate ID vector. The adjacent Runtime/Editor contract batch passes `65/65` in `0.380s`; managed Cargo/Release allocation and product percentile evidence remain pending. |
| Runtime767 TextInput property borrow | implemented_pending_validation | TextInput validation uses static candidate/mirror slices and borrows the selected text instead of allocating finite property vectors and cloning the current value. The adjacent Runtime/Editor contract batch passes `69/69` in `0.577s`; managed Cargo/Release allocation and product percentile evidence remain pending. |
| Runtime768 TextInput timing normalization | implemented_pending_validation | Timing lookup borrows its textual setting and streams alias normalization instead of cloning the setting and materializing a normalized String. The adjacent Runtime/Editor contract batch passes `73/73` in `0.889s`; managed Cargo/Release allocation and product percentile evidence remain pending. |
| Runtime769 selection Flags array capacity | implemented_pending_validation | Array-to-Flags normalization reserves its direct input bound before retaining nonempty textual values, avoiding geometric output growth. The adjacent Runtime/Editor contract batch passes `77/77` in `0.462s`; managed Cargo/Release allocation and product percentile evidence remain pending. |
| Runtime770 collection single state resolution | implemented_pending_validation | Array/map mutations resolve their state container once through validation and mutation; map add/set avoid a separate success-path tree lookup. The adjacent Runtime/Editor contract batch passes `82/82` in `0.829s`; managed Cargo/Release allocation and product percentile evidence remain pending. |
| Runtime771 table borrowed sort setting | implemented_pending_validation | Table sort/column-width read paths borrow event payloads and comparison/mode settings; the sort-direction transition retains its necessary current-column clone before same-map mutation. The current combined Runtime/Editor contract batch passes `120/120` in `0.080s`; managed Cargo/Release allocation and product percentile evidence remain pending. |
| Runtime772 Toast borrowed setting | implemented_pending_validation | Toast queue scans and presence checks borrow current/authored textual settings, owning only the identifier needed at state publication. The current combined Runtime/Editor contract batch passes `120/120` in `0.080s`; managed Cargo/Release allocation and product percentile evidence remain pending. |
| Runtime773 text-search streaming Unicode prefix | implemented_pending_validation | Shared non-ASCII prefix matching consumes Unicode lowercase scalars lazily instead of allocating a full candidate lowercase string; ASCII and query semantics remain unchanged. The focused source contract passes `4/4` and current combined Runtime/Editor batch passes `120/120` in `0.080s`; managed Cargo/Release allocation and product percentile evidence remain pending. |
| Runtime774 text-search streaming Unicode contains | implemented_pending_validation | Shared non-ASCII contains matching scans Unicode lowercase scalar streams with cloned iterator state for suffix checks rather than allocating a full candidate lowercase string; trimming, expansion, and overlap semantics remain unchanged. The focused prefix/contains/shared-search source batch passes `14/14` and current combined Runtime/Editor batch passes `120/120` in `0.080s`; managed Cargo/Release allocation and product percentile evidence remain pending. |
| Runtime775 menu typeahead text normalization | implemented_pending_validation | Typeahead builds its final filtered/trimmed Unicode lowercase search buffer once, retaining only a trailing-whitespace offset instead of an intermediate filtered string. The focused menu/typeahead/search batch passes `21/21` and the current combined Runtime/Editor batch passes `120/120` in `0.080s`; managed Cargo/Release allocation and product percentile evidence remain pending. |
| Runtime776 menu typeahead append reuse | implemented_pending_validation | Typeahead moves the active normalized buffer and appends the key payload, avoiding the separate combined `format!` result when capacity permits. The focused menu/typeahead/search batch passes `18/18` in `0.041s` and the current combined Runtime/Editor batch passes `120/120` in `0.080s`; managed Cargo/Release allocation and product percentile evidence remain pending. |
| Runtime777 menu typeahead search projection | implemented_pending_validation | Typeahead candidates retain one buffer for matching and publication, and multi-scalar detection exits after the second Unicode scalar instead of counting the complete input. The source contract passes `4/4`; lower pasted/active/fallback/Unicode-expansion coverage and `RUNTIME777_MENU_TYPEAHEAD_SEARCH_PROJECTION_BENCH_V1` join the deferred managed gate. The current combined Runtime/Editor batch passes `120/120` in `0.080s`; managed Cargo/Release allocation and product percentile evidence remain pending. |
| Runtime778 menu search query borrow | implemented_pending_validation | Menu search-filter normalization borrows the selected query setting, trims before allocating lowercase output, and preserves state/default precedence. The source contract passes `2/2`; lower state/default/whitespace coverage and `RUNTIME778_MENU_SEARCH_QUERY_BORROW_BENCH_V1` join the deferred managed gate. The current combined Runtime/Editor batch passes `120/120` in `0.080s`; managed Cargo/Release allocation and product percentile evidence remain pending. |
| Runtime779 menu search filter accumulator | implemented_pending_validation | Recursive menu filtering streams into one caller-owned accumulator with checkpoint rollback, removing per-node child result vectors while preserving preorder and focus order. The source contract passes `3/3`; lower unmatched-branch/order coverage and `RUNTIME779_MENU_SEARCH_FILTER_ACCUMULATOR_BENCH_V1` join the deferred managed gate. The current combined Runtime/Editor batch passes `120/120` in `0.080s`; managed Cargo/Release allocation and product percentile evidence remain pending. |
| Runtime780 menu label borrow | implemented_pending_validation | Menu search-tree label lookup borrows the source map value and clones only at retained-node ownership boundaries, preserving multi-ID labels and empty-label ID fallback. The source contract passes `3/3`; lower label/fallback coverage and `RUNTIME780_MENU_LABEL_BORROW_BENCH_V1` join the deferred managed gate. The current combined Runtime/Editor batch passes `120/120` in `0.080s`; managed Cargo/Release allocation and product percentile evidence remain pending. |
| Runtime781 menu child-values iterator | implemented_pending_validation | Menu search-tree child traversal streams the fixed child-property lookup sequence directly, removing one temporary `Vec<&UiValue>` per map node while preserving property order and descendant behavior. The source contract passes `3/3`; lower property-order coverage and `RUNTIME781_MENU_CHILD_VALUES_ITERATOR_BENCH_V1` join the deferred managed gate. The current combined Runtime/Editor batch passes `120/120` in `0.080s`; managed Cargo/Release allocation and product percentile evidence remain pending. |
| Runtime782 v2 surface-tree interaction shared catalog | implemented_pending_validation | v2 interaction capability inference now borrows `editor_showcase_shared()` directly instead of retaining a private owned 69-descriptor `OnceLock` clone. The source contract passes `3/3`; lower pointer-identity coverage and `RUNTIME782_SURFACE_TREE_SHARED_CATALOG_BENCH_V1` join the deferred managed gate. Managed Cargo/Release allocation and product interaction percentile evidence remain pending. |
| Runtime783 overlay static-key update | implemented_pending_validation | Popup/dialog reducers update existing fixed keys in place and allocate owned keys only on first insertion. The source contract passes `3/3`; lower key-identity/reference-source coverage and `RUNTIME783_OVERLAY_STATIC_KEY_UPDATE_BENCH_V1` join the deferred managed gate. Managed Cargo/Release allocation and product popup percentile evidence remain pending. |
| Runtime784 notification static-key update | implemented_pending_validation | Notification selection, focus, and unread-count publication update existing fixed keys in place and allocate owned keys only on first insertion. The source contract passes `3/3`; the current-source loader passes `1951/1951` across `544` modules and the merged focused hot-path set passes `145/145` in `0.150s`. Lower key-identity/reference-source coverage and `RUNTIME784_NOTIFICATION_STATIC_KEY_UPDATE_BENCH_V1` join the deferred managed gate; managed Cargo/Release allocation and product notification percentile evidence remain pending. |
| Runtime785 v2 style rule capacity | implemented_pending_validation | The v2 style resolver reserves the summed authored stylesheet rule bound before selector parsing and specificity sorting, preserving order and invalid-selector semantics. The source contract passes `3/3`; the current non-tooling core batch passes `1963/1963` across `549` modules in `14.682s`; managed Cargo/Release allocation and product style percentile evidence remain pending. |
| Runtime786 Runtime206 incremental publication capacity | implemented_pending_validation | Incremental project-resource publication reserves changed/previous record bounds for four temporary map/vector/set projections while preserving deduplication and filtering. The source contract passes `3/3`; the lower regression and `RUNTIME786_INCREMENTAL_PUBLICATION_CAPACITY_BENCH_V1` marker join the deferred managed gate. |
| Runtime788 TreeView selection output capacity | implemented_pending_validation | The indexed reducer reserves the TreeIndex-owned selected count before cloning ordered selected IDs, preserving toggle and source-order semantics. The source contract passes `3/3`; the lower count/order regression and `RUNTIME788_TREE_SELECTION_OUTPUT_CAPACITY_BENCH_V1` marker join the refreshed `552`-module/`1975/1975` Runtime/Editor local batch and `3724/3724` broader non-tooling regression. Managed Cargo/Release allocation and TreeView percentile evidence remain pending. |
| Runtime790 shared component catalog view | implemented_pending_validation | The default UI document compiler borrows the process-shared showcase catalog, while custom registries remain owned and the Editor retained union keeps Material precedence. Pointer-identity/union regressions and ignored `RUNTIME75_SHARED_COMPONENT_REGISTRY_BENCH_V1` are wired; managed Cargo/Release and product percentile evidence remain pending. |
| Runtime791 focus navigation output capacity | implemented_pending_validation | Focus navigation reserves the `UiTree.nodes` upper bound before recursive candidate collection, preserving enabled/visibility/order semantics. The TDD source contract passes `3/3`; the lower order/capacity regression and `RUNTIME791_FOCUS_NAVIGATION_CAPACITY_BENCH_V1` marker join the deferred managed lane. The refreshed 553-file/`1978/1978` Runtime/Editor local contract batch is green. The 4,096-node deterministic model removes 11 modeled growth events; managed Cargo/Release allocation and navigation percentile evidence remain pending. |
| Runtime792 surface-index target projection capacity | implemented_pending_validation | Surface and node hot-reload target projections reserve the saturating sum of their four category input lengths before borrowed-set deduplication, preserving first-seen order and duplicate semantics. The TDD source contract passes `3/3`; lower surface/node capacity regressions and `RUNTIME792_SURFACE_INDEX_TARGET_CAPACITY_BENCH_V1` join the deferred managed lane. The 16,384-target deterministic model removes 12 modeled growth events. The refreshed explicit-file Runtime/Editor performance-contract loader passes `1981/1981` across `554` files in `30.200s`; managed Cargo/Release allocation and hot-reload percentile evidence remain pending. |
| Runtime793 hot-reload compile-cache eviction capacity | implemented_pending_validation | Compile-cache eviction reserves the saturating sum of template-rebuild and removed-compiled target lengths before the existing borrowed extends. The TDD source contract passes `3/3`; lower empty/ordinary/overflow regression and `RUNTIME793_HOT_RELOAD_EVICTION_CAPACITY_BENCH_V1` join the deferred managed lane. The 16,384-target deterministic model removes 13 modeled growth events; managed Cargo/Release allocation and hot-reload percentile evidence remain pending. |
| Runtime794 hot-reload template-asset projection capacity | implemented_pending_validation | Per-surface template-asset filtering reserves the saturating sum of the two materialized target vectors before the existing chained filter/cloned extend. The TDD source contract passes `3/3`; lower empty/ordinary/overflow regression and `RUNTIME794_HOT_RELOAD_TEMPLATE_ASSETS_CAPACITY_BENCH_V1` join the deferred managed lane. The 16,384-target deterministic model removes 13 modeled growth events; managed Cargo/Release allocation and hot-reload percentile evidence remain pending. |
| Runtime795 UI resource-reference streaming visitor | implemented_pending_validation | UI asset resource references stream borrowed URIs through one callback instead of a temporary `Vec<&str>`, preserving traversal order, first-seen deduplication, and zero URI string clones. The source contract passes `3/3`; the ignored marker now samples 101 alternating pairs and reports the 51/50 first-run split; the current combined Runtime/Editor source-contract loader passes `2002/2002` across `561` files; lower order/ownership regressions and `RUNTIME74_UI_RESOURCE_REFERENCE_VISITOR_BENCH_V1` join the deferred managed lane. |
| Runtime796 compile-cache borrowed hash eviction | implemented_pending_validation | Compile-cache eviction uses borrowed `HashSet<&str>` membership while ordered cache/snapshot maps remain authoritative. The source contract passes `3/3`; the ignored marker now samples 101 alternating runs and reports `sample_count=101`; the current combined Runtime/Editor source-contract loader passes `2002/2002` across `561` files; lower multi-asset semantics regression and `RUNTIME74_COMPILE_CACHE_HASH_EVICTION_BENCH_V1` join the deferred managed lane. |
| Runtime797 compile-cache eviction-key capacity | implemented_pending_validation | Both temporary removal-key vectors reserve the unique requested asset-ID lower bound, preserving sparse-eviction behavior and ordered map authority. The source contract passes `3/3`; the current combined Runtime/Editor source-contract loader passes `2002/2002` across `561` files; lower two-collector regression and `RUNTIME797_COMPILE_CACHE_EVICTION_KEY_CAPACITY_BENCH_V1` join the deferred managed lane. |
| Runtime799 random-selector compiled weight table | implemented_pending_validation | Behavior-tree random-selector weights are resolved once during compilation and borrowed during ticks, removing the per-selection weight-vector allocation and parameter scans while preserving ID-first/positional fallback semantics. The v8 `f32 < &f32` failure in the extracted borrowed-slice selector is repaired with explicit dereference and locked by the `4/4` source contract; lower child-order/precedence/clamping coverage, `RUNTIME799_RANDOM_SELECTOR_WEIGHT_TABLE_BENCH_V1`, and current-source compile confirmation join the next managed lane. |
| Runtime800 package feature-definition streaming | implemented_pending_validation | Package feature definitions are consumed through an optional-first visitor, removing one temporary vector per registered package; the aggregate declaration bound is reserved before merge. The Runtime200/205 eligibility contract is green at `8/8`, with lower carrier-capacity coverage and `320→64` deterministic slot reduction. The v5 `E0364`/`E0603` helper-visibility failure is repaired and v6 Runtime builds at exit `0`; lower tests, `RUNTIME800_PACKAGE_FEATURE_DEFINITION_STREAMING_BENCH_V1`, Release, allocator, and product gates remain pending. |
| Runtime802 bridge dependency traversal scratch capacity | implemented_pending_validation | Bridge diagnostics retain one package-bound DFS visiting set for all roots and reserve always-populated graph indexes, while issue-only buffers remain lazy until reachability finds an issue. The TDD source contract passes `3/3`; lower cyclic scratch-clear/capacity coverage is wired. The 1,024-root deterministic model changes visiting-set allocation count from `1,024` to `1`; managed Cargo/Release and product catalog-build percentile evidence remain pending. |
| Runtime803 Runtime74 benchmark evidence hardening | implemented_pending_validation | Six ignored Runtime74 probes now use 101 alternating samples with balanced 51/50 first-order metadata and explicit P50/P95/P99 marker fields. The cross-file source contract passes `2/2`, and exact-file Rustfmt passes for all six owners; production algorithms and threshold assertions are unchanged. Managed Cargo/Release, complete-caller, allocator, and product percentile evidence remain pending. |
| Runtime804 empty dispatch-output fast path | implemented_pending_validation | `RuntimeUiSurfaceSet::record_dispatch_outputs` now returns before surface lookup, owned `UiTreeId` cloning, and both empty queue walks when host/component outputs are absent; non-empty request and secure-text revocation behavior is unchanged. The TDD source/model contract passes `4/4`; the merged non-tooling batch passes `3665/3665` across `870` files in `40.083s`. Managed Cargo/Release allocation and product input percentile evidence remain pending. |
| Runtime807 action revocation scratch capacity | implemented_pending_validation | `RuntimeUiActionRequestQueue::record_result` keeps the secure-revocation scratch vector at zero capacity until a real revoke, then reserves the remaining component-event bound once before appending. Rejection, redaction, supersession, queue limits, and ordering remain unchanged. The TDD source/model contract passes `4/4`; the merged non-tooling batch passes `3677/3677` across `873` files in `205.214s` with zero failures/errors/skips. Managed Cargo/Release allocation and product input percentile evidence remain pending. |

| Runtime808 font-admission dependency projection | implemented_pending_validation | Cross-surface font dependencies now use one sorted/deduplicated contiguous vector instead of a temporary `BTreeSet`, preserving lexical order, claim replacement, prepared admissions, and profile counters. TDD source/model contract `4/4`; lower source regression and ignored `RUNTIME808_FONT_ADMISSION_DEPENDENCY_BUFFER_BENCH_V1` marker are wired. The combined eight-slice focused batch passes `32/32`, and the current merged non-tooling loader passes `3693/3693` across `877` files in `83.085s` with zero failures, errors, or skips. Managed Cargo/Release and font-admission product percentile evidence remain pending. |
| Runtime809 AccessKit tree projection capacity | implemented_pending_validation | AccessKit snapshot projection reserves the snapshot-node bound (including an optional synthetic root) and each direct root/child bound before appending, preserving node/root/child order, focus fallback, and all AccessKit mappings. TDD source/model contract `4/4`; lower source regression and ignored `RUNTIME809_ACCESSKIT_TREE_PROJECTION_CAPACITY_BENCH_V1` marker are wired. The twelve-slice focused batch passes `48/48`; the strict non-tooling performance/pressure batch passes `2643/2643` across `682` files in `51.429s`; managed Cargo/Release allocation and AccessKit product percentile evidence remain pending. |
| Runtime810 v2 pseudo-state collector capacity | implemented_pending_validation | Static arena-node pseudo-state projection reserves the authored props/state map bound plus painter allowance before alias collection, preserving sorted/deduplicated state names and resolved painter aliases. TDD source/model contract `3/3`; lower alias/order/capacity regression and ignored `RUNTIME810_V2_PSEUDO_STATE_CAPACITY_BENCH_V1` marker are wired; deterministic `4,097`-entry model removes `12→0` growth events. Managed Cargo/Release allocation and style product percentile evidence remain pending. |
| Runtime819 accessibility diagnostic node-index deduplication | implemented_pending_validation | Accessibility snapshot validation now uses one `BTreeMap::entry` index for duplicate-ID admission and first-node lookup, removing the parallel `BTreeSet` while preserving duplicate diagnostics, callback counts, relation/cycle checks, focus fallback, and order. TDD source/model contract `3/3`; lower duplicate/order regression and ignored `RUNTIME819_A11Y_DIAGNOSTIC_INDEX_BENCH_V1` marker are wired; deterministic 4,096-node model changes auxiliary index allocations `2→1`. Managed Cargo/Release allocation and accessibility product percentile evidence remain pending. |
| Runtime820 dispatch host-request capacity | implemented_pending_validation | Runtime177's one-pass dispatch metadata collector lazily reserves the remaining-result lower bound on the first host request, keeping request-free batches at zero capacity and preserving request order/redraw semantics. TDD source/model contract `3/3`; lower empty/order/sparse regression and ignored `RUNTIME820_DISPATCH_HOST_REQUEST_CAPACITY_BENCH_V1` marker are wired; deterministic 4,096-result model changes `11→0` growth events with one reservation. Managed Cargo/Release allocation and dispatch product percentile evidence remain pending. |
| Runtime821 focus hovered-path in-place retention | implemented_pending_validation | Runtime200 focus cleanup now moves the existing hovered path with `mem::take` and filters it in place with `retain`, removing the replacement `Vec` allocation while preserving input-owner validation, order, duplicates, and empty behavior. TDD source/model contract `4/4`; lower order/capacity regression and ignored `RUNTIME821_FOCUS_HOVERED_RETAIN_BENCH_V1` marker are wired; the one-process Runtime/Editor batch passes `1416/1416` across `393` modules and the broader non-tooling batch passes `2210/2210` across `619` modules with zero failures/errors/skips; the deterministic 4,096-entry model changes one replacement allocation per reconciliation to zero new allocations. Managed Cargo/Release allocation and input product percentile evidence remain pending. |
| Runtime822 focus pointer-drag owner in-place retention | implemented_pending_validation | Runtime200 focus cleanup now moves `input.pointer_drags` out with `mem::take` and filters it in place with `BTreeMap::retain`, removing the temporary invalid-owner `Vec` and per-owner second-pass lookup while preserving valid drag payloads, key order, and input-owner validation. TDD source/model contract `4/4`; lower semantic regression and ignored `RUNTIME822_FOCUS_POINTER_DRAG_RETAIN_BENCH_V1` marker are wired; the current one-process non-tooling Runtime/Editor batch passes `2218/2218` across `621` modules with zero failures/errors/skips; the deterministic 4,096-entry model changes one temporary vector plus repeated invalid-owner removals per reconciliation to zero temporary vectors plus one map traversal. Managed Cargo/Release allocation and input product percentile evidence remain pending. |
| Runtime829 dependency cascade target capacity | implemented_pending_validation | UI asset reverse-dependency invalidation now lazily reserves the current first-wave fanout after the first admitted dependent, while borrowed hash visitation, BTree-ordered BFS publication, duplicate suppression, empty lookups, and self-cycles remain unchanged. TDD source/model contract `4/4`; lower fanout/empty/cycle regression and ignored `RUNTIME829_DEPENDENCY_CASCADE_TARGET_CAPACITY_BENCH_V1` marker are wired; the deterministic 4,096-target model changes `11→0` vector growth events; the latest one-process non-tooling batch passes `2239/2239` across `627` modules in `5.505s` and the nine-slice focused loader passes `33/33`. Managed Cargo/Release allocation and dependency-cascade product percentile evidence remain pending. |
| Runtime833 surface frame patch-range capacity | implemented_pending_validation | Exact changed-node render patch publication now reserves `node_ids.len()` once and extends directly into the temporary range buffer, preserving `None` full-snapshot fallback, empty-set zero capacity, BTree order, command filtering, range merge, and frame authority. TDD source/model contract `2/2`; lower source regression and ignored `RUNTIME833_SURFACE_FRAME_PATCH_RANGE_CAPACITY_BENCH_V1` marker are wired; deterministic 4,096-node model changes `12→0` growth events; the current one-process capacity smoke batch covers `169` files and passes `626/626` tests in `5.379s`. Managed Cargo/Release allocation and surface-frame product percentile evidence remain pending. |
| Runtime835 virtual geometry overlay capacity | implemented_pending_validation | BVH/visbuffer debug gizmo and line collectors now reserve input, per-node (`13`), and fixed-marker (`16`) bounds before direct extension, preserving filtering, parent connectors, colors, order, and empty fallbacks. TDD source/model contract `3/3`; lower source regression and ignored `RUNTIME835_VIRTUAL_GEOMETRY_OVERLAY_CAPACITY_BENCH_V1` marker are wired; deterministic dense models change `15→0`, `11→0`, and `3→0` geometric growth events. The one-process Runtime/Editor capacity/projection batch covers `171` files and passes `632/632` tests in `4.660s`, with zero failures/errors/load errors/skips. Managed Cargo/Release allocation and overlay product percentile evidence remain pending. |
| Runtime838 accessibility root projection capacity | implemented_pending_validation | Accessibility snapshot root projection now reserves `surface.tree.roots.len()` before the existing admission filters, preserving authored order, hidden/missing-root filtering, budget checks, and empty zero-capacity behavior. TDD source/model contract `2/2`; lower source/order regression and ignored `RUNTIME838_ACCESSIBILITY_ROOT_CAPACITY_BENCH_V1` marker are wired; deterministic 4,096-root model changes `11→0` geometric growth events. Managed Cargo/Release allocation and accessibility product p50/p95/p99 evidence remain pending. |
| Runtime839 animation missing-track diagnostic capacity | implemented_pending_validation | Compiled animation sequence diagnostics keep `missing_tracks` zero-capacity for successful compiles and reserve the source track-count bound only on the first missing entity/writer, preserving path order and payloads. TDD source/model contract `2/2`; lower lazy-success/order regression and ignored `RUNTIME839_ANIMATION_MISSING_TRACK_CAPACITY_BENCH_V1` marker are wired; deterministic 4,096-track model changes `11→0` growth events. Managed Cargo/Release allocation and animation product p50/p95/p99 evidence remain pending. |
| Runtime840 transient allocation output capacity | implemented_pending_validation | Render-graph transient lifetime allocation reserves the exact filtered lifetime count before emitting compiled allocations, preserving interval sort, slot reuse, IDs, and empty behavior. TDD source/model contract `2/2`; lower source/order regression and ignored `RUNTIME840_TRANSIENT_ALLOCATION_CAPACITY_BENCH_V1` marker are wired; deterministic 4,096-lifetime model changes `11→0` growth events. The focused batch passes `46/46`; the broad non-tooling loader passes `3907/3907` across `933` modules in `62.574s` with zero failures/errors/load errors/skips. Managed Cargo/Release allocation and render-graph product p50/p95/p99 evidence remain pending. |
| Runtime841 runtime-tree pseudo-state collector capacity | implemented_pending_validation | Runtime-tree pseudo-state projection now reserves the saturating authored-attribute, enabled-flag alias, node-flag, and painter-alias bound before collection, preserving retained-state filtering, sorted/deduplicated aliases, painter-family resolution, and clean-node semantics. TDD source/model contract `4/4`; Runtime810+Runtime841 source batch passes `7/7`; lower order/capacity regression and ignored `RUNTIME841_RUNTIME_PSEUDO_STATE_CAPACITY_BENCH_V1` marker are wired; the dense 4,096-entry model changes `12→0` growth events. Managed Cargo/Release allocation and style product p50/p95/p99 evidence remain pending. |
| Runtime842 TreeView metadata collection capacity | implemented_pending_validation | Default-interaction TreeView TOML node, borrowed/owned option, and disabled-ID collectors reserve each direct array bound before recursion, preserving table alias precedence, first-seen order, duplicate suppression, nested traversal, and range-selection semantics. TDD source/model contract `4/4`; lower nested-order/duplicate/capacity regression and ignored `RUNTIME842_TREE_METADATA_COLLECTION_CAPACITY_BENCH_V1` marker are wired; the deterministic 4,096-value model changes `11→0` growth events. Managed Cargo/Release allocation and TreeView product p50/p95/p99 evidence remain pending. |
| Runtime843 UI node resource registration output capacity | implemented_pending_validation | `UiAssetSurfaceIndex` lazily reserves resource-free node IDs on the first empty projection and seeds each metadata collector with the saturating authored-map key bound, preserving URI scheme/fallback policy, first-seen order, node ownership, and stale-edge removal. TDD source/model contract `4/4`; lower collector-capacity regression and ignored `RUNTIME843_NODE_RESOURCE_REGISTRATION_CAPACITY_BENCH_V1` marker are wired; the dense 4,096-node model changes `11→0` growth events. Managed Cargo/Release allocation and UI asset-registration product p50/p95/p99 evidence remain pending. |
| Runtime845 style-plan rule capacity | implemented_pending_validation | Template asset compilation now folds all resolved stylesheet rule lengths with saturating addition and reserves the `ParsedStyleRule` vector before selector parsing, preserving global order, selector errors, token-map sharing, declaration cloning, and empty-sheet zero capacity. TDD source/model contract `4/4`; lower bounded/empty regression and ignored `RUNTIME845_STYLE_PLAN_RULE_CAPACITY_BENCH_V1` marker are wired; the dense 4,096-rule model changes `11→0` growth events. The focused Runtime/Editor loader passes `86/86` across `22` modules in `0.069s`; the broad non-tooling loader passes `2374/2374` across `649` modules in `5.699s`, with zero load errors/failures/errors/skips. Managed Cargo/Release allocation and style-plan product p50/p95/p99 evidence remain pending. |
| Runtime846 navigation world projection capacity | implemented_pending_validation | Navigation projection reserves the agent-row bound for `agents` and `agent_positions`, then the obstacle-row bound for `obstacles`, before each existing drain. Deserialization filtering, stable ordering, transform fallback, and avoidance-index semantics remain unchanged. TDD source/model contract `4/4`; lower reservation/empty regression and ignored `RUNTIME846_NAVIGATION_PROJECTION_CAPACITY_BENCH_V1` marker are wired; the dense 4,096-row model changes `11→0` growth events per collector. The refreshed six-contract focused batch passes `24/24`; the one-process broad non-tooling loader passes `2382/2382` across `651` modules in `5.334s`, with zero load errors/failures/errors/skips. Managed Cargo/Release allocation and navigation product p50/p95/p99 evidence remain pending. |
| Runtime847 particle extract output capacity | implemented_pending_validation | Particle extraction now reserves the sorted dynamic-component owner bound for `emitters` and `bounds` while keeping sprite fanout lazy. Filtering, sprite sort, bounds values, and GPU-frame aggregation remain unchanged. TDD source/model contract `4/4`; lower reservation/empty regression and ignored `RUNTIME847_PARTICLE_EXTRACT_OUTPUT_CAPACITY_BENCH_V1` marker are wired; the dense 4,096-owner model changes `11→0` growth events per bounded collector. Managed Cargo/Release allocation and particle product p50/p95/p99 evidence remain pending. |
| Runtime851 style import resolution capacity | implemented_pending_validation | UI style resolution now reserves the known document import count before resolving borrowed imported styles, preserving one lookup per import, unknown-import errors, and widget/import/local stylesheet order. TDD source/model contract `4/4`; isolated lower resolution/error-order regression and ignored `RUNTIME851_STYLE_IMPORT_RESOLUTION_CAPACITY_BENCH_V1` marker are wired; the dense 64-import model changes `5→0` growth events. The refreshed eleven-contract Runtime/Editor loader passes `44/44`; the broad non-tooling loader passes `2402/2402` across `656` modules with zero load errors/failures/errors/skips. Managed Cargo/Release allocation and style-resolution product p50/p95/p99 evidence remain pending. |
| Runtime853 V2 style-rule filter retain | implemented_pending_validation | V2 static and runtime style filtering now retains the already compiled `ResolvedRule` table in place, removing the second `filter(...).collect()` buffer while preserving rule order and pseudo-state classification. TDD source/model contract `4/4`; existing V2 style-capacity and pseudo-state contracts join a focused `10/10` batch; lower order/capacity regression and ignored `RUNTIME853_V2_STYLE_RULE_FILTER_RETAIN_BENCH_V1` marker are wired. The deterministic 4,096-rule model changes one filter buffer per build to zero. Managed Cargo/Release, allocator, and style product p50/p95/p99 evidence remain pending. |
| Current shared-worktree regression receipt | implemented_pending_validation | After additional shared Runtime/Editor changes, the one-process non-tooling performance/pressure loader still passes `2402/2402` across `656` modules in `12.365s`, with zero failures, errors, load errors, or skips; this remains local source/model evidence and managed Cargo/Release, allocator, and product p50/p95/p99 gates remain pending. |
| Runtime854 runtime selector path single-buffer | implemented_pending_validation | Runtime V2 selector paths now build directly into one reversible path buffer, removing the intermediate ancestor-ID vector while preserving root-first order, component-state collection, and root-only host matching. TDD source/model contract `4/4`; lower order/host regression and ignored `RUNTIME854_RUNTIME_SELECTOR_PATH_SINGLE_BUFFER_BENCH_V1` marker are wired. The depth-128 model changes one intermediate buffer per build to zero. Managed Cargo/Release, allocator, and selector-style product p50/p95/p99 evidence remain pending. |
| Runtime855 V2 file source capacity | implemented_pending_validation | Runtime V2 file-source BFS now reserves the known `paths.len()` root-input bound for both the root queue and loaded-source output, preserving canonical de-duplication, transitive import order, and empty-input behavior. TDD source/model contract `4/4`; lower queue/source-order regression and ignored `RUNTIME855_V2_FILE_SOURCE_CAPACITY_BENCH_V1` marker are wired. The 4,096-unique-root model changes each bounded collector's growth events `11→0`. Managed Cargo/Release, allocator, and file-source product p50/p95/p99 evidence remain pending. |
| Runtime856 render-view camera filter retain | implemented_pending_validation | Scene-camera view extraction now reuses the capacity-sized descriptor vector with in-place `retain`, preserving sorted order and the selected-inactive-camera exception. TDD source/model contract `4/4`; lower order/empty regression and ignored `RUNTIME856_RENDER_VIEW_CAMERA_FILTER_RETAIN_BENCH_V1` marker are wired. The 8,192-camera model changes modeled growth events `13→0`; managed Cargo/Release, allocator, and render-view product p50/p95/p99 evidence remain pending. |
| Runtime857 animation drain output capacity | implemented_pending_validation | Animation clip-event draining reads the pending count first, keeps no-pending drains at zero capacity, and reserves the bounded aggregate `max_events` output for non-empty cross-batch sampling, preserving event/byte budgets, cursor requeue, ordering, and empty-drain behavior. TDD source/model contract `4/4`; lower order/empty regression and ignored `RUNTIME857_ANIMATION_DRAIN_OUTPUT_CAPACITY_BENCH_V1` marker are wired. The 4,096-event model changes modeled growth `12→0`; managed Cargo/Release, allocator, and animation-event product p50/p95/p99 evidence remain pending. |
| Runtime858 render post-process output capacity | implemented_pending_validation | Post-process volume collection now reserves the registered component upper bound for both extract and local-fog outputs, preserving zero-component behavior, layer filtering, and sort order. TDD source/model contract `4/4`; lower order/empty regression and ignored `RUNTIME858_RENDER_POST_PROCESS_OUTPUT_CAPACITY_BENCH_V1` marker are wired. The 4,096-component model changes both collectors' modeled growth `24→0`; managed Cargo/Release, allocator, and render post-process product p50/p95/p99 evidence remain pending. |
| Runtime859 logical text batch capacity | implemented_pending_validation | Screen-space logical text batch projection reserves `layout.lines.len()` before artifact/fallback emission, preserving order and rejection semantics. TDD source/model contract `4/4`; lower order/empty regression and ignored `RUNTIME859_LOGICAL_TEXT_BATCH_CAPACITY_BENCH_V1` marker are wired. The 4,096-line model changes modeled growth `12→0`; managed Cargo/Release, allocator, and text-render product p50/p95/p99 evidence remain pending. |
| Runtime860 selector candidate scratch capacity | implemented_pending_validation | Runtime73 terminal-selector scratch now computes a saturating upper bound across universal/host/id/class/component/state buckets and reserves only the deficit before extension, preserving empty zero-capacity, singleton, order, and deduplication semantics. TDD source/model contract `4/4`; lower dense/empty regression and ignored `RUNTIME860_SELECTOR_CANDIDATE_CAPACITY_BENCH_V1` marker are wired. The 4,096-rule model changes `13→0` geometric growth events; managed Cargo/Release, allocator, and selector-style product p50/p95/p99 evidence remain pending. |
| Runtime861 navigation fallback path deduplication in place | implemented_pending_validation | Baked-mesh fallback path deduplication now compacts the existing `Vec<NavPathPoint>` with `dedup_by`, preserving the `0.05` XZ threshold, first-point retention, order, and metadata while removing the second output allocation. TDD source/model contract `4/4`; lower `4,096`-point capacity-retention and duplicate-semantics regression plus ignored `RUNTIME861_NAVIGATION_PATH_DEDUP_IN_PLACE_BENCH_V1` marker are wired. The deterministic model changes modeled legacy output growth `11→0`; managed Cargo/Release, allocator, and navigation fallback product p50/p95/p99 evidence remain pending. |
| Runtime862 navigation polygon vertex projection capacity | implemented_pending_validation | Baked-polygon vertex projection now reserves the bounded index slice before filtering invalid indices, preserving valid-vertex order and output semantics while removing geometric growth on dense polygons. TDD source/model contract `4/4`; lower valid/invalid-index and capacity-bound regressions plus ignored `RUNTIME862_NAVIGATION_VERTEX_PROJECTION_CAPACITY_BENCH_V1` marker are wired. The deterministic `4,096`-index model changes modeled growth `11→0`; managed Cargo/Release, allocator, and navigation projection product p50/p95/p99 evidence remain pending. |
| Runtime863 UI V2 direct-reference convergence | implemented_pending_validation | UI V2 view/component/style wrappers now delegate their typed `direct_references` methods to the shared document collector, and both `ImportedAsset` and registry dependency extraction use that contract. This repairs the three v5 `E0599` failures; v6 Runtime builds at exit `0`, the focused Runtime/asset batch passes `32/32`, and lower managed tests, Release, allocator, and product gates remain pending. |
| Runtime864 handwritten dependency move append | implemented_pending_validation | Runtime88's borrowed hash index now records acceptance in a compact bit mask, reserves the exact accepted count, and moves owned candidate URIs instead of cloning them into a second full vector. Intentional RED `4/4` becomes GREEN `4/4`; the lower pointer/order regression and ignored Release marker are wired, the 4,096-candidate model changes URI clones `4096→0`, and the related batch passes `36/36`. Managed Cargo/Release, allocator, and asset-import product p50/p95/p99 evidence remain pending. |
| Runtime865 borrowed handwritten metadata dependency index | implemented_pending_validation | Runtime643's meta/root merge now borrows existing URI indexes, records one-byte dual-target admission flags, reserves exact counts, and clones only when both destinations require ownership. Intentional RED `4/4` becomes GREEN `4/4`; the lower divergent-meta/root parity model and ignored Release marker are wired, the dense model changes URI clones `20,480→4,096`, and the related batch passes `40/40`. Managed Cargo/Release, allocator, and asset-restoration product p50/p95/p99 evidence remain pending. |
| Runtime866 resolved dependency output capacity | implemented_pending_validation | Ordered resolved IDs reserve the authored dependency upper bound; diagnostics retain zero capacity until the first missing locator and then reserve once. Intentional RED `2/5` becomes GREEN `5/5`; the lower empty/missing regression and ignored Release marker are wired, the 4,096-value model changes growth `11→0`, and Runtime864/865/866 contracts pass `13/13`. Managed Cargo/Release, allocator, and project-import p50/p95/p99 evidence remain pending. |
| Runtime867 shader dependency direct append | implemented_pending_validation | Full-generation shader dependency projection appends uniquely owned provider locators directly into the retained dependency vector, reserving the authored import bound and removing the temporary locator vector while preserving provider order, ID deduplication, ambiguous-provider rejection, and metadata/runtime duplicate ownership. Intentional RED `1/5` becomes GREEN `5/5`; the 4,096-provider model changes temporary locator slots `4096→0`. Managed Cargo/Release, allocator, and project-import p50/p95/p99 evidence remain pending. |
| Runtime868 glTF image output capacity | implemented_pending_validation | glTF image decoding reuses the validated document image count for its ordered output capacity and preserves range/source checks, budget charge order, and first-error semantics. Intentional RED `2/5` becomes GREEN `5/5`; the adjacent snapshot contract is repaired, the combined Runtime866–868/glTF batch passes `19/19`, and the 4,096-image model changes growth `11→0`. Managed Cargo/Release, allocator, and glTF-import p50/p95/p99 evidence remain pending. |
| Post-Runtime868 local receipt | implemented_pending_validation | The earlier one-process non-Tooling loader completed naturally at `4227/4227` across `995` selected modules with zero load errors/failures/errors/skips; because its selection snapshot predates Runtime867/868, the latest Runtime799/866–868 plus glTF focused batch separately passes `23/23`. The current dated Runtime/Editor record audit matches `119/119` hashes across `26` records, Wiki validation passes `272/272` with one existing metadata warning, and scoped Rustfmt/diff checks pass. Managed current-source compilation, Release, allocator, and product percentiles remain pending. |
| Runtime799/866–868 v9 handoff | managed_validation_running_asynchronously | The compile repair and three asset-import slices were submitted together with current Runtime→Editor→App source as PID `33768`; logs are the v9 async batch receipts. No waiting or monitoring follows; compile, lower ignored Release markers, allocator evidence, and product percentiles remain pending. |
| Post-Runtime865 broad local receipt | implemented_pending_validation | The one-process non-Tooling loader passes `4216/4216` tests across `993` files in `458.125s`, with zero load errors/failures/errors/skips; `19` current dated Runtime/Editor optimize records match `100/100` source hashes, Wiki validation passes `272/272`, and scoped Rustfmt/diff checks pass. Managed v7, Release, allocator, ignored-marker, and product percentile gates remain pending. |
| Runtime861 focused navigation batch receipt | implemented_pending_validation | One process covers Runtime861 plus the existing navigation projection, dispatch-ownership, tree-focus, and UI-navigation-index contracts and passes `31/31` tests in `0.033s`, with zero failures, errors, or skips. This is local source/model evidence; managed Cargo/Release, allocator, and navigation fallback product p50/p95/p99 gates remain pending. |
| Runtime861/862 focused navigation batch receipt | implemented_pending_validation | One process covers Runtime861, Runtime862, and the existing navigation projection, dispatch-ownership, tree-focus, and UI-navigation-index contracts and passes `35/35` tests in `0.062s`, with zero failures, errors, or skips. This is local source/model evidence; managed Cargo/Release, allocator, and navigation product p50/p95/p99 gates remain pending. |
| Runtime08d/861/862 expanded navigation contract receipt | implemented_pending_validation | One process covers the existing Runtime08d borrowed-index contract plus Runtime861, Runtime862, navigation projection/dispatch ownership, tree focus, and UI navigation index contracts and passes `38/38` tests in `0.139s`, with zero failures, errors, or skips. Runtime08d now recognizes the extracted `polygon_vertices` owner while retaining borrowed-slice/no-copy assertions; managed Cargo/Release, allocator, and navigation product p50/p95/p99 gates remain pending. |
| Post-Runtime861 expanded source-contract receipt | implemented_pending_validation | The refreshed one-process explicit performance-or-contract loader covers `969` non-tooling files and passes `4100/4100` tests in `225.799s`, with zero load errors, failures, errors, or skips. Two shader-prewarm Cargo command lines printed by fixture tests are local fixture output rather than managed Windows Release/Cargo acceptance; allocator and Runtime/Editor product p50/p95/p99 gates remain pending. |
| Post-Runtime862 expanded source-contract receipt | implemented_pending_validation | After the Runtime08d lower-contract compatibility repair, the one-process explicit performance-or-contract loader covers `970` non-tooling files and passes `4104/4104` tests in `228.769s`, with zero load errors, failures, errors, or skips. Two shader-prewarm Cargo command lines printed by fixture tests are local fixture output rather than managed Windows Release/Cargo acceptance; allocator and Runtime/Editor product p50/p95/p99 gates remain pending. |
| Post-document-audit broad source-contract receipt (2026-09-21) | implemented_pending_validation | The same one-process explicit performance-or-contract loader again covers `970` non-tooling files and passes `4104/4104` tests in `274.515s`, with zero load errors, failures, errors, or skips. Fixture-emitted Cargo command lines are not managed Windows Release/Cargo evidence; allocator and Runtime/Editor product p50/p95/p99 gates remain pending. |
| Current 2026-09-20 Rustfmt owner audit | implemented_pending_validation | A single `rustfmt --edition 2021 --check` invocation covers `48` Rust owners referenced by the current Runtime/Editor 2026-09-20 optimize records and exits `0`; this is syntax/format evidence only and does not replace managed Cargo, allocator, or product percentile validation. |
| Runtime855 performance-contract batch receipt | implemented_pending_validation | One process covers `658` non-tooling performance-contract files and passes `2420/2420` tests in `33.490s`, with zero load errors, failures, errors, or skips; this local source/model receipt does not replace the widened all-contract, managed Cargo/Release, allocator, or file-source product percentile gates. |
| Runtime856 focused V2 batch receipt | implemented_pending_validation | The combined Runtime853/854/855/856 plus V2 rule-capacity, pseudo-state-capacity, and Editor856 source/model batch passes `26/26` tests in `0.009s`, with zero failures, errors, or skips; managed Cargo/Release, allocator, and render-view/Material Editor product percentile gates remain pending. |
| Runtime857/858/859 + Editor857/858 focused batch receipt | implemented_pending_validation | One process covers the preceding V2/render/material contracts plus Runtime857, Runtime858, Runtime859, Editor857, and Editor858 and passes `46/46` tests in `0.017s`, with zero failures, errors, or skips; managed Cargo/Release, allocator, and product percentile gates remain pending. |
| Pre-Runtime860 all-contract regression receipt | implemented_pending_validation | The pre-Editor856 widened single-process non-tooling loader covered `957` contract files and passed `4042/4042` tests in `125.129s`; the pre-Runtime860 expanded source-contract loader covered `967` files and passed `4092/4092` tests in `375.582s` under the explicit performance-or-contract filename filter, with zero failures, errors, load errors, or skips. These historical local receipts do not replace managed Cargo/Release or product percentile gates. |
| Pre-Runtime860 expanded source-contract receipt after Runtime859/Editor858 | implemented_pending_validation | The pre-Runtime856 expanded loader covered `961` non-tooling files whose names contain `performance` or `contract` (excluding tooling/export/coordinator) and passed `4068/4068` tests in `149.452s`; the pre-Runtime857 loader covered `962` files and passed `4072/4072` tests in `139.499s`; the pre-Runtime860 loader covered `967` files and passed `4092/4092` tests in `375.582s`, with zero load errors, failures, errors, or skips. Managed Cargo/Release, allocator, and Runtime/Editor product percentile gates remain pending. |
| Post-Runtime860 expanded source-contract receipt | implemented_pending_validation | The one-process explicit performance-or-contract loader now covers `968` files and passes `4096/4096` tests in `142.495s`, with zero load errors, failures, errors, or skips. Two shader-prewarm Cargo command lines printed by fixture tests are local fixture output rather than managed Windows Release/Cargo acceptance; allocator and Runtime/Editor product p50/p95/p99 gates remain pending. |
| Current optimize-record coverage | implemented_pending_validation | All `25` Runtime/Editor optimize records dated 2026-09-20 resolve to their optimize indexes and Astra feature references with zero missing links; this is a discoverability audit only and does not close managed validation. |
| Runtime835-inclusive broad contract receipt | implemented_pending_validation | A single non-tooling `test_*contract.py` loader covers `825` Runtime/Editor modules and passes `3349/3349` tests in `69.899s`, with zero failures, errors, load errors, or skips. This is local source/model evidence; managed Cargo/Release allocation and product p50/p95/p99 gates remain pending. |
| Runtime824 input timer drain capacity | implemented_pending_validation | Four retain-based timer drains lazily reserve their current map bound on the first expiration, preserving BTreeMap order, moved payloads, pending timers, and zero-capacity no-expiry ticks. TDD source/model contract `3/3`; lower all-four-kind regression and ignored `RUNTIME824_INPUT_TIMER_DRAIN_CAPACITY_BENCH_V1` marker are wired. The latest merged non-tooling Runtime/Editor receipt passes `2224/2224` across `623` modules with zero failures/errors/skips. Managed Cargo/Release allocation and input product percentile evidence remain pending. |
| Editor828 cross-surface batch receipt | implemented_pending_validation | The current shared non-tooling Runtime/Editor loader includes the Editor828 revoke-capacity contract and passes `2227/2227` across `624` modules with zero failures, errors, or skips; managed Cargo/Release and scheduler/input product percentile evidence remain pending. |
| Editor830 cross-surface batch receipt | implemented_pending_validation | Editor830 layout preset-name capacity is wired into the shared loader with focused source/model contract `4/4`; the refreshed one-process non-tooling Runtime/Editor batch passes `2239/2239` across `627` modules in `5.505s`, and the nine-slice focused loader passes `33/33` in `0.017s`, with zero failures/errors/skips. Managed Cargo/Release and layout-preset product percentile evidence remain pending. |
| Latest 22-slice focused receipt (Runtime819/820/821/822/824/829 + Editor813–830) | implemented_pending_validation | One-process focused loader covers 22 current Runtime/Editor performance contracts and passes `77/77` tests in `0.019s`, with zero failures, errors, or skips. This supersedes the narrower nine-slice smoke receipt for local evidence; managed Cargo/Release and product p50/p95/p99 evidence remain pending. |
| Latest refreshed full non-tooling receipt | implemented_pending_validation | The same one-process loader was rerun after the current-worktree reconciliation: `627` contract files, `2239/2239` tests, `0` failures/errors/skips, `14.128s`. The timing is a local source-contract receipt, not product latency evidence; managed Cargo/Release and p50/p95/p99 gates remain pending. |
| Recent optimize-index coverage audit | implemented_pending_validation | Added missing Runtime803/804, Runtime792/793/794/797/799/800/801/802, and adjacent recent micro-slice links to `docs/plans/optimize/zircon_runtime/index.md`; all linked targets resolve and preserve the managed-validation boundary. This is documentation/index evidence only; Cargo/Release and product p50/p95/p99 gates remain pending. |
| Post-audit full non-tooling receipt | implemented_pending_validation | The one-process loader was rerun after the index reconciliation and still passes `2239/2239` tests across `627` performance-contract files in `5.134s`, with zero failures, errors, or skips. This confirms the current source/model batch remains green; it does not provide managed Cargo/Release or product p50/p95/p99 evidence. |
| Latest batched Runtime/Editor non-tooling contract receipt | implemented_pending_validation | One process loaded `835` Runtime/Editor `test_*contract.py` modules after excluding export/tooling/coordinator names and passed `3396/3396` tests in `34.531s`, with zero failures, errors, or skips. This broad source/model receipt complements the 627-file performance-contract batch; managed Cargo/Release and product p50/p95/p99 gates remain pending. |
| Current-source fingerprint freshness audit | implemented_pending_validation | Eleven historical micro-slice snapshots differ from the current shared tree; documented same-file follow-ups explain Runtime821→822, Runtime793→794, Runtime800 eligibility follow-up, and the adjacent Editor805→806/819→821/824→825 updates. Historical hashes remain intact, and current source/model contracts are green; managed Cargo/Release and product gates remain pending. |
| Recent completion-list coverage audit | implemented_pending_validation | All `20` recent Runtime and `32` recent Editor optimize records (2026-09-17…19) are linked from their optimize index and referenced by an Astra `plan_sources` entry. This is discoverability evidence only; managed Cargo/Release and product p50/p95/p99 gates remain pending. |
| Recent record metadata-shape repair | implemented_pending_validation | Six YAML records and two plain-format records now expose explicit deterministic performance-status metadata; implementation and managed-validation states remain unchanged, and no production code was modified. |
| Fingerprint-difference focused receipt | implemented_pending_validation | Seven affected current-source contracts rerun together: `7/7` files and `29/29` tests in `0.011s`, zero failures/errors/skips. This confirms source/model semantics after later same-file edits; managed Cargo/Release and product gates remain pending. |

## Plan Completion List

| Records | Area | Status | Static evidence |
| --- | --- | --- | --- |
| 785 | Runtime73 v2 style rule capacity | implemented_pending_validation | The v2 style resolver preallocates the total authored stylesheet rule bound before materialization, preserving specificity/order and selector errors. The source contract passes `3/3`; the pre-785 Runtime/Editor prefix batch passes `1270/1270` across `349` modules in `10.629s`; the Runtime785 Release marker joins the deferred managed gate. Managed Cargo/Release allocation and product style percentile evidence remain pending. |
| 795–797 | Runtime74 resource-reference streaming, compile-cache borrowed hash eviction, and eviction-key capacity | implemented_pending_validation | Runtime795/796/797 source contracts each pass `3/3`; the combined current source-contract loader passes `2002/2002` across `561` files; lower Rust order/ownership, multi-asset semantics, and bounded-collector regressions plus all three ignored Release markers are wired. Managed Cargo/Release allocation and product hot-reload/reference percentile evidence remain pending. |
| 799 | Runtime22/08F random-selector compiled weight table | implemented_pending_validation | The source contract passes `4/4`; the lower table-order/precedence/clamping regression and ignored Release marker are wired. The slice preserves the broader Runtime22 random-authority migration boundary; managed AI-runtime Cargo/Release allocation and selector percentile evidence remain pending. |
| 800 | Runtime200 package feature-definition streaming | implemented_pending_validation | The original source contract passed `4/4`; the current Runtime200/205 eligibility follow-up passes `8/8`, with lower visitor/order/provider and carrier-capacity regressions wired. The deterministic model removes 256 temporary package vectors and reduces a mixed `320`-slot bound to `64`; managed Cargo/Release allocation and catalog-build percentile evidence remain pending. |
| 801 | Runtime73 terminal-selector singleton candidate fast path | implemented_pending_validation | TDD source contract `3/3`; current Runtime801-inclusive source-contract batch `212/212` across 60 modules; focused Runtime73/Runtime785/text-decoration cross-slice batch `14/14`; broader non-tooling performance-contract discovery `2196/2196` across 597 modules; empty/singleton/order lower regressions are wired, and scoped Rustfmt/diff checks pass. Zero/one-candidate paths retain scratch clearing and skip sort/dedup while multi-candidate order remains unchanged. Managed Cargo/Release and product style-latency evidence remain pending. |
| 802 | Runtime58 bridge dependency traversal scratch capacity | implemented_pending_validation | TDD source contract `3/3`; lower cyclic scratch-clear and registration-bound graph-capacity coverage is wired. The deterministic 1,024-root model removes 1,023 visiting-set allocations while clean catalogs retain lazy issue-only buffers; diagnostic source order, duplicate suppression, and cycle behavior remain unchanged. Managed Cargo/Release allocation and catalog-build percentile evidence remain pending. |
| 803 | Runtime74 Release benchmark evidence hardening | implemented_pending_validation | Six helper probes use 101 alternating samples, balanced 51/50 first-order metadata, and raw nearest-rank P50/P95/P99 fields. The cross-file source contract passes `2/2`; exact-file Rustfmt passes for all six Rust test owners. This hardens evidence shape only; managed Cargo/Release, complete-caller, allocator, and product percentile acceptance remain pending. |
| 804 | Runtime200 empty dispatch-output fast path | implemented_pending_validation | The dispatch projection checks its two owned output vectors before surface lookup and `UiTreeId` clone, removing the empty-event projection work while preserving non-empty queues and secure-text revoke behavior. The source/model contract passes `4/4`; the merged non-tooling batch passes `3665/3665` across `870` files with zero failures/errors/skips. Managed Cargo/Release allocation and product input p50/p95/p99 evidence remain pending. |
| 807 | Runtime200 action revocation scratch capacity | implemented_pending_validation | The action queue keeps its revocation scratch at zero capacity until the first actual secure-reference revoke, then reserves the remaining component-event upper bound once. All revoke/reject branches use the helper; the source/model contract passes `4/4`, and the merged non-tooling batch passes `3677/3677` across `873` files in `205.214s` with zero failures/errors/skips. Managed Cargo/Release allocation and product input p50/p95/p99 evidence remain pending. |
| 809 | Runtime78/11A AccessKit tree projection capacity | implemented_pending_validation | Snapshot nodes reserve the exact node bound plus the optional synthetic root; synthetic-root and direct node children reserve their source lengths before ordered append. The source/model contract passes `4/4`, the lower source regression and `RUNTIME809_ACCESSKIT_TREE_PROJECTION_CAPACITY_BENCH_V1` marker are wired; the twelve-slice focused batch passes `48/48`, and the strict non-tooling batch passes `2643/2643` across `682` files in `51.429s`. Managed Cargo/Release allocation and AccessKit product p50/p95/p99 evidence remain pending. |
| 810 | Runtime73 v2 pseudo-state collector capacity | implemented_pending_validation | The static arena-node pseudo-state collector reserves the authored props/state bound plus painter allowance before sorting/deduplicating aliases; the source/model contract passes `3/3`, lower alias/order/capacity coverage and `RUNTIME810_V2_PSEUDO_STATE_CAPACITY_BENCH_V1` are wired, and the deterministic `4,097`-entry model removes `12→0` growth events. Managed Cargo/Release allocation and style product p50/p95/p99 evidence remain pending. |
| 841 | Runtime73 runtime-tree pseudo-state collector capacity | implemented_pending_validation | The runtime-tree collector reserves authored attributes, enabled component/node flag alias fanout, and painter aliases with saturating arithmetic before collection; source/model contract `4/4`, Runtime810+Runtime841 batch `7/7`, lower order/capacity coverage, and `RUNTIME841_RUNTIME_PSEUDO_STATE_CAPACITY_BENCH_V1` are wired. The dense 4,096-entry model removes `12→0` growth events. Managed Cargo/Release allocation and style product p50/p95/p99 evidence remain pending. |
| 842 | Runtime75 TreeView metadata collection capacity | implemented_pending_validation | The default-interaction TOML collectors reserve direct array capacity for ordered IDs and deduplication sets before recursive traversal, preserving alias precedence, order, duplicate suppression, and disabled membership. Source/model contract `4/4`; lower regression and `RUNTIME842_TREE_METADATA_COLLECTION_CAPACITY_BENCH_V1` are wired; the deterministic 4,096-value model removes `11→0` growth events. Managed Cargo/Release allocation and TreeView product p50/p95/p99 evidence remain pending. |
| 843 | Runtime11a UI node resource registration output capacity | implemented_pending_validation | Resource-free node output reserves the retained tree-node bound only on the first empty projection, while each metadata collector seeds URI capacity from the three authored map lengths. Source/model contract `4/4`; lower collector-capacity regression and `RUNTIME843_NODE_RESOURCE_REGISTRATION_CAPACITY_BENCH_V1` are wired; the deterministic 4,096-node model removes `11→0` growth events. Managed Cargo/Release allocation and UI asset-registration product p50/p95/p99 evidence remain pending. |
| 819 | Runtime03 accessibility diagnostic node-index deduplication | implemented_pending_validation | The snapshot validator uses one entry-based ordered node index for duplicate rejection and retained first positions, removing the parallel set/tree allocation while preserving diagnostic order and callback counts. The source/model contract passes `3/3`; lower duplicate/order coverage and `RUNTIME819_A11Y_DIAGNOSTIC_INDEX_BENCH_V1` are wired; the deterministic 4,096-node model changes auxiliary index allocations `2→1`. Managed Cargo/Release allocation and accessibility product p50/p95/p99 evidence remain pending. |
| 820 | Runtime177 dispatch host-request capacity | implemented_pending_validation | The one-pass outcome collector reserves the remaining result-count lower bound only when the first host request appears, preserving empty-path zero capacity and ordered/redraw semantics. The source/model contract passes `3/3`; lower empty/order/sparse coverage and `RUNTIME820_DISPATCH_HOST_REQUEST_CAPACITY_BENCH_V1` are wired; the deterministic 4,096-result model changes `11→0` growth events with one reservation. Managed Cargo/Release allocation and dispatch product p50/p95/p99 evidence remain pending. |
| 821 | Runtime200 focus hovered-path in-place retention | implemented_pending_validation | Focus cleanup moves `focus.hovered` out and applies `retain` before restoring the same buffer, so valid paths keep their capacity and invalid owners are removed in original order. The source/model contract passes `4/4`; the lower order/capacity regression and `RUNTIME821_FOCUS_HOVERED_RETAIN_BENCH_V1` marker are wired; the current one-process Runtime/Editor batch passes `1416/1416` across `393` modules and the broader non-tooling batch passes `2210/2210` across `619` modules with zero failures/errors/skips; the 4,096-entry deterministic model changes one replacement allocation per reconciliation to zero new allocations. Managed Cargo/Release allocation and input p50/p95/p99 evidence remain pending. |
| 822 | Runtime200 focus pointer-drag owner in-place retention | implemented_pending_validation | Invalid pointer-drag cleanup moves the existing map out and applies `BTreeMap::retain`, preserving valid entries while removing the temporary owner vector and repeated removals. The source/model contract passes `4/4`; the lower semantic regression and `RUNTIME822_FOCUS_POINTER_DRAG_RETAIN_BENCH_V1` marker are wired; the current one-process non-tooling Runtime/Editor batch passes `2218/2218` across `621` modules with zero failures/errors/skips; the deterministic 4,096-entry model changes one temporary vector plus per-invalid-owner lookup pass to zero temporary vectors plus one map traversal. Managed Cargo/Release allocation and input p50/p95/p99 evidence remain pending. |
| 835 | Runtime virtual-geometry overlay capacity | implemented_pending_validation | BVH/visbuffer gizmo and line collectors reserve input, `13` lines per BVH node, and the fixed `16`-line marker before direct extension; source/model contract `3/3`, lower regression and `RUNTIME835_VIRTUAL_GEOMETRY_OVERLAY_CAPACITY_BENCH_V1` are wired, dense models change `15→0`, `11→0`, and `3→0` growth events, and the merged capacity/projection batch passes `632/632` across `171` files in `4.660s`. Managed Cargo/Release and overlay product p50/p95/p99 evidence remain pending. |
| 661–664 | Text shaping/Unicode and streaming tab layout | implemented_pending_validation | Focused source contracts and scoped formatting checks pass. |
| 665–666 | Surface-owned navigation index streams | implemented_pending_validation | Navigation source/model contracts pass. |
| 667, 671–672 | Popup projection and pseudo-state/cache hot paths | implemented_pending_validation | Popup/cache source contracts pass. |
| 668, 675–676 | Incremental layout reports, constraint workspaces, and root routing | implemented_pending_validation | Layout source contracts pass. |
| 669–670, 674 | Render extraction and fixed-output control/overlay command buffers | implemented_pending_validation | Render source contracts pass. |
| 673 | Modal restore scratch streaming | implemented_pending_validation | Focus/modal source regression passes. |
| 677–678, 681, 686–689 | Pointer component, navigation dispatch, hit-test output capacity, control-index hash directories, and hit-grid/test-API compile hardening | implemented_pending_validation | New source regressions pass; route/navigation/hit-query/control-index contracts pass. |
| 690 | Style resolver/resource diagnostic, binding, and analog performance benchmark compile hardening | implemented_pending_validation | Source guards and Runtime performance-contract batch pass. |
| 691 | Text cache, input-model, and render-cache test API/ownership compile hardening | implemented_pending_validation | Source guards and Runtime performance-contract batch pass. |
| 692 | Accessibility snapshot/empty-vector test-contract compile hardening | implemented_pending_validation | Source guards and Runtime/Editor performance-contract batch pass. |
| 693 | Action DTO, render-domain assertion, and icon fixture compile hardening | implemented_pending_validation | Source guards and Runtime/Editor performance-contract batch pass. |
| 697 | UiTree layout-slot accessor migration in layout/template/asset/dirty-domain tests | implemented_pending_validation | No direct private `.slots` access remains in the six affected files; batched Runtime/Editor/Text contracts pass. |
| 698 | Segmented `UiRenderSubmission` public-frame assertion contract repair | implemented_pending_validation | Segment/extract assertions compile against the current DTO; batched Runtime/Editor/Text contracts pass. |
| 699 | Incremental layout frame report `Arc` assertion boundary repair | implemented_pending_validation | Published report is compared through `Arc::as_ref()`; batched Runtime/Editor/Text contracts pass. |
| 700 | Runtime200 borrowed dispatch route sharing and pointer-hover hot paths | implemented_pending_validation | Route/hover source contracts and pressure models pass `41/41`; managed Release latency/allocation evidence remains pending. |
| 701 | Runtime text wrapping `TextShapingOutcome` test-boundary repair | implemented_pending_validation | Three wrapping fixtures explicitly unwrap valid outcomes; final Runtime/Editor/Text contract batch passes. |
| 702 | Residual arrangement, projected-hit, navigation-index, and ellipsis test API repairs | implemented_pending_validation | Focused Runtime contracts pass; no production failure semantics changed. |
| 703 | Render-graph alias, segmented UI plan, SDF report, vertical text, and visibility benchmark test-API repairs | implemented_pending_validation | Latest batched Runtime/Editor/Text contracts and focused Runtime set pass; managed Release evidence remains pending. |
| 705 | Hit-route publication traversal scratch reuse across disconnected components | implemented_pending_validation | Route-index/route-sharing contracts pass; managed Release allocation and latency evidence remains pending. |
| 706 | Accessibility detached-visibility path and cycle scratch reuse across disconnected components | implemented_pending_validation | Accessibility extraction/indexed-focus contracts pass; managed Release allocation and accessibility latency evidence remains pending. |
| 707 | Notification keyboard navigation borrows the bounded visible-entry filter | implemented_pending_validation | Notification visibility/popup contracts pass; managed Release allocation and input-latency evidence remains pending. |
| 708 | Accessibility relation-target pruning uses one in-place retain pass | implemented_pending_validation | Accessibility extraction contracts pass; managed Release allocation and accessibility latency evidence remains pending. |
| 709 | Accessibility name/description resolution reuses a pre-sized node-ID scratch buffer and reserves child-filter lower bounds | implemented_pending_validation | Focused accessibility and rich-text contracts pass; managed Release allocation and accessibility latency evidence remains pending. |
| 710 | Accessibility description resolution probes borrowed text and clones only `#` reference candidates | implemented_pending_validation | Runtime accessibility/rich-text contracts `24/24` pass; managed Release allocation and accessibility latency evidence remains pending. |
| 711 | Accessibility child filtering checks the authoritative node map without cloning a key set | implemented_pending_validation | Focused accessibility/rich-text contracts and source guard pass; managed Release allocation and accessibility latency evidence remains pending. |
| 712 | UiTree dirty-domain index tracks mutable nodes incrementally; cold clears defer the first scan, bulk mutation and surface dirty-candidate/summary assembly avoid intermediate snapshots | implemented_pending_validation | Runtime/Editor performance-contract batch `1340/1340` passes; latest isolated Runtime/Editor performance batches `1146/1146` and `581/581`, pressure batches `142/142` and `122/122`, focused layout/surface batches `65/65` and `63/63`, tree/invalidation regressions, Rustfmt parser, and scoped diff checks pass; managed Rust and Release allocation/time evidence remain pending. |
| 714 | Text-decoration line-range lookup keeps the binary touched-line fast path, safely falls back for unordered interface DTOs, reuses a touched source map for caret projection, and avoids a redundant touched-line range predicate | implemented_pending_validation | Runtime text-decoration/source-map contracts `9/9` and the latest combined Runtime/Editor batch `212/212` across 60 modules pass, with Rustfmt and scoped diff checks green; managed Rust and Release text-edit evidence remain pending. |
| 715 | Rich-table shrink-sum allocation removal and gap-aware TrackMetrics authority | implemented_pending_validation | Combined rich-table/rich-text/text-decoration/Editor projection batch `54/54` passes; deterministic models reach zero shrink-sum temporary vectors and linear span work. Managed Rust and Release allocation/time plus p50/p95/p99 evidence remain pending. |
| 717 | Inline-widget arrangement reserves known direct-child, managed-binding, and preorder scratch capacities | implemented_pending_validation | Batched Runtime text/infrastructure/layout/navigation contracts `105/105` pass; source guard, Rustfmt, and scoped diff checks pass. Managed Rust and Release allocation/time evidence remain pending. |
| 720 | ECS UI projection reserves the retained node-count output capacity and borrows component flags | implemented_pending_validation | RED/GREEN source probes/in-file guards and the focused hierarchy/editor contract batch `57/57` pass; Rustfmt and scoped diff checks pass. Managed Rust and Release allocation/latency evidence remain pending. |
| 721 | Host-font admission deduplicates borrowed references before owned Arc allocation | implemented_pending_validation | RED/GREEN source probes, host-font source guard, Rustfmt, and scoped diff checks pass; managed Rust and Release font allocation/latency evidence remain pending. |
| 723 | Hit-route and arranged-visibility publication reserve known output capacities | implemented_pending_validation | RED/GREEN source probes, production capacity regressions, route/visibility source guards, and the batched Runtime/Editor contract suite pass; managed Rust and Release allocation/latency evidence remain pending. |
| 725 | Arranged-visibility publication merges sorted node IDs and visibility bits into one authoritative map traversal | implemented_pending_validation | RED/GREEN source probe, ordering/visibility regression, visibility source guard, and the post-725 baseline 347-module/1331-test Runtime/Editor batch pass; later Runtime726/727 follow-ups are recorded below. Managed Rust and Release allocation/latency evidence remain pending. |
| 726 | Full hit-route publication reuses the parent index resolved during the validation walk | implemented_pending_validation | RED/GREEN route source contract, existing deep/cycle/missing-parent regressions, and the refreshed batched Runtime/Editor suite pass; managed Rust and Release lookup/allocation/latency evidence remain pending. |
| 727 | Hit-grid entry projection reserves the authoritative draw-order upper bound and reuses its arranged-node index before filtering | implemented_pending_validation | RED/GREEN hit-grid source contract, existing hit-query behavior/pressure coverage, and the refreshed batched Runtime/Editor suite pass; managed Rust and Release allocation/latency evidence remain pending. |
| 728 | Pointer component-state accumulation keeps one-node transitions allocation-free, reserves true multi-node root output, and promotes to the existing ordered batch set only for distinct multi-node changes | implemented_pending_validation | RED/GREEN source and accumulator regressions plus the combined Runtime/Editor batch pass; managed Rust and Release allocation/latency evidence remain pending. |
| 729 | Full hit-grid rebuilds consume a lazy bounded cell-index iterator; incremental patches explicitly collect only changed-entry memberships | implemented_pending_validation | RED/GREEN iterator and row-major/invalid-span regressions, hit-grid budget/route contracts, and the one-process 349-module/1341-test batch pass; managed Rust and Release allocation/latency evidence remain pending. |
| 730 | Hit-grid reverse `node_id -> cell` maps retain stable entry vectors and insert only missing IDs during rebuild | implemented_pending_validation | RED/GREEN reverse-map source guard, capacity-preservation regression, and the batched 351-module/1345-test Runtime/Editor result pass; managed Rust and Release allocation/latency evidence remain pending. |
| 731 | Navigation tab base/group/MUI-root position maps use reusable lookup-only `HashMap`s while sorted candidate vectors remain the ordering authority | implemented_pending_validation | RED/GREEN navigation source guard, stable-bucket/pruning regression, navigation contract `7/7`, and the batched 351-module/1347-test Runtime/Editor result pass; managed Rust and Release allocation/latency evidence remain pending. |
| 732 | Navigation group position-map rebuild probes retained buckets before cloning `UiNavigationGroupId` keys | implemented_pending_validation | RED/GREEN source guard, existing stable-bucket/pruning regression, Rust parse check, and batched UI contracts `772/772`; managed Rust and Release allocation/latency evidence remain pending. |
| 733 | Navigation first-group candidate is reduced in the primary node stream without temporary vectors | implemented_pending_validation | RED/GREEN source guards, one-stream Rust regression, Rust parse check, and batched UI contracts `772/772`; managed Rust and Release allocation/latency evidence remain pending. |
| 734 | Navigation modal/MUI candidate buckets retain capacity and stable group keys | implemented_pending_validation | RED/GREEN source guards, lower Rust group/MUI-root capacity/pruning regressions, Rust parse check, and batched UI contracts `772/772`; managed Rust and Release allocation/latency evidence remain pending. |
| 735 | Navigation first-group candidate map retains stable keys across rebuilds | implemented_pending_validation | RED/GREEN source guards, lower Rust `seen` reset/prune and key-identity/capacity regression shape, Rust parse check, and batched Runtime/Editor UI contracts `105/105`; managed Rust and Release allocation/latency evidence remain pending. |
| 736 | Virtual-list reconciliation retains transactional candidate slot/key/generation buffers | implemented_pending_validation | RED/GREEN source guard rejects per-request slot-map cloning; explicit `UiVirtualListSlotMap::clone_from` plus lower warm-capacity/pointer and protected-rebind atomicity regressions are recorded; the latest 172-module Runtime/Editor UI batch passes `773/773` in 1.514s; managed Rust and Release allocation/latency evidence remain pending. |
| 737 | Node-pool owned key lookup and one-pass residency reporting | implemented_pending_validation | RED/GREEN source guard rejects the old desired-key clone and separate residency walks; lower pointer/bucket/residency regressions, focused `27/27` contracts, the 173-module/776-test UI batch, and the broader 548-module/2057-test Runtime/Editor performance-plus-pressure batch pass; managed Rust and Release allocation/latency evidence remain pending. |
| 738 | Reflection node-index hash lookup and retained rebuild capacity | implemented_pending_validation | RED/GREEN capacity, lookup, and duplicate-path winner regressions; focused Runtime UI node-pool/reflection/layout contracts pass `30/30`, and the direct 91-module Runtime UI batch passes `466/466` in `15.490s`; the preceding 548-module/2057-test Runtime/Editor static result remains the broad baseline; managed Rust and Release allocation/query-latency evidence remain pending. |
| 739 | Pointer/navigation dispatch handler hash lookup | implemented_pending_validation | TDD RED/GREEN source contract `4/4`; private exact-key handler tables use `HashMap` while per-key registration order and route/phase traversal remain unchanged; managed Rust and Release allocation/input-latency evidence remain pending. |
| 740 | Active pointer table exact-key index | implemented_pending_validation | TDD RED/GREEN source contract `3/3`; `HashMap<UiPointerId, usize>` preserves ordered entries and middle-remove semantics while eliminating linear lookup scans; the comprehensive non-tooling Runtime/Editor performance/pressure batch passes `2039/2039` across 548 files; managed Rust and Release allocation/input-latency evidence remain pending. |
| 741 | Runtime UI prototype-store streams the canonical registry iterator | implemented_pending_validation | TDD RED/GREEN source contract `2/2`; removes the temporary `AssetRegistryIndex::entries()` vector while preserving canonical order, filtering, artifact loading, and aliases; the comprehensive non-tooling Runtime/Editor performance/pressure batch passes `2039/2039` across 548 files; managed Rust and Release allocation/startup-latency evidence remain pending. |
| 742 | Dynamic cross-surface pointer capture and position state use hash lookup | implemented_pending_validation | TDD RED/GREEN source contract `3/3`; private `Option<u64>` capture/position tables and helper signatures use `HashMap` without changing cleanup or routing order; the comprehensive non-tooling Runtime/Editor performance/pressure batch passes `2039/2039` across 548 files; managed Rust and Release allocation/input-latency evidence remain pending. |
| 743 | Runtime206/Runtime85 source-contract robustness repairs | implemented_pending_validation | Whitespace-stable posting and project-root guards pass the focused `25/25` optimization set; the refreshed non-tooling Runtime/Editor batch covers `555` modules and passes `2065/2065` in `24.966s`; managed Rust and Release evidence remain pending. |
| 744 | Runtime206 secondary tag/package/path-prefix query postings | implemented_pending_validation | TDD source/behavior contracts preserve composed filters, labeled subassets, source removal, empty-bucket retirement, and borrowed path bounds; the refreshed `555`-module/`2065`-test batch passes; managed Rust query-plan/latency evidence remains pending. |
| 745 | Runtime206 referencer ordering uses a borrowed binary UUID sort key | implemented_pending_validation | TDD source contract and binary/display-order regression pass; the deterministic model removes `65,536` owned sort keys, and the refreshed `555`-module/`2065`-test batch passes; managed Rust query-latency evidence remains pending. |
| 746 | Runtime206 registry bulk-build capacity and streamed dependency bootstrap | implemented_pending_validation | Primary maps reserve iterator bounds and dependency paths stream without the all-at-once staging vector; resolved/unresolved behavior regressions and the refreshed `555`-module/`2065`-test batch pass; managed Rust build/allocation evidence remains pending. |
| 747 | Runtime UI naming-boundary contract repair | implemented_pending_validation | The neutral Agent Chat host comment no longer triggers the unclassified Runtime naming audit; targeted naming-contract rerun passes `6/6`, while the unrelated WOC dependency failure remains out of scope. |
| 748 | Runtime200 pointer hover large-path membership scratch reuse | implemented_pending_validation | Surface-owned non-serialized membership scratch, equal/small-path fast paths, and capacity ceiling are covered by the new source contract and focused hover batch (`13/13`); the Rust capacity regression and `RUNTIME200_HOVER_DIFF_MEMBERSHIP_SCRATCH_BENCH_V1` Release marker join the deferred managed gate. |
| 749 | Runtime200 primary Touch/Pen pointer membership counter | implemented_pending_validation | Table-owned Touch/Pen primary counters update with upsert/source transitions/remove/clear; `UiInputManager` no longer scans all active entries for a new touch-like pointer. Source/lifecycle/pressure contracts pass `15/15`; the Rust lifecycle regression and `RUNTIME200_PRIMARY_POINTER_SOURCE_COUNTER_BENCH_V1` marker join the deferred managed gate. |
| 750 | Runtime200 route terminal state bit | implemented_pending_validation | The route-step builder carries its ancestor stop state into the out-of-route terminal append guard, replacing the accumulated-step scan while preserving route ordering and stop semantics. Focused source/behavior contract passes `3/3`; the Rust regression and `RUNTIME200_ROUTE_TERMINAL_SCAN_BENCH_V1` marker join the deferred managed gate. |
| 751 | Runtime200 generic route preview path borrow | implemented_pending_validation | `populate_generic_route_trace` selects bubble/focus preview input by borrow, removing the intermediate route clone while preserving precedence and fallback. Focused source/behavior contract passes `3/3`; the Rust regression and `RUNTIME200_ROUTE_PATH_BORROW_BENCH_V1` marker join the deferred managed gate. |
| 752 | Runtime77 input effect result capacity reservation | implemented_pending_validation | `apply_dispatch_reply_core` reserves the known effect upper bound for applied, rejected, host-request, and component-event projections before the transaction loop. The RED/GREEN source contract passes `4/4`; lower Rust capacity regression and `RUNTIME77_EFFECT_RESULT_CAPACITY_BENCH_V1` marker join the deferred managed gate. |
| 753 | Runtime77 pointer reply projection capacity | implemented_pending_validation | Pointer reply fixed/dynamic effect bounds and pointer/text merge projections reserve known capacities; text `widget_events` are now retained while preserving ordering and rebasing. The RED/GREEN source contract passes `5/5`; lower Rust capacity-hint/widget-event regression and `RUNTIME77_POINTER_REPLY_CAPACITY_BENCH_V1` marker join the deferred managed gate. |
| 754 | Runtime82 secure-text presentation bounded capacity | implemented_pending_validation | `UiSecureTextPresentation::new` materializes hard lines once, reserves a mask-safe display bound, outer cluster/line bounds, and each grapheme iterator's upper bound for logical ranges without changing masking, separators, or bidi semantics. The RED/GREEN source contract passes `4/4`; lower semantic/overflow regression and `RUNTIME754_SECURE_TEXT_PRESENTATION_CAPACITY_BENCH_V1` marker join the deferred managed gate. The final merged Runtime/Editor source-contract batch passes `1852/1852` across `517` modules in `159.897s`; managed Cargo/Release remains pending. |
| 755 | Runtime75 command-palette filtered capacity | implemented_pending_validation | `sync_filter_state` filters parsed command entries through a source-first/query-second helper that reserves `entries.len()` before cloning retained IDs, preserving empty-source behavior, duplicate IDs, and source order. TDD source contract is RED (3 expected obligations) then GREEN `4/4`; lower regression and `RUNTIME755_COMMAND_PALETTE_FILTERED_CAPACITY_BENCH_V1` marker join the deferred managed gate. The adjacent Runtime/Editor command-palette/capacity batch passes `34/34` in `0.146s`; managed Cargo/Release remains pending. |
| 756 | Runtime75 command-palette entry streaming | implemented_pending_validation | `command_entry_list` now streams nested `UiValue` command leaves into one root output vector and reserves the conservative root shape hint, preserving map/string/enum interpretation, invalid-leaf rejection, order, and duplicates. TDD source contract is RED (3 expected obligations) then GREEN `4/4`; lower nested-order/root-capacity regression and `RUNTIME756_COMMAND_PALETTE_ENTRY_STREAM_BENCH_V1` marker join the deferred managed gate. The combined Runtime755/756 adjacent Runtime/Editor batch passes `38/38` in `0.427s`; managed Cargo/Release remains pending. |
| 757 | Runtime75 TreeView ID collection capacity | implemented_pending_validation | Tree node, borrowed/owned string, and disabled-option collectors reserve the direct array/flags bound before recursive traversal, preserving first occurrence, ordering, duplicate suppression, and flags semantics without a pre-scan. The source contract passes `4/4`; lower nested-order/capacity coverage and `RUNTIME757_TREE_ID_COLLECTION_CAPACITY_BENCH_V1` join the deferred managed gate. The adjacent Runtime/Editor batch passes `42/42` in `0.314s`; managed Cargo/Release remains pending. |
| 758 | Runtime75 keyboard option-entry streaming | implemented_pending_validation | Keyboard `option_entry_list` now streams nested array leaves into one root vector with conservative capacity hints, preserving identity aliases, labels, empty filtering, order, and duplicates. The source contract passes `4/4`; lower nested-order/capacity coverage and `RUNTIME758_KEYBOARD_OPTION_ENTRY_STREAM_BENCH_V1` join the deferred managed gate. The adjacent Runtime/Editor batch passes `46/46` in `0.266s`; managed Cargo/Release remains pending. |
| 759 | Runtime75 keyboard option-ID streaming | implemented_pending_validation | Keyboard `option_id_list` now streams nested array/map IDs into one capacity-hinted vector, preserving alias precedence, scalar/array empty behavior, order, and duplicates. The source contract passes `4/4`; lower nested-order/empty-rule coverage and `RUNTIME759_KEYBOARD_OPTION_ID_STREAM_BENCH_V1` join the deferred managed gate. The adjacent Runtime/Editor batch passes `50/50` in `0.131s`; managed Cargo/Release remains pending. |
| 763 | Runtime75 menu-search projection capacity | implemented_pending_validation | Filtered membership and preorder ID projection reserve direct arrays/flags and direct option-child counts before recursive append, preserving search order, deduplication, and focus behavior without a pre-scan. The source contract passes `4/4`; lower nested-ID/capacity coverage and `RUNTIME763_MENU_SEARCH_PROJECTION_CAPACITY_BENCH_V1` join the deferred managed gate. The adjacent Runtime/Editor batch passes `58/58` in `0.110s`; managed Cargo/Release remains pending. |
| 764 | Runtime75 menu-search streaming | implemented_pending_validation | Recursive menu-search option construction now streams into the root/actual child owner vectors and borrows the top-level ID while descending, preserving top-level indexes, child order, labels, identity aliases, and default focus candidates. The source contract passes `4/4`; lower tree-order/focus-index coverage and `RUNTIME764_MENU_SEARCH_STREAM_BENCH_V1` join the deferred managed gate. The adjacent Runtime/Editor batch passes `58/58` in `0.110s`; managed Cargo/Release remains pending. |
| 765 | Runtime75 menu typeahead option-ID borrow | implemented_pending_validation | Typeahead current-index lookup reads the existing ordered `OptionEntry` IDs directly, preserving focused/selected/value/value-text/group-value precedence while removing the temporary cloned ID vector. The source contract passes `4/4`; lower value-precedence coverage and `RUNTIME765_MENU_TYPEAHEAD_OPTION_ID_BORROW_BENCH_V1` join the deferred managed gate. The adjacent Runtime/Editor batch passes `62/62` in `0.061s`; managed Cargo/Release remains pending. |
| 766 | Runtime75 indexed keyboard-entry streaming | implemented_pending_validation | Indexed keyboard navigation streams nonempty parsed IDs directly into its retained output, preserving candidate-property and option order while eliminating the full intermediate ID vector. The source contract passes `3/3`; lower property/order/capacity coverage and `RUNTIME766_INDEXED_KEYBOARD_ENTRY_STREAM_BENCH_V1` join the deferred managed gate. The adjacent Runtime/Editor batch passes `65/65` in `0.380s`; managed Cargo/Release remains pending. |
| 767 | Runtime75 TextInput property borrow | implemented_pending_validation | Validation property order and mirror targets use static ordered slices; validation borrows its selected state/default text instead of cloning it. Required/min/max grapheme checks, priority, and mirroring stay unchanged. The source contract passes `4/4`; lower candidate/mirror-order coverage and `RUNTIME767_TEXT_INPUT_PROPERTY_BORROW_BENCH_V1` join the deferred managed gate. The adjacent Runtime/Editor batch passes `69/69` in `0.577s`; managed Cargo/Release remains pending. |
| 768 | Runtime75 TextInput timing normalization | implemented_pending_validation | Timing evaluation borrows its textual setting and compares normalized aliases by streaming separator-skipping Unicode lowercase characters, removing both the setting clone and normalized String materialization. Change/input/live/value-changed, blur/focus-out/focus-lost, and commit fallback behavior remain unchanged. The source contract passes `4/4`; lower alias/fallback coverage and `RUNTIME768_TEXT_INPUT_TIMING_NORMALIZATION_BENCH_V1` join the deferred managed gate. The adjacent Runtime/Editor batch passes `73/73` in `0.889s`; managed Cargo/Release remains pending. |
| 769 | Runtime75 selection Flags array capacity | implemented_pending_validation | Array-to-Flags normalization streams direct values into a vector reserved to the direct Array bound, preserving String/Enum retention, empty/non-textual filtering, order, and property ownership. The source contract passes `4/4`; lower filtering/capacity coverage and `RUNTIME769_SELECTION_FLAGS_CAPACITY_BENCH_V1` join the deferred managed gate. The adjacent Runtime/Editor batch passes `77/77` in `0.462s`; managed Cargo/Release remains pending. |
| 770 | Runtime75 collection single state resolution | implemented_pending_validation | Array set/remove/move and map add/set/remove retain one resolved mutable state container through validation and mutation. Map add uses BTreeMap Entry, map set uses one mutable lookup, and map remove uses one removal lookup while preserving replacement, bounds, error, ordering, ownership, and reference-source behavior. The source contract passes `5/5`; lower success/error coverage and `RUNTIME770_COLLECTION_SINGLE_RESOLUTION_BENCH_V1` join the deferred managed gate. The adjacent Runtime/Editor batch passes `82/82` in `0.829s`; managed Cargo/Release remains pending. |
| 771 | Runtime75 table borrowed sort setting | implemented_pending_validation | Table event payload, comparison/mode, and column-width reads propagate borrowed String/Enum slices through read-only paths, preserving aliases, row ordering, and sort-model publication while removing transient clones. The sort-direction transition retains its necessary state-derived column clone before same-map mutation. The source contract passes `4/4`; lower sort/width coverage and `RUNTIME771_TABLE_BORROWED_SORT_SETTING_BENCH_V1` join the deferred managed gate. The current combined Runtime/Editor batch passes `120/120` in `0.080s`; managed Cargo/Release remains pending. |
| 772 | Runtime75 Toast borrowed setting | implemented_pending_validation | Toast queue scan/current presence paths borrow textual settings, and authored-message synchronization owns only the value written to retained state; queue precedence, fallback, popup, and expiry behavior remain unchanged. The source contract passes `5/5`; lower synchronization coverage and `RUNTIME772_TOAST_BORROWED_SETTING_BENCH_V1` join the deferred managed gate. The current combined Runtime/Editor batch passes `120/120` in `0.080s`; managed Cargo/Release remains pending. |
| 773 | Runtime75 text-search streaming Unicode prefix | implemented_pending_validation | The shared non-ASCII prefix matcher streams `char::to_lowercase` output and preserves multi-scalar case mappings without a temporary candidate buffer. The source contract passes `4/4`; lower Unicode expansion coverage and `RUNTIME773_TEXT_SEARCH_STREAMING_PREFIX_BENCH_V1` join the deferred managed gate. The current combined Runtime/Editor batch passes `120/120` in `0.080s`; managed Cargo/Release remains pending. |
| 774 | Runtime75 text-search streaming Unicode contains | implemented_pending_validation | The shared non-ASCII contains matcher scans `char::to_lowercase` output with cloned suffix iterator state, preserving multi-scalar case mappings and overlap without a temporary candidate buffer. The source contract passes `4/4`; lower Unicode expansion/overlap coverage and `RUNTIME774_TEXT_SEARCH_STREAMING_CONTAINS_BENCH_V1` join the deferred managed gate. The current combined Runtime/Editor batch passes `120/120` in `0.080s`; managed Cargo/Release remains pending. |
| 775 | Runtime75 menu typeahead text normalization | implemented_pending_validation | `keyboard_text_search` filters controls, trims leading/trailing Unicode whitespace, and lowers into one returned buffer while retaining only a trailing suffix offset. The source contract passes `3/3`; lower equivalence coverage and `RUNTIME775_MENU_TYPEAHEAD_TEXT_NORMALIZATION_BENCH_V1` join the deferred managed gate. The current combined Runtime/Editor batch passes `120/120` in `0.080s`; managed Cargo/Release remains pending. |
| 776 | Runtime75 menu typeahead append reuse | implemented_pending_validation | `menu_typeahead_searches` moves the prior normalized buffer and appends the payload in place, preserving active/expired/repeated/fallback state transitions. The source contract passes `3/3`; lower state-transition coverage and `RUNTIME776_MENU_TYPEAHEAD_APPEND_REUSE_BENCH_V1` join the deferred managed gate. The focused menu/typeahead/search batch passes `18/18` in `0.041s` and the current combined Runtime/Editor batch passes `120/120` in `0.080s`; managed Cargo/Release remains pending. |
| 777 | Runtime75 menu typeahead search projection | implemented_pending_validation | `MenuTypeaheadSearch` retains one candidate buffer for matching and state publication, while the multi-scalar decision reads no farther than the second Unicode scalar. The source contract passes `4/4`; lower pasted/active/fallback/Unicode-expansion coverage and `RUNTIME777_MENU_TYPEAHEAD_SEARCH_PROJECTION_BENCH_V1` join the deferred managed gate. The current combined Runtime/Editor batch passes `120/120` in `0.080s`; managed Cargo/Release remains pending. |
| 778 | Runtime75 menu search query borrow | implemented_pending_validation | Menu search-filter normalization borrows the selected query setting, trims before allocating lowercase output, and preserves state/default precedence. The source contract passes `2/2`; lower state/default/whitespace coverage and `RUNTIME778_MENU_SEARCH_QUERY_BORROW_BENCH_V1` join the deferred managed gate. The current combined Runtime/Editor batch passes `120/120` in `0.080s`; managed Cargo/Release remains pending. |
| 779 | Runtime75 menu search filter accumulator | implemented_pending_validation | Recursive menu filtering writes into one shared accumulator and rolls back unmatched branches by checkpoint, avoiding per-node child vectors while preserving preorder/focus order. The source contract passes `3/3`; lower order/rollback coverage and `RUNTIME779_MENU_SEARCH_FILTER_ACCUMULATOR_BENCH_V1` join the deferred managed gate. The current combined Runtime/Editor batch passes `120/120` in `0.080s`; managed Cargo/Release remains pending. |
| 780 | Runtime75 menu label borrow | implemented_pending_validation | Menu search-tree label lookup borrows map text and clones only for retained option ownership, preserving multi-ID labels and empty-label ID fallback. The source contract passes `3/3`; lower label/fallback coverage and `RUNTIME780_MENU_LABEL_BORROW_BENCH_V1` join the deferred managed gate. The current combined Runtime/Editor batch passes `120/120` in `0.080s`; managed Cargo/Release remains pending. |
| 781 | Runtime75 menu child-values iterator | implemented_pending_validation | Menu search-tree child traversal streams the fixed child-property lookup sequence directly, removing one temporary `Vec<&UiValue>` per map node while preserving property order and descendant behavior. The source contract passes `3/3`; lower property-order coverage and `RUNTIME781_MENU_CHILD_VALUES_ITERATOR_BENCH_V1` join the deferred managed gate. The current combined Runtime/Editor batch passes `120/120` in `0.080s`; managed Cargo/Release remains pending. |
| 782 | Runtime75 v2 surface-tree interaction shared catalog | implemented_pending_validation | v2 interaction capability inference borrows the process-wide `editor_showcase_shared()` catalog instead of caching a private owned registry clone. The source contract passes `3/3`; lower pointer-identity coverage and `RUNTIME782_SURFACE_TREE_SHARED_CATALOG_BENCH_V1` join the deferred managed gate. Managed Cargo/Release and product interaction percentile evidence remain pending. |
| 783 | Runtime75 overlay static-key update | implemented_pending_validation | Popup/dialog fixed-key publication mutates existing values in place and owns a key only for first insertion, preserving reference-source clearing and alias semantics. The source contract passes `3/3`; lower key-identity/reference-source coverage and `RUNTIME783_OVERLAY_STATIC_KEY_UPDATE_BENCH_V1` join the deferred managed gate. Managed Cargo/Release and product popup percentile evidence remain pending. |
| 784 | Runtime75 notification static-key update | implemented_pending_validation | Notification selected-id, focused-index, and unread-count publication mutates existing values in place and owns keys only on first insertion, preserving reference-source clearing and navigation semantics. The source contract passes `3/3`; the current-source loader passes `1951/1951` across `544` modules and the merged focused hot-path set passes `145/145` in `0.150s`. Lower key-identity/reference-source coverage and `RUNTIME784_NOTIFICATION_STATIC_KEY_UPDATE_BENCH_V1` join the deferred managed gate; managed Cargo/Release and product notification percentile evidence remain pending. |
| 786 | Runtime206 incremental project resource publication capacity | implemented_pending_validation | Incremental resource publication reserves the changed-record and previous-record bounds for its four temporary map/vector/set projections, preserving ID deduplication, source filtering, and unordered map materialization. The TDD source contract passes `3/3`; the lower upper-bound regression and `RUNTIME786_INCREMENTAL_PUBLICATION_CAPACITY_BENCH_V1` marker join the deferred managed gate. |
| 788 | Runtime11A TreeView selection output capacity | implemented_pending_validation | The TreeView reducer reserves the exact selected-set length before ordered selected-ID projection, preserving source order and selection semantics. The TDD source contract passes `3/3`; the lower count/order regression and `RUNTIME788_TREE_SELECTION_OUTPUT_CAPACITY_BENCH_V1` marker join the current `552`-module/`1975/1975` Runtime/Editor local batch. |
| 790 | Runtime75 shared component catalog view | implemented_pending_validation | The default compiler borrows the shared showcase registry, custom overrides stay isolated, and the retained registry preserves the 258-ID Material-wins union. Pointer identity, override isolation, union/precedence regressions, and `RUNTIME75_SHARED_COMPONENT_REGISTRY_BENCH_V1` are present; managed Cargo/Release and product percentile evidence remain pending. |
| 791 | Runtime11A focus navigation output capacity | implemented_pending_validation | Focus navigation reserves the tree node-count upper bound before recursive candidate collection, preserving root/child order and focus semantics. The source contract passes `3/3`; the lower order/capacity regression and `RUNTIME791_FOCUS_NAVIGATION_CAPACITY_BENCH_V1` marker join the deferred managed gate. The 4,096-node model removes 11 geometric growth events; managed Cargo/Release and navigation percentile evidence remain pending. |
| 792 | Runtime11A surface-index target projection capacity | implemented_pending_validation | Surface/node hot-reload target aggregation reserves the saturating sum of four category inputs before borrowed-set deduplication, preserving category and first-seen order. The source contract passes `3/3`; lower order/capacity regressions and `RUNTIME792_SURFACE_INDEX_TARGET_CAPACITY_BENCH_V1` join the deferred managed gate. The 16,384-target model removes 12 geometric growth events; the current `554`-file/`1981/1981` Runtime/Editor source-contract batch is green. Managed Cargo/Release and hot-reload percentile evidence remain pending. |
| 793 | Runtime74 hot-reload compile-cache eviction capacity | implemented_pending_validation | Compile-cache eviction reserves the saturating sum of the two materialized target vectors, preserving order, duplicate handling, and report semantics. The source contract passes `3/3`; lower empty/ordinary/overflow regression and `RUNTIME793_HOT_RELOAD_EVICTION_CAPACITY_BENCH_V1` join the deferred managed gate. The 16,384-target model removes 13 geometric growth events; managed Cargo/Release and hot-reload percentile evidence remain pending. |
| 794 | Runtime74 hot-reload template-asset projection capacity | implemented_pending_validation | Per-surface template-asset projection reserves the saturating two-input bound before filtering, preserving category order and membership. The source contract passes `3/3`; lower empty/ordinary/overflow regression and `RUNTIME794_HOT_RELOAD_TEMPLATE_ASSETS_CAPACITY_BENCH_V1` join the deferred managed gate. The 16,384-target model removes 13 geometric growth events; managed Cargo/Release and hot-reload percentile evidence remain pending. |

## Batched verification

- Runtime785 v2 style-rule collection now reserves the summed authored stylesheet rule bound before
  selector parsing and specificity sorting. Its source contract passes `3/3`; the current
  non-tooling core batch passes `1963/1963` across `549` modules in `14.682s`; the Runtime785
  Release marker remains in the deferred managed gate.
- Runtime786 adds input-bounded reservations to the Runtime206 incremental project-resource
  publication maps, removed-locator vector, and source-removal set. Its source contract passes
  `3/3`; the lower upper-bound regression and `RUNTIME786_INCREMENTAL_PUBLICATION_CAPACITY_BENCH_V1`
  marker are wired into the same deferred Runtime/Editor lane. The deterministic four-family model
  removes 216 geometric growth events; the focused Runtime206/Runtime85 batch passes `23/23`, and
  the current Runtime/Editor performance-contract loader passes `1966/1966` across `549` modules.
  These are local allocation/source-model receipts only.
- Runtime788 reserves the index-owned selected-set length before the ordered TreeView selection
  projection. Its source contract passes `3/3`; the lower count/order regression and
  `RUNTIME788_TREE_SELECTION_OUTPUT_CAPACITY_BENCH_V1` marker are wired into the same deferred
  Runtime/Editor lane. The 65,536-item model removes 15 geometric growth events. The refreshed
  Runtime/Editor source-contract batch loads `552` modules and passes `1975/1975` tests in
  `6.114s`, with zero failures, errors, or skips. This is local allocation/source-model evidence
  only.
- Runtime790 records the shared component-catalog ownership slice: the default compiler borrows
  the process-wide showcase registry, and the retained Editor union remains Material-wins. The
  adjacent Runtime01/shared-catalog contract batch passes `12/12` across two modules; the ignored
  `RUNTIME75_SHARED_COMPONENT_REGISTRY_BENCH_V1` marker remains in the deferred managed lane.
- Runtime791 reserves the `UiTree.nodes` upper bound before recursive focus-navigation candidate
  collection. Its source contract passes `3/3`; the lower enabled/order/capacity regression and
  `RUNTIME791_FOCUS_NAVIGATION_CAPACITY_BENCH_V1` marker are wired into the deferred managed lane.
  The deterministic 4,096-node model removes 11 modeled growth events. The refreshed
  Runtime/Editor performance-contract batch loads 553 files and passes `1978/1978` tests in
  `5.513s`, with zero failures, errors, or skips. These are local source/model receipts only;
  managed Cargo/Release allocation and navigation percentile evidence remain pending.
- Runtime792 reserves the summed four-category upper bound before surface and node hot-reload
  target projection. Its source contract passes `3/3`; the lower order/capacity regressions and
  `RUNTIME792_SURFACE_INDEX_TARGET_CAPACITY_BENCH_V1` marker are wired into the next deferred
  managed lane. The deterministic 16,384-target model removes 12 modeled growth events. The
  refreshed explicit-file Runtime/Editor source-contract batch loads `554` files and passes
  `1981/1981` tests in `30.200s`, with zero failures, errors, or skips. These are local
  source/model receipts only; managed Cargo/Release allocation and hot-reload percentile
  evidence remain pending.
- Runtime793 then reserves the two-input upper bound before compile-cache eviction target
  admission. Its source contract passes `3/3`; the lower empty/ordinary/overflow regression and
  `RUNTIME793_HOT_RELOAD_EVICTION_CAPACITY_BENCH_V1` marker are wired into the same deferred
  managed lane. The deterministic 16,384-target model removes 13 growth events; cache eviction
  order and report semantics remain unchanged. The post-format combined Runtime/Editor
  source-contract batch covers `556` files and passes `1987/1987` in `5.867s`; the broad
  non-tooling regression batch covers `920` files and passes `3737/3737` in `445.407s`, with
  zero failures, errors, or skips.
- Runtime794 reserves the same two-input upper bound before each active-surface template-asset
  filter. Its source contract passes `3/3`; the lower empty/ordinary/overflow regression and
  `RUNTIME794_HOT_RELOAD_TEMPLATE_ASSETS_CAPACITY_BENCH_V1` marker are wired into the same
  deferred managed lane. The deterministic 16,384-target model removes 13 growth events while
  preserving target order and membership.
- Runtime795 streams UI resource URIs directly into the existing reference normalizer and
  deduplicator, removing the temporary URI pointer vector while preserving traversal order and
  first-seen ownership. Its source contract passes `3/3`; the lower order/ownership regressions
  and `RUNTIME74_UI_RESOURCE_REFERENCE_VISITOR_BENCH_V1` marker (101 alternating pairs,
  51 legacy-first and 50 optimized-first) are wired into the deferred lane.
- Runtime796 keeps compile-cache and invalidation-snapshot maps ordered but changes temporary
  eviction membership to borrowed `HashSet<&str>` probes. Its source contract passes `3/3`; the
  lower duplicate/missing/retained/evicted semantics regression and
  `RUNTIME74_COMPILE_CACHE_HASH_EVICTION_BENCH_V1` marker (101 alternating samples,
  `sample_count=101`) are wired into the deferred lane.
- Runtime797 gives both temporary eviction-key vectors an input-sized lower-bound reservation
  from `asset_ids.len()`, avoiding whole-cache over-reservation while removing geometric growth in
  the modeled one-key-per-request shape. Its source contract passes `3/3`; the lower two-collector
  regression and `RUNTIME797_COMPILE_CACHE_EVICTION_KEY_CAPACITY_BENCH_V1` marker are wired into
  the deferred lane.
- The refreshed Runtime/Editor source-contract batch after Editor789 loads `552` modules and
  passes `1975/1975` tests in `5.012s`, with zero failures, errors, or skips. It includes the
  Runtime786/Runtime788 capacity slices and Editor787/Editor789 hot-path contracts; this remains
  local source/model evidence only.
- After the Editor787 final-page boundary repair, the same batched loader was rerun and again
  passed `1975/1975` across `552` modules in `6.114s`; the focused Runtime206/Runtime85/
  Editor787/Editor789/Runtime788 set passes `32/32`. A randomized 60,000-case Scene Picker parity
  model also passes. Managed Cargo/Release and product percentile gates remain pending.
- The broader non-tooling Runtime/Editor Python regression discovery then passed `3724/3724`
  across `915` modules in `326.952s`, with zero failures, errors, or skips. This remains local
  regression evidence and does not replace managed Cargo/Release or product percentile gates.
- The merged recent Runtime/Editor/input/style focused batch covers `61` modules and passes
  `316/316` tests in `5.877s`, with zero failures, errors, or skips, including Runtime785.

- Runtime757 adds direct array/flags capacity reservations to TreeView's recursive ID collectors
  without changing their existing recursion or deduplication paths. Its source contract, the
  Runtime755/756 command-palette contracts, and adjacent Runtime/Editor capacity guards pass
  `42/42` in `0.314s` in one Python process. This local receipt postdates the earlier
  `1852/1852` snapshot, which predates Runtime755–757, and does not replace managed
  Cargo/Release allocation or product percentile evidence.
- Runtime758 adds the shared keyboard option-entry collector to the same batched scope. The
  source contract, Runtime755–757 contracts, and adjacent Runtime/Editor capacity guards pass
  `46/46` in `0.266s` in one Python process. This remains local source/model evidence only;
  managed Cargo/Release allocation and product keyboard percentile gates remain pending.
- Runtime759 adds the keyboard option-ID collector to that same one-process scope. The source
  contracts for Runtime755–759 and adjacent Runtime/Editor capacity guards pass `50/50` in
  `0.131s`; this remains local source/model evidence only, with managed Cargo/Release allocation
  and product keyboard percentile gates pending.
- Runtime763/764 add menu-search direct-bound capacity reservations and shared recursive option
  streams to the same one-process scope. The source contracts for Runtime755–759, Runtime763/764,
  and adjacent Runtime/Editor capacity guards pass `58/58` in `0.110s`; this remains local
  source/model evidence only, with managed Cargo/Release allocation and product keyboard
  percentile gates pending.
- Runtime765 adds the menu typeahead borrowed current-index lookup to the same one-process scope.
  The source contracts for Runtime755–759 and Runtime763–765, plus adjacent Runtime/Editor
  capacity guards, pass `62/62` in `0.061s`; this remains local source/model evidence only, with
  managed Cargo/Release allocation and product keyboard percentile gates pending.
- Runtime766 adds direct indexed-keyboard ID streaming to the same one-process scope. The source
  contracts for Runtime755–759 and Runtime763–766, plus adjacent Runtime/Editor capacity guards,
  pass `65/65` in `0.380s`; this remains local source/model evidence only, with managed
  Cargo/Release allocation and product keyboard percentile gates pending.
- Runtime767 adds static TextInput candidate/mirror property slices and borrowed validation text to
  the same one-process scope. The source contracts for Runtime755–759 and Runtime763–767, plus
  adjacent Runtime/Editor capacity guards, pass `69/69` in `0.577s`; this remains local
  source/model evidence only, with managed Cargo/Release allocation and product input percentile
  gates pending.
- Runtime768 adds borrowed TextInput timing settings and streaming alias normalization to the same
  one-process scope. The source contracts for Runtime755–759 and Runtime763–768, plus adjacent
  Runtime/Editor capacity guards, pass `73/73` in `0.889s`; this remains local source/model
  evidence only, with managed Cargo/Release allocation and product input percentile gates pending.
- Runtime769 adds direct-bound Array-to-Flags selection normalization to the same one-process
  scope. The source contracts for Runtime755–759 and Runtime763–769, plus adjacent Runtime/Editor
  capacity guards, pass `77/77` in `0.462s`; this remains local source/model evidence only, with
  managed Cargo/Release allocation and product selection/input percentile gates pending.
- Runtime770 adds single-resolution collection Array/Map mutation paths to the same one-process
  scope. The source contracts for Runtime755–759 and Runtime763–770, plus adjacent Runtime/Editor
  capacity guards, pass `82/82` in `0.829s`; this remains local source/model evidence only, with
  managed Cargo/Release allocation and product collection-edit percentile gates pending.
- Runtime771/772 now participate in the refreshed shared scope. The source contracts for
  Runtime755–759, Runtime763–781, and adjacent Runtime/Editor guards pass `120/120` in `0.080s`;
  this remains local source/model evidence only, with managed Cargo/Release allocation and
  product table/notification percentile gates pending.
- Runtime773/774 add shared text-search Unicode prefix/contains streaming helpers. The focused
  prefix/contains/shared-search contracts pass `14/14`, and the refreshed Runtime/Editor batch
  passes `120/120` in `0.080s`; lower Unicode expansion/overlap coverage and their two Release
  markers are wired for the deferred managed lane. This remains local source/model evidence only; managed
  Cargo/Release allocation and product menu/typeahead/filter percentile gates remain pending.
- Runtime775 adds one-buffer menu typeahead text normalization. Its source contract passes `3/3`,
  focused menu/typeahead/search batch passes `21/21`, and the refreshed Runtime/Editor batch
  passes `120/120` in `0.080s`; lower control/Unicode whitespace/case-expansion equivalence
  coverage and its Release marker are wired for the deferred managed lane. This remains local
  source/model evidence only; managed Cargo/Release allocation and product keyboard percentile
  gates remain pending.
- Runtime776 moves the prior typeahead buffer into the combined search and appends the new payload
  in place. Its source contract passes `3/3`, focused menu/typeahead/search batch passes `18/18`
  in `0.041s`, and the refreshed Runtime/Editor batch passes `120/120` in `0.080s`; lower
  active/expired/repeated-key coverage plus its Release marker are wired for the deferred managed
  lane. This remains local source/model evidence only; managed
  Cargo/Release allocation and product keyboard percentile gates remain pending.
- Runtime777 reduces each menu typeahead candidate to one owned buffer and checks at most two
  Unicode scalars to determine whether it is multi-scalar. Its source contract passes `4/4`, and
  the refreshed Runtime/Editor batch passes `120/120` in `0.080s`; lower pasted/active/fallback/
  Unicode-expansion coverage plus its Release marker are wired for the deferred managed lane. This
  remains local source/model evidence only; managed Cargo/Release allocation and product keyboard
  percentile gates remain pending.
- Runtime778 makes menu search-query normalization borrow the selected state/default setting and
  trim before lowercasing. Its source contract passes `2/2`, and the refreshed Runtime/Editor batch
  passes `120/120` in `0.080s`; lower precedence/whitespace coverage plus its Release marker are
  wired for the deferred managed lane. This remains local source/model evidence only; managed
  Cargo/Release allocation and product menu-search percentile gates remain pending.
- Runtime779 streams recursive menu-filter matches into one shared accumulator and rolls back
  unmatched branches by checkpoint. Its source contract passes `3/3`, and the refreshed
  Runtime/Editor batch passes `120/120` in `0.080s`; lower preorder/focus-order coverage plus its
  Release marker are wired for the deferred managed lane. This remains local source/model evidence
  only; managed Cargo/Release allocation and product menu-search percentile gates remain pending.
- Runtime780 makes menu label lookup borrow source map text and clone only when a retained option
  takes ownership. Its source contract passes `3/3`, and the refreshed Runtime/Editor batch passes
  `120/120` in `0.080s`; lower label/fallback coverage plus its Release marker are wired for the
  deferred managed lane. This remains local source/model evidence only; managed Cargo/Release
  allocation and product menu-search percentile gates remain pending.
- Runtime781 replaces the per-node borrowed child-value vector with a lazy iterator over the
  canonical child-property names. Its source contract passes `3/3`, and the refreshed
  Runtime/Editor batch passes `120/120` in `0.080s`; lower property-order coverage plus its
  Release marker are wired for the deferred managed lane. This remains local source/model
  evidence only; managed Cargo/Release allocation and product menu-search percentile gates
  remain pending.
- Runtime782 removes the v2 surface-tree interaction module's private owned showcase registry and
  resolves capabilities through the process-wide shared catalog. Its source contract passes `3/3`;
  the lower pointer-identity regression and `RUNTIME782_SURFACE_TREE_SHARED_CATALOG_BENCH_V1`
  marker are wired for the next combined Runtime/Editor lane. This remains local source/model
  evidence only; managed Cargo/Release allocation and product interaction percentile gates remain
  pending.
- Runtime783 routes popup/dialog fixed-key publication through an in-place overlay setter, removing
  temporary key allocations for already materialized state while preserving reference-source
  invalidation. Its source contract passes `3/3`; the lower key-identity/reference-source
  regression and `RUNTIME783_OVERLAY_STATIC_KEY_UPDATE_BENCH_V1` marker are wired for the next
  combined Runtime/Editor lane. This remains local source/model evidence only; managed Cargo/Release
  allocation and product popup percentile gates remain pending.
- Runtime784 routes notification selected-id, focused-index, and unread-count publication through an
  in-place setter, removing temporary fixed-key allocations while preserving reference-source
  invalidation and navigation semantics. Its source contract passes `3/3`; the lower key-identity/
  reference-source regression and `RUNTIME784_NOTIFICATION_STATIC_KEY_UPDATE_BENCH_V1` marker are
  wired for the next combined Runtime/Editor lane. This remains local source/model evidence only;
  managed Cargo/Release allocation and product notification percentile gates remain pending.
- The pre-783 batched current-source Runtime/Editor loader completed `542` modules and `1945` tests in
  `51.776s`, with `0` failures, errors, or skips. This receipt includes Runtime782 and the preceding
  slices but was launched before Runtime783 landed; it is local source/model evidence only and does
  not replace managed Cargo/Release allocation or product percentile evidence.
- A subsequent current-source Runtime/Editor loader completed `543` modules and `1948` tests in
  `42.622s`, with `0` failures, errors, or skips. This batch includes Runtime782 and Runtime783;
  it is local source/model evidence only and does not replace managed Cargo/Release allocation or
  product percentile evidence.
- A fresh current-source rerun completed the same `543` modules and `1948` tests in `6.362s`, again
  with `0` failures, errors, or skips. This confirms the current source-contract batch after the
  record updates; it remains local source/model evidence only.
- The pre-784 full non-tooling Runtime/Editor discovery completed `906` modules and `3697` tests in
  `364.761s`, with `0` failures, errors, or skips. It was launched before Runtime784 landed, so the
  current 784 source is covered by its focused contract and the current merged batch; this receipt is
  local regression evidence only.
- The current-source Runtime/Editor performance-contract loader completed `544` modules and `1951`
  tests in `8.523s`, with `0` failures, errors, or skips. This batch includes Runtime784 (pre-785) and remains
  local source/model evidence rather than managed Cargo/Release or product percentile acceptance.
- The focused Runtime748–783 and Editor760–762 source-contract set completed `142/142` tests in
  `0.108s` in one process, covering the recent Runtime hot paths, overlay/catalog slices, and
  Editor capacity guards. This is local source/model evidence only; managed Cargo/Release and
  product percentile evidence remain pending.
- The merged focused Runtime/Editor hot-path set including Runtime784 completed `145/145` tests in
  `0.150s` in one process. This is local source/model evidence only; managed Cargo/Release and
  product percentile evidence remain pending.
- The batched Runtime/Editor input and compile-contract probe covered `24` modules and passed
  `176/176` tests in `8.157s`, with zero failures, errors, or skips. This is source-contract
  evidence only and does not represent a Rust Cargo compile.
- Runtime755/756 reserve the parsed-entry upper bound for command-palette filtered IDs and stream
  nested command leaves into one root output vector, retaining source/query short-circuit order,
  field interpretation, duplicate order, and presentation state behavior. Their contracts,
  adjacent Runtime command-palette/query/index guards, and recent Runtime/Editor capacity
  contracts pass `38/38` in `0.427s` in one Python process. Scoped Rustfmt passes. This local
  receipt succeeds the earlier `1852/1852` source-contract snapshot, which predates both slices;
  it does not replace managed Cargo/Release allocation or product percentile evidence.
- The latest current-source Runtime UI plus Editor asset contract batch passes
  `623/623` in `15.573s`; the focused index/input/asset set passes `37/37` in
  `0.039s`. The broader Runtime/Editor performance-contract and pressure
  discovery passes `1364/1364` in `8.303s` in one process. These are local
  source/model receipts and do not replace managed Cargo or Release measurements.
- Editor760's AssetContent visible-group capacity contract is included in the
  current single-process Runtime/Editor batch. The slice reserves clipped
  `node_rows` before the shared non-virtual append while leaving virtualized
  materialization unchanged; its managed Cargo/Release and product percentile
  gate remains pending.
- Editor761's timeline-ruler capacity contract is included in that same
  single-process batch. It reserves the derived interval bound plus endpoint
  slots without changing the 4,096 cap or tick ordering; managed Cargo/Release
  and product percentile evidence remain pending.
- The post-Editor760/761 merged Runtime/Editor performance-contract batch loads
  `515` modules and passes `1844/1844` in `47.661s`; the focused capacity set
  passes `21/21` in `0.127s`. Both are local source/model receipts and do not
  replace managed Cargo/Release or product percentile acceptance.
- Editor762 then added the page-chrome visible-tab prefix reservation while
  preserving active-tab fallback, deduplication, clipping, order, and overflow.
  The refreshed one-process Runtime/Editor performance-contract loader includes
  it, loads `516` modules, and passes `1848/1848` in `103.121s`; the focused
  six-contract capacity set passes `25/25` in `0.479s`; Wiki validation passes
  `272/272` pages with one pre-existing metadata warning. These are intermediate
  source/model receipts; managed Cargo/Release and product percentile evidence
  remain pending.
- The page-tab reserve was then tightened to the finite tab-lane minimum-slot
  bound, avoiding a full model-count allocation for a fixed-width page bar.
  Its refreshed focused source/lower regression passes `4/4`; the `1848/1848`
  shared receipt predates this tightening and is superseded by the final
  `1852/1852` batch below.
- Runtime754 then bounded secure-text presentation allocations: a mask-safe
  display-string bound, the materialized hard-line count, the source-byte
  cluster bound, and each consumed grapheme iterator's upper bound now cover
  the retained projections. Mask/separator/source-offset and bidi behavior remain unchanged. Its
  focused contract is included in the seven-contract `29/29` capacity batch;
  the lower regression and ignored `RUNTIME754_SECURE_TEXT_PRESENTATION_CAPACITY_BENCH_V1`
  marker join the same deferred managed lane. The final merged count below also
  includes this Runtime contract.
- The final current-source one-process Runtime/Editor performance-contract
  loader includes Editor762's finite-lane tightening and Runtime754, covers
  `517` modules, and passes `1852/1852` in `159.897s`. The focused seven-contract
  capacity batch passes `29/29` in `0.513s`; Python compilation, scoped Rustfmt,
  diff checks, and Wiki validation remain green. These are local source/model
  receipts only and do not replace managed Cargo/Release or product percentile
  evidence.
- The long-running full non-tooling Runtime/Editor discovery launched before
  Editor761's contract file was added completed `3589/3589` tests in `2970.014s`.
  It is a broad regression receipt only; the post-Editor761 `1844/1844`
  performance-contract batch predates Editor762, while the intermediate
  `1848/1848` batch predates the finite-lane tightening and Runtime754. The
  final `1852/1852` batch above is the current source/model receipt, and managed
  Cargo/Release/product percentile evidence remains pending.
- After the Runtime742/Editor738 updates, the current combined Runtime/Editor
  performance-contract and pressure loader covers 356 modules and passes
  `1368/1368` in `33.787s` in one process. The comprehensive non-tooling
  Runtime/Editor loader then covered 548 files and passed `2039/2039` in
  `22.986s`; both are local source/model evidence, while managed Windows
  Cargo/Release allocation and product p50/p95/p99 measurements remain pending.
- Latest batched Runtime + Editor + Runtime Text performance-contract runs:
  Runtime `1146/1146` passed in `3.768s`, Editor `581/581` in `0.848s`, and
  Runtime Text `143/143` in `0.934s` (`1870/1870` aggregate).
- The batched Runtime/Editor pressure-model runs passed `154/154` and
  `129/129` respectively (`283/283` aggregate; `2.626s` and `2.325s`).
- A post-Editor607/accessibility-resolver rerun on 2026-09-12 passed Runtime
  performance `1146/1146` in `4.528s`, Editor performance `581/581` in
  `1.042s`, and the full pressure batches `154/154` and `129/129` in
  `2.494s` and `2.216s`; the focused accessibility/rich-text contracts passed
  `24/24`.
- A single unified local loader reran the four performance/pressure discoveries
  plus the focused accessibility, rich-text, and Editor607 consumer contracts:
  the deduplicated aggregate `2041/2041` passed in `10.835s`.
- After Runtime711 switched child membership to the published map, the same
  deduplicated loader reran and passed `2041/2041` in `11.943s`; this is the
  latest local aggregate for the current source snapshot.
- The current single-invocation Runtime/Editor performance-plus-pressure batch loaded
  535 matching modules and passed `1991/1991` in `8.659s`.
- After the Runtime712 dirty-summary destination extension and the parallel Editor713 cache
  remap, the refreshed single-invocation batch loaded 536 modules and passed `1993/1993` in
  `12.193s`; this remains local static evidence only.
- After the Runtime714 text-decoration DTO fallback, the same batch was rerun in one
  invocation: 536 modules, `1993/1993` passed in `12.015s`; this is the current local
  snapshot and remains static evidence only.
- The Runtime717 inline-widget capacity slice was then checked in one focused invocation with
  text/infrastructure, rich-inline geometry, layout, navigation, and surface contracts:
  `105/105` passed in `5.900s`; this does not replace the broader aggregate and remains local
  source/contract evidence.
- A fresh single-invocation non-tooling Runtime/Editor performance-plus-pressure loader covering
  343 modules passed `1320/1320` tests in `79.827s` after Runtime717; no coordinator or Cargo
  process was involved.
- A later all-contract non-tooling Runtime/Editor discovery loaded 558 modules and 2308 tests;
  it retained five unrelated boundary/naming/tech-stack failures (three runtime naming/legacy
  ownership checks and two tech-stack anchor checks). No tooling or unrelated migration owner
  was changed to hide those failures.
- After the Editor718 lazy hierarchy-flag extension, the same non-tooling
  performance-plus-pressure batch reran across 343 modules and passed `1320/1320` in `10.083s`;
  this is shared static evidence and does not replace the managed Cargo gate.
- The final current-source rerun, including Runtime720/721 and Editor719/722, covered the same
  343 modules and passed `1320/1320` in `94.188s`; this remains source/contract evidence only.
- Runtime720 then reserved the retained node-count upper bound and removed per-node component-flag
  clones in ECS UI projection; the corrected focused hierarchy/editor contract batch passed
  `57/57`, with no DTO or ordering changes. This remains source/contract evidence only.
- Runtime721 then moved host-font reference deduplication ahead of ownership: cache hits no longer
  allocate an Arc per duplicate/input reference. Existing host-font lifecycle coverage and the
  same focused source-contract batch remain green; this is source/contract evidence only.
- Runtime723 then reserved the known arranged-node count for hit-route publication and the
  retained node-id count for arranged-visibility rebuild. Production capacity regressions and
  route/visibility source guards pass (`18/18` focused), and the broader route/visibility/surface
  targeted batch passed `150/150` across 31 modules in `33.702s`; the final one-process
  non-tooling Runtime/Editor batch then covered 343 modules and passed `1320/1320` in `5.353s`.
  This remains source/contract evidence only and joins the same deferred managed gate without a
  new coordinator request or status query.
- Runtime726 then reused the parent index discovered during the full hit-route validation walk,
  removing the duplicate composition lookup. The focused route/visibility/activity contracts
  passed `20/20`, and the refreshed one-process non-tooling Runtime/Editor batch covered 347
  modules with `1333/1333` tests passing in `7.969s`; this remains source/contract evidence only.
- Runtime727 then reserved the draw-order upper bound for hit-grid entry projection, reused the
  arranged-node index for route entry selection, and replaced the closure/collect path with an
  explicit fail-closed loop. The focused route/visibility/activity
  contracts remained `20/20`, and the refreshed one-process non-tooling Runtime/Editor batch covered
  347 modules with `1333/1333` tests passing in `7.969s`; this remains source/contract evidence only.
- Runtime728 then added a scalar-first pointer-state accumulator: duplicate updates for one node
  stay allocation-free, while a second distinct node promotes to the existing deterministic
  `BTreeSet` batch path; true multi-node batches reserve their bounded root output. The
  owner-structure/route/visibility/activity/hit-grid batch passes
  `32/32`, and the refreshed one-process non-tooling Runtime/Editor performance-plus-pressure
  loader covers 347 modules with `1333/1333` tests passing in `6.334s`. No coordinator request or
  status query was issued.
- The Runtime662/664 text projection contract was then made tolerant of rustfmt's multiline
  const-generic call formatting without weakening the source guard. The final combined
  non-tooling Runtime/Editor performance-plus-pressure batch (including that text contract)
  covers 348 modules and passes `1338/1338` in `5.388s`; this remains local source/contract
  evidence only.
- Runtime729 changed the shared bounded cell projection from an eager per-entry `Vec<usize>` to
  a lazy row-major iterator. Base and projected full rebuilds consume it directly; geometry patch
  paths collect only their retained changed-entry memberships. The new iterator contract and
  adjacent hit-grid contracts pass, and the final one-process non-tooling Runtime/Editor batch
  covers 349 modules with `1341/1341` tests passing in `8.415s`; this remains local
  source/contract evidence only.
- Runtime730 then retained `entry_cells` map nodes and per-entry vector capacity for stable node IDs
  during reverse-map rebuilds. The lower capacity regression and reverse-map source contracts pass;
  the combined Runtime/Editor batch covered 351 modules and passed `1345/1345` tests in `39.080s`;
  this remains local source/contract evidence until the managed Windows Release allocation and
  latency gates run.
- Runtime731 then moved navigation tab position lookups to reusable `HashMap` buckets. Sorted
  candidate vectors remain the deterministic ordering authority; stable group/root buckets are
  retained and stale scopes are pruned. The same single-process non-tooling Runtime/Editor batch
  covered 351 modules and passed `1347/1347` tests in `64.780s` (earlier warm runs completed in
  `10.868s` and `121.072s`); the navigation contract passed
  `7/7`, with no new coordinator request or status query.
- Runtime732 then removed the remaining stable-group key clone from navigation position-map
  rebuilds: existing nested buckets are updated through `get_mut`, and only new groups clone an
  owned `UiNavigationGroupId` for insertion. The source contract passed `8/8`, and the latest
  same-source batched Runtime/Editor UI contract invocation loaded 172 modules and passed
  `772/772` tests in `1.571s`; this remains local evidence with no coordinator request or status
  query.
- Runtime733 then folded first directional group-target selection into that same retained-node
  stream. It compares each candidate against the current winner and removes the temporary
  per-group vector, per-candidate key clones, and second per-group sort. The updated navigation
  source contract passes `9/9`; the same batched UI invocation remains `772/772` across 172
  modules in `1.571s`, with no coordinator request or status query.
- Runtime734 then retained modal/MUI candidate vectors across navigation rebuilds, pruning only
  empty scopes and appending stable group candidates through borrowed-key lookup. Its navigation
  source contract passes `11/11`; the same batched UI invocation remains `772/772` across 172
  modules in `1.571s`, with no coordinator request or status query.
- Runtime735 then retained `first_candidate_by_group` entries and owned group-key buffers across
  rebuilds with a `seen` reset/prune lifecycle. Its navigation source contract passes `12/12`,
  the lower key-identity/capacity regression shape is green, and the batched Runtime/Editor UI
  invocation covers 21 modules with `105/105` tests passing in `25.523s`; this remains local
  source evidence with no new coordinator request or status query.
- Runtime736 then retained transactional virtual-list candidate slot/key/generation buffers per
  owner. Warm reconciliation uses `clone_from` plus publication swaps, while protected-slot
  rejection leaves the published assignment unchanged. The updated virtual-list contracts are
  included in the one-process Runtime/Editor UI batch; the current 172-module rerun passes
  `773/773` in `1.514s`. Managed Release allocation/latency gates remain pending, with no new
  coordinator request or status query.
- Runtime737 then moved the desired component/control-id/path fields into an owned node-pool
  lookup key and restored them before retained-child merge; residency reporting now combines
  node and bucket counts in one walk. The focused node-pool/layout-order/
  incremental-layout contracts pass `27/27`; the current one-process Runtime/Editor UI batch
  covers 173 modules and passes `776/776` in `1.962s`. This is local source evidence only; the
  managed Release allocation and node-pool/navigation latency gates remain pending, with no new
  coordinator request or status query.
- A broader non-tooling Runtime/Editor performance-contract and pressure loader then covered
  548 modules and passed `2057/2057` tests in `15.174s`; this is local static evidence only and
  leaves the same managed Release allocation/latency gate pending.
- Runtime738 then switched the lookup-only reflection node index to a capacity-retaining
  `HashMap`, reserving only missing capacity during rebuild while preserving ordered
  tree/node insertion and duplicate-path precedence. The focused Runtime UI
  node-pool/reflection/layout batch passes `30/30`, and the direct 91-module Runtime UI batch
  passes `466/466` in `15.490s`; this joins the existing managed gate without a new coordinator
  request or status query.
- Editor724 added registry-length reservations to activity view/window snapshots; its refreshed
  single-invocation baseline covered 347 modules and passed `1331/1331` tests in `6.361s`.
- The focused Runtime/Editor UI dirty/layout/surface/tree/hit/invalidation batch passed
  `240/240` across 48 modules in `28.741s`.
- A broader non-tooling Runtime/Editor discovery ran `3468` tests and retained eight
  unrelated boundary/naming/tech-stack failures; no tooling or unrelated owner files were
  modified in response.
- The prior full Runtime-only performance-contract discovery passed
  `1146/1146` in `3.768s`, including the repaired benchmark fixtures in records
  690–691; the post-Editor607 rerun passed the same `1146/1146` in `4.528s`.
- Latest focused pointer/navigation/input/hit-query batch: `132/132` passed
  in `0.143s`.
- Runtime UI performance-contract batch: `213/213` passed in `0.371s`,
  including the scrollbar target-index and hit-grid contracts affected by
  records 686–689. Runtime Text contracts additionally passed `143/143` in
  `0.766s` for the intrinsic-measurement and rich-layout repairs.
- New navigation/menu/pointer/hit-test capacity source guards: `4/4` passed;
  test-import/API repair guards passed `6/6` and `5/5` in their respective
  batches.
- Accessibility resolution key-scratch source and focused guards passed
  `4/4`; the Runtime711 child-filter map-membership guard passed `3/3` in the
  same deferred batch, which also included the Runtime UI accessibility
  extraction pressure path.
- Scoped Rustfmt checks and `git diff --check` passed for the touched
  production modules; Git emitted only existing line-ending notices.
- The current-source Runtime206 follow-up batch (records 743–746), together
  with the Editor742 row-capacity contract, was rerun in one process: `555`
  Runtime/Editor modules and `2065/2065` tests passed in `24.966s`. This is
  local source/model evidence only; managed Cargo/Release allocation and
  product latency gates remain pending.
- After Editor743, the shared one-process non-tooling Runtime/Editor
  performance-plus-pressure loader covers `556` modules and passes `2068/2068`
  tests in `29.116s`; the Runtime743–746 slices and Editor742/743 contracts
  are included. Runtime747's separate naming audit also passes `6/6` after its
  wording repair. These are local source/model receipts only; managed
  Cargo/Release and product latency gates remain pending.
- The current all-contract non-tooling discovery loaded `868` modules and ran
  `3553` tests in `561.258s`; Runtime naming now passes, and the only
  remaining failure is the deferred WOC dependency assertion
  (`test_woc_consumes_runtime_preference_service_without_sync_compat_trait`).
  A focused Runtime206/Runtime85 plus Editor reference invocation passes `37/37` in
  `111.865s` in one process.
- The full discovery reported four failures in `tooling32_*` contracts. Those
  are outside this batch and tooling is intentionally deferred per the task
  direction; no tooling source was changed. The failing modules are
  `test_tooling32_ai_effort_narrow_stream_performance_contract`,
  `test_tooling32_cleanup_interrupted_index_performance_contract`,
  `test_tooling32_integration_candidate_lease_index_performance_contract`, and
  `test_tooling32_snapshot_patch_batch_performance_contract`.
- Runtime748 then moved the large-path pointer hover membership set into
  surface-owned scratch. The new source contract and the existing hover
  diff/pressure contracts pass `13/13` in one invocation; the deterministic
  warm-allocation model and ignored Release marker are recorded without
  claiming managed latency or allocation acceptance. The post-change broad
  Runtime/Editor performance-plus-pressure loader covers `557` modules and
  passes `2072/2072` in `13.644s`.
- A subsequent all-current Runtime/Editor Python contract invocation covered
  `869` modules and passed `3557/3557` tests in `337.155s` in one process.
  This broad source-contract receipt is local evidence only and does not
  replace managed Rust or product performance acceptance.
- Runtime749 extends that same batch with Touch/Pen primary-pointer source
  counters; the all-current invocation now covers `870` modules and passes
  `3562/3562` tests in `396.106s` in one process. The lower Rust regression,
  Release marker, and managed pointer-input percentile gate remain pending.
- A post-regression-edit confirmation of the same batched scope again covered
  `870` modules and passed `3562/3562` with zero failures, errors, or skips in
  `517.932s`; this loader wall-clock receipt is not a product p50/p95/p99
  measurement.
- The Runtime750 route-terminal-state-bit and Editor744 history-projection
  slices were then validated together with the adjacent pointer/reference/
  history/input contracts: the focused cross-surface invocation passed
  `73/73`; separate current-source performance-contract discoveries passed
  Editor `622/622` and Runtime `1198/1198`. These are local source/model
  receipts; managed Cargo/Release allocation and product p50/p95/p99 gates
  remain pending.
- Runtime751's generic route preview-path borrow joined that same batch; the
  refreshed focused invocation passed `76/76`, while current Runtime and
  Editor performance-contract discoveries passed `1201/1201` and `622/622`.
  These local receipts do not replace the managed Release gate.
- A fresh single-process cross-surface performance-contract run on the current
  tree loaded both Runtime and Editor discovery patterns and passed `1823/1823`
  tests in `22.612s`. The three new Python contracts also pass `py_compile`,
  the eight touched Rust files pass `rustfmt --edition 2021 --check`, and wiki
  validation reports `272/272` pages with zero errors (one pre-existing missing
  skill-path warning). This remains local source/model evidence; managed Cargo,
  Release allocation, and product p50/p95/p99 gates remain pending.
- After Runtime752, the same batched Runtime/Editor performance-contract loader
  passed `1827/1827` tests in `10.183s`. The effect-result capacity contract is
  included; scoped Rustfmt, `py_compile`, and wiki validation remain green.
  This is still local source/model evidence and does not advance the managed
  Cargo/Release or product percentile gate.
- After Runtime753, the same one-process Runtime/Editor loader passed
  `1831/1831` tests in `9.169s`; both new Runtime77 capacity contracts are
  included. Scoped Rustfmt, `py_compile`, and wiki validation remain green,
  while managed Cargo/Release and product percentile evidence stay pending.
- After retaining text `widget_events` in the pointer/text merge, the same
  one-process Runtime/Editor performance-contract loader passed `1832/1832`
  in `7.609s`; the managed Cargo/Release and product percentile gate remains
  pending.
- The full non-tooling Runtime/Editor Python discovery then completed with
  `3581/3581` tests passing in `441.896s` in one process. This remains local
  regression evidence; managed Cargo/Release and product p50/p95/p99 evidence
  is still pending.
- The post-Editor745 shared Runtime/Editor performance-contract loader passed
  `1836/1836` in `9.621s` in one process, keeping the Runtime capacity slices
  green while adding the Activity projection contract. This is local source/model
  evidence only; managed Cargo/Release and product percentile gates remain pending.
- A later current-source rerun of the same shared loader passed `1836/1836` in
  `14.186s` after the Editor745 contract assertion tightening. This remains local
  source/model evidence and does not replace managed Cargo/Release acceptance.
- The latest current-source non-tooling Runtime/Editor discovery passed
  `3585/3585` tests in `775.037s` in one process. This fresh batched regression
  receipt remains local evidence; managed Cargo/Release and product percentile
  acceptance are still pending.
- The subsequent full non-tooling Runtime/Editor discovery passed `3585/3585`
  tests in `478.170s` in one process. This confirms the cross-surface Python
  regression batch, but remains local evidence rather than managed Cargo/Release
  or product p50/p95/p99 acceptance.
- The Editor745 lower harness also ran independently under optimized `rustc` and
  reported synthetic p95 `65800ns -> 38100ns` with modeled growth `11 -> 0`.
  This belongs to the Editor slice and is not a Runtime product measurement.

## Current local closeout receipt (2026-09-18)

After Runtime800, the latest single-process Runtime/Editor performance-contract
loader passes
`2010/2010` tests in `10.198s`, with zero failures, errors, or skips. The
focused Runtime793/Runtime794/Runtime795/Runtime796/Runtime797/Runtime799/
Runtime800/Editor790/Editor798 set passes `29/29`; the additional Runtime02/08/19 and
Editor09 focused source batch passes `39/39` in `97.492s`. The last broad
all-surface `test_*performance_contract.py` discovery also passes `2438/2438`
in `92.731s` on the post-sample-count rerun (the earlier receipt was `64.061s`);
this includes existing tooling contracts as read-only evidence,
while tooling production remains unchanged and deferred. The prior broad
non-tooling Runtime/Editor
regression receipt remains the Runtime794/Editor790 baseline (`921` files,
`3740/3740` tests in `882.184s`); it was not rerun for the newer source-contract
files. Runtime786 and Runtime788 capacity models report zero reserved-growth
events (216 and 15 legacy events, respectively); the Scene Picker parity model
passes 60,000 randomized cases; Runtime799's selector model removes one
temporary allocation and 1,048,576 parameter probes per 1,024-child selection;
Runtime800's package model removes 256 temporary package vectors per 256-package
build (256 → 0).
Rustfmt, Wiki, scoped plan-path references and current source-fingerprint checks,
trailing-whitespace, and scoped diff checks pass. These are local source/model
receipts; managed Cargo/Release, allocator, and product p50/p95/p99 gates remain
pending. Tooling remains intentionally deferred.

The current-source batched rerun is the authoritative local receipt for the
new records and is kept as one combined invocation rather than per-task
validation. The prior broad regression receipt is retained as a clearly labeled
baseline; neither receipt replaces managed Cargo/Release or product percentile
acceptance.

A final same-source rerun under shared-workspace load also passed `2002/2002`
with zero failures, errors, or skips in `112.795s`; that historical receipt
predates Runtime800 and is retained only as a contention observation. The
`10.198s` result above is the authoritative post-Runtime800 local batch. These
elapsed times are Python harness/host observations, not product p50/p95/p99
measurements.

### Latest batched source-contract refresh (2026-09-18)

The earlier Runtime802/Runtime799/800 plus Editor787/789/790/798 recent-slice set passed
`66/66` across `19` modules in `0.063s`. The current single-process non-tooling discovery,
including the Editor799 viewport-toolbar projection contract, passes `2205/2205` across `598`
modules in `6.538s`, with zero failures, errors, or skips. The lower regression files mounted by
the prior batch pass the exact scoped Rustfmt check; these local receipts remain source/model
evidence and do not replace managed Cargo, Windows Release allocation, or product percentile
gates.

The subsequent current-source batch added the Editor800 autosave diagnostic-capacity contract and
passes `2208/2208` across `599` non-tooling modules in `8.264s`, with zero failures, errors, or
skips. It is retained as one combined local receipt rather than per-task validation; managed
Cargo/Release, allocator, and product percentile gates remain pending.

The Runtime803 benchmark-evidence slice then passed its focused cross-file source contract `2/2`
and the exact scoped Rustfmt check for all six Runtime74 probe owners. The merged non-tooling
source-contract batch, explicitly including Runtime803, passes `2210/2210` across `600` files in
`7.592s`, with zero failures, errors, or skips. It changes only ignored probe sample shape and
marker reporting (101 alternating samples, 51/50 order metadata, and P50/P95/P99 fields); no
production algorithm or threshold was changed. The managed Release, complete-caller, allocator,
and product percentile gates remain pending.

### Runtime804 empty dispatch-output fast path (2026-09-19)

`RuntimeUiSurfaceSet::record_dispatch_outputs` now checks its two owned output vectors before
surface lookup and `UiTreeId` cloning. Empty host/component results therefore avoid one owned
tree-id clone and two empty queue walks per routed event; non-empty host requests, component
actions, and secure-text revocation retain their existing order and limits. The focused
source/model contract passes `4/4`. A single merged non-tooling contract invocation loaded
`870` files and passed `3665/3665` tests with zero failures, errors, or skips in `40.083s`.
This is local source/model evidence only; managed Cargo/Release allocation and product input
p50/p95/p99 gates remain pending.

### Runtime807 action revocation scratch capacity (2026-09-19)

`RuntimeUiActionRequestQueue::record_result` now keeps the secure-reference revoke vector at
zero capacity until an actual value is revoked. The first append reserves the saturating
remaining `component_events` bound, eliminating geometric growth on revoke-heavy batches while
avoiding a revocation-scratch allocation when every action is admitted. All revoke/reject branches use
the helper; queue limits, redaction, supersession, ordering, and request admission are unchanged.
The focused source/model contract passes `4/4`; the merged non-tooling invocation loaded `873`
files and passed `3677/3677` tests with zero failures, errors, or skips in `205.214s`. This is
structural allocation-shape evidence only; managed Cargo/Release and product input p50/p95/p99
gates remain pending.

### Runtime808 font-admission dependency projection (2026-09-19)

Cross-surface UI font dependencies now collect into one sorted/deduplicated
contiguous vector instead of a temporary `BTreeSet`. Lexical order, duplicate
collapse, prepared admission order, claim replacement, failure handling, and
profile counters remain unchanged. The focused source/model contract passes
`4/4`; the lower source regression and ignored
`RUNTIME808_FONT_ADMISSION_DEPENDENCY_BUFFER_BENCH_V1` marker are wired. The
combined eight-slice Runtime/Editor focused batch passes `32/32`; the current
one-process non-tooling loader passes `3693/3693` across `877` files in
`83.085s` with zero failures, errors, or skips. This is local source/model
evidence only; managed Cargo/Release, allocation, and font-admission product
percentile evidence remain pending.

### Runtime809 AccessKit tree projection capacity (2026-09-19)

The AccessKit adapter now reserves the exact snapshot-node bound plus the
optional synthetic root before projecting nodes. Synthetic-root children and
each non-empty accessibility node's direct children likewise reserve their
source lengths before appending, preserving source order, root/focus fallback,
and all role/state/action/relation mappings. The focused source/model contract
passes `4/4`; the lower source regression and ignored
`RUNTIME809_ACCESSKIT_TREE_PROJECTION_CAPACITY_BENCH_V1` marker are wired. The
twelve-slice Runtime/Editor focused batch passes `48/48`; the strict
non-tooling performance/pressure batch passes `2643/2643` across `682` files
in `51.429s`. This is allocation-shape evidence only;
managed Cargo/Release, allocator, and AccessKit product p50/p95/p99 gates
remain pending.

### Runtime820 dispatch host-request capacity (2026-09-19)

The Runtime820 follow-up to the Runtime177 single-pass collector now reserves
the remaining-result lower bound when the first host request appears. Its
focused source/model contract passes `3/3`; the lower empty/order/sparse
regressions and ignored `RUNTIME820_DISPATCH_HOST_REQUEST_CAPACITY_BENCH_V1`
marker are wired. The deterministic 4,096-result model changes eleven
geometric growth events to zero with one reservation while request-free batches
remain zero-capacity. Managed Cargo/Release, allocator, and dispatch product
p50/p95/p99 evidence remain pending.

### Runtime821 focus hovered-path in-place retention (2026-09-19)

Runtime200 focus cleanup now moves `focus.hovered` out with `mem::take`,
filters it in place with `retain`, and restores the same allocation. Input-owner
validation, duplicate entries, source order, and empty behavior are unchanged;
tree-change reconciliation no longer allocates a replacement vector when the
hover path is stable. The focused source/model contract passes `4/4`, the lower
order/capacity regression and ignored
`RUNTIME821_FOCUS_HOVERED_RETAIN_BENCH_V1` marker are wired, and the deterministic
4,096-entry model changes one replacement allocation per reconciliation to zero
new allocations. Managed Cargo/Release, allocator, and input product
p50/p95/p99 evidence remain pending.

The post-Runtime821 one-process Runtime/Editor source-contract batch loads `393`
modules and passes `1416/1416` tests. The broader current non-tooling batch
loads `619` modules and passes `2210/2210` tests, with zero failures, errors, or
skips. These are local source/model receipts only; managed Cargo/Release,
allocator, and product percentile gates remain pending.

### Runtime822 focus pointer-drag owner in-place retention (2026-09-19)

Runtime200 focus cleanup now moves `input.pointer_drags` out with `mem::take`,
filters invalid owners in place with `BTreeMap::retain`, and restores the same
map. Valid drag payloads, key order, and input-owner validation are unchanged;
the temporary invalid-owner vector and per-owner second-pass removals are gone.
The focused source/model contract passes `4/4`, the lower semantic regression
and ignored `RUNTIME822_FOCUS_POINTER_DRAG_RETAIN_BENCH_V1` marker are wired, and
the deterministic 4,096-entry model changes one temporary vector plus repeated
invalid-owner lookups per reconciliation to zero temporary vectors plus one map
traversal. The current one-process non-tooling Runtime/Editor batch loads `621`
modules and passes `2218/2218` tests with zero failures, errors, or skips.
Managed Cargo/Release, allocator, and input product p50/p95/p99 evidence
remain pending.

### Runtime824 input timer drain capacity (2026-09-19)

The four existing retain drains now capture their map length and lazily reserve
the returned expiration vector on the first expired item. Typeahead, submenu,
tooltip, and toast order/payload/pending semantics remain unchanged, while a
no-expiry tick keeps zero capacity. The focused source/model contract passes
`3/3`; the lower all-four-kind regression and ignored
`RUNTIME824_INPUT_TIMER_DRAIN_CAPACITY_BENCH_V1` marker are wired. Managed
Cargo/Release, allocator, and input-product p50/p95/p99 evidence remain
pending.

### Runtime829 dependency cascade target capacity (2026-09-19)

UI asset reverse-dependency invalidation now reserves the current first-wave
fanout only after the first genuinely admitted dependent. The borrowed
`HashSet` visited index, `VecDeque` traversal, BTree-ordered sibling walk,
duplicate suppression, empty lookup, and self-cycle behavior are unchanged.
The focused source/model contract passes `4/4`; the lower fanout/empty/cycle
regression and ignored
`RUNTIME829_DEPENDENCY_CASCADE_TARGET_CAPACITY_BENCH_V1` marker are wired. The
4,096-target deterministic model changes `11` geometric growth events to `0`;
managed Cargo/Release, allocator, and dependency-cascade product p50/p95/p99
evidence remain pending. The latest one-process non-tooling Runtime/Editor
loader passes `2235/2235` across `626` modules, and the eight-slice focused
loader passes `29/29`, with zero failures, errors, or skips; these are local
source/model receipts only.

### Runtime833 surface frame patch-range capacity (2026-09-19)

`UiSurface::mark_surface_frame_rebuild_dirty` now reserves the exact
changed-node-set upper bound for its temporary render patch-range vector and
extends the existing command lookup directly into that buffer. `None` keeps
the full-snapshot fallback, an empty set retains zero capacity, and BTree
iteration order, missing-command filtering, range merging, and frame
publication authority are unchanged. The focused source/model contract passes
`2/2`; the lower source regression and ignored
`RUNTIME833_SURFACE_FRAME_PATCH_RANGE_CAPACITY_BENCH_V1` marker are wired. The
4,096-node deterministic model changes `12` geometric growth events to `0`.
Managed Cargo/Release, allocator, and surface-frame product p50/p95/p99
evidence remain pending.

### Runtime835 virtual geometry overlay capacity (2026-09-19)

Runtime835 extends the Runtime virtual-geometry debug-overlay projection with
bounded temporary-buffer reservations. BVH and visbuffer gizmo collectors now
reserve their input snapshot lengths, BVH lines reserve `13` entries per node
(12 box edges plus one valid parent connector), and the fixed visbuffer marker
reserves its `16` line segments. Direct extension preserves filtering, source
order, parent lookup, colors, geometry, and empty fallbacks. The focused
source/model contract passes `3/3`; the lower source regression and ignored
`RUNTIME835_VIRTUAL_GEOMETRY_OVERLAY_CAPACITY_BENCH_V1` marker are wired. Dense
4,096-node models change geometric growth events `15→0`, `11→0`, and `3→0`.
The one-process Runtime/Editor capacity/projection batch covers `171` files
and passes `632/632` tests in `4.660s`, with zero failures, errors, load
errors, or skips. Managed Cargo/Release, allocator, and overlay product
p50/p95/p99 evidence remain pending; no coordinator status is polled.

### Runtime838 accessibility root projection capacity (2026-09-19)

Runtime838 extends the Runtime accessibility snapshot extractor with a bounded
temporary-buffer reservation: the published root vector reserves the authored
`surface.tree.roots.len()` bound before the existing admission filters. Root
order, hidden and missing-root filtering, deadline/budget accounting, focus and
relation semantics, and empty-root zero-capacity behavior remain unchanged. The
focused source/model contract passes `2/2`; the lower Rust order/source
regression and ignored `RUNTIME838_ACCESSIBILITY_ROOT_CAPACITY_BENCH_V1` marker
are wired, and exact Rustfmt plus Python compilation pass. A dense 4,096-root
model changes geometric growth `11→0`. Managed Cargo/Release, allocator, and
accessibility product p50/p95/p99 evidence remain pending; no coordinator status
is polled.

### Runtime839 animation missing-track diagnostic capacity (2026-09-19)

Runtime839 keeps the compiled animation sequence's report-only `missing_tracks`
vector at zero capacity for successful compiles and reserves the known source
track-count bound only when the first missing entity or property writer is
admitted. Missing-path order and payloads, successful writer compilation,
sampling, and apply semantics remain unchanged. The focused source/model
contract passes `2/2`; the lower lazy-success/order regression and ignored
`RUNTIME839_ANIMATION_MISSING_TRACK_CAPACITY_BENCH_V1` marker are wired, and
exact Rustfmt plus Python compilation pass. A dense 4,096-track model changes
geometric growth `11→0`. Managed Cargo/Release, allocator, and animation
product p50/p95/p99 evidence remain pending; no coordinator status is polled.

### Runtime840 transient allocation output capacity (2026-09-19)

Runtime840 reserves the exact filtered lifetime count after transient
allocation admission and stable interval sorting, before emitting one compiled
allocation per retained lifetime. Slot reuse, allocation IDs, interval
validation, bucket identity, and empty behavior remain unchanged. The focused
source/model contract passes `2/2`; the lower source/order regression and
ignored `RUNTIME840_TRANSIENT_ALLOCATION_CAPACITY_BENCH_V1` marker are wired,
and the deterministic 4,096-lifetime model changes geometric growth `11→0`.
The shared focused batch passes `46/46`; the one-process broad non-tooling
loader passes `3907/3907` across `933` modules with zero failures, errors, load
errors, or skips. These are local source/model receipts only. Managed
Cargo/Release, allocator, and render-graph product p50/p95/p99 evidence remain
pending; no coordinator status is polled.

### Runtime841 runtime-tree pseudo-state collector capacity (2026-09-19)

Runtime841 extends the Runtime73 runtime-tree style path with a saturating
capacity bound before pseudo-state alias collection. Authored attribute count,
currently enabled component flags, node flags, and the two-slot resolved
painter allowance are combined without a second map scan. Retained-state
filtering, alias order, sorting/deduplication, painter-family resolution, and
clean-node behavior remain unchanged. The TDD source/model contract passes
`4/4`; Runtime810 plus Runtime841 pass `7/7`; the lower order/capacity
regression and ignored `RUNTIME841_RUNTIME_PSEUDO_STATE_CAPACITY_BENCH_V1`
marker are wired. The 4,096-entry deterministic model changes geometric
growth `12→0`. Managed Cargo/Release, allocator, and Runtime73 style product
p50/p95/p99 evidence remain pending; no coordinator status is polled.

The combined Runtime841/Runtime810/Runtime840/Editor840/Editor09/Runtime19
source batch passes `30/30` in `0.041s`. The current non-tooling
performance-contract loader passes `2340/2340` across `640` modules in
`7.026s`, with zero load errors, failures, errors, or skips. These are local
source/model receipts only; managed Cargo/Release and product percentile gates
remain pending.

### Runtime842 TreeView metadata collection capacity (2026-09-20)

Runtime842 adds direct-array reservations to the default-interaction TreeView
metadata collectors. Node IDs, borrowed/owned option IDs, and disabled IDs now
reserve the currently admitted TOML array bound before recursive traversal;
table alias precedence, first-seen ordering, duplicate suppression, nested
children, and the existing exact range-selection capacity remain unchanged.
The TDD source/model contract passes `4/4`; the lower nested-order/duplicate
regression and ignored `RUNTIME842_TREE_METADATA_COLLECTION_CAPACITY_BENCH_V1`
marker are wired. A deterministic 4,096-value model changes geometric growth
from `11` events to `0`. Managed Cargo/Release, allocator, and TreeView
product p50/p95/p99 evidence remain pending; no coordinator status is polled.

The post-Runtime842 focused Runtime/Editor source batch passes `40/40` tests
in `0.046s`, including the TreeView, UI asset-registration, owner-structure,
and adjacent Runtime/Editor capacity/interface slices. The latest batched
non-tooling performance/pressure loader covers `647` modules and passes
`2366/2366` tests in `5.524s`, with zero load errors, failures, errors, or
skips. These are local source/model receipts only; managed Cargo/Windows
Release, allocator, and product p50/p95/p99 gates remain pending.

### Runtime843 UI node resource registration output capacity (2026-09-20)

Runtime843 keeps the all-resource path allocation-free while lazily reserving
the resource-free node report on its first empty projection. Metadata-bearing
nodes seed their URI collector with the saturating sum of authored attributes,
slot attributes, and style override map lengths. URI scheme filtering, fallback
policy, first-seen order, node ownership, and stale-edge removal remain
unchanged. The focused source/model contract passes `4/4`; the lower collector
capacity regression and ignored
`RUNTIME843_NODE_RESOURCE_REGISTRATION_CAPACITY_BENCH_V1` marker are wired.
The dense 4,096-node model changes geometric growth `11→0`. Managed
Cargo/Release, allocator, and UI asset-registration product p50/p95/p99
evidence remain pending; no coordinator status is polled.

### Runtime845 style-plan rule capacity (2026-09-20)

Runtime845 counts the total authored rule bound across resolved stylesheets with
saturating addition, then reserves the `ParsedStyleRule` output before the
existing selector parse loop. Global rule order, selector error propagation,
per-sheet token-map sharing, declaration cloning, and empty-sheet zero capacity
remain unchanged. The TDD source/model contract passes `4/4`; the lower
bounded/empty regression and ignored
`RUNTIME845_STYLE_PLAN_RULE_CAPACITY_BENCH_V1` marker are wired. A dense 4,096
rule model changes geometric growth `11→0`. Managed Cargo/Release, allocator,
and style-plan product p50/p95/p99 evidence remain pending; no coordinator
status is polled.

The Runtime845-inclusive focused Runtime/Editor loader covers `22` modules and
passes `86/86` tests in `0.069s`; the broad non-tooling performance/pressure
loader covers `649` modules and passes `2374/2374` tests in `5.699s`, with zero
load errors, failures, errors, or skips. These are local source/model receipts
only; managed Cargo/Windows Release, allocator, and product p50/p95/p99 gates
remain pending.

### Runtime846 navigation world projection capacity (2026-09-20)

Runtime846 reserves the known dynamic-component row counts before projecting
navigation agents, agent positions, and obstacles. Invalid descriptors and
missing transforms are still filtered exactly as before, and the sorted row
order plus avoidance-index construction remain authoritative. The TDD
source/model contract passes `4/4`; the lower reservation/empty regression and
ignored `RUNTIME846_NAVIGATION_PROJECTION_CAPACITY_BENCH_V1` marker are wired.
A dense 4,096-row model changes each zero-capacity collector's geometric
growth from `11` events to `0`. Managed Cargo/Release, allocator, and
navigation product p50/p95/p99 evidence remain pending; no coordinator status
is polled.

The focused six-contract batch covering Runtime842/843/845/846/847 and Editor844
passes `24/24` tests in `0.013s`, with zero failures, errors, or skips. The
refreshed broad non-tooling loader covers `651` modules and passes
`2382/2382` tests in `5.334s`, with zero load errors, failures, errors, or
skips; these remain local source/model receipts only.

### Runtime847 particle extract output capacity (2026-09-20)

Runtime847 captures the sorted dynamic-component owner count before particle
extraction and uses it as the upper bound for `emitters` and `bounds`. Sprite
fanout remains lazy because authored particle/HUD arrays can exceed one entry
per owner. Existing filtering, stable global sprite sort, bounds calculation,
and GPU-frame aggregation remain unchanged. The TDD source/model contract is
`4/4`; the lower reservation/empty regression and ignored
`RUNTIME847_PARTICLE_EXTRACT_OUTPUT_CAPACITY_BENCH_V1` marker are wired. A
dense 4,096-owner model changes each bounded collector's growth `11→0`.
Managed Cargo/Release, allocator, and particle product p50/p95/p99 evidence
remain pending; no coordinator status is polled.

### Runtime851 style import resolution capacity (2026-09-20)

Runtime851 reserves the UI style resolver's borrowed imported-style vector from
`document.imports.styles.len()` and resolves each reference through the same
single map lookup and unknown-import error path. Widget styles, imported
stylesheet/token composition, local stylesheet order, and the existing exact
final `sheets` reservation remain unchanged. The TDD source/model contract
passes `4/4`; the isolated lower resolution/error-order regression and ignored
`RUNTIME851_STYLE_IMPORT_RESOLUTION_CAPACITY_BENCH_V1` marker are wired. A
dense 64-import model changes geometric growth `5->0`. Managed Cargo/Release,
allocator, and UI style-resolution product p50/p95/p99 evidence remain pending;
no coordinator status is polled.

### Runtime853 V2 style-rule filter retain (2026-09-20)

Runtime853 keeps the compiled `ResolvedRule` table and removes the opposite
selector class in place with `retain`, so static resolution and the runtime
pseudo-state index no longer allocate a second `filter(...).collect()` vector.
The helper compares the existing `uses_pseudo_state()` classification and
therefore preserves the source sort/order and selector semantics. The TDD
source/model contract passes `4/4`; the existing V2 style-capacity and
pseudo-state contracts join a focused `10/10` batch. The lower regression keeps
static and runtime order plus retained capacity, and the ignored
`RUNTIME853_V2_STYLE_RULE_FILTER_RETAIN_BENCH_V1` marker reports paired
legacy/optimized p50/p95/p99 samples. A deterministic 4,096-rule model changes
one filter buffer per build to zero. Managed Cargo/Release, allocator, and UI
style product p50/p95/p99 evidence remain pending; no coordinator status is
polled.

The refreshed eleven-contract Runtime/Editor loader passes `44/44` tests in
`0.047s`; the broad non-tooling performance/pressure loader covers `656`
modules and passes `2402/2402` tests in `33.492s`, with zero load errors,
failures, errors, or skips. These are local source/model receipts only; managed
Cargo/Windows Release, allocator, and product p50/p95/p99 gates remain pending.

### Runtime854 runtime selector path single-buffer (2026-09-20)

Runtime854 builds the retained-tree selector path directly into one
`Vec<SelectorPathNode>` while walking from the target to the root, then reverses
that buffer in place and restores the root-only `is_host` bit. The old
ancestor-ID vector and second node-map lookup loop are gone; component-state
collection, root-first order, selector matching, and tree ownership are
unchanged. The new TDD source/model contract passes `4/4` after a RED run with
three structural failures; the lower order/host regression and ignored
`RUNTIME854_RUNTIME_SELECTOR_PATH_SINGLE_BUFFER_BENCH_V1` marker are wired. A
depth-128 deterministic model changes the intermediate ID-buffer count from
`1->0` per path build. Managed Cargo/Release, allocator, and selector-style
product p50/p95/p99 evidence remain pending; no coordinator status is polled.
The focused V2 selector-path/filter-retain/style-capacity/pseudo-state batch
passes `14/14`; after Runtime855, the five-slice batch passes `18/18`. The
pre-Editor856 widened one-process loader passed `4042/4042` across `957`
contract files in `125.129s`, with zero load errors, failures, errors, or skips. The
current expanded source-contract loader (performance-or-contract filename
filter, tooling/export/coordinator excluded) passes `4072/4072` across `962`
files in `139.499s`, with the `961`/`4068` receipt retained as pre-Runtime856
historical context and the `957`/`4042` receipt retained as pre-Editor856
historical context.

### Runtime855 V2 file source capacity (2026-09-20)

Runtime855 seeds the V2 file-cache BFS queue and loaded-source output with the
known `paths.len()` root-input bound. Canonical-path de-duplication,
transitive import resolution, source discovery order, and empty-input behavior
remain unchanged. The TDD source/model contract passes `4/4` after a RED run
with two structural failures; the lower queue/source-order regression and
ignored `RUNTIME855_V2_FILE_SOURCE_CAPACITY_BENCH_V1` marker are wired. A
4,096-unique-root no-import model changes both bounded collector growth-event counts
from `11->0`. Managed Cargo/Release, allocator, and file-source product
p50/p95/p99 evidence remain pending; no coordinator status is polled.

### Runtime856 render-view camera filter retain (2026-09-20)

Runtime856 reuses the capacity-sized scene-camera descriptor vector in
`build_render_view_extract` instead of allocating a second zero-capacity
`filter(...).collect()` buffer. The selected camera remains included even when
inactive, active cameras retain the existing sorted order, and the no-camera
fallback is unchanged. The TDD source/model contract passes `4/4` after a RED
run with one missing lower owner; the lower order/empty regression and ignored
`RUNTIME856_RENDER_VIEW_CAMERA_FILTER_RETAIN_BENCH_V1` marker are wired. The
8,192-camera model changes modeled growth `13->0`; the focused V2 batch passes
`26/26`. Managed Cargo/Release, allocator, and render-view product p50/p95/p99
evidence remain pending; no coordinator status is polled.

### Runtime857 animation drain output capacity (2026-09-20)

Runtime857 reads the pending count before `drain_animation_clip_events` allocates:
no-pending drains keep zero capacity, while non-empty drains reserve the bounded
`max_events` output before sampling pending batches. Event and byte budgets,
cursor requeue, unavailable-asset handling, order, and empty-drain behavior are
unchanged. The TDD source/model contract passes `4/4`; the lower order/empty
regression and ignored
`RUNTIME857_ANIMATION_DRAIN_OUTPUT_CAPACITY_BENCH_V1` marker are wired. The
4,096-event model changes modeled growth `12->0`, and the current combined
focused batch passes `46/46`. Managed Cargo/Release, allocator, and
animation-event product p50/p95/p99 evidence remain pending; no coordinator
status is polled.

### Runtime858 render post-process output capacity (2026-09-20)

Runtime858 reads the registered post-process component count once and reserves
both extract and local-fog output vectors from that upper bound, using zero when
the component is absent. Layer filtering, shape handling, priority/entity sort,
and empty-world behavior remain unchanged. The TDD source/model contract passes
`4/4` after a RED run with one structural and one wiring failure; the lower
order/empty regression and ignored
`RUNTIME858_RENDER_POST_PROCESS_OUTPUT_CAPACITY_BENCH_V1` marker are wired. A
4,096-component model changes both collectors' modeled growth `24->0`, and the
combined focused batch passes `38/38`. Managed Cargo/Release, allocator, and
render post-process product p50/p95/p99 evidence remain pending; no coordinator
status is polled.

### Runtime200/205 capacity-eligibility follow-up (2026-09-18)

Runtime200 package declaration capacity and Runtime205 runtime-feature capacity now use the
same product-catalog eligibility predicate as merge. Carrier and test-fixture registrations
remain inspectable and are still skipped by merge, but no unreachable definition slots are
reserved. The focused Runtime200 source contract is `8/8`; lower package and runtime carrier
capacity regressions are wired. The deterministic mixed model changes `320` reservations
(`256` carrier plus `64` eligible) to `64`, removing 256 unreachable slots while preserving
merge semantics. Exact-file Rustfmt for the six touched Runtime200/205 files and scoped diff
checks pass. Managed Cargo/Release and product p50/p95/p99 gates remain pending.

### Current-worktree non-tooling revalidation after shared changes (2026-09-19)

The same single-process Runtime/Editor non-tooling contract loader was rerun
against the current shared checkout after Runtime808. It loaded `877` files and
passed `3693/3693` tests with zero failures, errors, or skips in `83.085s`. The
combined eight-slice focused batch passes `32/32`. This is refreshed local
source/model evidence for the Runtime rows above; managed Cargo/Release,
allocator, and product percentile gates remain pending.

After Runtime824 and Editor827, the current one-process non-tooling loader loads
`623` modules and passes `2224/2224` tests with zero failures, errors, or skips.
This supersedes older local receipts for the shared source tree; it remains
source/model evidence and does not replace managed Cargo/Release or product
percentile gates.

After Editor828, the current one-process non-tooling loader loads `624` modules
and passes `2227/2227` tests in `22.343s` with zero failures, errors, or skips.
The six-slice focused loader passes `21/21`. This is the latest shared
Runtime/Editor source-model receipt; managed Cargo/Release, allocator, and
product percentile gates remain pending.

After Editor829, the current one-process non-tooling loader loads `626`
modules and passes `2235/2235` tests in `7.762s` with zero failures, errors,
or skips. The eight-slice focused loader passes `29/29`. This supersedes the
preceding local source/model receipt; managed Cargo/Release, allocator, and
product percentile gates remain pending.

Editor830 is now included in the shared source tree. Its focused source/model
contract passes `4/4`; the refreshed single-process Runtime/Editor loader
passes `2239/2239` across `627` modules in `5.505s`, and the nine-slice focused
loader passes `33/33` in `0.017s`, with zero failures, errors, or skips. This is
one batched receipt rather than per-task validation. Managed Cargo/Release,
allocator, and layout-preset product percentile gates remain pending.

The full current non-tooling Runtime/Editor performance-contract loader loaded
`389` Runtime/Editor modules and passed `1407/1407` tests in one process,
including Runtime819/820 and the adjacent Editor slices. This broad receipt is
local source/model evidence only; managed Cargo/Release, allocator, and Runtime
product percentile gates remain pending.

### Exact optimize-record source coverage audit (2026-09-19)

The completion-list audit scanned the 155 Runtime optimize records dated
2026-09 with `implementation_status: implementation_complete` and checked each
record path/report ID against the Runtime/Editor Astra feature ledgers. Four
legacy Astra entries had only their parent review in `plan_sources`; the exact
Runtime200 active-pointer and Runtime77 dispatch-handler records were added to
their corresponding feature entries. The rerun reports `0` missing exact
Runtime/Editor Astra sources. The paired Runtime/Editor source-contract recheck
ran in one process (`4` files, `13/13` tests overall, `0` failures/errors/
skips). This remains documentation/source evidence only; managed Cargo/Release
and product p50/p95/p99 gates remain pending.

### Current-tree batched contract recheck (2026-09-19)

The current shared checkout was reloaded in one process across 835 non-tooling
Runtime/Editor contract files. All `3396/3396` tests passed in `52.871s`, with
zero load errors, failures, errors, or skips. This refreshes source/model
coverage only; it is not managed Cargo, Release, or product percentile proof.

The companion performance/pressure discovery (647 Runtime/Editor files) passed
`2391/2391` tests in `15.836s`, also with zero load errors, failures, errors,
or skips. It remains deterministic source/model evidence rather than a
Windows Release or product p50/p95/p99 receipt.

### Runtime838/839/Editor837 current batched source receipt (2026-09-19)

The focused twelve-contract Runtime/Editor batch covering Runtime838,
Editor837, Runtime833/835, Editor834/836, and the adjacent Runtime/Editor
capacity/projection contracts passes `47/47` tests in `0.030s`, with zero
failures, errors, or skips. The broader one-process non-tooling loader covers
`859` Runtime/Editor contract modules and passes `3485/3485` tests in
`34.136s`, also with zero failures, errors, load errors, or skips. These are
local source/model receipts only; managed Cargo/Windows Release, allocator,
and product p50/p95/p99 gates remain pending.

After Runtime839, the focused thirteen-contract Runtime/Editor batch passes
`49/49` tests in `0.043s`, with zero failures, errors, or skips. The refreshed
one-process non-tooling loader covers `861` Runtime/Editor contract modules and
passes `3489/3489` tests in `33.065s`, also with zero failures, errors, load
errors, or skips. This supersedes the preceding local source/model receipt;
managed Cargo/Windows Release, allocator, and product p50/p95/p99 gates remain
pending.

### Runtime contract-repair additions (2026-09-19)

The current shared batch also carries the Runtime Interface public-root
receipt repair and the Runtime random-checkpoint generation-error completion
record. Together with Runtime840 and the adjacent Editor projection slice,
the combined source-contract process passes `23/23` tests in `0.025s`; exact
export-source Rustfmt passes. These are local source/model receipts only.
Runtime Interface Cargo, Windows Release, allocator, and product p50/p95/p99
evidence remain pending under the external-worktree admission gate.

The detailed completion records are [Runtime Interface UI exports](19-ui-interface-exports.md),
[Runtime random-checkpoint generation error](08/2026-09-18-random-checkpoint-generation-error-contract.md),
and [Runtime asset-root validation](02-asset-root-validation.md).

### Runtime11/15 task lock-poison guard repair (2026-09-20)

The Runtime task-graph cutover left one Runtime15 structure guard asserting
removed `JobStateInner`/`lock_task` owners. The guard and JobSystem mirror now
follow the folder-backed `TaskNode` and `job_scheduler/pending.rs` owners,
including their actual poison-regression test files. Exact Rustfmt, source
anchor, and diff checks pass; the final batched Runtime audit passes `14/14`
tests in `18.218s` with zero failures, errors, or skips; no production scheduling
behavior changed. The optimization record is [Runtime11 task lock-poison guard
repair](../../../../plans/optimize/zircon_runtime/11/2026-09-20-task-lock-poison-guard-repair.md).
Managed Windows Cargo/Release and product gates remain pending.

### Recent-record Rustfmt convergence (2026-09-20)

The current recent-record Rustfmt batch covers `169` Rust files referenced by
the Runtime/Editor optimize set (`74` Runtime, `84` Editor, and `11` shared
plugin/interface owners) and passes with zero diffs. It required only mechanical formatting in the
Editor787 lower-test owner and two Runtime74 benchmark owners; no production
behavior changed. Managed Cargo/Release, allocator, and product percentile
gates remain pending.
The Astra implementation/test path audit also resolves all `1180/1180`
Runtime/Editor references with zero missing files.
The Runtime853 addendum independently resolves its two implementation owners
and one source-contract owner (`3/3`) with zero missing paths. The Runtime854
addendum likewise resolves its two implementation owners and one source-contract
owner (`3/3`) with zero missing paths. The Runtime855 addendum likewise resolves
its two implementation owners and one source-contract owner (`3/3`) with zero
missing paths.

### Runtime859 logical text batch capacity (2026-09-20)

Runtime859 reserves `layout.lines.len()` before logical artifact and visual
fallback batch emission. Line order, source ranges, glyph-artifact ownership,
stale/missing/incomplete rejection, and empty-layout behavior remain unchanged.
The TDD source/model contract passes `4/4`; the lower order/empty regression and
ignored `RUNTIME859_LOGICAL_TEXT_BATCH_CAPACITY_BENCH_V1` marker are wired. A
4,096-line model changes modeled growth `12->0`, and the focused five-slice
Runtime/Editor batch now passes `46/46`.

### Post-Editor857 lower-model refresh (2026-09-20)

After correcting the Editor857 lower model's expected plugin-node count, all
three isolated lower Rust owners compile and pass their non-ignored tests:
Runtime857 `2/2`, Runtime858 `2/2`, and Editor857 `2/2`; each retains one
ignored managed-Release benchmark marker. The refreshed one-process
performance-or-contract loader covers `967` files and passes `4092/4092` tests
in `375.582s`, with zero load errors, failures, errors, or skips. These are
local source/model receipts only; managed Cargo/Release, allocator, and product
percentile gates remain pending. Runtime859 and Editor858 are included in the
newest receipt below.

### Pre-Runtime860 expanded refresh after Runtime859/Editor858 (2026-09-20)

One local lower-owner batch compiles Runtime857, Runtime858, Editor857, Runtime859,
and Editor858; all five owners pass `2/2` non-ignored tests (`10/10` total), each
retaining one ignored managed-Release marker. The refreshed one-process
performance-or-contract loader covers `967` files and passes `4092/4092` tests
in `375.582s`, with zero load errors, failures, errors, or skips. Two existing
shader-prewarm fixture Cargo command lines were printed from inside tests; this
is local source/model evidence, not managed Windows Release/Cargo acceptance.
Managed Cargo/Release, allocator, and Runtime/Editor product percentile gates
remain pending.

### Runtime860 selector candidate scratch capacity (2026-09-20)

Runtime860 extends the existing Runtime73 terminal-selector index without
changing selector authority. `collect_candidate_indices` clears the reused
scratch, computes a saturating upper bound across the buckets that can match
the current node, and reserves only a capacity deficit before the existing
extensions and sort/deduplication. Empty nodes remain zero-capacity, while a
dense first lookup no longer grows the vector geometrically.

The intentional RED/GREEN source/model contract passes `4/4`; the lower owner
contains dense-order, empty-capacity, and ignored
`RUNTIME860_SELECTOR_CANDIDATE_CAPACITY_BENCH_V1` coverage. The deterministic
4,096-rule model changes `13` geometric growth events to `0`, and the combined
Runtime/Editor focused source-contract batch passes `61/61`. This is local
source/model evidence only. The post-Runtime860 expanded loader covers `968`
files and passes `4096/4096` tests in `142.495s`, with zero load errors,
failures, errors, or skips; two shader-prewarm Cargo command lines printed by
fixture tests are not managed Windows Release/Cargo acceptance. Managed
Windows Cargo/Release, allocator, and selector-style p50/p95/p99 gates remain
pending. The detailed record is
[`Runtime860 selector candidate capacity`](../../../../plans/optimize/zircon_runtime/73/2026-09-20-selector-candidate-capacity.md),
and the completion entry is
[`Runtime860 feature`](860-selector-candidate-capacity.md).

### Runtime861 navigation fallback path deduplication in place (2026-09-20)

Runtime861 keeps the baked-mesh fallback path's caller-owned point vector and
uses `Vec::dedup_by` for the existing adjacent XZ-distance predicate. The
first point, threshold (`0.05`), order, and `NavPathPoint` metadata semantics
remain unchanged; only the second zero-capacity output vector is removed.

The intentional RED/GREEN source/model contract passes `4/4`; lower coverage
checks `4,096` unique points retain input capacity, preserves duplicate
semantics, and wires the ignored
`RUNTIME861_NAVIGATION_PATH_DEDUP_IN_PLACE_BENCH_V1` marker. The deterministic
model changes modeled legacy output growth from `11` to `0`. Exact Rustfmt and
Python compilation pass. The refreshed broad explicit performance-or-contract
loader covers `969` non-tooling files and passes `4100/4100` tests in
`225.799s`, with zero load errors, failures, errors, or skips; two shader-
prewarm Cargo command lines printed by fixtures are not managed acceptance.
This remains local source/model evidence only; managed Windows Cargo/Release,
allocator, and navigation fallback p50/p95/p99 gates remain pending. The detailed record is
[`Runtime861 navigation path deduplication`](../../../../plans/optimize/zircon_runtime/169/2026-09-20-navigation-path-dedup-in-place.md),
and the completion entry is
[`Runtime861 feature`](861-navigation-path-dedup-in-place.md).

### Runtime862 navigation polygon vertex projection capacity (2026-09-20)

Runtime862 moves baked-polygon vertex projection into a small owner helper that
reserves the bounded index-slice length before filtering invalid indices. Valid
vertices remain in source order, invalid indices remain omitted, and the
`Vec3` conversion is unchanged; the optimization removes geometric growth for
dense polygons without changing the navigation asset contract.

The intentional RED/GREEN source/model contract passes `4/4`; lower coverage
checks the reserved index bound and invalid-index filtering and wires the
ignored `RUNTIME862_NAVIGATION_VERTEX_PROJECTION_CAPACITY_BENCH_V1` marker. The
deterministic `4,096`-index model changes modeled growth from `11` to `0`.
The combined Runtime861/862 navigation batch passes `35/35` in `0.062s`; after
the Runtime08d lower-contract compatibility repair, the expanded navigation
batch passes `38/38` in `0.139s`. Scoped Rustfmt/Python compilation pass. This
remains local source/model
evidence only; managed Windows Cargo/Release, allocator, and navigation
projection p50/p95/p99 gates remain pending. The detailed record is
[`Runtime862 navigation vertex projection capacity`](../../../../plans/optimize/zircon_runtime/169/2026-09-20-navigation-vertex-projection-capacity.md),
and the completion entry is
[`Runtime862 feature`](862-navigation-vertex-projection-capacity.md).

### Post-Runtime862 broad source-contract refresh (2026-09-20)

After the Runtime08d borrowed-index contract was adapted to the extracted
`polygon_vertices` owner, the one-process explicit performance-or-contract
loader covers `970` non-tooling files and passes `4104/4104` tests in
`228.769s`, with zero load errors, failures, errors, or skips. The two
shader-prewarm Cargo command lines printed by fixture tests are fixture output,
not managed Windows Release/Cargo acceptance. Managed Cargo/Release, allocator,
and Runtime/Editor product p50/p95/p99 gates remain pending.

### Current optimize-record hash audit (2026-09-21)

The current-source audit scanned all `25` dated `2026-09-20` Runtime/Editor
optimize records and recomputed `75` SHA-256 entries from their source tables.
All referenced files exist and all `75/75` hashes match; this is a record
integrity receipt only and does not advance managed compilation, allocator, or
product percentile acceptance.

The companion record-structure audit found all `25/25` current records carry
implementation, validation, performance, source-snapshot, and acceptance
sections; no required section is missing.

### Post-document-audit broad source-contract refresh (2026-09-21)

The same one-process explicit performance-or-contract loader was rerun after
the record-structure and hash audits. It again covers `970` non-tooling files
and passes `4104/4104` tests in `274.515s`, with zero load errors, failures,
errors, or skips. Fixture-emitted Cargo command lines are not managed Windows
Release/Cargo evidence; allocator and Runtime/Editor product p50/p95/p99 gates
remain pending.
The focused Runtime08d/861/862 navigation batch was also rerun in one process
and passes `38/38` in `0.021s`, with zero failures, errors, or skips.

## Managed gate

All rows remain `implemented_pending_validation`. The first managed submission
(`astra-runtime-text-batch-20260911-01`) was rejected with
`validation_copy_overlay_not_owned`; after scoped lease/attribution, the Cargo
retry (`astra-runtime-text-batch-20260911-02`) was rejected by the external
`E:\\Git\\zr_vm` dirty-worktree gate. A separate static text ticket was
accepted asynchronously, but it is not a Runtime UI compile or performance
ticket. No Cargo process was started and no coordinator status was polled.
Product performance acceptance still requires an owner-attributed multi-task
Windows Release ticket with allocation/time and p50/p95/p99 evidence.
