---
related_code:
  - zircon_runtime_interface/src/buffer.rs
  - zircon_runtime_interface/src/handles.rs
  - zircon_runtime_interface/src/manifest.rs
  - zircon_runtime_interface/src/runtime_api.rs
  - zircon_runtime_interface/src/status.rs
  - zircon_runtime_interface/src/version.rs
  - zircon_runtime_interface/src/lib.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
reference_sources:
  - dev/Fyrox/fyrox-impl/src/plugin/dylib.rs
  - dev/godot/core/extension/gdextension.cpp
tests:
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime_interface/src/tests/plugin_api_contracts.rs
  - current-source Windows dynamic runtime ABI tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime interface ABI foundation 性能静态审查（2026-07-23）

## 范围与覆盖

`zircon_runtime_interface/src/{buffer.rs,handles.rs,manifest.rs,runtime_api.rs,status.rs,version.rs,lib.rs}`当前源 **7/7** 个 Rust 文件、**375** 行已逐文件阅读，并追踪 `ZrOwnedByteBuffer`到 runtime producer、Editor gateway、App runtime-library consumer与 plugin SDK owner。`lib.rs`、`runtime_api.rs`、`version.rs`含其他会话未提交的 V3 reactive-wake cutover，本轮仅按 current-source只读审查，未吸收或修改。

## 性能结论

- `ZrByteSlice`、`ZrByteBufferRef`、三个 runtime handle、`ZrStatus`与 manifest descriptor均为 pointer-dense/Copy POD；成功状态和 empty buffer不分配。当前接口基础本体没有哈希、锁或调度热点。
- `ZrOwnedByteBuffer`允许 producer移交已有 Vec allocation并由 producer free callback回收，避免导出前再复制。但 Editor `SessionGateway::capture_frame`通过 `GatewayOwnedOutput::into_vec()`对完整 foreign RGBA执行 `to_vec()`，随后释放原 buffer；正常 1080p/4K capture产生一次全帧复制和双 owner峰值，继续确认 **PERF-MVP-023**。正常 viewport应传GPU/generation handle；显式跨进程 capture/fallback才承担有预算的一次 copy。
- native command output同样是 plugin分配 owned buffer、host复制到新 Vec再 free的跨 allocator双 owner模型，继续归 **PERF-MVP-542**。接口已存在 caller-provided `ZrByteBufferRef`形态，应在版本化 bounded output sink中复用，而不是为每个高频命令保留 owned transfer热路径。
- profile、operation、host request和plugin event的当前消费者直接从 foreign bytes执行 serde decode，再释放 wire buffer，没有先复制 raw bytes；但 producer仍 `serde_json::to_vec`并为每次非空page新建 owned allocation。它们是有界控制面，继续由 Runtime10已有 event page/operation budget验收，不误列为帧 RGBA同级问题。
- canonical empty output能跳过 JSON、allocation和 free callback；Runtime10已有 empty host-request证据。`ZrByteBufferRef`的 capacity/written协议与 `ZrOwnedByteBuffer` owner/free协议必须保持单一权威，不能让 consumer猜测或跨 Windows CRT直接接管 foreign Vec。
- current V3 re-export/version变更尚处 atomic runtime/app migration状态；根文件静态存在不等于 producer/loader/cadence已验收，继续按 Runtime10 M2边界处理。

## 优化设计

1. viewport正常帧跨 ABI传 `texture/surface + generation/fence` handle，Editor消费同一GPU resident；fallback使用固定数量 async readback slots和shared frame owner，stable frame不做CPU readback、RGBA clone或reupload。
2. command/event/operation大输出采用 caller-provided capacity、required-size重试或host allocator transfer合同；每次成功只允许一个 authoritative allocation owner，并给 max bytes、deadline、remaining/age。
3. owned buffer只保留低频或显式 capture兼容边界；consumer统一 validate owner shape/status并以RAII释放，empty不调用 allocator。ABI版本升级不得同时维护两条无界热路径。

这些目标分别由 PERF-MVP-023、542与 Runtime10现有 bounded event/operation计划拥有；本轮不修改 foreign V3 cutover源码。

## 参考引擎对照

Fyrox的 dylib插件直接返回同进程 Rust trait object并明确不建议生产使用，不能作为Zircon稳定 C ABI或跨 allocator所有权范本；可借鉴的只有 library lifetime pinning。Godot GDExtension把函数表与初始化级别解析放在加载慢路。Zircon必须保留显式 C ABI、owner token/free和版本协商，同时让大帧/大输出走 shared handle或 caller-owned bounded storage。

## 动态验收

1. current-source interface/runtime/editor/app ABI layout、owner/free、malformed output、empty output和 V3协商 tests；Windows不同 CRT/动态库组合无 double-free、leak或 owner错配。
2. 720p/1080p/4K，stable/resize/device-loss，capture 1/60 Hz：记录GPU readback、RGBA allocated/copied bytes、peak RSS、free callback和gateway p95；正常 viewport readback/copy=0，显式 capture copy≤1且有ring预算。
3. command/output 0/1KiB/1MiB/256MiB、calls 1/1M、threads 1/16：记录producer/consumer allocation、copy、required-size retry与RSS；成功输出单 owner、bounded sink不复制完整payload。
4. event/operation pages 0/1/64/10k：记录JSON encode/decode bytes、alloc、remaining/age/drop；empty alloc=0，nonempty受 max events/bytes/deadline约束。

动态门禁、atomic V3 migration和产品 F0/F4 trace未完成，因此该切片继续保留在 `pending.md`，不进入 `review.md`。
