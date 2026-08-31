---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: native-sdk-callback-global-panic-hook
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/06
plan_link_mode: child_record_only
related_code:
  - zircon_plugins/plugin_sdk/src/native.rs
  - zircon_plugins/native_dynamic_fixture/native/src/lib.rs
  - zircon_runtime/src/plugin/native_plugin_loader/ffi_panic_guard.rs
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_calls/output_sink.rs
tests:
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sdk --features native --locked -- --nocapture
  - cargo test -p zircon_runtime --lib native_plugin --locked -- --nocapture
  - concurrent native callback storm with a process-global panic-hook sentinel
---

# Runtime06：native SDK callback全局panic hook交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Plugin SDK与examples 23/23逐Rust文件性能审查，PERF-MVP-491
- 修复责任计划：`docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md`
- 交接原因：Runtime06拥有native plugin lifecycle、ABI callback与reload/unload并发语义；源码局部修复不能替代真实loader和多线程hook sentinel验收。
- 共同验收：Runtime10确认host/plugin两侧FFI panic status与diagnostics契约一致
- 生命周期键：`native-sdk-callback-global-panic-hook`

## 失败现象与复现证据

`zircon_plugin_sdk::native::catch_native_callback_panic`原先在每次callback前后调用`std::panic::take_hook/set_hook`，并为临时空hook分配`Box`。panic hook是process-global状态；并发插件callback会竞争同一全局authority，调用成本与线程数无关地进入ABI热路径，还可能暂时覆盖宿主或诊断系统安装的hook。

本轮已用源码RED→GREEN删除hook交换，只保留`catch_unwind(AssertUnwindSafe(callback))`和既有panic→`ZIRCON_NATIVE_PLUGIN_STATUS_PANIC`映射。宿主`ffi_panic_guard.rs`本来就采用相同的直接catch形状。由于受管Cargo lane被其他会话占用，并发hook sentinel与真实native loader尚未动态运行，本记录保持open。

## 最低共享层根因

SDK把“防止panic跨FFI”与“静默默认panic输出”混为一体。前者只需要unwind guard与typed status；后者没有per-callback安全修改process-global hook的实现方式，必须由宿主长期诊断策略统一拥有。

## 架构修复验收

- normal/panic callback均不得调用`take_hook`、`set_hook`或分配临时hook；panic仍转换为ABI panic status并保留静态diagnostics指针。
- 1/8/64线程、每线程1/1k/1M callback记录hook swaps、allocator calls、global wait与p95；hook swaps/临时hook alloc/global wait均为0。
- 安装sentinel process hook后并发执行normal与panic callback，执行前后hook identity/行为不被插件SDK替换；宿主和插件侧panic状态码一致。
- native dynamic fixture的command panic、normal output ownership/free和loader unload/hot-reload回归通过。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止用Mutex包住`take_hook/set_hook`；这仍会把全局串行与分配留在每callback路径。
- 禁止在每个插件或command wrapper复制一套panic guard；SDK/host各自只保留一个owner。
- 禁止吞掉panic后返回OK或丢失typed panic status。

## 修复结果与回传

Open state: `静态修复已落地，等待受管Cargo、并发hook sentinel与native loader产品验收`; no dynamic pass is claimed.

## 2026-08-29 host output callback closure

- F1 current-source 复核发现 `NativePluginOutputSinkV4.write` 指向的
  `behavior_calls/output_sink.rs::write_host_output_v4` 是宿主暴露给 native plugin 的第五类 private
  host callback，但旧 F1 guard 只覆盖 host function table 的四个 callback，并且仍读取
  已拆分前的 `host_api_adapter.rs` façade。output writer 因此没有进入 panic guard，结构
  guard 也可能 false-green。
- `ffi_panic_guard.rs` 现只保留一个 generic `catch_unwind` kernel，并分别投影
  `ZrStatus`、`u32` 与 `NativePluginCallbackStatusV3`。正常路径不交换 process-global
  panic hook、不建立全局锁，也不分配临时 hook；output sink panic 返回静态 diagnostics
  与 `ZIRCON_NATIVE_PLUGIN_STATUS_PANIC`。
- `behavior_calls/output_sink.rs` 现是 output bytes、null-slice、checked length、host byte
  budget、`try_reserve` 与薄 FFI wrapper 的唯一 owner。捕获 panic 后同时标记 host-owned sink
  terminal；command report 优先保留 PANIC typed status，因此 foreign callback 即使忽略
  writer 返回值并返回 OK，也不能发布部分 payload 或把 panic 降级为普通成功。
- F1 source guard 直接读取 `bridge_scope`、`ecs_registration`、`host_callbacks` 与
  `behavior_calls/output_sink` 当前 owners，新增 `register_system_v2` 和 output sink 覆盖，并删除五份
  未参与断言的 declaration-only 文档编译输入。
- Windows Rust 1.94.1 独立 harness 在 `E:` 直接包含 current production guard/sink 源码并执行：
  正常写入、预算拒绝、null context 与三种 panic projection 共 `6/6` GREEN；F1
  current-source callback guard 在 owner hard cut 后复跑 `1/1` GREEN；编排 root/child 为
  687/102 行。该证据不替代本记录声明的受管 Cargo、
  1/8/64-thread sentinel 与真实 native loader 产品验收，failure 继续保持 `open`。
