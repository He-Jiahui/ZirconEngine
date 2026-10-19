# S77：resolver 测试入口路径修复

当前是单文件 source-ready，六个原用例的新源码原生验收待执行。S74/build194 的固定输入基线已独立核对：PS5.1.26100.9711/Pester3.4.0，实际 -File runner 以绝对 Script 运行当前测试；XML 只有第2行空 Path 的 synthetic script-error，六个原 It 全未执行。35项前后输入相同，两个进程均自然终态/双EOF/Job0。[S74 原始回信](E:/Git/ZirconEngine/.jenkins/records/engine-audit-20261009-01a12097-s74-validation.response.json)。

Sol93 只修改 [入口第2行](E:/Git/ZirconEngine/tools/tests/windows-path-resolver.Tests.ps1:2)，通过当前脚本的 MyInvocation.MyCommand.Path 取目录再沿原相对路径解析仓库。保留 Join-Path/Resolve-Path、生产 resolver 和六例全部断言；无 cwd 或硬编码 fallback。L1、L3–94 逐字节相同，+1/-1、+53 B，仍94行。原版 edc34164cbd3cb06279e279960816792af8b1718da69c737c52dee937c58c01b；新版 ac96c78d6ccc3860dde237803312d8d03df90817e142b7ad865bd817a796ad33。

PS5.1 ParseFile errors0、guard errors0/warnings0、三路独立 Luna 静态复核通过。静态调用链仅支持候选设计，不声称已证明实际自动变量值或 Pester bug。[精确补丁](E:/cargo-targets/zircon-local/reports/engine-audit-20261009-01a12097/s77-resolver-entry.patch)、[静态检查](E:/cargo-targets/zircon-local/reports/engine-audit-20261009-01a12097/s77-resolver-entry.guard.json)、[新执行合同](E:/Git/ZirconEngine/.jenkins/records/engine-audit-20261009-01a12097-s77-resolver-entry-repair-handoff.md)。

Jenkins 应保持 PS5.1/Pester3.4 和 -File 调用形状，以新注册 runner/XML 路径执行六个原名称：

- rejects drive-relative paths before normalizing their per-drive working directory
- hides a verbatim prefix when rejecting a drive-relative path
- resolves an uncreated child through its existing junction ancestor
- normalizes an uncreated dotdot tail only after resolving its junction ancestor
- resolves an uncreated child through a temporary SUBST drive
- uses one physical identity for hard-linked file aliases

须每例实际通过、total/passed6、failed/skipped/pending0，退出码0、source pins不变、TestDrive与本次SUBST清理、native身份/终态/EOF/Job0齐备。旧失败保留，不复跑原输入，不改断言、升级Pester或注入脚本自动变量。运行失败从最低实际原因处理。

S75 独立 cleanup 测试已通过窄范围验收；fixture 生命周期/三消费者、原S50、MVP00/模块整体/产品/性能仍开放，统一队列27项不增不闭。释放0 B；无新 Cargo 构建、源码副本或清理操作。
