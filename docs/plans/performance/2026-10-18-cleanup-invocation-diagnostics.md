# S73：InvocationInfo 原生失败的最小对照

S71 的 [真实 Jenkins response](E:/Git/ZirconEngine/.jenkins/records/engine-audit-20261009-01a12097-s71-validation.response.json) 已读取：job zircon-management-tests/build180/queue2211，Windows PowerShell5.1.26100.9711，cleanup 回归 exit1；旧 test92 的 InvocationInfo 引用断言失败。原 stderr 指向 Assert helper旧9行，旧92归因由实际 response 和源码顺序共同支持。worker/版本查询/测试进程的保存记录均有 wait、EOF、管道关闭及 Job active=0；这些是历史证据核验。

此前 reply-only 文件检索漏掉 .response.json，关于 S71 尚未执行的历史判断由上述实际失败更正。S72 仍无本轮新 response，旧 S71 执行不覆盖 S72 staging；原 S50 身份及既有 inconclusive 也不由这次 scoped test 替换。

失败只证明两个 InvocationInfo 对象引用不同。旧 evidence 保存的是 mutable ErrorRecord；未保存即时字段，也没有无 helper 对照。普通传播/getter身份变化、helper调用期间变化、真实诊断内容丢失仍是待区分假设，不据此修模块或放宽断言。

Sol89（gpt-6.1-sol/max/fork none）仅修改 [cleanup 回归](E:/Git/ZirconEngine/tools/tests/mvp-test-cleanup.Tests.ps1:1)：显式 -DiagnoseInvocationInfo 进入子作用域，四 case共用同一 typed throw/catch/barethrow/finally 和 outer pipeline 位置；每组新建主错与两个清理错误。

| case | helper 路径 |
|---|---|
| no-helper | 不调用 helper |
| helper-empty | 传空 ordered Steps，绑定失败也保留真实传播 |
| helper-success | 一个成功步骤 |
| helper-failures | 两个失败步骤后继续成功步骤 |

inner catch、cleanup前后、outer catch即时保存 primitive 字段和重复getter引用比较；外catch后另观察保留的原ErrorRecord及InvocationInfo，输出六组快照、五组字段变化名。快照包含错误类型/消息/身份/类别、源路径/行列/文本/位置、InvocationName、MyCommand名称/类型、ScriptStackTrace。JSON只含固定schema primitive/数组，不序列化错误/命令对象、BoundParameters或Definition。字段名依据 [Microsoft InvocationInfo API](https://learn.microsoft.com/en-us/dotnet/api/system.management.automation.invocationinfo?view=powershellsdk-7.6.0)；该文档不证明PS5.1的实际传播语义。

源码前 pin 5b44ba09074429647bbee487a1c8b3a324b0ee8bff78564b00fc7dbea70bc743/8081B157L；后 pin 99fd0282410f00ac20eb6481e5661703cd6cd0c4f2ab83b9009808d9481df3d0/19722B340L/1792321667314862600。精确 +183/-0L，LF/无BOM；移除参数头与诊断块后原157行逐字节复原。原92/93断言保持于 [275行](E:/Git/ZirconEngine/tools/tests/mvp-test-cleanup.Tests.ps1:275)/276行，诊断输出后继续全部原回归。[helper](E:/Git/ZirconEngine/tools/tests/MvpTestCleanup.psm1:1) b4f0ae0d461bf45a41575244e8cfe1b4981d321977722a946e90a4eba00b237b 未改。

三路独立并行 Luna/max/fork none 后审均静态 PASS：调用形态与隔离、immutable snapshot及序列化、原文delta与合同。Root实际 PS5.1 parser（7539b1）0错误；单文件 [guard](E:/cargo-targets/zircon-local/reports/engine-audit-20261009-01a12097/s73-cleanup-invocation-diagnostics.guard.json)（5e9183）0错误0警告，真实write grant检查开启。实际 main/HEAD仍为 a80b4c1e23d7d4910782112c2cdfc8025851262b；这些静态检查不替代行为执行。

[完整证据](E:/cargo-targets/zircon-local/reports/engine-audit-20261009-01a12097/s73-cleanup-invocation-diagnostics.json) 与 [单文件增量](E:/cargo-targets/zircon-local/reports/engine-audit-20261009-01a12097/s73-cleanup-invocation-diagnostics.patch) 已登记；delta SHA c470c32794c852efb588ffe785b7e0e01648407c3158a93e0ef289d6c14ce3d7/12298B197L，依赖既有S71父patch和S72当前staging。已按用户授权保存 [Jenkins交接](E:/Git/ZirconEngine/.jenkins/records/engine-audit-20261009-01a12097-s73-cleanup-invocation-diagnostics-handoff.md)，要求当前五源pin、四组JSON、随后原回归真实exit/EOF。S73仅本地留信；尚无收件/ACK/执行证明，诊断尚未运行或修复根因。

另一个 [S69/S70 实际失败](E:/Git/ZirconEngine/.jenkins/records/engine-audit-20261009-01a12097-s69-s70-validation.response.json) 是 Pester3.4.0入口 line2 Join-Path Path null，六条原路径断言未运行；本轮未混入其修复。三 consumer suites、S72失败注入、真实 issuer/owner/namespace/register/delete/release迁移、原S50/MVP00/产品及性能继续开放。

统一队列27项，只更新既有S56第25项，无重复新增、无关闭、无模块进入已验收账本。Root本轮测试/原生query/提交均0。没有源码副本、临时helper或编译产物；实际释放0 B，登记证据保留，共享/未知/其他会话产物未清理，清理未关闭。
