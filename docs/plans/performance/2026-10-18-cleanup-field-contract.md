# S75：以冻结字段验证 cleanup 错误保真

当前 S75 独立 cleanup 测试已通过窄范围原生验收：普通与 schema2 诊断完整回归均成功。模块整体、消费者、MVP 和性能仍保持开放。

S73 的真实四对照揭示了测试契约问题：无 helper 时 inner/outer InvocationInfo 也不是同一引用，而所记录字段一致。三路 Luna 已独立核对 raw marker、归档诊断、response 投影、源文件前后身份及 native 终态；五个历史 native process 的真实退出码是 1/0/1/0/1，最后一个是实际诊断命令。原回归确实失败，不能把四对照 JSON 当作完整测试通过。[S73 回信](E:/Git/ZirconEngine/.jenkins/records/engine-audit-20261009-01a12097-s73-validation.response.json)。

修复集中在 [cleanup 测试](E:/Git/ZirconEngine/tools/tests/mvp-test-cleanup.Tests.ps1:14)：将已有 16 字段快照与变化比较提升为共享测试函数，保留 null、类型与大小写，在 inner catch、finally 清理之前冻结原始字段和 Exception 引用。正式断言比较冻结快照，继续保护 Target/Exception 身份、错误 id、类别、stack；原来的清理步骤、typed errors、warning、cleanup-only、nested 累积和成功输出隔离保持。

原始错误传播可能创建不同 ErrorRecord/InvocationInfo 对象；本次只取消已被 no-helper 反例否定的跨传播引用要求。没有把“引用必须不同”写成新契约，也没有证明所有 PowerShell 字段或所有命令边界保真。当前 16 字段和本次实际路径是结论范围；生产 helper 保持原内容。

四组可选诊断保留六阶段快照、五组变化及引用观测，先输出 JSON 便于失败定位，再验证字段保真、分组路径、错误与输出行为。SchemaVersion 从 1 升至 2，明确不再把 nullable 字段归一为空串；Root 与 Jenkins 的本次 verifier 同步消费 schema2，旧 schema1 证据保留。Jenkins 还需逐项核对 events/warnings/failuresteps 原始数组与数量，补足脚本 join 断言的数量精度。

本次精确变化为 +95/-42，340→393 行；源码 SHA-256 `4dc32b71a6e5ef93b11784edbf9878f94a209cbde5c2f37989bc6286c647ae95`。[精确补丁](E:/cargo-targets/zircon-local/reports/engine-audit-20261009-01a12097/s75-cleanup-field-contract.patch)、[静态回执](E:/cargo-targets/zircon-local/reports/engine-audit-20261009-01a12097/s75-cleanup-field-contract.guard.json)绑定该版本。PS5.1 ParseFile、单文件 guard 和三路独立静态复核均通过；这些检查没有运行目标脚本。

[审查报告](E:/cargo-targets/zircon-local/reports/engine-audit-20261009-01a12097/s75-cleanup-field-contract.json)记录准确阅读覆盖、原始证据等级、前后指纹及保留范围。[Jenkins 交接](E:/Git/ZirconEngine/.jenkins/records/engine-audit-20261009-01a12097-s75-cleanup-field-contract-handoff.md)固定五个源码输入，要求普通/诊断两次完整回归各自 exit0、最终通过标记和 native wait/readers/EOF/Job0。

S74 当前输入基线已执行并在测试第2行失败，六例尚未执行；S77 另开单文件入口修复，新源码待重测。S72 消费者、真实 fixture 生命周期、原 S50、MVP00/产品/性能验收保持开放，统一队列仍为27项。

释放 0 B；仅交付有归属的精确 patch、报告、静态证据与交接文档，历史、共享、未知归属及其他会话修改保留，cleanup 未关闭。所有 Root 静态工具进程已完成，新的 Jenkins 执行状态以独立回信为准。

2026-10-18 原生验收：Jenkins build196、PS5.1.26100.9711，两次完整命令均 exit0、各1个最终成功标记、stderr空；独立核对4份raw、schema2四组/24快照/16字段/20比较、精确events/warnings/failuresteps及3个PID/birth/终态/双EOF/Job0。当前五源pins一致。仅接受独立cleanup测试，不把此结果泛化为consumer或fixture通过。[实际回信](E:/Git/ZirconEngine/.jenkins/records/engine-audit-20261009-01a12097-s75-validation.response.json)、[当前归档](E:/cargo-targets/zircon-local/reports/engine-audit-20261009-01a12097/s76-s77-native-reconciliation-and-resolver-entry.json)。
