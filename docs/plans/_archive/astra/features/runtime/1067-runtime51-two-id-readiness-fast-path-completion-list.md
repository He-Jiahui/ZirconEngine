---
doc_type: feature-completion
candidate_id: runtime51-two-id-readiness-refresh-release-profile-v4
status: implementation_applied_validation_pending
implementation_status: source_and_profile_applied
validation_status: rustfmt_passed_semantic_parity_unexecuted_release_profile_unexecuted
review_status: pair_v4_and_many_streaming_delta_static_reviews_passed_cargo_pending
measurement_status: ignored_release_profile_pending
product_status: product_gate_open
record_target: docs/plans/astra/features/runtime/1067-runtime51-two-id-readiness-fast-path-completion-list.md
plan_sources:
  - docs/plans/optimize/zircon_runtime/51/2026-08-24-resource-readiness-single-refresh-fast-path.md
  - docs/plans/optimize/zircon_runtime/51/2026-09-29-resource-readiness-two-id-fast-path.md
  - docs/plans/mvp/02-f1-project-and-assets.md
implementation_files:
  - zircon_runtime/crates/zr_resource/src/manager/resource_manager.rs
tests:
  - zircon_runtime/crates/zr_resource/src/manager/resource_manager.rs
  - zircon_runtime/crates/zr_resource/src/manager/tests/readiness_pair_profile.rs
  - zircon_runtime/crates/zr_resource/src/manager/tests/readiness_pair_profile/batch_profile.rs
---

# Runtime1067 / Runtime51 two-ID readiness refresh completion list

The v4 production source, regression coverage, ignored Release profile, and this completion list
are applied to the shared checkout after exact ownership and preimage guards. Managed validation
and actual performance acceptance remain pending.

| Acceptance item | Candidate evidence | Required gate | Status |
| --- | --- | --- | --- |
| Preserve empty and one-ID behavior | Empty input retains readiness generation identity. The existing one-ID publication and unchanged-generation regression remains. | Compile and run the focused `zr_resource` tests in the grouped managed Runtime batch. | pending |
| Normalize an exact pair | Distinct pair IDs are ordered before source updates; duplicate IDs use one update. Three-or-more inputs retain sorted/deduplicated ID normalization, including the item peeked for cardinality; the current follow-up streams source updates without a second update Vec. | Run reverse-order, duplicate-pair, and larger-batch readiness regressions. | pending |
| Preserve missing/lost readiness semantics | Removing a pair member removes its published readiness row; a surviving record whose dependency is missing remains `Failed`. | Run readiness behavior tests against current source. | pending |
| Structural performance result | Exactly-two input normalization avoids the ID and update `Vec`s and its sort/dedup. Projection scratch inside `apply_updates` remains. | No whole-operation allocation-free claim is made. | structural lower bound only |
| Paired exact-two Release profile | An ignored test compares the pre-fast-path Vec/sort/dedup/update-Vec algorithm with the current pair API over distinct, duplicate, dangling-dependency, and lost-row fixtures. It checks semantic parity before timing. Fixture setup, state rebuild, and drop are outside each timer; black-boxed input and readiness diagnostics are inside both timers. Three warm-up pairs and 101 alternating measured pairs produce raw samples and P50/P95; P95 uses rank 96 of 101. | Run the ignored profile in the grouped managed Release library test for package zr_resource using explicit -p zr_resource; record the raw output and environment before drawing a numeric conclusion. | pending |
| F1 M2.2 relevance | The path contributes to Runtime registry/readiness publication used by the F1 asset path. | F1 project creation, current-generation registry/import, reopen, and product gates remain separate. This optimization does not close or unblock F1. | open |
| Product performance | No latency, allocation-rate, RSS, energy, or product result is claimed. | Select an applicable current-source workload and device/profile, then collect managed Release and product evidence. | open |

Scratch rustfmt and isolated patch replay passed. Independent v4 static delta review passed. The semantic parity test and ignored Release profile were not executed; no Cargo validation or Release samples are claimed. The v4 manifest records source preimages and candidate/patch hashes. This record does not claim managed validation or product acceptance.

## 2026-10-01 后续会话与批量验证输入

旧会话 `astra-optimize-20260926-batch-a` 已被外部管理操作取消，其来源和历史回执保留。当前 chat 的实际后续会话为 `01a0df1a-f2b0-7480-948f-ccc4ba982d12`，基线为 `bc02eefafead65dbf5050482110e8175250a5e77`、epoch 628。相关两处 Resource 源码及四份 Runtime/Editor 记录于 07:50 UTC 完成精确 public 移交和 live lease 领取；未撤销外部租约。

当前 Resource 库验证输入由实际模块声明、依赖及编译期资源导出，共 630 项：84 项 Resource 测试构建 Rust 来源、494 项 Interface 依赖 Rust 来源、7 项 Math Rust 来源及 manifest、build/spec/schema、真实模板和 workspace metadata 输入。对后续会话基线重算为 291 项当前 overlay 与 339 项基线等价输入。来源清单记录 `offline-candidates/root-resource51-current-native-baseline-v2/receipt.json`，SHA `d4cdc05c165cf27756176b8c653863394f11e03c347e08a562485fa9b0fb23af`。保留当前 package service metadata 入口的外部新字节；Resource `--lib` 不编译该入口。

06:25 UTC 的 293 路径 public 预览含 197 项有效外部租约；这阻止完整来源准入。日志 `2026-09-30-resource51-native-successor-exact293-inputs-and-records-public-preview.json`，SHA `90fd69c171721de1f874090220f47dacc00b529603accc940f752a15566d51ff`。六路径后续移交不关闭这些依赖准入缺口。

已准备同包同来源的三个批量 lane：`zr_resource --lib` 的 locked check、普通全部库测试及 ignored Release 两-ID原始样本 profile；使用 `+1.94.1` 与受管 direct-network 配置。当前未提交、未取得 ticket、未运行 Cargo，仍需当前来源准入和完整 seal。准备稿不是验证通过；源码等价清单不是编译结果。不连续查询编译状态，不改 tooling，不缩小真实依赖来源以绕过阻塞。

## 批量验收完成清单

- [x] 应用两-ID固定数组路径及真实语义回归、配对 Release profile。
- [x] 为当前后续会话整理并重算 630 个真实 Resource 库验证输入。
- [x] 准备同来源 check、全部普通库测试、ignored Release profile 的三 lane 批量请求。
- [x] 应用并登记多 ID 更新 Vec 去除及真实语义/不变代际回归，完成独立静态审查和格式/精确重放检查。
- [ ] 关闭外部租约与来源归属准入缺口，完成实际受管提交。
- [ ] 修复分组编译/测试实际失败并取得终态通过回执。
- [ ] 归档原始 Release 样本并按适用产品预算证明性能达标。

## 2026-10-01 多资源刷新后续修复

实际共享源码已删除三项及以上输入的第二个 `Vec<ResourceReadinessSourceUpdate>`。ID 输入仍先完整收集、排序和去重，随后借用互不重叠的 registry/runtime/payloads 字段，逐项把更新交给 readiness 投影。空、单 ID、双 ID 路径保留既有行为；源记录仍克隆，ID Vec、依赖闭包和发布 scratch 仍存在，不宣称整次操作无分配。

同一个私有来源构造 helper 供各路径使用。当前 `ResourceData` blanket `as_any` 只返回 self，源码读取无用户回调；如果后续来源构造或克隆引入可 unwind/副作用，必须重新审查逐项读取与旧 eager 构造的失败边界。本次没有新增事务性 panic 保证。

增加四种真实 fixture 的多 ID 重复输入与 legacy 算法等价回归，并验证重复刷新保留 Arc 代际；既有三种资源批量回归增加 changed_row_count 和不变代际检查。测试先在私有候选中写出，尚未执行 red/green，既有 ignored 双 ID Release 配对 profile 保留。

- production `resource_manager.rs` SHA：`f5d478f0a4e9c4131e7e730df62fa65037919e885531e26f3bbd892c8cc5014b`。
- `readiness_pair_profile.rs` SHA：`ebe4e5d7d22d8483d8b1c20bdbe6ff78b6f2b03086539c3abd516a257e8f2972`。
- 合法 lease、当前字节 guard、两处写入和 baseline.attribute 日志：`2026-10-01-runtime51-many-readiness-update-streaming-two-source-guarded-apply.json`，SHA `8b4f783432743e5ea925468c283d4ecbc58a64f45bb6caa187054cdf4c4d8fdd`。
- 格式、scoped diff、两目标精确隔离重放均通过；replay 回执 SHA `0fd2afe283563659413cf97050422d8b6e65665c8a3074e1a2576950a0fb03c0`。
- 独立 gpt-6-luna max 增量静态审查无阻塞发现，接收并核实来源记录 SHA `9ec23a4217e99992843d75c2bd6bc7673d5a7c7d3b88ea505915177560db6969`；不代表编译或测试已运行。

实际源码与四份记录完成后，630 输入仅上述两处来源发生变化，291 overlay 路径集合不变。重查回执 `offline-candidates/root-resource51-post-streaming-current-batch-v3/current-input-recheck.json` SHA `3f26d1a74cf8f9e63c76bac8f707993add7fb638f526758977fdcd43d19edf42`。一次实际修复节点的 public 准入检查：除已归属本会话两处来源，剩余289项中92项eligible、197项ineligible；196项有有效外部租约，package service 项由 registered executable 所有者占有。日志 `2026-10-01-resource51-post-streaming-exact-remaining-input-admission-preview.json` SHA `6e6a7aa381bc40f8853eaf3b159b7b05e0375233bf64d6d201bfcc885134c753`。未执行移交、编译提交或状态监控。批量 check、普通库测试与 Release 结果仍开放。

## 多资源批次性能夹具

当前应用新增 `readiness_pair_profile/batch_profile.rs`，并由父模块声明加载。普通回归覆盖 3/64/1024 个不同资源的依赖链，四类输入为变化、变化且重复、无变化、无变化且重复；比较真实旧 Vec 算法和当前流式算法的全部行、diagnostics，以及无变化时的 Arc 身份。

ignored Release profile 对 12 个同工作量情形分别取 3 对预热和 101 对交替样本，输出原始时间、P50/P95/P99、分配次数、请求字节和相对 peak live 字节。计时在分配计数器关闭时运行；分配来自另一份等价初态的独立调用。计数器为已有的进程级采样，要求隔离单线程执行；这些字节不是进程 RSS。

- [x] 批次夹具已实现，两个源码目标精确回放和格式检查通过；应用时重新校验 preimage 与 lease。
- [x] gpt-6-sol 独立批次夹具静态审查完成，三个实际源哈希匹配，未发现确认的阻塞缺陷；执行验收保持开放。
- [ ] 普通批次语义回归执行并通过。
- [ ] Release 原始样本与两-ID profile 在同一受管 lane 执行；当前未采样。
- [ ] 从真实结果判断该批次是否改善延迟与分配，并满足原计划产品门。

Release lane 的测试过滤应为 `readiness_pair_profile`，参数 `--ignored --nocapture --test-threads=1`，统一覆盖原来的 pair 与新的 batch 两个 ignored profile；普通库测试保留全部非 ignored 回归。上述实现与静态检查不构成性能达标证据。


## 当前多规模来源与合并批次（631 输入）

新增 batch_profile 的两个源码路径与两份记录已实际应用并 baseline.attribute，生产流式刷新源仍为 `f5d478f0a4e9c4131e7e730df62fa65037919e885531e26f3bbd892c8cc5014b`。父 profile 当前 SHA `5386ec21411091d8739bf0f2512b9ec39d567461e254a2b52f89b9bd605fc0d1`；新增 batch_profile SHA `1947a409cb6555fee304f748637cbe9720e6940658de500eee7b7a6ffddb67c9`。合法 guard、lease、应用与归属回执 `2026-10-01-runtime51-many-scale-profile-two-source-two-record-guarded-apply.json`，SHA `20fe2166726ac7092b2460f99abc55c8035a6dfcc7e2cb54f36ce64abd4b2c8b`。

当前源码闭包为 631 项、292 overlays，其中 3 处 Resource 源码属于本会话。当前输入重查 `offline-candidates/root-resource51-many-scale-current-batch-v4/current-input-recheck.json`，SHA `faf58663b373b857a023639c0fe85be0dcf2ca8039001bf4880699894c28499a`。同一次源码节点准入检查的剩余 289 路径为 91 eligible、198 ineligible：196 个外部有效租约，以及 Hub manifest 和 Runtime package service 的两个 registered executable 所有者；保持其真实当前字节。日志 `2026-10-01-resource51-many-scale-exact-remaining-input-admission-preview.json`，SHA `57a70bef740b317f6f21e2e2c4567466f8c28c6e957459a20ae97b710a1c866c`。

合并的三 lane 请求位于 `offline-candidates/root-resource51-many-scale-current-batch-v4/prepared-three-lane-requests.json`，SHA `8a9f676035571d99f866cbdf380ba6e6e5bc25c057e91fb1081b2a1b4472098d`；来源 fingerprint `d19e7820e9ae90d55b9f1559edb95ca1e48fa962e66119c40477cb3064983ffe`。普通 lane 保留全部库回归；Release lane 统一覆盖 pair 与 batch profile，`--ignored --nocapture --test-threads=1`。当前三源独立静态审查已返回，记录见下文；完整来源准入和实际执行仍开放。

独立审查 `offline-candidates/root-runtime51-many-scale-profile-v1/sol-independent-review.json`，SHA `e716e63e7b27ec7ca1dad409bca23b7003c6e58172b074a0f25c297ff5c0e5eb`，findings 为空。结论只覆盖当前三源：批次夹具为 Pending/Loading 依赖链，没有 Ready payload 或 runtime 状态变化样本；相对分配峰值不是 RSS；逐项流式构造的未来可 unwind 回调仍须重审。静态审查未执行 Cargo、回归或性能采样。

当前三 lane 未提交、ticket 为 0、Cargo 未执行；准备稿和 public 预览不是通过验收。继续独立功能修复，到有实际修复结果的节点再核验已有异步回执，不实时跟踪编译。前面的 630/291、197 阻挡和父 profile 旧 hash 属于较早历史节点，不表示这次源码闭包。

## 2026-10-01 Ready / Failed 真实 payload 回归

在既有普通 `readiness_payload.rs` 中增加三项回归；沿真实 public register/reload/fail/retry 路径检查 payload type 更换、pair / many dependency Failed → Ready 传播、重复和次序输入、旧 generation 不可变与 unchanged Arc 保留。原先未跟踪源码的既有字节已归档；不是重建或覆盖旧测试。

源码应用回执为 [本批源码应用回执](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime-two-contracts-apply-v1/30path_guarded_source_and_record_apply.json)，SHA `341298c648148d7d0b694237a991e883ef775ea7a9d9e549ddd75cec99c9c85d`。冻结 [manifest](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime51-ready-transition-regressions-v1/manifest.json) SHA `834836d60fecd406572806b8fc0d6c7a996501d61f87731713eebfaceeba4581`；[独立静态审查](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime51-ready-transition-independent-review-v1/review.json) SHA `73b32832b0bd90a8a540ed32556537ab1377cfbf810042c9e576bf1106075672`，无新增必修问题。

- [x] 准备三项真实 Ready / Failed 回归并完成精确重放、格式和独立静态审查。
- [x] 应用真实 payload 回归，核对 live lease 和来源归属，并保留原未跟踪源码。
- [ ] 重新封存完整验证来源并取得 managed 准入回执。
- [ ] 在兼容批次中执行全部正常 Resource 测试，并修复实际失败。
- [ ] 执行统一 pair / batch Release profile；G39 10K / 100K / 1M、真实 RSS 与数字预算继续保持开放。

静态审查与测试源码不是已执行测试或性能证据。没有 Cargo、测试或性能达标声明；V5 的 631 / 292 来源计数属于应用前快照，应用后必须重新封存。

应用与归属检查以本批回执的实际终态和文件 SHA 为准；若回执为 pending 或 failed，相应源码步骤保持开放。正常测试、Release 和性能预算须由后续 managed 批次独立验收。


## 2026-10-01 · Ready/Failed 应用后验证批次封存

[当前 V6 三个兼容验证任务](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-resource51-current-after-ready-apply-v6/three_compatible_requests.json) SHA `d0a314c969c7e85b653b559e0a68e75b215729765d3e2d28ebccbd35ae472a54` 与 [631 输入 source seal](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-resource51-current-after-ready-apply-v6/post_apply_source_seal.json) SHA `714653ceabd7bf5b697032b20063124541402fe0cfb7cb9c74f05dc6ce10eb0b` 已按已应用的 Ready/Failed 回归重新封存：631 个 current 输入、293 个 overlays；三个兼容任务为 Resource library check、完整普通 lib tests、合并 pair 与 many readiness 的隔离 Release profiles。三者均未提交，未生成新验证 ticket。

仅 readiness_payload 和外部当前 zr_resource/Cargo.toml 与上一冻结不同。Cargo manifest 新增 test-support+profiling integration target；这三个显式 --lib 任务不构建该 target，其他包/依赖/features/lib 配置字节保留，未豁免更广 boundary 验证。外部当前 manifest 与另外六个所需 executable 来源须依法准入，不能换成 Git committed bytes 或人工抢占归属。

- [x] 保留已应用 Ready/Failed 源码与记录，并完成当前验证输入的精确重新封存。
- [ ] 完成全部 current overlay 归属和 live lease 后，在一次派发阶段提交三个任务并记录稳定 identity。
- [ ] 后续源码里程碑再核验接受回执；提交后继续功能修复，不等待或连续监控编译。
- [ ] 以实际 Check/普通回归/101 paired samples/allocations 和原产品数字门验收；当前性能仍 pending。

协调器离线请求日志已保存；offline 不等于已提交或已通过。没有外部 worktree committed-byte 替代，也没有重新冻结或抢占他人 executable 源码。

## 2026-10-01 当前源码与批量验收记录

- Ready/Failed 双 ID 快路径的先前共享写入及 source attribution 保留；本轮没有改该 Rust 实现，也没有复用历史测试为当前性能通过结论。
- Resource V6 已准备三个兼容验证 lane（库检查、完整正常库测试、组合 Release profiling），631 输入 / 293 overlay；仍未提交，当前 foreign source/manifest attribution 与具体所有权门未闭合。不得将 pending/dry-run 计为通过。
- Runtime Core V3 支持回归和 Scene V4 预算桥已独立静态审查；Runtime UI 与 Editor 的实际 Core owner 和调用迁移继续在私有完整 cohort 中修复，整批编译门保持开放。
- [ ] 完整正常测试与多 ID/小队列回归通过。
- [ ] 实际组合与隔离性能样本满足原计划数值预算。

## 2026-10-03 当前测试与性能验收

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
