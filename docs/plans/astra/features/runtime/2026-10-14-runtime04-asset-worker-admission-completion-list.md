---
title: Runtime04 Asset Worker Admission Completion List
date: 2026-10-14
implementation_status: source_reviewed
validation_status: native_and_performance_pending
---

# Runtime04：资产任务关闭准入修复完成列表

对应 [优化记录](../../../optimize/zircon_runtime/04/2026-10-14-asset-worker-closed-admission-record.md)。本表记录局部源码修复；全资产系统与动态库卸载里程碑仍待各自验收。

- [x] crate-private fallible submit 拒绝时归还原闭包，保留 public spawn/私有 lease 契约。
- [x] IO 拒绝时回滚 exact-owner entry 和 charge，解锁后终止/取消 expiry/释放闭包和 key/Arc。
- [x] merge/cache 先于新准入，已准入任务排空；admitted 先于完成发布取得 registry。
- [x] 新增真实 IO 拒绝、已准入 drain/merge/cache、通用 helper 所有权三个回归。
- [x] gated 用例先释放/排空/join并释放 timer，再断言；真实拒绝覆盖活 timer 注册。
- [x] 修复两个既有 Barrier 用例及其 owned timer fixture 的失败清理顺序，保留释放前观察和原断言；生产两文件与普通用例数量不变。
- [x] 三路径 raw SHA/基线/R2 累计 patch/evidence 冻结，实际 packet writer/readback exit 0，V1 证据保留。
- [x] 最终 scoped rustfmt、diff、静态 wire/inventory exit 0；历史失败保留。
- [x] 精确 guard 实际 exit 0、0 errors / 0 warnings；独立源码复核无静态阻断。
- [x] 静态确认 13 worker + 3 Windows TaskPool 普通，与旧 Runtime 普通选例无交集；3 ignored 不计普通 pass。
- [ ] 第 13 批原生 ordinal 2 floor 89，新增 16 名字逐一通过；Windows TLS 例实际执行。
- [ ] 其余 arms、16 producer 身份、独立 ordinal 11 及 2 Python arms 全通过。
- [ ] 原始性能、60 行 Runtime03 绝对/相对预算、54 行 Runtime44 RSS 双上限及资源/产品门槛通过。
- [ ] request/attempt 关联完整 source/config/lock/features/toolchain/environment 前后封存，无漂移。
- [ ] 原生 workers/timers/TaskGraph 实际完成/join、物理 E 根输出与子进程临时目录、归属清理和终态回执核实。

helper 探针不构成真实 request 最终捕获对象 Drop 时点的动态证明，该调用点当前由静态检查和独立审查支持。源码记录冻结时第 13 批为计划交接，原生/性能 pending；后续正式交接/回执以任务记录为准。

保留必要 patch/静态证据/计划记录供验收校验；未创建本地编译或原生临时产物，实际释放 0 B。
