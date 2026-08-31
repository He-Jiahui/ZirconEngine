---
related_code:
  - zircon_editor/src/core/script_build
  - zircon_runtime_interface/src/script_diagnostics
base_reports:
  - docs/plans/performance/01/2026-07-30-editor-core-script-build-current-review.md
  - docs/plans/performance/01/2026-08-16-editor-core-script-build-generation-current-architecture-review.md
owner_plans:
  - docs/plans/zircon_editor/editor/04-pie-and-simulation.md
  - docs/plans/zircon_editor/editor/13-script-compilation-management.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Developer/Windows/LiveCoding/Public/ILiveCodingModule.h
  - dev/UnrealEngine/Engine/Source/Developer/Windows/LiveCoding/Private/LiveCodingModule.cpp
  - dev/UnrealEngine/Engine/Source/Developer/HotReload/Private/HotReload.cpp
tests:
  - tools.tests.test_editor13_script_build_orchestrator_contract
doc_type: currentness-revalidation
status: static_current_revalidated_dynamic_and_product_integration_pending_structural_cutover_required
---

# Editor core script-build currentness重验（2026-08-23）

## 当前冻结与产品可达性

| scope | Rust文件 | physical lines | bytes | tests | ordered path + NUL + raw bytes + NUL SHA256 |
|---|---:|---:|---:|---:|---|
| `zircon_editor/src/core/script_build/**` | 5/5 | 1,589 | 52,563 | 26 | `b4993c5e480afb0312d90ce7803b6f37acbd5648a6a7c2233277ec9a07e8cdda` |

5/5文件和26个测试已完整复读。相对2026-08-16冻结（1,582行），生产字节不变；最近7行漂移
只是`tests.rs`为满足Rust借用规则先保存dispatch再调用`complete`。精确搜索确认模块外生产caller仍为0：
没有watch、command、Play、EditorJob、VM/compiler或diagnostics-sink接线。因此这是MVP接线前P0容量与
正确性门，不是当前产品帧热点，也不能产生WPR/RenderDoc优化结论。

## 已正确关闭的旧问题

- watch first-event max latency为1,000 ms，不能再被持续事件无限滑动；
- incremental paths同时受20项和64 KiB约束，超限折叠为full-rebuild sentinel；
- active加最多one queued request替代旧无界FIFO；1M Command/Play风暴保持单generation；
- `Watch < Command < Play`只提升priority/play intent，不追加相同工作；
- completion按线性request/step dispatch校验，last outcome由`Arc`共享；
- diagnostics cursor按generation/request/step拒绝replay与较旧completion。

Performance01当前`PERF-MVP-557`仍把“无限滑动deadline”和“无界VecDeque”列为现状，已与源码相反，
必须由计划owner纠正，不能据此重复实现第二套queue。

## 当前结构瓶颈

### P0：request id错误承担四种generation

`ScriptBuildRequest::new`直接用request id构造`ScriptBuildGeneration`。系统没有独立的source revision、
artifact identity、ledger validation receipt或runtime binding generation，无法证明某个artifact来自哪份
源码、refresh绑定应用了哪份artifact、Play究竟等待哪个runtime session中的binding。

目标链必须是：

`ScriptSourceGeneration -> ScriptBuildIntent/Ticket -> ScriptArtifactGeneration ->
LedgerValidationReceipt -> ScriptBindingGeneration(runtime session)`。

同source的Command/Play只合并observer与latest Play waiter；request id仅是observer/ticket identity。

### P0：失败/取消删除更晚源码事实

任一非success completion会take active、take queued并clear全部pending watch状态；现有测试明确要求这一
行为。generation N编译失败可因此丢失N+1编辑，让旧artifact无限驻留。失败/取消只能终止精确active
ticket；latest pending source generation必须保留并按policy继续，shutdown使用显式fence/terminal receipt。

### P0：bounded log retention不等于bounded ingress

`ScriptBuildDiagnosticsSink::project`对每条diagnostic执行format、jump path ownership和一次同步
`EditorLogService::emit`。现有storm测试恰好证明256条全部emit后只保留8条；若编译器返回1M错误，仍可
执行1M次日志sink/file/fanout工作。Runtime13需提供count+bytes+deadline bounded pages；Editor17/
`PERF-MVP-644`批量admit并返回continuation/truncation receipt，cursor只跨成功admit的page前进。

固定三步`Vec`和dispatch最多20个PathBuf clone是P1，产品caller为0且generation合同尚错，本轮不做
局部allocation优化。

## Unreal源码依据

- `ILiveCodingModule.h:20-29`区分Success、NoChanges、InProgress、CompileStillActive、Failure与
  Cancelled，证明compile request不能压缩成一个无差别完成态。
- `LiveCodingModule.cpp:699-724`拒绝重叠compile，并在异步模式立即返回InProgress；`808-924`只在
  patch sync boundary完成load/reinstance/GC后广播patch completion。
- `HotReload.cpp:945-1005`拒绝第二次compile，构造changed-module集合，异步compile成功后把精确
  changed modules传入独立reload阶段。

Zircon应转移single-flight、changed-module identity和compile/apply分阶段合同，不复制Unreal的全局
object scan/GC；VM应发布immutable artifact/binding generations并在budgeted safe point应用。

## 依赖有序计划与验收

1. Editor13定义source intent与ticket分离，失败/取消保留latest pending source；保留现有admission界。
2. Runtime13返回content-addressed artifact和ledger digest；validation/binding按source/artifact/session
   拒绝stale completion。
3. Editor14/Runtime11用唯一shared job/process/I/O authority执行，绑定entries/bytes/age/deadline、cancel
   和`script_artifacts` exclusion；禁止私有thread pool/watcher/output reader。
4. Editor04把每个Play waiter绑定required source，只有匹配binding在active runtime session被接受才恢复。
5. Editor17按bounded diagnostic page批量ingress；接线后执行save storm/manual build/build-before-Play/
   failure/cancel/session replace的31-run F4 WPR CPU/wait/I/O/RSS/power矩阵。

| gate | acceptance |
|---|---|
| admission 1/20/21/10K paths、64B/64KiB/1MiB、1M triggers | active+pending intents <=2；first-event latency与path bytes有界；same source compile <=1 |
| changes during compile/validate/bind，success/failure/cancel/supersede | newer source fact loss=0；stale artifact/binding/diagnostic/Play apply=0；每个receipt携带精确identity |
| diagnostics 0/1/1K/1M rows、64B/8KiB、consumer stall 0/60s | count+bytes+deadline pages；producer per-row file/fanout=0；RSS/I/O有界；一个truncation receipt |
| product integration | UI compile/process/I/O wall=0；只用shared job authority；稳定editor frame script-build work=0；F4分布门通过 |

## 本轮静态门

- `python -m unittest tools.tests.test_editor13_script_build_orchestrator_contract`：8/8通过。
- `rustfmt --edition 2021 --check`：5/5通过；scoped `git diff --check`通过。
- 未修改production。未运行Rust/Cargo、VM/job/Play/commandlet integration、WPR或功耗；模块继续pending，
  无里程碑commit或企微通知。

