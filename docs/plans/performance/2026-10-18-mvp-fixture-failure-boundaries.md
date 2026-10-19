# MVP 夹具消费者失败清理（S71，待受管验收）

任务：`engine-audit-20261009-01a12097`。沿用 [性能计划](index.md)、[待验收账本](pending.md) 和 [夹具迁移合同](2026-10-18-mvp-fixture-migration-contract.md)。本轮属于 MVP00 的现有 S56 问题，统一队列仍为 27 项；没有新增重复发现或关闭模块验收。

初始化取得 root 后的写入失败，原先可能越过 finally；串联清理失败又会跳过后续释放并覆盖主错误。release 测试还用过期 ZirconBuilds 正则拒绝现行 cargo-targets 根。本轮修正这三项消费者子原因。

```mermaid
flowchart LR
    I["Jenkins 夹具发放方：迁移待完成"] --> P["Fixture paths helper：本轮未改"]
    P --> S["snapshot / staging / release：已返回的根"]
    S --> C["共用有序清理函数"]
    C --> R["关闭已取得的租约和 worktree"]
    C --> P
```

| 范围 | 已实现的行为 |
|---|---|
| [snapshot](../../../tools/tests/mvp-acceptance-staging-snapshot.Tests.ps1) | 初始化进入外层 try；marker-race lease 有局部 finally；独立资源分别尝试清理。unlink 与恢复写入保留依赖顺序。 |
| [staging](../../../tools/tests/mvp-staging.Tests.ps1) | 先保存 partial fixture；构造失败和 suite 结束共用清理；worktree remove、prune、root 清理分别执行；passed 移至清理后。 |
| [release](../../../tools/tests/mvp-staging-release.Tests.ps1) | 已返回 root 统一交给生命周期 helper；移除过期正则和物理存在性短路；passed 移至清理后。预算 DTO 的原有字符串含义保留。 |
| [共享清理函数](../../../tools/tests/MvpTestCleanup.psm1) | 按声明顺序继续独立步骤，保存 typed ErrorRecord 和 warning；保留主错误的 bare throw；仅清理失败时使测试失败。 |
| [新增行为回归](../../../tools/tests/mvp-test-cleanup.Tests.ps1) | 保护局部资源/私有 helper 解析、错误保真、继续清理、嵌套诊断追加及成功输出时序；尚未执行。 |

实际 checkout 为 `E:/Git/ZirconEngine`，main HEAD `a80b4c1e23d7d4910782112c2cdfc8025851262b`。准确五文件指纹见 [Jenkins 交接](E:/Git/ZirconEngine/.jenkins/records/engine-audit-20261009-01a12097-s71-fixture-failure-boundaries-handoff.md) 和 [完整报告](E:/cargo-targets/zircon-local/reports/engine-audit-20261009-01a12097/s71-mvp-fixture-failure-boundaries.json)；不能仅凭 HEAD 归因共享 checkout。Sol/max 实现后交回写入窗口，三个 Luna/max 并行独立静态复核全部通过。审查只覆盖报告列出的源码窗口；未读范围、locator、diff 和指导文件分别计数，不表示整模块已读或验收。

已执行的结构检查：Windows PowerShell 5.1.26100.9711 Parser.ParseFile 五文件 0 errors；精确五文件 change guard passed，writeGrantCheck=true，0 errors/0 warnings；diff 与格式检查通过。staging 保持 CRLF、其他四文件保持 LF，均无 BOM；内嵌 Add-Type C# 一块内容未变。详细 [patch](E:/cargo-targets/zircon-local/reports/engine-audit-20261009-01a12097/s71-mvp-fixture-failure-boundaries.patch) 与 [guard 回执](E:/cargo-targets/zircon-local/reports/engine-audit-20261009-01a12097/s71-mvp-fixture-failure-boundaries.guard.json) 已保留。它们均为静态证据。

待 Jenkins 执行 Windows PowerShell 5.1 的新增行为回归，随后在支持的真实 issuer 下执行三套 suite 及失败注入。必须验证构造未返回 fixture 但 root 已取得时的补偿、多重失败、租约关闭、缺失物理根、删除与登记释放分别失败、真实 exit/EOF/清理回执。共享函数测试不能替代真实消费者路径。

完整 issuer/owner/namespace/instance、unknown-owner 删除缺口、退役 acquire/release 退出，以及七 suite 的受支持路由继续保持开放。staging 部分内层 finally 与 release probe Dispose 也保留为后续候选。原 S50 身份及 pending/process_identity_inconclusive 状态未改，本轮没有重查或重提。

本轮没有运行行为测试、Rust 构建、产品或性能采样；没有模块、MVP00 或里程碑验收。原计划的性能工具能力及前后五次独立采样仍待满足产品门槛。实际释放 0 B；五个源码文件、报告、patch、guard 和交接保留，存量及未知归属产物保留，清理未关闭。
