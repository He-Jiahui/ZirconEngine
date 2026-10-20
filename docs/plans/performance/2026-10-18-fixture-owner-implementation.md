# S76：以实际 Jenkins runner 承载 fixture 生命周期

当前是已授权的实现合同，fixture helper 尚未迁移、七套 suite 尚未验收。继续使用现有问题 `S56-MVP-FIXTURE-RETIRED-DEPENDENCY`，队列仍为 27 项。

三路 Luna 与 Sol92 核对了 [helper](E:/Git/ZirconEngine/tools/mvp/MvpTestFixturePaths.psm1:142) 的未知记录删除分支：[测试](E:/Git/ZirconEngine/tools/tests/mvp-test-fixture-paths.Tests.ps1:74) 先 Remove/release，再于第 78 行重建同路径新实例，却要求第 87 行再次删除。路径和 PID 相同不传递实例归属。合法跨 scope 清理应保留；`-Force` 可能重置模块缓存，但本次没有证明某次 New→Remove 之间实际发生重载。

实际链路已固定：S73 generated worker → sealed delivery.environment → driver_environment → local_cargo._run_owned_command。已读环境构造链没有专用 fixture 上下文；[Jenkins gap 回执](E:/Git/ZirconEngine/.jenkins/records/engine-audit-20261009-01a12097-s75-validation.response.json)亦确认当前入口未建立。通用 artifacts.json 是归属记录，不能单独签发删除权。缺口应通过实施消除，不再把“等待现成 issuer”当作用户授权限制；不恢复退役 coordinator。

最小实现采用下一份真实 suite 命令的受控 PowerShell wrapper，在 helper 模块之外持有唯一存活 owner。绑定 task/request/attempt、driver/component digest、command/invocation、实际 PID/birth、suite、批准物理 root/namespace。保留 New(Prefix)→DisplayPath string、Remove(Path)，不把上下文扩散给业务消费者，也不新增远程服务。先沿用默认 E:/cargo-targets 下现有 mvp-test-fixtures namespace；runroot/temp 不满足当前 first-leaf 策略，不能静默切换。

- New：登记 Reserved 先于写入，排他创建新 leaf，已有 leaf 拒绝；取得真实目录身份和不跟随 reparse 的句柄后转 Active。
- Remove：先核对存活 owner 与同实例，再删除、确认物理完成、登记终态。失败保留 CleanupFailed 和可重试证据。
- 重导入：模块表仅缓存，通过仍存活的 runner owner 恢复同实例。终态记录只允许“仍不存在”的幂等返回；同路径后来创建的新实例拒绝并保留。
- KEEP：记录 Retained 和保留条件，wrapper 不补删。上下文丢失、未知或替换实例均保留目录并报告。
- 复用 resolver 已有目录句柄及 handle-relative 删除，不复制底层实现。长期句柄与真实消费者兼容性仍需原生验证。

源写集先核当前代码，不机械重做 S67 的旧 13 文件清单。S69 已实施的 resolver 导入修复保持；重点是 helper、fixture 合同测试的旧正例、wrapper/context 和实际 command producer，并核 acceptance KEEP、release 根 guard、其余实际消费者。Root 统一接口窗口；Jenkins 负责 command 生成、受管执行及清理。[实现协调信](E:/Git/ZirconEngine/.jenkins/records/engine-audit-20261009-01a12097-s76-fixture-owner-implementation-handoff.md)。

验收覆盖无上下文/错误 owner/调用/root 的写入前拒绝、未知与同路径新实例保留、合法同实例重导入恢复、创建/删除/终态登记失败、KEEP、兄弟隔离、OperationalPath、root/parent junction 拒绝、子 junction 目标保留及七套真实 suite。结论绑定源码和 native PID/birth、退出码、wait/readers/双 EOF/Job0。静态审查不替代执行。[本轮证据](E:/cargo-targets/zircon-local/reports/engine-audit-20261009-01a12097/s76-s77-native-reconciliation-and-resolver-entry.json)。释放 0 B，cleanup 保持开放。
