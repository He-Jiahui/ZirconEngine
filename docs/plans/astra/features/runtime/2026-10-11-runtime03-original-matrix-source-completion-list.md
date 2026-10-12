# Runtime03 原矩阵 Source 修复完成列表（2026-10-11）

对应[原矩阵 Source 修复记录](../../../optimize/zircon_runtime/03/2026-10-11-original-matrix-source-repair-record.md)与[Runtime03 原计划](../../../optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md)。

本列表跟踪 Source proposals 到实际采用的状态。Root 的实际 apply receipt 确认四个 Source packet 已采用到 canonical；native/performance 与产品验收仍 pending。

## Proposal 与 Source review

- [x] 冻结原矩阵 proposal 的候选与 Source 边界；未复制 Source tree，未改 tooling。
- [x] Recorder inactive-capture packet 已采用：在既有 ignored test 保留 active 45 样本并增加 inactive 45 样本。90 是1个 Rust test 内的采样形状，不是90个 Rust cases。Root Source review SHA 521bc7e755ffe096305832c9b194432e90ef48afc8bcb98d2ce392d7f84ef2ff。
- [x] DiagnosticStore 真实 authority packet 已采用：36 matrix rows（3 series × 3 history × 4 requested consumer Hz），一个 ignored Rust case，3 repeats × 360 writer frames、181,533,240 次 public record calls。独审 source-only accepted。
- [x] ConfigStore atomic set-if-changed 已采用 v2 packet：4个新增 regression cases、15个既有 cases / 56个既有 assert表达式 / 24个既有方法保留。Root peer Source review SHA 31fb4c3582968aa8d0592220250cdde521433dc25280292c4821b0c769638b85。
- [x] Config原始完整矩阵已采用 candidate-003：6 shapes × flush/shutdown两种 completion × 5 samples = 60 observations，属于1个 ignored Rust case；采用版本666行 producer，并包含最近64项窗口与错误路径修正。Root peer Source review SHA 7fc04908d7f19f80c9a39cb4158dcc5f5218c6873b3087af2d2878d69074a12b。
- [x] 明确1B为字符串payload，完整 JSON序列化大小单独报告；serialize 与 return-dispatch区间分开。2秒是 worker安全等待，不是 Drop总deadline；五个样本的p95不增加为性能预算。
- [x] 明确原计划§9.4没有 absolute latency/allocation budget。本组proposal不替代M1–M7、lock wait、full product performance、Jenkins或milestone验收。
- [x] 登记 Root 提供的 request05 closed prelaunch refusal：source-guard拒绝发生在 Cargo invocation前，cargoInvoked=false。receipt SHA 60d094c80769669a5666eb175f66fca13963f742208b12f9af0eb686e7c8fb4a；未读取terminal文件。
- [x] 实际采用回执确认四个 packet 的 canonical Source apply exit 0；90 guard observations、65 unique inputs、10个 Source 文件的 raw/LF SHA 与 bytes postimages 精确绑定；sourceCopies=0、gitIndexWrites=0、foreign controls 保持不变。回执 SHA 1cf1409dc5a508463bc983dee01f6d35f5d12504b6254e07305a12e9520e8908。

## 未完成验收

- [x] 完成四个实际采用 packet 的 Source review 与 canonical guard reconciliation；apply receipt 精确记录65个 unique guard inputs和所有受管理 postimages。
- [x] 将四个 reviewed packet 实际采用到 canonical Source；source adoption 已完成，不代表运行期验收。
- [ ] 运行原 profiling active/inactive矩阵，核验全部90个采样形状及完整预算证据。
- [ ] 运行 DiagnosticStore全部36行；核对 raw samples、实际 cadence、writer/reader GetThreadTimes、full history conservation、joined worker 与完整 footer。writer/reader CPU不表示全 Runtime CPU。
- [ ] 运行 Config全部60 samples；核对1B字符串payload和完整JSON长度、序列化/stage/atomic replace/return dispatch以及flush和last-owner shutdown原始数据；独立验证真实进程退出与所需deadline。
- [ ] 完成 Runtime03完整原计划、native/performance/product、M1–M7、Jenkins与milestone验收。

一个矩阵行、一个样本、Source review或closed prelaunch refusal都不构成原矩阵通过。未来运行缺失项保持 null/pending，不填0。详见[Source repair record](../../../optimize/zircon_runtime/03/2026-10-11-original-matrix-source-repair-record.md)。
