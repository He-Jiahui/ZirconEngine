---
related_code:
  - zircon_runtime_interface/src/runtime_api
  - zircon_runtime_interface/src/project
  - zircon_runtime_interface/src/resource
  - zircon_runtime_interface/src/serialization
  - zircon_runtime_interface/src/world_sync
  - zircon_runtime_interface/src/ui
implementation_files:
  - zircon_runtime_interface/src/runtime_api
---

# Runtime Interface 工程化差距综合审查（Review-only）

## 结论

`zircon_runtime_interface` 已有大量 ABI DTO、预算常量、generation、分页与诊断类型，但当前仍更接近 Rust lockstep contract 集合，而非可跨 DLL、跨语言、跨版本长期运行的稳定边界。已有 Interface01-16 报告是详细 canonical owner；本篇只总结当前必须优先收敛的公共契约，不重复其 finding。

## 关键发现

### IF-P0-01：Foreign memory ownership 仍不能仅靠 Rust carrier 保证

- **证据：** `src/lib.rs:30-46` 重新导出 owned/borrowed buffer 与大量预算合同；`buffer`、`runtime_api`、`plugin_api` 相关现有报告确认 owned carrier、slice、release callback 和 status diagnostics 仍依赖调用方传入的 pointer/length/capacity/token 组合。`Clone, Copy` 的表面 ABI 形状不能证明唯一释放权。
- **风险：** forged release、double free、allocator mismatch、借用生命周期越界会直接破坏 Runtime DLL/Plugin 安全。
- **目标：** host-owned allocation registry + opaque allocation id + generation/session binding；borrowed output 只在受控 call scope 内有效；释放必须验证 owner、allocator、generation、长度和终态。
- **硬切/验收：** 删除可由 caller 重建 `Vec` 的释放路径；C header、Rust、跨语言 fixture、child-process fault、重复释放和跨-generation release 必须全部 fail-closed。

### IF-P0-02：API version 形状检查不足以证明 BuildSet/target/schema 相容

- **证据：** `src/runtime_api` 暴露 `ZrRuntimeApiV8` 及 `validate_runtime_api_v8_shape`（`src/lib.rs:110-115`），但现有 Interface15/16 报告确认 shape/size/version 之外仍缺 Build Set ID、target/data model、feature/schema fingerprint、export manifest 和 capability negotiation。
- **风险：** 两个都满足 V8 shape 的 DLL 仍可能来自不同编译器、feature、资源 schema 或平台 ABI，造成静默错配。
- **目标：** `BuildSetIdentity + InterfaceSpec + TargetIdentity + CapabilitySnapshot + SchemaCatalog` 组成 admission commit；ready 只能在 request-bound ledger、所有 slot 验证和 session generation 固化后发布。
- **验收：** N/N-1 skew、错误 target、缺 feature、不同 schema、真实 DLL unload/fault、C header/layout/export 逐槽校验。

### IF-P1：DTO 家族仍有多套 identity、budget、page/resync 和错误语义

现有 UI、world sync、plugin event、operation、profile、diagnostics、project/resource/reflection DTO 各自定义 limit、generation、cursor、status 和 JSON/bytes 编码；缺少统一 `ContractEnvelope { schema, owner, session, generation, sequence, budget, payload, disposition }`。结果是 producer 预算与 host accounting 可漂移，拒绝/丢失/重试/断线无法形成连续证据。应按最小共享层建立 envelope、single-pass accounting、cursor/final/gap/ack/resync 和 typed terminal error，而非继续新增 facade。

### IF-P1：稳定身份仍不能覆盖所有 owner domain

`handles` 有 session/viewport/plugin 等句柄，但 Entity/World/operation/resource/plugin/native allocation/UI window 的 owner epoch、project identity、generation 和 exhaustion 规则不统一。裸 `u64` 或可序列化 token 不能作为跨 session 资格。应为每类公开 handle 定义 owner domain、generation、retirement、stale error 与耗尽策略，并把 project/build-set/session 绑定写入 wire contract。

### IF-P1：错误与成功仍容易在跨层被压平

Status、diagnostic buffer、operation result、frame/pick/world query、UI action 和 plugin callback 之间存在 bool/string/empty payload 的降级空间。接口必须区分 `Accepted`、`Committed`、`Published`、`Observed`、`Rejected`、`Cancelled`、`Failed`、`Stale`，并携带 effect receipt、source generation 和 retry disposition；不能以非空 bytes 或 `true` 宣称产品副作用。

## 目标 ABI 架构

`BuildSetAdmission -> InterfaceSpec validation -> Session/Owner lease -> Request budget preflight -> producer execution -> bounded output registry -> host decode/account -> typed receipt -> ack/resync -> terminal shutdown`。所有 public function table 均需绑定同一 session/generation，所有异步输出均需可分页、可重同步、可退休。

## 参考证据边界

Unreal RHI/模块边界用于说明 capability、resource lifetime 和模块版本不能仅由函数表名称证明；Fyrox plugin/resource contracts 用于 Rust owner 参考；Bevy App/Plugin 数据驱动边界用于 composition；Godot extension ABI/init/cleanup 用于反向清理；Unity Graphics 仅用于本地 `dev/Graphics` 可读的资源/着色器版本化证据。闭源或缺少本地源码的行为不作断言。

## 实施与验证

依赖顺序：`IF-M0 source freeze -> IF-M1 ownership/allocator -> IF-M2 BuildSet/schema/target admission -> IF-M3 envelope/page/resync -> IF-M4 real DLL/cross-language/skew/fuzz -> IF-M5 release qualification`。在 MVP F0-F5 完成前只进行契约设计和验证规划。动态验证必须覆盖跨语言 C fixture、N/N-1、child-process crash/unload、fault injection、malformed length/depth/count、replay/resync 和长时间 allocation census。

## 状态与产出记录

- 2026-09-02：完成 review-only 综合归纳；未修改 Interface production code，未运行 Cargo 或 DLL 验证。
- 状态：`in_progress`；当前大量 Interface/跨 crate 并行改动，实施前必须重取 fingerprint。
