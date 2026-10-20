# S72：staging 内层清理与重复实现精简（待动态验收）

任务：`engine-audit-20261009-01a12097`；沿用 [性能计划](E:/Git/ZirconEngine/docs/plans/performance/index.md) 与 [S67 迁移契约](E:/Git/ZirconEngine/docs/plans/performance/2026-10-18-mvp-fixture-migration-contract.md)。最低 MVP00 门槛继续开放。

本切片只修改 [staging suite](E:/Git/ZirconEngine/tools/tests/mvp-staging.Tests.ps1)，承接 [S71 错误与清理契约](E:/Git/ZirconEngine/docs/plans/performance/2026-10-18-mvp-fixture-failure-boundaries.md)。条件静态证据显示：CIM 查询、PID 文件读取或 manifest 恢复在 finally 内失败时，可能替换正在传播的主错误，并阻断后续独立清理。owner 是测试消费者；未声称产品路径已发生这些错误。

调用链为 suite typed catch + bare throw → [Invoke-MvpTestCleanup](E:/Git/ZirconEngine/tools/tests/MvpTestCleanup.psm1) → ordered callbacks。七个边界只保存真正逃逸的 ErrorRecord；已消费的负例错误继续由原 catch 处理。PID 读取、转换和 Stop/Wait 保留在一步，已有 CIM 路径兜底独立在后。equal-content 的 fixture 属性恢复与依赖 manifest 重建保持同一步，临时 DLL 删除在独立后步；恢复失败仍需报告，不能声称两者一致。

五处相同 CIM 路径兜底合并为 suite 私有 `Stop-MvpStagingFixtureProcesses`，保留 GetFullPath/TrimEnd、ExecutablePath 前缀筛选、Stop-all → Wait-all、Timeout5 与 SilentlyContinue。该策略未证明进程身份、owner、实际终止、exit 或 EOF；本切片未改变其完成判定。Clear 的静默删除策略保留。独立审查未发现 release SafeFileHandle 固定探针的 actionable 缺陷，因此未改该处。

Sol/max 完成核心设计与精确补丁准备，交回 owner 后由 Root 使用标准 apply_patch 落地；三个 Luna/max 并行前审与另外三个并行后审已完成。后审限定范围均静态 PASS；PowerShell 5.1.26100.9711 ParseFile 为 0 error，精确单文件 guard 为 0 error/0 warning，writeGrantCheck=true、contributor=false。语法和静态检查不替代真实行为回归。

当前 main HEAD `a80b4c1e23d7d4910782112c2cdfc8025851262b`。staging 四字段：SHA256 `b68ffd42d9d911b16728e292d1a74ca7d49dbff9a7324f52ca89a2dcb0b328f1`，181377 B / 2386 L，mtime_ns `1792319552583451700`；原有 CRLF、无 BOM 和嵌入 C# 均保持。精确增量补丁 SHA256 `611c057b695893ba43b82fa33064e246778de4b70089c10e9f258f4b133bb45a`，98 行新增、89 行删除、净增 9 行；不把源码去重当作性能测量收益。before 由 HEAD 与 S71 五文件父补丁在内存重建，校验 SHA/bytes/lines；mtime 仅是历史实测值，没有创建源码副本。

Jenkins 下一步在实际 Windows PowerShell5.1 执行 [既有 cleanup 行为回归](E:/Git/ZirconEngine/tools/tests/mvp-test-cleanup.Tests.ps1)。真实 suite 需支持的 issuer 路径，并覆盖 E+cleanup R 保留原错、损坏/不可读 PID 文件后仍尝试 CIM、equal-content manifest 恢复失败仍尝试临时 DLL 删除、已消费预期错误后出现恢复失败必须失败。记录实际 exit、EOF、warning/error 与每项清理结果；当前这些行为运行数为 0，无本切片 Rust 编译需求。

[完整证据](E:/cargo-targets/zircon-local/reports/engine-audit-20261009-01a12097/s72-staging-inner-cleanup.json)、[增量补丁](E:/cargo-targets/zircon-local/reports/engine-audit-20261009-01a12097/s72-staging-inner-cleanup.patch)、[实际 guard](E:/cargo-targets/zircon-local/reports/engine-audit-20261009-01a12097/s72-staging-inner-cleanup.guard.json) 与 [Jenkins 交接](E:/Git/ZirconEngine/.jenkins/records/engine-audit-20261009-01a12097-s72-staging-inner-cleanup-handoff.md) 已保存。S72 补丁依赖 S71 父补丁；共享 main 已含两者，不盲目重放覆盖其他修改。records 文件表示本地留信，尚无指定 Jenkins 会话读取、ACK、排队或执行证据。

沿用原 S50 request/operation `engine-audit-20261017-mvp00-runtime-default-s50-r1` 和 runtime `start-1792141433756007100`。本轮未重查、重提或更换 identity。统一队列仍为 27 项，仅更新既有 S56，关闭 0 项；issuer/namespace/register-before-write/同实例物理删除后 release、MVP00、模块/里程碑/产品/性能验收均开放。

本轮释放 0 B；只保留已登记的报告、补丁、guard、交接和计划记录，没有创建临时脚本、源码副本或构建树。共享/未知/其他会话产物继续保留，整体 cleanup 未关闭。
