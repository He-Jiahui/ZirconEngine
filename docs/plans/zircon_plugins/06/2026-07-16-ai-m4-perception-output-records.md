---
related_code:
  - zircon_plugins/ai/plugin.toml
  - zircon_plugins/ai/runtime/src/perception.rs
  - zircon_plugins/ai/runtime/src/perception/adapter.rs
  - zircon_plugins/ai/runtime/src/perception/components.rs
  - zircon_plugins/ai/runtime/src/perception/scan.rs
  - zircon_plugins/ai/runtime/src/perception/stimuli.rs
  - zircon_plugins/ai/runtime/src/plugin/registration.rs
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_plugins/sound/runtime/src/engine/state/storage.rs
  - zircon_plugins/sound/runtime/src/service_types/sources.rs
  - zircon_runtime/src/core/framework/ai/perception.rs
  - zircon_runtime/src/core/framework/sound/emission.rs
  - zircon_runtime/src/plugin/extension_registry/register/metadata.rs
implementation_files:
  - zircon_plugins/ai/plugin.toml
  - zircon_plugins/ai/runtime/Cargo.toml
  - zircon_plugins/ai/runtime/src/lib.rs
  - zircon_plugins/ai/runtime/src/manager.rs
  - zircon_plugins/ai/runtime/src/manager/perception.rs
  - zircon_plugins/ai/runtime/src/perception.rs
  - zircon_plugins/ai/runtime/src/perception/adapter.rs
  - zircon_plugins/ai/runtime/src/perception/components.rs
  - zircon_plugins/ai/runtime/src/perception/scan.rs
  - zircon_plugins/ai/runtime/src/perception/stimuli.rs
  - zircon_plugins/ai/runtime/src/plugin.rs
  - zircon_plugins/ai/runtime/src/plugin/registration.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/events.rs
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_plugins/sound/runtime/src/engine/state/storage.rs
  - zircon_plugins/sound/runtime/src/service_types/manager_state.rs
  - zircon_plugins/sound/runtime/src/service_types/manager_trait/source.rs
  - zircon_plugins/sound/runtime/src/service_types/sources.rs
  - zircon_runtime/src/core/framework/ai/mod.rs
  - zircon_runtime/src/core/framework/ai/perception.rs
  - zircon_runtime/src/core/framework/sound/components.rs
  - zircon_runtime/src/core/framework/sound/emission.rs
  - zircon_runtime/src/core/framework/sound/manager/source.rs
  - zircon_runtime/src/core/framework/sound/mod.rs
  - zircon_runtime/src/plugin/extension_registry/register/metadata.rs
tests:
  - zircon_plugins/ai/runtime/src/tests/perception_runtime.rs
  - zircon_plugins/ai/runtime/src/tests/registration.rs
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_plugins/sound/runtime/src/tests/playback/gameplay_emission.rs
plan_sources:
  - docs/plans/zircon_plugins/06-ai.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - user: 2026-07-16 严格按 zircon_plugins 架构计划完成插件功能
doc_type: milestone-detail
---

# 2026-07-16 AI M4 Perception 产出记录

Plan: docs/plans/zircon_plugins/06-ai.md
Milestone: M4
Status: completed
Files: ["docs/plans/zircon_plugins/06/2026-07-16-ai-m4-perception-output-records.md", "docs/zircon_plugins/ai/runtime.md", "docs/zircon_plugins/plugin-sdk.md", "docs/zircon_plugins/plugin_sdk/registration.md", "docs/zircon_plugins/sound/runtime.md", "docs/zircon_runtime/core/framework/ai.md", "docs/zircon_runtime/core/framework/sound.md", "zircon_plugins/ai/plugin.toml", "zircon_plugins/ai/runtime/Cargo.toml", "zircon_plugins/ai/runtime/src/lib.rs", "zircon_plugins/ai/runtime/src/manager.rs", "zircon_plugins/ai/runtime/src/manager/perception.rs", "zircon_plugins/ai/runtime/src/perception.rs", "zircon_plugins/ai/runtime/src/perception/adapter.rs", "zircon_plugins/ai/runtime/src/perception/components.rs", "zircon_plugins/ai/runtime/src/perception/scan.rs", "zircon_plugins/ai/runtime/src/perception/stimuli.rs", "zircon_plugins/ai/runtime/src/plugin.rs", "zircon_plugins/ai/runtime/src/plugin/registration.rs", "zircon_plugins/ai/runtime/src/tests/mod.rs", "zircon_plugins/ai/runtime/src/tests/perception_runtime.rs", "zircon_plugins/ai/runtime/src/tests/registration.rs", "zircon_plugins/animation/runtime/src/evaluation/pipeline/events.rs", "zircon_plugins/plugin_sdk/src/registration.rs", "zircon_plugins/sound/runtime/src/engine/state/storage.rs", "zircon_plugins/sound/runtime/src/service_types/manager_state.rs", "zircon_plugins/sound/runtime/src/service_types/manager_trait/source.rs", "zircon_plugins/sound/runtime/src/service_types/sources.rs", "zircon_plugins/sound/runtime/src/tests/playback.rs", "zircon_plugins/sound/runtime/src/tests/playback/gameplay_emission.rs", "zircon_runtime/src/core/framework/ai/mod.rs", "zircon_runtime/src/core/framework/ai/perception.rs", "zircon_runtime/src/core/framework/sound/components.rs", "zircon_runtime/src/core/framework/sound/emission.rs", "zircon_runtime/src/core/framework/sound/manager/source.rs", "zircon_runtime/src/core/framework/sound/mod.rs", "zircon_runtime/src/plugin/extension_registry/register/metadata.rs", "zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_classification/runtime/systems.rs"]

## Scope delivered

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M4 | T1 组件、预算扫描与遗忘 | `完成` | 2026-07-16 | typed/dynamic source 与 receiver 共用稳定排序；flattened pair cursor 公平推进，预算限制 pair 数，刺激独立 aging/forgetting。 |
| M4 | T2 可选 physics sight 遮挡 | `完成` | 2026-07-16 | `physics.query.v1` 通过 weak `BridgeImport` 解析；缺失/禁用/reload/revoke 明确退化为 range+cone，不持有 concrete manager。 |
| M4 | T3 hearing 事件适配 | `完成` | 2026-07-16 | hearing/animation 共享有界 ingest；World-owned backlog 跨 event-store rotation 重试；Sound per-World journal 非破坏读取并报告覆盖缺口。 |
| M4 | SDK 与结构 | `完成` | 2026-07-16 | component/resource/event/system 全部走 owner-aware SDK builder；owner mismatch 返回 typed error；生产 owner 均低于 800 行。 |
| M4 | Testing | `完成` | 2026-07-16 | AI、Sound、plugin SDK 全包和 Runtime upward build 均受管 exit 0；最终独立 review Critical 0 / Important 0。 |

## 架构边界

- 中立 `AiHearingStimulusEvent`、Perception DTO、Sound emission/journal DTO 与 manager trait 留在 `zircon_runtime::core::framework`；扫描、缓存、事件适配和 Sound journal 存储留在各自可选 runtime 插件。
- Perception 系统在 `Update` 先于 behavior tick，World resource 持有 ECS cursor、Sound sequence 和 durable backlog。错误路径在返回前恢复 resource，不用全局 closure map，也不创建隐藏成功旁路。
- Sound journal 按 `WorldHandle` 分区，每区容量 1024、独立 sequence、非破坏读取。跨 World 洪泛不能驱逐或误报另一 World；共享状态锁按审查发现 F2 使用 poison recovery。
- component metadata 注册由 SDK module owner 传递；descriptor 的 `plugin_id` 必须匹配 `<plugin>.runtime` owner。重复 slot 与 owner 不匹配均返回 typed registry error，不依赖 `expect` 不变量。

## Fresh testing evidence

- AI runtime：managed job `14793e415ec1442c8de52545b1d59eed`，Cargo build/test/doctest 成功，exit 0。
- Sound runtime：managed job `0d9f89ecd3b04a95acb77adfe2ec87cd`，含独立 reader、单/跨 World overflow、muted filter 和 poisoned-state recovery，exit 0。
- plugin SDK：managed job `839e57989b4842ccab52fb635cc71548`，owner-bound component builder 与 forged foreign owner 拒绝覆盖，exit 0。
- Runtime upward：managed job `62e2f5f56994496e984a1f3eb4142d1b`，`zircon_runtime -SkipTest` build 成功，exit 0。
- 静态：30 个本次 Rust 文件 `rustfmt --check` 通过；精确 `git diff --check` 通过；plugin structure audit 的 manifest、free-function registration、compatibility shim、SDK mirror 与 distribution boundary 均为 0 违规。

## Review

- Review：最终独立只读复审 Critical 0 / Important 0；未运行第二套 Cargo，未修改 review 范围。

## 未完成边界

- M3-T4 `patrol_detect_chase_scenario` 已解除 M4 依赖，但仍是下一独立切片，未在本里程碑伪标完成。
- M5 行为树图编辑器、运行时节点高亮、Blackboard 面板与 perception overlay 仍未完成。
- 整体 AI package 因上述两项继续保持 `Experimental / Partial`；只有 `runtime.feature.ai.perception` 在 M4 runtime 边界为 `Complete`。
