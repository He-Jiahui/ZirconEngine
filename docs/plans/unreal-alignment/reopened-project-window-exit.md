# 先关闭项目 A、在欢迎页打开 B 后，主窗退出仍看到 A 的关闭状态

ZE 基线 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`。本提交仅调查，不改引擎。结论限定于**同一 retained 宿主已经正常接纳项目 B** 的路径；不要求实现无缝热切换，也不把一个新启动的编辑器进程算作同一实例。

## 1. 两个动作各自正常，连起来哪里出错？

正常打开一个干净的工程 A，执行工作台 [File > Close Project](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/assets/ui/editor/unreal/menus.contract.toml#L1-L5)。[关闭命令](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/core/commands/defaults.rs#L212-L221)使用项目关闭入口，完成后 [释放当前项目并返回欢迎页](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/project_close.rs#L568-L621)，没有请求退出整窗。接着在欢迎页打开另一已存在的干净工程 B；[正式 Open Existing 回调](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/welcome_session/actions/project.rs#L41-L66)把 manager 接纳的项目应用回当前宿主。待 B 的 Runtime activation 和 Scene document 均已生效，再按同一主窗口的关闭按钮。

## 2. 影响与边界

[真正的原生主窗口关闭入口](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/native_window_close.rs#L30-L79)受理的是 B，并非在试图重试 A。但窗口随后进入自己的 `RecoveryRequired`，普通退出不完成。此处未开始 B 的关闭事务，不能由此声称 B 的持久账本进入 Recovery、文件丢失、必须强杀或每种启动路径必现。新进程直接打开 B 与 A 已关闭但仍停留 Welcome 的情况均要分别控制。本题不需要模型导入、后台票据、插件故障、脏文档或 Play。

## 3. 为什么仍然看到上一工程？

[项目级关闭收尾](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/project_close.rs#L624-L643)调用 [coordinator 状态转移](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/host/project_session_close/coordinator.rs#L42-L75)；`finish_closed` 留下 A 的 operation，并保留 `Closed`。专用 reset 会恢复 `Decision` 并清 operation，但在当前产品路径中只有 [再次执行项目级关闭时的 reset](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/project_close.rs#L170-L186)调用它。欢迎页接纳 B 没有构造另一个 retained host，也没有让 native X 经过上述项目级入口。

主窗先 [比对当前 B 与批准的 B](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/main_window_close.rs#L248-L260)；这个检查本来应该通过。随后 [quiesce 的 Closed 分支](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/project_close.rs#L368-L398)给出 `None`，[主窗对照自己的批准身份](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/main_window_close.rs#L282-L305)拿 `None` 与 `Some(B)` 比较，于是在尚未开始资源释放前正确拒绝不一致。应调查的是谁拥有这个过期状态，而不是绕过最后的身份检查。

## 4. Unreal 的可比合同，以及不可比之处

`FUnrealEdMisc::SwitchProject`（`Engine/Source/Editor/UnrealEd/Private/UnrealEdMisc.cpp:2262–2324`）记录目标工程并走主窗正常关闭；退出时启动新实例（`Engine/Source/Editor/UnrealEd/Private/UnrealEdMisc.cpp:1400–1441`）把下个工程交给新进程。因此 UE 不会原样持有 ZE 这份进程内 coordinator。不是说 UE 有同一 bug，也不是建议移植 UE 的整个关闭框架。

`System.Promotion.Project Promotion Pass.Step 1 Blank Project Creation.Create Project`（`Engine/Source/Editor/GameProjectGeneration/Private/Tests/GameProjectAutomationTests.cpp:214–249`）要求用户先切换工程才继续下一步，并不自动执行同一个主窗口 A→B→X。Gauntlet `EditorBootTest`（`Engine/Source/Programs/AutomationTool/Gauntlet/Unreal/Automation/UE.BootTest.cs:196–223`）走命令式退出，也不是两次项目 activation 后的原生 X。区别真实原测和源码推导，不声称已运行 UE。

## 5. 修复责任与去重

在实际能接纳 B 的既有通道里，不得复用 A 的终态做 B 的关闭判决；必须证明 A 已完成而 B 是经权威接纳的新 activation。不要在 `Quiescing`、`Committing` 或 `RecoveryRequired` 一概清状态，不要仅因目录相同复用操作号，也不要吞掉 `operation != activation`。[Editor51 明确的进程重启方向](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/optimize/zircon_editor/51/2026-08-31-project-session-transition-authority.md#L134-L144)及[未完成的后续切换任务](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/optimize/zircon_editor/51/2026-08-31-project-session-transition-authority.md#L177-L185)应保留；候选修复不得暗中扩大同进程热切换合同。此处也不寻求把全量进程重启塞进一个小修复。

与 [#221](https://github.com/He-Jiahui/ZirconEngine/pull/221) 不同：#221 是**同一** activation 的已终态 Import 仍被等候；本题没有 Import，而是前后两个 activation 的 owner 不同。[#201](https://github.com/He-Jiahui/ZirconEngine/pull/201)、[#202](https://github.com/He-Jiahui/ZirconEngine/pull/202) 是首次目录/日志而非关闭准入。全状态 PR 标题/正文、开放文件以及相邻项范围内未发现同题。

## 6. 需要怎样的原生证明？

从正式预检项目 A 的冷启动宿主开始。先用真实关闭动作得到 A 的完成与 guard 释放，再由欢迎页正式路由打开 B，核对不同的 activation/根、B 的 Scene/Ready 和无脏项。随后走 root 原生 close 回调和正常 tick：一边记录窗口状态，另一边确认 B 的 Session/heartbeat/ledger 到了哪一步。正控用同一构造的单个 B，或先明确以 File 关闭 B；不能直接填 coordinator 的 operation、直接改 phase、跳过身份保护、手工重建一份 World 或伪造另一个 Session。在真输入到达之前，不把夹具错误或 wall-clock timeout 称为本题 RED。

本机 UE 参考 6.0.0 / UE5 / CL 0，无可核验上游 SHA；参考文件字节 SHA-256：

- `GameProjectAutomationTests.cpp`：`8b72cec4b488d8b07bd5d523bd2a0875188fffe33054a217a7794fd448108f90`
- `UnrealEdMisc.cpp`：`c5b99eeced51018ae594524edd0a11420aca049140d6f7d8b952ede2d8ce5a9f`
- `UE.BootTest.cs`：`8552fae6ab131dba61d16b86a95c2844eb0f0ed4c41bf3a6f259be1dcf9440e7`

## 7. 同一宿主 A→Welcome→B 后，原生 X 确认保留旧 owner

在真实 Ready 项目上走同一 retained root 与 native close callback。对照**通过**：普通冷项目 X 后退出，manager 无 active admission、锁已释放。主例**失败**：File 关闭 A 后 A 的独占 Session 锁不存在，仍是同一窗口；再由已启用的 Welcome Open Existing 打开另一实盘工程 B，核到 B 的独立 operation、活动 Scene/document、无脏项。主窗 X 后的失败不是超时，而是 `The close coordinator no longer owns the approved activation.`：当时主窗不退出、已阻断会话 UI，coordinator 仍保留 A 的 Closed operation，而 manager 中恰是 B；B 的正式记录仍为 Ready。

1 过 / 1 败，0 次 OBJ 导入；没有人工 reset coordinator、替换 World 或故意制造代际失配。结果仅代表真实 retained 回调与项目会话；不是实际 Windows 鼠标/桌面消息或不同进程生命周期验收。下一步只校正已完成的旧 activation 所有权，不能跳过 current identity 比对、清空 Recovery/未完成释放或在 B 上沿用 A 的操作。

Windows x64/MSVC 1.94.1，`zircon_editor --no-default-features --lib --profile test`。实测输入 [同一测试版本](https://github.com/He-Jiahui/ZirconEngine/commit/9583255a8750df136adfea4441f3a91e7ee262cd)，编译回执 `18da72291619df0ce94a579617630a114dd0110f7cab9290b6722a5cbbf7072f`，实际执行 `acad30d2baa51a829719426381d274a5b7b67484cb2f84a47a69274b12fc3fe9`，stdout `054fba0d2036278039475ef99397f3394387d263807081be6f03709ed4fb5a97`。
