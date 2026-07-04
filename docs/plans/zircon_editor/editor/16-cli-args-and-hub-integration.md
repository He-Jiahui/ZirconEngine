---
related_code:
  - zircon_app/src/bin/editor.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_app/src/entry/entry_runner/runtime_session_args.rs
  - zircon_editor/src/core/gui_startup_request.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_hub/src/process/editor_launch.rs
  - zircon_hub/src/settings/hub_config.rs
  - zircon_hub/src/build/command.rs
reference_sources:
  - dev/godot/main/main.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/CommandLine.h
plan_sources:
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
status: planned
---

# 16 控制台入口参数与 zircon_hub 交互

## 参照证据（dev/）

**godot 编辑器 CLI 全形态**（`dev/godot/main/main.cpp` 实测行号）：

```
:563  "-e, --editor"                        # 编辑器模式（同体异态：一个可执行体）
:576  "--path <directory>"                  # 工程寻址
:623  "--headless"                          # 无显示服务器
:698  "--export-release <preset> <path>"    # 无头导出三连
:701  "--export-debug <preset> <path>"
:702  "--export-pack <preset> <path>"
:1590 if (arg == "-e" || arg == "--editor") # 解析即分派
```

要点：**导出等无头任务是编辑器可执行体的一等参数**（不是独立工具）；preset 名是 CLI 与工程数据的衔接点（15 计划同款）。

**UE 命令行权威**（`CommandLine.h:58-135`）：`FCommandLine::Get/Set/Append` 单一权威串 + `AddToSubprocessCommandLine/BuildSubprocessCommandLine`——**子进程参数继承是内建 API**（编辑器起 PIE/工具进程时按上下文过滤透传），04 Play 子进程的参数组装照此。

## 现状与证据（zircon）

**入口极薄 + 解析链清晰**（`entry_runner/editor.rs:40-94`，2026-07-05 复核）：`bin/editor.rs` 3 行转发；三段顺序解析——`parse_diagnostic_log_startup_args`（诊断组，先行以初始化进程日志）→ `EditorGuiStartupRequestArgs::parse`（GUI 组）→ 无 GUI 意图时 `EditorCliOperationRequest::parse`（操作组）；**操作路径的输出契约已有雏形**：`run_editor_operation` 响应经 `serde_json::to_string` 打 stdout 后直接返回（:66-70）——commandlet 的 JSON 输出口径以此为基线；GUI 路径 `bootstrap → LoadedRuntime::load_default → RuntimeSession::create_with_profile(runtime, b"editor") → run_editor_with_config`。

**GUI 参数已有**（:154-261）：`--project / --builtin-view / --create-project / --project-name / --location / --template`，折算 `EditorGuiStartupRequest::{OpenProject{project_path}, OpenBuiltinView{descriptor_id}, CreateProject(NewProjectDraft)}`（`gui_startup_request.rs:6-9`）。

**无头操作参数已有**——v1 计划设想的 commandlet 雏形实际存在（:313-361）：`--operation / --args / --operation-group / --list-operations / --operation-stack / --headless`，折算 `EditorCliOperationRequest`，直连 `EditorOperationRegistry`（08 计划将并入命令注册表，`callable_from_remote: bool` 字段即其门禁）。

**runtime 侧参数**：`--project / --runtime-session-profile / --log-level / --log-filter`（`runtime_session_args.rs:37-52`）。

**hub 侧**（实测复核）：`EditorLaunchCommand::new(executable, request: EditorLaunchRequest)` + 三构造器/入口（`from_staged_engine(engine_root, request)` / `from_preferred_engine` / `preferred_editor_executable[_exists]`——**staged 引擎根到 editor 可执行体的寻址约定已成型**）+ `command_line() -> Vec<String>`（组 `--project` 或 `--create-project` 族）+ `launch_editor(&cmd) -> Result<Child, HubError>`（`process/editor_launch.rs:14-92`）——**纯 fire-and-forget**：拿到 `Child` 后无握手、无 ready 回报、无退出关注；`HubConfig { recent_projects: Vec<RecentProject>, project_metadata }`（`settings/hub_config.rs:22`）；hub 亦有 `BuildCommandOptions { python_path, cargo_path, source_dir, output_dir, profile, jobs }` 组 `--targets/--out/--mode/--cargo/--jobs` 调 zircon_build.py（`build/command.rs:6-43`）——source-engine 构建职责边界清晰（CLAUDE.md 既定，不动）。

**缺口**：三段解析（诊断/GUI/操作）各自手工 match，无统一 `EditorLaunchArgs` 事实源；`--operation` 门禁未接 `callable_from_remote`；无 `--scene/--layout/--safe-mode`；hub 启动无握手/单实例/回写（editor 内新建工程 hub 的 recent_projects 不知情）；04 Play 需要的 `--play-scene/--play-report-pipe` 未定；15 需要的 `export --preset --resume` 未注册。

## 目标

1. **`EditorLaunchArgs` 统一事实源**（`zircon_app/src/entry/cli/`）：三段解析合一为类型化结构（诊断组/启动意图组/无头操作组/hub 握手组），保持**零新依赖手工解析**（现风格延续，clap 引入需单独裁决记状态节）；诊断组先行初始化日志的既有时序保留（合一后仍是「先诊断后其余」两拍解析）；`EditorGuiStartupRequest` 与 `EditorCliOperationRequest` 从其派生（两类型保留，构造收敛）；commandlet JSON 输出沿用既有 stdout serde 口径；参数矩阵文档 `docs/zircon_app/cli.md`（含 runtime_preview 参数）。
2. **参数补全**：`--scene <AssetRef 文本>`（打开工程后聚焦场景）/ `--layout <preset-id>`（06 预设）/ `--safe-mode`（12：仅内建插件）/ `--diagnostics <path>`（既有诊断组归入）；runtime_preview 增 `--play-scene <path>` + `--play-report-pipe <name>`（04 M1 会签定名）。
3. **Commandlet 框架**（既有 `--headless --operation` 通道升级）：
   - `--run <commandlet>` 别名规范化（godot `--export-*` 的显名风格）：`--run export --preset <name> [--resume]`（15）/ `--run migrate-assets`（10）/ `--run build-scripts`（13）/ `--run audit-registry`（10）/ `--run plugin-list`（12）——均为 08 合一注册表中 `callable_from_remote=true` 的命令，`--run` 即其 CLI 投影，**不另建注册表**；
   - 无头引导：不建窗口、不物化工作台，`EditorContext` + gateway + jobs 照常（08 registry 迁 core 后天然可用）；退出码语义：0 成功 / 1 失败 / 2 参数错 / 3 能力缺失（`required_capabilities` 不满足）；
   - 无头模块面：编辑器 Lazy 模块链对 Graphics/UI 的依赖在 headless 下的裁剪——按 `Headless` session profile（既有五态之一）+ `EditorCoreProfile` 能力降级路径验证，若模块内核不支持则为 commandlet 定义精简 profile（runtime profile-selection 文档矩阵扩一行，owner 会签）。
4. **hub↔editor 协议 v1**（fire-and-forget 升级）：
   - 握手：hub 传 `--hub-session <token> --hub-protocol 1`；editor 就绪后回报 `ready{pid, project}` / 失败 `fail{reason}`——载体取**文件信箱**（`<project>/.zircon/hub/<token>.json` 原子写）为 M3 基线，命名管道升级留位（跨平台成本证据裁决）；hub 超时（10s）判失败。
   - 单实例：`<project>/.zircon/lock`（PID + 心跳时间戳）；二次打开同工程 → 检活 → 活则发聚焦信号（信箱文件）退出，死则夺锁。
   - 工程回写：editor `CreateProject/OpenProject` 成功 → 追加写共享 recent 文件（`~/.zircon/hub/recent_projects.json` 原子写，hub 启动/焦点时合并进 `HubConfig.recent_projects`）——避免双进程写 hub TOML 本体。
   - 边界不变：source-engine 构建留 hub（`BuildCommandOptions` 原样）；editor 不反调 hub。
5. **子进程参数透传**（UE `BuildSubprocessCommandLine` 语义）：`EditorLaunchArgs` 提供 `subprocess_args(context)` ——Play 子进程继承 `--log-level/--log-filter/--project` 并注入 play 组参数（04 消费）。

## 非目标

- 不引入网络/远程启动；不迁移 hub Installs/自更新；不做 `-e` 式同体异态（editor/runtime_preview 双可执行体格局保持——zircon_app feature 门控既定）；commandlet 的各任务实现归各计划（本计划只建框架与注册投影）。

## 架构设计

### 模块布局

```
zircon_app/src/entry/cli/
  mod.rs / launch_args.rs / parser.rs / subprocess.rs
zircon_editor/src/core/commandlet/
  mod.rs / runner.rs        # 无头引导 + 退出码 + --run 分派（08 注册表投影）
zircon_editor/src/core/hub_link/
  mod.rs / handshake.rs / instance_lock.rs / recent_writeback.rs
zircon_hub/src/process/     # editor_launch.rs 扩：token 注入 + 信箱等待 + 超时
zircon_runtime_interface/src/hub_protocol/   # 信箱 JSON DTO（11 壳，双端共用）
```

### 参数矩阵（定稿草案，M1 文档化为准）

| 组 | 参数 | 消费者 |
| --- | --- | --- |
| 启动意图 | `--project` `--create-project --project-name --location --template` `--builtin-view` `--scene` `--layout` | GuiStartupRequest / 10 |
| 运行形态 | `--safe-mode` `--headless` `--diagnostics` `--log-level` `--log-filter` | 12 / commandlet / 17 |
| 无头任务 | `--run <cmd>` + 命令自有 flag（`--preset --resume` 等，经 `payload_schema_id` 校验）；兼容期保留 `--operation` 族并记退役债 | 08 注册表 |
| hub | `--hub-session --hub-protocol` | hub_link |
| play（runtime_preview） | `--play-scene --play-report-pipe` | 04 |

### 深度测试

新增 commandlet = 08 注册一条 `callable_from_remote` 命令，`cli/`、`commandlet/` 零改动（`--run` 分派即注册表查询）；hub 协议以假对端夹具（tempdir 信箱）双向测试，不起真 hub 进程。

## 里程碑

### M1 EditorLaunchArgs 收敛与参数补全

- 切片 1.1：`entry/cli/` 落地；三段解析迁入合一（`entry_runner/editor.rs` 的 parse 链改为单次构造，旧散点删除）；参数矩阵文档 `docs/zircon_app/cli.md`。
- 切片 1.2：`--scene/--layout/--safe-mode` 贯通消费点；runtime_preview play 组参数（与 04 M1 联合切片）；`subprocess_args` 透传。
- 测试阶段：`cargo test -p zircon_app --locked`（解析矩阵：合法/非法/组合/顺序无关）+ `cargo test -p zircon_editor --lib --locked`；手验 `cargo run -p zircon_app --no-default-features --features target-editor-host --bin zircon_editor -- --project <夹具> --layout debug` 等组合记状态节。

### M2 Commandlet 框架

- 切片 2.1：`core/commandlet/runner.rs`：无头引导（headless profile 裁剪会签结论落地）+ `--run` 分派 + 退出码四值；`--operation` 族兼容映射。
- 切片 2.2：首批接线：`export`（15 M3 联合）与 `plugin-list`（12 数据源直出，最小验证件）；`migrate-assets/build-scripts/audit-registry` 随各自计划落地即注册（本切片留登记位）。
- 测试阶段：子进程集成测试（`--run plugin-list` 退出码 0 + JSON 形状；`--run` 未知命令退出码 2；能力缺失退出码 3）；CI 可选 job 挂 `--run export` 夹具工程。

### M3 hub 协议 v1

- 切片 3.1：`hub_protocol` DTO + editor 侧 `hub_link/`（握手信箱/单实例锁/聚焦信号）+ hub 侧 `editor_launch.rs` 扩（token/等待/超时）。
- 切片 3.2：recent 回写共享文件 + hub 合并读取；全链手验（hub→editor→回写→hub 刷新）。
- 测试阶段：假对端往返测试（握手成功/超时/协议版本不符三路径）；单实例锁矩阵（活进程聚焦/死锁夺回/并发夺锁原子性）；`cargo test -p zircon_runtime_interface --locked`（DTO）。证据记状态节。

## 风险与开放问题

- headless 下编辑器模块链的 Graphics/UI 依赖是最大不确定项——M2 前置会签；最坏情形 commandlet 走 runtime `Headless` profile + 编辑器核心服务子集（context/commands/jobs/asset），工作台族模块不激活，能力缺失的命令退出码 3 诚实拒绝。
- 文件信箱协议的实时性（hub 等待轮询间隔）与锁心跳的进程崩溃窗口：轮询 250ms/心跳 2s/判死 6s 初值，实测调参记状态节。
- `--operation` 族与 `--run` 并存的退役时点：M2 记债，待外部脚本迁移证据（hub/CI 无引用）后删除——硬切换纪律，不长期双轨。
