# Runtime03：复用冻结 profile 导出数据（2026-10-09）

对应[原优化计划](../03-core-runtime-diagnostics-profiling-config-review.md)与[功能完成列表](../../../astra/features/runtime/2026-10-09-runtime03-owned-profile-export-completion-list.md)。成功导出路径的源码优化已接入，实际性能验收待量测。

## 实际改动与收益范围

`export_report()` 已在 recorder 锁内取得拥有型冻结 `ProfileSnapshot`，此前把它借给导出函数，成功构造 report 时再次 clone 全部 rows、String 和 retention 数据。现有导出核心用 `Cow<ProfileSnapshot>` 共享同一流程：拥有型入口在最后 `into_owned()` 移入 report；公开借用入口与 after-native hook 保留，在原成功返回点继续 clone 一次。

这消除拥有型成功路径的一次完整深克隆。recorder 冻结时的 clone、分析器、DTO、格式化、I/O 保留，尚无当前源码前后 allocation/CPU/RSS 数值证明。文件 admission、整个 session owner、native JSON、hook、可选 Perfetto 或旧文件移除、三种 hotspot、summary 和 typed error 顺序保持；capture epoch、scope/frame/thread 身份和 ring 截断逻辑保留。

新增 `runtime03_owned_profile_export` 的真实 public 回归，覆盖 joined worker、旧 epoch、frame/span/counter ring 覆盖、parent/frame/order、active 导出后 stop/drop、reset 新 capture 与冻结 report/JSON 的隔离。两个 include_perfetto 配置只有在 `profiling-chrome` 启用时才能覆盖真实 public Perfetto 分支，需额外 feature arm。

## 源码与异步证据

补丁 SHA256 `de25b92171af4d970702d2a4556f990ca3665d591e121dc2aaac13b5c1262710`；实际接入收据 `runtime-next-core-v4/actual-canonical-apply.json` SHA256 `118d2c94cab77cd2de1aba02a83a90825370efc6268596fd5b2f958a52ff5bd5`。独立源码审查 `runtime03-owned-export-core-review-v1/review.json` accepted，SHA256 `851c58a9e1ff2cb28af6ab6fa5a9e87e29157480a6009f7fda88cb02cb2c6ae6`；实际 rustfmt、scoped diff check 均为 0，control/index/manifest/lock hash 未变。目录均在 `E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/`。

实际文件 LF hash：`profiling/mod.rs` 为 `2d89dbbeac8b4028fac0b6e6a236e8c73fc04bca0aac6cc7eece871bd6969098`；`profiling/export.rs` 为 `7061dcef3b43889bbdc275dedbff383b116e403ca9bfa0d931e88a2ad9399853`；新 test 为 `55c01389b5f5697cd29842142f06588aab522536257fe18c16ac8c383a9e1622`。实际 raw hash 见接入收据。

原非嵌套 worker producer、scope/recorder/export/session-owner/streaming 测试与原 Runtime03 性能矩阵完整保留。13 命令 successor 草案仍未绑定或启动，将补入新 public test、profiling-chrome 与原导出 Release producer；提交后继续功能修复候选，实质代码/证据检查点一次核对终态。前一 batch Cargo101 失败历史保留，不能验收本次新源改动。

## 完成列表

- [x] 源码复用已有拥有型冻结数据，保留借用 API、owner 与错误顺序。
- [x] 真实 public 回归已接入；独立审查、格式与差异检查通过。
- [ ] feature-off、profiling、profiling-chrome 原生编译与实际 test 匹配通过。
- [ ] 原 recorder/export/session-owner/streaming 回归与 Release producer 实际通过。
- [ ] 同一实际导出 workload 的绑定生产 before/after allocation、时间与 RSS 收益达标。
- [ ] 原 worker/series/history/consumer、产品帧扰动、Jenkins 与里程碑门槛达标。
