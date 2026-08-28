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
  - zircon_editor/src/core/plugin/mod.rs
tests:
  - 1/100/10k/100k contribution post-extend collection length/sort/generation scaling benchmark
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
- 1/100/10k/100k contribution benchmark 记录真实 post-extend collection length、sort count 与 generation publish；失败原子性、owner diagnostics 文本/顺序与稳定 materialization 顺序完全等价。

## 禁止临时方案

- 不得牺牲失败原子性或让半个 contribution 可见。
- 不得给每个 asset type 建互不失效的临时缓存。
- 不得用无序迭代改变插件贡献或诊断顺序。

## 修复结果与回传

Open state: `2026-07-18 registry validate-then-commit delta、binary ordered insertion、registry generation、EditorPluginCatalog same-generation Arc cache 与 host extension/capability generation cache 均已落地；definition/open/create/context/workbench projection 已统一复用。静态门通过，仍待 source-bound 1/100/10k/100k Cargo 证据、独立 review、failure return 与 managed commit，因此本记录保持 open。`

当前切片与阻塞证据：[`2026-07-18-asset-type-registry-delta-generation.md`](2026-07-18-asset-type-registry-delta-generation.md)。

2026-07-22 external tests增量：单contribution内multi-entry批量排序与single-entry binary insert已成立；但`thousand_incremental_entries...`仍反向逐条commit并要求generation+1000，导致Vec搬移累计O(N²)并让consumer cache失效千次。PERF-MVP-562新增catalog-generation transactional batch：按asset type聚合全部plugin deltas、一次验证/merge/sort、一次原子publish；本failure在1/100/10k/100k entries moved-bytes/generation counter与Cargo/F0/F4 reload完成前继续open。

2026-07-23 方案 A 已获批准并完成 source implementation：`AssetTypeRegistry::apply_contributions` 以 contribution 为失败隔离单元，对 materialized base + pending claims 做 validate/stage；`registry/batch.rs` 按 asset type 单次 finalize collection，并在至少一个有效 contribution 时仅推进一次 generation。`apply_contribution` 已硬切复用单元素 batch；`EditorPluginCatalog` 收集整个 catalog generation 后单次调用，并用 traversal sequence 将延迟 asset errors 合回 view/drawer/tool-mode 等原诊断位置。新增 valid-invalid-valid、all-invalid、new-type recovery、cross-type、template/command finalize、1/100/10k/100k counter 与 catalog 顺序测试；静态 batch contract 3/3 GREEN。当前仍缺 source-bound focused/broad Cargo、F0/F4 reload、独立 review、failure return 与 managed commit，因此本 failure 保持 open。

2026-07-23 受管验证失败已按所有权留在 Coordinator01 后续处理：exact10 materialized copy `4b29be7dcd6748f9a25073b4ee04e8e3` 的 run `0bb42035dbd24951ab41f053ac850cc8` 在 rustc 前 exit 101，stderr 明确为 validation source 根缺失 `Cargo.toml`。copy 终态 `removed`，未产生 Editor09 编译/测试结果；不得将此 failure 转 fixed，也不得让 Editor09 吸收 root manifest 或外部 sibling dependency。

2026-07-23 独立 review 首轮 `0/1/2` 的唯一 Important（existing collection 实际 sort 全量但 counter 仅记 delta）已修复为 post-extend full length，并补 5 + 10 = 15 双 collection 回归；两个文档 Minor 同步关闭。增量复审 `0/0/0`。当前关闭门仍只剩 Coordinator01 closure 后的 source-bound Cargo、F0/F4、failure return 与 managed commit，本 failure 继续 open。

## 2026-08-28 static guard owner repair

The batch contract guard still read the retired
`zircon_editor/src/core/editor_plugin.rs` catalog owner, which is absent from
the committed tree. Catalog contribution materialization now belongs to
`zircon_editor/src/core/plugin/extension_materialization.rs`; that owner keeps
the single `apply_contributions` call and restores diagnostic traversal order
with the sequence-key sort. The guard now reads that committed owner while
retaining the same batch, ordering, and 1/100/10k/100k assertions.

All required assertions were checked directly against the four `HEAD` blobs
before the guard edit, so this repair does not depend on concurrent Editor
worktree overlays. This is static guard maintenance only. The source-bound
managed Cargo and performance evidence, independent review, failure return,
and terminal integration remain required; this handoff stays `open`.
