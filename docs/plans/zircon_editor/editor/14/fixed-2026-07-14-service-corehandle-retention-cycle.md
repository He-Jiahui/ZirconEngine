---
handoff_kind: fixed
status: fixed
created_at: 2026-07-13
summary_slug: service-corehandle-retention-cycle
origin_plan: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
fixing_plan: docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
origin_child_dir: docs/plans/zircon_editor/editor/14
fixing_child_dir: docs/plans/zircon_runtime/runtime/02
related_code:
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
  - zircon_runtime/src/core/runtime/state/service_entry.rs
  - zircon_runtime/src/core/runtime/handle/resolution.rs
  - zircon_runtime/src/core/runtime/handle/activation/unload_mutation.rs
  - zircon_runtime/src/core/runtime/handle/core_handle.rs
  - zircon_runtime/src/core/runtime/weak.rs
  - zircon_runtime/src/navigation/module.rs
  - zircon_editor/src/ui/host/module.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/host/editor_ui_host.rs
tests:
  - runtime service instance does not retain CoreRuntime after external handles are dropped
  - editor manager service releases CoreWeak after CoreRuntime drop
  - repeated editor runtime fixture creation returns worker threads to baseline
  - builtin navigation module obeys Driver-to-Driver and Manager-to-Driver dependency layers
  - isolated default and editor.runtime_diagnostics product startup reaches first frame
  - current 3157-test Editor full-lib reaches a natural summary without thread accumulation or a fixture hang
resolved_at: 2026-07-14
---


# Runtime 02：服务实例反向持有 CoreHandle 导致 Runtime 永久保活

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md`
- 来源执行切片：Editor14 M2 / Editor15 M1 full-lib 自然结束门根因下沉
- 修复责任计划：`docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md`
- 关联后续计划：`docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`
- 交接原因：线程以每组 16 条持续累积的最低根因已经从“每个 Runtime 创建任务池”继续下沉为
  Runtime 核心服务注册表的强所有权闭环。服务对象能被 `CoreRuntimeInner` 持有，同时服务构造函数又能把
  `CoreHandle` 存回服务对象；该约束归 Runtime02 的 core/runtime/service-registry 所有权合同，不归
  Editor14 的调度配额，也不能先由 Runtime11 的共享任务池方案掩盖。

## 失败现象与复现证据

Windows official validator job `9c0bba0554b042c2b3c5a139a8bb10a7` 的 Editor full-lib test child
达到 5547 threads 后停止产生 harness summary。对同一 test binary 使用
`--test-threads=1 --nocapture` 后，进程仍从 8 threads 持续增长到 3165、3887、4091；窄分区
`tests::host::manager::` 单线程执行时峰值 549 threads，最终自然结束为
62 passed / 17 failed / 3035 filtered out（50.78s）。线程创建时间以当前机器每个 Runtime 的
16 个 task-pool worker 为一批，证明线程占用随测试 fixture 创建的 Runtime 数线性累积。

当前源码给出完整且无需推测的强引用闭环：

```text
CoreRuntime
  -> Arc<CoreRuntimeInner>
  -> CoreRuntimeInner.services
  -> ServiceEntry.instance: Option<ServiceObject>
  -> Arc<EditorManager>
  -> EditorManager.host: EditorUiHost
  -> EditorUiHost.core: CoreHandle
  -> Arc<CoreRuntimeInner>
```

具体证据如下：

- `CoreRuntime::new()` 创建 `Arc<CoreRuntimeInner>`；`CoreRuntimeInner` 内部直接拥有 services map。
- 服务解析完成后，`CoreHandle::resolve_existing_service_inner` 把工厂结果写入
  `ServiceEntry.instance`。
- Editor 模块工厂调用 `EditorManager::new(core.clone())`，随后把 manager 作为 `ServiceObject`
  交回注册表。
- `EditorManager` 拥有 `EditorUiHost`，而 `EditorUiHost` 字段 `core` 是强 `CoreHandle`；
  `CoreHandle` 内部又是同一个 `Arc<CoreRuntimeInner>`。
- Runtime 已具备 `CoreWeak`，且 `ModuleContext` / `PluginContext` 已使用弱引用，说明非拥有型反向访问
  已有权威合同；Editor service 当前绕过了这条所有权边界。
- `deactivate_module` 的卸载路径可以显式把 service instance 置空，但普通 `drop(CoreRuntime)` 无法触发
  环内对象析构；依赖调用者先显式卸载才能避免永久保活，不满足 Rust 资源 owner 的 drop 后置条件。

诊断日志：`.codex/tmp/editor14-full-nocapture-rootcause-20260713.out.log`、
`.codex/tmp/editor14-full-nocapture-rootcause-20260713.err.log`、
`.codex/tmp/editor14-manager-pool-leak-20260713.out.log`、
`.codex/tmp/editor14-manager-pool-leak-20260713.err.log`。

### 2026-07-13 Render18 上层产品启动复现

Render18 HybridGI 的 Editor 实际值/降级诊断窗口验收在同一 `CoreWeak` 硬切源码上增加了产品级复现：

- 当前一致源码可通过
  `cargo build -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --locked --jobs 1`；
  首轮 build 曾在并发修改期间出现 `CoreError::RuntimeUnavailable` E0599，重跑一致快照后通过，证明该
  E0599 是编译期间 Runtime/Editor 源码快照漂移，不是当前 API 缺失。
- 使用独立 `ZIRCON_CONFIG_PATH`、独立 `LOCALAPPDATA/APPDATA` 和 DX12/WGPU validation 启动
  `zircon_editor.exe --builtin-view editor.runtime_diagnostics`，进程只创建 0x0 无标题辅助窗口，约
  59 秒后以 `thread 'main' has overflowed its stack` 退出；没有可验收主窗口。
- 使用另一套独立配置启动默认布局并设置 `ZIRCON_EDITOR_EXIT_AFTER_FIRST_FRAME=1`，同样未完成首帧，约
  96 秒后以 Windows exit `-1073741819` 和同一 main-thread stack overflow 退出。这排除了
  Runtime Diagnostics 单视图、用户现有 session schema 和 HGI stats 数据链作为最低原因。
- 非隔离默认启动会更早被旧 session JSON 的 `missing field summary` 阻断，因此不能用于判定当前
  产品 startup；隔离复现才是本失败的权威上层证据。

新增日志位于 `.codex/tmp/hybrid-gi-editor-isolated-20260713-1655/stderr.log` 与
`.codex/tmp/hybrid-gi-editor-isolated-default-20260713-1658/stderr.log`。Render18 不在本 handoff 修复前
申领真实 Editor Runtime Diagnostics UI 门完成，但其 provider -> `RenderStats` -> pane payload 单元链已
独立通过，不应与本 startup 生命周期失败混为一项实现缺陷。

## 最低共享层根因

最低已证实根因是 Runtime02 的 service-registry 所有权合同允许“Runtime 拥有的 service instance
反向拥有 Runtime 强根”。一旦任一服务把工厂参数 `&CoreHandle` clone 后长期保存，就会形成无法由
`Drop` 打开的 `Arc` 环；该 Runtime 内的 `TaskPools`、scheduler、event bus、其他 service instance
及它们的 worker/receiver 会一起永久保活。

Runtime11 的“每 Runtime task-pool 预算”和“asset worker 是否重复记账”仍需在本失败修复后重新测量，
但它们不是当前 549/5547 threads 无界累积的最低根因。进程级共享池只能降低环中泄漏线程的数量，
不能恢复 Runtime 生命周期，因此不得先用共享池关闭本失败。

## 架构修复验收

- 明确定稿 service factory 参数只用于构造期访问；被 Runtime registry 拥有的 service instance 不得
  长期保存强 `CoreHandle`。需要反向访问 Runtime 时使用 `CoreWeak`，并在每次操作边界显式 upgrade，
  Runtime 已销毁时返回类型化错误或无操作结果。
- Editor host 的强引用硬切为非拥有型合同；不保留兼容字段、双构造器、旧 accessor 或强/弱两套路径。
- 增加生命周期回归：创建 `CoreRuntime` 与 `CoreWeak`，注册并激活依赖模块，解析
  `EditorManager`，释放所有外部 service Arc 后 drop Runtime，断言 `CoreWeak::upgrade()` 为 `None`。
- 增加失败路径回归：service 初始化失败、模块激活失败、项目打开失败及 panic unwind 后均不得保活
  Runtime 或其 service registry。
- 增加资源回归：连续创建并释放至少 128 个隔离 editor Runtime fixture，线程峰值有界且结束后回到
  基线；随后复验 `tests::host::manager::` 与 Editor full-lib 自然产生 summary。
- 增加产品启动回归：当前源码的 `zircon_app --bin zircon_editor` 在独立配置下，默认布局与
  `--builtin-view editor.runtime_diagnostics` 都必须创建尺寸有效的原生主窗口；默认布局的
  `ZIRCON_EDITOR_EXIT_AFTER_FIRST_FRAME=1` 必须自然返回 0，不得 stack overflow。Render18 随后复跑真实
  Runtime Diagnostics 窗口截图和 HGI actual/fallback 可见值验收。
- Runtime02 修复回传后，Runtime11 才重新测量 task-pool/asset worker 的独立预算问题；不能把本失败
  的生命周期验收并入或替换为 Runtime11 的共享任务资源验收。

## 禁止临时方案

- 禁止以进程级共享 `TaskPools`、test-only 单线程池、全局 Runtime 单例或复用带业务状态的 Runtime
  隐藏强引用环。
- 禁止要求所有调用方必须先显式 `deactivate_module` / `shutdown` 才能安全 drop；显式关停可用于
  有序业务收尾，但不能成为打破 registry 自拥有环的唯一手段。
- 禁止在 `Drop` 中依赖清空环内 registry：强引用环存在时 `CoreRuntimeInner::drop` 本身不会开始。
- 禁止保留 `EditorUiHost` 强 `CoreHandle` 兼容字段、旧构造器、re-export shim 或调用点特判。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| Runtime02 core/runtime service registry / Editor14 M2 | service instance 反向强持有 Runtime | `open-最低所有权环已证实并路由` | 2026-07-13 | 源码闭环为 `CoreRuntimeInner.services -> ServiceEntry.instance -> EditorManager -> EditorUiHost.core -> CoreHandle.inner -> CoreRuntimeInner`；manager 窄分区单线程峰值 549 threads，full diagnostic 峰值 4091 threads，official full gate 5547 threads 无 summary。 |
| Runtime02 / Navigation startup support | Driver/Manager 依赖层级硬切与产品首帧 | `product-startup-passed-full-editor-summary仍由Editor14阻断` | 2026-07-13 | 产品首次进入 Runtime 时由注册表拒绝 `SceneNavigationRuntime Driver -> BuiltinNavigationManager Manager`。内部实现硬切为 `navigation.runtime.Driver.BuiltinNavigationRuntime`，场景 Driver 与公开 Manager 均向下依赖并通过 `resolve_driver` 共享实例，未保留旧 Manager 名。当前源码产品构建 8m08s 通过；独立配置默认入口与 `editor.runtime_diagnostics` 均观察到 1688×980 `Zircon Editor` 主窗口，`ZIRCON_EDITOR_EXIT_AFTER_FIRST_FRAME=1` 下自然退出 0、无 stack overflow。专项 lib-test 已进入当前源码编译，但被并行 Frameworks05 的 `tests/prelude.rs` 三项未同步导出阻断，未伪称该测试通过。 |
| Runtime02 / Editor14 向上最终验收 | current Editor full-lib 自然 summary | `fixed-CoreWeak资源环验收完成-独立功能失败保留` | 2026-07-14 | CoreWeak 后的 queued Export 停点被证明是一次性测试 gate 未断开 sender，显式断开后 exact 1/1、0.01s；下一停点是 Windows 两个未分组 `for` 循环把 5000+5000 行放大为约 2500 万行 stderr，分组后 exact 1/1、9.40s，output-capture 既有合同 2/2。随后同一 fresh 3157-test 程序以 `--nocapture --test-threads=1` 自然结束：2975 passed / 144 failed / 38 ignored，2833.59s，first/min/peak/last threads=`1/1/1657/59`；旧 4091/5547 threads 持续增长、永久停滞与 stack overflow 均未复现。Editor14 资源 failure 已具备回传条件；144 项功能断言和 Runtime11 瞬时峰值预算保持独立。 |

## 修复结果与回传

- 根因：Runtime service registry 以 Arc 保存服务实例，而 registry-owned EditorManager、Foundation config/event、Animation、Physics、Sound 等服务反向保存 CoreHandle，形成 CoreRuntimeInner.services 到服务再回到 CoreRuntimeInner 的强引用环，使 Runtime、task pools 和工作线程在外部 handle 释放后仍永久存活。产品启动复验还暴露 Navigation Driver 反向依赖 Manager 的层级错误。
- 架构修复：将 registry-owned 服务统一硬切为 CoreWeak，仅在具体操作边界显式 upgrade 并以 CoreError::RuntimeUnavailable 报告生命周期失效；删除强 CoreHandle 字段/兼容构造路径。Navigation 内部共享实现硬切为 Driver，场景 Driver 与公开 Manager 都向下依赖并 resolve 该 Driver，不保留旧 Manager 注册名或桥接层。
- 验证：Runtime 五项 CoreWeak 精确生命周期测试均通过；Editor registry manager、project-open failure/panic 与 128 次隔离 fixture 精确通过，128 fixture first/peak/last threads=1/23/5；Animation 101、Physics 59、Sound 368 项通过；Editor 产品默认与 runtime_diagnostics 两入口首帧自然退出 0；最终 3157-test full-lib 自然结束为 2975 passed、144 failed、38 ignored、2833.59s，线程 first/min/peak/last=1/1/1657/59，旧 4091/5547 持续增长未复现。
- 回传：Runtime02 service CoreHandle retention cycle 已完成 CoreWeak 硬切并回传 Editor14；Editor14 的两个确定性测试夹具停点也已独立修复并回传 Editor08。144 项 UI/Editor 功能失败不属于本 failure，Runtime11 的瞬时线程峰值预算继续独立 open。
