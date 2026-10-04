---
handoff_kind: failure
status: open
created_at: 2026-08-18
summary_slug: missing-subasset-parent-fallback
origin_plan: docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/optimize/zircon_runtime/04
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/reference_resolver.rs
  - zircon_runtime/src/asset/reference_resolution_error.rs
  - zircon_runtime/src/asset/migration/resolver.rs
  - zircon_runtime/src/asset/importer/ingest/import_model.rs
tests:
  - cargo test -p zircon_runtime --lib asset::reference_resolver::tests::resolution_reports_guid_path_repair_dangling_and_conflict_states --locked --jobs 1 -- --exact --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib asset::importer::ingest::import_model::tests::importer_outcome_exposes_complete_guid_repair --locked --jobs 1 -- --exact --nocapture --test-threads=1
---

# Runtime 04：缺失 subasset label 静默回退父资产

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md`
- 来源执行切片：P1-8 missing subasset semantic identity
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：最低共享原因位于 Runtime04 project reference resolver；调用方不能可靠判定被回退的父资产是否仍是原语义目标。

## 失败现象与复现证据

`entry_by_hint` 对缺失 labeled locator 执行 `.or(base_entry)`；GUID 快速路径也会在 persisted sub 与 registry label 不一致时直接返回。现有测试把 `#MissingMesh` 修复为无 label 父资产，导致重新保存后永久改变引用目标。

## 最低共享层根因

resolver 把 GUID/path 身份修复与 subasset 语义修复合并处理，没有要求 resolved registry entry 保持 persisted label，也没有为缺失 label 返回 typed dangling diagnostic 和同源候选。

## 架构修复验收

- stale GUID 仅可修到仍存在的 exact labeled entry，并保留相同 subasset label。
- 缺失 label 在 GUID 与 path-hint 两条路径均返回 typed dangling error，候选稳定列出同源 labeled entries，绝不退回父资产。
- migration 将该错误归类为 dangling reference；原 importer repair reproduction 与 Runtime/Editor/App 批量上行门通过。

## 禁止临时方案

- 禁止 importer/call-site 捕获错误后删除 label、重试父 locator 或伪造 repair。
- 禁止 registry alias、隐式 label rename、测试专用分支或削弱 subasset identity 断言。

## 修复结果与回传

Open state: `validation_pending`; no pass is claimed.

## 2026-09-09 current identity-contract continuation

The existing Runtime04 fixing Session `failure-roll-01a07160-runtime04`
reconciled the archived source ownership through transfer fingerprint
`3aee4b2393137af81af4a51f2379649b021e8e65f2d8cfe043efb2a65854ff65` and froze
the unchanged resolver pair in snapshot 3300. Current hashes match their
attributions and ObjectStore bytes: `reference_resolver.rs`
`c215371aa3bc51188ab22ebb9b0b4446b1eacffd7681de9b15e59309f61058c7` and
`migration/resolver.rs` `2feabd38460db9214ede6d05c29f2e6e0a4ddc48bceaefed3f0734c549d3e6d8`.

The original managed migration batch `2b2e7158e28c4cbdaa15fff4c6517426`
executed four tests: 2 passed and 2 failed. Both failures were the stale
expectations that a parent GUID should be replaced by a labeled mesh GUID;
the actual resolver returned the required `RegistryConflict`. The missing
labeled-subasset negative test passed. No production source was changed by
that batch.

The current Runtime87 contract and asset migration/importer documentation both
state that GUID is authoritative, a GUID/subasset mismatch is `Conflict`, and
missing labels cannot fall back to the parent. Snapshot 3304 updates only the
resolver and migration tests: the shared resolver now checks missing labels for
both an existing parent GUID and an absent GUID; migration tests reject both
parent-GUID conflict forms and verify a moved subasset hint preserves the mesh
GUID. Rustfmt and whitespace checks pass. Final focused managed validation is
pending; the first migration retry was rejected before a job was accepted due
to Cargo pool owner `cf40bbda43f5443f8e980a9396a2aa90`, with empty jobs/tests.
No accepted request was replayed.

## 2026-09-09 managed identity-contract closure

The corrected source was sealed in
`E:/cargo-targets/zircon-engine/cache/build-benchmarks/runtime04-subasset-final-3304-r2-20260909`,
with 10,968 files and manifest
`ba5f340ed904ecf6f1905e1a53fdebee4c959fbcfa5d8a821b19e63db2dcc918`.
The migration resolver batch ran the four exact `retired_migration_` tests:
job `fd6afaf6e96f4598830052d49c2ea5a5`, 4 passed, 0 failed, 0 ignored.
The reference resolver batch ran its three tests in job
`214ae209281d4e6986a4200b5214cbb7`, 3 passed, 0 failed, 0 ignored. Both
receipts bind the immutable `ba5f340e...` source manifest.

The same input initially exposed a separate lower fixture omission: after the
`mip_bias = 0.5` correction, `max_anisotropy = 8` was also absent while the
test asserted it. The first descriptor rerun therefore recorded job
`11f08f2ba0b34ffb90f5544136bb1009`, 26 passed, 1 failed, 0 ignored; no
production default was changed. Snapshot `3305` records the test-only fixture
addition with source hash
`dfa7019f6c3be58c11e6508d019ab1bdd534652fdbd34118da54dfd032ef86f7`.

The final descriptor input
`E:/cargo-targets/zircon-engine/cache/build-benchmarks/runtime04-subasset-final-3305-20260909`
contains 10,968 files and manifest
`3bfe12829402a3c0e72beaea5fdfff8d2d7c33125d97c8a8dc24f5b69899e15c`.
Job `2c03b7c385ba4b5eb4e31dbec084ad19` ran all 27 exact descriptor tests:
27 passed, 0 failed, 0 ignored, including
`import_settings_parse_texture_metadata_tokens`. The fixture handoff is
linked in `failure-2026-09-09-texture-metadata-token-fixture-missing-mip-bias.md`.

This lower Runtime04 chain now has dynamic evidence for the resolver,
migration, and descriptor fixture. The importer upward batch, independent
zero-finding review, canonical `failure return`, and coordinator closeout
remain pending; no fixed return or commit is claimed.

The originally listed importer filter
`importer_outcome_exposes_complete_guid_repair` is no longer present in the
current `import_model` test module. A one-time run of that stale filter returned
0 tests and no job acceptance. The current test name is
`importer_rejects_path_candidate_when_guid_is_unregistered`; its follow-up
submission was rejected before admission because the compatible Cargo pool was
owned by job `3c868f3a9f53493b8d4e0812e6845660` (`cargo_reuse_pool_busy`). No
accepted request was replayed; the importer upward gate remains pending.
