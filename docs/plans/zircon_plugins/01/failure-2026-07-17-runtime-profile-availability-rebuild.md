---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: runtime-profile-availability-rebuild
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_plugins/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/runtime_profile/defaults.rs
  - zircon_runtime/src/plugin/runtime_profile/availability.rs
  - zircon_runtime/src/plugin/package_manifest/builtin_catalog.rs
  - zircon_runtime/src/builtin/runtime_modules/availability.rs
tests:
  - runtime profile availability build-count benchmark
  - 1/100/1000 provider selection scaling test
  - availability report byte-equivalence test
---

# Plugins01：runtime profile availability 重建与二次投影

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：MVP plugin package-manifest/runtime-profile 逐文件静态审查
- 修复责任计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 交接原因：profile、builtin catalog、provider projection 和 bootstrap assembly 必须共享一次冻结结果，不能在单个 helper 内留下第二套缓存 authority。

## 失败现象与复现证据

`runtime_profile_availability` 等 bootstrap assembly helper 每次先调用
`RuntimePluginDescriptor::builtin_catalog()` 重建完整 descriptor catalog。`RuntimeProfileDescriptor::for_id()`
也通过 `builtin_profiles()` 构造全部六个 profile，再只取一个。

registration-report 路径先把 linked/native provider id 分别放入 `Vec<String>`，用线性 `push_provider_id`
去重；随后 availability core 又把两个 Vec clone 成 `HashSet<String>`。project manifest selection 同样通过
`Vec::iter_mut().find` 去重。P 个 provider/selection 时，启动投影包含 O(P²) 查重与至少一轮可避免的 String
复制；该路径当前只在 bootstrap/export assembly 可达，未发现逐帧调用，因此列为启动规模问题而非 frame 热点。

## 最低共享层根因

启动 assembly 没有一次性的 `{profile, descriptor catalog, provider membership}` 冻结 projection。
各 helper 重新生成默认 profile/catalog，再在 Vec 与 HashSet 之间重复投影，导致 build count 和复杂度没有统一预算。

## 架构修复验收

- 单次 bootstrap/export generation 中，builtin profile 与 descriptor catalog 各构建一次并由 consumers 借用。
- linked/native provider membership 从 registration reports 直接建立最终集合，不经过线性去重 Vec 和第二次 String clone。
- manifest selection required 合并使用线性索引，同时保持首次出现顺序和 `required` OR 语义。
- 1/100/1000 provider/selection benchmark 证明 build count 恒定、查重线性；报告分类、顺序、reason 文本逐项等价。
- 产品启动 trace 证明该 projection 不进入 frame/tick 路径；若 editor 需要刷新，按 plugin generation 失效而不是无界全局缓存。

## 禁止临时方案

- 不得分别缓存 `builtin_profiles()` 和每个 availability helper 的结果，形成多个失效源。
- 不得为了 HashSet 查重改变报告首次出现顺序或 required 合并规则。
- 不得把 startup-only 静态形态描述成稳定帧瓶颈；优先级必须由规模 benchmark 与产品 trace决定。

## 修复结果与回传

Open state: `待 Plugins01 建立 bootstrap availability generation projection 与规模化 build-count 回归`。

### 2026-07-18 实现进度（focused GREEN，broad 待队列）

- `RuntimeProfileDescriptor::for_id` 已改为按 id 直接构造单个 profile，不再先构造六项 builtin profile catalog。
- 新增 generation-local availability projection；descriptor、manifest selection 与 provider membership 只投影一次。registration reports 直接借用 package id，linked bootstrap 直接持有最终 `HashSet<String>`，export 复用同一 `RuntimePluginCatalog` 并借用最终 membership。
- runtime/profile/feature assembly 复用预计算报告；显式 project manifest 的报告字节语义保持一致，core module 失败报告仍携带 availability。
- 已新增 1/100/1000 registration/linked membership 与 selection 线性计数、单次 runtime/export catalog build、registration filter/dedup、descriptor/catalog byte parity 和 feature/non-feature manifest byte parity 回归。
- scoped rustfmt、`git diff --check` 与源码守卫已通过；第四轮独立静态复审为 Critical 0 / Important 0。
- canonical Rust 1.94.1 focused Cargo job `1a1c62ba1cb84df9be0ca8a80d2d3967`
  （run `3e9788ee378744c3b2ddf62a5bf0933e`）已 GREEN：12 passed / 0 failed / 8478 filtered。
  manifest selection 的 1/100/1000 步数为 1/100/1000，availability projection 为 2/200/2000；
  同一 gate 同时覆盖报告字节等价、首次顺序与 required OR、registration filter/dedup、runtime/export
  单次 catalog build 以及 feature manifest 字节保持。
- canonical broad `plugin_extensions` reservation `9d7890f3d0af458488425c798f6948a6`（source-manifest
  fingerprint `9e73dc1ec0ed653b126d387434cf93ca1dc68d3d728f6575b1799bd99f6e35ec`）未绑定
  job 即已 released，因此不计 broad 通过；在新 broad GREEN 与 failure return 完成前，
  本记录仍不声明 failure fixed。
- managed job `93f88e221e244b93b176afa90a07cdff` 保留的 current-source test binary（SHA-256
  `0EAD8F289E845A8730E84EAEB51D7A97C545C306421BF2D623EAC0BCFB12B5A7`）追加执行
  `profile_availability_projection` 整组为 `8 passed / 0 failed / 4302 filtered`。实测 selection
  `1/100/1000` 行分别 `1/100/1000` steps，availability 分别 `2/200/2000` steps，并通过
  catalog/manifest byte parity、required OR 与 registration filter/dedup。broad gate 与 fixed return 仍保持待完成。
- 2026-07-22 逐文件性能复审补充：availability report按indexed runtime id命中后不再clone descriptor projection row；源码守卫/rustfmt/diff通过。线性projection实现保持不变，但PERF-MVP-534仍要求Editor轮询使用generation-owned compact category/index/summary，避免每次重建reason String，并让required failure的category与`missing_required`共享唯一row owner。broad gate/fixed return状态不变。

### 2026-07-30 实现状态（受管验证待队列）

- `RuntimePluginAvailabilityGeneration` 现在在 runtime plugin 公共边界提供不可变 category index、runtime-id index 与 compact summary；其 row 借用 package id 并以枚举保存原因，完整 reason/id 条目仅由 `RuntimePluginAvailabilityRow::detail` 或 `materialize_report` 在导出/诊断边界显式生成。
- required failure 只保存一个 entry row，primary category 和 `missing_required` 通过索引共享该 row；新增 `availability_generation_shares_required_rows_and_materializes_report_bytes` 覆盖指针同一性、summary、公开重导出和旧报告字节等价。
- 受限源范围 `rustfmt --config skip_children=true --check`、`git diff --check` 及公开边界/单行所有权源码守卫已通过。当前全局受管 Cargo 队列由其他 Session 占用，尚未获得本切片 current-source focused/broad 结果；本 handoff 保持 `open`，不得回传 fixed。
