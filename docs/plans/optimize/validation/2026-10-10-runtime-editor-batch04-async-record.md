# Runtime / Editor 第四批异步验证记录

日期：2026-10-10。请求：`runtime-editor-source-repair-batch-20261010-04`。

## 实际启动

2026-10-10 06:43:13 UTC，现有独立本地入口已通过隐藏后台进程启动。启动回执只证明入口进程已创建；编译是否开始、测试与性能结果仍未知。Jenkins 尚未提交，等待用户提供现有协调器入口。

日志与请求目录：`E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/batch-native-v4/actual-request-v1`。清单 SHA256：`7181910a55118b32c6f9d80260fa35337571ea369887c01adb0c72ce51977f8e`；绑定 439 个输入、原 15 项命令和 14 个原性能 producer。Runtime #2 最低 20 个测试；Editor #15 最低 174 个测试。原 Release 矩阵、预算、profile、features 和完整 offender-empty 要求保持。

## 编译期间的后续修复

- Core / Particle 两文件、Runtime02 charge 单文件、Runtime44 时间戳四文件、BlendSpace 三文件已实际采用，详细记录见对应完成列表。
- paint/hit fixture 的六处实际 MutexGuard 修正已通过独立审查；两文件完整候选尚未采用，十二个原测试名与 V9 筛选条件无重叠。
- Generated 接口三文件四处漏改调用正在修复；Tools 真实 Source 的完整有序闭包检查正在准备。候选留在 outbox，维持后台批次输入固定。

## 待验收

不读取 PID、进度或部分编译日志，不循环查询协调器。完成一组后续修复审查后只核对终态回执；缺失时保留 pending，继续独立工作。终态还需核对当前输入、实际测试名、非零数量、所有原性能矩阵及输出物理路径。入口启动或退出码不能代替 native、性能、产品或 Jenkins 验收。

参见 [Runtime 完成列表](../../astra/features/runtime/2026-10-10-runtime-editor-batch04-completion-list.md)、[Editor 完成列表](../../astra/features/editor/2026-10-09-editor01-component-consumer-validation-list.md)。

## 2026-10-11 · 第四批终态失败增量

本节是本批次唯一的 2026-10-11 增量；前文历史保留。已核对 [immutable terminal/closed-log audit](../../../../.codex/outbox/runtime-editor-batch04-terminal-checkpoint-v1/terminal-and-closed-log-audit.json)，audit SHA256 为 `053e56ee8bf8ecc3b51225321f6c439d9627f509abfc8378180e59411db7bc09`。闭合日志路径为 `E:\\cargo-targets\\zircon-local\\source-repairs\\current-main-20261008\\batch-native-v4\\actual-request-v1\\01-three-package-check.log`，SHA256 为 `0e1b07648e2367905ff01987c9d9d3d90ae174dc1ff0654640f6e9a796b7e021`；terminal SHA256 为 `0690866eed759ba941096c1d24993884ed9a371360e9654c258cf752eab1495b`。

- [x] 终态为 `failed`：15-arm 中仅 #1 `three-package-check` 运行并以退出码 `101` 结束；#2–#15 共 14 项均为 `not-invoked-prerequisite-failed`。
- [x] 完整 Source guard 保持：`sourceUnchanged=true`、`changedSourceFiles=[]`；仅 1 个 cargo 命令被调用，Rust tests 与 performance producer 均为 `0`。
- [x] 三条 compiler diagnostics 归并为两个修复根因：Runtime 的 `RuntimeWakeRegistration::disabled()` 私有访问错误（E0624，两处）；Editor 生产调用 `to_host_contract_workbench_window_nodes` 被 `#[cfg(test)]` export 隐藏（E0425）。闭合后另有三条实际 Source repair 记录：Editor export `bff7b069…`、Tools 闭包 `e2c76c35…`、RuntimeWake `9d86388f…`；这些不改变本次失败输入的 Source guard，也不代表该批次通过。
- [x] 下一次仍按原 15 arms / 14 producers 统一验收，待采用筛选修正为：#2 生命周期 exact-name 最低数从 20 增至 22；#15 在 174 基础上加入 12 个既有 fixture 用例和 5 个 Generated 用例（既有 4、新增 1），最低数为 191。此计划尚未作为 native/performance 结果执行或接受。
- [x] 首次 substantive terminal reconciliation checkpoint 仅 1 次；无 PID poll、partial-log read、compiler wait 或 coordinator monitor（0/0/0/0）。无 Jenkins 提交声明。
- [ ] 本次 Runtime/Editor native、performance、product acceptance 仍 pending；修复后沿用同一 15-arm 批次统一验证，不对单个 arm 单独验证。
