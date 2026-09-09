---
related_code:
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/lifecycle.rs
  - zircon_runtime/src/core/runtime/frame_clock.rs
  - zircon_app/src/entry/product_composition
  - zircon_app/src/entry/export_bootstrap.rs
  - zircon_editor/src/core/project
  - zircon_editor/src/core/document
  - zircon_runtime_interface/src/runtime_api
implementation_files:
  - zircon_runtime/src/core/runtime
  - zircon_app/src/entry
  - zircon_editor/src/core/project
  - zircon_editor/src/core/document
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/mvp/index.md
  - docs/engine-architecture/runtime-interface-convergence.md
tests:
  - zircon_runtime/src/core/runtime/tests
  - zircon_app/src/tests
  - zircon_editor/src/core/project/tests
  - zircon_runtime_interface/src/tests
doc_type: workflow-detail
---

# 产品生命周期

本页描述从进程启动到关停的共同语义。不同宿主（client、editor-host、server、serialized runtime）可以替换外层实现，但不能跳过模块、session、句柄和错误边界。

## 1. 配置与 profile 解析

入口先读取 `EntryConfig`/`ProductCompositionRequest`，解析 `EntryProfile`、`RuntimeProfileId`、`RuntimeTargetMode`、平台能力和插件要求。解析结果会保留 provenance，便于诊断“来自默认值、项目 manifest 还是命令行”。

```rust
use zircon_app::entry::{EntryConfig, EntryProfile, ProductCompositionRequest};

let config = EntryConfig::new(EntryProfile::Editor);
let request = ProductCompositionRequest::new(config);
```

具体构造器会随 profile 类型变化；调用者应使用 crate root re-export，而不是复制 feature 选择表。若必需插件或宿主能力缺失，解析阶段返回 `ProductHostConfigError`，不能等到窗口创建后才静默降级。

## 2. 描述符收集与模块图冻结

App 收集 builtin module、linked first-party catalog 和 export root 下的 native plugin descriptor，合并成 `ModuleDescriptor` 图。`CoreRuntime::register_module` 负责注册，`sort_module_activation_order` 负责检测重复名、缺依赖和环。激活前图会冻结；运行中不应直接插入未排序的模块。

## 3. 四阶段模块生命周期

每个模块按以下顺序推进：

```text
Registered -> Initializing(build) -> Ready(ready) -> Active(finish)
                                      |                  |
                               timeout/error       cleanup -> Stopping -> Unloaded
```

- `build`：注册服务、系统、事件和扩展槽；不得假定依赖模块已经 finish。
- `ready`：轮询异步依赖（GPU surface、资源、插件能力），受 ready timeout 约束。
- `finish`：所有依赖 ready 后进行最终接线，进入可运行态。
- `cleanup`：按反向依赖顺序释放系统、manager、driver 和 plugin；带 drain timeout。

## 4. Session 与 World

Runtime session、editor project session、viewport 和 watch 都有独立身份。Editor 打开项目时先做 probe/preflight，再激活 `ProjectSessionId`，然后打开 Scene 文档。Serialized gateway 的 session identity 与 DLL API 版本必须一致；旧 endpoint 替换后，旧 handle 只能返回 stale/invalid 错误。

## 5. 帧循环

```text
host event pump
  -> input normalization
  -> runtime tick / time policy
  -> PreUpdate / Update / LateUpdate / FixedUpdate
  -> render extract snapshot
  -> render graph prepare/queue/submit
  -> present / host output
  -> diagnostics and wake decision
```

`FrameClock` 的 `tick_time`/`advance_time_by` 产生 `FrameTimeSnapshot`；固定步上限防止失速后无限追赶。发生系统睡眠、窗口恢复或外部时钟跳变时，通过 `submit_clock_discontinuity` 生成 rebase receipt，而不是手动修改时间字段。

## 6. 操作与异步结果

World query、viewport pick、asset import、save、plugin event 和 editor command 都可能返回 operation/watch/ticket。每个异步完成路径必须检查：session identity、generation、取消状态、请求限制和当前 lifecycle。宿主应先 drain/harvest 结果，再释放 allocation；超出 byte limit 的载荷返回结构化错误。

## 7. 持久化与重开

编辑器保存流程是：捕获 history save token -> 序列化 authoring 文档 -> 原子写入 -> `mark_saved_if_unchanged`。保存期间若有新事务，不应错误清除 dirty。重开时重新 probe/迁移/激活，不复用旧 World 引用；资产引用、实体 UUID 和 transform 从持久数据恢复。

## 8. 关停

关停顺序应由产品 shutdown policy 统一驱动：停止新帧/新操作 -> 终止 Play -> drain scoped tasks -> 反向停模块 -> 释放 gateway/session -> flush diagnostics/log -> 退出。`CoreRuntime::shutdown_registered_modules_with_drain_timeout` 为模块反向清理提供集中入口；任何超时必须保留未完成条目和 owner 信息。

## 生命周期错误排查

1. 看 `ProductExitClass`/`CoreError`/`GatewayError` 的阶段字段，不先猜 UI 问题。
2. 检查 profile 的 feature 和 manifest capability。
3. 检查 session/generation 是否在异步回调前发生变化。
4. 检查最低共享支撑层（resolver、resource readiness、serialization）而非在 App 层加旁路。
5. 用[测试与平台](testing-platform/index.md)中的受管验证命令复现。
