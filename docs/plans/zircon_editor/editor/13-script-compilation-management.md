---
related_code:
  - zircon_runtime/src/script/mod.rs
  - zircon_runtime/src/script/vm
  - zircon_runtime/src/scene/dynamic_scene/scene/mod.rs
  - zircon_editor/src/core/editor_event/mod.rs
  - zircon_app/src/entry/entry_runner/editor.rs
reference_sources:
  - dev/Fyrox/editor/src/lib.rs
  - dev/godot/editor/plugins/script_editor_plugin.h
  - dev/UnrealEngine/Engine/Source/Developer/HotReload
plan_sources:
  - docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor/04-pie-and-simulation.md
  - docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md
status: in_progress
---

# 13 脚本编译管理

## 参照证据（dev/）

**Fyrox 构建队列模型**（`dev/Fyrox/editor/src/lib.rs:327-338`）：

```rust
Build {
    queue: VecDeque<CommandDescriptor>,   // 多步构建命令队列（顺序执行，前步失败即止）
    process: Option<std::process::Child>, // 当前步的外部进程
    play_after_build: bool,               // 构建成功 → 自动进 Play
}
```

要点：**构建是命令队列不是单命令**（编译→拷贝产物→校验可以是三步）；`play_after_build` 把 Build 与 Play 编排为单用户动作——04 已采纳该状态机，本计划提供其 `Building` 态的内容物。

**godot 即时反馈**（`script_editor_plugin.h`）：保存即重解析、错误即时进面板、脚本与场景编辑无缝穿插——**编译反馈延迟是脚本 DX 的核心指标**，触发策略必须支持保存即编。

**UE 双机制教训**（`Developer/HotReload`）：Hot Reload（整模块换装）因对象数据损坏被 Live Coding（函数级补丁 + Object Reinstancing）取代——**热接入协议必须显式处理"实例状态如何迁移"**，否则宁可重启会话。

## 现状与证据（zircon）

**脚本子系统是自研 VM**（`zircon_runtime/src/script/vm/` 实测条目：`backend/ gameplay_host/ host/ module/ plugin/ runtime/ scene_hook/ + capability_set.rs handles.rs runtime_context.rs`）。**编译接缝已定位**：`vm/backend/zr_vm_project_backend/real_backend/package.rs` 是当前唯一含 compile 语义的文件——M2 会签的 VM 编译入口以该 backend 为锚点，不另起炉灶。依 `runtime/13` 权威计划（现状节实测）：

- 绑定宏三族：`zircon_host_function / zircon_host_module / ZirconScriptType`（经 `zircon_runtime_reflection_macros` 再导出）；
- 宿主函数权威清册：52 函数 / 6 模块 / 2 类型描述符（`docs/zircon_runtime/script/vm/host/function_ledger.md`）；
- marshalling 三分类：值类型（serde）/ 句柄（handles.rs）/ 序列化缓冲；
- 脚本-ECS 单点：`gameplay_host` facade 是唯一玩法面。

**结论：主脚本形态是 VM 模块（源→VM 模块产物），非 Rust dylib**——编译管理的主对象是 VM 模块编译；Rust crate 动态库形态为 runtime/13 的开放路线，本计划作条件里程碑。

**热接入底座**：`DynamicScene` 热重载队列 + `AssetReloadFrameApplyReport { applied, failed, stale, pending_count }` 按帧应用报告（`reports.rs:70-73`）——脚本模块若走资产管线（`import_*` 家族增脚本导入器），重载通道免费获得。

**编辑器侧空白**：无编译触发（watch/手动/Play 前置皆无）；无诊断面板契约；无编译状态呈现；CI 入口应注册 `build-scripts` commandlet 并通过 `--run build-scripts` 调用，当前尚未注册。

## 目标

1. **`ScriptBuildOrchestrator`**（编辑器侧）：
   - 触发源三路：保存触发（watch 脚本源目录，去抖 300ms 合批且首事件 1000ms 硬截止）/ 手动命令（08：`zircon.script.build`）/ Play 前置（04 `Building` 态委托）；
   - 构建准入为 active generation + 至多一个 coalesced pending generation，按 `Watch < Command < Play` 提升意图；`BuildStep::{CompileModules(Vec<ModuleRef>), ValidateLedger, RefreshBindings}`——多步、前步失败即止；
   - 状态机 `Idle / Building{queue_pos} / Succeeded{artifacts} / Failed{diagnostics}`，事件入 01 bus；`play_after_build` 语义由 04 消费。
2. **诊断契约**（DTO 入 `zircon_runtime_interface/src/script_diagnostics/`）：`ScriptDiagnostic { severity, module, file, line, col, code, message, related: Vec<RelatedInfo> }`——VM 编译器产出、编辑器汇聚；面板数据源（按模块分组/按严重度过滤/点击跳转——外部 IDE 或内部查看器，17 设置项 `script.editor.open_mode`）；状态栏徽标（错误/警告计数）。
3. **产物热接入**：VM 模块产物走资产热重载队列（脚本模块注册为资产类型——09 `AssetTypeRegistry` 一条目 + importer 一枚）；应用报告 `AssetReloadFrameApplyReport` 已按帧回流 → 02 `WorldFact::AssetReloadApplied` → 编辑器提示；**实例状态迁移**：VM 层 reload 前对模块实例状态做 serde 快照、reload 后按字段名重放（runtime/13 的 marshalling 值类型通道复用），字段失配处置（缺省填/丢弃）入报告——UE 教训的显式防线；迁移失败 → 保持旧模块运行 + 诊断。
4. **Play 链路与互斥**：`Building` 期间 Play 按钮转等待（04 状态机）；编译失败 → 中止 Play 并聚焦诊断面板；编译 job 与导入 job 在 14 门面同属互斥资源（脚本产物即资产，避免编译写/导入读竞态）。
5. **CI/无头**：`build-scripts` commandlet（16 注册表）：全量编译 + 诊断 JSON 输出 + 非零退出码——`zircon_editor --run build-scripts` 是唯一产品无头入口。
6. **（条件）Rust crate 形态**：若 runtime/13 落地 dylib 玩法路线——cargo 子进程封装（继承 `tools/dev-fast-build.ps1` 的 profile/共享 target-dir/磁盘策略纪律）、产物校验、重载即重启 PIE（不做状态迁移的诚实降级）。

## 非目标

- 脚本语言/VM/绑定本体（runtime/13）；内置代码编辑器（外部 IDE 优先，内部只读查看器）；函数级热补丁（Live Coding 等价物，远期）；调试器（断点/单步——独立计划，诊断契约为其预留 `code` 字段）。

## 架构设计

### 模块布局

```
zircon_editor/src/core/script_build/
  mod.rs
  request.rs          # request/step/dispatch/completion 值对象
  orchestrator.rs     # 状态机 + 三触发源 + bounded generation admission + step 队列
  watch.rs            # 后续源目录监视适配（复用 runtime asset/watch 事件，经 gateway）
  diagnostics_sink.rs # 汇聚 + 面板数据源投影
zircon_runtime_interface/src/script_diagnostics/   # DTO
# VM 编译入口与实例状态快照协议在 zircon_runtime/src/script/vm/（runtime/13 owner 会签）
```

### 数据流

```
源变更(watch 去抖合批) | 手动命令 | Play 前置
  → orchestrator 组 BuildStep 队列 → 14 job(Compile 类别, 互斥组=script_artifacts)
  → VM 编译(runtime script owner 入口) → ScriptDiagnostic 流式回传(进度 sink)
  → Failed: 面板+徽标+事件 | Succeeded: 产物落资产路径 → 热重载队列
  → AssetReloadFrameApplyReport → 状态迁移报告 → bus → 通知
```

### 深度测试

假编译器夹具（可编程延迟/诊断序列/产物）驱动 orchestrator 全状态机——真实 VM 未接线前即可完成 M1 全部验收；真实接线只替换 `BuildStep::CompileModules` 的执行体。

## 里程碑

### M1 编排器与诊断面（假编译器）

- 切片 1.1：`orchestrator.rs` 状态机 + 三触发源（watch 去抖合批/命令/Play 委托接口）+ 队列语义（前步失败即止）；假编译器夹具。2026-07-18 已完成纯领域生产核心与测试夹具；异步 completion 绑定原 dispatch 的 request+step 双身份，拒绝同 request 旧步骤迟到推进当前步骤。2026-08-11 已补 first-event max latency、20 路径/64KiB 双预算、单 pending generation、Command/Play single-flight、Play precedence 与显式 Cancelled outcome；受管 Rust 测试仍待仓库级 artifact gate 解禁，见 [子计划记录](13/2026-07-18-script-build-orchestrator-m1.md)与 [open failure](13/failure-2026-07-22-script-build-debounce-admission-backpressure.md)。
- 切片 1.2：`script_diagnostics` DTO + `diagnostics_sink.rs`（分组/过滤/跳转动作发 bus 消息）+ 状态栏徽标数据源。2026-08-11 已完成 DTO、Editor17 canonical log severity/jump 投影与 generation/request/step 有界去重的 current-source 实现；静态合同 8/8、scoped rustfmt/diff check GREEN，受管 Rust 测试仍被仓库级未登记 D/E/F artifact gate 阻塞，failure 保持 open，见 [交接记录](13/failure-2026-08-05-script-build-diagnostics-editor-log-source-bridge.md)。
- 测试阶段：`cargo test -p zircon_editor --lib --locked`（状态机全迁移矩阵/去抖合批时序/队列中断/诊断汇聚分组）+ `cargo test -p zircon_runtime_interface --locked`（DTO 往返）。更新 `docs/zircon_editor/core/script_build.md`。

### M2 真实 VM 接线与热接入

- 切片 2.1：VM 编译入口会签接线（runtime/13 owner）：`CompileModules` 执行体 + 诊断流式回传；脚本模块资产类型注册（09）+ 导入器。
- 切片 2.2：实例状态快照-重放协议（runtime script owner 实现，编辑器消费报告）；失配处置矩阵；失败保旧模块路径。
- 测试阶段：`cargo test -p zircon_runtime --lib --locked`（编译入口/快照重放往返/字段失配矩阵）+ 端到端：夹具工程改脚本→保存→编译→热载→行为变化断言（`function_ledger` 契约不回归）。

### M3 Play 链路与 commandlet

- 切片 3.1：04 `Building` 委托接通（play_after_build 全链）+ 14 互斥组落地。
- 切片 3.2：`build-scripts` commandlet（诊断 JSON + 退出码）注册；CI 可选 job 挂夹具工程。
- 测试阶段：Play 前置失败中止路径测试；commandlet 子进程集成测试（退出码/JSON 形状）；证据记状态节。

### M4（条件）Rust crate 形态

- 前置：runtime/13 采纳 dylib 路线。切片：cargo 封装（dev-fast-build 纪律）/产物校验/重载即重启 PIE。
- 测试阶段：真实小工程端到端（CI 可选）；共享 target-dir 竞态规避验证。

## 风险与开放问题

- 实例状态快照-重放的保真度依赖 runtime/13 反射迁移能力排期——会签若判定 M2 内无法交付，热接入降级为「重载即重建模块实例（状态清零）+ 显著提示」，诚实降级而非静默损坏（UE 教训）。
- watch 去抖窗口与用户 IDE 批量保存（格式化器多文件写）的交互：合批窗口设置化（17），默认 300ms，超 20 文件强制合为单次全量编译。
- 诊断跳转的外部 IDE 协议（vscode://、fleet://）仅尽力而为，失败回退内部查看器——不作为验收项。

## 产出记录与时间

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

产出明细位于 [2026-07-18 ScriptBuildOrchestrator M1.1 子计划](13/2026-07-18-script-build-orchestrator-m1.md)。M1.2 的 DTO/canonical-log sink 已有 current-source 实现但受管 Rust 验证未完成；状态栏徽标、M2 真实 VM、M3 Play/job/commandlet 接线仍未完成，本计划保持 `in_progress`。

- 2026-08-11 性能修复：watch batch 具备首事件 1000ms 硬截止与 20 路径/64KiB 双预算，超限立即折叠 full-rebuild sentinel；无界 `VecDeque` 已替换为单 pending generation，等价 Command/Play 共享 typed request/generation id，并按 `Watch < Command < Play` 提升到 latest Play resume intent。独立 Rust 行为夹具 6/6（含 1M explicit storm）、Editor13 静态合同 8/8、scoped rustfmt/diff check GREEN。failure 仍 open：Editor14 共享 job ticket 的 entry/bytes/oldest-age 与取消接线、产品 caller F4 trace、current-source managed Cargo 待完成；见 [open failure](13/failure-2026-07-22-script-build-debounce-admission-backpressure.md)与 PERF-MVP-557。
- 2026-07-30 current-source性能复核：`core/script_build/**`4/4、912行、13 tests已按稳定SHA逐文件复读；20-path+sentinel、Arc outcome与linear dispatch ticket继续成立。除`core/mod.rs`导出外仍无watch/command/Play/job/VM/commandlet产品caller，不把接线前风险写成当前UI实测。PERF-MVP-557补充fixed three-step Vec、dispatch最多20个PathBuf clone、持续debounce starvation及Command/Play无界admission；M2/M3前必须用source generation、bounded/coalesced ticket和Editor14/Runtime11唯一job owner收口。rustfmt/whitespace GREEN，managed Cargo、规模counter与F4仍待；证据见`../../performance/01/2026-07-30-editor-core-script-build-current-review.md`。
