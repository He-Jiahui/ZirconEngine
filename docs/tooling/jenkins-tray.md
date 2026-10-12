---
related_code:
  - tools/dev/jenkins-validate.ps1
  - tools/dev/jenkins_validate.py
  - tools/jenkins/tray/formal.py
  - tools/jenkins/tray/formal_app.py
  - tools/jenkins/tray/formal_startup.py
  - tools/jenkins/tray/__main__.py
  - tools/jenkins/deployment/keeper_control.py
  - .jenkins/deployment-spec.json
implementation_files:
  - tools/dev/start-jenkins-sandbox.ps1
  - tools/jenkins/install-jenkins-tray-startup.ps1
  - tools/jenkins/jenkins_coordinator.py
plan_sources:
  - docs/plans/jenkins-coordinator-pilot.md
  - docs/tooling/jenkins-coordinator-pilot.md
tests:
  - tools/jenkins/tests/test_keeper_control.py
  - tools/jenkins/tests/test_keeper_control_transactions.py
  - tools/jenkins/tray/tests/test_config_operations.py
  - tools/jenkins/tray/tests/test_service.py
  - tools/jenkins/tray/tests/test_startup.py
  - tools/jenkins/tray/tests/test_lifecycle.py
  - tools/jenkins/tray/tests/test_terminal.py
  - tools/jenkins/tray/tests/test_coordinator_activation.py
  - tools/jenkins/tray/tests/windows_ui_acceptance.py
  - tools/jenkins/tray/tests/windows_runtime_acceptance.py
  - tools/jenkins/tray/tests/windows_recovery_acceptance.py
doc_type: operator_workflow
---

# Jenkins Windows 托盘

当前正式托盘使用工程部署规格 `E:\Git\ZirconEngine\.jenkins\deployment-spec.json`，打开 [本机 Jenkins](http://127.0.0.1:18080/)。Jenkins Home 位于 `.jenkins\jenkins_home`；运行组件、agent、日志、缓存、临时目录和托盘状态均在工程 `.jenkins` 内。流水线编译产物独立使用请求明确指定的真实 `D/E/F:\cargo-targets`，默认 `D:\cargo-targets`。

托盘退出不会停止 Jenkins。使用“停止 Jenkins”菜单执行有边界的停止流程；登录启动项只启动托盘，不自动启动 Jenkins。

正式菜单包含状态、打开 Jenkins、启动、停止、重启、查看日志、刷新、登录启动项和退出。界面必须位于当前用户的 `WinSta0\Default` 桌面，Explorer 图标和实际右键菜单分别核验；聊天沙箱私有桌面的隐藏窗口不能代表桌面托盘可用。

启停请求由正式受限 keeper 执行，绑定原部署清单、选中版本、精确 PID 出生时间、exe 和命令摘要。界面只提交持久化命令并观察原结果；输出未知时保留原意图，不重复发起。停止须核对原生进程树终态、输出流 EOF 和历史保护。日志及控制状态位于 `.jenkins\logs` 和 `.jenkins\state\tray`；keeper 协议证据位于 `.jenkins\state\formal-lifecycle`。

截至 2026-10-10，正式选中版本为 `0c1f8adb…`。可见托盘、实际右键菜单及停止、启动、单请求重启均已实测通过；服务恢复后 controller、Windows agent 和 79 个插件就绪。两轮受控 Rust 增量补丁均通过真实编译、测试与正式验收，第二轮复用兼容增量池并重新执行测试。实际阶段及回执见[启动恢复记录](../plans/jenkins-session-coordination/01/2026-10-09-selected-runtime-recovery.md)。这些结果不关闭整仓、迁移回退或 M5–M8 的其余门槛；旧本地协调器持续退役。2026-10-11 runtime、M5 与缓存维护边界见[状态记录](../plans/jenkins-session-coordination/01/2026-10-11-runtime-completion-and-cache-maintenance.md)。

## 正式增量补丁入口

日常入口为 [jenkins-validate.ps1](../../tools/dev/jenkins-validate.ps1)，按 `dry-check`、`submit`、`query` 顺序处理同一个请求。`-SessionId` 使用本会话已有的源码授权，`-RequestId` 和 `-AttemptId` 标识本次工作；补丁提交携带 `-Patch`、逐路径的修改前 SHA-256 清单 `-BeforeHashes`、验证范围 JSON `-Coverage`，并用 `-Dependency` 声明范围内需要封存的依赖。清单和 JSON 可使用绝对文件路径。明确指定 `-BuildRoot` 后不自动换盘。`dry-check` 只检查输入，`submitted` 只表示已提交，正式结果以原请求的验收回执为准。

```powershell
& 'E:\Git\ZirconEngine\tools\dev\jenkins-validate.ps1' query `
  -Repository 'E:\Git\ZirconEngine' `
  -SessionId '<原 session ID>' -RequestId '<原 request ID>' `
  -AttemptId '<原 attempt ID>' -BuildRoot 'D:\cargo-targets'
```

共享 checkout 中已应用的修改使用 `-AlreadyApplied`，同时携带不可变 `-BaseHead`、`-BeforeHashes`、`-AfterHashes` 及 `-DependencyHashes`；无依赖也须显式传入空对象。协调器核对前后 hash、归属与封存输入，不覆盖外来编辑。交付未知时保留原身份并查询，不能换 ID 重投。

已验收且不提交 Git 的任务，可用 `release-source` 交接本任务的编辑占用：携带原 session/request/attempt、原 `-PatchRequestRef` 和独立 `-ReleaseRequestId`。它按路径版本和 hash 核对后保留源码、验收、消费者及产物引用；同一释放 ID 再调用只读对账原结果。该入口已实测完成第二轮源码交接，并验证重复调用不改变状态库。提交、推送、通知和 GC 分别遵循任务已有授权。

下文保留独立 pilot 的历史配置、命令和验收说明，供既有回执核对；它们不作为当前工程部署的操作入口。

## 历史 pilot 配置

配置文件由 `python.exe -B tools/jenkins_tray/launch.py --config <config> configure ...` 创建，当前配置必须包含：`repoRoot`、`pilotRoot`、`pythonExecutable`、`pythonwExecutable`、`vsDevCmdPath`、`stateDir`、`rootIdentity`、`pythonSha256` 和 `pythonwSha256`。`pilotRoot` 必须是经过物理身份绑定的根级 `D:\cargo-targets`、`E:\cargo-targets` 或 `F:\cargo-targets` 下的 Jenkins pilot 根。

所有操作重新校验根身份、Jenkins manifest、解释器摘要和物理路径。状态观察不改写 Jenkins 请求日志或生命周期状态；生命周期记录写入配置的 `stateDir`，而 Jenkins 自己的 receipts、历史和构建数据仍由 pilot 根保管。

## 命令

当前主配置文件为 `E:\Git\ZirconEngine\.codex\state\jenkins-tray\main.json`。从仓库根使用配置绑定的解释器运行：

```powershell
$config = 'E:\Git\ZirconEngine\.codex\state\jenkins-tray\main.json'
$python = (Get-Content -Raw $config | ConvertFrom-Json).pythonExecutable
& $python -B 'E:\Git\ZirconEngine\tools\jenkins_tray\launch.py' --config $config status
& $python -B 'E:\Git\ZirconEngine\tools\jenkins_tray\launch.py' --config $config start
& $python -B 'E:\Git\ZirconEngine\tools\jenkins_tray\launch.py' --config $config stop-preview
& $python -B 'E:\Git\ZirconEngine\tools\jenkins_tray\launch.py' --config $config reconcile
& $python -B 'E:\Git\ZirconEngine\tools\jenkins_tray\launch.py' --config $config validate-config
```

每条命令返回 JSON。`status` 返回 `stopped`、`starting`、`ready`、`busy`、`degraded` 或 `error`，并包含 controller、agent、活动构建和队列数量。直接执行 `start`、`stop` 或 `reconcile` 使用已验证的 `python.exe` 完成管理操作；托盘 UI 通过 detached background worker 调用相同入口，避免阻塞窗口消息循环。状态读取是只读的；worker wrapper 将结构化响应写入 `stateDir/responses`，生命周期状态写入 `stateDir/operation.json` 和对应 operations 记录。UI 丢失响应时从这些记录恢复，不会盲目重复副作用。

`stop-preview` 先读取活动构建、队列数量、控制器和节点身份，返回 `confirmationRequired`、`createdAt`、`rootIdentity`、`activeBuilds` 和公开的 PID/创建时间记录。活动构建存在时，停止命令必须提交未过期且完全匹配的确认记录；确认有效期为 120 秒。托盘先显示任务清单，点击“否”不会创建停止操作。CLI 使用时，将预览响应的 `result` 对象保存到本配置 `stateDir/confirmations` 下的 JSON 文件，审阅后运行 `stop --confirmation-file <该文件的绝对路径>`；无运行任务时可直接运行 `stop`。

停止先进入 quiet-down，只取消已核验的确切活动构建。Pipeline `/stop` 未收敛时，3 秒后对同一构建发一次 `/term`，8 秒后仍运行才发一次 `/kill`；每次升级都重新核验构建参数。`/kill` 可能跳过 Pipeline 的归档步骤，因此必须继续等待独立 keeper 执行器的本地回执、Windows Job 零进程、stdout/stderr EOF 和 Cargo 缓存锁释放。整个取消与资源对账期限为 120 秒，未证实终态就保留原操作并显示异常，不关闭控制器或抹掉失败。该流程符合 [Jenkins 构建中止接口](https://www.jenkins.io/doc/book/using/aborting-a-build/)。

资源终态通过后，先持久化队列，再验证并停止受管节点与控制器。排队请求的身份、历史、构建产物、receipts 和锁保留；再次启动恢复本次拥有的静默模式，继续原队列。

托盘每五秒在后台刷新一次状态。网络、进程身份和生命周期调用不在窗口消息线程执行；worker 超时或退出会留下可对账记录。只有当前根、owner、keeper、PID 创建时间和 Java 运行证据全部匹配时，托盘才报告在线或执行停止。

## Windows 登录启动

```powershell
& powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File tools/jenkins/install-jenkins-tray-startup.ps1 -Action Query -ConfigFile $config
& powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File tools/jenkins/install-jenkins-tray-startup.ps1 -Action Install -ConfigFile $config -DryRun
& powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File tools/jenkins/install-jenkins-tray-startup.ps1 -Action Install -ConfigFile $config
& powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File tools/jenkins/install-jenkins-tray-startup.ps1 -Action Update -ConfigFile $config
& powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File tools/jenkins/install-jenkins-tray-startup.ps1 -Action Remove -ConfigFile $config
```

脚本只管理 `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` 中以 `ZirconJenkinsTray-` 加配置摘要命名的值。它验证解释器摘要、物理路径和 `validate-config` 结果，写入的命令固定为：

```text
"<pythonw.exe>" -B "<repo>\tools\jenkins_tray\launch.py" --config "<config.json>" tray
```

脚本不创建 Windows 服务或管理员计划任务；`Query` 不写注册表；`-DryRun` 不写注册表；`Remove` 只删除命令完全匹配的自有值并保留其他登录项。

## 异常恢复与卸载

Windows 重启或 keeper 退出后，点击“启动 Jenkins”会先执行已有所有权恢复协议，检查旧 keeper 确切死亡、历史回执、封存驱动、物理根和原生锁，追加新的所有权链。身份不明、历史未终结或锁仍占用时拒绝恢复，不创建第二次执行。

启停时退出或重启托盘，后台 `python.exe` 会继续原操作。托盘读取持久化记录，工作进程仍活跃时等待；已退出时对账原操作。超时记录保留供排查，运行 `reconcile` 观察原结果。日志菜单打开本根的 `logs/tray`；不要公开凭据文件、agent secret 或原始 Java 命令行。

卸载先对主配置执行 `Remove`，再使用“退出托盘”。它不删除 Jenkins 数据或停止服务；需要关闭服务时先用“停止 Jenkins”。Explorer 重建任务栏时，托盘响应 `TaskbarCreated` 重新添加图标；重复运行同一根的托盘会退出第二个实例。

## 协调器启用和功能测试

`tools/jenkins_coordinator.py activate` 仅在 Windows 托盘全项验收、独立 pilot I1–I6 回执、当前根/驱动/在线节点与退役标记全部匹配时，创建 `.codex/state/jenkins-coordinator/active.json` 和对应独立激活回执。它不改写既有验收回执或旧协调器数据。检查当前启用状态：

```powershell
& $python -B 'E:\Git\ZirconEngine\tools\jenkins_coordinator.py' status
& $python -B 'E:\Git\ZirconEngine\tools\jenkins_coordinator.py' submit --request-file $request --bundle $bundle
& $python -B 'E:\Git\ZirconEngine\tools\jenkins_coordinator.py' reconcile --request-file $request
& $python -B 'E:\Git\ZirconEngine\tools\jenkins_coordinator.py' verify --request-file $request
```

`$request` 和 `$bundle` 必须是已准备的不可变 RequestIdentity JSON 与封存 ZIP 的绝对路径，不能用新身份重投待处理请求。功能测试按 [jenkins-coordination 技能](../../.codex/skills/jenkins-coordination/SKILL.md) 执行，目前支持固定 Python、managed Cargo 与 fault 模板。启用记录允许协调器功能测试和有边界的命令提交，每次提交仍检查当前身份与源码摘要。

## 验收边界

托盘验收单独记录自动检查与隔离 Windows 实测，包括菜单和浏览器、后台启停、真实运行任务取消、原队列恢复、原操作跨托盘重启、keeper 恢复、单实例、图标重建、路径/摘要拒绝和注册项幂等性。以对应 `jenkins-windows-tray` 回执及其引用的原始证据为准；失败和超时证据也保留。

运行协调器与全工作区、里程碑及生产迁移接受是独立门槛。当前输入上的每个命令必须取得自己的接受回执；旧 sealed-input 试点结果不授予当前全工作区编译通过，也不自动迁移或重放旧服务的队列。旧数据库、队列、历史、产物、锁和失败回执继续保留。

参考：[Microsoft Shell notification area](https://learn.microsoft.com/en-us/windows/win32/shell/notification-area)、[Run and RunOnce registry keys](https://learn.microsoft.com/en-us/windows/win32/setupapi/run-and-runonce-registry-keys)、[Jenkins quietDown](https://javadoc.jenkins.io/jenkins/model/Jenkins.html#doQuietDown())。
