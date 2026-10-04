---
related_code:
  - zircon_plugins/plugin_sdk/src/dist.rs
  - zircon_plugins/plugin_sdk/src/native.rs
  - zircon_plugins/plugin_sdk/src/native/tests.rs
  - zircon_plugins/plugin_sdk/src/test.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_abi.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_callbacks.rs
  - zircon_runtime/src/plugin/native_plugin_loader/load_discovered.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/01/2026-07-22-native-plugin-loader-live-host-static-review.md
  - docs/plans/performance/01/2026-08-23-plugin-sdk-runtime-registration-factory-ownership-current-architecture-review.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
  - docs/plans/zircon_plugins/13-standalone-plugin-build.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp
tests:
  - current plugin SDK final native distribution and test-support slice 4 of 4 Rust files and 20 tests reviewed
  - plugin SDK current tree 21 of 21 Rust files and 39 tests reviewed
  - native capability Python source contract RED then GREEN
  - plugin SDK focused Python contracts 4 of 4 passed
  - focused rustfmt and diff check passed
  - added Rust ABI-call-count behavior test not executed because current-source managed Cargo is unavailable
  - allocator counters, WPR and power pending
doc_type: implementation-evidence
status: static_21_of_21_m0_implemented_dynamic_blocked
---

# Plugin SDK native distribution与能力协商复审（2026-08-23）

## 范围与当前性

已逐行复读`dist.rs`、`native.rs`、`native/tests.rs`和`test.rs`当前**4/4**文件、**2,489物理行、100,486 B、
20 tests**；SHA-256分别为`c3d20ea2...7067`、`ae71d9d2...44d9`、`ac206f5f...1d9e`、
`9ba78455...9f8f`。至此`zircon_plugins/plugin_sdk/src/**`当前**21/21 Rust文件、6,035行、229,320 B、
39 tests**已完成静态复审。

`native.rs`在本轮开始前已有8行import/macro rustfmt dirty；本轮保留该差异并在当前源码上实现能力协商M0。新增Rust行为测试
记录ABI回调次数，但受管Cargo不可执行，未运行；新增Python源码契约先RED后GREEN。三个SDK性能契约加既有editor contribution
契约共4项通过，focused rustfmt与diff check通过。

## 当前源码判定

### dist宏是正确的静态ABI基线

仓库有**49个生产dist宏使用点**：full runtime+editor 6、runtime-only 33、editor-only 10。三个宏把descriptor、behavior、
entry report、bridge method table、required/denied文本全部展开为static/const；export函数只返回稳定指针，缺宿主报告也预建为static。
这里没有加载期TOML序列化、Vec/String构造、锁、线程或逐帧逻辑。宏源码重复很大，但首先是维护/编译成本，不能包装成运行时优化。

`NativePluginStatic<T>`只为封闭的不可变ABI carrier实现`Sync`，repr-transparent layout overhead=0；buffer owner token、panic guard、
host-owned output sink和versioned owned DTO边界合理。command/registration TOML只应在load generation parse一次；
`command_manifest_v4_is_current_and_dense`的`BTreeSet`是冷路径O(C log C)，在真实命令表规模/parse占比数据前不是MVP热点。

### 已实现M0：每个entry只验证一次host ABI

runtime loader对每个被请求的runtime/editor entry调用一次symbol，并把report保存到`LoadedNativePlugin`。旧SDK对每个required/denied
capability都重新解引用host table并调用`host_abi_version()`；在全部required通过且没有denied命中时，调用次数为**R+D**。

当前实现拆出一次`host_functions_v3_are_compatible`和已验证表上的capability probe。每个有约束的entry协商ABI回调静态上界变为
**R+D -> 1**；无约束entry保持0，required失败仍短路denied。public all/any/single helper的空列表、null host、ABI mismatch和
host_handle=0行为保持原状；没有缓存宿主指针，也没有改变ABI布局。

这只消除重复ABI回调。宿主提供`host_has_capability`时，每项仍构造一个`CString`；runtime callback随后对同一grant C string做一次
delimiter token scan，因此协商仍为**O((R+D) * G_text)**并有最多R+D次临时分配。正确结构是下一版host ABI/load generation把
capability identity canonicalize为slot/bitset或一次构建的immutable set，entry按静态slot数组验证；不得把跨DLL borrowed Rust
HashSet或session-global mutable cache塞进v3 ABI。

### test helper错误进入37个生产feature编译面

`test.rs`有491行并构造完整`CoreRuntime`、catalog、scene plan和module activation；作为测试隔离helper，其每次build完整runtime是
正确语义，不应为测试速度共享全局runtime。问题是它随`runtime` feature公开编译：当前30个runtime和7个editor Cargo依赖站点
都解析/type-check该模块，而仓库仅1个外部测试源真实使用`TestRuntime`。

这主要影响clean/incremental编译、rmeta与可能的未裁剪debug产物，不是产品帧耗时。后续hard-cut为显式`test-support = [runtime]`，
SDK自身用`cfg(any(test, feature = "test-support"))`，外部integration test只在dev-dependency启用；先量clean/incremental build和
rmeta/binary map，再改37个站点，避免Cargo feature union或测试target失配。

## Unreal源码依据

`PluginManager.cpp:2034-2080`只在`PluginsToConfigure`非空时完成发现、enable、process与mount，随后清空配置集合；
`2884-2923`按loading phase遍历enabled plugin并加载module。`ModuleManager.cpp:992-1023`首先命中稳定module记录，已加载时直接返回，
miss才进入加载/初始化路径。可转移原则是descriptor/capability/schema在load generation协商一次，stable report/slot供session使用；
metadata或帧调用不能再次字符串协商、TOML parse或创建manager。

Zircon的静态C ABI表和versioned owned buffer是需要保留的优势。Unreal的FName/TMap或module pointer不能直接跨Zircon DLL边界；
Zircon应在host-owned generation内生成稳定capability slot，ABI仅传值/slot/owned DTO，reload先构造candidate再原子发布。

## 量化验收

矩阵为providers P=1/28/49/100、entries E=1/2、required/denied Q=0/1/8/100/1k、grants G=0/1/100/1k、reload R=0/1/100。
记录descriptor/entry calls、host ABI calls、CString alloc count/bytes、grant scans/bytes、TOML parse/build、DLL load wall p50/p95、
main-thread wall、RSS和energy。M0源码门为ABI calls/entry(Q>0)=1；终态门为schema/capability canonicalization<=1/provider generation、
stable query parse/build=0、capability check O(Q)或O(bitset words)、cross-session pointer/cache sharing=0。

编译面另测37站点clean/incremental check/build的wall、peak RSS、rmeta/codegen units与最终binary map；`test-support` hard-cut要求生产
runtime/editor依赖编译`test.rs`=0、integration test parity=100%。当前Rust行为测试、current-source Cargo、allocator receipt、F0/F4
WPR/RSS/power均未完成；无launchable current-source executable，不运行WPR。本切片非渲染，不要求RenderDoc。SDK静态21/21也不
等于动态验收完成，本轮不迁入`review.md`、不提交milestone、不发送完成企微。
