---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: runtime-script-binding-hotpath
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/13
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/script/vm/scene_hook.rs
  - zircon_runtime/src/script/vm/runtime_context.rs
  - zircon_runtime/src/script/vm/gameplay_host
  - zircon_runtime/src/script/vm/behavior_bridge.rs
  - zircon_runtime/src/script/vm/host/script_call_table.rs
tests:
  - cargo test -p zircon_runtime --lib script --locked --jobs 1 -- --nocapture --test-threads=1
  - powershell -NoProfile -Command "Select-String -Path 'zircon_runtime/src/script/vm/scene_hook.rs','zircon_runtime/src/script/vm/gameplay_host/*.rs' -Pattern 'node_records\(\)|from_value|current_script_runtime_call_context|expect_string'"
---

# Runtime13：script binding与gameplay host热路径交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：P5 Runtime script 96/96逐Rust文件性能审查，PERF-MVP-442/443
- 修复责任计划：`docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md`
- 交接原因：Runtime13拥有script binding、host call frame与gameplay capability语义；最低根因是这些契约仍以per-frame JSON/full-World projection和per-call owned context执行。
- 生命周期键：`runtime-script-binding-hotpath`

## 失败现象与复现证据

Fixed/Update scene hook各自取得全部node records，clone/deserialize `script.bindings`，再过滤成第二层Vec；每binding格式化String key、按package name扫描slots并同步call export。gameplay host每call clone`ScriptRuntimeCallContext`，String参数、input snapshot、dynamic JSON与World node records又产生多层复制/扫描，复合操作重复获得World access。

本轮只完成五项局部止损中的Runtime13部分：稳定behavior callback不重复写cache、entity existence走World索引、host handle validity不clone record。它们不解决binding projection、borrowed call frame、typed property/query index或World access scope，交接保持open。

## 最低共享层根因

Runtime13已定义清册、capability与ScriptCallTable，但没有把scene binding component generation编译成active callback table，也没有让调用上下文借用稳定的manager/input/world handles。PERF-MVP-331的owned ScriptCallSite context和本交接是同一调用架构的两层，不得分别新增兼容路径。

## 架构修复验收

- 按script-binding component/world generation发布active dense projection；Fixed/Update共享一次frame snapshot，stable generation不再node scan/JSON decode/key format/package resolve。
- 每binding持generation-aware callback handle；start只在新增、enable或generation变化触发，reload原子重绑且错误/调用顺序等价。
- ScriptCallSite与gameplay host统一borrowed/arena `HostCallFrame`；module/function/capability/String/Bytes和runtime context稳定调用零深clone。
- 一个host call只进入一次显式World access scope；entity/component/property/nearest/count使用typed index/change projection，禁止用`node_records()`或JSON作为内部热路径。
- 以1/100/10k nodes/bindings和1/100/1M calls记录clone/alloc/visits/resolve/lock/main-thread p95，并通过current-source Runtime script Cargo与F2/F4产品trace。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止缓存完整World/node-record/JSON snapshot跨generation而没有精确失效；禁止用全局锁保护“优化缓存”。
- 禁止在未声明scheduler access与callback thread-affinity前直接parallel iterator执行脚本。
- 禁止保留owned HostCallContext与borrowed HostCallFrame双生产路径。

## 修复结果与回传

Open state: `源码修复已完成，等待当前源码的受管验证`; this handoff is not fixed or
returned yet.

Implemented, not yet accepted:

- Scene bindings now build one active dense projection keyed by world handle
  and the `script.bindings` dynamic-component generation. Stable generations
  reuse callbacks rather than rescanning nodes, decoding JSON, formatting keys,
  or resolving packages per frame.
- Each active binding retains its generation-aware callback handle. Replacing a
  deserialized world, including a replacement that removes bindings, advances
  the relevant dynamic-component generations so stale projections cannot be
  reused. Focused coverage exercises direct and staged world replacement.
- Host exports use borrowed `ScriptHostCallFrame` arguments; the hot path no
  longer maintains a second owned host-call-context bridge. Runtime reflection
  access is issued by `VmPluginHostContext`, with cross-crate fixtures limited
  to the explicit non-default `test-support` feature.
- Production `ScriptRuntimeCallContext` construction and TLS installation are
  crate-private. Cross-crate ZrVM fixtures now use only
  `ScriptRuntimeTestContext` and `with_script_runtime_test_context` under that
  same feature, so a production backend cannot replace the active runtime
  context.
- `VmReflectionWorldAccess` no longer lends a raw `World` from its persistent
  token. It issues an HRTB-bound `VmReflectionWorldOperation` only while a
  runtime script scope is active; the ZrVM reflection dispatcher performs its
  dense-token read/write inside that non-retainable operation ticket.

Static evidence: the script-binding boundary audit and selected `rustfmt
--check` scopes pass; `cargo metadata --no-deps --locked --format-version 1`
succeeds. A second source review identified the call-frame clone, private test
construction, production-context, and persistent-world-borrow findings; the
current source addresses them, while final re-review and Cargo evidence remain
pending. These checks do not establish runtime performance or behavior.

Required before `failure return`: run the declared current-source Runtime13
managed Cargo gate and the originating performance acceptance/trace gates. The
1/100/10k binding and 1/100/1M host-call measurements remain acceptance
evidence, not a claim made by this record.
