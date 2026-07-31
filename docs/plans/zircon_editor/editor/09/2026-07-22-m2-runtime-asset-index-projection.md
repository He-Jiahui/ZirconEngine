---
owner_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
milestone: M2
slice: runtime-asset-index-projection
status: source_complete_review_clean_external_text04_blocked
related_code:
  - zircon_editor/src/core/asset/mod.rs
  - zircon_editor/src/core/asset/index.rs
tests:
  - zircon_editor/src/core/asset/index/tests.rs
  - zircon_editor/tests/editor_asset_index_projection.rs
  - tools/tests/test_editor09_runtime_asset_index_projection_contract.py
---

# Editor09 M2.1 Runtime Asset Index Projection

Plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
Milestone: M2
Status: source_complete_review_clean_external_text04_blocked
Files: ["docs/zircon_editor/core/asset_index.md", "tools/tests/test_editor09_runtime_asset_index_projection_contract.py", "zircon_editor/src/core/asset/mod.rs", "zircon_editor/src/core/asset/index.rs", "zircon_editor/src/core/asset/index/tests.rs", "zircon_editor/tests/editor_asset_index_projection.rs"]

## 范围

本切片执行 `09-editor-asset-management.md` 的 M2.1。编辑器只持有 runtime
`AssetRegistryIndex` 与 `.zmeta` v7 `AssetMetaDocument` 的内存快照，浏览器行查询借用
runtime entry 的 UUID、path、type、tags、dependency 与 source digest。编辑器只拥有
watch dirty、importing 等瞬态展示状态，不新增平行 registry、sidecar、digest 或文件扫描。

## 实施阶段

- [x] 注册并领取 7 文件精确 Session scope。
- [x] 以静态合同和 Rust 单元/集成测试锁定 RED 行为。
- [x] 实现 `EditorAssetIndex`、借用行投影、`.zmeta` v7 ingest 与 watch reconcile。
- [ ] 运行受管 Windows Cargo 门；首轮精确门被 Text04 栅格接口漂移阻断，精确静态门与格式门已通过。
- [x] 完成独立 review 并关闭全部 finding。
- [ ] 受管 Cargo GREEN 后更新本记录并创建受管 milestone commit。

## 测试阶段

- 静态门：`python -m unittest tools.tests.test_editor09_runtime_asset_index_projection_contract -v`。
- 聚焦门：`cargo test -p zircon_editor --test editor_asset_index_projection --locked --jobs 1 -- --test-threads=1`。
- 单元门：随 `zircon_editor` lib gate 覆盖 `core::asset::index::tests`。
- 验收必须使用 Coordinator01 固化 source manifest；共享源码上的偶然 GREEN 不计入产出。

## 产出记录与时间

- 2026-07-22：状态 `source_complete_static_green_validation_pending`。已确认 runtime registry
  与 `.zmeta` v7 为唯一权威，M2.1 精确 7 文件 Session 已激活并取得租约；
  `EditorAssetIndex`、借用行投影、元数据原子 ingest、watch dirty reconcile、单元测试和
  公开集成测试均已落地。独立初审 `0/4/1` 暴露 preview/import 语义混用、pending path
  tombstone、同 document 子项残留、隐式 import completion 与测试缺口；修复后复审又以
  `0/1/0` 暴露逐 document 全表 retain，现已改为 `document_uuid -> projected UUID set`
  反向成员表。最终复审 `0/0/0`，静态合同 9/9 GREEN、精确 `rustfmt` 与
  `git diff --check` 通过。受管 Windows Cargo 与 milestone commit 尚待完成。父 M2 保持 `pending`。
- 2026-07-22：状态更新为 `source_complete_review_clean_external_text04_blocked`。精确
  source-manifest 门禁 job `ceb4440353f8477d8c9c4433cf8c562e` / run
  `b94d23c4b1694098a2edec57c5c977d3` 自然结束，exit code 101、测试 0；唯一阻断为
  Text04 的 `TextRasterWorkTarget`、`TextRasterCompletionDrain` 字段与 target drain API 漂移，
  已写入 `docs/plans/zircon_runtime/text/04/failure-2026-07-22-raster-target-completion-api-drift.md`。
  Editor09 精确 7 文件未出现编译诊断，最终 review 仍为 `0/0/0`；待 Text04 受管 SHA 后必须
  重新物化冻结副本并重跑，旧 job 不计 GREEN，父 M2 与 milestone commit 继续保持 `pending`。
- 2026-07-22 性能复核：`replace_runtime_registry`命中pending path的路径已从clone path+二次
  HashSet remove改为`retain`中直接转移UUID，源码守卫、rustfmt/diff通过。`rows()`仍触发runtime
  registry collect+path sort，registry replacement仍全量校验metadata/document/import/dirty集合；按
  PERF-MVP-556并入Editor09唯一asset generation与visible-page投影，动态未验收，当前状态不变。
