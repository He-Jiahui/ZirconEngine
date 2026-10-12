# Runtime / Editor batch05 异步验证记录

日期：2026-10-11。稳定请求：runtime-editor-source-repair-batch-20261011-05。状态：本次请求已以 prelaunch source-guard refusal 关闭；cargoInvoked=false，原 15 条命令均未调用，0 Rust cases、0 performance samples。一次性关闭核验确认自有 worker 已退出。以下保留原异步派发历史；本次关闭不是编译、测试、性能或产品验收通过。

## Batch05 终态更正：prelaunch refusal

冻结 manifest ID 为 runtime-editor-source-repair-batch-20261011-05，SHA256 a3b8742b5959a445533517d8431e609c5eb354d5303467b5efddbadbe4006d1f。请求在 Cargo 启动前因 source guard 关闭，状态为 source-guard-refused；受影响输入是 .git/index 与 zircon_editor/src/ui/retained_host/ui/pane_data_conversion/generated_bottom_projection.rs。原 15 条命令全部 not-invoked，因此 0 Rust cases、0 performance samples，不能从这次尝试推导通过或失败的测试结果。

- [closed audit](../../../../.codex/outbox/runtime-editor-batch05-terminal-checkpoint-v1/prelaunch-refusal-closed-audit.json)：SHA256 7bc288763fed4df08487eccfac6ff3cff1f644af575736d433804298becb4a42，1,460 bytes。
- [checkpoint-002](../../../../.codex/outbox/runtime-editor-batch05-terminal-checkpoint-v1/checkpoint-002.json)：SHA256 b9f6ab9e2348f5682b1704173e940cb4087dfea23d93af9802708ae913595d6a，1,440 bytes。
- Source refusal evidence is recorded at E:/cargo-targets/zircon-local/source-repairs/current-main-20261011/batch-native-v5/actual-request-v1/prelaunch-refusal.json, SHA256 60d094c80769669a5666eb175f66fca13963f742208b12f9af0eb686e7c8fb4a, 258 bytes.

自有 worker 退出由一次性证据确认；没有等待进程或实时监控。该证据只关闭这次 prelaunch request，不代表 Native、full performance、Runtime/Editor 产品或 Jenkins 验收完成。Runtime03 四个 Source packet 已有独立采用记录；其 B06 统一准备及原矩阵运行仍待后续安排，见[Runtime03 Source 修复记录](../zircon_runtime/03/2026-10-11-original-matrix-source-repair-record.md)。

## 一次提交

- [冻结 manifest](../../../../../../cargo-targets/zircon-local/source-repairs/current-main-20261011/batch-native-v5/actual-request-v1/manifest-proposal.json)：SHA256 a3b8742b5959a445533517d8431e609c5eb354d5303467b5efddbadbe4006d1f；479 个当前命名输入、12 个 toolchain 输入、14 份实际源码采用回执。
- [最终独立绑定审查](../../../../.codex/outbox/runtime-editor-v10-static-entry-review-v1/actual-v11-request-final-binding-review.json)：SHA256 54cc70bc07fb8390e9961b6768c9f839d03e1982fa11867269e39eae3e49fc12。
- [启动回执](../../../../../../cargo-targets/zircon-local/source-repairs/current-main-20261011/batch-native-v5/actual-request-v1/dispatch-receipt.json)：提交于 2026-10-11T14:20:45.8708876Z；PID 28384；实际创建时间 2026-10-11T14:20:45.7941561Z。这些字段只记录进程提交，不表示 Cargo 或测试通过。
- 后台入口及 compiler wait 次数为 0；未读取进行中的日志、进度或 PID 状态。提交后继续 Runtime03 inactive-capture 对照和 editor 旧消费者契约修复。
- 本团队 canonical 代码在本批次运行期间冻结。既有 runner 仍严格检查完整主包及命名输入；共享索引或其他会话源码变化可能导致拒绝或 Source guard 失败。
- 冻结时 index.lock 存在，作为 479 个输入之一；本 manifest 没有移除缺失历史输入。更早的外部 lock 消失只属于当时观测，不等于当前事实。本团队未删除、重建或写入共享索引。

## 原批次合同

原 15 条命令与 14 个原性能 producer 合同保留。Runtime 第 2 项下限 22，Editor 第 15 项下限 191；新增选择覆盖 2 个既有 Wake case、12 个既有 fixture case、5 个 Generated case，其中仅 1 个新 Rust case。原 profile、feature、ignore 入口、workload、矩阵和阈值不降低。

Runtime03 仅原 producer 的 Source hash 因已审查采用的 cap-aware 修复合法更新；原 9 个组合、每组 5 次采样、最大 640,000 次实际写入及 131,072 hard cap 保留。

| 序号 | 原批次任务 | profile | 测试数量要求 |
| --- | --- | --- | --- |
| 1 | three-package-check | debug | 编译检查 |
| 2 | runtime-scoped-regressions | debug | 22 |
| 3 | editor-scoped-regressions | debug | 1 |
| 4 | runtime-boundary-integration | debug | 1 |
| 5 | editor-output-policy | debug | 8 |
| 6 | app-startup-diagnostics | debug | 1 |
| 7 | registry-authoritative-order | debug | 1 |
| 8 | profiling-enabled-regressions | debug | 12 |
| 9 | runtime-release-evidence | release | 10 |
| 10 | registry-g39-release-evidence | release | 1 |
| 11 | profiling-original-release-evidence | release | 2 |
| 12 | editor-release-evidence | release | 4 |
| 13 | profiling-chrome-owned-export-regressions | debug | 1 |
| 14 | profiling-original-export-release-evidence | release | 2 |
| 15 | editor-production-asset-contract-regressions | debug | 191 |

## 输出与终态核对

原请求计划复用项目配置指定的 E:/cargo-targets/zircon-local/win32/codex-default/target 与同级 build，并使用 E:/cargo-targets/zircon-local/cache；但 Cargo 未启动，所以这些只是请求中的目标路径，本次没有构建或缓存运行证据。没有创建源码副本或镜像；旧 D 盘构建、共享缓存和兄弟目录没有在本次文档关闭中清理或释放。

旧 runner 不读取 requiredNamed 字段或 minimum 数量，且其终态缓存审计仍固定描述 D 盘 cache/scratch；该历史输出不能证明本批 E 盘缓存与临时文件验收。既有 wrapper 按目标盘符检查实际 E 盘输出。由于本请求未启动 Cargo，没有本批 E 盘 compiler/cache/scratch 输出可供验收；若未来另行启动批次，须在该批次中核对实际路径与临时目录，并保留共享或未知兄弟目录。

上述终态对账是成功执行原请求时的验收要求。Batch05 在 Cargo 前即已关闭，未到达 binary、命名测试、producer marker、矩阵或阈值验收；这些结果记为未运行/未验收，不让本次关闭冒充性能验收。未来单独安排的批次仍须按其授权范围提供完整证据。

## 独立验收与产物生命周期

本请求曾经由现有 independent local-cargo 异步入口提交，但在 prelaunch source guard 处关闭，Cargo 未调用；它不是 Jenkins、milestone 或真实产品验收。上一 batch04 的失败历史保持原样，也不构成 Batch05 的测试结果。

[产物清单](../../../../../../cargo-targets/zircon-local/reports/runtime-editor-optimize/artifacts.json)属于此前登记的历史记录，本次不读取或改写它。一次性 evidence 已确认本批自有 worker 退出；未核验或清理共享、历史、未知归属的构建与缓存，也不声称释放空间。Batch05 request 已关闭为 prelaunch refusal；更广泛的性能目标及其后续产物生命周期仍独立未完成。

[Runtime 完成列表](../../astra/features/runtime/2026-10-11-runtime-editor-batch05-completion-list.md)；[Editor 完成列表](../../astra/features/editor/2026-10-11-editor01-batch05-validation-completion-list.md)。
