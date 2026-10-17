# Runtime03：失败配置接管保留旧 owner

日期：2026-10-15；任务 optimize-20261013-01a13451。五处源码、静态检查和独立源码审查已完成；原生测试、性能和产品验收仍待 Jenkins 实际终态。

## 问题与修复

原路径注册先发布新 epoch，再恢复、读取和解析配置、启动 worker。新配置损坏或线程创建失败时，激活失败却已经撤销旧 manager 的写入资格。

现在全局 registry 锁只覆盖路径 gate 的查找/回收；取得 gate 前释放全局锁。同一路径 gate 内 checked_add 取得候选 epoch，完成恢复、读取、完整 JSON map 解析和 idle worker 启动，全部成功才发布 epoch。解锁后合并具体 String/Value 到 Core，再返回 manager。

失败保留旧 epoch 和原错误。pending 候选取消保护防止失败时遗留的引用在重试复用 epoch 后恢复资格。初始化 panic 先被捕获，正常释放 gate/lease 后再 resume_unwind，避免给旧 owner 的 mutex 新增 poison。gate 内无递归 fence.commit 或 worker Drop/join；Core 合并具体内建值的路径已核实，没有普通可恢复失败点，分配失败 abort 不作为事务承诺。

注册仍使用既有词法路径键，不解析文件系统别名，不提供跨进程锁。成功接管保留既有替换语义；Runtime55 FND-P0-002 的 broker/exclusive lease、活跃 owner 冲突 fail-close 仍 OPEN，本轮不能关闭。

## 源码与回归

- [zircon_runtime/src/foundation/runtime/config_manager.rs](../../../../zircon_runtime/src/foundation/runtime/config_manager.rs)：8976 → 10509 B；after SHA256 `621fbcc9f1d8f28e83f97d3a88ef56047be43397d2653b7bcc69096df436ebb8`。
- [zircon_runtime/src/foundation/runtime/config_manager/commit_fence/registry.rs](../../../../zircon_runtime/src/foundation/runtime/config_manager/commit_fence/registry.rs)：3338 → 4615 B；after SHA256 `ce26f927dc655f16ea36222bba4b3b218cbf3f42a4faf1abce7a80691a6f7ba9`。
- [zircon_runtime/src/foundation/runtime/config_manager/commit_fence/fence.rs](../../../../zircon_runtime/src/foundation/runtime/config_manager/commit_fence/fence.rs)：5198 → 6611 B；after SHA256 `609842d6a5170e36fd649bd165225a8b8eba976538b8cad6a42edc3bab77b5ea`。
- [zircon_runtime/src/foundation/runtime/config_manager/commit_fence/tests/cases.rs](../../../../zircon_runtime/src/foundation/runtime/config_manager/commit_fence/tests/cases.rs)：6975 → 11725 B；after SHA256 `514868eb65a6d3d15d1852edfcefb9c8d51a682ff66729b16067fe49e7dcc331`。
- [zircon_runtime/src/foundation/tests/cases.rs](../../../../zircon_runtime/src/foundation/tests/cases.rs)：13337 → 17785 B；after SHA256 `27672254a9eebbd57cab6ac8db066f0b211728ca4e436c6505ceee39cf53ef8c`。

新增四个 ordinary，保留三个本轮选中的既有 ordinary。下列七个 FQN 必须逐个实际通过，ignored fence benchmark 不计普通测试下限：

- `foundation::runtime::config_manager::commit_fence::tests::path_commit_gate_registry_reclaims_only_after_the_last_fence_drops`
- `foundation::runtime::config_manager::commit_fence::tests::path_commit_epoch_exhaustion_never_reauthorizes_a_stale_fence`
- `foundation::runtime::config_manager::commit_fence::tests::path_commit_failed_initialization_preserves_the_current_owner`
- `foundation::runtime::config_manager::commit_fence::tests::path_commit_initialization_holds_the_path_gate_until_epoch_publication`
- `foundation::runtime::config_manager::commit_fence::tests::path_commit_failed_initialization_reclaims_the_unowned_path_gate`
- `foundation::tests::foundation_failed_config_activation_preserves_the_existing_path_owner`
- `foundation::tests::config_manager_persists_values_to_disk`

公开回归走真实文件绑定、模块注册/激活、resolver、set/flush 路径：旧 owner 已落盘后，损坏 JSON 使新 Runtime 激活失败，旧 manager 仍能保存。所有 Runtime、manager、handle 和 worker 最后 owner 释放后才清理独有夹具。未来夹具创建前检查二进制目录的物理路径位于 E:/cargo-targets/zircon-local；本会话未运行该测试或创建夹具。

原配置矩阵驱动和 worker 源码只读不变；原私有加载探针保留供矩阵使用，生产初始化使用无递归提交的新读取路径。epoch 耗尽、候选取消、gate 覆盖初始化、最后 fence 回收都仍需原生验证。

## 完成与验收

- [x] exact5 授权后实施，保留共享 index 和其他会话修改。
- [x] 冻结 patch/source-binding；raw bytes、HEAD/index、18 只读证据和两份计划已核；config_store 后续仅15行文档新增，按单独只读见证重绑定。
- [x] scoped guard 通过，0 errors、0 warnings；独立源码 peer 通过。
- [x] 写入本记录和 Astra 对应列表，准备 B18 多任务验证材料。
- [ ] Jenkins 同一 Runtime 验证命令执行全部七 FQN，0 failed、0 ignored；原 16 Cargo、2 Python、16 producer、legacy11 全部保留。
- [ ] 实际源码、配置、锁、features、工具链与环境执行前后封印；保留 request/attempt/raw 结果，相关输入漂移拒绝该次验收。
- [ ] 原 Runtime03 helper 70%/75%、完整 60 样本矩阵、资源和新增绝对 P95 门槛全部同时通过。
- [ ] Runtime55 FND-P0-002 的同路径多 owner 合同独立修复和验收。

绝对 P95 仍按 [冻结规则](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime03-absolute-p95-criteria.json)：flush/最后 owner 关闭上限（ms），1B×1=50/75，1B×1000=75/100，1MiB×1=75/100，1MiB×1000=100/125，100MiB×1=1500/1600，100MiB×1000=1750/1850。12 单元格各五原始样本，近邻秩 P95 取该格最大值。保留原辅助和资源门槛，不合池、不丢异常值、不自动放宽；阈值是工程目标，本轮没有持久化性能实测结果。

## 证据与异步交接

[source-binding](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime03-failed-takeover-v1-source-binding.json)；[增量 patch](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime03-failed-takeover-v1.patch)；[scoped guard](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime03-failed-takeover-v1-static.json)；[归属与进程记录](../../../../.codex/outbox/reports/optimize-20261013-01a13451/artifacts.json)；[Astra 完成列表](../../astra/features/runtime/2026-10-15-runtime03-failed-takeover-completion-list.md)。

当前 config_store 注释变更来源 UNKNOWN；未修改它，也未重写原冻结包。新增 [只读见证重绑定](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime03-failed-takeover-v1-readonly-witness-reconciliation.json) 明确替代这一项旧 witness 和 B18 selected 身份；执行逻辑和安全结论未变，实际编译仍须完整前后封印。

原计划：[Runtime03](03-core-runtime-diagnostics-profiling-config-review.md)；[Runtime55](55-runtime-foundation-module-config-event-service-driver-manager-persistence-lifecycle-product-integration-review.md)。

B18 在源码、静态、独立 peer 和文档真实 writer exit0 后，通过 .jenkins/records 合批交接。文件交接不证明队列、原生或性能通过；随后继续 Editor02/Runtime55 源码任务，不等待或实时跟踪编译。后续完成实际源码工作后，仅按既定预算读取一次准确终态，缺失原因保留 UNKNOWN。

本会话无自有编译/cache/native fixture/helper/sourcecopy；实际释放 0 B。必要源码归属、失败、静态和后续原生记录保留；历史未知进程身份与其他会话活动产物不删除，不宣称全部清理完成。

