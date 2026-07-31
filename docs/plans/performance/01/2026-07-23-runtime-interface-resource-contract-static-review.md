---
related_code:
  - zircon_runtime_interface/src/resource
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
reference_sources:
  - dev/bevy/crates/bevy_asset/src/event.rs
  - dev/bevy/crates/bevy_asset/src/id.rs
  - dev/bevy/crates/bevy_asset/src/assets.rs
  - dev/godot/core/io/resource.cpp
tests:
  - zircon_runtime_interface/src/tests/resource_contracts.rs
  - current-source Windows zircon_runtime_interface resource tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime interface resource 合同性能静态审查（2026-07-23）

## 范围与覆盖

`zircon_runtime_interface/src/resource/**`当前源 **14/14** 个 Rust 文件、**770** 行已逐文件阅读，并完整阅读 `resource_contracts.rs`。生产引用核对确认这些类型不是孤立 DTO：`zircon_runtime::asset`直接把 `AssetReference`、`ResourceLocator` 与 `AssetUuid`公开为 asset facade 类型，runtime registry、importer、editor retained host和多个插件均在使用。

## 性能结论

- `ResourceManager::list_resources() -> Vec<ResourceRecord>`强制调用方取得整份 owned 宽记录。当前实现每次克隆 registry 全量记录，再以 `primary_locator.to_string()`排序；retained editor在任意 resource event batch 后从 UI/host 路径调用它。事件 drain 已有 256 条/stream与 600 us预算，但持续导入/热重载会跨 tick 重复触发主线程 O(N log N) 排序、locator String和依赖/诊断/哈希字段深克隆。该接口层证据并入既有 **PERF-MVP-500**，不建立第二套优化真相。
- `resource_status(&str) -> Option<ResourceRecord>`使已经持有 typed `ResourceLocator`的 editor caller先 `to_string()`，runtime再 parse/normalize，并克隆完整 record；`resolve_ready_handle`最终只需要 id、kind、state和失败诊断。PERF-MVP-500应提供 typed identity/status row与 selected detail分层，稳定查询不得走 String round-trip或宽 record clone。
- `ResourceLocator::parse/new`即使没有反斜杠也先 `replace`分配，随后为每个 path component分配 `String`、收集 `Vec<String>`并 `join`；package id合法性还临时 `collect::<Vec<_>>()`。`AssetReference::from_locator`和 `ResourceId::from_locator`又格式化完整 locator String；`stable_uuid_from_components`构造第二份 joined String并对它执行两次 hash。project scan和glTF/model subasset生成会按资源放大这些分配，新增 **PERF-MVP-564**。
- `ResourceEvent`拥有 current/previous locator，ResourceManager fanout会按 subscriber克隆事件。typed receiver专用过滤线程已由 PERF-MVP-492删除；底层共享有界 generation event log/cursor、identity-only hot event和按需 detail继续归同一项，不在 interface 层建立第二个 queue。
- `ResourceHandle<T>`、`UntypedResourceHandle`与 `ResourceId`本体均为小型 Copy identity，typed/untyped转换无分配；marker/kind/state/event-kind枚举没有当前热路径问题。`ResourceRecord::failure_reason`为线性诊断查找，但仅 error detail路径，保留。

## 优化设计

1. Runtime04随 resource generation发布唯一 immutable ordered compact rows（例如 `Arc<[ResourceSummaryRow]>`）和 typed lookup；事件只携 identity/kind/revision。Editor持有 generation并只为 visible page/selected detail请求宽记录。旧 `list_resources`在迁移期只能委托同一 generation，不得重新扫描、排序或深克隆 registry。
2. 为 canonical locator/稳定 ID提供单遍 writer/hash sink：直接按 `scheme://path#label`字节流喂给两个 hasher，保留现有 UUID bit-for-bit；locator normalize只分配最终 canonical path，package id用单 component检查，删除 replace/component-String-Vec/join中间层。
3. `resource_status`增加 typed locator或 ID入口，并拆分 compact status与 diagnostic detail；已有 typed caller禁止先格式化再解析。

第 2 项是内部、可测试的低风险切片，但涉及持久稳定 ID语义；按 brainstorming 规则，先提供 golden corpus与设计批准，再改代码。

## 参考引擎对照

Bevy 的常规 `AssetEvent<A>`只携带 Copy `AssetId<A>`，路径和 load error走专用失败事件；默认 runtime identity还可使用紧凑 index。Godot `ResourceCache::get_ref`与 `get_cached_resources`发布引用，而不是深复制完整资源对象。Zircon应保留跨进程稳定 UUID和显式 locator语义，但热事件与稳定枚举同样应分离 identity、共享 generation和按需 detail。

## 动态验收

1. current-source `zircon_runtime_interface` resource contract tests；补固定 locator/label/package/AssetUuid/ResourceId golden corpus，改造前后 UUID、serde和错误优先级 bit-for-bit不变。
2. assets 1/1k/100k，stable 60 Hz、1% changed、event burst 1/256/10k：记录 registry scans、wide-record/locator/hash String clone bytes、sort comparisons、generation builds与 UI p95；stable build/sort/deep clone=0，changed近 delta，visible/selected按需。
3. locator segments 1/16/256、path bytes 16/1KiB/64KiB：记录 replace/component Vec/String/join和 stable-id joined buffer分配；目标每次 normalize只保留最终 canonical allocation，stable hash中间 String=0。
4. subscribers 1/8/64与 rename/reload burst：复用 PERF-MVP-492的 bounded ring/cursor门禁，hot event locator clone=0，overflow/coalesce/age语义确定。

动态门禁、generation API和产品 F4 资产浏览 trace未完成，因此该目录继续保留在 `pending.md`，不进入 `review.md`。
