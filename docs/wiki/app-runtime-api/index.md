---
related_code:
  - zircon_app/src/lib.rs
  - zircon_app/src/entry/mod.rs
  - zircon_app/src/entry/entry_profile.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_runtime/src/dynamic_api/mod.rs
  - zircon_runtime_interface/src/runtime_api/mod.rs
  - zircon_runtime_host/src/lib.rs
implementation_files:
  - zircon_app/src/lib.rs
  - zircon_app/src/entry/mod.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_runtime/src/dynamic_api/mod.rs
  - zircon_runtime_interface/src/runtime_api/mod.rs
  - zircon_runtime_host/src/lib.rs
plan_sources:
  - user: 2026-09-09 为 ZirconEngine 构建引擎说明书级 Wiki
tests:
  - zircon_app/src/entry/tests
  - zircon_runtime/src/dynamic_api/tests
  - zircon_runtime_interface/src/tests/abi_safety_contracts.rs
  - zircon_runtime_host/src/foreign_output/tests.rs
doc_type: category-index
---

# 应用入口与运行时 API

本分区说明 ZirconEngine 从产品进程启动，到动态加载运行时、创建会话、驱动帧循环，再到跨 ABI 交换数据的完整路径。它对应虚幻文档中的“Programming with C++ / Engine Architecture / Runtime API”交叉区域，但使用 Zircon 当前 Rust crate 和 C ABI 事实作为唯一依据。

## 阅读路线

1. [应用入口与导出启动](bootstrap-and-entry.md)：`zircon_app`、`EntryProfile`、`EntryRunner`、`ProductComposition` 和导出产品启动。
2. [动态运行时与 ABI V8](dynamic-runtime-abi.md)：动态库发现、BuildSet 预检、V8 函数表、必需与可选槽位。
3. [运行时会话与宿主输出](runtime-session-and-host-output.md)：`ZrRuntimeSessionConfigV3`、事件/帧循环、宿主请求、所有权与熔断。
4. [世界同步 API](world-sync.md)：查询、观察、失效通知以及编辑器投影的 Rust 调用方式。
5. [序列化与版本化契约](serialization-contracts.md)：持久化 envelope、迁移链、规范文本和二进制格式。
6. [兼容性与排错](compatibility-and-errors.md)：错误码、诊断生命周期、版本升级规则和常见故障定位。

## 分层职责

| 层 | crate | 主要责任 | 不负责 |
| --- | --- | --- | --- |
| 产品入口 | `zircon_app` | CLI、产品角色、模块/插件组合、动态库持有、原生窗口宿主 | ECS、渲染与场景业务实现 |
| ABI 数据契约 | `zircon_runtime_interface` | `#[repr(C)]` DTO、句柄、状态码、函数指针、JSON DTO、预算常量 | 会话状态与 OS/GPU 对象 |
| ABI 宿主适配 | `zircon_runtime_host` | 校验运行时输出、限额 JSON 解码、释放所有权、协议熔断与指标 | 动态运行时业务行为 |
| 动态实现 | `zircon_runtime::dynamic_api` | V8 入口导出、会话表、帧、事件、世界同步和运行时分配登记 | 产品 CLI 和编辑器 UI 生命周期 |

典型控制流如下：

```text
zircon_runtime executable / zircon_editor / exported product
    -> zircon_app::EntryRunner or export bootstrap
    -> resolve product profile and project root
    -> preflight runtime artifact + BuildSet
    -> load library and resolve zircon_runtime_get_api_v8
    -> validate/copy ZrRuntimeApiV8
    -> create ZrRuntimeSessionHandle from ZrRuntimeSessionConfigV3
    -> submit events / tick / present / query
    -> validate and release every runtime-owned output
    -> destroy session, unload library, destroy product composition
```

## 功能状态

| 能力 | 状态 | 当前边界 |
| --- | --- | --- |
| Editor、Runtime、Headless 产品入口 | 已实现 | `EntryProfile` 和产品角色解析均由 `zircon_app` 持有 |
| 链接式与原生插件导出启动 | 已实现 | 返回完整 `ProductComposition`，调用者必须保留到产品退出 |
| 动态运行时函数表 | 已实现 | 仅接受冻结的 `ZrRuntimeApiV8`；无旧表回退 |
| 会话启动配置 | 已实现 | 当前为 `ZrRuntimeSessionConfigV3`，包含 profile、项目根、Play 场景、报告出口与 wake sink |
| 原生 surface present | 部分实现 | V8 中为可选槽位；缺失时宿主可采用 CPU frame capture/presenter 路径 |
| accessibility、profiling、host request | 部分实现 | ABI 槽位存在但属于可选能力，宿主必须先判空 |
| 世界查询/观察/失效通知 | 已实现 | JSON 传输、运行时签发 `WatchToken`、输出按 allocation id 释放 |
| 版本化序列化和迁移 | 已实现 | schema id + version envelope，拒绝未知未来版本 |
| 面向不可信原生库的进程隔离 | 规划 | 当前指针安全仍依赖同进程提供者满足 ABI 前置条件 |
| 跨任意发行版的稳定插件 SDK | 未承诺 | 当前 ABI 是仓库内部锁步契约，兼容性由 BuildSet 与精确表形状保证 |

## 名称辨析

当前函数表名是 `ZrRuntimeApiV8`，入口符号是 `zircon_runtime_get_api_v8`。文档中出现的 V1、V2、V3 通常是单个 DTO 或函数签名族的版本，例如 `ZrHostApiV1`、`ZrOwnedResultV2`、`ZrRuntimeSessionConfigV3`。这些版本号不能互相替代，也不能据此推断存在“ABI V3 函数表”。
