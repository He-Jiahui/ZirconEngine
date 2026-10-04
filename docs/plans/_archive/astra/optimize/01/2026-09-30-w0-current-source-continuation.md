# W0 当前源码续接 — 2026-09-30

## 范围与证据状态

执行 [Astra 主计划](../01-review-and-repair.md) 与用户目标文件的完整 W0–W8 范围：全部 optimize 非 tooling 缺口、首批 P0/P1、zr_vm/WOC、本地 Hub 账号/团队/商城/云同步、Windows 产品、回归、性能和独立复审。目标文件 SHA-256 为 `31de5490f3ea004fc403011f867e6b63435adb2ff0f54fc20355ed924477179a`。此记录是未完成工作的续接，未接受任何新产品或性能门槛。

当前主会话 `astra-goal-20260929-01a0f033` 注册回执：`d7b1b58af17a4d24965ce1f6eb5563ea`。主计划与本记录由 archived 来源执行 eligible ownership transfer：fingerprint `6bbd9e060fcbc8ea3f9e5980db2b2542606d16dd7322aeedc48fb2de7cf46baf`，apply 回执 `5197fc47567746578e5b63b624c65ce9`；编辑前精确 claim 回执 `15a3d2e3a28d4347b143ac49ce3d169e`。既有主计划内容保留。

用户明确授权核验后移交旧 primary。两会话的 Codex binding、execution handle、active Cargo handle 与 live lease 均未发现；状态变更回执分别为：

- W0 `astra-w0-crosswalk-20260926` → cancelled：`7e595c650134419da3cc29f63f5d2524`。
- ED-A6 `astra-ed-a6-m4-deterministic-ids-20260926-01a0df17-01` → cancelled：`7f727a129a7245f98dc1f96a63c458f9`。

取消只释放旧计划占用，保留源码和待验收记录；活跃 foreign owner 的源码仍须正式 handoff。

## W0 库存与未读边界

[2026-09-26 报告库存](2026-09-26-w0-report-inventory.md) 的 576 个 top-level 报告路径，记录了 61 个包含有界 source-check 行的路径、515 个没有 qualifying 行的路径。61 并非整篇报告已审，更不是 finding 已关闭。

私有结构审查包位于 `C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/w0/`，对应 2026-09-29 工作树快照：

| Artifact | SHA-256 | 意义与限制 |
| --- | --- | --- |
| `report-snapshot.jsonl` | `e2f88e1f71b82fdbd113428dbb3f249bb6de8115ae652a99b93d0797f190adb2` | 576 个报告的路径/字节证据；575 与旧库存 hash 相同，Runtime76 已漂移 |
| `finding-id-candidates.jsonl` | `5ce75335f07b7272f78a5e41ec7104f1796ae375f5ba20567960d98bb0d00a74` | 27,629 个 report-qualified、未合并结构候选；包含 finding、gate、reference 与 range，不能当作原 finding 数 |
| `nested-review-source-candidates.jsonl` | `a2dd31a1208b083eb0cbcd73eefd24031229d626b272e7ed76f946bef573d38c` | 2,378 个 nested Markdown 中按文件名/H1 选出的 62 个发现候选；其余 2,316 未由该 lane 阅读 |
| `extraction-summary.json` | `73591eaed90fdcb39e54745220f370e4b18cc424a8616b80c9a43dc0b589e8f6` | extraction 计数；canonical/alias/accepted finding 数均为 null |

69 个报告没有该结构规则可提取的 priority ID，558 个 heading locators 仍待语义审查。九个 excluded supplemental notes 不并入分母。结构包通过结构校验，但没有因此接受源码、测试或性能结论。后续 Core/UI 与领域源码复核是增量证据，必须逐 ID/hash 入账后才调整库存，不能把该结构快照自动升级为 source_checked。

### 2026-09-30 有界源码复核波次

初始 exact-ID 证据如下，原 37 行/11 条关系 snapshot 已保存在私有 `w0/snapshots/pre-source-refresh-2026-09-30/`；下表指纹绑定该历史版本。37 行按原报告 namespace 保留，状态为 5 `confirmed_open`、24 `partially_implemented`、8 `implemented_pending_validation`；11 条关系只记录有明确文本依据的 refresh/supersedes/crosswalk。每行保留 report/source SHA-256、有界行段、最低 owner 候选、依赖、测试/产品/性能门槛。`tests_run=0`，canonical 与 accepted 数仍为 null。

| Artifact | SHA-256 |
| --- | --- |
| `current-source-review-wave-2026-09-30.jsonl` | `ee51bfd0bd6644da4566c1124a017e162bbb9f6cb9ec87c065836a7e197f7381` |
| `currentness-crosswalk-edges-2026-09-30.jsonl` | `635a65ded6a2b0c5af8142d3d3056a72d846b1adf44e92e5d245ebcf0424201c` |
| `current-source-review-wave-summary.json` | `3aeff44903dbb59b452c91fef797158b2aadba1eb602d36f281f93853dfc93c4` |
| `2026-09-30-current-source-delta.md` | `c00578e6cdf540c8e0b1b1f351864f9c63c14e9f7b4838437eb3d26121168047` |

关键源码结论与 namespace 约束：

- Runtime72 `RCL-P0-002` 的单模块 activation dependency-closure race、`RCL-P0-003` 的锁内 service destructor 均 confirmed open。后者同时覆盖 unload 与 failed activation/reactivation；`CoreWeak` 允许 destructor 合法重入。源码/test 修复必须证明 reentrant/panicking Drop 与 concurrent resolve/unload，不能只替换单一 `instance = None`。
- Runtime157 显式 refresh Runtime01 `P0-1..3`；Runtime72 `RCL-P0-001..006` 是 additive namespace，没有 Runtime157→72 直接 currentness edge，不将 RCL-P0-003 改名为 Runtime157 P0-3。Interface08 refresh Interface01 的五个原 P0；Interface15 certification 是独立门槛。
- Interface08 原报告的“host table 仅检查 version”“destroy 没有 deadline”已与源码不符：当前检查 alignment、exact version/size，销毁已有 absolute drain deadline。指针生命周期、producer materialization、BuildSet/真实 DLL、cancel/escalation/App 首次销毁失败策略仍有剩余门槛，不能因此关闭整项。
- Runtime11A `P0-6/7` 仍缺 aggregate hit-grid membership/query/bytes 与统一 topology authority/cycle/depth 约束；Runtime11B font、secure IME/product 与 zeroization 门槛仍开放。headless 真实 DLL 测试仍 ignored；Runtime PlatformHost/SurfaceLease/UI window pump 缺 production App 集成。

该波次不改变 61/515 的旧路径覆盖含义。复核期间重新 hash 576 报告发现 Runtime174 相对 2026-09-29 结构包漂移，当前为 `113e762934127c142e3f0b1d3f60113b2e2614286370c398d3264bef360aa21d`，不在这 37 行波次内；Runtime76 相对更早库存的漂移另行保留。其余原行和 nested 未读范围未自动升级。

根会话整合后的独立 freshness 检查覆盖该波次的 85 个 distinct `(path, captured hash)` 键，发现 10 个 source hash 已变化、相关 report hash 未变化：Core lifecycle state/handle/activation 及 contention test、CoreRuntime/task graph、SurfaceLease registry 和 UiTree。以上 37 行仍是表中 artifact 指纹绑定的历史源码快照，不能全部称为当前 head 结论；W0 lane 正在重新检查变化合同并保存旧 snapshot，不能只重写 hash 沿用旧 rationale。结构检查通过不等于 freshness 或测试通过。

## 已冻结 Runtime UI 候选

原 finding 为 Runtime11A `P0-2`。最低 owner 是 Dynamic Session 的 RuntimeUiSurfaceSet 与 UiInputManager。真实单调时钟、事件 stamp、共享 surface timestamp、timer demand 和 `DynamicSessionState::tick_frame` 接线已落源；错误使用 `RuntimeUiInputTimers`。表内哈希是此次候选，不是全 Runtime 闭包。

| Repo path | SHA-256 |
| --- | --- |
| `zircon_runtime/src/dynamic_api/session/error.rs` | `bdacd475f54ab47192b52adb457199f90f012bd72efa3956a14b4d1af1f3df39` |
| `zircon_runtime/src/dynamic_api/session/runtime_ui.rs` | `b6faaf7b4461e7c725a06516fcf06408f564ecd0255cf617c06abbc63f7bde05` |
| `zircon_runtime/src/dynamic_api/session/runtime_ui/input_routing.rs` | `76efb6a809a425bfc062d2ed6ecaed389b6ee7ba9d3a01a15326725129266754` |
| `zircon_runtime/src/dynamic_api/session/runtime_ui/tests.rs` | `8de3314b3e4abcb122d8c13a57fdfbcb5f1e9560361f7ae2a66752f80912a6f7` |
| `zircon_runtime/src/dynamic_api/session/state.rs` | `056cb42e745ee8fee09289c2e8d1592d866841e0a6c89bbee6ec362c89d53e2b` |
| `zircon_runtime/src/dynamic_api/session/tests/frame_demand.rs` | `e8a5dcf681a3054a028719bf358cab71c16047331c43ad7a9b0a281599973d9e` |
| `zircon_runtime/src/dynamic_api/session/tests/runtime_ui_surface.rs` | `3b88c4c0d031077cbe371e494679a700fbe0bafb88939d4740e5585c0cebf7f1` |
| `zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs` | `f9e4c7582cda56b3905f03a24c9094b2c72546863715b15542290afada18bf54` |
| `zircon_editor/src/ui/retained_host/host_contract/window/redraw.rs` | `9accba7254c68675f62450ed7c9119c4b7c01867aef12f7da16c8b12895853fa` |
| `zircon_editor/src/ui/retained_host/host_contract/globals/state.rs` | `bbab6ec77a6753cc50270c7f5c2039725954866b43681a689f0e23609dae44c2` |

`runtime_ui_surface.rs` 是注册输入，UI lane 未修改。原七路径 attribution 回执 `03ce9c86b60145fe816e1fc8360a8ec4` 与此前十路径回执 `bbc2434c648849e989c96241ca0568d2` 保留为历史；pump 前后 owner 合同加强后的两个 Editor 文件 attribution 为 `88964d36c42748d69955775e203a8b4a`。当前十路径清单 `ui-time-candidate-20260930T2157.json` SHA-256 为 `fc0bc400cbb5dfc9ce888ad08399541d49472d1d49a852e6fb6219d3168c1e7a`，独立复审按字节核验全部哈希。lease TTL 为 300 秒；验证封存前须由 owner 续租，历史 claim 不构成现行写权限。

此前独立审查发现 `about_to_wait` 消费 wake 后 scene tick 失败会丢失下一次唤醒。当前生产 tick 调用 `UiHostWindow::complete_runtime_frame_tick`，失败保留原错误并以 50/100/200/400/800ms、随后最多 1s 重试；gateway owner/generation 变化取消旧重试，成功清除重试状态。功能测试直接消费真实 host wake、drain redraw，再检查失败恢复与 generation 替换；不以源码字符串测试替代该行为。对应过滤集包括 `runtime_frame_failure_after_consumed_wake_retries_until_success_with_capped_backoff` 与 `runtime_frame_owner_generation_change_cancels_stale_retry_and_resets_backoff`。这些测试仍未由 Cargo 执行，P0-2 保持 `partially_implemented`，实际 FFI/宿主消费链与时序验收仍开放。

成功返回也按 pump 前后 `(PlayInstanceId, generation)` 判定 authority：一致才应用 `SleepUntil` 或 `Continuous`，不同则更新 owner 并丢弃旧 demand。新增 `runtime_frame_owner_change_discards_successful_stale_frame_demand` 和 `runtime_frame_current_owner_preserves_successful_frame_demand` 使用实际 host wake/redraw 状态验证两类 demand、替换 generation 与 None owner。独立复审未发现具体阻点；格式和 scoped diff 通过，Cargo/产品门槛仍待执行。

## 当前验证回执与下一动作

计划执行的单一 Windows 受管命令为：

```powershell
validate-matrix.ps1 -Package zircon_runtime -NoDefaultFeatures -Features shipping-editor -LibTests -TestFilter dynamic_api::session
```

受管 live-source 路径会自行 reservation、封存实际相关输入并检查 drift，不能把 foreign dependency 的源码归属改为本 lane。请求 `33b9c64fd0574f82b3e6fd45f11fe532` 已调和到 terminal failed，错误为 `unmanaged_artifacts_detected`。拒绝发生在 Cargo 启动前；没有 check/test 执行结果、测试集合计数、接受的 source-manifest digest 或性能结果。不得重复该请求来掩盖失败。

下一步由验证 lane 读取现有 artifact/storage 诊断，确认路径、归属和存活执行器，交给最低责任 owner 处理；不删除未明归属的产物，不改写协调器绕过门槛。只有原因变化后才提交受影响批次。UI lease/attribution 修复不会使该失败回执变为通过。

后续 read-only artifact audit 请求 `37ecbfc0a8f64caf89361e7abcf60154` 已调和至 completed（2026-09-30 08:06:09Z），结果为 `unmanaged: []`；Astra 未执行清理。产物拒绝条件已变化，旧失败回执仍保留。UI 修正 helper 的路径拼写后，matrix 请求 `c9420544606e48fd8e25e6aa2486e68f` 证明七个路径均为 `integration_ready`，hash 与上表相同；UI admission 仍因新发现的 lost-wake 缺口而暂停，独立 AI 验证可继续。

### 后续 admission 结果

AI effects 的 corrected overlay 为 17 个 Rust 路径与注册的 Runtime174 plan；错误加入的 08f 文档及两个不存在的 test placeholder 不属于最终 overlay。验证 lane 的人工传递 hash 曾出现拼写错误，已直接按字节重算 `nodes/integration.rs` 为 `172abb133d14bede592a339ffc5950181b1bdf02210146303981d02856f5e3ec`。原失败保留；后续 manifest 必须程序化生成和比对，不能以聊天中的 digest 拼写为封存输入。

corrected admission 首先返回 `validation_ticket_external_worktree_dirty`（details request `f8a93a27ec144ca0896f4ccd38bf3a17`）。按既有 `coverage.externalWorktreeCaptures` 显式捕获 `E:/Git/zr_vm`、HEAD `0651abb41fd9d87d01ff56bae1da3d1ba3116146` 后，请求 `ca4c57d9135b47e5a04e32242e90fce1` terminal failed，details request `472374e0834042c1a1fe1f17814de6e7`，错误 `validation_ticket_external_worktree_changed`：`tests/parser/test_ssa_core_model.c` 在归档 file_verify 阶段由 `7cf43866…` 变为 `2988645c…`，长度仍为 38,960 bytes。这是外部树变化的具体证据；未生成 ticket，未运行 Cargo。checkpoint lane 仅修改私有候选，不是该 parser 路径的修改 owner。

下一次提交刷新 exact source/dependency fingerprints 与 owner 的 live lease/attribution，以支持的 read-only dirty capture 封存真实消费者闭包；捕获过程中 drift 则保留失败，不清空 sibling、不改写 tooling、不改变外部归属。编译器进程存活本身不证明源字节变化；支持的捕获仍须校验文件、index 与 HEAD。AI check 后才执行 effects 测试；Core LIFE-A4 的 package check 与 World/SessionOwner 两类过滤集分别列明。UI host lost-wake 候选已补源，下一批包含 Runtime 与 Editor 实际消费者。以上均无 passing Cargo 结果。

## 同期实施与独立审查

| 原 ID / 本地 alias | 当前证据与最低 owner | 未完成 gate |
| --- | --- | --- |
| RG-A4；精确 numbered alias 仍待复核 | `access_scope_tracker` / `resource_state_plan` 已计数 plane visits；compile caller 的活跃 owner 阻止集成新聚合调用 | 新 graph 回归实际通过、全部真实 work 计数、release p50/p95/p99；`RG166-P0-001` graph-to-RHI authority 独立开放 |
| ASSET-A3；精确 numbered alias 仍待复核 | one-pass discovery 私有候选保留 compound/nested identity 与预算；现行 discovery 来源有 foreign finalizing owner | 正式集成；64-file open 次数、失败原子性、retained memory、配对 release p95；ASSET-A2/A4 的既有实现须各自验证 |
| LIFE-A4；`RTASK-P1-07`，另有 `SEMR-P1-036/037` 生命周期 ledger | Plugins12 的三个已移交源码路径已实现 World late-success commit，Err 仍 rollback/requeue；SessionOwner test 已用 callback-start latch 验证两轮 deadline 返回、同一 callback 与 code owner 保留，释放后才 Retryable→Joined | package check 与两个独立过滤集真实执行；共享 deadline、最终 census、retry/owner 保留及真实 Windows/DLL 退出 |
| ED-A6 → `ED72-P0-01`, `ED72-P1-10`, `ED72-P0-02` | 逐叶投影、精确 surface→live leaf 路由、空 retain 全部退役及浮动 toolbar 实例选择已有候选；完整 per-leaf persistence 仍在推进 | 精确分屏保存/恢复、camera/toolbar/focus、短窗口/DPI、真实 WGPU 和 release 交互 |
| Hub03 `P0-05` / local-service S6 | snapshot/staging/CAS bridge、receipted publication recovery、entry/dir/aggregate-read 预算、marker-last 重试、exact account scope 与 switch guard、Arc-held blocking lease 已有源码候选；最新静态复审 hash 见下 | terminal receipt 后重启清理须 broker project metadata；ancestor junction TOCTOU 须 anchored IO；完整 apply/conflict/recovery、package-lock digest 仍为空集合占位；Rust/Keycloak/双客户端/Tauri/性能尚未通过 |
| `RT-AI-P1-015` | typed SetBlackboard/EmitEvent staged batch、validated overlay、World receipt；最终 18 路径已 attribution | Rust 执行、真实 World host tick；UpdateBlackboardDistance service 合同；SaveGame 跨 load exactly-once |
| `RT-AN-07` | reverse sampler/loop producer/cursor/FIFO continuation 修复已进入独立 animation scope | 反向区间/loop/endpoints/cursor 实际回归；完整 notify identity、ack 和产品动画输出 |
| WOC / ZrVM checkpoint；原 App finding 对照尚须逐项确认 | 私有 Create/prototype/function-cache/GC/preflight 候选通过 exact-hash 独立静态审查，含真实 C API mutate-then-throw→rollback→saveState regression；尚未改 shared source，正在保持边界的 private 模块拆分 | 正式归属/集成；完整状态恢复、post-rollback moving GC、失败原子性、Windows 实际回归/replay/WOS118 round-trip 与输入 BuildSet 身份 |
| PLUGIN-A3 | signed receipts、host policy 与 typed target producer 已有候选；生产 helper 仍把 index 当 singular policy，正常包操作当前失败。private consumer migration、host-owned selection 与 required load transaction 正在实施 | 正常 UI install→activation；不可信包执行零 DLL entry；必选失败阻止 Ready；拒绝 reload 保留旧代；真实 DLL/release |

Hub 控制器 6/6 与 TypeScript noEmit 的通过只绑定其当时输入；独立审查后的 native/storage/UI 改动需重新跑受影响检查。既有 78/78 服务和 40/40 Keycloak/HTTP 结果仍只覆盖历史记录的输入，不接受此新 bridge。

Hub 本次静态复审输入为 `zircon_hub/src/account/cloud/staging.rs` SHA-256 `60cfe06c82551223fa64d31d3a71e0bd452bfab2b29a1aac017b228a670da9f8` 与 `zircon_hub/src/tauri_app/account_commands/cloud_sync.rs` SHA-256 `8fd7727b4492bde8ed4d03925bd06b846f03fecd5b2ff0a390bd39d383491e69`。新 budget/recovery guard 回归只是源码，尚未执行；source review pass 不关闭上表的 ancestor race 或 terminal restart gap。后续源字节变化须更新 review/hash 与受影响验证。

## 退出门槛

W0 仍须完成全部原 finding 语义去重、最低 owner、依赖、当前源码 hash、测试与性能门槛映射；W1–W8 各自的产品退出条件不缩减。Windows 固定 Ryzen 7 5800H / RTX 3060 Laptop，排除虚拟显示 adapter；release 一般路径 p95 回退不超过 5%，新增热点目标为等价正确性基线 p95 改善至少 20%，保留 p50/p95/p99、分配/访问、RSS/VRAM。微基准、mock、静态名称与 pending receipt 都不替代产品输出。

全部预计测试真实执行通过、性能实测通过、独立审查关闭且子计划记录关联准确 snapshot 后，才可接受对应 finding；此续接保持 Goal active。

## 已导入的冻结复核证据

当前导入记录含 37 原报告 ID 行、12 条显式关系，状态仍为 5 confirmed open、24 partial、8 implemented pending validation，测试执行数 0。新增 Interface15→Interface07 只 refresh 相同 RI-CERT-P0-001 certification ID，保留二者 namespace，不与 Interface08 的语义/host-safety ledger 合并。10 个 source drift 已逐合同重新读 cited spans，状态未变化；source refresh audit 保留旧/current hash 与复核理由。

| Canonical artifact | SHA-256 |
| --- | --- |
| [2026-09-30-source-review-wave.jsonl](2026-09-30-source-review-wave.jsonl) | `8d6d68c499d52c354a992a76bc05ffb3ed41ff4ccb9abc21b1f487d3954aefc5` |
| [2026-09-30-source-review-crosswalk.jsonl](2026-09-30-source-review-crosswalk.jsonl) | `a767494fbc95ed76994efcb0b8b2660e6c0b9ddbb1fa111c35f1c1735d4db38f` |
| [2026-09-30-source-refresh-drift.jsonl](2026-09-30-source-refresh-drift.jsonl) | `faafe9af67408d41c14053224d8c445a8b80ef3a6bd81aecea770c3bd8db5d59` |
| [2026-09-30-source-review-summary.json](2026-09-30-source-review-summary.json) | `8144dd4d25a89adcfc8d1a790f00ff11636be52dc7130332ff2803d5e90cc2ff` |

导入时检查 85 个 distinct `(path, captured SHA-256)` 键，当前漂移数为 0，具体差异保留在摘要。85 个键是 78 个 source-evidence 键与 10 个 report 键的并集，交集 3；source-evidence 包含 3 个报告路径，不能写成 78 个生产源码文件。旧 61/515 路径覆盖含义、27,629 candidates、未读 nested 范围及 canonical/accepted=null 全部保留。

摘要的 producer_summary 是冻结的私有生产记录，外层记录此次正式导入；该记录不是 Cargo 或产品/性能通过证据。后续源字节变动须重新检查受影响合同与验证闭包。

## 本轮当前态核验

正式导入的四个源码审查 artifacts 已检查 JSON/JSONL 结构、37 个唯一 report-qualified ID、12 条关系、10 条 semantic refresh、summary artifact hashes、文档链接与 scoped diff；全部通过。freshness 再核验仍是 85 个 `(path, captured hash)` 键、0 漂移。该检查不执行 Rust、产品或性能门槛。

外部 zr_vm 的旧编译器终止观察已被新活动取代：本轮存在 cmake PID 393、ninja PID 492 的 GCC 作业；新 HEAD 为 `4beaaccb0c93d18dd68232fe88b5e1ef261fb8f7`，dirty status 为 242 个条目，parser hash 为 `a9659e4b2e80c71e3ebe8c53f9b14f235675e83f725fbe1762a15046ff85f982`。这是 capture 前观察，不构成稳定封存。验证 lane 正以支持的隔离输入路线处理新 pin；历史失败仍绑定当时的 `0651abb…`。

Hub 源码仍处修复窗口：legacy v1 Unknown 已保留 journal、阻止 retry/reconcile/ack，并显示明确迁移结果；该修正的独立源码复审通过。TypeScript/浏览器回归与 native Tauri/服务/DLL 验收分别记录。

## 2026-10-01 当前证据与回修

受管 Windows 命令 `validate-matrix.ps1 -Package zircon_runtime -NoDefaultFeatures -Features shipping-editor -CargoProfile shipping -SkipTest` 实际执行一次，job `0532ad4f1d634a72947a60e0c895c75d`、session `validate-matrix:01a0f03b-e8ab-7062-b208-9c3b18ec8887`、目标位于 `D:/cargo-targets/zircon-engine/pool/94e3b4a3…`。终态为 exit 1 / released，`compile_input_changed` 指向 `zircon_runtime/src/plugin/extension_registry/register/event_registration.rs`；当前 hash `23ef045354fe9e367b999d7cd4a0b704dd1ac64d580ddab9b890721255775a63`，lease owner 为 `source-audit-20260930-01a0f168`。失败发生在 closure planning，Cargo 未启动。pool 中旧的 `source-manifest.json` 是前一作业残留，不得归于此次检查，也不得据此宣称已封存当前 UI/Core。等待该 owner 的稳定窗口；原因没有变化前不重提相同批次。

Hub Web 实际结果保存于 `E:/cargo-targets/zircon-engine/hub-web-validation/astra-20260930T213924/summary.json`：两个 `tsc --noEmit` 配置均 exit 0；74 项 Node/Chromium 回归执行 71 pass、3 fail、0 skipped、0 cancelled，151 个输入 hash 前后无漂移。浏览器使用 mock Tauri/account transport，原清单未捕获四个由夹具读取的 Rust 本地化文件，因此这组结果仅按记录的输入范围使用，不构成原生窗口、服务或完整翻译闭包验收。

三项失败已追至原始夹具：两项恢复 package journal 缺少 v2 `targetMode`，一项 successful upload 后仍断言 discard revision 4，而 confirmed head 已为 5。修正夹具保持 target summary、分别验证 Editor/Client 恢复及相同 operation ID；Cloud 浏览器验证 stage/discard 使用 revision 5。Catalog 浏览器夹具同时迁移为 target 隔离的 v2 inventory/receipt，新增切换目标后独立 inventory 与安装操作。受影响批次在 2026-10-01 01:12Z 完成 44/44 pass、0 fail/skipped/cancelled，155 个输入前后无漂移；四个 Rust 本地化输入已加入清单。此次不重跑未受影响的 TypeScript 检查。

Core 私有 `RCL-P0-002` 候选首次独立复审发现 joined activation 的 `Completed(Ok(()))` 没有持有 fresh reservation。修订 patch SHA-256 为 `1f68baa83503f4cbaa248de9a1b728147498e48aebe4ab53c41796183a16ba72`，补重新 admission、保留 joined Err 和 prior-activation/dependent-latch/deactivate 回归；尚待修订版独立复审和正式归属，未应用。插件 helper v2 私有候选 `6ffef22a2e3987e88edab59ce59d877e4fcef85ce1c94c54abad57dae2b9c3d2` 与 Editor DTO 四文件 patch `4f0b8ad8016ce2cf26a510114fc0e9d293a48831be5442e75ec91c9aac821146` 已经源码复审，但当前 owner 移交仍待授权/核验。现有 Editor save 是 returned-error compensation，未证明两文件 crash-atomic。

Core 后续复审又查出 batch failure 会覆盖 already-Running provider 的成功 receipt。当前最终私有 patch 为 `8cd725921a4c8fadddccf979d1ccb27431aed1982ce3c1a5a4979705782d2b95`：`LifecycleTransactionSet::finish(Err)` 先保留 Running member 的 Ok，再赋予其他 unresolved member 失败。独立复审核对四个未变 baseline、apply-check 和四项有界 interleaving/reentry 回归，未发现剩余 P1；上段旧 revision 属于审查历史。最终候选仍未应用或经 Cargo 执行，不能关闭原 finding。

原 37-row/12-edge 冻结 artifacts 结构仍通过。2026-10-01 01:10Z 的 freshness 核验发现 85 个 `(path, captured hash)` 中 1 个漂移：`zircon_runtime/src/ui/surface/input/effect/text_services.rs` 从 `b01138648e288c12937a5229b02c35fcd3c0b7f1e4277bb616b44679edb50e58` 变为 `f7fee278acdf3a5cebd48d81a00aaf6c571d8dba7e2f17cd02f10bddae1b6830`。前文 0 漂移属于当时观察，原 artifacts 保持历史输入；新的 W3 text/IME 结论须按最新字节复核。任何上述 source review 或回执均不关闭 W0–W8。

### 浏览器布局持久证据

此前两轮返回的 `E:/cargo-targets/zircon-engine/hub-web-validation/` 证据目录随后不可读，当前无法重新打开其完整 JSON/log/screenshots；44/44 与初测数字按已取得的 tool 输出保留，不能伪造丢失的清单。已调整私有验证 helper：缓存、Vite、Chromium profile 和临时文件仍位于批准的 E 盘根目录；只把 log/JSON/PNG 复制到持久证据目录。

最新矩阵在 2026-10-01 01:57Z 完成 16/16、0 fail/skipped/cancelled，155 输入无漂移。完整清单、实际 argv、结果和 24 张 screenshot hash 见 [持久 summary](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-web-evidence/astra-20261001T014850/summary.json) 与 [实际 TAP log](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-web-evidence/astra-20261001T014850/layout-matrix.log)。覆盖 account Team、service catalog 和 license dialog 的 360/768/1280/1920、中英文；root 已实际查看 account 中文 360、catalog 英文 360、license 中文 360 与 catalog 英文 1920 四张，未见文字/控件裁切。其余图片保留并由布局测试检测 viewport/元素 overflow。全部路由、native Tauri、真实账号/安装/云服务及性能仍待各自验收。

Root 修正的三个测试文件 hash 为 `catalog_service.test.mjs` → `5f5946903b6eebacde6f8c0f4aa83392beca65b778cc2c9063bd08d08047c6b0`、`catalog_service_browser.test.mjs` → `db0f05b71e7dea7f3ada00feb4a717f7b5294ac9597e12a7c650fb69eebf4cc6`、`cloud_sync_browser.test.mjs` → `82acdccc1fc5552f0d6e81bb83963e42f5d9c4057155bd00fdb221162d048c38`。Plugin 两个测试 attribution 为 `7970439ce8294ca4b047133b53d500b8`，Hub browser attribution 为 `8a1c28e6f1314c5b9cadb332be0e0bff`。Hub Rust 新增 backend method-spy 回归实际调用 retry/reconcile：v1 targetless Unknown 保留、PackageTargetRequired、PackageClient-load/helper-query 均未调用；package/process/tests 当前候选已源码复审，但 Rust 执行尚未通过。下一批使用 `account-broker,local-service` 验证独立 Broker/Service 闭包；它不验证 Tauri Desktop 或未移交的 runtime helper。

### 第二批 W0 原报告证据

2026-10-01 02:24Z 正式导入 v5 的四个生成 artifact，保留第一批冻结字节。12 个 producer manifest 与其所有声明 artifact 的 hash 匹配；30 个不同 report-qualified ID 保留 33 条观察，Runtime02 `P1-1..3` 的三组重复观察没有丢弃。28 条 crosswalk 是生产观察，跨报告关系与状态文字保持原记录，未按相似描述合并。两批 exact-ID 交集为空，合计 67 个 ID；它们不代表全目录 canonical finding 总数。

| 第二批 artifact | SHA-256 |
| --- | --- |
| [逐 ID 核验](2026-09-30-w0-second-wave-review.jsonl) | `dccceb090592eea01f48f885ef84e53d202e6470c647b4481c44a4c23ec673d1` |
| [映射观察](2026-09-30-w0-second-wave-crosswalk.jsonl) | `d1112228744c798263611bec04f1865d4addf5f3b9bf98bba532792bdb43f2f8` |
| [有界摘要](2026-09-30-w0-second-wave-summary.json) | `f6fda8ee41837ccbedb05174212e7ed693ac64d544e3f6ee7eb19f279f662d3c` |
| [生产清单](2026-09-30-w0-second-wave-manifest.json) | `449ac7901e99fccc5839d7bd8aeee5fd85e147cb85c85270191852b6d7b11bf3` |

Root 导入前独立核对清单中的 87 个路径/hash 证据条目，加上各观察报告 hash 后为 90 个 distinct `(path, hash)` 键，均匹配 v5 捕获字节。生产观察此前唯一漂移为 `keyboard_clipboard.rs` 从 `bc1b080daf314caff910f8c8e0e133ee981972fd3e1ab84199b731218c48d655` 变为 `4e0f88c76a6d204eab5fe6012fd3ee47daf2165d574c7c5ff47cef186aca587d`，v5 已逐行复核 325–330，仍在主事务后 append clipboard effect，`RUII-P0-001` 保持 partial。90 个键包含报告与库存路径，不能写成 90 个生产源码文件。

eligible future-path transfer fingerprint 为 `34e43a84c92d258e474432aae2597610c445fbaf3d3052ec932ea99a74ed1dd9`，apply 回执 `a5c4621c1f3c41fe939d8a39708ddbc5`；四个路径获得 `child_output_allowed`，精确 claim 为 `505f8d4aea0f4b5fb404e22a3319f717`，attribution 为 `41dbe5c653cc47139679bfdad07fabf1`。没有整篇报告覆盖率晋级，tests/accepted 均为 0，61 checked /515 unread、9 supplemental、2,316 nested unread 的范围未缩减。

子代理 API 限流后 root 恢复原 Hub session 的 17 个 native candidate 路径租约（`7f9c587fa5c74111b2fe29eb5d9864ba`）及原 Plugin session 的 package 三路径租约（`3e561aa2313b41d19ad8960d89ee1020`），保留 source owner。Plugin 为 method-spy 临时目录创建修正后的 `tests.rs` hash 是 `5e6f7524db5197851476385ba46c95da94d4e3d5627ae5b5fa44f4aa01410cea`，attribution `b5e6cf944aed4703a261f06b5f5e682b`；service 的 archived 来源保持原归属。续约本身不构成编译通过。

实际 Windows 受管命令为 `cargo check -p zircon_hub --no-default-features --features account-broker,local-service --locked --profile shipping`。首作业 `4f7298b4c4e2466ebfc7815144b15231` 在编译前 JSON 元数据目录消失后以 exit 1 终止并释放，未进入 Cargo。确认终态后只将该元数据改存持久任务目录；编译 TEMP 仍由受管脚本设置在批准的 Cargo scratch。重提作业 `7a82ccfaa0e9458eaecff63cf8d6f8eb` 于 2026-10-01 02:56Z 实际完成，Cargo exit 101、validator exit 1、租约释放且存活进程列表为空。输入清单 579 条，封存同步报告 input hash `76e9a6474d1d3052c61e619998b0ff1ddb0089aef3acfc5f33ec93f21f8a6c0e`；root 逐一核对 20 个 Hub/Plugin frozen candidate 与实际编译输入一致。job 的 `source_copy_job_id`/`source_copy_manifest_hash` 为空，不用它们伪造独立 source-copy 回执。失败为 cloud staging/sync 引用 `desktop` 才启用的 `crate::projects`，snapshot/staging/sync 引用未启用的可选 `zircon_runtime_interface`，共 5 个编译错误。修复由 Hub 功能边界 owner 负责；未执行 Rust 测试，不能接受 Broker/Service、Desktop 或产品门槛。[持久终态、日志与输入清单](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-native-evidence/7a82ccfaa0e9458eaecff63cf8d6f8eb/summary.json)。

浏览器补查在 02:49Z 完成现有路由/新建项目弹窗 4/4、155 输入无漂移、12 张截图，见 [路由 summary](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-web-evidence/astra-20261001T024826/summary.json)。四宽度遍历 11 个路由，但英文分支只翻译标题/部分按钮，项目和设置字段仍有中文，因此不接受完整英文产品布局。03:10Z 的团队管理弹窗矩阵 8/8、零跳过、155 输入无漂移，覆盖四宽度、中英文、编辑/转移/邀请/撤销，见 [团队 summary](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-web-evidence/astra-20261001T030855/summary.json)。为避免仓库缓存，只在私有测试副本改变绝对导入、原 web root 与 E 盘 Vite cacheDir，断言/夹具/测试体保持一致。该轮持久 PNG 实际只有 1280/1920 的 16 张；360/768 的 4 项补拍于 03:13Z 完成并通过，新增 16 张持久 PNG，155 输入无漂移，见 [窄窗口团队 summary](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-web-evidence/astra-20261001T031252/summary.json)。两轮共保留 32 张图片，原矩阵仍是 8 个唯一测试用例。全部均为 Chromium 与 mock Tauri/account transport，真实原生与服务验收保持开放。

### 2026-10-01 04:40Z 集成与验证续接

用户授权两个旧 owner 在确认无存活执行器后取消并精确移交。最终核验发现 `astra-optimize-20260926-batch-a` 有新 heartbeat 和五条活跃文档租约，因此保留该会话，helper 仍未移交。旧 Source comment audit 没有活租约、绑定执行器或活跃构建，已按授权 cancelled（回执 `93a988a60e6f43d1ae8eb203525b324b`），源码 hash 不变，新的 source-audit 会话全部 3,388 条租约保留。Editor 四个持久化路径以 fingerprint `1a0603474b71af74d2fdc9fe69a63bd53bf31f6d428e6863f961c84e6ca02aef` 正式移交并集成（apply `2915e1baf1ae4053a92b309686a77b96`、attribution `90f25905442146cb97dcce0c543e1f4d`）：workspace v2 增加逐视口状态，v1 显式迁移补空表，欢迎会话恢复接入该表。原生窗口测试 fixture 另按 archived owner 精确移交，已更新 DTO，hash `b7fd128b2cbf5e16a58f18a95fa91610467504bb7b2a360def901daef46182dd`。格式和 scoped diff 通过，Rust 回归、保存重开及真实双视口产品门槛尚未执行。

Hub 完整 native 字面文本/设置/页标题/模板投影，加长路径的浏览器矩阵于 03:50Z 实际执行 17 项，13 pass、4 fail、零跳过，160 输入无漂移：[失败 summary](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-web-evidence/astra-20261001T034255/summary.json)。窄窗口复现确认 engine/user 入口被隐藏、英文面板按钮越界；表格行菜单位于合法横向滚动区，新增实际滚动、打开菜单和焦点恢复检查。Root 已正式移交 TopBar/HubPanel 并以 flex 流和紧凑入口修复。参考 [Navbar 3](https://pro.reactbits.dev/docs/app-ui/navbar/navbar-3)、[Navbar 6](https://pro.reactbits.dev/docs/app-ui/navbar/navbar-6) 和 [Data Table 8](https://pro.reactbits.dev/docs/app-ui/data-table/data-table-8)，实际查看 iframe DOM、flex 关系和独立控件；保留表格自己的横向滚动 owner，不复制参考源码。04:30Z 两个 TypeScript noEmit 和 360px 中英文四项测试通过，160 输入无漂移，实际 30 张 PNG；helper 错把不存在的 project-browser menu 纳入 32 张预期，summary 仍如实标记整批未通过，已修正预期为每宽度 30 张，并用当前源码执行四宽度完整矩阵：[窄窗口 summary](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-web-evidence/astra-20261001T042725/summary.json)。浏览器 transport 保持 mocked，不能替代真实 native/service 验收。

Hub feature boundary 的独立 source review 无剩余 P0/P1，已直接 LF 集成三处 desktop gate，`cloud/mod.rs` hash `e7516acc73cb4debe419475b35b700c74284947604b076049efc8c844ab8cab9`。重提 attempt3 在 04:32Z 以 exit 1 终止，request `bbe10efccc5f440caa53a9ae324f78f5` 的终态为 `unmanaged_artifacts_detected`，唯一路径是本轮 `E:/cargo-targets/zircon-engine/hub-web-validation`，Cargo 没有启动。该缓存目录虽在批准磁盘根目录内，却未登记 fixture；后续使用协调器已有 fixture-acquire/release 路线，保留持久 log/JSON/PNG，当前原生检查与 Rust 测试仍待执行。没有修改或绕过受管 gate。

### 2026-10-01 06:12Z 增量证据与待验收项

W0 第三批已正式导入，包含 15 个报告限定观察行、3 条显式 refresh 关系：Runtime211 的 `Runtime155-P1-001..009`、Runtime163 的 `P1-3/5/6`、Interface09 的 `RHOST-P0-001..003`。与前两批无 report-qualified ID 碰撞，三批共 82 个有界观察行，并非全目录去重后的 finding 数。导入前对 21 个唯一报告/源码路径的 44 项证据核对通过。各行仍为 `unreviewed`、`not_accepted`，tests_run=0；61/515 和 nested 未读边界不变。

| Canonical artifact | SHA-256 |
| --- | --- |
| [第三批源码观察](2026-09-30-w0-batch3-review.jsonl) | `4b4137f6ef0feeb0fc4ef6f55664a80f3c8a14473c0f94ab4361d85728a68fbe` |
| [第三批 refresh 关系](2026-09-30-w0-batch3-crosswalk.jsonl) | `952396c0c146b947094dc9ec1fdf707789e928d0da69c32ae316845571372cb1` |
| [第三批摘要](2026-09-30-w0-batch3-summary.json) | `e56a25f936d274995912e3ca6722fc3def274fc01a78ca1f85caf1f7f916d1ae` |
| [第三批清单](2026-09-30-w0-batch3-manifest.json) | `73f304162d51d9f323a11d563750deb8e3c8e41c6ba28aa157959ee512f1715a` |

第三批 apply `07716f4d017842658105060b25e77511`、claim `e10ad81bceec4751b29280191df109ca`、attribution `9be4f937cac54fa2a323b22fadcc34ee`。第一批历史 37 行再次 freshness 检查发现 `surface_lifecycle.rs`、`action_requests.rs`、`text_services.rs`、`bounded_json/writer.rs` 四个源路径漂移；保留历史捕获，不沿用旧 rationale 改写 hash。

Hub 修复后的四宽度、中英文、长路径、11 路由和 native 字面文本投影矩阵于 04:44Z 完成 **17/17**、零失败/跳过/取消、160 输入无漂移、120 PNG：[完整浏览器 summary](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-web-evidence/astra-20261001T043630/summary.json)。TopBar/HubPanel 独立源码复审无 P0/P1，报告 hash `d8387dff86998ce104330bba04c9121569ab17c04b4e65539e9f4c893a64326c`。后续浏览器缓存改用 coordinator fixture；360px 补查 4/4、30 PNG 后自己的缓存已清理并 release。Mock transport 的浏览器证据不关闭真实 Windows/service/performance 门槛。

attempt4 在 Cargo 前因 foreign 未登记 fixture 被拒绝，未删除外部产物。之后支持的 artifact audit `c390b34d9a5c48ccbca36e596d38a173` 返回 unmanaged 空集，才重提 attempt5。作业 `def7b07fadc748f58a90e3f0ebd65023` 实际 Cargo exit 101、validator exit 1，终止释放且存活进程为空。579 条输入，input hash `dac05fe6cf8a78781bc8d518ce0300772eda3712af02c02bb1d194663f8df7c2`，20 个 frozen native 候选均与编译输入匹配。原五个 feature 错误消失；七个新错误来自旧 `self.policy` 字段、非双向的 `Ancestors` 和四处不稳定 Windows metadata 方法：[持久 attempt5 证据](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-native-evidence/def7b07fadc748f58a90e3f0ebd65023/summary.json)。

三个最低层根因已修复：使用 `policy_index`，收集祖先后反向遍历，新增 handle `FileIdInfo` identity 比较 volume 和完整 128-bit file ID；保留目录身份检查及 OS 错误，加入同内容不同文件/硬链接回归。四路径封存 patch `fd326a8c6aa411f98d52ce90e829c6f2dfc7a3f989cc3d2082cba0e8b96f424c`，格式及 scoped diff 通过，独立复审进行中。Hub attribution `1e5aac0706ff4e07b0ebaf6fa3ebbc32`、Plugin process attribution `3ee12cdf1b6b4c639cdafcd94dd85143`。attempt6 于 06:12Z 启动同一受管无桌面 Broker/Service check，21 个 native 候选已冻结；回执与通过证据尚未取得，Rust 测试未执行。

05:47Z 最终核验旧 helper owner 的 heartbeat 未再变化，租约过期且无执行器/构建/绑定；按用户已有授权 cancelled `astra-optimize-20260926-batch-a`（`104ef68154914761a66358b5e64a2be3`），129 个源码 hash 不变、新审计会话 3,388 条租约保留。helper 精确移交 apply `d82abb83812d4f58bf6973ee5b5b2ce6`、claim `435c0010e6f74d5bb816161f2da2b047`，06:09Z 集成已复审 v2，仅额外整理导入排序；当前 hash `f0a1c90f07f2de2f29daf1a578bf65e397cc131f4835264fc635bd09206533a9`，attribution `fdd8f2ec24f4459ba597b12e5a3b2bc9`。Helper 按 typed target 读取 pinned policy index、commit 边界复核后发布 installed selection；其 binary/Rust 回归、Editor 消费者、真实 DLL/Ready 和产品门槛仍待完成。

## 2026-10-01 受管测试与精确续接

Hub 无桌面 Broker/Service shipping check 已取得实际通过证据：作业 `f03235330d1440ebaf9c8c66b148063f` released/exit 0，input hash `f082d0e295124275ae9b8a02815bb1a036be7b9fda048cc2a0df17396b02164d`，580 条输入与 21 个冻结 native 候选逐一精确匹配；见 [持久 check 证据](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-native-evidence/f03235330d1440ebaf9c8c66b148063f/summary.json)。这不包含测试或产品验收。随后全 Hub lib-test 批次 `30983fe2175a4639902ca8e325fd5b66` released/exit 1、无存活进程，input hash 相同；先决 `cargo check --tests` 共 24 条错误：11 条 lib-test 编译错误（常量导入、Unix libc、native 参数比较、私有 journal 状态）和 13 条未启用 Desktop 的集成测试错误。实际测试执行数为 0；保留原日志，修复最低层后只重跑受影响批次。

共享 TopBar/HubPanel 修复后的 account/catalog/license 浏览器矩阵 `astra-20261001T062204` 实际 16/16、零失败/跳过/取消，158 输入无漂移、24 PNG；见 [当前矩阵证据](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-web-evidence/astra-20261001T062204/summary.json)。受共享样式影响的 Team 管理矩阵 `astra-20261001T101926` 已实际 8/8、零失败/跳过/取消，158 输入无漂移、32 PNG，覆盖 360/768/1280/1920、中英文与编辑/转移/邀请/撤销；[当前 Team 证据](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-web-evidence/astra-20261001T101926/summary.json) SHA-256 `108429dd4fe6ec12b2075a7e912f89e0427defba7f8a359f6c0073fc1844edaf`。Root 查看了窄窗中英文与宽窗实际 PNG；夹具已清理并 release `97ed3cca148544c49c15dc01aa355418`。这些是 Chrome/mock transport 证据，真实 native/service/product/performance 门仍开放。

第一批历史 37 行再次结构检查通过，85 个捕获 path/hash 中当前 6 个源码路径漂移：`surface_lifecycle.rs`、`input_routing.rs`、`action_requests.rs`、`text_services.rs`、`text/service/projection.rs`、`bounded_json/writer.rs`。原 JSONL 与 rationale 保持冻结，不将新 hash 冒充旧结论；W0–W8 不因此接受。

Penpot UI timer 按支持续接澄清使用自己的原 SID：normal heartbeat `835e2940c50240e8b709d726ec1b8296`、exact claim `0c3750388a104c509ba077c6b2217563`、attribution `613791ee7339460ca653828c3ce3a71b`；只扩大两个生产 delegate 到 session scoped visibility，postimage `135ab40c59f9e1603059c7fff371bce53ee8c2e746539d797739f4f56cd3bd35`。自己的受管副本 `bfcb2a33ced949448afe66a394a35cfe` 在 closure_planning 返回 `validation_copy_compile_time_resource_missing`，未执行 Cargo；当前 foreign 测试的字面资源存在，不能把已移除 planner 字节的错误归到当前源码。[精确一次续接回执](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/penpot-ui-own-sid-continuation/continuation-terminal-handoff.md)。三条原 Failure 生命周期保持 open，原 build/tickets 未改动，无重复导入。

## 2026-10-01 11:35Z Hub 实测失败回修与独立实施

原 24 条测试编译错误由五个准确路径修复，Desktop 集成测试只在其真实 feature 下准入，原错误日志保持冻结。作业 `e2708a5ba4e44baa8f3a1e2bd855c7e5` 已 released/validator exit 1、Cargo test exit 101、无存活进程；`cargo check --tests` 实际通过。输入 hash `3bbfde6c7a425c79e0e6552821fc168364b7a0d2a0cae341c3719961734f3a6b`、580 条输入、23 个 native 候选全部精确匹配。实际执行 **172** 项、**171 pass/1 fail**、零 ignored/filtered，310.94s；[完整失败终态](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-native-evidence/e2708a5ba4e44baa8f3a1e2bd855c7e5/summary.json) SHA-256 `7c9a713de144e3bd467636a585ab7e1f3da2738fc2644991e08e1ec2f9de259b`。不将这个失败批次记为通过。

唯一失败 `account::tests::logout_cancels_login_waiting_for_the_initial_state_lock` 在原 `account/mod.rs:702` 超时。测试释放状态锁后只等待 authentication，未继续轮询负责在状态锁内发送取消信号的 logout future；独立复审确认是测试驱动缺陷。生产 state-lock→cancel→operation 顺序保护 package commit 授权，保持原样。现有测试改为在原一秒上限内 `tokio::join!` 同时驱动两者，保留 Cancelled/登出成功断言，新增锁边界、无 subject/凭据及唯一取消代断言。原 SID `astra-hub-sync-20260929-01a0f033` claim `1629ebfd67644d5c976d4835862345f5`，postimage `7963e6944ab45c45e0ae748d38899b916013959833344c3082e61b195b2cd459`；[准确集成归属](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-logout-test-driver-repair/integration.json)。受影响的七项 Broker 测试以 `account::tests::` 提交一次受管 Windows shipping 回归，冻结候选 hash `a61cd98775350beaeef7518dae27120af55a6e6ec46993c2a0ec7c802c98516a`；原请求已在 Cargo 前准入拒绝，精确终态见下文；尚未执行七项回归，不重复整批 172 项。

LIFE-A1 当前 `engine_task_graph.rs` hash `9f62e5153870ca6555ff466665d5335560c48dcc1c72226cba12728fb8896528` 已保存 Closing 强引用 scope 集合及最终 stopped census，现有测试 `astra_life_a1_shutdown_retry_and_repeat_preserve_scope_census_after_handles_drop` 的文件 hash `c60b8a2c380f8c5856f5c161c35d3f02ba623ba5797c9c482010bbcb10cf3696`。这纠正计划里的旧 Weak-only 描述，状态仍为源码已实现、受管/产品待验证。另一个原 finding `RCL-P1-006` 的 module best-effort cleanup 已形成十路径私有 r2：保留首个错误、独立模块继续清理、Stopping dependent 阻止 provider 卸载、deadline 列出未尝试项，并保留 session 外围任务未 quiescent 时的边界。Patch hash `0deac4c7f110f691055d03659a6b7e2cfd9f82459b835f3e5fe36a99d2c9fd2e`、manifest hash `144ad0c54284fc8a41618842039ca59aba00e51bee3de158afb54f1cc394a080`；仅私有格式与 apply-check 通过，独立复审及准确 owner 移交待完成，不据此接受 LIFE-A1、LIFE-A4 或 RCL-P1-006。

## 2026-10-01 Hub 准入终态及私有修复组合

原七项 Broker 请求 `8a32bf3d11354f35a67341761e0383fb` 于 11:39:30.098691Z terminal failed，错误 `unmanaged_artifacts_detected` 指向 `D:/cargo-targets/jenkins-pilot-01a0f6e1`。这是 Cargo 前准入拒绝，未封存新 Cargo 输入、未启动测试；原 172 项的 171/1 结果保持不变。[原请求持久记录](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-account-tests-attempt3-command_requests-terminal.json) SHA-256 `de9d0637c6102d5a772221040b2bbab957bfb20af0624863a1e5659b62523968`。外部目录、产物和 live owner 保留。

当前源码复核确认 package 进程把原始 root identity 与绑定 typed target 的 journal owner 摘要比较，两者属于不同摘要域。私有候选保留两种身份，并让进程比较原根身份、持有 root pin 到进程 scope 结束。另一个候选让根身份复用已有 `FileIdInfo` 读取器；历史 journal owner 不被静默接受，显式恢复设计和实际重启门仍待完成。云恢复候选保留 project/private/target-parent handles 到扫描、有限读取和同一已开对象删除结束，保持原 per-file 限额；r3 持有同一个 stage anchor 到 marker/payload 校验及临时文件清理结束，并加入两次读取之间等字节目录替换回归。

[14 路径准确私有组合](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-private-repair-combined-r2/manifest.json) SHA-256 `4dbcd8b46080128b9a28d162070da8223ad979e5c0bf8cefe7f533348ef8bdbc` 合并了三份冻结源码包和重复文档追加。仅复用准确 Rust 格式证据及组合 `git apply --check`，不构成编译、测试或 source consent。原 Hub/Plugin SID 归属保留；协调器连接拒绝时无共享集成，独立复审、受管原生回归、真实 Tauri/Keycloak/安装/冲突恢复和性能门保持开放。

Root 后续 Runtime04 回执明确：原 UI failed copy 基于 Git base 的测试使用旧资源字面路径，当前测试已有新路径。同一 Runtime04 owner 的 single-path snapshot 仅封存该测试，编译 spec prepared 且未提交；Resource producer generation 仍需自身闭包。[依赖终态只读核对](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/penpot-ui-own-sid-continuation/runtime04-dependency-terminal-observation.json) SHA-256 `94ecf5b8f995d6a2c17015072f3ce2757036a6c8e159215d369c157dc90e4de5`。此证据不重建历史 immutable seal，不扩大 UI overlays，不重复 import/validation；原三条 Penpot lifecycles 与 lower/original/upward 门保持 open。


## 2026-10-01 W0 第四批、Hub 源码集成和 Core 组合

W0 第四批新增 24 条 report-qualified 源码观察：Runtime167 Physics、Runtime169 Navigation、Runtime221 Animation 和 Runtime173 Network 各六条。逐字 namespace 保留，前 82 条观察无碰撞；四批共 106 条有界观察，不是全目录去重 finding 数。14 条关系来自明确报告文本，`id_merge=false`，不以共享主题授权合并。状态为 20 `confirmed_open`、4 `narrowed_open`，`tests_run=0`、accepted=0；四个报告仍未完成整篇语义审查，61 checked/515 unread 与 nested 2,316 未读文件边界保持不变。

| 第四批 canonical artifact | SHA-256 |
|---|---|
| [review](2026-10-01-w0-batch4-review.jsonl) | `9ca9bc4ea5b83865c219c8fb9b3c7ac73d7d078460bf8b92c6cded896996fb2c` |
| [crosswalk](2026-10-01-w0-batch4-crosswalk.jsonl) | `1fe60a3d24cd1998f2052fafc8e9a50eaa5c3091ae60df79b16016d340c85c9c` |
| [summary](2026-10-01-w0-batch4-summary.json) | `e68ff8229e494d57fa33b02cda2268ac77cb9c7b0d28e4bd49a422ab888e7685` |
| [manifest](2026-10-01-w0-batch4-manifest.json) | `2036a604804c94a68902aa17a081847ab994196af57a246757139c01a8b11d6a` |

四条过期报告已按当前源码缩小：Physics runtime catalog 有注册但 editor composition 未闭合；Navigation 无输入返回 empty+warning，未调用旧 simple-surface bake；Animation reverse sampling 已存在，剩余 cursor identity/seek/notify；HTTP/WS 已解析 root manager，RPC 仍构造 private manager。Network worker ingress `try_send` 丢弃与下载 partial/resume/cache 内存状态另有准确行证据。原报告 ID 与旧正文保持原样，未接受这些能力。

Hub 恢复 r3 六个 Rust 路径在原 `astra-hub-sync-20260929-01a0f033` 下精确集成：三个 future 模块经 preview `4f2d701a1d8d4966937493b928ea5a92` / apply `f06ab2a27bbb439b9f9f1bb8d5513027` 纳入 scope，exact lease `f0d64c4ea3bd4f12afe03796a1c2b3be`，正常 attribution `6a7244b6614b43d194f2066cd074fcef`。`staging.rs` 为 `c282d0970e68a6bbefef82c29a23f946cb5a0922d5b209d89de119800888f71d`，anchored tree 为 `29dd88c1943fff188b8a5a35dcc612f25b70b93354da77cdf9032c2287e0044b`；完整六路径哈希见[精确集成回执](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-recovery-anchor-repair-r3/owned-source-integration.json)，SHA-256 `4835ab2a9f4453af6bd9bf7efdf177fb7e918366cb212f6551b633221e88f384`。独立复核[当前生产字节和归属](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-recovery-anchor-repair-r3/production-confirmation.json)，SHA-256 `a631749686e28cc41124f8d9d9ac16fe8400f991762b4eacf854612c80208578`。这是源码集成证据，实际测试仍为 0；native recovery、original Hub 回归和 desktop restart 门保持开放。

Hub 三份私有候选的独立复审 C 已封存，JSON SHA-256 `8531d7e135a57fec0581435c865f5a9f7b79458e46dd0f7ee28c33dc68a7cf40`。唯一 P1 `C-P2-001` 是 full FileIdInfo 切换改变持久化 package owner，旧 Unknown journal 无显式迁移且同 operation retry 冲突；原安装实施线继续修复。package/root/full-ID 与 foreign 部署文档尚未集成；历史 14 路径组合和旧 acceptance 记录保持冻结。

当前受管 Cargo 产物门：原 `artifact.audit` 请求 `9b75bbd9d23142f0a49ed28c54c8c3b6` 经持久记录核对为 completed，报告六个未登记目录。三项为 `D:/cargo-targets/mvp-test-fixtures-40872`、`-5728`、`-6132`，另外是 Penpot coordinator bootstrap、`E:/cargo-targets/coordinator-runtime`、`E:/cargo-targets/engine-audit-tools-01a0f06a`；外部目录和有效 owner 保留，没有清理、重提 Cargo 或将前次拒绝记为通过。[原审计只读调和](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/root-current-wave-admission-20261001.json)，SHA-256 `62c2e754a88001dc47ecf26d14506444b6066365ef3373d5400f147856e4749f`。

Core `RCL-P0-002/003`、`RCL-P1-006/008` 的[20 路径私有组合](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/core-lifecycle-combined-r2/manifest.json) SHA-256 `f6f1c56b40ac38a25eb261a51a9642b2a3c6339e15f5d6fb83c75a9d0bbb209c`、patch `dd6f4267853bd74b6c566e3677a4d9b7c5bfe929da3c2772ea78e6782290bacb`。共享 activation/retirement/ready/shutdown 路径组合后保留当前 foreign event-bus API `runtime.rs` hash `2b9d6764e7369a22a522e27353c5c3391e03fac6bbc2c1f45dc7259cb93c5eef`。测试修正了冻结图后的注册顺序，并覆盖 failed activation/reactivation/batch reset 的重入析构；格式与 apply-check 仅为源码检查。独立组合复审、原 source owner 准入、受管回归和 Windows 产品/性能均未通过；共享 Core 源码未修改。


## 2026-10-01 20:42Z 组合源码、实际前端回归与剩余生产入口

本增量由原 W0 SID `astra-goal-20260929-01a0f033` 维护，未新建 Session、failure 或 Cargo 批次。
本节引用的候选保持私有；已有归属和前述真实测试结果按各自准确输入保留。W0 四批仍为 106 条有界观察，
576/61/515 库存与 nested 2,316 未读边界不变；没有新增 canonical accepted finding。

| 实施线 | 准确私有产物 | 当前检查与剩余合同 |
|---|---|---|
| Hub package lock / 旧 owner 恢复 | [38 路径组合 manifest](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-recovery-lock-combined-r1/manifest.json) `edac2fc1738b506f025ffbdc19549bf5d0974fbd1fe753180290ab15144fbc17`；patch `743777795be009affd4f074f3e8b3d040bc2fa2cbfbdfffa55782050de92be67` | 真实 native project-lock producer、空 selection 的 policy/generation 复核、60 KiB canonical lock 上限、完整 FILE_ID_INFO、旧 journal 原字节迁移和同 operation Reconcile UI 已组合。原生包锁复审中提出的两项 P1 已形成源码修正；完整组合独立复审与 Native 原 owner source consent 尚未完成。未执行 Rust、helper 或产品验收。 |
| Navigation World 发布 | [21 路径 manifest](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/providers/nav-world-guard-r3/manifest.json) `4e7355162c51aaba419cfb6b1e036c22e278daf53ecad5e1964110585a7068ac`；patch `871b66146b1cacbb5ba19c4a8e99a165a4a37352a214a03cf33f943a4b184422` | 同步、tiled、dirty harvest 使用原 LevelSystem 的 weak live owner；generation/replacement/lifecycle epoch 与短 manager publication 同受锁保护。卸载／重载 ABA、World 编辑后 harvest、有限 collider 原子附加等 14 项 lower 测试列入清单但执行数为 0。旧 operation BakeScene/BakeSurface 的 pure prepare backend、triangle mesh/heightfield/volume provider 仍开放。 |
| Asset 通用 component 持久化 | [44 路径 v4 manifest](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/asset-scene-2d-persistence/generic-persistence-manifest-v4.json) `88b688efb45406241db5fd340c06926d1eb15ae32ee93049a6cfb69bea0b94fb`；patch `995367e9019b3b8f4dd2c4f11924a48bbc1b4bdf99644c501e0703b60beb79b3` | 原报告 G07/Runtime109 要求的生产 authority 尚未闭合：普通 `scene_asset.rs` create/save/load 仍重新创建 builtin registry，第三 provider 测试显式调用私有 registry 入口。已向原实施线返回准确调用点，要求正常生命周期注册和普通 save/close/reopen 回归。Root 核对的 44 个 baseline/current hash 匹配，43/44 candidate hash 匹配；一项封存后 source drift 已单独保留，待一致的新 revision。Rust 执行数 0。 |
| Editor 逐叶路由 | [20 路径 r3 manifest](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/editor-toolbar-candidate-r3/toolbar-baseline-candidate-manifest.json) `107d9fd158fff430505ca8475150fd18ea1ef466096db187ea936f7fd9c454b6` | committed ViewInstanceId 经 ForView journal/executor 传播，Scene descriptor 允许独立叶实例，真实 Workbench open/split/toolbar 回归已加入。所有测试未执行；SplitClone/restore 的实际 session seed 闭包和 Scene/workspace durable transaction 继续实施，尚无保存关闭重开验收。 |
| Core lifecycle | [20 路径 r3 manifest](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/core-lifecycle-combined-r3/manifest.json) `2dbc3caf0b36f43817457f1f2ea36d0f9d05099426e97a081b1084875b8e884f` | r3 修正 first-error fixture 的注册顺序，增加正 Duration wrapper 的最后析构超时后重复 cleanup 回归。沿用未变更 shutdown 子集的复审证据；完整组合复审因 429 中断，[精确中断记录](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/core-lifecycle-combined-r3/independent-review-d-interruption.json) 保持 open。未执行 Rust，未移交 foreign Core 源码。 |

Hub 真实前端验证限于 [准确复用回执](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-owner-migration-ui-r1/validation-closure-r2.json)，
SHA-256 `9adcb8827b7eec88d1286f8e66d2c91c682f752e7c0d17e71a8c00e6fb19d166`：
composed TypeScript `--noEmit` exit 0，实际 controller/SSR 回归 **8 pass / 0 fail**，覆盖同 ID 的显式目标恢复、
目标/receipt/旧 owner 校验、logout stale response、重复操作 fencing、typed protocol 与中英文无猜测目标 UI。
首次 SSR 在语义测试前因缺少实际 assets 目录失败，原日志保留；补齐准确 assets 闭包后执行八项回归，
未重跑不受该输入变化影响的 TSC。此结果不代替浏览器交互、Tauri、Rust/helper、Keycloak、云同步或 release 性能。

Hub 正常 preview 原请求 `c4dacc81d4834e3a9bcb6c148823a4e2` 于
`2026-10-01T20:15:06.268846+00:00` terminal failed，错误 `command_execution_interrupted`，
服务重启前未生成持久 preview result。原请求只读 GET 的终态已保留于
[准确终态](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-recovery-lock-combined-r1/hub-admission-terminal-interruption.json)，
SHA-256 `fadaf0ad14708d9e3badd984c88ab1eded287c425cea854cb94fdd9df9f05c47`。
没有 transfer apply、source attribution、部分 Hub 集成或重复 preview；实际 Native 绑定／source consent 门仍独立保留。

Asset 的 [Root 精确核对](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/asset-scene-2d-persistence/root-verification-v4.json)，
SHA-256 `d36002fc02cbc0cfcfce96c359866eb1e80e854676737e4b36440bc469da7f09`，
记录严格 `git apply --check --whitespace=error` exit 0 只针对冻结 patch；
不把有 drift 的 candidate tree 或未闭合的正常 provider 入口当作已验证输入。
所有 private 原版和后续 revision 的原 bytes、comments 与 hash 保留。

原 UI timer、RG166 与 Native 三条 Penpot lifecycle 继续 open。
Runtime04 single-path snapshot 的依赖闭包说明、尚未提交 compile spec 和 Resource producer generation 门沿用前节；
原 lower/original/upward tests、同一次 managed build-editor 发布和 live Penpot Editor/source/token/font/media/locale 门仍未通过。
未重复 import、validation、route load，未产生 monitor/wake/quiet window、external zr_vm hold、commit、push 或 WeCom。

## 2026-10-01 22:13Z 前端交互、W0 第五批与消费端修复边界

本节只记录准确输入上的增量，所有 Rust、native helper、真实产品和性能门仍按原范围开放。
原 fixing SID、source/comment provenance、leases、tickets 和失败证据保留；没有新建 Penpot
Session/failure、重复 import/validation/route load、外部 hold、monitor/wake、commit、push 或 WeCom。

### W0 第五批

新增 24 个 report-qualified 键：Runtime139 `AUD-P1-001..006`、Runtime174 `RT-AI-P1-001..006`、
Runtime171 `RT-PFX-01..06`、Runtime172 `RT-TER-01..06`。Root 复核全部 106 个先前键，
新键无碰撞；五批共 130 条有界观察，不是全域 canonical finding 或 accepted 数。
25 个源码和 4 个报告 current hash 匹配，43 个源码 span 范围有效；21 条关系保持
`identity_merge=false`、`id_merge=false`。Audio 两条为 `narrowed_open`，其余 22 条为
`confirmed_open`，Rust 执行数与 accepted 数均为 0。

原第五批错误地把库存的 61 个报告路径加上 25 个源码路径。独立 r2 纠正单位与分母，
保留原始四个产物：库存仍是 576 个报告、61 条既有 checked 路径和 515 条 unreviewed 路径，
各报告的整篇语义审查没有因此完成。历史 bounded row 的源码 evidence 路径并集为 222，
不表示这些旧捕获目前均新鲜或通过验证；nested 2,316 未读文件、69 无编号报告与 558 手工
anchor 仍开放。原 collision 检查遗漏 `original_report` 字段的 37 个键；r2 已准确检查全 106 个。

[W0 第五批 r2 manifest](2026-10-01-w0-batch5-manifest.json) SHA-256 `1a38ea4fe7233e36646fa3ee98536eb6e7ebebcaff3ee2676bb8e754bc991a5d`，
[r2 summary](2026-10-01-w0-batch5-summary.json) SHA-256 `4e24aac70b4d1d6dfea15969f7b17e89be04ef430273c8ccf0baffcbdbac8b72`。
review 原 bytes SHA-256 `4bd6dcae8255f6d770f06cb471a1ead80b64be2c2038ed5705f256dff6a1b7cc`、
crosswalk `69a0205f8baa42ad5cd16dc6fee176c8fa2308b441b484647ef3bafe30d9cae0` 未改。
新四路径已通过原 W0 SID 的正常 preview/apply 纳入 scope；本次仅发布原精确映射记录，实施与测试 acceptance 仍开放。

### Hub 实际浏览器交互

生产 TS/TSX 输入未变化。真实 Chromium 在 composed Hub 页面执行 10 个唯一交互场景：
8 个恢复页面场景覆盖 360/768/1280/1920 与中英文，另复用同一输入上已通过的
missing-receipt retry 和 history-error 两个守卫场景。8 个受影响场景的新执行结果为
8 pass / 0 fail / 0 skip，保存 32 张截图；先前两个守卫为 2 pass。
缺失 target 必须手动选取，已有 target 只读，同 operation ID 恢复；busy 阶段拒绝重复提交，
缺失 receipt 保留未知项，历史读取失败禁用写操作。

原 r1 的八个页面场景因测试读取 MUI 零宽 placeholder 而在第一条空值断言失败，原回执和
完整 harness 保留。r2 改为检查实际 input value 后只重跑这八项；没有更改产品源码或重跑
不受影响的两个守卫。Root 实际查看 360 英文缺失目标、360 中文既有目标、1920 英文 busy
三张截图，不声称逐张查看全部 32 张。[准确浏览器覆盖回执](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-owner-migration-ui-r1/browser-closure-r2.json) SHA-256
`927aaeca6c73d13bace214ed20fe9b52d5b842f4c0e6ea8f17ade80fa87e2901`。
浏览器受控 IPC 不替代 Rust、原生 Tauri、Keycloak、安装、云冲突或 release 性能验收。

### 独立源码复审与实施

Navigation 21 路径 r3 的有界独立源码复审未发现 P0/P1；[Nav review](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/providers/nav-world-guard-r3/independent-review-d.json) SHA-256
`e1880c6d260cedc44b68ca6c5764b1af58eb4a3874abc23a76d1757a6ab63484`。
原 14 个 lower regressions 没有执行，正常 operation BakeScene/BakeSurface 的 pure backend
仍在原实施线续接，triangle mesh/heightfield/volume provider 和产品门保持开放。

Core r3 的 9 路径 shutdown/consumer 子集独立复审未发现 P0/P1 源码缺陷；[Core shutdown review](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/core-lifecycle-combined-r3/independent-shutdown-review-f.json)
SHA-256 `46e071ec1472ae6a3c5f30753733bbbde2501cdc3f45e6ebb76ba40dc000bbf2`。
P2 缺口是 dynamic Session 未直接证明 incomplete module report 阻止后续 teardown 并保留
retry ownership；对应新私有回归正在补齐。完整 activation/tests/combined coverage 未闭合，
上游中断不是审查通过。[Core 原归属只读核对](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/core-lifecycle-combined-r3/owner-source-boundary-20261001T2134.json) SHA-256
`7f72adc49d54f5a97dabefd08bb013d2090721d6a4302732ccad1719099f524a`：
`core/runtime/runtime.rs` 的活跃外部 owner `01a0df1a-f2b0-7480-948f-ccc4ba982d12` source
consent 未建立；UI `session/state.rs` 保留原自己的 SID。没有移交或半份 Core 集成。

WOC 普通 transaction 原先以 saturating successor 在 `u64::MAX` 重复进入同一 tick。
[WOC tick exhaustion manifest](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/woc-tick-exhaustion-r1/manifest.json) SHA-256 `f5a86d61948183ac01ad2be6bc8215ece235769b09a6951a9aebd46016087685`，
patch `190bf6ce5ca64766870130565d272c1c496a933c541d493a0f8f85b867c151f3`：两路径私有候选
先检查 successor，在 encode/checkpoint/VM 前拒绝，保留 committed snapshot，并沿用
Offline/Server/Client 终态。两项行为回归验证最后合法 tick、VM/投影不进入及重复终态。
此为原 App03 `WOC-APP-P0-006` 的有界低层消费者修复；Runtime24 `IDENTITY-P1-025`
只列为相关合同，没有创建新 Failure 或合并 finding。Rustfmt/apply-check exit 0，测试执行数 0，
独立复审、原 source admission、真实 ZrVM producer 闭包、lower/original/upward 与 Windows
WOC 产品/性能门仍开放。

Sound 最低 activation 层的旧实现先 deactivate 再 fallible AudioManager::new，会在候选设备
失败时损失旧实例。实际配置与设备切换的 private prepared-owner 修复继续实施；尚无冻结
终态或动态设备证据。Runtime139 的配置/target 条目、Runtime218 provider/device 合同和
Hub 原生包 owner 仍各守自己的来源，不能以其中一个源码修正接受整个领域。

### 正常协调器终态

r4 两份文档的实际 CRLF postimage 保留：`1354bf1668f32502461bf3dcc47d19ee59359b3fad0e1ace5cebdd1dd95892bb`
和 `f5ecbcad3a746b5e0ace4ac260fc6e8dc8489114b69014d4b93622b58ad9969a`。
原正常 `baseline attribute` 请求 `807fdc7028a047d5b28e08f10f0af6db` terminal failed/internal_error。
只读 DB 虽观察到同 SID 的 matching postimage attribution，失败命令终态仍未通过；
没有重放、手工改 DB 或把局部效果当 completed receipt。[r4 attribution 终态](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/astra-current-plan-receipts-20261001-r4/attribute-terminal-boundary.json) SHA-256
`9ac652491013e024129fc167940e2bc647ace252bb4821743f92128825b5f735`。
原 r5 私有候选保留；本次六路径增量使用独立准确 postimage，终态 attribution 另需正常回执。
既有六个未登记 Cargo 目录门、Hub preview 中断、Native source consent 及 Runtime04
compatible Resource producer 闭包继续开放；没有重复提交 unchanged validation。


第五批四个 future path 的正常 preview `9badcb0486d74e26bf525f4018b5b9d0` / apply `5a318d80811647e9a4f0b882a25c0c40` 已完成，
只扩展原 W0 Session scope；没有外部 owner、lease 或 status 变更。记录写入与归属命令的最终结果单独保留，
不覆盖 r4 失败终态，也不升级 finding、Rust 或产品 acceptance。


## 2026-10-02 01:30Z 精确源码与未通过门

W0 前五批仍为 130 条有界观察、130 个不同 report-qualified 键。第五批正常 attribution
`b2c67b758029414c93fa4f0244d857fa` 已 completed，见 [第五批终态](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/w0/batch5-publication-r1/attribution-terminal.json)。
第六批原 review 实际只有一个换行、零行；r2 虽有 18 行，但 14 个 provisional_key 错指其他
原 ID/报告，摘要也把 36 个证据路径写成 19。两版均未导入，原产物保留，原作者继续不同 r3。
报告库存 576/61/515 和未读边界保持历史含义，不把源码路径或刷新观察算成新报告验收。

WOC App03 `WOC-APP-P0-006` 的两路径精确修复已在原 W0 SID 正常移交、落源并归属：
transaction postimage `1f81e7b0875c0facb1bc1d1132d533a246dae61b99be472ccbd2d52f1d04afd8`，
tests postimage `bdd9efc99e30ba2633de4887a6c5ce93481f526520bfe3a1eb5757e46a976253`。
checked successor 在 encode/checkpoint/VM 前拒绝溢出，普通 ClientFrameDriver outer rollback
保留 terminal fault/retry 资格；独立源码复审已关闭测试计数错误。归属请求
`ac321f04860d4ae1867409e56e3d650d` completed，[WOC 精确终态](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/woc-transaction-terminal-rollback-r4/attribution-terminal.json)
SHA-256 `a2227f0a76dfb26b8649a9a79318bcb8eb8ce14825850b3caf93151edc995bbd`。
声明的 21 个 lower tests 实际执行数 0；真实 ZrVM producer、原 client presentation、
lower/original/upward、Windows 产品和性能门仍 open。关联 Runtime24 finding 不合并。

Core activation r3 独立复审有两个 P0、两个 P1，原作者在不同 r4 修复；真实 Headless
Session 停机回归 r3 的三项源码问题已由独立 H 关闭，未编译或运行。Editor r4 还有 ForView
非穷尽 match、测试导入、None 保存越界及 crash pair/逐叶恢复门，原作者在不同 r5 修复。
IME r3 的 ABI 导出 P0、实际原生回调/显式请求/真实 caret publication 缺口及矩形范围门
仍待不同 r4 修复；ZrVM r1 九个源码路径复审有 14 项未关闭问题，修订 r2 继续进行。
这些完整源码复审不是 pass，精确终态见 [复审核验](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/verified-source-review-terminals-20261002-r1.json)。

Asset v5 的 48 路径 postimage/raw baseline/current 哈希核验无漂移，但普通 World 保存
物理路径越界、实际 RuntimeExtension→ProjectManager provider ingress 仍为 P1 open。
Graphics consumer 的宣布产物被作者补写测试后覆盖；root 冻结不同 r2 和漂移收据，独立
复审继续，旧 metadata/patch 字节未保留，不重建旧历史 seal。Hub 38 路径源码复审已闭合，
实际 native/helper/服务/Keycloak/Tauri 与性能验收仍 open。

六个未登记外部 Cargo 目录门仍阻止新的受管执行；没有重复提交 unchanged validation。
Core/Native 活跃外部 owner consent 与 Runtime04 compatible Resource producer 闭包仍未
建立。原三个 Penpot lifecycles、三组原 SID、route、既有 tickets 和 failure 证据保持原状。
本次没有新 Failure/Session、状态复活、外部源码/lease 回收、commit、push 或 WeCom。


### 本次精确出处与验证边界

| 范围 | 当前可复用证据 | 下一项原合同门 |
| --- | --- | --- |
| WOC | 正常 preview `dc9f691c26134138b0cce391ef36764a` / apply `7a6486b5315c4d26bd332d858ca06778`，两个准确字节和 completed attribution；原 archived owner 状态未变 | [21 项测试准备](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/woc-transaction-terminal-rollback-r4/declared-test-gates-preparation.json)：未提交，不是 Cargo 证据；实际 producer 输入闭包、lower transaction/original client presentation/upward roles |
| Core 真实 consumer | [H r3 独立复审](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/P/core-shutdown-consumer-tests-r3/independent-consumer-tests-review-h-r3.json)：profile/module、实际 census、直接 provider 身份及 incomplete gate oracle；findings=0、tests=0 | 同 parent production overlay 编译和原 Headless test，活跃外部 runtime owner source consent |
| Core activation | G 两项 P0：失败 batch 没有先关闭 admission/drain/保存 retry owner；观察者完成点 census 不一致。另两项 P1：lazy service 漏重置、batch-origin receipt 混淆 | 同 source owner r4 及不同审查者；实际 late waiter、lazy build、held call、fail/retry gates |
| Editor | F/G/H 合并六项源码/覆盖问题；r4 frozen 27 路径上下文，各自准确 hash | r5 同一原实施线，Scene/workspace fail-atomic pair、None 物理路径、真实双叶恢复和原 ForView dispatch |
| IME | App A 三项 P1；Runtime G 一项 P0、一项 P1、一项 P2，[Runtime IME review](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/private-ime-candidate-r3/independent-runtime-ime-review-g.json) | 正常 Windows callback + 显式 V2 request + 实际 caret/DPI/resize/window-generation，V1 fallback/source arbitration；actual multiwindow/native receipt |
| ZrVM | E/G/F 总计 P0=1、P1=12、P2=1；[Restore review](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/vm-checkpoint-repair/revision-20261001-r1/independent-restore-review-f.json) 包含 commit 后 allocating Value_Copy、fallible remembered-set barrier、native-yield gate | 原 producer r2，Create/rollback fault atomicity、真实绑定 CTest、GC/alias/control-state、WOC parity；未运行 CMake/C/Cargo |
| Asset | [48 路径/consumer 核验](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/asset-core-source-terminal-verification-20261002-r1.json) SHA-256 `58ea04f45481cab2c0489dd93903c72dfb39f7c66b393b35a58bbe869cd0f550`；最终 v5 metadata alias 固定 | 正常 World 保存共享 resolver 物理准入；真实 provider bridge/reopen；另外 producer/fixture 复审与原 product/performance |
| Graphics | [root 冻结及漂移收据](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/rg166-consumer-frozen-r2/root-freeze-and-drift-receipt.json) SHA-256 `3e12d402988b92f69c61e731d4d7626f7b3d3efb29ff06aceb29606583cf4520`；6 edited paths + 1 frozen context | 不同审查者验证正常 CompiledScene/GPUScene/RHI/WGPU、完整 input closure、实际 queue/resource/device-loss 与 Windows GPU 性能 |
| W0 第六批 | [未发布原因](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/astra-current-plan-receipts-20261002-r6/batch6-rejection.json)；原空 review 与 r2 都保留 | 原作者新 r3 的全部 namespace、源码/caller/span、真正 report gate 与 prior overlap 核验；没有提前增加观察数 |

第五批 completed 回执关闭的是准确记录归属；旧 r4 attribution
`807fdc7028a047d5b28e08f10f0af6db` 的 failed/internal_error 仍保留，不由后来成功覆盖。
本 r6 只更新这两份原 W0 文档；准确 postimage 与正常归属终态另存，不把结构检查视为
Rust、产品、性能或原三 Failure 的 acceptance。


## 2026-10-02 02:26Z 第六批逐项映射与完整复审边界

本批为 Runtime167 `PHY3-P1-001..006`、Runtime170 `RT-AN-01..06`、Runtime173
`NET-RT-001..006` 的 18 条有界当前源码观察。与前五批 130 个键逐项比对后，14 个新键、
4 个旧键刷新，规范观察总量为 **148 条 / 144 个不同 report-qualified 键**。这不是已验收
finding 数或全目录覆盖率。原空 batch6、错误 r2、作者 r3 与 root 不同 r4 都保留；没有
创建、导入或关闭任何 Failure。17 条 crosswalk 关系全部 `identity_merge=false`。

根线核验了三个原报告的准确段落、34 个当前文件哈希、2 个明确缺失路径、65 个源码 span、
18 个正常 caller span，并另外固定 8 个测试源码上下文。修正作者沿用的 Runtime221 metadata
为原报告 Runtime170；每个键始终等于原报告路径加原 finding ID。规范行包括逐项 lower /
original / upward / Windows 产品门、明确回归场景、存在但未执行的复用测试及性能测量范围。
见 [第六批准确输入核验](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/w0/batch6-root-verification-r3/root-verification.json) 和规范 manifest `2026-10-02-w0-batch6-manifest.json`，SHA-256
`9df9c887ad0c074cc6d6dfda463323ec7652d4b919cade2ce2284f058c465af4`。测试与性能实际执行数仍为 0，全部条目 acceptance open。

当前源码结论已收窄：Physics 配置在 `set_value` 成功后才清 backend/commands、写内存，
不再沿用旧报告的 memory-before-persist 指控；完整 validate/prepare/rebuild/generation 门
仍 open。Net 已有 root 与 HTTP/WebSocket collector，这两条 factory resolve 原 root manager；
剩余四 feature、RPC/replication owner、World stages、effective config 和 Editor provider
仍待实施。Animation 的 facade 在同一模块包裹 DefaultAnimationManager，不能据两个注册
名字断言两个独立 solver；跨 builtin/plugin 目标与版本化 artifact/evaluator 仍未验收。
规范行状态为 **10 confirmed_open / 8 narrowed_open**；库存 576/61/515 仍是历史界限。

Asset v5 全 48 路径 E/F/H 复审有 5 个 P1 观察和 4 个 P2 验证缺口，其中两个 ingress
观察指向同一最低支持合同，保留原 ID 及关系。最低源码合同包括 cache wire version/迁移、
组件引用表验证、World 保存物理根准入和真实 RuntimeExtension→项目重开 codec ingress。
原作者不同 v6 修复中，见 [Asset 全 48 路径复审处置](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/asset-scene-2d-persistence/root-independent-v5-disposition.json)；没有凭源码或 apply-check 关闭测试/产品门。

Graphics r2 独立复审确认生产 `Arc` 被 cfg(test) 隐藏，另有 defining API 补丁输入闭包、
真实绑定/queue lifecycle 回归和前序未来时间 provenance 门。原作者不同 r3 继续修复，
[Graphics 独立复审](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/rg166-consumer-frozen-r2/independent-graphics-consumer-review-e.json) 保留原 r2 和未留存旧 seal 的准确边界。Sound 的 19 列出路径冻结后，
14 个非测试与 5 个测试路径分别独立复审；Navigation 的 24 路径 operation delta 相对私有
前序 guard，57 文件基线有 15 项与共享 current 不同/缺失，组合闭包尚未直接移交构建。

原三个 Penpot lifecycle、原 UI/RG166/native SID、source-only postimage 和 tickets/routes
保持原状。六个未登记外部 Cargo 目录、活跃外部 Core/native source consent、Runtime04
compatible Resource producer 闭包及所有受管 lower/original/upward/Windows 产品/性能门
仍 open。没有新 Rust 提交、commit、push、WeCom 或外部 lease/status 变更。


### 精确记录和下一项门

| 范围 | 可复用源码证据 | 原退出门 |
| --- | --- | --- |
| W0 batch6 | [18 行规范观察](2026-10-02-w0-batch6-review.jsonl)、[17 条关系](2026-10-02-w0-batch6-crosswalk.jsonl)、[摘要](2026-10-02-w0-batch6-summary.json)、[manifest](2026-10-02-w0-batch6-manifest.json)；130 prior 键逐项核验，原报告优先级保留 | 当前观察不是整改验收；未读的其他 ID/报告、实际 lower/original/upward/product/perf 门继续开放。batch7 原 12-ID 审计尚未发布，不计入本批 |
| Physics | 普通 tick 仍走单步 incoming delta，projection 扫完整节点；配置持久化排序局部已修复 | 0..N 固定步、单 World owner、fail-atomic rebuild、同代 Scene/events/query receipt；1K/10K/100K 与故障产品测量 |
| Animation | plugin 与 builtin/evaluator/artifact 仍分离，IBM 为 detached Data，两个 palette 仍 recompute；IK 为 pure jobs；直接 Transform writes 无跨系统 writer arbitration | 同 ABI artifact/provider，IBM/remap import/cook/load goldens，typed sampling、bounded job、writer claims、IK normal project；1/64/1024/10000 actors |
| Net | 正常 catalog 收集 root + HTTP/WebSocket，仍缺其余四 feature；Editor resolver 未有 Net，config 未消费，egress no-op | 六 feature owner/activation snapshot，真实 client/server/Editor 与 World stages、wire/security/fault；1/16/64/256/1K connections 和 allocations/latency tails |
| Asset | v5 全 48 路径独立 scope union 无漏；5 P1 +4 P2，源码 contract 四组 | v6 同 owner，legacy artifact wire、完整 marker/table reject、共享物理 resolver、实际 producer 注册/撤销/reopen；script+generic 同存、stable digest、非空 generic row 性能 |
| Graphics | r2 六 changed +一 context，E 复审 JSON SHA `dce648e75ae3e9bd615252830583a5f222422ed13f6fa6f2dfc1daeb99bc75a7` | 修复 production Arc、准确 API predecessor/full patch、实际 CompiledScene→GPUScene→RHI/WGPU receipt/ticket、original2 execution_packet tests 和 Windows GPU |
| Sound | [Sound root 字节冻结](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/sound-device-root-frozen-r1/root-freeze-receipt.json)，候选与 applied 私有 tree hash一致，但不是编译或执行 | actual Kira/CPAL normal device/voice last-good、generation/ABA/source commit、stop completion；独立 source/tests、managed/device/product/perf |
| Navigation | [Navigation root 字节冻结](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/providers/nav-operation-root-frozen-r1/root-freeze-receipt.json)；24 delta paths 和57组合candidate 哈希准确，15基线差异保留 | raw-current→组合 patch/原 guard 顺序，真实 World-owned apply与frame lock、bake/clear/restore、cancel/replacement/unload、原下游 Editorundo/redo |

本批原作者的 `required_performance_gates` 有正确性 golden cases 混入；原标签在每行
root provenance 中保留，规范性能门独立列出测量对象及等价前置条件。一般 release p95
不得回退超过 5%，新增热点默认目标为等价 p95 改善至少 20%；更严格既有预算继续适用。
没有测量样本、没有假定任何门已通过。相关已有测试只确认函数在准确源码中的存在与有限
oracle，不等于其测试二进制已受管封存或实际运行。

本次六路径准确字节写入与 terminal attribution 单独留存；不会用后续成功覆盖旧失败
receipt。r6 两份文档及其正常 completed attribution 保留为本次基线，追加段落不改旧字节。


## 2026-10-02 04:03Z 第七批精确源码映射与复审边界

第七批固定 Runtime213 `RT213-P1-001..006` 与 Runtime225
`MSP4-P1-025/026/027/005/019/033` 的 12 个原 P1 finding。六批规范输入仍为
148 条观察 / 144 个 report-qualified 键；本批无旧键重叠，准确发布后总量为
**160 条观察 / 156 个不同键**。它们是源码观察，全部 acceptance open，测试、产品和
性能执行数为 0。576/61/515 报告库存界限保留，不将局部行复核计为整份报告验收。

原作者四件记录与 root 核验均保留。root 固定 27 个原 source-evidence 路径，另核验
真实 caller 与测试上下文，合计 36 个源码/测试文件、6 个报告文件。历史有界 source-evidence
路径并集为 203，不能据此宣称当前全树 coverage。规范行补齐最低责任路径、实际正常调用
跨度、存在但未运行的测试函数及有限 oracle、逐项 lower/original/upward/Windows 产品、
故障和性能门。规范 manifest SHA-256 `11b99670a3104694384fa38af9b43df7ada328cca2489eff8730f3e25a7ec622`。

源码结论收窄为 **1 narrowed_open / 11 confirmed_open**：RenderScene 已有正常
renderer→registry/projector→streamer→owned GPUScene journal 调用，旧 tests-only 前提
不再成立；唯一跨产品 mutation authority 与实际 GPU generation 门仍开放。Visibility 已
传递 previous static/dynamic index 和 FrameHistoryValidationKey，剩余是广域 primitive
收集、组合 typed receipt、实际 plan executor、GI bridge 和共享 early/late/final visibility。
Shader 的 substring capture、visiting/completed cycle 混同、重复 source 所有权/预算、
variant exhaustion panic、disk 两次提交和 error-proxy 同步 compiler 等待仍有有界源码证据。

crosswalk 四条关系均不合并 finding 身份。root 将原文 `Runtime89` 别名准确关联到唯一
现存 89 编号 RenderGraph 报告；作者猜测的 `89-runtime-visibility-review.md` 缺失不等于
Runtime89 报告缺失。原错误 edge 在 root provenance 内保留，未读该报告 finding。

Editor r5 的 30 个候选路径与 current raw baseline 无差异；私有补丁重建首次被系统 Git
autocrlf 转换为 CRLF，保留该 receipt。仅为新重建命令设置 autocrlf=false 后，30 个
原始 postimage 哈希全部匹配。F7/G12/H11 完整独立复审进行中，不据此关闭 durable
Scene/workspace、序列化 startup restore 或真实 Editor 产品门。Core r4 21 路径组合
与 Nav r1 24-path delta 的原记录保留；Core 正常回滚 admission/cleanup、Nav move/ABA/
in-flight budget 的源码问题继续由其实施线修复，测试与产品门均开放。

三个 Penpot lifecycle、原 SID、source-only postimage、tickets/routes 保持原状。
Runtime04 单测 snapshot 与 compile preparation 不算 Cargo 通过，兼容 Resource producer
仍需其原 owner 封存。六个未登记外部 Cargo 目录、活跃外部 Core/native consent、原 managed
lower/original/upward、Windows 产品及性能门仍开放。没有新 Rust 编译、Failure import、
foreign owner/status/lease 变更、commit、push 或 WeCom。


### 原 finding 与最低退出门

| 原 ID | 当前最低合同与复用证据 | 保留的退出门 |
| --- | --- | --- |
| RT213-P1-001 | 正常 RenderScene registry/journal/projector 已接 renderer；no-op generation/World lineage unit fixtures | VIS213-G01/G11/G12；真实 submission failure/retry/release、稳定帧零解析与单 dirty；1K/10K/100K |
| RT213-P1-002 | 正常 constructor 有 previous index reuse，仍广域收集/规划；history dirty/full-rebuild fixtures | VIS213-G02/G06/G11；persistent add/remove/change transaction、稳态 visits/alloc 与 1/4/16 views |
| RT213-P1-003 | actual from_frame_views 保留 relevance_generation=0；已有 FrameHistoryValidationKey | VIS213-G12/G17/G22；scene/bounds/view/policy/history typed receipt、逐代 swap reject、正常全部 consumer |
| RT213-P1-004 | instance/particle/instancing plans 的声明/生产/fixture 存在，正常 executor 闭包仍未建立 | VIS213-G10/G15/G16/G32；64 实例真实 instance_count>1、正确 GPU work/ticket、cancel/OOM/stale，100K |
| RT213-P1-005 | optional GI input 到 constructor 后仍被忽略，空 plan/feedback 被 stats 消费 | VIS213-G22/G25/G27；真实 generation bridge 或 explicit unavailable、completion feedback，0/1K/100K probes |
| RT213-P1-006 | VG prepare 与 FrameVisibility submit 分别存在 | VIS213-G18/G21/G22/G23；early/depth/late 真 GPU 执行、共享 final receipt、CPU oracle 与 cut/resize/multiview/device-loss |
| MSP4-P1-025 | import 正常调用 name substring validator；原 shader fixture 仅 happy path | RT-MSP-G01/G02/G05/G06；parsed binding/type/stage/span reflection、comment/substr/local false-ready negatives |
| MSP4-P1-026 | material→ensure_shader_source 正常递归，旧共享 descendant fixture 未覆盖 cycle | RT-MSP-G07/G08；immutable graph/SCC fail-close、自环/多节点环/diamond/budget/last-good；1/100/10K nodes |
| MSP4-P1-027 | ShaderAsset/prepared/assembly/Mesh 拥有多个 String/segments | RT-MSP-G09/G18/G19 与 M225.1/.4/.6；CAS source owner、include provenance、字节/depth/node budget 和 migration |
| MSP4-P1-005 | 正常 Mesh resolver→variant interner，现有 exhaustion 测试 catches panic | RT-MSP-G09/G10/G18 与 M225.3/.6；typed no-mutation exhaustion、pin/tombstone/retirement、epoch/slot ABA 与 10K churn |
| MSP4-P1-019 | 正常 Mesh/prewarm 调 disk.write，payload/meta 独立 atomic_write | RT-MSP-G09/G18/G19 与 M225.1/.6；单 manifest commit、冲突 reread exact verify、crash/disk-full/restart，10K entries |
| MSP4-P1-033 | 正常 draw census→error requirement admission 同步 finish compiler | RT-MSP-G23/G24/G26 与 M225.4/.6；startup-installed all-target bundle、blocked compiler 无 frame wait、真实 visual/device recovery |

第七批[规范观察](2026-10-02-w0-batch7-review.jsonl)、[关系](2026-10-02-w0-batch7-crosswalk.jsonl)、
[摘要](2026-10-02-w0-batch7-summary.json)和[manifest](2026-10-02-w0-batch7-manifest.json)
保留原 ID、优先级和 milestone references；新的场景描述不是虚构的已存在测试函数。
原报告的产品/故障/Unreal parity 条件保持开放，额外的 source guards 不替代真实 runtime
oracle。一般 release p95 回退不超过 5%，等价新热点目标至少改善 20%；测量需固定真实
RTX3060 adapter 和 workload，交错 before/after、报告 p50/p95/p99、alloc/RSS/VRAM。
本批没有任何实测样本。

root 源码核验：[结构与字节 receipt](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/w0/batch7-root-verification-r1/root-verification.json)；
Editor：[准确私有补丁重建 receipt](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/editor-toolbar-root-frozen-r5/root-exact-reconstruction-r2.json)。
六路径 byte publication/terminal attribution 单独记录，追加前六批原文 CRLF 字节全部保留。
W0 第八批的原 12-ID 审计尚未发布，不计入本批。当前未完成范围和审查问题不会因文档发布而关闭。


## 2026-10-02 05:20Z 第八批源码更正与两批发布边界

第八批保留 Runtime187 `ECS4-P1-001..006` 和 Runtime190 `RCM6-P1-001..006`
的 12 个原 P1 身份，与前六批和第七批均无 report-qualified 键重叠。前六批仍为
148 条观察 / 144 个不同键；第七、八批准确写入且归属回执终态后，本阶段为
**172 条观察 / 168 个 report-qualified 键 / 136 个 literal ID**。这些是源码观察，
测试、产品和性能执行数为 0，全部验收门开放。576/61/515 整报告库存界限保留；
历史有界 source-evidence 路径并集为 225，不能当作当前全树或整报告验收。

第八批 root 固定 23 个原 source-evidence 路径；连同实际 caller/test 上下文，共
32 个源码/测试文件和 16 个关系证据报告。30 处实际测试定义存在但未运行，移除
一处“测试作为正常生产 caller”的标注，并将 16 条 crosswalk 指向准确列表条目。
原作者 02:00Z 标签不构成已见证的历史 seal；root 独立记录当前字节与捕获时间。
原报告和原作者记录保留，关系均不合并 finding 身份。

当前结论为 **7 narrowed_open / 5 confirmed_open**。EntityRegistry 代际检查、
StableQueryOrderIndex 稳定顺序、稀疏/密集互斥的值所有权、ComponentStorageLocation
以及 PendingComponentRow 的最终预检已有源码和有限单测。多份派生索引本身不能证明
身份损坏；剩余是跨 owner/layout generation、明确排序语义、故障与性能合同。正常
clone/load 重建是 owner-local projection reconstruction，未证明存在任意外部 reset。
Camera 仍有 11/14 reflection skip、混合 source/compiled policy、缺少本批检查的
endpoint/lens/rig/shake role、裸 active EntityId 与静默非法选择。Editor viewport
正常提供 camera snapshot，绕开全局 active selector；全局 setter 修复不等于真实
Editor endpoint 产品验收。每行已记录最低 owner、正常调用、有限 oracle 和逐项
lower/original/upward、Windows、故障与性能门。

Core r4 完整 14+7 复审确认三处回滚缺陷，原实施线 r5 继续修复；Sound r1 的 14+5
完整复审确认候选初始静音和 config/state 并发两处 P1，r2 继续修复。Editor r5
三段复审尚未全部终态，崩溃夹具新旧值相同和 ViewInstanceId 类型边界仍需关闭。
所有私有补丁、原失败、真实 Windows/native GPU/audio/Tauri/Keycloak、性能门均保留。

协调器正常查询已返回既有 W0 心跳 `e798a4769cf5410baafdb3a759a048a3` completed，
原 SID/scope/baseline 保留。旧离线 queueId 的历史 requestId 绑定没有伪造；没有
重复心跳、daemon restart、foreign status/lease 回收、Failure import、Rust 构建、
commit、push 或 WeCom。已有维护与源码 owner、六个外部 Cargo 目录的实际门仍有效。

第八批[源码观察](2026-10-02-w0-batch8-review.jsonl)、[关系](2026-10-02-w0-batch8-crosswalk.jsonl)、
[摘要](2026-10-02-w0-batch8-summary.json)和[manifest](2026-10-02-w0-batch8-manifest.json)
可重现本批原 ID、证据与剩余门；记录发布不关闭任何产品验收。

## 2026-10-02 精确源码修订与原验证边界

八个已归属 W0 批次仍为 **172 条观察 / 168 个 report-qualified 键 / 136 个 literal ID**。
第九至十二批是私有作者记录，独立核验和正式发布尚未完成；第十三批只读核对 Physics 原 ID，
不增加当前 canonical 数量，也不证明整报告验收。

Core r5 的 21 个候选输入与 Sound r2 的 20 个输入均已按原始字节重建，当前基线无漂移。
Core 作者补丁的 apply-check 失败留在原回执，root 原始字节补丁的私有检查与重建通过。
以上均为源码证据，实际编译和测试数为 0。Core 回滚修订正在独立生产与测试复审；
Sound 实际 owner 的 start/config/stop ABA、prepare/commit PCM 及 rejected candidate 保留旧输出
仍待补齐。测试线程在 setter 之前发信号、或由 fixture 持锁执行 try_lock，不能证明 owner 已到达锁门。

ZrVM r2 的普通 map 校验错误已由定义 API 确认：普通 HashSet_Add 使用独立 GC pair，仅增加
elementCount；capture/preflight 使用 pairPoolUsed 判断普通 active bucket 数，会拒绝合法 map。
同一 producer 在不同 r3 修复，保留 alias、GC、Create/rollback 原子性及原 WOC finding 身份。
IME r5 保留重复 Ime 导入、零宽 caret 与正常安装/候选 publication/失败恢复的复审边界；
Editor r6、Graphics r4、Asset、Nav 与 Shader 私有修订继续，不将未封存 staging 当作完成。

既有 artifact.audit 的较新历史终态有十个未登记目录，旧六目录记录保持原样。07:35Z 正常
claim 返回 coordinator_retired，旧 service/workers/leases/validation/integration 已禁用；
用户随后明确授权按退役后的本地证据路径继续。旧审计不被改写为当前本地 wrapper 的准入门。
保留所有旧数据库、queued work、locks、SID、leases、attributions 与 receipts，不重启或恢复
旧服务。当前源码哈希与明确 scope/handoff 仍保护活跃重叠源码；Core/Native 重叠 owner、
Hub Runtime producer 与 Runtime04 compatible Resource producer 的实际输入和验收门保持开放。
原 UI timer135ab 与 RG166 Copy7aa 已有归属保持原状，三条 Penpot 生命周期继续开放，
原 lower、build-editor 与 source/token/font/media/locale 产品门均未通过。

性能环境只枚举确认 Ryzen 7 5800H、RTX 3060 Laptop 6 GiB、驱动 617.14 与 GPU UUID。
实际进程 adapter 选择、分辨率、质量、workload、交错测量与 p50/p95/p99/RSS/VRAM 仍待执行。
准确私有出处：[本轮源码及门记录](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/astra-current-plan-receipts-20261002-r7/source-wave-r7.json)。

没有新增 Session/failure、重复 import/validation、route load、monitor、wake、external hold、
commit、push 或 WeCom。源码、原生产品与性能 acceptance 全部保留未通过状态。

### 明确规则切换与本地记录

用户答复「允许按退休后的本地证据路径继续（推荐）」。现有 W0 文档在 Root 独占 scope
与精确基线下追加本条；历史 attribution 不调用已退役 API 更新，原值保留为迁移证据。
[退役拒绝与规则边界](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/coordinator-retirement-boundary-20261002-r2/terminal-boundary.json)
保留原 claim 拒绝、变动规则与 wrapper 字节。独立本地命令须先封存实际依赖闭包，并使
所有 target/build/cache/Cargo home/TEMP 物理落在 D/E/F drive-root cargo-targets 内，
保留原生 Cargo locks 和至少 35 GiB 空间。独立命令结果不代表 formal Jenkins 验收；
原 lower/original/upward、真实 Windows 产品与性能门保持开放，直到实际证据通过。

## 2026-10-02 W0 batch9–12 精确映射落源

本条对应四个私有原稿及分代校正，48 个新 report-qualified 身份全部保留原 finding ID。
当前累计为 220 条观察 / 216 个 report-qualified 键 / 183 个 literal ID；八个既有批次
不改写。Runtime189 与 Runtime225 的 MSP4-P1-005 仍是两个报告身份，只记录共同实现族。
全目录 576 / 61 / 515 基线及未读范围保持原样。

| 批次 | Canonical 四件记录 | 校正输入 |
| --- | --- | --- |
| 9 | [review](2026-10-02-w0-batch9-review.jsonl) · [crosswalk](2026-10-02-w0-batch9-crosswalk.jsonl) · [summary](2026-10-02-w0-batch9-summary.json) · [manifest](2026-10-02-w0-batch9-manifest.json) | 原稿和 B9 r1 保留；B9 r2 修正封存清单的自哈希问题 |
| 10 | [review](2026-10-02-w0-batch10-review.jsonl) · [crosswalk](2026-10-02-w0-batch10-crosswalk.jsonl) · [summary](2026-10-02-w0-batch10-summary.json) · [manifest](2026-10-02-w0-batch10-manifest.json) | 原稿、r1/r2 保留；B10 r3 复核变化后的 World mirror |
| 11 | [review](2026-10-02-w0-batch11-review.jsonl) · [crosswalk](2026-10-02-w0-batch11-crosswalk.jsonl) · [summary](2026-10-02-w0-batch11-summary.json) · [manifest](2026-10-02-w0-batch11-manifest.json) | r1 保留；独立 owner correction、报告清单去除数值别名重复、free-function correction |
| 12 | [review](2026-10-02-w0-batch12-review.jsonl) · [crosswalk](2026-10-02-w0-batch12-crosswalk.jsonl) · [summary](2026-10-02-w0-batch12-summary.json) · [manifest](2026-10-02-w0-batch12-manifest.json) | 原坏 JSON manifest 原字节保留；r1 修正语法并核对实际 20 个输入；owner correction 独立保留 |

24 条 B11/B12 owner correction 均有具体类型/函数、仓库路径、完整哈希和定义行。
NET-P0-005 的真实 owner 是 websocket backend 的 `listen_websocket` 自由函数，返回
`TungsteniteWebSocketListener`；不把该函数描述为不存在的关联方法。空租约、静态定义或
组内 delivery 均不提供 foreign source consent。

B10 的 `zircon_runtime/src/scene/world/event_mirror.rs` 从原稿
`e067eeaf1ea9952e089eb122248757b2f09e3a4dc3e80bb2d8459e5c13e2bdfb` 变化为
`681c006c9145aedb86027a205d6b6fa347ec6c2f278d1dccbe27cd7846f18cb2`。
r3 已复核 Instant/deadline、bounded reclaim/requeue/retry 与 callback deadline；
REV-P1-01 的 shared taxonomy 合同仍为 confirmed_open。旧源码哈希保留为原稿证据；
没有重新创建该 failure 或更改原 owner 状态。

[Root 分代规范化准备](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/w0/root-publication-candidate-r2/normalization-preparation.json)
SHA256 `f16ca6b24973a0b9595ce7be4b6148e56f0380ed6ec7c4d267fcaea37ba8ec9a`，
保留每个原稿/qualified artifact 和独立 correction 的原始字节及完整哈希。
[独立结构核对](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/w0/batch9-12-root-input-verification-r3/root-input-verification.json)
SHA256 `1daa65b14117b27a206f15af3bfe7aa00dfbc72d81e4643844aafafcde814739`，
48 个 literal table row / qualified key / 原优先级 / 当前责任定义 / 普通 caller span /
已有 named test definition 均可追溯，155 个当前输入哈希无漂移。每个 manifest 明确
私有 raw evidence 根与 source/report→raw capture 路径；历史相对 capture 路径继续
依据原 qualified generation 解析，不迁入 canonical 文档位置解释。

本轮同时封存两个既有实现线的独立源码结论：

- [Core V14/W7 完整 21 文件复审](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/core-lifecycle-root-frozen-r5/root-final-independent-review-union-r1/root-final-independent-r5-disposition.json)
  SHA256 `1da70d5b346cd6844aaeb05c3a3910cd5d3489c8c503c97199982763e9bfe94d`。
  W-R5-001 静态缺口仍开放：cleanup 成功后的真实 ServiceObject 析构 panic 可令 retry
  重放 cleanup。同一作者 r6 私有修补继续；Stopping dependent 的 provider barrier 保留。
- [Hub 当前生命周期 29 文件及 17 个实际测试定义](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-service-lifecycle-root-source-audit-r1/root-current-raw-capture-r1/root-current-source-audit-disposition.json)
  SHA256 `43d576b4c6713f190ec778a6d975e13aba667191b8e2fc57f77aa01c952cc566`。
  当前 shared deadline、DB job ownership、mutation receipt 为 implemented_pending_validation；
  未建立新 lower source bug，不重复实现。真实 Windows 三进程、DPAPI、阻塞 I/O 终止/
  重启、掉电持久化与性能门均未通过。

所有这些证据均 `tests_run=0`，没有 Cargo/compiler/native/product/performance 执行。
协调器真正退役时刻是 `2026-10-02T07:19:48.379193+00:00`；07:35Z 是 Root 正常 claim
被拒绝的观察时刻。原 metadata 字节保留，当前规范化记录明确区分二者。用户答复
「允许按退休后的本地证据路径继续（推荐）」适用于现有目标内的精确范围与输入；
独立本地命令结果不代表 Jenkins 或 milestone/product acceptance。
保留原 SID、源码及 comment provenance、旧 attributions、leases、queued requests/tickets
和全部 failure evidence；没有退役 API 调用、任意 status/归属切换、duplicate import/
validation、route load、quiet-tree、周期 monitor/wake、外部 zr_vm hold、commit、push 或 WeCom。
三个 Penpot 生命周期及原 lower/original/upward 和实机源码/token/font/media/locale 产品门保持开放。

## 2026-10-02 batch13 与实际本地验证续接

第 13 批只映射 Runtime186 `PHY4-P1-001..012` 的原表108–119；Runtime219没有新增身份。
12 行、37 个 raw/current 输入和独立源码复核均已核对；作者00:00Z标签是 placeholder，
不是历史封存时刻，规范记录另列 Root 当前字节核对时间。当前累计 **232 / 228 / 195**
观察/报告限定键/literal ID；原12批字节与576 / 61 / 515整报告界限保留。
[review](2026-10-02-w0-batch13-review.jsonl) · [crosswalk](2026-10-02-w0-batch13-crosswalk.jsonl) ·
[summary](2026-10-02-w0-batch13-summary.json) · [manifest](2026-10-02-w0-batch13-manifest.json)。

### 已执行的 Hub 本地 correctness

- Local execution `c2b52440196f493f838ad8e0e0004bcb`，退出0；实际80/80通过、0失败、0忽略、16过滤。
  原指定的17个lifecycle/jobs/snapshot/maintenance/route测试全部在实际stdout名单中。
- 命令：`test --locked -p zircon_hub --lib --no-default-features --features local-service service:: -j 2`。
  [原终态](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-service-validation-20261002-r1/terminal-result.json)
  SHA256 `487f4db2916787ecc38bf255b6ba160a46fe5dc00d8065bda995d61a2d8042c2`。
  [Root核对](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-service-validation-20261002-r1/root-local-result-reconciliation-r1/root-local-result.json)
  SHA256 `d33d18a5a4f0207fce31117bb23216ceadfea92664ac20e9fe4aa0202894ec6f`。
- 40个audit/mount输入与当前、staged源码一致。实际target/build/cache/CargoHome/TEMP均为物理
  `D:/cargo-targets/zircon-local`路径；父进程C: TEMP不是Cargo子进程的有效TEMP。
  Metadata registry来自stage cargo-home，而wrapper实际CARGO_HOME为共享D-root cache；
  stage metadata封存不能单独证明loaded registry同代，另有实际cache资格核对，未重复运行。
- Windows三进程、真实Keycloak签发/refresh/logout/JWKS、DPAPI、durable ShutdownReceipt、
  SQLite句柄/掉电恢复、产品/性能/Jenkins验收仍开放。此次未启动服务、Keycloak或桌面。

### WOC 最低修复与原证据

原local execution `cc99ac80fb0345b99f620d8cd925c65a` 退出101，原Runtime transaction测试
执行0/21；`FixedTickInputRef.commands: &[Command]` 在payload144被再次借用，产生E0277/E0282。
最低修复仅将`for command in &self.commands`改为`for command in self.commands`；完整原bytes和
邻近既有Command准入改动保留，原SID `astra-goal-20260929-01a0f033` 未变。
[落源凭据](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/woc-borrowed-encoder-compile-repair-20261002-r2/root-source-publication.json)
SHA256 `495d7b656ecc16c14bb3078862f89fc57f6918cc66390f7be6a20911a6a160ea`。
payload当前postimage `f984d92af3e8e67fb8fc658b68ae8ec2cd1ef2b2e81e60acd01ecef6645e25bc`。
新r2先运行普通`woc_protocol` command_admission/protocol，实际通过才复验同代原21个transaction；
原失败副本不修改、旧票不重放，尚未取得新运行终态或产品验收。

### 同一实现线的源码回修

[Asset v6完整31+23复核](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/asset-scene-generic-root-frozen-v6-r2/root-final-independent-review-union-r1/root-final-independent-v6-disposition.json)
SHA256 `38e5470e3944d7886a116d48b10d86da9f027651cbdd32fe73b138bee59f31cb`。
注册失败留descriptor、强ProjectManager listener生命期、撤销后liveWorld保存漏动态行和普通
provider activation mount仍有明确源码门；同作者distinct v7继续修复，不吸收foreign native/Core。
[IME r6完整17+23复核](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/private-ime-root-frozen-r6/root-final-independent-review-union-r1/root-final-independent-r6-disposition.json)
SHA256 `78daa68007368d62cfbc3d3155251f870413c409e149d565c95c58e116e78495`。
同作者r9私有41源候选修复编译导入/可见性、失焦Background、IMM预算；测试执行仍0，
真实两窗口、producer清理/代际重试、source-owner、性能与原门开放。其他封存候选继续独立复核。

本次W0行和候选复核的`tests_run=0`不覆盖上面的Hub真实80个测试。所有原SID、leases、
attributions、queued work、failure/comment证据保留；没有退役API、任意status切换、新Session/
failure/import、duplicate validation、route load、monitor/wake、quiet-tree、外部zr_vm hold、
commit/push/WeCom。三个Penpot生命周期及原build-editor/live Source/token/font/media/locale门仍开放。

## 2026-10-02 batch14/15、源码发布与本地执行续接

Root 当前核对时间：`2026-10-02T19:44:33.084160+00:00`。本节是新的有限证据边界，旧批次和失败回执保持原字节。

### 当前 finding 身份和原始元数据

| 范围 | 新 finding atom | 独立当前输入 | 当前结论 |
|---|---:|---:|---|
| Runtime170 原104/110/116/122/128表行 | 5 | 38 | RT-AN-09原P0，其余原P1；10 narrowed_partial，其他confirmed_open |
| Runtime175 RT-GAS-01..28 | 28 | 58 | 原P1；每项owner、正常producer/consumer、测试定义/mount已独立核对，未执行 |
| Runtime175 P2原无ID列表 | 12 | 同上 | ordinal身份明确标记nonliteral，未发明原finding ID |
| Runtime175 G1..G24 | 单独24个gate | 同上 | 22 Fail / 2 Partial；不计作finding或原literal finding ID |

当前累积 **277 finding观察 / 273 report-qualified finding键 / 226 distinct原literal ID**，
含12个非literal ordinal。另有24 gate，原始finding+gate行合计301。键使用报告路径，
不因裸RT-AN ID碰撞合并不同报告。Runtime221/198/222仅保留currentness关系；Runtime175
未打印继承P0组的ID，本次没有补造身份。原13批canonical字节已逐一核对保留。

- [batch14 review](2026-10-02-w0-batch14-review.jsonl)、[crosswalk](2026-10-02-w0-batch14-crosswalk.jsonl)、[summary](2026-10-02-w0-batch14-summary.json)、[manifest](2026-10-02-w0-batch14-manifest.json)。
  独立JSON `e6045d9b3ecc2c5ed6b46bcac1df93d1df19dc7d3ab161ea2c424ced5b192941`；当前manifest71fcd...38 captures匹配。
  原30-input/db63 interval字节不可用，原summary30和post-final变更史保持资格边界，未重建历史封存。
- [batch15 review40](2026-10-02-w0-batch15-review.jsonl)、[gates24](2026-10-02-w0-batch15-gates.jsonl)、[crosswalk](2026-10-02-w0-batch15-crosswalk.jsonl)、[summary](2026-10-02-w0-batch15-summary.json)、[manifest](2026-10-02-w0-batch15-manifest.json)。
  独立JSON `ff3bd0772fa66203cfed538270084a49a7a5ce3866ed2646e48a54a2634a059b`。原作者错误路径的B14-absence元数据保留；
  Root实际正确路径B14四件均存在，另列补充资格。RT-GAS-18/G11共5处clock.rs146–205
  原span超出203行，补充实际146–203；这只是anchor元数据问题，原输入没有字节/hash漂移。

独立库存核对JSON `f91f3659d4bc79c98de71d4445378f033d7b0fd90a29391dd2bf7768eade2636`：六目录顶层编号报告576=App9/Editor276/Hub7/
Plugins22/Runtime245/Interface17；历史61 source-observed/515unreviewed标签不提升为整篇接受。
Runtime174和Runtime76报告原索引SHA与当前SHA不同，原索引字节保留，按受影响子域重核。

### 实际发布、执行与剩余门

- RG166消费修复r5实际共享发布6改+1新，41个postimage零不匹配。
  [发布回执](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/rg166-consumer-source-publication-r5-r1/terminal-source-publication.json) SHA `a35ee464c970dd847151328295c69ac6f48a3c693fd43fc16497b1fb9b80d192`。
  原SID `astra-rg166-device-lowering-20260930-01a0f033` 和原execution_packet7aa99fd9保留。
  类型化queue/device负路径与正常scene fixture源码已独立复核；该fixture encoder目前只有debug marker，
  未证明实际渲染图像、设备执行、frame或性能。Windows RHI/WGPU全下层与原UIfeature compile另行准入。
- Hub服务二进制本地build实际退出0；[终态](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-local-service-binary-build-20261002-r1/terminal-result.json)
  SHA `29e28d921bcd260c39182d8cff758da8e7c62229089ecb6c408a83ab2c68b58a`，EXE SHA `45dcaaa2346d830d1f764f394f31ae325a818d46a8189503d747dbb130364643`。
  前次service::真实80/80通过证据继续使用；缓存/depinfo资格为独立说明，不把stage metadata当loaded完整seal。
  真实Keycloak与服务进程已经启动并取得就绪检查，正常账号/团队/商城/云同步、graceful ShutdownReceipt、
  重启读回及原生桌面仍待实际终态；不宣称这些产品或性能门通过。
- WOC协议借用迭代修复和正常测试bootstrap两处clone已共享发布；
  [同SID源码回执](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/woc-protocol-bootstrap-reuse-source-publication-20261002-r3/root-source-publication.json) SHA `8c284b9991344151857d27dc7d4bff3421861102b206aa118b4776b8498b38e1`。
  payload `f984d92af3e8e67fb8fc658b68ae8ec2cd1ef2b2e81e60acd01ecef6645e25bc`，
  tests/protocol.rs `4d809cc3a3b5767aa5ad24142f10eb6ea88f66e3831912f41b43705d02c7ae11`。
  新独立执行 `c364ed55120d456284da0fdeec4bde30` 正在跑正常command_admission/protocol，
  实际通过才跑同closure原Runtime21 transaction。旧E0277/E0282/E0382失败copy/0测试证据保持。
- Nav r3作者归一化scene patch上下文不能应用原CRLF，原失败check1保存。新的Root补丁只替换
  scene部分原始字节，正常raw check0/apply0、59个原候选重建零不匹配，7production/4tests delta复核中。
  [Root字节回执](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/nav-operation-root-frozen-r3-r3/root-freeze-r3.json) SHA `f8ff7deb29e7a5fba9d06d9f52c8cea3e9b01cc1d749e4a0d111f39e0543cc88`。
  shared state.rs CURRENT6fc构造/共享shutdown delta与Core/Nav/IME仅由原私有composer合成；
  未将旧候选全文件覆盖当前共享来源，也未产生测试/产品证明。

本次canonical文档/JSON记录tests_run=0。协调器退役后保留所有原Session身份、attribution、
历史receipts和failure/comment来源；独立local wrapper证据不等于Jenkins、里程碑或产品验收。
三个Penpot生命周期、原build-editor、live Editor/Source/token/font/media/locale、GPU、
原生两窗口/IME/可访问性、WOC实际VM和双进程、性能及Jenkins门全部继续开放。

## 2026-10-02 WOC 原正常命令实际通过的有界续接

Root核对时间 `2026-10-02T20:20:42.688049+00:00`。原有执行 `c364ed55120d456284da0fdeec4bde30` 同一closure，
按下层通过再跑原事务的顺序完成，未重复提交/重试/更改stage源码。
原事务总注册22个：21个correctness实际通过，`borrowed_tick_state_release_performance_gate`
既有release性能测试正常ignored，未改或删。Root原先假设ignored=0的核对在写入前失败，
本补充保留实际1 ignored，未重复执行Cargo，也不宣称性能通过。

| 实际命令（均通过独立local wrapper） | 结果 | 精确原receipt SHA256 |
|---|---|---|
| `test --locked --manifest-path examples/woc/native/Cargo.toml -p woc_protocol --test command_admission --test protocol -j 2` | 22 passed / 0 failed / 0 ignored / 0 filtered；原6个指定测试齐全 | `cf8dc2234acba9694c5ac4a11b54210e634c3629be61d29aef38f4d808ee9fa2` |
| `test --locked --manifest-path examples/woc/native/Cargo.toml -p woc_runtime --test transaction -j 2` | 21 passed / 0 failed / 1 ignored / 0 filtered；原21个名称齐全 | `caf0484d51262e450c057336681870d7db9c880a70dd377202372ff0dff1c0c1` |

[作者合并终态](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/woc-r3-lower-validation-20261002-r2/combined-terminal-result.json) SHA256
`1e4ef8ea4356cc42e788e14658ee79a653667e191d66ccf47652e643b5424e38`；
[Root独立读回](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/woc-r3-lower-validation-20261002-r2/root-local-result-reconciliation-r2/root-local-result.json) 保存原日志hash、实际执行名称、212个
相关source/raw-current/stage的精确当前相等核对和原Session身份。下层结束时17个dep-info
覆盖41个WOC路径，原事务后43个dep-info覆盖48个WOC路径；完整缓存/编译graph、native
VM/product/perf/Jenkins资格不因此提升。原11756-file历史snapshot中的inactive drift保持说明。

协议结束 `2026-10-02T19:39:44Z`，原事务结束 `2026-10-02T19:50:04Z`。构建产品与cache
位于 `D:/cargo-targets` 的物理family目录，wrapper Python SHA
`300fbea9fdb76946708b20b65e0c0cebc2f3c74cea3d94d43665ecd2844b17e1` 和PowerShell SHA
`2146c834078d6da729bcce5521d83cf90e42ec3cceaf0b3f6d9c0ae9e10f5c89` 原字节保留。

`woc_runtime defaultFeatures=[]`，本次没有启用engine-host/backend-zr-vm；正确的原包路径
为 `examples/woc/native/plugins/woc_runtime`。这43个实际测试不证明真实native VM/backend、
App client/server/bot、双进程/窗口、性能或Jenkins，也不关闭三个Penpot生命周期。后续同原
目标按实际native producer/consumer和当前VM修复的独立source gate继续，不用默认feature
结果替代原生验收。原E0277/E0282/E0382失败copy、0测试和source/comment来源完整保留。

## 2026-10-02 W0 batch16 与新的独立源码边界

Root 核对时间 `2026-10-02T21:50:41.548627+00:00`。App09 的 53 个原 finding（3 P0 / 40 P1 / 10 P2）和 Interface07
的 61 个原 finding（1 P0 / 48 P1 / 12 P2）已按完整原 atom 记录；另存 48 个原 gate。
Interface15 的 61 个 finding refresh 与 32 个 gate refresh 单独保存，均为非新 finding 身份。
当前累计 **391 条 finding 观察 / 387 个报告限定 finding 键 / 339 个不同原 literal ID**。
12 个历史无 literal ID 的 Runtime175 P2 ordinal 保留；原声明 gate 累计72。
`RI-CERT-P0-001` 已在 Interface15 的历史记录出现，按裸 ID 去重后的增量为113，不能写成340。

[第16批 finding](01/2026-10-02-w0-batch16-review.jsonl)、[gate](01/2026-10-02-w0-batch16-gates.jsonl)、
[successor刷新](01/2026-10-02-w0-batch16-refresh.jsonl)、[摘要](01/2026-10-02-w0-batch16-summary.json)
和[manifest](01/2026-10-02-w0-batch16-manifest.json)保留255条原行分类及原报告/comment来源。
独立资格审查 SHA `8212ba9b62e8140177bf1df24677ec2379efff40d3e6520810bd9a00211253ef`，
129个capture（119仓库路径/10私有证据）与当前字节逐项相等；17个超出当前文件长度的
行号端点保留并限定为元数据，未重写历史 seal。作者60文件/9445行口径不作为语义 JSONL
数量；发布前实际为32 JSONL/538语义行/15 review-bearing/277 finding。作者 broad clean
和独立摘要将 batch15 P2 误称 Interface07/15 的文字保留并限定，实际原记录属于 Runtime175/222。
完整576编号报告库存和历史61/515有限观察边界保持；本次不宣称全目录阅读、产品或性能完成。

Core r8 的24私有postimage及9个增量已在原始字节副本重建；原Core shared→r6→r8谱系
仍待完整当前publication envelope。三个 live Session module/construction/state 的 IME 和
Nav consumer wiring由唯一合成线处理，未用旧私有整文件覆盖当前construction/shutdown。
IME r11 有56显式路径：42复用、14当前辅助上下文、1个私有routing重复import更正；
Root独立汇总 `1b013d50586428d4472368315de24c6f029af584fee554e9ef87378a28eff7e4`。
Nav r3 的59路径/48原字节复用/11独立delta的汇总为
`a7fe76285a1220f7b69b277508c8084ddfbce05d1b402b32ef85166bbcc46db3`；
实际provider、丢失worker、deadline、Editor/Session及MAX边界oracle门保持开放。
Sound r4 的21候选/30当前输入（含2声明尚不存在新文件）已精确重建，Root receipt
`aa39751c52cc0185fb7a3609f6844d87cc013162ac84f3f347e8d43538228a20`；
prepare静音与CPAL回调退出的两个新独立P1仍需原作者核验和修复，native PCM/性能未执行。

Hub实际本地deployment和同一SQLite/CAS重启读回receipt
`11e3100f2ce44b7e644298f63ef037d814cc120b95c4bd8bcd0bb259792d0e92`已封存，
readiness/正常退出/端口关闭和212992字节数据库/36字节store-id保持相等；realm已有且0users，
账号/团队/商城/cloud PKCE真实业务尚未验收。WOC此前22+21实际通过及1 ignored release
性能测试的精确旧receipt继续复用；native backend/VM/client/server和性能门仍开放。
以上新增候选/审查没有执行Rust测试或改写生产源；当前完整Runtime本地输入封存/验证及
Hub Desktop材料准备仍在推进。协调器退役后不调用旧claim/heartbeat/status/submit/API；
三个Penpot原生命周期、原SID/tickets/routes、build-editor及live Source/token/font/media/locale，
产品/性能/Jenkins保持开放。没有commit、push或WeCom。


## 2026-10-03 W0 第17–21批原身份归档与当前证据

Root 核对时间 `2026-10-03T04:32:50.739549+00:00`。新增510条原 finding 观察：UI Runtime76/81/82 为148，
Hub01为48，Runtime166/89为126，Editor72/54为127，Runtime85为61。
累计 **901条 finding 观察 / 897个报告限定键 / 849个不同原 literal ID**；
12个既有无literal ordinal保留。原声明gate累计428，另存Hub07的28条
当前资格gate；它们不计入原gate。新增311条successor/currentness refresh均不增加finding身份。

独立审查 `0973fe9af1c153ea17ad42fe1a7f3e7f2f0418a55a4a5bcfe95098dfcbe9430e` 的旧B17/P0及B20归属hold按原文补证和Root派生记录限定：
B17保留148个完整atom及144个原表gate，四个P0的原文closing contract另有
资格补证 `429d025a977a7035262478bc8906e1a3ee7bc4422124c0e8e3a182692d4c8ca0`。原gate表存在性更正 `e263b3a85a1451234722b5d72bf5d9ac9a582730ebac11f5188651ee161b3394`
确认所有补证引用均为已有gate；144原表行不增不减，四P0仍开放，未新增gate或验收。Editor54的67个finding与36个gate
恢复原54报告键、原行号和原文本，Editor127仅作非新refresh；`ED54-P0-03`的
`recompute_if_dirty`/`mark_host_projection_committed`差异及G01库存文字差异保留在当前刷新。
原作者packet、重写过的B17 supplement元数据限制、B20两个不同manifest和3条
第19批 report_provenance均作为原始证据保存；未重构无法恢复的历史seal。

[第17批](2026-10-03-w0-batch17-summary.json)、
[第18批](2026-10-03-w0-batch18-summary.json)、
[第19批](2026-10-03-w0-batch19-summary.json)、
[第20批](2026-10-03-w0-batch20-summary.json)、
[第21批](2026-10-03-w0-batch21-summary.json)分别链接原finding、原gate、刷新、
manifest及精确Source代。所有新映射仍为源码证据，动态/产品/性能门开放；完整576报告
库存、历史61/515有限观察边界继续保留，不能据此宣称全目录验收。

当前Hub S6 typed Present package-lock已落源；四路径桌面Rust编译修复的publication
为`bb83a80d37f868ee2d595b1268fa39e2da189371c637ec94150d7c23df57bb7d`，尚待当前代正常编译。
旧Desktop R2 TypeScript/Vite实际exit0，Rust actualexit101及其漏源/10类型错误保留。
Hub旧二进制的真实HTTP13项负向断言已通过，终态`72ee2d32697f009d483ca70fda52e42c905d31a75167491fd832c229b21b5a3e`；
它不验证当前S6或原生桌面。见[服务责任计划](../../features/hub/02-local-service-authority.md)。
Runtime R4下层510pass/2ignored复用；原UI feature编译因正常`target/result.rs`漏源actualexit101，
八个原测试未执行。Root有限终态`aca8146fda238de08f3085067d8f06eb6644d23a91b52a1a7bb283fe533755ab`
保留原UI/Render哈希及三个原failure/SID，下一步仅准备完整新输入。Editor42路径及Nav37直接路径已精确落源，receipt分别为
`3ccda793de54d5c203c5934eb16984adc62409473d3021a189ffd13804247fb8`与`9adb98e26c37fd286f5915c49642299df702ee88c53c078ecc076fa82ed1059b`；
独立复审均为source-addressed-unrun，Core Session组合尚未据此通过。Core/Asset/IME/
Sound/VM候选独立复审、normal build-editor、live Penpot Source/token/font/media/locale、
Windows产品、性能和Jenkins仍开放；协调器已退役，无register/heartbeat/status/claim/submit，
没有新failure/import、commit、push或WeCom。

## 2026-10-03 Root B22/B23 exact original-identity publication

第22/23批新增 **291条原finding观察 /170条原始gate**，累计
**1192观察 /1188 report-qualified keys /1110个不同literal ID /12条无标签finding ordinal /598条原gate**。
原报告键、literal ID、行号和原文本按独立复审及raw captures保存；64条successor finding/gate
refresh不新增finding或原gate。B22的44个无标签gate保留`gate_id:null`与原ordinal；
09F3的M0–M15是16个roadmap milestone，不混作finding或gate。作者Runtime98的
`09F3-P0-*`对应字段仅作元数据，原09f3报告仍用其真实`P0-*` literal ID。

[第22批](2026-10-03-w0-batch22-summary.json)包含09f3 GI、Runtime98 current-source
addition和Runtime28硬件光追；[第23批](2026-10-03-w0-batch23-summary.json)包含
Runtime26原粒子问题、99d文件的Runtime103和Runtime171非新刷新。Source和参考引擎
raw generation、before/after输入以及当前单独哈希观察分别存放manifest；参考捕获不构成
竞品性能证明。B23已返回packet曾被作者覆盖，当前实际review哈希为
`91b02e48cef57b3986d1ef0d235a26a3fc2df65a5adffd4bd700f24b24c139d0`；
旧seal原字节不可恢复，保留distinct erratum并拒绝重构历史不可变seal。
独立复审JSON `7236a6c139b79a0f73ae01317583fac867e3d6d98ec3e7174b29436b8b6d7a9d`
不替代执行；其Markdown格式更正后哈希为`e7fd43d46315d3039d19a97cd240cfe3e2a184f185a9108cc53d967ad2ee3e06`，
旧Markdown字节不可恢复。

本次只补齐已有W0记录，tests_run=0，所有finding、真实GI/RT/粒子输出、GPU golden、
fault/scale/soak、Editor链路、同画质性能、Jenkins及产品门仍开放。
高级能力按Core/asset/provider/RHI与真实图调度的最低责任模块依赖推进；
Source-only和静态能力名字不算通过。原failure/SID、路由、tickets及已接受证据均保留。

### 2026-10-03 original UI/Render R5 compile and single-test Source repair

原UI SID `astra-ui-time-20260929-01a0f033-r2`及Render
`astra-rg166-device-lowering-20260930-01a0f033`不变；三个canonical failure仍开放。
R5正常同feature `+1.94.1 check --locked -p zircon_runtime --lib --tests
--no-default-features --features target-editor-host,runtime-ui-integration-tests -j 2`
在29633文件精确输入完成actual exit101，终态
`fe1da336d1f49491413d2c008c09f0a6c20df490a3e8a868fdd1db8e64461e57`；
原stderr `b15e769ad18091ab2fefe96eeab665e5c47d963158abdd90912a3023c4729beb`保存。
R4漏的正常`target/result.rs`已在R5完整封存；当前blocker是
`zircon_runtime/tests/zui_reactbits_agent_workspace_contract.rs:186`绑定未声明mut，
其219/243/263的`compute_layout`需要mutable retained Surface。八个原测试未执行。

Root在真实Source仅加该绑定`mut`，原SHA
`ad1a419f2baf869761808b0ffe07f439d7ebcf15998acce49ebc1600044a33a3`
到post `d0b74de51051844a86ff3baead08657c09fad4072c98e4bf85aa074362bc6fb2`，
保留所有production、comment和断言；publication
`b97fb16d6a8108e5da747a444f2887c502aa10a8c88f2a97bf0b982134d06430`
normal apply-check/diff-check0，shared scoped index不变，source-only并未成为验证pass。
下一次仅R5同代input加该单路径postimage，distinct R6，不改failed copy，
原RHI510pass/2ignored回执`69415347f99f363a4eedfb9cfa0632bbdfca5ec56b4214d15e810093d64ffe59`复用。
bounded handoff `a6f6c793c0c36ed03011c6bcb1144a013fc2992730fe6bbf9950a10bc27424ec`
保留旧原始编译、Source修复和下一项声明；normal compile/eight tests、build-editor/live
Penpot Source/token/font/media/locale、产品/性能/Jenkins门继续开放。无旧coordinator API、
新Session/failure/import、quiet/monitor/wake/route load、commit/push/WeCom。


## 2026-10-03 Hub R3 实际终态与 VM 精确源边界

Hub R3不可变输入 `a21f521e78b988793e99eae65e8226697869befc124978731b6cec6ad2f9c590`
普通check实际exit0，普通Hub lib实际96 passed（80service+16file_io）、0failed/0ignored。
旧expected81 service清单有一个不在该代Rust源码中的测试名，按实际集合记录，未关闭缺失名字。
Runtime Interface同批实际783passed/7failed/101ignored；只有PTY终态，原完整日志不可恢复。
原始6项package-lock通过证据保留，7项下层修复正在继续。封存输入前后0漂移；已有执行只
绑定这一代输入，不重复提交。详见现有[Hub合同记录](../../features/hub/02-local-service-authority.md)，执行回执
`b58d847282c49e7056977ad99a8a6053d1bc74e5c7d17a89f5f050c38ceec714`。

VM原owner候选16个修改路径已按精确原字节落源，publication
`2dee8eaa68454da9b60ae8f62615083ec1a73d288e8d73b860456e69c9a3acc0`。
手动LIB_DIR经过物理批准路径校验，docs postimage
`3c6b135a4ae04d982d652e28fe44c89f574d862165dcb3d90998e41165be5f99`，build.rs
`b149e3451846e52bd913c7ead24bd1b667e94404ffa5e9d82b9b9766edd96d35`；foreign CMake
测试尾段、所有预存源改动和shared index保留。作者交回后覆盖了部分封存元数据/文档，
第一次publication在写源前被hash检查拒绝；当前落源采用Root已保存的原始正确37文件候选、
原535bf4封存回执和独立复审d0862a42，漂移单独登记e6181ee1，没有重构旧seal。
正常Windows shared/static原生构建输入仍在准备，实际CMake/CTest/Cargo为0，WOC/EngineHost
双进程、性能、Jenkins门开放。

Sound独立复核发现设备retirement超时后正常播放/音量等控制仍可访问保留Kira句柄的P1；
20路径原候选尚未落源。原owner正按最小共享控制准入边界准备distinct r9修复，保留pending
worker与原24路径候选。Core/IME/Asset接口组合继续，尚未据Source/apply-check宣称通过。
原failure/SID、原lower/original/upward、normal build-editor、现场Penpot Source/token/font/media/
locale、native product、性能和Jenkins门均保留。


## 2026-10-03 W0 B24 原始身份与当前报告边界

第24批新增294条原finding观察、109条原gate；累计 **1486观察 /
1482 report-qualified keys /1301不同literal ID /84条无标签finding ordinal /707条原gate**。
原始literal ID、report path、原文本及行号均保留。Runtime213的P1-001..048映射Runtime94，
48条为NONNEW；P1-049..060与P2-001..012是其当前报告24条自有finding。Runtime172和
Editor232共58条successor finding与36条gate保留为NONNEW，未新增原finding或gate。
Runtime29的5条继承owner决策不是finding；Editor16的60条P1和12条P2仍为null original ID，
只使用原ordinal定位。泛用P0/P1/P2标签可跨报告重复，原身份必须按报告限定。

Runtime09B验收矩阵25行中只有369–377行的9项9.3硬检查计gate；16条维度/指标/要求另存，
378行统计工具要求不计执行gate。封存33个当前源码与12个Unreal主参考的原始输入和
before/after字节，selected dirty状态仅限该范围，未请求whole-tree quiet。声明但缺失的
LandscapeStreamingProxy参考仍记录缺失；没有生成该文件或把参考捕获写成竞品性能证明。

[第24批记录](2026-10-03-w0-batch24-summary.json)与独立复审
`176a0d9b638d727087c21a390f05401115e006400454e51002e188764a45920d`
绑定原作者sealed artifact manifest `3753601a488b8659e45e4b372e73bc90308ffb14e2defc0b7ace6b3e4c3d1989`
及3份distinct errata。所有源码、GPU/Editor实测、fault/scale/soak、产品及性能/Jenkins门继续开放；
本步Rust编译/测试为0，B25尚未纳入。高级VirtualGeometry typed executor候选只按最低责任模块
准备，尚未实现或验收真实GPU调度/资源/dispatch/readback链路。


## 2026-10-03 W0 B25 物理与动画原身份复核

第25批纳入49条历史finding及95条当前报告finding，新增96条声明gate。累计为
**1630观察 /1626 report-qualified keys /1396不同literal ID /84条无标签finding ordinal /803条声明gate**。
08a/08c的原始`P1-1`等literal ID按报告限定；22条历史gate保持null原ID，以原行号定位。
当前Physics/Animation各自报告的74个literal gate不与历史行合并。Runtime219/221的15条
currentness/finding刷新和10条gate刷新仍为NONNEW，未增加finding或gate总数。

[第25批记录](2026-10-03-w0-batch25-summary.json)绑定Root冻结88个原始artifact（2369259B）与独立复核
`88474d1b4ee8700968fedec4acaf0502d5113d9f9870c359adf91cac3ca468b3`。
49条历史finding到当前owner只建立一对多ledger，未宣称精确逐项映射或修复完成；6份报告、
73份源码/参考捕获和原r1/r2 errata均保留。B24+B25的NONNEW刷新小计为121条finding/currentness
及46条gate，未推算更早批次的全局刷新总数。原始Physics/Animation产品合同、真实后端、
source/compiled pose、normal consumer、碰撞cook/save/load、超时/重试、Windows native、
Editor/性能及Jenkins门继续开放；此步源码修改及Rust测试为0，全目录覆盖仍未验收。


## 2026-10-03 W0 B26 原报告身份补录

第26批仅补录Runtime94与Editor138的137条原finding（P0=5/P1=108/P2=24）和80条原gate。
累计为 **1767观察 /1763 report-qualified keys /1533不同literal ID /84条无标签finding ordinal /883条声明gate**。
217条原报告行与封存字节一致；按报告路径加原ID扫描现有记录未发现重复。
Editor138的G01–G32仅与其他报告的未限定ID重名，不合并身份。

[第26批记录](2026-10-03-w0-batch26-summary.json)仅保存Runtime213第52行实际列出的六组Runtime94后继范围关系，均为NONNEW。
48条Runtime94原问题保持开放；不建立逐项配对。三组数量不一致为12对9、9对10和6对8，
数量相等的其余分组也未据此接受逐项语义映射。
Editor138到Editor232、Runtime94的P2与gate关系仍未确认。此步仅补录原始身份，
未宣称当前生产源码已修复或验收；源码修改与Rust测试为0，所有原GPU、Editor、
native、产品、性能及Jenkins门保持开放，全目录仍未完成。


## 2026-10-03 当前源码与独立本地证据边界

- Hub 当前 R3 服务二进制在相同封存输入上由 D 盘正常 Cargo 编译取得 exit 0；产物为
  18,273,792B、SHA256 `ae819c721046d39acabaf52fb1c0112ddc9d37151928b58b4b792ed5e37799c2`。
  最初 runner exit 1 的错误探测路径和实际产物分别保留于 [Hub actual binary](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-r3-d-binary-root-terminal-reconciliation-r2/root-terminal-reconciliation.json)。既有 96 项普通库测试窄域复用。
  23 条正常 API、13 条负例与干净 Keycloak 代次仍未通过；[Hub native review](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/B/hub-r3-guarded-native-independent-review-u-r1/review.json) 的路由断言、历史数据
  输入和信号前进程身份缺口正在独立新验证包修复。
- 原 UI/render 的 R9 编译在 `2026-10-03T18:17:10.393416Z` 以 101 退出，Scene 测试把
  `spawn_node` 的 Result 传给 EntityId 消费者。R10 仅在两份既有测试的三个调用加 `.unwrap()`；
  场景、断言和原注释保留。封存输入 29,633 文件/304,786,436B，独立编译已启动，原回归尚无通过
  终态。见 [R9 failed compile](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/runtime-foundation-r9-root-failed-compile-exact-qualification-r1/qualification.json)、[R10 source](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/runtime-foundation-r10-root-source-publication-r1/terminal-source-publication.json)。原 request/SID/三个生命周期保持开放；匹配原代次的 510 项 RHI
  证据仅窄域复用，原 build-editor、live Penpot/source/token/font/media/locale、产品与性能门开放。
- [Interface source](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/runtime-interface-hub-r3-root-parser-source-publication-r1/terminal-source-publication.json)、[Sound R11 source](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/sound-r11-root-source-publication-r1/terminal-source-publication.json) 与 [VM R8 source](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/vm-r8-root-source-publication-r1/terminal-source-publication.json) 分别限定尾逗号解析、退休期间最终提交和物理编译输出目录的
  源码边界。各自源码收据 tests_run=0；正常消费者、VM shared/static + Rust/CTest、原生后端、
  Editor、产品、性能和 Jenkins 验收仍开放。历史 placeholder driver 没有执行。
- [W0 remaining](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/w0-remaining-scope-inventory-r3/summary.json) 保留原 576 份报告分母，其中 503 份尚无合格 canonical 身份记录。
  [9 supplemental paths](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/w0-nine-supplemental-path-classification-nav-r1/manifest.json) 将额外 9 路径分类为 8 份修复/状态交接与 1 份研究资料，沿用原主报告身份；
  未增加主 finding 或通过数。B26 的 1767 观察/1763 report-qualified keys/1533 literal ID/
  84 null ordinal/883 gate 不变。

本地证据按已授权的协调器退休规则执行；原归属、索引、失败与 queued evidence 保留。
待验证状态仍开放，全域目标尚未完成。


## 2026-10-03 W0 B27 与 R11 输入边界

[第27批摘要](2026-10-03-w0-batch27-summary.json)记录R04/R05/R06/R07/R08B/R08D/R08E/
R08F/R08G/R09A/R09C/R09D的275条原finding（P0=24/P1=202/P2=49）和212条原gate。
286处原finding位置的11处重复保留为观察，原ID按报告限定。累计
**2053观察 /2038 report-qualified keys /1533不同literal ID /84 null finding ordinal /1095 gate**。
本批43种literal ID已在其他报告出现，全局literal数不增加，跨报告身份独立。
21条既有baseline/关系行保留NONNEW，source path仅为归属提示；当前源码、原生产品、
性能与Jenkins尚待验证。576份主报告与额外9路径分类不变，未接受全目录覆盖。

原R10编译于2026-10-03T20:05:14.803450Z实际退出101；完整stderr
`44b43e3715fdcd4f4549ac7ff1486c30e8def5a1e685635f5d0ef5f2cc893eb7`保留19条诊断。
[五项R11源码收据](../../../../../.codex/tmp/astra-current-evidence-20261004-root-01a0f033/runtime-foundation-r11-root-source-publication-r1/terminal-source-publication.json)
只修复原测试的boxed receipt、owned winit handler、rich compiled parse消费、SceneResult
解包与真实render输入；原断言、ignored原生/性能用例和注释保留。
[新输入](../../../../../.codex/tmp/astra-current-evidence-20261004-root-01a0f033/runtime-foundation-root-materialization-r11-r1/preparation-terminal.json)
为29,633文件/304,786,878B，manifest `9f46aa68ffe7bb2121ecb7e08cfc076ee4c810dc06687b0be54cc21a79b43979`，
仅原R10加五个postimage。原声明回归链与15项受影响普通测试尚无通过终态。
原request/SID/三生命周期、build-editor/live Penpot/source/token/font/media/locale、
产品/性能/Jenkins均开放；当前36处其他消费者源码变化由另一代次复审。


## 2026-10-04 W0 B29 与 R12 精确输入

[第29批摘要](2026-10-04-w0-batch29-summary.json)保留22份插件原报告的1,448个独立报告限定键，
其中1,256个literal ID、192个无编号severity ordinal；P0=42/P1=1,136/P2=270。
累计 **3501观察 /3486 keys /2634不同literal ID /276 null ordinal /1095原gate**。
3,526个混合上下文不是3,526条验收门；430条checklist分类包含标题，原gate分母不增加。
910条作者strip()序列化差异已从报告原行保留缩进。22条报告关系和16条历史P0移交仅为NONNEW。
本批未证明当前插件source/native parity、provider、正常产品或性能；576+9分类不变。

原R11完整编译于2026-10-03T23:07:21.177633Z退出101，仅两条E0425；stderr
`09ec399a3c5aae41ae05a2b4e85e7ff4668f411a455b2c32f00023d5b1b26ac8`，原回归未启动。
[R12源码](../../../../../.codex/tmp/astra-current-evidence-20261004-root-01a0f033/runtime-foundation-r12-root-source-publication-r1/terminal-source-publication.json)
只修改lighting测试的辅助签名，postimage `dcae76d5b3f71891cf30480968fcfd91b6d2aa8aa16e22aae0d35ddefe23f137`，
[独立复审](../../../../../.codex/tmp/astra-current-evidence-20261004-root-01a0f033/runtime-foundation-r12-test-type-independent-review-u-r1/review.json)
确认唯一调用仍克隆共享句柄，未改工作量、断言、ignored profile或生产API。
[新输入](../../../../../.codex/tmp/astra-current-evidence-20261004-root-01a0f033/runtime-foundation-root-materialization-r12-r1/preparation-terminal.json)
仅R11加这一个postimage，29,633文件/304,786,805B；编译与22原回归+15受影响回归正在执行，未取得通过终态。
原request/SID/三生命周期及所有lower/original/upward/live产品/性能/Jenkins门保持开放。
当前其他消费者与独立VM/Hub/领域provider的新代次不加入原UI输入；旧failedcopy未手工修补。
