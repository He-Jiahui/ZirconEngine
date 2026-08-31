---
title: Runtime50 M0 Manager Handle Runtime Provenance
category: zircon_runtime
report_id: Runtime50-M0
date: 2026-08-24
baseline_head: 0e2bdaa9d3f6949e351ce4e77ccf1aca9e7032b1
baseline_epoch: 383
session_id: optimize-runtime50-manager-handle-runtime-provenance-r1-20260824
implementation_status: implementation_complete
validation_status: infrastructure_blocked
review_status: independent_review_pending
---

# Runtime50 M0 Manager Handle Runtime Provenance

## 目标与边界

本切片关闭父计划 M0 中两个相互放大的基础缺陷：`ManagerServiceHandle<T>` 的
`index/generation/service` 可由外部 crate 改写，且句柄没有绑定创建它的 `CoreRuntime`。旧 resolver 只把三个字段
还原为 `RegisteredServiceIdentity`，因此被替换为另一 Core 的有效 identity 后会在错误的 runtime 上成功解析。

本切片不是 manager 热路径性能优化，不修改 `core/runtime/handle/resolution.rs`、错误枚举、service registry 数据
结构或 call-lease 算法。Runtime46 当前正在收敛 factory/resolution 行为；Runtime24/50 后续负责稳定 runtime identity
和 typed mismatch error；PERF-MVP-628 继续负责 HashMap、全局锁、downcast 与重复 resolve 的 profiling/结构优化。

## Current-source 与参考复核

实现前逐行复核了：

- `core/manager/service.rs`、`resolver.rs`、`tests.rs`；
- `CoreHandle`、`CoreWeak`、`RegisteredServiceIdentity` 和 manager descriptor/factory 路径；
- Runtime 内部与首方插件的 handle 字段 consumer；外部 crate 的直接字段访问只发现 content-download manager 一处；
- Runtime50 current-source review 已读取的 Unreal SubsystemCollection、Bevy typed resource access、Godot singleton
  registration 与 Unity ContextContainer owner/slot 模型。

参考结论保持一致：Unreal 的 subsystem collection 绑定 owner lifetime，Bevy 的 typed access 绑定具体 World，Godot
对重复注册 fail-close，Unity 的局部 context 以 typed slot 绑定容器实例。Zircon 本轮只吸收共同底线“handle 必须绑定
其 owner/container”；不复制 UObject、全局 singleton、ECS resource 或新的全局 runtime ID。

## RED 合同

新增行为 fixture 创建两个各自注册同名 `RuntimeBoundManager` 的 Core。测试先取得 A/B 两个句柄，再把 A 句柄的三个
公开 identity 字段替换为 B 的值。旧实现随后直接构造 B 的 `RegisteredServiceIdentity`，会返回 B 的 manager；这证明
generation 校验不能替代 runtime provenance，也证明仅把字段改私有而不绑定 runtime 不能形成纵深防御。

共享 managed Cargo lane 在 RED 写入时由 Text03 占用，后续还有 Shader06 reservation，因此没有绕过协调器运行本地
Cargo。RED 的动态执行与最终 GREEN 必须在 managed lane 释放后补齐；当前状态不能写成测试通过。

## 实现

1. `ManagerServiceHandle<T>` 新增私有 `CoreWeak` provenance；捕获句柄时由 `CoreHandle::downgrade()` 写入，不增加
   Core 强引用。
2. resolver 在 registry lookup 前以 allocation identity 做一次 O(1) 指针比较；跨 Core 句柄 fail-close 为现有
   `CoreError::ServiceUnavailable(service)`，不侵入 Runtime46 冻结的错误 owner。
3. `index/generation/service` 从 `pub` 硬切为 `pub(crate)`，不保留兼容字段、构造器或 forwarding facade。
4. 新增只读 `service_name()`；唯一外部直接字段 consumer 已迁移到该 API。Runtime 内部 owner 暂保留 crate 内字段
   访问，后续与 compiled service directory 一并收紧。
5. `Clone` 克隆 weak provenance；`PartialEq` 同时比较 runtime allocation identity，避免两个 Core 的同值 identity
   被视为同一 handle；`Debug` 不输出地址。
6. 新增测试覆盖原 runtime 正常解析、cross-runtime identity substitution 拒绝、跨 runtime handle 不相等，以及
   manager handle 不延长 Core 生命周期。

## 复杂度与性能边界

resolve 新增一次弱指针读取和一次指针相等比较：时间 O(1)，不加锁、不分配、不做 I/O、不升级 Weak，也不改变
既有 HashMap/Mutex/downcast/Arc clone 路径。只有 mismatch 错误路径沿用现有 service-name 字符串分配。句柄捕获和
clone 各增加一次 Weak 创建/克隆；它们不是每次 manager 方法调用的 call lease。

这不是性能优化，因此没有生成虚假 benchmark 或功耗结论。manager 热路径必须按父计划先用 ETW/WPR 与 focused
benchmark 量化 hash、锁等待、downcast、Arc clone 和重复 resolve，再设计 compiled slot/call-lease，不以本次
正确性修复替代结构优化。

## 验证与状态

- [x] current-source、consumer 与参考引擎边界复核完成。
- [x] RED 行为 fixture 先于 production 修改写入，旧执行路径已静态复核。
- [x] weak runtime provenance、cross-Core fail-close 与字段 hard cut 已实现。
- [x] 唯一外部字段 consumer 已迁移，未保留旧公开字段兼容面。
- [x] 3 个 Rust 文件 exact `rustfmt --check` 通过。
- [x] 3 个 Rust 文件 scoped `git diff --check` 通过。
- [x] 三张 immutable-manifest managed validation ticket 均已进入终态并保留失败证据。
- [ ] managed focused manager test 实际执行并确认 GREEN。
- [ ] managed `zircon_runtime` core-min production build 与 content-download package consumer compile。
- [ ] 独立 reviewer 对 ownership、错误语义、API hard cut 和测试充分性复核。
- [ ] coordinator immutable manifest、service commit 与自动 WeCom 量化通知。

当前精确源码文件 SHA-256：

- `core/manager/service.rs`: `5a202c70d1fbf4d752f560165c9abab2ec9f20794e83457ad203577ea62503bc`
- `core/manager/tests.rs`: `78caf9ddcd72055285d95ffcad2c21c16674ccd0af83cb974c2d0815d6a45275`
- `zircon_plugins/net/features/content_download/runtime/src/manager.rs`:
  `fdaa70823a0b1a4474e59cb14d04c7db50b2d1e8466181ca662e4dfd17ddd8e3`

### 2026-08-24 受管验证终态

三张票据都在运行 Rust 命令前失败，不能记为产品 RED 或 GREEN：

- focused manager ticket `efce5bb8396f426293816969522a6a5e` 与 core-min build ticket
  `71a6f5050dab405ab2f10644813a7def` 均为 `validation_copy_baseline_drift`；materialization 报告同一组
  Runtime74 foreign paths：`ui/surface/binding_targets.rs`、template compiler 的
  `binding_param_resolver.rs`/`control_scope.rs`，以及两份 control-scope test owner。
- content-download consumer ticket `d3a780bf6a114588b3ba152771943c30` 为
  `validation_copy_cargo_target_missing`，失败阶段是 `closure_planning`。后续必须用
  `zircon_plugins/net/features/content_download/runtime/Cargo.toml` 的直接 package manifest 重提，不能原样重试
  `zircon_plugins/Cargo.toml` workspace-root 命令。

源码 blob 未因这些基础设施失败改写。Runtime74 baseline owner 收敛后，应以新的子记录 hash 加原三份 Rust hash
冻结 manifest，再提交 focused/core-min/direct-manifest 三张验证；在此之前不进入 finalize 或 service commit。

在 managed Cargo 与独立复审完成前，本里程碑保持 `implementation_complete / validation_pending`，不得提交。
