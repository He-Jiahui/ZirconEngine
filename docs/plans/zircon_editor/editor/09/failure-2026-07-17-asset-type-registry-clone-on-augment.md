---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: asset-type-registry-clone-on-augment
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/09
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/asset/type_registry/registry.rs
  - zircon_editor/src/core/editor_plugin.rs
tests:
  - 1/100/1000 contribution clone/allocation scaling benchmark
  - failed contribution atomicity and diagnostic-order parity
  - plugin catalog asset projection generation test
---

# Editor09：asset type registry 增量贡献 clone-on-augment

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_editor/src/core/asset` 15/15 Rust 文件逐文件审查
- 修复责任计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 交接原因：贡献原子性、有序 materialization 与 catalog generation 都属于 Editor09 asset registry authority。

## 失败现象与复现证据

`AssetTypeRegistry::apply_contribution` 为保持失败原子性，在每次 augment 前深 clone 完整 `MaterializedEntry`，然后合并、重复排序 templates/commands，再替换原 entry。随插件贡献数量和 descriptor payload 增长，同一 asset type 的累计复制接近二次放大；`EditorPluginCatalog::editor_extensions` 每次重建整个 registry 时会重放全部贡献。

`EditorHostEventController::asset_type_definition`、asset open、creation/context lookup 与每次 full reflection 的 browser/activity projection 还会重新 `materialize_enabled_asset_types`：从 builtins 开始 clone/replay 所有 enabled registrations。一个查询或一次 UI event 因此可能先重建完整 registry，再只读取一个 definition；full snapshot 还把同一 registry 投影到两个 asset workspace。

## 最低共享层根因

registry 没有 validate-then-commit delta 或 generation finalize 阶段，只能靠 clone 全 entry 获得事务性，并在每次增量后重新维持排序。

## 架构修复验收

- 先在 borrowed existing entry 上完成只读冲突/字段验证，再对成功 delta 就地提交一次；失败路径不 clone/不改变 registry。
- templates/commands 用有序唯一索引或批量 generation finalize，避免每个贡献后全量排序。
- registry 与 editor plugin catalog generation 绑定，未变 generation 不重放 builtin/plugin contributions。
- 单 definition/creation/context/open lookup 和 workbench snapshot 复用同一 enabled-capability generation registry，不得 per query materialize。
- 1/100/1000 contribution benchmark 总复制字节与查找近线性；失败原子性、owner diagnostics 文本/顺序与稳定 materialization 顺序完全等价。

## 禁止临时方案

- 不得牺牲失败原子性或让半个 contribution 可见。
- 不得给每个 asset type 建互不失效的临时缓存。
- 不得用无序迭代改变插件贡献或诊断顺序。

## 修复结果与回传

Open state: `待 Editor09 实现 validate-then-commit delta 与 generation finalize，并回传规模/原子性证据`。
