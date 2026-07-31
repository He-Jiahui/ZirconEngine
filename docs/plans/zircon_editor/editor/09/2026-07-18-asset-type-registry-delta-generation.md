# Editor09 asset type registry delta generation

## 目标与边界

- 删除 `AssetTypeRegistry::apply_contribution` 的 `MaterializedEntry` 深 clone 回滚路径。
- 保留失败原子性、旧字段/集合诊断文本与顺序、稳定 materialization 顺序。
- 同一 `EditorPluginCatalog` generation 只 materialize 一次 extension/asset registry。
- 将同一 catalog generation 的 asset type contributions 聚合为一个事务批次：失败 contribution 隔离跳过，有效 contribution 一次 finalize/publish，registry generation 最多推进一次。
- 本切片不吸收已被 Editor10 修改的 `docs/zircon_editor/core/asset.md`，也不触碰 watcher、catalog manager、preview worker 或 retained paint owner。

## 已批准设计（方案 A）

- 批次按 plugin registration 与 contribution 的原始遍历顺序消费；每个 contribution 是最小失败隔离单元。
- contribution 先对当前 registry 与本批次已接受的 pending claims 做完整 borrowed validation；失败时不留下 scalar、template 或 command 的部分占用，后续 contribution 继续处理。
- 所有有效 contribution 按 asset type 聚合到 pending delta；每个触及的 collection 在 finalize 时执行一次 extend/sort，新 definition 也只在完整 materialize 后发布。
- 批次有至少一个成功 contribution 时 registry generation 精确推进一次；全失败或空批次时 definition 与 generation 均不变。
- `apply_contribution` 直接复用单元素批次核心，不保留旧的逐项 commit 兼容实现。
- catalog 构建用单调 diagnostic sequence 合并普通 extension 错误与延迟返回的 asset type 错误，最终诊断顺序与现有遍历顺序严格一致。
- batch staging/finalize 下沉到 `type_registry/registry/batch.rs`；`registry.rs` 保留 registry façade 与单项入口，`mod.rs` 只保留模块声明/导出。

## 前置实现（已完成，待本里程碑收敛）

- [x] existing contribution 先 borrowed preflight，再执行 infallible in-place commit；失败 definition/generation 均不变。
- [x] existing 与同一 delta 内的重复 template/command id 均在 commit 前拒绝，并保留 owner 诊断。
- [x] template/command 用 binary insertion 维持稳定有序，不再每次贡献后 full sort。
- [x] 新 definition 在发布前验证 delta 内唯一性，初始 batch 只排序一次。
- [x] registry 单 contribution 成功推进 generation、失败不推进；本里程碑将其硬切为 catalog 批次单次推进。
- [x] `EditorPluginCatalog` 以 generation + `OnceLock<Arc<EditorExtensionCatalogReport>>` 缓存 materialization；register 原子失效缓存。
- [x] `editor_extensions()` 硬切为共享 Arc report，调用方不保留 owned clone 兼容路径。
- [x] Workbench shell 以 extension-registration generation + 排序 capability snapshot 缓存 `Arc<AssetTypeRegistry>`；注册失效、能力值变化 miss。
- [x] definition/open/create/context 与 Activity/Browser workbench projection 全部走 `enabled_asset_types_for_shell`；raw materializer 仅作 cache-miss builder。
- [x] 新增单 contribution 失败原子性、旧 1,000 次逆序增量/有序性、clone/sort 源码守卫与 same-generation Arc identity 测试；旧 `generation + 1000` 断言属于本里程碑必须删除的反向契约。

## M3 catalog-generation transactional batch

### M3.1 契约测试与静态 RED

- [x] 新增 real-code 测试：`valid -> invalid -> valid` 仅拒绝中间项，两个有效 delta 同时发布，错误保持输入位置，generation 仅 `+1`。
- [x] 新增 all-invalid、空批次、新 definition 先失败后成功、跨 asset type pending claims 与无部分占用测试。
- [x] 将 1,000 次逐项 commit 反向测试替换为 1/100/10k/100k 批次规模；断言单次 generation publish、collection 单次 finalize/sort 与无逐项 binary insertion。
- [x] 新增 catalog 全局诊断顺序测试，覆盖 asset type 错误前后均存在 view/drawer/tool-mode 错误的场景。
- [x] 新增 `tools/tests/test_editor09_asset_type_registry_batch_contract.py`；已观测缺少 batch module/API、旧逐项 loop 的预期 RED，生产实现后 3/3 GREEN。

### M3.2 Registry batch staging/finalize

- [x] 在 `type_registry/registry/batch.rs` 实现 generation-scoped pending delta、按 contribution 原子 validate/stage、按 asset type finalize。
- [x] existing definition 不做 full-entry clone；pending owner claims 与 entry owners 共同参与冲突判断。
- [x] 每个触及 collection 在 finalize 时一次 extend/sort；失败 contribution 的 scalar/collection claims 全部回滚到 contribution 起点。
- [x] crate-internal `AssetTypeRegistry::apply_contributions` 返回带原始 input index 的有序错误与真实 finalize counters；`apply_contribution` 复用同一批次核心。

### M3.3 Catalog 单批次物化与顺序诊断

- [x] `EditorPluginCatalog::build_editor_extensions` 收集整个 catalog generation 的 asset contributions 后只调用一次 registry batch API。
- [x] 普通 extension 与 asset type 错误统一带 traversal sequence，最终按 sequence 稳定合并，不改变历史诊断顺序。
- [x] same-generation `Arc<EditorExtensionCatalogReport>` identity 与 host extension/capability generation cache 保持不变。

### M3.T 测试阶段与关闭门槛

- [x] 静态契约 3/3、exact-scope rustfmt、`git diff --check` 与结构约束通过。
- [ ] 通过 Coordinator01 validation-copy 运行 source-bound focused gates：`editor_asset_type_registry`、`editor_plugin_sdk`，再运行 `zircon_editor --lib` broad gate。
- [ ] 记录 1/100/10k/100k 实际计数证据；不得用源码文本猜测或 shared-tree/source-raced Cargo 结果替代。
- [ ] 独立 review `Critical/Important/Minor = 0/0/0`，完成 failure -> fixed return、closeout checker 与 exact-manifest managed commit。

## 剩余验收与后续

- [x] `materialize_enabled_asset_types` 绑定 host extension/capability generation，单 definition/open/create/context/workbench snapshot 不再按查询重放 contributions。
- [ ] source-bound `cargo test -p zircon_editor --lib editor_asset_type_registry --locked` 与 `editor_plugin_sdk` focused gate。
- [ ] 1/100/10k/100k 受管规模数据、独立 review 0/0/0、failure -> fixed return 与 exact-manifest managed commit。

Coordinator01 的 `validation-copy-external-sibling-path-dependency` 仍为 Cargo 前置，且当前共享 Cargo lane 被外部受管任务占用；不得用共享树盲跑或 exit 0 的 source-raced 结果替代。本切片只记录真实完成的 registry/catalog source，不把整个 performance failure 标为 fixed。

2026-07-23 测试阶段实证：validation-copy plan `bb5ab7278e394f53a279708163f8866d` 与 materialized copy `4b29be7dcd6748f9a25073b4ee04e8e3` 均接受 exact10 manifest；受管 run `0bb42035dbd24951ab41f053ac850cc8` 执行 `cargo check -p zircon_editor --locked --color never`，在进入编译前以 exit 101 终止，原始 stderr 为 `could not find Cargo.toml in ...\source or any parent directory`。copy 已由协调器自动记录为 `removed`。该结果只证明 Coordinator01 尚未把 repo-local Cargo closure 装入 stable copy，不是 Editor09 编译判定；focused/broad/F0/F4 继续待办。

2026-07-23 独立 review 首轮 `0/1/2`：Important 指出 existing collection 的 finalize counter 只统计新增项但实际 sort 完整 vector；两个 Minor 为模块文档未列 `registry/batch.rs` 与 failure 残留 1/100/1000 文案。修复后 counter 在 extend 后读取完整 collection 长度，并新增 existing 5 + batch 10 = report 15 的 template/command 回归；文档与 1/100/10k/100k 契约同步完成。增量复审 `Critical/Important/Minor = 0/0/0`，但 reviewer 明确未把缺失的 Cargo 编译视为通过。

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据/剩余项 |
| --- | --- | --- | --- |
| 2026-07-18 11:46-12:08 +08:00 | `source_complete_static_green_managed_validation_blocked` | borrowed validate/in-place commit、delta 内唯一性、binary insertion、registry generation、plugin catalog Arc cache、host extension/capability generation cache、全 consumer 复用、1k/原子性/identity/cache 测试与独立模块文档 | RED 已观测；Editor09 registry static suite、rustfmt、`git diff --check` 通过。13 路 exact scope 由 `editor09-asset-type-delta-generation-r2-20260718` 持有；待 Coordinator01 fixed return 后 managed Cargo、1/100/1000 数据、review、return/commit。 |
