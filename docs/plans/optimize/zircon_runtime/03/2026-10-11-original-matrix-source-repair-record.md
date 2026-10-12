---
related_code:
  - zircon_runtime/tests/runtime_profiling_recorder_performance.rs
  - zircon_runtime/src/core/runtime/diagnostics/store.rs
  - zircon_runtime/src/runtime_diagnostics/collect.rs
  - zircon_runtime/src/foundation/runtime/config_manager
plan_sources:
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
doc_type: source-repair-record
title: Runtime03 original matrix source repair record
date: 2026-10-11
implementation_status: four_source_packets_adopted_native_performance_pending
validation_status: native_performance_product_pending
performance_status: original_matrix_not_run
---

# Runtime03 原矩阵 Source 修复记录

日期：2026-10-11。本文登记原 Runtime03 §9.4 profiling、DiagnosticStore 与 Config persistence 三组完整矩阵的 Source proposal，并附上同日实际采用更正。四个 Source packet 已应用到 canonical；本文没有运行 Cargo、Rust/native、Release producer 或产品性能采样，也不宣称验收通过。

## 原矩阵合同

| 所有者 | 原维度 | 需要观察的内容 | Source proposal 范围 |
|---|---|---|---|
| Profiling recorder | inactive/active；threads 1/8/64；events per thread 0/100/10,000 | elapsed/ns per event、allocation count/bytes、lock wait、drop rate、retained bytes、workload disturbance | 在既有 ignored producer 中保留 active 45 个样本，并加同形 inactive 45 个样本。 |
| DiagnosticStore | series 1/541/10,000；history 1/64/1,000；consumer 0/30/60/120 Hz | 每帧分布、writer/reader CPU 与完整性、历史覆盖守恒、实际 cadence 与线程退出 | 一个 Windows Release ignored producer 产生36个矩阵行；每个 cell 3 repeats、每 repeat 360 writer workload frames。 |
| Config persistence | payload 1B/1MiB/100MiB；不同 key burst 1/1,000；完成方式 flush/last-owner shutdown | serialization、stage/write/atomic replace、return dispatch、coalescing 与 shutdown flush | 一个 Windows Release ignored producer；6 shape × 2 completion modes × 5 samples = 60 个观测样本。 |

这些行数和样本数不是 Rust test case 数。Recorder 的 active 45 + inactive 45 是既有单个 ignored Rust test 内的90个采样情形，不是90个测试；DiagnosticStore 的36个cell和Config的60个样本也各属于单一 producer 测试。原计划§9.4没有冻结绝对耗时或分配预算；本次没有新增性能门槛。该 Source 工作不能替代 Runtime03 的 M1–M7、全产品性能、锁等待、帧扰动或 Jenkins 验收。

## Profiling recorder：inactive capture proposal

既有 [多线程 recorder producer](../../../../../zircon_runtime/tests/runtime_profiling_recorder_performance.rs) 是 ignored Release test，当前 active 组合为 3 thread counts × 3 event counts × 5 samples = 45 个采样情形。已冻结 candidate 在同一个测试内加入 inactive 对照，仍保留全部 active workload，因此总计90个采样情形，但仍是1个 Rust test case。

- [candidate patch](../../../../../.codex/outbox/runtime03-original-producer-inactive-capture-v1/candidate.patch) SHA256 19d941e501f8319820d03ae7a506a9334514332b94fb905f0786dde0e2cbd674。
- Root 独立 Source review SHA256 521bc7e755ffe096305832c9b194432e90ef48afc8bcb98d2ce392d7f84ef2ff（路径由 Root 保管）。
- 该静态 packet 盘点时 canonical adoption 尚未发生；同日实际采用了相同 patch SHA，采用回执见末节。native、Release/performance 与原计划验收仍 pending。

保留指标应区分活动 capture 与 inactive scope 开销；只有候选 Source 形状，不代表运行时已经产生或满足任何阈值。所列矩阵不自动补齐原计划的 lock wait、drop rate、retention 及 workload disturbance 量化；其预算需由计划 owner 冻结。

## DiagnosticStore：真实 authority 矩阵 proposal

冻结的 proposal 经独立 Source review 接受；下列文件是最初的静态 Source packet，实际采用更正见末节：

- [candidate-003.patch](../../../../../.codex/outbox/runtime03-diagnostic-store-original-matrix-v1/candidate-003.patch) SHA256 68f301ee9945a895734485b513d19b582376ab64b2dafbeec78238b294095afc。
- [independent review](../../../../../.codex/outbox/runtime03-diagnostic-store-original-matrix-v1/independent-review-v1.json) SHA256 8c2d234275fdf02881dd43238a5c9098d7cb83e098a1452f7b2c3f862090a9d5。
- [artifact association](../../../../../.codex/outbox/runtime03-diagnostic-store-original-matrix-v1/artifact-association-v1.json) SHA256 7ba867a2f9ca11afa42ae88d943f6be957803fc4f2b29659a637a6565f815e8b。
- 独审结果记录 source-only 状态：canonicalApplied=false、nativeExecuted=false、performanceResultsAvailable=false、fullRuntime03Accepted=false。实际采用发生在这份 review 之后；它没有提供 native 或性能证据。

proposal 通过实际 CoreRuntime owner 配置 DiagnosticStore，并经 CoreHandle::record_diagnostic 与真实 collect_runtime_diagnostic_store 路径执行；不以合成 helper 替代生产 store。矩阵是 series 1/541/10,000 × history 1/64/1,000 × consumer requested 0/30/60/120Hz，共36行。每个 cell 有3 repeats，每次360 writer workload frames；producer请求 writer 120Hz，总计181,533,240次 public record 调用（warmup 44,908,920，measured 136,624,320）。

0Hz 不启动 reader，测量区间不调用 collector；两个边界 authority audit 与 consumer reads 分开计数。读写线程在最终审计前 join。proposal 用 Windows GetThreadTimes 收集当前 writer/reader 自身 CPU 的原始整数计数；这仅覆盖这两个 worker，不是完整 Runtime CPU。实际读频率和 drain cadence必须从证据确认，不能由 requested Hz 推导为已达成。独立写入锁意味着 snapshot 可能看到部分 workload batch，不能称为原子 product frame。36行不等于36个Rust case；实际36个 cell、raw evidence、完整 footer、所有 worker 退出与输出 guard 尚未执行验收。

## Config persistence：atomic set 与原矩阵 producer proposals

### ConfigStore atomic set-if-changed

candidate 提议用一个 ConfigStore mutex guard 比较借用的旧 JSON 并发布变化值：相等值保留原 Arc identity，变化值 move 到 replacement Arc；相等的未使用输入和旧 map Arc 在显式释放 store lock 后 drop。CoreHandle 只增加 pub(crate) forwarding；public API bodies 与 load/snapshot ownership 不改。DefaultConfigManager 仍为相同值请求持久化并保留失败后的 retry 语义；RecordingWriter 的 lazy/closure 修正只处理既有 fixture 的 eager subtraction debug panic。

- [candidate.patch](../../../../../.codex/outbox/runtime03-config-atomic-set-if-changed-v1/candidate.patch) SHA256 104dd81f25c5d2b6c9ac369d146f87712069deac6ed56c61c18089d673a014dd。
- [initial v1 review packet](../../../../../.codex/outbox/runtime03-config-atomic-set-if-changed-v1/review.json) SHA256 bfde135342eec8fa7a47207a74b970e35cea8a6a757372ce520f714eb92ec4e6：当时 peer review 与 canonical adoption pending。实际采用选用 v2 packet 与 peer review，见末节。
- proposal 增加4个真实 regression cases，保留15个既有 cases、56个既有 assert 表达式和24个既有方法；这些正确性回归不等于性能矩阵。

### 原 ConfigManager/atomic writer 矩阵

proposal 使用真实 worker、stage 与 atomic writer/fence 链路。1B 是字符串 payload；完整 JSON 序列化大小另行报告。payload 1/1MiB/100MiB × burst key 1/1,000 是6种 shape；对 flush 与 last-owner shutdown 两种完成方式分别采5个样本，共60个采样观测，仍是1个 ignored Windows Release Rust test case。serialize 与 return-dispatch区间分别记录，不混成一个延迟值。

- [candidate-001.patch](../../../../../.codex/outbox/runtime03-config-original-matrix-v1/candidate-001.patch) SHA256 77947ad537e5b8c242dd2daa8f2df9207dd270a89eddc1b290f0ab61c5c537f6。
- [source-binding-001.json](../../../../../.codex/outbox/runtime03-config-original-matrix-v1/source-binding-001.json) SHA256 417e59735362233c611778cf5e471c748a8ca4541b32a122a7d80d5a59ed6cb1。
- [mechanical-evidence-001.json](../../../../../.codex/outbox/runtime03-config-original-matrix-v1/mechanical-evidence-001.json) SHA256 9e47969dfb799d54d08aabdf92128d6f68045993495384f51b28025e7563f421。
- 最初 candidate-001 静态绑定18个 source guards、643行 producer代码；最终采用 candidate-003（666行）的 Source binding 与 peer review见末节。

## 2026-10-11 实际 Source 采用更正

Root 的实际 apply 回执确认四个 packet 均已采用到 canonical Source。该回执报告 actual apply exit 0、90个 guard observations、65个 unique guard inputs，10个受管理 Source 文件的实际 postimages 精确绑定；combined source check exit 0。它也记录 sourceCopies=0、gitIndexWrites=0、foreign controls（包括 `.git/index` 与 editor generated projection）前后保持相同。以下版本号以实际 apply receipt 的 packet bindings 为准：

| Source packet | 实际采用版本 | review | Source 状态 |
|---|---|---|---|
| Recorder inactive capture | [candidate.patch](../../../../../.codex/outbox/runtime03-original-producer-inactive-capture-v1/candidate.patch), SHA256 `19d941e501f8319820d03ae7a506a9334514332b94fb905f0786dde0e2cbd674` | Root review `521bc7e755ffe096305832c9b194432e90ef48afc8bcb98d2ce392d7f84ef2ff` | 已采用；保留原active 45并增加inactive 45个采样情形。 |
| DiagnosticStore original matrix | [candidate-003.patch](../../../../../.codex/outbox/runtime03-diagnostic-store-original-matrix-v1/candidate-003.patch), SHA256 `68f301ee9945a895734485b513d19b582376ab64b2dafbeec78238b294095afc` | independent review `8c2d234275fdf02881dd43238a5c9098d7cb83e098a1452f7b2c3f862090a9d5` | 已采用；36 rows 属于一个 ignored producer。 |
| ConfigStore atomic set-if-changed | [candidate-v2.patch](../../../../../.codex/outbox/runtime03-config-atomic-set-if-changed-v1/candidate-v2.patch), SHA256 `28b7a11b7e61801a73c9925c338f1c9bf9580f55a5c029af65011961761d184f` | Root peer review `31fb4c3582968aa8d0592220250cdde521433dc25280292c4821b0c769638b85` | 已采用；v2 为采用版本，原 v1 review仅为历史 packet。 |
| Config original matrix | [candidate-003.patch](../../../../../.codex/outbox/runtime03-config-original-matrix-v1/candidate-003.patch), SHA256 `4701c1b0ef4e8d9b83c8f9ab2d21bc520597c4c0b9fecc654129becd59463b94` | Root peer review `7fc04908d7f19f80c9a39cb4158dcc5f5218c6873b3087af2d2878d69074a12b` | 已采用；666行 producer包含最近64项窗口与错误路径修正。 |

唯一实际 apply 回执为 [actual-canonical-apply.json](../../../../../.codex/outbox/runtime03-four-source-repairs-actual-adoption-20261011-v1/actual-canonical-apply.json)，SHA256 `1cf1409dc5a508463bc983dee01f6d35f5d12504b6254e07305a12e9520e8908`，26,642 bytes。回执确认10个受管理 Source files 的 raw/LF SHA 与 bytes、postimages及外部控制前后值。它记载 `nativeRuns=0` 与 `fullPerformanceAccepted=false`。因此 Source adoption 已完成；所有运行期矩阵、性能与产品验收仍 pending。

2秒是producer/worker安全等待参数，不是 Drop 的整体deadline。每种 shape 仅5个样本，p95 不足以构成稳定性能预算；原计划没有为此指定绝对阈值，本 proposal 不新增阈值。Flush/shutdown native raw samples、context/source fingerprints、输出目录物理 guard、线程终止证明及所有矩阵完整性均未验收。

## Request05 prelaunch source-guard refusal 记录

根据 Root 提供的 closed checkpoint-002 摘要，request05 在启动 Cargo 前因 source guard 拒绝：cargoInvoked=false、source-guard-refused，涉及输入 .git/index 与 zircon_editor/src/ui/retained_host/ui/pane_data_conversion/generated_bottom_projection.rs，时间 2026-10-11T14:20:47.320432Z。Root 报告的 closed receipt SHA256 为 60d094c80769669a5666eb175f66fca13963f742208b12f9af0eb686e7c8fb4a，checkpoint-002 SHA256 为 b9f6ab9e2348f5682b1704173e940cb4087dfea23d93af9802708ae913595d6a。不读取任何terminal或native文件；上述只是Root提供的closed摘要。该 refusal 说明这次 request 没有进入 Rust cases/performance producer；它不验收或否定上述 Source proposals。

## 后续验收

所有 proposal 在实际采用前须按各自 review status 完成独立复核，并对受影响 canonical Source 做新鲜 guard reconciliation。随后按授权的 Windows Release 环境一次运行完整原矩阵，保留 raw samples、源/硬件/profile、噪声区间、完整输出与线程终止证据。不存在的观测保留为 pending/null；不得以0代替。

- 四个 Source packet 的 canonical adoption：已完成，见上方实际采用回执；native/runtime evidence 仍 pending。
- Recorder active+inactive 原矩阵及其性能预算：pending。
- DiagnosticStore 36 cell 与实际 cadence、writer/reader CPU、完整退出/输出 evidence：pending。
- Config 60 samples、atomic persistence、安全完成与真实 shutdown deadline：pending。
- Runtime03 M1–M7、锁等待、完整产品性能、Jenkins 与 milestone acceptance：pending。

见[Runtime03 原矩阵 Source completion list](../../../astra/features/runtime/2026-10-11-runtime03-original-matrix-source-completion-list.md)。
