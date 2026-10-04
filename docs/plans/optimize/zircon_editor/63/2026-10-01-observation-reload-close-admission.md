# Editor63 · Nonmutating observation 与 atomic close decision

## 本轮源码修复

R3-r3 对普通 committed observation 保持无副作用，为 pending dirty 和 close decision 保留 engine/history/route/revision 的原子身份。reload 在任何 flush 或 prepared seed 消耗前，预检 pending commit + target history clear 的完整预算、dirty journal、selection revision、context 类型及 route。

MAX−1 计数原会在第一步变更后第二步耗尽，现用完整 mutation cost 预检拒绝；MAX−2 成功、clean 单步及不同 history 预算都有私有回归。[editor63_r3r3](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-editor63-close-prompt-pending-v2/prepared-r3-r3.json) SHA `a1279bcddfde76fc785f48fc65e72c483a86963486c1dc3e6dae18807ec53da0`；[editor63_r3r3_review](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-editor63-r3-atomic-reload-independent-review-v1/review-r3-r3.json) SHA `9f9f89404e63b63566ca336a05758f3117ba6bacf257094833f0644e49de5c76`，24 候选精确重放，独立静态审查无新增必修项。这些不是已执行测试。

- [x] 完成普通观测无 mutation、pending 身份和双变更预算的私有修复与精确静态复核。
- [ ] 完成 V4 common writer admission、native Edit origin participant、同 close ID 保存/Discard/Play/终态退休，以及真实 persistence terminal receipt。
- [ ] 合成 V4、R3-r3、Editor05 origin guards、Runtime02 owned handles 和 Runtime09 全 42 typed terminal/deadline 源契约。
- [ ] 完整来源准入后应用并按批次执行正常回归，修复实际失败。
- [ ] 实测关闭/保存/重开、竞争、延迟及原有性能门，取得 terminal managed 证据。

共享源码未应用；整波 `appliable=false`。仅 history tokens 为空不能证明 native/raw authoring 变更已持久化，Clean/Save 必须绑定真实 source generation 和保存完成回执。closing capability 不随 UI scope drop 重开 writer；未持久化或未证明的 native writer 应返回 typed decision，不得伪报保存完成。

验证按兼容的 Check、正常回归、ABI/boundary 与独立 Release batch 合并提交。提交后继续功能修复，在下一源码里程碑核对已有回执，不连续监控编译，不以静态记录关闭 F4/F5 或性能门。

[对应优化记录与完成列表](../../../astra/features/editor/1071-editor63-transaction-observation-close-admission-completion-list.md)。


## 2026-10-01 · 实际 Edit writer 生命周期修正

[实际 shell/project lifecycle census](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-editor63-close-authoring-admission-v4/native/shell-project-lifecycle-census.json) SHA `15e677ec980e4b8f9cc3bea998b0b7b17b15dc491826b24696cfedd3cc8b5ab8` 确认 startup DLL gateway 是 projectless App shell；实际 Edit world 经 EditorAuthoringWorld / InProcessGateway 持有 LevelSystem。Project close 不能退休整个 App shell，activation 也不能额外 create_session(project) 生成重复项目。最低共同 admission owner 正在向中性 Runtime/Interface 支持层修正，覆盖 public LevelSystem clone 与 with_world_mut 等真实写入入口。

- [ ] 完成最低 owner fence、checked revision 与持久化终态证明；没有 history token 不能当作 Clean。
- [ ] 将 Project close 的 Edit/Play/文档/资产/watcher 退休证明，与 MainExit 的 shell actual shutdown 分开，并保持同一次关闭的绝对 deadline。
- [ ] 实际旧项目 terminal receipt 后，在真实 manager activation route 安装 fresh project authority；旧 Arc 永久退休，不通过重新开放旧 guard 恢复。
- [ ] 封存真实 owner/caller 表，合成 typed DTO/ABI/consumer 生命周期并执行正常回归及产品性能批次。

V4 整波仍 nonapplicable；以上为发现与在修复依赖，不是已完成关闭、保存或重开验证。

## 2026-10-01 关闭与 authoring owner 后续修复（V4 未可应用）

- 实际 Edit owner 仍为 EditorAuthoringWorld → InProcessGateway → LevelSystem；ProjectClose 之后存活的 startup DLL shell 只由 MainExit 退休。旧 Arc/Level registry、VM 内部写入和 same-project reload 的一向退休继续在最低层闭合，不能以 bool 或 census 代替实际消费者接线。
- 原生与 retained checkpoint 已提供新的静态切片，但 Engine 中性 owner 消费、save/source proof、activation rollback、shell release 尚未整体冻结和验收。
- Idle pending close 必须收到实际 terminal native wake；仅设置 AtomicBool 不算完成。正在复用 HostEventLoopWake/EventLoopProxy 的窄 post 能力，并处理 unlock-before-last-permit/post 和 attach/recheck 的 lost-wake 边界。
- 跨 DLL 的 remaining timeout 重建会延长 queue/lock wait 后的截止时间；新的公共 monotonic absolute carrier、同 CloseID 首次截止时间与 protocol identity 匹配仍为未完成门，不宣称当前相对 timeout 满足 whole deadline。
- Runtime UI 与 retained presentation 的新接纳失败传播必须保持 closing authority、旧 render/cache/dirty 和统一 publication；不得通过 force hide 或新默认 Core 绕过关闭。
- [ ] V4 whole consumer/ABI/source composition、行为测试和真实 save/close/native wake 通过。
- [ ] Editor172 单绝对截止时间与真实性能预算验收通过。

## 2026-10-01 后继独审与统一提交边界

- Scene viewport V2 的首次 Stale marker 拒绝/重试和唯一借用 reset prepare/commit 已经五文件增量独审静态闭合，真实 6 源 / 55 项 Runtime API 绑定也替换了旧错误依赖。五个回归仍未执行，外层 authoring/borrow/type/native/性能门开放。
- Retained B68 已独审 68 个源文件、两个完整原始字节回放、Rust 2021 格式与旧断言保留。新的 geometry/window-metrics 暂存后继正在接入真实全部窗口统一 publication；单个窗口成功不能替代后续窗口接纳失败时的整体旧 frame/cache/dirty 保留。
- Asset 的 31 个外部测试 fixture 已迁移 179 个构造调用并保留 1558 个断言与 185 个测试名，私有 replay/格式通过；源码独审、整批正常测试与数值性能未完成。
- Editor63 checkpoint3 的旧持久化 RAII、跨 catch uncertainty 与 Scope 权限传递已静态复审；新增普通 cancel 的 OwnerKey 依赖及 gizmo setter 错误转换仍是明确 P1 后继修复。实际 activation/world registry/native terminal wake 与绝对跨 DLL clock/ABI 后继仍待组合与验收。
- 本次只准备记录与已独审 EventBus contention 两源 profile，不写未冻结 Editor 大 cohort，也没有编译/测试/性能通过结论。

## 2026-10-01 补充：实际调用方、窗口计数与生命周期审查

- [x] 记录独立静态审查：窗口成功 pane 发布与非空 geometry addon 发布补回原有 rebuild 计数；拒绝、纯 geometry 和空 addon 不计数，旧 wrapper 不重复计数。
- [x] 记录剩余 11 个 Editor 测试调用方的私有迁移：485 条旧断言和 37 个旧测试保留，构造与执行借同一个实际保留的 Core。测试尚未执行。
- [x] 记录 UiSurface 专用 projection checkpoint 与 actual manager secure-store rebind 的私有修复；普通 Clone 的跨 surface 隔离保持。实际 pointer caller 后继、Scene mode/world/focus 的准入顺序及整体类型闭合仍待完成。
- [x] 记录 Engine checkpoint5 与 clock3 私有增量独审：V3 receipt 验证、activation terminal wake、held permit 转发和错误阶段缓存 deadline 的已确认问题已静态修复。
- [ ] Retained actual host 失败清理、安装与 hydration 顺序、fresh watch proof、EditorState 终态 metadata acknowledgment、startup Pending owner retention 完成整体组合并通过运行验证。
- [ ] 完成整批 managed 编译与行为测试；执行原计划 Release latency、allocation 和 RSS 数值验收。上述私有源码和静态重放不能作为测试通过或性能达标证明。

审查依据：离线候选目录中的 `root-runtime02-editor-ui-retained-pane-counters-v2-independent-review-v1`、`root-runtime02-editor-ui-remaining-caller-tests-independent-review-v1`、`root-runtime02-surface-projection-checkpoint-independent-review-v2`、`root-editor63-v4-engine-checkpoint5-independent-review-v1`、`root-editor63-native-abi10-clock3-independent-review-v1`。最新归属与字节绑定见本次公开准入及实际写入收据。

## 2026-10-01 补充：终态 metadata、旧来源拒绝与永久错误唤醒

- [x] 记录 Engine checkpoint8 的私有修复：取得实际 Detached lease 后先清理 gateway identity、loaded 状态与诊断，再清理 EditorState 并消费 opaque metadata acknowledgment。[Engine 独审](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-editor63-v4-engine-checkpoint8-independent-review-v1/review.json) SHA `9433c3aa85e5be250533c223c2536007a4e0f0d3d6114509c755bfc306ac2b5f` 核对 51 路径及两文件增量；真实消费者由 [Retained 86 源独审](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-editor63-retained-checkpoint8-independent-review-v1/review.json) SHA `50b5f892fd2e389c63cf951b3629472320ee4bf8c13d132397697416a9030026` 绑定 actual Controller acknowledgment、Manager guardReleased/ledgerClosed 与 HostProof，确认释放 holder 前的 metadata 顺序修复。完整 native terminal proof 尚未验收。
- [x] 记录 [原始来源刷新独审](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-editor63-retained-checkpoint9-refresh-independent-review-v1/review.json) SHA `5a27e52d350c5cfca4604e58cbd9da0d6ef88305da32951c6e8329cd2c863abe`：commit 在读取 current Root 之前校验并接纳 original source，真实旧 Engine retired / 新 Arc owner 用例保留旧来源拒绝后新 key 可用。当前 fixture 使用空结果和 None Root，完整 populated workspace ABA 场景与行为执行仍开放。
- [x] 修复永久错误反复唤醒的私有 P2：terminal_workflow 仅对真实 typed EngineBusy 注册原 snapshot wake，永久 InvariantViolation 保留同 owner 与诊断；实际 status setter 对相同诊断在 invalidate 前返回。[单文件增量独审](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-editor63-retained-checkpoint10-wake-independent-review-v1/review.json) SHA `3ddd793725d946b124b03063c9894ab1c21f7b8d1ea454a18ee0eafd669172b9` 独立核对原始回放及三份实际调度源码，无新增必修项。永久错误用例覆盖分类 helper 和真实 idle producer closure；Busy 用例覆盖实际 reservation、拒绝、unlock 后一次 wake 与同 owner retry，均未执行完整 watch controller/native idle 测试。
- [ ] 完成真正 bounded startup/query/destroy 的 Session 生命周期 provider：既有 API10 authoring Snapshot/BeginClose 会先阻塞等待，Mark 只处理内层，Release 不删除 SessionSlot；新的必要 V11 生命周期 DTO/table hard cut 由 Runtime 唯一 writer 实现。上层须保留实际 partial CoreLease、manager、host state 和 code owner，直到同一 first deadline 下的实际 owner/module/graph/wake/log Joined 与 slot removal 后再释放。
- [ ] 修复实际 Native SessionSlot 的 code_owner 使用 Arc<()> 占位的已确认缺口：必要 ConfigV4 必须接纳由 App 实际 Arc<Library> 提供的 purpose-bound code lease，覆盖 factory、Pending、panic、整个 owner 清理和物理 worker join；retain/release 在锁外，当前 FFI 调用由 App call lease 保持到返回后。此后继尚未作为真实 provider 冻结或运行验收。
- [ ] 合成整波 owner/caller/ABI、全部窗口发布及原始关闭/保存/重开回归，合法应用后统一提交兼容验证批次；实际 native terminal wake、两次关闭、Editor172 绝对 deadline 与 Release/分配/RSS 产品性能验收继续开放。

本节记录精确私有源码修复与独审，未宣称共享大 cohort 已应用、测试执行通过或性能达标。编译提交后继续后续修复，只在新的源码里程碑核对已有 request identity，不持续监控或阻塞等待。


## 2026-10-01 · 当前源码组合进展（执行与性能验收待完成）

- Engine V11 checkpoint9 最终 r2 仅改实际 gateway 与 tests 两源，保留其余 C8 49 源和 Authoring V3。[独立增审](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-editor63-v4-engine-checkpoint9-v11-consumers-independent-review-v1/review.json)（SHA `775b6faeeb0433427cefb826525b65d7e5ee4ee7a73fc52f38271dfca42c8aca`） 确认同一原始 deadline 在每次 wake 后调用前检查，避免 continuous wake 越过时限；实际 headless linked code owner 是 process-lifetime purpose，不是占位 Arc<()>。变更文件测试名 21→23、断言宏 49→53，均未执行。
- Native interface17 的 V11/ConfigV4 hard cut 与 App 实际 Library purpose/call leases 已分别私有冻结；[接口独审](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-session-v11-interface-independent-review-v1/review.json)（SHA `0ea9efa347858615a4a1f115b27969d91d05d22e1e8eeb22511bbe3a9a00d67d`），[App 十一源原始回放](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-tracked-v2-editor-ui-source/session-v11-app-owner-integration-v1/raw-replay-longpaths-receipt.json)（SHA `865590dc0fc637934811ba07049b31ddab7ecfad6392ef099d955be021b0e00e`）。完整 Native producer、动态 DLL 路径、真实 Ready/Joined 和 Slot removal 仍开放。
- 生命周期组合必须先消费真实 Authoring/Retained terminal proof 并 ReleaseAfterTerminal，再开始 App destroy/Slot removal；移除后不再调用 native endpoint。原 State metadata Ack 只能由真实 State 与 facade 消费；构造前失败仍缺 purpose-bound no-State 生产证据，部分 State/原始 Core、UI、manager、gateway owners 必须保留。
- 同关闭 ID 保存/Discard/Play、partial factory 失败恢复、全部窗口发布及 separate ProductCore module/physical graph 终态依然待组合与运行验收。[C8 不可应用骨架](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-editor-state-project-authority-join-successor-v1/manifest.json)（SHA `4825c67c47e7469dee5a99a31be32b918008303f693b528a87f9f661950deab1`） 不进入共享源码，不能以 Unknown 永久拒绝关闭实际 gate。

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
