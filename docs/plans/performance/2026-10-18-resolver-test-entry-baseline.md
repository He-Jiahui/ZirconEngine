# S74：resolver 测试入口的当前基线

当前状态为待执行的原生基线，目标测试源码保持不变；没有新增模块、MVP 里程碑或性能验收。

S69 R1/R2 的真实 PS5.1/Pester 3.4.0 记录只有一个脚本加载失败节点，原六例没有进入。错误发生在 [测试第 2 行](E:/Git/ZirconEngine/tools/tests/windows-path-resolver.Tests.ps1:2)，早于模块导入和 Describe。旧回执只固定了另 11 个源码输入，未固定目标 test/runner 的当时身份；本轮当前 pin 不能补全历史身份。

三路并行 Luna 分别核对入口 owner、历史 native/原始输出、Pester 文件调用链。Pester 已读路径将绝对文件名保留到 `& $Path`，没有证明它把目标脚本重建成无路径文本。四组 S74 输入的前后指纹稳定；生产 resolver 主体仍未在本轮完整阅读。各路具体读取范围及真实工具终态保存在 [S74 报告](E:/cargo-targets/zircon-local/reports/engine-audit-20261009-01a12097/s74-resolver-test-entry-baseline.json)。

PowerShell 规范说明 `$PSScriptRoot` 与 `$MyInvocation.MyCommand.Path` 的当前脚本含义；规范不能代替失败作用域的实际值。不能把现有失败归因为 PS5.1 不支持该变量，或直接认定为 Pester 缺陷。[about_Scripts](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_scripts?view=powershell-5.1)、[自动变量](https://learn.microsoft.com/en-us/powershell/module/Microsoft.PowerShell.Core/about/about_automatic_variables?view=powershell-5.1)。

Sol90 与 Root 采用 baseline-first：关闭源码窗口，要求 Jenkins 登记并生成独立八行 runner，保留原 `powershell.exe -File` 形态、Pester 选择/导入/调用参数、全部六例；仅改变 owned 输出路径和 fixtureOwner。新旧 runner、目标 test、resolver、实际 Pester 与实际加载依赖均前后固定。[具体交接](E:/Git/ZirconEngine/.jenkins/records/engine-audit-20261009-01a12097-s74-resolver-test-entry-baseline-handoff.md)列出准确 argv、输入指纹、六例名称与 native/EOF/清理门槛。

当前六例全通过且 native exit=0 时无需源码修复；同一入口失败得到当前输入归因后，才评估最小候选。静态候选、历史 XML、记录落盘均不构成当前通过。

本轮补齐两项回信状态：[S72 monitor](E:/Git/ZirconEngine/.jenkins/records/engine-audit-20261009-01a12097-s72-validation.monitor.response.json)是已读取的依赖 ACK，复用 S71 失败并明确 S72 新命令提交为 0；[S73 实测回信](E:/Git/ZirconEngine/.jenkins/records/engine-audit-20261009-01a12097-s73-validation.response.json)记录四对照已执行、原回归仍失败。无 helper 的对照也有内外 InvocationInfo 引用变化，而所采 primitive 字段一致，尚需独立核对原始诊断。它们修正此前“尚未见回信”的观察状态，不改写历史记录或提升验收。

本轮未改测试/生产源码，Root 未运行目标测试、构建、Jenkins 客户端或原 S50 查询；仅交接待执行基线。既有统一问题队列仍为 27 项，更新现有 fixture 验证项，不新增重复发现、不关闭问题。释放 0 B；历史失败证据、共享产物、未知归属与其他会话修改均保留，cleanup 仍未关闭。
