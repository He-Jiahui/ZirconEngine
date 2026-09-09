---
related_code:
  - zircon_runtime/src/script/mod.rs
  - zircon_runtime/src/animation/mod.rs
  - zircon_runtime/src/navigation/mod.rs
  - zircon_runtime/src/dynamic_api/mod.rs
  - zircon_runtime/reflection_macros/src/lib.rs
implementation_files:
  - zircon_runtime/src/script
  - zircon_runtime/src/animation
  - zircon_runtime/src/navigation
  - zircon_runtime/src/dynamic_api
  - zircon_runtime/reflection_macros/src
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md
tests:
  - zircon_runtime/src/script/vm/tests
  - zircon_runtime/src/animation/sequence/tests.rs
  - zircon_runtime/src/navigation/runtime/tests.rs
  - zircon_runtime/src/dynamic_api/tests
doc_type: category-index
---

# 脚本、反射与动画

本分区说明可选脚本 VM、统一反射、动态运行时 API，以及动画和导航运行时。脚本域受 Cargo feature `script` 门控，动态 API 还要求 `dynamic-api`；动画和导航既可作为 runtime 模块，也可通过插件提供更完整的 backend。

## 文档地图

- [VM、Host 与插件](vm-host-and-plugins.md)：backend registry、host module、能力集合、GC、脚本插件发现和热重载。
- [反射、脚本宏与调用点](reflection-and-macros.md)：`ZrReflect`、类型/字段 schema、读写、Host function/module 宏和文档生成。
- [动态 API 交界](dynamic-api-boundary.md)：Runtime API V8、session、脚本/插件如何通过 ABI 安全 DTO 交互。
- [动画与导航](animation-navigation.md)：clip/graph/state machine、sequence 编译、baked navmesh、agent tick 和 backend 限制。

## feature 组合

| feature | 提供能力 |
| --- | --- |
| `animation` | `AnimationModule`、`DefaultAnimationManager`、clip/graph/state machine/sequence |
| `navigation` | 内建 navigation module、baked mesh 查询和 agent tick |
| `script` | VM host、反射 world access、脚本场景系统和插件管理 |
| `dynamic-api` | `dynamic_api` session、ABI V8、shader prewarm 入口；会带入多个 runtime 域 |

Server profile 默认不启用脚本/图形/文本；可用性必须由 profile 和 capability 报告确认。
