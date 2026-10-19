---
status: in_progress
---

# Runtime Asset Validation Worklist

## Asset gates remain open

Asset queries remain manager/facade/service owned; do not introduce an asset server alias. Handle state, failed-resource diagnostics, reload transitions, watcher failures/debounce, bounded worker queues and diagnostics retain their original behavior regressions.

The resource transaction owner is crates/zr_resource/src/manager/commit.rs; it validates reload transition publication. Artifact scene roundtrips and watcher-focused results do not substitute for broader asset:: / worker_pool validation. Material/shader cache schemas and family/record-set aggregation keep independent owners, with one immutable asset management generation.

### Independent mirrors and pending work

The Runtime04 worklist, runtime index, asset facade, worker pool, watcher, artifact, core resource, M0 review and runtime-interface convergence remain nine independent inputs. The Cargo gate case below parses status: in_progress, the six pending rows and exact five command identities. They are retained gate identities; future actual commands require scoped feature/target/profile declarations.

The recorded 2026-07-10 source count25 and later 2026-08-30 source count26 are different dated inventories. Preserve both numeric obligations without calling either a fresh measurement. All broader gates remain pending.

This worklist retains its declared status independently of module policy, the runtime index and archive evidence. No new test/product acceptance is recorded.

### Retained source status identities

| Contract | Responsibility / evidence boundary |
|---|---|
| `runtime_04_15_asset_artifact_material_shader_owner_split_static_passed_cargo_deferred` | Material/shader owners preserve schema/variant order and conversion behavior. |
| `runtime_04_15_asset_management_generation_static_implemented_cargo_deferred` | Family classification and record sets share one immutable asset-management generation. |

### Pending asset work

| Obligation | State |
|---|---|
| 1.1 句柄语义裁决 | Cargo 待验证 |
| 1.2 转移表测试 | Cargo 待验证 |
| 2.1 背压策略 | Cargo 待验证 |
| 2.2 请求去重 | Cargo 待验证 |
| 2.3 诊断计数 | Cargo 待验证 |
| worker pool 当前状态守卫 | Cargo 待验证 |

Original gate command identities (not undeclared current execution recipes):

- `cargo test -p zircon_runtime --lib load_state --locked -- --nocapture`
- `cargo test -p zircon_runtime --lib resource --locked`
- `cargo test -p zircon_runtime --lib asset:: --locked`
- `cargo test -p zircon_runtime --lib worker_pool --locked -- --nocapture`
- `cargo test -p zircon_runtime --lib watch --locked -- --nocapture`

### Source and guard ownership

The finite source, guard and provenance inventory for these responsibilities is:

- `zircon_runtime/src/asset/management.rs`, `src/asset/pipeline/worker_pool.rs`, `src/asset/facade/load_state.rs`.
- `src/asset/watch/watch_loop.rs`, `src/asset/facade/event.rs`, `record_sets.rs`.

### Validation interfaces

Keep these existing validation and domain interfaces in the gate:

- `runtime_04_asset_pipeline_cargo_gate_stays_visible_until_asset_validation`, `runtime_04_asset_facade_query_surface_stays_manager_owned_and_server_free`, `hot_reload_transitions_through_reloading_state_and_emits_modified_event`, `runtime_04_asset_pipeline_mirror_docs_match_structure_audit_counts`.
- `reload_failure_emits_reload_failed_event_and_lands_failed_state`, `worker_pool_bounded_queue_rejects_overflow_with_explicit_error`, `dangling_handle_queries_report_not_loaded_instead_of_panicking`, `worker_pool_diagnostics_track_in_flight_and_failure_counts`.
- `asset_pipeline_boundary`.

### Recorded targets

These retained targets belong to the original source snapshots, not a new measurement. The source dates/identities below keep distinct inventories separate.

- `retired_worker_request_sender_references = []`; source 882273fe611f:L233.
- `expected_artifact_store_roundtrip_count = 4`; source 882273fe611f:L233.
- `artifact_acceptance_reference_count = 3`; source 882273fe611f:L233.
- `watcher_acceptance_reference_count = 1`; source 882273fe611f:L233.
- `expected_watcher_acceptance_count = 7`; source 882273fe611f:L233.
- `expected_worker_diagnostic_count = 7`; source 882273fe611f:L233.
- `retired_worker_new_references = []`; source 882273fe611f:L233.
- `old_watch_debounce_references = []`; source 882273fe611f:L233.
- `missing_behavior_test_anchors = []`; source 2026-07-10:L57.
- `missing_cargo_gate_anchors = []`; source 882273fe611f:L233.
- `expected_source_file_count = 26`; source 2026-08-30:L278.
- `expected_source_file_count = 25`; source 2026-07-10:L57.
- `behavior_test_anchor_count = 24`; source 2026-07-10:L57.
- `expected_guard_file_count = 22`; source 2026-07-10:L57.
- `missing_doc_anchors = []`; source 882273fe611f:L233.
- `test_anchor_count = 28`; source 2026-07-10:L57.

### Contract details

The following original contract labels remain in scope; they do not establish new validation:

- `mirror_docs_guard_present = true`.

The retained asset-pipeline audit snapshot records `risks = []`. This is its dated diagnostic inventory, not a fresh risk assessment, Cargo result or product acceptance.

### Clause provenance

These content identities preserve the source of this transfer. They are not fresh measurements, active-file dependencies or acceptance receipts.

- `04-asset-pipeline-alignment.md`, SHA-256 `882273fe611f289e5194b6fda9a3b9e5c7dca427f86bae227ff3eafef8f30fc0`, selected source lines 51-295.
