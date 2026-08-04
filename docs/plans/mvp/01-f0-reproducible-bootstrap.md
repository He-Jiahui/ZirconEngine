---
related_code:
  - zircon_app/Cargo.toml
  - zircon_app/src/bin/editor.rs
  - zircon_app/src/bin/runtime_preview.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_app/src/entry/runtime_entry_app
  - zircon_editor/src/ui/retained_host
  - zircon_runtime/src/dynamic_api/session
related_tests:
  - zircon_app/src/entry/entry_runner/editor/tests/gui_startup.rs
  - zircon_app/src/entry/entry_runner/editor/tests/host_config.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_app/src/entry/runtime_entry_app/config/app_config.rs
  - .github/workflows/profile-feature-contract.yml
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/mvp/00-current-source-baseline-recovery.md
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
status: blocked_by_00
gate: F0
last_refined: 2026-07-24
---

# F0 可重复启动 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: 使用 `subagent-driven-development`（推荐）或 `executing-plans`。测试阶段必须使用 `zircon-dev-validation`、`prefer-windows-validation` 和当前 coordinator validation copy。

**Goal:** 从受支持的干净产品配置构建并启动 runtime/editor，输出可操作的启动诊断，并在呈现首帧后确定性干净退出。

**Architecture:** 先固定 profile feature contract，再构建并 stage 两个产品二进制，最后分别验证成功启动、预期失败和资源释放。F0 只证明产品壳可重复运行，不使用项目内容或编辑功能代替后续门槛。

**Tech Stack:** `zircon_app` binaries、target profile features、Windows Winit/WGPU host、runtime dynamic library、PowerShell process harness。

---

## 1. 入口条件

- [ ] [`00-current-source-baseline-recovery.md`](00-current-source-baseline-recovery.md) 阶段退出清单全部完成。
- [ ] 本 Session 已注册到 F0，已领取 App entry、staging/test harness 和本子计划 lease。
- [ ] 当前 validation manifest 绑定 `Cargo.lock`、Rust toolchain、Windows target architecture 和 source fingerprint。
- [ ] Runtime 02、Runtime 14、Editor 01 中直接影响启动的 open failure 已重新分类为 blocking 或 deferred。

## 2. 固定支持矩阵

F0 必须覆盖以下产品输入：

| 产品 | package/bin | feature 配置 | 启动模式 |
|---|---|---|---|
| Runtime | `zircon_app` / `zircon_runtime` | `target-client,platform-winit,input-gamepad,gamepad-gilrs` | 无项目基础启动；首帧退出 |
| Editor | `zircon_app` / `zircon_editor` | `target-editor-host` | Welcome/默认启动；首帧退出 |
| Runtime library contract | `zircon_runtime` | `target-client`、`target-editor-host` | profile feature check |

`target-server` 继续由全局 profile contract 保护，但不要求它产生桌面窗口；不得用 server/headless 成功代替两个桌面产品。

`zircon_shader_pbr_viewer` 与 runtime binary 一样由 `target-client` 启用，但它不是 F0 的 runtime/editor 产品启动验收对象；profile contract 仍必须覆盖该 binary，防止它经 default feature 或额外 authoring 依赖泄漏。

## 3. 非目标

- 不验收项目创建、资产扫描、场景渲染或编辑器 authoring。
- 不要求高级插件、export wizard 或非默认 editor panels 加载。
- 不接受只构建 `zircon_editor` library crate 而未构建 `zircon_app --bin zircon_editor`。
- 不从 Cargo target 目录直接把零散 EXE 当作完整 staged product。
- 不用 `zircon_shader_pbr_viewer` 的构建或启动结果替代 runtime/editor 产品门禁。

## 4. M1.1 Profile 与二进制构建合同

### 目标

支持矩阵在当前源码下无隐式 default feature 泄漏，两个桌面 binary 都能从 coordinator validation copy 构建。

### 实现切片

- [ ] 验证 `zircon_app/Cargo.toml` 的现有 binary `required-features` 合同：editor 为 `target-editor-host`，runtime 与 `zircon_shader_pbr_viewer` 为 `target-client`；profile matrix 必须覆盖三者且不得引入隐式 default feature。
- [ ] 扩充 profile contract：禁止 `target-editor-host` 依赖 server-only surface，禁止 runtime desktop profile 静默依赖 editor crate 的 authoring owner。
- [ ] 确认 runtime dynamic library、first-party plugin registration 和 editor gateway 所需 DLL/asset 清单可以由 staging owner枚举，而不是运行时猜测 Cargo target 布局。
- [ ] 对 profile 不兼容、缺少 required feature 和 ABI/version mismatch 保留 typed/actionable build or startup error。

### 测试阶段：F0 Profile Build Gate

- [ ] 通过 validator 运行完整 `-RunProfileFeatureContract`，覆盖现有七个 profile case。
- [ ] 由 coordinator validation action 构建 `zircon_app --bin zircon_runtime` 的固定 runtime feature 配置。
- [ ] 由 coordinator validation action 构建 `zircon_app --bin zircon_editor --no-default-features --features target-editor-host --locked`。
- [ ] 构建输出必须来自同一 Windows validation copy；不得使用历史 target artifact。
- [ ] 出现失败时回到最低 package/profile owner，修复后先重跑该 profile，再重跑两个产品 binary。

### 退出证据

- [ ] 七项 profile contract 通过。
- [ ] runtime/editor product binaries 均在 current source 上成功构建。
- [ ] validation manifest 记录 feature、toolchain、target、source fingerprint 和产物位置。

## 5. M1.2 Staged product 与成功启动

### 目标

产品从独立 staging 根目录启动，不依赖当前工作目录碰巧存在的 DLL、template 或 asset；呈现首帧后 exit code 为 0。

### 实现切片

- [ ] 在批准的 `D:\ZirconBuilds`、`E:\ZirconBuilds` 或 `F:\ZirconBuilds` 根下定义 source-bound F0 staging 目录；不得覆盖其他 validation run。
- [ ] stage runtime/editor EXE、runtime library、必需 first-party plugin、editor/runtime UI asset、项目 template 和启动所需配置。
- [ ] 增加 staging manifest，列出逻辑产物、源 build artifact、目标相对路径和内容 hash；运行时不解析该 manifest 作为 fallback。
- [ ] 使用现有 `ZIRCON_RUNTIME_EXIT_AFTER_FIRST_FRAME=1` 与 `ZIRCON_EDITOR_EXIT_AFTER_FIRST_FRAME=1` 入口，验证两者只在实际 presented-frame 路径之后退出；环境变量存在或配置值为 true 本身不构成首帧证据。
- [ ] process harness 捕获 stdout、stderr、exit code、启动耗时、窗口创建/首帧诊断和 teardown 完成标记。
- [ ] 运行结束后确认无残留子进程，staging 目录可重命名并删除验证副本，证明没有持久文件句柄。

### 测试阶段：F0 Product Startup Gate

- [ ] 从 staging 根而不是 repo root 启动 runtime，等待首帧退出并断言 exit code 0。
- [ ] 从 staging 根启动 editor Welcome/默认路径，等待首帧退出并断言 exit code 0。
- [ ] 重复一次相同启动，确认没有依赖第一次生成的临时文件才能成功。
- [ ] 运行后检查 process tree、窗口 host、runtime session、plugin host 和日志 writer 均完成 Drop/close。

### 退出证据

- [ ] 两个产品均产生窗口/首帧已呈现诊断，而不是在初始化前提前退出。
- [ ] 两个产品两次运行 exit code 均为 0，无 lingering process/file handle。
- [ ] staging manifest 与日志属于同一 validation run。

## 6. M1.3 可操作失败与干净退出

### 目标

常见启动失败返回非零 exit code 和稳定、可定位的诊断，不 panic、不无限等待、不留下半初始化进程。

### 实现切片

- [ ] 为 editor 缺失 runtime library、无效 builtin view/互斥 startup args、不可读 staging asset 建立产品级错误断言。
- [ ] 为 runtime 无法初始化 surface/device、无效 project 参数和 ABI/version mismatch 建立 typed startup diagnostic。
- [ ] 确保错误包含 component、requested path/profile、根因和恢复建议；不得只输出 `failed` 或 debug dump。
- [ ] teardown 对 initialization partial state 幂等：window、gateway、plugin host、runtime session 和日志 writer 只释放一次。
- [ ] process harness 对超时执行受控终止并把超时视为失败；不得把强杀后的 exit 当作 clean exit。

### 测试阶段：F0 Startup Failure Gate

- [ ] 在 staging 副本中逐个制造声明的可恢复失败，不修改权威 build 产物。
- [ ] 断言每个失败非零退出、诊断字段完整、没有 panic/backtrace-only 用户消息。
- [ ] 恢复被移走的 staging 输入后重新运行成功路径，确认失败没有污染持久状态。
- [ ] 检查所有失败路径无残留进程和文件锁。

### 退出证据

- [ ] 成功与失败路径均确定性结束。
- [ ] 失败诊断可由 operator 直接定位到缺失输入、无效参数或不兼容 profile。
- [ ] F1 可以复用同一 staged editor 开始项目创建/打开。

## 7. F0 阶段退出清单

- [ ] M1.1、M1.2、M1.3 全部通过。
- [ ] runtime/editor 均由 `zircon_app` product binary 证明，不是 library-only smoke。
- [ ] build、stage、run 和 diagnostics 绑定同一 current-source validation copy。
- [ ] 成功路径至少两次运行，失败路径恢复后还能再次成功。
- [ ] 无遗留进程、文件锁或依赖 repo working directory 的隐藏输入。
- [ ] 只在本表写一条 accepted F0 outcome。

## 状态与产出记录

每个里程碑测试通过后记录一次；实现切片不单独写入产出记录。

| 里程碑 | 范围 | 状态 | 完成日期 | 验证批次 / 残余风险 |
|---|---|---|---|---|

## Code Review 收敛结果（2026-08-01）

- 已按当前 `zircon_app/Cargo.toml` 固定三项 binary feature 合同，并明确 `zircon_shader_pbr_viewer` 只进入 profile 泄漏检查、不替代 runtime/editor 产品启动门禁。
- 已把 M1.2 收敛为验证现有环境变量入口确实在 presented-frame 后退出；source guard 或配置布尔值不能替代窗口呈现证据。
- 当前 source 只证明机制和声明存在，尚无同一 current-source validation copy 的 build/stage/run 证据，因此 F0 继续保持 `blocked_by_00`，所有验收复选框保持未完成。
