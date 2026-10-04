---
doc_type: feature-completion
status: source_applied_managed_validation_pending
validation_status: no_cargo_or_tests_executed
performance_status: real_release_samples_and_budgets_pending
date: 2026-10-01
plan_sources:
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
---

# Runtime1071 / Runtime02 · Task 与 Event admission 完成列表

本轮 EventBus 修复的源码已按完整 25 路径契约应用；应用回执为 [本批源码应用回执](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime-two-contracts-apply-v1/30path_guarded_source_and_record_apply.json)；测试和性能未执行。

- [x] 准备并独立静态复核 25 个 EventBus / App / 文档路径，完成 count / bytes、所有或无 fan-out、owned rejection、Reliable 契约与接收克隆最后释放计费。
- [x] 保留真实 close / unsubscribe / poison / replacement 回归和 Release raw publish / receive、retained lease、可用时 RSS 采样用例。
- [x] 核对本批源码应用、live lease、来源归属并保留当前源码 preimage。
- [ ] 封存完整 current 验证闭包并通过 managed 准入。
- [ ] 在一次派发阶段提交兼容 Check、正常回归和统一 Release profiles，修复实际失败。
- [ ] 测量 diagnostics on / off、fan-out、contention、paused consumer 的延迟、吞吐、allocation 和真实 RSS；建立缺失基线和回归预算。
- [ ] 合成并验证 detached / tracked scope、raw pool / helpers、非 inline observer 和全部 shutdown owner 调用闭环。

[EventBus 源码清单](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-eventbus-admission-v2/prepared.json) SHA `defabbe017b02f3a6c9e49115c8c90260fdbd3621e2040c2f66e373688d37249`；[独立静态审查](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-eventbus-v2-independent-review-v1/review.json) SHA `f4a4539ab1d5d85a907c0b4e362d895ae773d86253904a98a6523ab9c4fb44ee`，无确认的必修问题。[优化记录](../../../optimize/zircon_runtime/02/2026-10-01-task-event-owner-admission.md)。

detached 27 源和 tracked 扩展候选属于独立未应用依赖；固定数字 `wait_all` 输出不计为性能。声明 retained weight、冻结 payload 字节与 process RSS 分别验收。RT02-P1-6、DLL unload、全部 worker census、MVP F0–F5 及产品性能继续开放。

应用与归属检查以本批回执的实际终态和文件 SHA 为准；若回执为 pending 或 failed，相应源码步骤保持开放。正常测试、Release 和性能预算须由后续 managed 批次独立验收。

## 2026-10-01 · 真实 detached profile 因果断言修复

原 profile 只断言任意 Err，可能把 runtime stopped 等错误标为 count/weight 配额命中。v2 在输出样本前检查实际 `TaskGraphAdmissionError` variant、scope owner、count capacity=1，或 weight requested > remaining=1。[profile_v2](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-detached-profile-v2/manifest.json) SHA `f1d98721d97b968cc5ed156ee3d721c15a543af8c937a7dc9d7ef86e03dc0c0b`；[profile_v2_review](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-detached-profile-independent-review-v1/review-v2.json) SHA `622ea1f03ac2a0c4a5ad88a3821d515621de486c876da6f9d05474bbaf77590b`，该具体必修问题已由独立静态增量审查关闭。

- [x] 修复上述 quota 因果断言，保留原冻结版本及真实 101 rounds、Task ID、typed terminal、worker census 和最终 clone lease 证据。
- [ ] 将三新模块和声明合入最终 tracked V2 owner；旧 v1 scope 父源码不得覆盖新支持层。
- [ ] 修复 scene I/O scope 默认 16MiB 与 lane 512MiB weight 配置不一致，以及拒绝后 held admission 的 late observer inline 路径。
- [ ] 合成 graphics/ECS/UI outer caller，移除遗留 `TaskPools::process_default` 初始化路径并保持真实同 graph owner。
- [ ] 在统一 managed 批次运行 Check/普通回归/隔离 Release，基于实际结果修复并取得性能验收。

这些 task/profile 候选仍未应用或未执行；已有 EventBus 25 路径应用事实维持独立。declared retained weight、encoded payload bytes、process RSS 与 allocation 分别核验；缺失数字预算和真实采样不计为达标。


## 2026-10-01 · EventBus contention 与底层 owner 支持修复

[EventBus contention V2](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-eventbus-contention-profile-v2/prepared.json) SHA `386485d965149336f4bac979055b5ac1b2c9a3ec6dfb1ddd654a59d3852870cb` 已修复原用例只核对长度和序号的缺口：每个正常 delivery 核对预期 topic、该 topic 允许的 producer、完整原始 Value；held-clone 首次接收及重试也核对完整事件。[V2 独立增量审查](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-eventbus-contention-independent-review-v1/review-v2.json) SHA `a37d97eeb00e3e5a5f7fe9bfb64a06f5020ccec3c9117300019a53f23ada3265` 已在该冻结候选中关闭唯一 P1，旧 V1 不变；源码准入和实际执行仍待完成。

- [x] 完成上述完整事件因果断言的私有源码修复与独立增量复核。
- [x] 在私有 graphics fix1 删除未初始化且未使用的 text_scheduler 字段，保留 scheduler 转入 TextRenderState 的真实路径；[graphics 必填字段修复独审](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-graphics-ecs-v2-independent-review-v1/review-graphics-fix1.json) SHA `382ec6d2b7dc920e13726bae0ebdbf785fdfa9edddebc2ae96449eb26e8f41f5` 关闭该原始具体缺陷。
- [ ] 完成 18 文件 TaskGraph/TaskHandle/TaskNode/Scheduler/callback dispatcher 支持检查点的独审、所有真实调用迁移和最终合成。[18 文件调度支持检查点](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-tracked-admission-v2-support-checkpoint-v1/manifest.json) SHA `01a1dff2e6fed2b453d619bf965f7307b7305d4b972f0e5623967076b7fa7d0b` 只覆盖支持层，不计完整产品闭包。
- [ ] 对 scene I/O 20 文件预算与晚注册 observer 修复做独立复核：预算由 lane bytes、有限 entry/observer、节点元数据和 IO 并行度 checked 推导；observer 返回 typed Result，保留 exact F，通知进 owner queue；共享应用和实际保存回归仍未完成。
- [ ] 完成 UI/Editor 全部真实调用的 typed failure 传播和 same-Core driver 绑定；同 graph worker inventory 的 TextRenderState 修复仍在可变 UI 候选，不能计入本次冻结接受。
- [ ] 完整准入后合并 Check、正常回归及 Release profiles 批次，取得真实原始样本和数字门通过。

contention 用例覆盖 same/different topic、fan-out、payload、diagnostics on/off/sampled 和 clone 压力。不同 topic 的消费者数量变化不能单独归因于 commit mutex；用例 wall time 含断言与完成工作，不是生产路径纯 publish 成本。allocation、RSS peak、数值基线及产品预算仍开放。当前只有原 EventBus 25 路径及其既有回执确认应用；这些新增 profile/task 候选不以静态复核声称测试或性能通过。

## 2026-10-01 支持层增量修复记录（编译与性能待验收）

- Core V3 的 25 源文件切片已完成五文件独立增量审查：`run_context` 使用 `TaskAdmissionStorage::Context`，四个 shutdown 回归传入真实截止时长，timer fixture 使用实际 domain worker inventory。原 timer/dispatcher continuation 预约修复复用未变源码证据；新增 exact/one-byte/after retained-charge 与 completion 拒绝回归未执行。
- Scene V4 的实际 context wrapper 预算缺陷已在源码层闭合；五项 support 依赖通过新桥接记录绑定 Core V3，其中只有 scope 一行改变。整批 composition/typecheck、quota/observer 行为与真实样本仍开放。
- UI231 独审发现的 IME retained-request 缺陷已有四文件私有修复：刷新请求先经过现有 text-service owner/surrounding-text 校验并写入 Surface，再更新 host/reply/applied copies；失败保留此前已应用请求并独立记录 rejected refresh。两个真实排版/软换行回归增加完整请求相等断言，先写回归但没有观察到 red/green 执行。另三文件修复 14 个实际 UiSurface 参数遗漏；此时增量独审与最终 2021 格式组合仍开放。
- 当前已落共享源码仍仅以先前 30-path apply 回执为准；上述新 Core/Scene/UI cohort 均为私有候选，没有据此宣称共享修复已生效。
- [ ] 同批托管检查及正常测试通过。
- [ ] 实际 Release 样本、分配/RSS、任务 quota 与数值性能预算达标。

## 2026-10-01 最新独审与当前验收边界

- UI231 的两个具体 P1（IME final request 未留在 Surface、14 个实际调用遗漏 execution 参数）已由七目标增量独审逐一确认源码修复；alias 后继 2021 格式桥也核对。整批 UI 阈值、执行、类型与 native 仍待验。
- 新动态 UI V2 让实际 Session wrapper 的 metadata/sequence 创建发生在同一次真实 UI 接纳之后，准入失败不再先递增序号。Settings V2 将原始 observer 闭包实测 storage 传给实际 typed slot，8192 字节拒绝回归指向真实预算检查；两项均已独审静态闭合、未执行。
- Graphics fixture 11 源已独审同 Core 构造与引用寿命；Asset、Scene viewport V2 与 retained B68 的新候选也保持真实 Core owner。Scene 首次 Stale marker 与唯一借用 reset prepare/commit 静态修复已核；多窗口统一 native publication、外部 pointer callers 和关闭 producer 仍在整合。
- 上述增量 cohort 仍是私有候选；本次只写优化/完成记录，不写新的 Runtime 或 Editor Rust 实现。实际共享 Rust 改动仍以已记录的合法 source apply 回执及当前源码哈希为准。
- [ ] 完整兼容托管批次的检查与正常测试通过。
- [ ] 实际 Release/分配/RSS/原生产品样本满足原计划数值性能门槛。

## 2026-10-01 补充：宿主操作寿命与实际 Session 清理顺序

- [x] 更新实际写入事实：EventBus contention 的父模块和 admission_contention 两源已整体写入并确认归属，见 [六路径归属终态](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-editor-profile-attribution-reconciliation-v11/fresh_source_attribution_after_exact_previous_request_not_found.json) SHA `cb096b43f605a884acbc2611a5d61817a491e8cf7b516b375d9bcd97a3f85c4b`。普通回归及 Release 样本尚未执行。
- [x] 完成宿主准入操作九源私有 V2 修复与独审：真实节点、submission、context 与 graph guard 保持到宿主 body 完成；完成回执以私有字段及只读借用保留返回 payload，先销毁 payload 再释放 handle，关闭后 body 结果与真实 TaskTerminal 分别保留。[九源冻结](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-host-local-admitted-operation-support-v2/manifest.json) SHA `ef157b94a5a007de9e5cdd492b15d5ddf6180f0f71b48ada93fca839df40ed0e`；[独审](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-host-local-admitted-operation-independent-review-v2/review.json) SHA `b9af9140a9118554d6d9019ece5822177a23df193e745a555b34071641c79d0d` 核对 18 次原始回放、九文件 Rust 2021 格式检查，四个旧测试名和 27 条旧断言保留，新增返回 Rc payload 保留 quota 后同 owner 重试源码用例，合计五测试、36 断言，均未执行。
- [x] 记录 Managed Session 33 源有限独审：[冻结源码](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-managed-session-owner-checkpoint-v1/manifest.json) SHA `3054be7ad1c509cfac29af27b13b968721a2d324c5c11d5a23f9acdd233b5eda`；[独审](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-managed-session-owner-independent-review-v1/review.json) SHA `eb80c9816b3ba4773621209e058e4a38ca038de2e5f749aa244bc5473aaf5fa9` 的 66 次原始回放与 33 文件格式检查一致，596 条旧断言、124 个旧测试名均保留。实际准入成功先发布有效 native handle，启动结果留在同一 persistent task 内；这不表示启动 Ready 或完整生命周期验收。
- [x] 已在新的私有 35 源后继修复首次模块清理顺序：pinned body 只执行 session-affine 清理，外层等真实 canonical TaskHandle terminal 后才开始 module/physical graph/wake/log 清理；log 槽在 factory 前定义，acquire 后立即转入，factory Err/panic 保留实际 lease。[35 源冻结](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-managed-session-owner-checkpoint-v2/manifest.json) SHA `745ef9632dedec8d711238ee794b3fc5148ff6bc67e5a825c0e18d1cadb1b8f8`；[非作者增量独审](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-managed-session-owner-independent-review-v2/review.json) SHA `3272c1875faaaaa3aee5b3a6b98519f51a773015f93ce53add2bba8a8d688ab8` 核对 70 次原始回放及五个增量 Rust 2021 检查，旧 624 个断言词法 token、128 个测试名保留，新两项因果源码回归未执行；R1 清理顺序与 R2 实际 Host V2 scope/graph 绑定在源码边界闭合。
- [ ] 完成命令/回复真实 byte arena、过期 deadline 的串行阶段推进、bounded startup query/destroy、partial Core/manager/code owner 保留与整个 Runtime caller 组合。
- [ ] 合成 Scene 的实际 first mutation、DTO/capture 重量与同次 admitted leaf；统一提交兼容 Check、普通回归及隔离 Release 批次，取得原计划延迟、分配、RSS 和 quota 的实际数值证据。

本节新增 owner 和 caller 修复为私有冻结候选；共享 Rust 应用以实际 apply 与归属终态为准。未执行的源码回归、原始回放和格式检查不计为测试通过或性能达标。

## 2026-10-01 补充：实际 buffer 支持与上下文身份

- [x] 已写入共享 `MaterialOverrideSet::retained_payload_bytes()`：按实际 `Vec` capacity 与 Copy slot 元素表示 checked 计费，资源对象 referent 不属于这个 buffer。[实际单源写入和归属收据](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-material-slot-helper-current-admission-v1/one_material_slot_helper_guarded_actual_apply_and_attribution.json) SHA `ba049db7251f942e6db2e1547f1c813ea24d28e85cda0a08edd7df58d7ac90ab` 确认公开 eligible 移交、fresh lease、原始字节归档、diff check 为 0 和 attribution 完成；[独立静态审查](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-material-override-retained-payload-independent-review-v1/review.json)已核对实际类型与 buffer，未执行 Rust 测试或性能测量。
- [x] 私有 same-activation validator 已独审：真实 ScopeInner Arc 创建 token Weak lineage，JobScheduler/UiDriver 纯身份检查拒绝 foreign Core 和同 Core 不同 activation；旧 admitted context 在 Closing/Cancelled 后继续有效，不进行第二次准入。[七源独审](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-activation-context-validator-independent-review-v1/review.json) SHA `58015cb780befe547c5effd52765c953bd25951713461b3dcd6c53bec4c45802`；四项因果源码回归未运行，旧 same-pin retirement body 保持原始字节。实际 CoreV3 TaskNode provider 与 shared 观察依赖分别记录，最终组合仍须保留真实 provider。
- [x] 私有 same-pin terminal retirement 支持已独审，沿用 primary 的物理 worker submission，普通 single-use 和 Successful dependency policy 保留；退休控制仅依赖真实任意 terminal，不需新准入。[退休支持独审](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-pinned-terminal-retirement-independent-review-v2/review.json) SHA `3ac4c52405dff62bf753ae294caa258ac34ee7928f69cb80cc3028744dad8295`。注册 API 自身不强制 deadline/启动屏障，Native 必须在 factory enable 前预接受控制；它也不屏蔽显式 cooperative cancel。
- [ ] 完成实际 Native V11/CreateConfigV4 的 DLL purpose lease、固定 startup receipt、同 id/首次绝对 deadline 的串行清理与物理 join 后 Slot removal；完成 Pointer full UiSurface known/opaque payload 计量和调用方采用纯 validator。
- [ ] 合法应用完整 caller cohort 后统一提交兼容 Check、普通回归、ABI 与隔离 Release 批次；取得原计划 latency、allocation、retained quota 与真实 RSS/产品性能数值达标证据。

本节只有 material buffer helper 是新增共享 Rust 写入；其余明确标注为私有候选。旧冻结、foreign owner、原测试与性能门保留；异步编译提交后继续功能修复，不持续查询或等待编译。


## 2026-10-01 · 当前源码组合进展（执行与性能验收待完成）

- [x] 私有 Native V11 interface17 和 status2 已独审；status 文本上限不充当临时分配或 RSS 证据。
- [x] App provider8 + export3 已精确原始回放，保留实际 Library、同 attempt/deadline 及拒绝 owner；完整 Native/caller 执行待完成。
- [ ] 完成 scoped/reply 全 caller、Arena/Wake 独审、World/字体真实预算、合法 module/worker 清理及整批执行。
- [ ] 一次派发兼容 Check、正常回归、ABI/package 与隔离 Release；修复实际失败并取得原计划 latency、allocation、retained quota、真实 RSS 数值达标证据。

详细范围和不可替代的验收门见[所属优化记录](../../../optimize/zircon_runtime/02/2026-10-01-task-event-owner-admission.md)。

本节 `[x]` 仅表示明确范围的私有源码修复或独立静态审查；`source_reviewed_execution_pending` 不表示已写入共享 Rust、测试通过或性能达标。兼容任务合并提交后继续功能修复，不逐项编译、不阻塞等待或持续查询编译状态。


## 2026-10-03 当前修复与批量验收

本节更新到 V49。Resource51 的三个文件已实际应用到当前 Main；其余修复按私有源码及独立审查范围记录。编译、原测试和性能达标继续待验收。

- [x] Resource51 无过滤查询直接复用当前有序 entries，去掉额外筛选和同序排序；类型过滤分支与其他过滤行为保留。三个原顺序测试补上真实查询对照，另加独立 10K／100K／1M listing profile。实际三文件写入、原字节保护和限定差异检查通过；原索引、资源管理器及 22／24 性能场景保留，未移植 Source6。普通测试与测量尚未执行，输出 Vec 分配没有标为零。
- [x] Core supplied-scope V2 保留原执行器关闭／停止原因优先级，再读取目标 scope；Loader V2 精确绑定该真实 API，pending 发布前保留原 typed refusal。独立源码审查关闭已列依赖问题，相关原测试仍待执行。
- [x] App／Editor 三文件 V3 修复全部 19 个真实配置出生点，包括两处隐藏默认配置的便捷调用；保留 155 个断言表达式、41 个 selector、原 Runtime／Minimal 配置和生产缺失 policy 的拒绝。
- [x] Font／B2 的 791 文件已完成列明范围的独立源码审查与实际来源核对，保留同一个 cooked Header／Vec／adapter；容量拒绝先释放原字节再退款。审查仅覆盖明确列出的用途与调用体，Arc 弱引用尾部、DLL／native、allocator、RSS 和原性能门继续开放。
- [x] World 暂存复制使用同一原 policy 的可失败 try_clone，保留原大小错误优先级、运行容器恢复及真实活跃 schedule；JSON 原定义导出已恢复。Resource／World 两表事务先准备再变更 epoch／事件，拒绝保留原行与 owner，原资源在 mutex 外释放；完整 schedule 交换保留真实绑定。各限定源码复审通过，新增原用例仍未运行。
- [x] Operation 八文件使用 tick 传入的原 Core Compute 调度器自动准入；保留真实 TaskHandle、原错误 Arc、取消／过期原因与一次 slot 释放。独立源码复审通过；操作完成事件没有等同于 Core 账本或 allocator 归零，原 32 并发／4 MiB 编码限制没有当成物理堆证明。
- [x] 三个真实原生收尾用例已封存通过，历史失败保留；自然排空与超时强制清理分别判定。首次八项验证在 Cargo 前被严格清单拒绝，0 项编译／测试执行；补齐两个 cc 版本的 8 个真实 src/target 模块后，29649 项源码清单与八项配置通过限定复审，原 worker／命令保持。
- [x] 修正后的八项异步验证已由一个隐藏 worker 一次启动：两项 all-targets 检查、RHI／Wgpu 各 Debug 与 Release 测试、两项 Cosmic 测试。启动前 D 盘实际余量 37.944 GiB，原 35 GiB 门槛通过；启动日志已封存，0 次启动后结果读取、0 次等待。结果保持 pending，原首次清单失败与容量拒绝历史保留。旧 build 目录压缩仍待完整收据，禁止新编译使用，未把当前余量归因于尚未验收的压缩。
- [x] Editor 的 31 文件修复包完成独立源码复审：84 处旧默认构造移除，保留 203 个 selector、1,076 个断言表达式、原 Compute 构造参数和已加载 World 的 policy。DynamicScene schema／adapter 与同 owner 出生接口也通过限定源码复审；原调用体、目录别名和资源转移寿命保留，测试尚未执行。
- [ ] Native Schedule V1 发现真实 P1：回滚时再次分配 header 可能掩盖最初的拓扑错误；修复新版本进行中。字体 805 文件及 root／standalone Cargo／lock 候选按真实用途合并，保留当前其他域修复；10 处后续调用重叠继续处理。解析器、全套 Runtime／Editor 原测试、DLL／弱引用物理寿命及原性能验收仍开放。
- [ ] Resource51 原三次 6666 对样本的 P95／P99 高于基准分组仍为 4／7、1／3、2／5，原尾延迟没有标成已修复。R9 独立用途版本仍排序，不能套用当前 Main 有序索引的免排序收益。继续按原 count／bytes／join、CPU、RSS、I/O、allocation、p50／p95／p99 及 Editor 原性能门槛验收。

[V49 实际代码、修复与验收状态](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime-editor-current-source-results-and-records-v49/current-source-and-results.json)；[源码复审与异步动作来源](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime-editor-current-source-results-and-records-v49/qualified-source-review-and-dispatch-bindings.json)；[Main 三文件实际写入](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime51-current-main-unfiltered-listing-actual-guarded-apply-20261003-v1/actual-guarded-source-apply.json)；[最近压缩结果离散读取](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-main-resource51-and-font791-qualified-source-milestone-existing-compression-result-v1/existing-compression-milestone-capture.json)。

累计 30 次离散编译结果读取、26 次 terminal 封存、0 次编译等待。勾选只对应明确列出的实际代码应用、私有源码复审或已封存原生用例；普通测试与性能完成标准保持。本次记录写入没有改变 Rust、索引或 tooling；本里程碑已实际应用的 Main Rust 为三个文件。未提交、推送或外部通知，Goal 保持 active。

[本次八项真实异步启动日志](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-actual-native-eight-focused-check-test-one-shot-launch-20261003-v2/actual-hidden-worker-launch-receipt.json)。
