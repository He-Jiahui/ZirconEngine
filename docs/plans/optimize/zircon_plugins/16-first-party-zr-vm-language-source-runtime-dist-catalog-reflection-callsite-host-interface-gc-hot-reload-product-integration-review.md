---
title: First-Party Zr VM Language Source、Runtime、Dist、Catalog、Reflection Callsite、Host Interface、GC、Hot Reload 与 Product Integration 工程化差距
category: zircon_plugins
report_id: Plugins16
review_date: 2026-08-19
baseline_head: 25e09a23178000f2e783ce2143cf70a8b118d404
baseline_epoch: 333
related_code:
  - zircon_plugins/zr_vm_language/plugin.toml
  - zircon_plugins/zr_vm_language/runtime/Cargo.toml
  - zircon_plugins/zr_vm_language/runtime/src
  - zircon_plugins/zr_vm_language/runtime/src/backend.rs
  - zircon_plugins/zr_vm_language/runtime/src/plugin.rs
  - zircon_plugins/zr_vm_language/runtime/src/call_site
  - zircon_plugins/zr_vm_language/runtime/src/reflection_host
  - zircon_plugins/zr_vm_language/runtime/src/host_interface
  - zircon_plugins/zr_vm_language/runtime/src/real_backend
  - zircon_plugins/zr_vm_language/dist/Cargo.toml
  - zircon_plugins/zr_vm_language/dist/src/lib.rs
  - zircon_plugins/first_party_runtime_catalog/Cargo.toml
  - zircon_plugins/first_party_runtime_catalog/src
  - zircon_plugins/first_party_editor_catalog/Cargo.toml
  - zircon_plugins/first_party_editor_catalog/src
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - zircon_app/src/entry/runtime_library/tests.rs
  - zircon_runtime/src/script/vm
  - zircon_runtime/src/dynamic_api/session/script_systems.rs
  - zircon_runtime/src/dynamic_api/session/tests/frame_diagnostics.rs
  - zircon_runtime/src/dynamic_api/session/tests/vampire_gameplay.rs
  - zircon_runtime/src/dynamic_api/session/tests/vampire_hud.rs
  - zircon_runtime/src/dynamic_api/session/tests/vampire_menu.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/language
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/augmentation/capabilities.rs
  - zircon_runtime/src/builtin/runtime_modules/ids/plugin_id.rs
  - zircon_runtime/src/platform/tests/app_feature_manifest.rs
  - docs/zircon_runtime/script/vm
  - examples/vampire/zircon-project.toml
  - examples/vampire/scripts/vampire_game/plugin.toml
  - examples/vampire/scripts/vampire_game/plugin.zrp
  - examples/vampire/scripts/vampire_game/main.zr
  - examples/vampire/scripts/vampire_game/bin/.zr_cli_manifest
  - examples/vampire/scripts/vampire_game/bin/main.zro
tests:
  - zircon_plugins/zr_vm_language/runtime/src/tests
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/tests.rs
  - zircon_plugins/zr_vm_language/runtime/src/call_site/tests.rs
  - zircon_plugins/zr_vm_language/runtime/src/reflection_host/tests.rs
  - zircon_plugins/first_party_runtime_catalog/src/tests
  - zircon_runtime/src/script/vm/tests
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_runtime/12-woc-zrvm-package-kernel-world-state-schedule-serialization-runtime-review.md
  - docs/plans/optimize/zircon_runtime/21-zr-language-parser-type-system-semir-bytecode-package-loader-vm-runtime-review.md
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
  - docs/plans/optimize/zircon_editor/31-script-source-code-editor-build-compiler-hot-reload-debugger-visual-script-class-component-authoring-review.md
  - docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
  - docs/plans/optimize/zircon_runtime_interface/04-profiling-plugin-event-script-diagnostic-manifest-crate-ownership-consolidation-review.md
  - docs/plans/optimize/zircon_runtime_interface/05-runtime-host-foreign-output-safe-api-ownership-admission-budget-fuse-observability-review.md
  - docs/plans/optimize/zircon_runtime_interface/07-contract-certification-abi-layout-version-skew-cross-language-fuzz-test-architecture-review.md
  - docs/plans/optimize/zircon_app/06-vampire-roguelite-example-project-asset-script-gameplay-evidence-product-integration-review.md
  - docs/plans/zircon_plugins/08-zr-vm.md
  - docs/plans/zircon_plugins/08/failure-2026-07-18-woc-zrvm-deterministic-bulk-cross-platform-runtime.md
  - docs/plans/zircon_plugins/08/failure-2026-07-19-runtime13-script-call-table-hardcut-consumer.md
  - docs/plans/zircon_plugins/08/failure-2026-07-22-runtime-script-vm-hotpath.md
  - docs/plans/zircon_plugins/08/failure-2026-08-01-zrvm-vampire-behavior-test-ownership-gap.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/VerseVM/VVMContext.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/VerseVM/VVMHeap.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/VerseVM/VVMCollectionCycleRequest.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/VerseVM/VVMDebugger.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/VerseVM/VVMDebuggerVisitor.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/VerseVM/VVMBytecodeAnalysis.cpp
  - dev/godot/core/object/script_language.h
  - dev/godot/core/object/script_language.cpp
  - dev/godot/core/debugger/script_debugger.h
  - dev/godot/modules/gdscript/gdscript_cache.cpp
  - dev/godot/modules/gdscript/gdscript_vm.cpp
  - dev/Fyrox/fyrox-impl/src/script/mod.rs
  - dev/Fyrox/fyrox-impl/src/script/constructor.rs
  - dev/Fyrox/editor/src/plugins/inspector/editors/script.rs
  - dev/bevy/crates/bevy_reflect/src/type_registry.rs
  - dev/bevy/crates/bevy_reflect/src/func
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/RenderGraph/RenderGraphEditorRemoteDebugSession.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 16 · First-Party Zr VM Language Source、Runtime、Dist、Catalog、Reflection Callsite、Host Interface、GC、Hot Reload 与 Product Integration 工程化差距

## 1. 结论

`zircon_plugins/zr_vm_language`不是完全空壳。它已经把具体ZrVM依赖从`zircon_runtime`移到插件owner，提供backend family、package project loader、native host module、预编译`ScriptCallTable`、反射catalog generation guard、capability-gated extension registration、显式session/registration/runtime析构顺序、GC时间预算转发以及hot-reload state migration。默认feature下的中立注册测试和`backend-zr-vm`下的4个真实session测试也证明了局部适配器不是纯mock。这些基础应保留。

但当前产品事实仍是“可选的源码内适配层”，不是工程级脚本产品。runtime crate默认feature为空，普通Client和Editor Host都不启用ZrVM provider；`dist`依赖默认runtime，因此也不启用真实后端。这个NativeDynamic包明确声明stateless、schema 0、空command/event、无invoke/save/restore/unload/bridge/on-host-ready，只返回registration manifest。manifest却同时列出Client、Server、Editor Host和`script.behavior.v1`，实际只有同进程linked Rust接口能工作，动态包不具备等价执行能力。

真实后端又依赖仓库外`E:/Git/zr_vm`和本机`ZR_VM_RUST_BINDING_LIB_DIR`。本轮外部仓库仍位于`8a843bdd7a5aadbbf2deac7242a825cf64c084c8`且有54项working-tree变化，Zircon的Cargo lock不能固定其C源码、CMake选项或动态库。每次装载package都在Runtime启动路径同步打开workspace、增量编译并启动session；所有package、export、lifecycle、GC和析构共享一把进程级`Mutex<()>`。`unsafe impl Send/Sync`的安全论证也只依赖这把锁，没有VM thread attach、context role、multi-world ownership或故障隔离合同。

跨边界数据仍然很窄。宿主值只有null/bool/int/float/string/bytes/handle；任意ZrVM Array都被解释为byte array并逐元素转换，class/object/map/typed array/optional/result没有无损协议。state和reflection字段使用JSON字符串，extension registration会复制全部字符串。反射token虽然避免每帧名称查找并能拒绝stale catalog，但token是进程全局递增整数，调用只有`token + entity + JSON`，没有package principal、字段级read/write权限、world generation、批事务或snapshot一致性。

产品验证没有闭合。插件真实后端测试只编译最小临时project、执行4条生命周期/GC/注册路径；Vampire的10个gameplay/HUD/menu/diagnostic测试仍在Runtime中永久`ignore`并写着“coverage moved to the zr_vm_language plugin owner”，插件owner却没有对应测试。Vampire tracked `.zr_cli_manifest`还保存绝对Windows路径并列出不存在的`.zri/AOT C`。历史文档中的15/15或10个Vampire通过记录属于旧BuildSet，不能覆盖当前source、当前外部dirty VM或默认产品feature。

Runtime07拥有通用VM package/lifecycle/concurrency，Runtime21拥有Zr parser/type system/bytecode/compiler/VM语义，Editor31拥有脚本编辑、构建、调试和Visual Script，Plugins01/06拥有native distribution与catalog truth，Interface04/05/07拥有diagnostic/ABI/budget认证，App06拥有Vampire产品。它们已持有最高优先级问题。本篇不重复累计P0，登记 **0项新增P0、48项P1、12项P2**；本篇唯一拥有Zr VM Language单包从manifest、provider、backend、host/reflection/GC、dist、catalog、App到产品证据的纵向闭环。

## 2. 审查边界、规模与currentness

### 2.1 物理冻结

| 范围 | 文件 / 文本行 / bytes / tests | 冻结事实 |
|---|---:|---|
| `zircon_plugins/zr_vm_language` | 40 / 4,573 / 163,822 / 41 | manifest 1、runtime production 29、runtime tests 8、dist 2 |
| `zircon_runtime/src/script/vm` | 101 / 18,834 / 650,843 / 155 | 中立backend、host、reflection、GC、package、manager、reload、scene systems与tests |
| App/catalog/runtime integration | 24 / 5,676 / 207,726 / 109 | App feature、runtime/editor catalogs、builtin rows、dynamic session与11个ignore |
| docs与Vampire产品输入 | 21 / 1,923 / 267,926 / 0 | 14份VM文档；6份文本产品输入和1份`.zro`二进制 |
| 选定纵向链 | 186 / 31,006 / 1,290,317 / 305 | 185个文本文件、1个binary artifact；11个ignore |
| selected fingerprint | `ad5cf7653809c0fcebca7298d73f7460da8d3530eba7f6fa2ee7d8666aa5d7e5` | 186个tracked path排序，以小写path、空格与file SHA-256组成LF串，无末尾LF后重算SHA-256 |

源revision为`25e09a23178000f2e783ce2143cf70a8b118d404`，coordinator baseline epoch为333。插件包40个文件自身clean；选定范围中的`dynamic_api/session/script_systems.rs`与`script/vm/scene_system.rs`存在其他会话的import排序变化，未改变本轮语义判断，也未被本文修改。外部ZrVM的54项变化和这些邻接变化共同要求`source_recheck_required: true`。

### 2.2 历史计划与开放Failure

`docs/plans/zircon_plugins/08-zr-vm.md`曾把M1-M5局部实现记录为完成，但当前仍有四份开放failure：确定性bulk/cross-platform runtime、`ScriptCallTable` hard-cut consumer、VM hot path、Vampire行为测试owner。它们分别记录真实CLI的array hash、enum/meta、helper binding、constructor、object field、string crash、dependency graph crash、missing module hang问题，以及Cargo lock、内存策略、watcher、World delta、性能和产品测试缺口。

本篇保持四份failure为open，不把当前源码中的局部改进冒充`fixed-*`。历史record回答“当时局部任务是否执行”，开放failure和本报告回答“当前产品是否满足工程资格”；两者不能互相覆盖。

### 2.3 动态证据边界

本轮为E3静态review，没有运行Cargo、真实ZrVM CTest、Vampire、Editor、NativeDynamic或跨平台测试。305个test attribute是库存而不是本轮通过数。外部仓库dirty、真实后端非默认、主工作树有并发修改，因此任何历史测试结果都必须绑定原BuildSet理解。

## 3. 当前真实产品链与断点

~~~text
ordinary Client / Editor Host
  -> script contracts and builtin VM manager exist
  -X first-party-zr-vm-language-runtime-plugin not enabled
  -X backend-zr-vm not enabled
  -> required project selection can exist without executable provider

explicit linked source provider
  -> backend family zr_vm:project
  -> package discovery validates only outer project path
  -> process-global ZrVM lock
  -> open workspace -> incremental compile -> start session at runtime load
  -> scalar/string/byte/handle host ABI + JSON state/reflection
  -> optional named lifecycle exports

NativeDynamic dist
  -> descriptor + registration manifest
  -X no real backend feature
  -X no executable commands/events/state/bridge/lifecycle

Vampire product
  -> required zr_vm_language selection + tracked source/artifact
  -> 10 owner-transferred Runtime behavior tests remain ignored
  -X no equivalent plugin-owned current product receipt
~~~

“crate可注册”“feature开启后能编译一个最小脚本”“dist descriptor能加载”和“普通产品能安全、可重现、可调试地运行脚本”是四个不同gate。当前只能局部证明前三者中的前两项。

## 4. 应保留的底座

| 基础 | 保留理由 | 收敛条件 |
|---|---|---|
| concrete backend归插件owner | Runtime不再反向依赖特定语言实现 | provider、toolchain与carrier必须形成同一ActivationReceipt |
| backend family和严格selector | `zr_vm:project`拒绝旧fallback selector | selector解析必须绑定artifact/toolchain/ABI compatibility |
| prepared `ScriptCallTable` | package load时解析名称，callback走dense site | 编译预算、stable schema ID、principal和batch transaction补齐 |
| catalog generation guard | prepared/commit/abandon/stale路径已有测试 | token绑定world/package/generation并支持显式rebind receipt |
| capability-gated extension registry | 四个extension channel有caller认证与generation | 扩为typed async contract、revoke/quiesce、budget与diagnostic |
| explicit native drop order | session、registration、runtime析构顺序清楚 | 用VM正式thread/context API替代process mutex安全假设 |
| neutral hot-reload/state schema | 有prepare、commit、rollback与type table基础 | 编译产物先于reload准备完成，state改为typed bounded binary |
| cooperative GC schedule | slot顺序、时间预算和overrun有基础 | 接入heap bytes、pressure、preemption、per-world并发与soak证据 |

## 5. 参考实现约束

### 5.1 Unreal VerseVM

VerseVM的`FContext`把外部I/O、heap access、running、allocation和handshake能力拆成不同类型；soft/hard handshake与stop-the-world明确管理线程和GC交点。`VVMBytecodeAnalysis.cpp`从opcode构建CFG，要求branch枚举与分析同步，并验证basic block进入边的failure-context stack。Debugger接口能枚举frame、location和register/value。Zircon无需复制其实现，但不能用“一把全局锁 + 可选export + 字符串错误”替代线程状态、字节码admission、GC协作和调试合同。

### 5.2 Godot

`ScriptLanguage`同时定义source、instance、validation、thread enter/exit、reload、debug stack/locals/globals/evaluate、profiling和frame入口；GDScript cache维护parser phase、正反依赖、abandoned parser与递归失效。Zircon当前把外部compiler/LSP、Runtime package loader和Editor产品拆开却没有共同workspace generation，因此必须建立统一LanguageProvider contract，而不是再向backend trait追加零散方法。

### 5.3 Fyrox与Bevy

Fyrox Script以稳定UUID恢复serialized script type，constructor同时保存name、source path和assembly；lifecycle使用typed context。Bevy `TypeRegistry`同时维护`TypeId`、完整type path、短名歧义集合、递归type dependency与显式TypeData。它们支持Zircon建立稳定ScriptType/Module/Field/CallSite identity；不能继续让路径字符串、全局递增token和JSON payload承担身份与兼容。

### 5.4 Unity Graphics适用性

本地Unity Graphics镜像没有通用语言VM。其RenderGraph remote debug session只可作为“远端数据必须携带compatibility并显式拒绝不兼容”的跨进程precedent，不用于推导脚本语义、GC或语言性能。该负向结论是刻意的适用性边界。

## 6. P0路由与去重

本篇没有新增P0。以下硬阻断继续由canonical owner持有：

1. Runtime21的4项P0继续拥有外部toolchain不可复现、`.zro` admission、artifact事务与语言语义一致性。
2. Editor31的5项P0继续拥有默认产品装配、双编译authority、Script Source、Script Class/Component和Debugger/Visual Script缺失。
3. Plugins01/06继续拥有NativeDynamic假能力、package trust、catalog/profile provider closure。
4. Runtime07继续拥有进程级mutex、slot重入/并发、调用预算、lifecycle和产品规模证据。
5. Interface04/05/07继续拥有typed diagnostic、foreign ABI、budget/fuse与cross-language certification。

Plugins16的48项P1只描述这些owner在Zr VM Language纵向链上的具体落点，不得重复提升为新的P0。

## 7. P1：纵向工程差距

### 7.1 Provider、Catalog、Carrier 与 Toolchain

| ID | 差距 | 所需重构 |
|---|---|---|
| NZR-P1-001 | 普通Client与Editor Host都不链接ZrVM provider | Project preflight生成`LanguageActivationPlan`，required selection缺provider时在启动前fail-close |
| NZR-P1-002 | `dist`不启用真实后端且没有任何执行方法 | NativeDynamic提供等价VM service/bridge，或从manifest删除该carrier与接口声明 |
| NZR-P1-003 | manifest的Client/Server/Editor与desktop平台只是声明 | effective capability按target/platform/carrier/backend artifact计算并生成receipt |
| NZR-P1-004 | backend依赖未固定的兄弟仓库和环境动态库 | 复用Runtime21 `ZrToolchainBuildReceipt`，插件只消费digest固定SDK/artifact |
| NZR-P1-005 | package在Runtime装载时同步源码编译 | Editor/cook先产生immutable BuildSet，shipping runtime只做验证和装载 |
| NZR-P1-006 | provider缺失到`load_package`才返回BackendUnavailable | App预检feature、catalog registration、native library和backend selector的完整闭包 |
| NZR-P1-007 | Dynamic Session会以`zr_vm_language.runtime`名义补内建scene systems | builtin fallback必须归中立owner且不能制造provider已存在的外观 |
| NZR-P1-008 | catalog/App测试只验证registration/module name | 增加每target/carrier的真实load、execute、shutdown与missing-provider contract matrix |

### 7.2 Backend、Lifecycle、Concurrency 与 Reload

| ID | 差距 | 所需重构 |
|---|---|---|
| NZR-P1-009 | 所有ZrVM实例共享进程级mutex | 建立per-runtime/per-world execution owner、VM正式并发模型和可量化contention gate |
| NZR-P1-010 | `unsafe impl Send/Sync`只由全局锁注释支撑 | 接入native thread attach/context role/stack root API并以模型测试证明迁移安全 |
| NZR-P1-011 | open/compile/start/export/GC/drop都可能在锁内长时间阻塞 | phase化operation，支持deadline、cancel、progress、quiescence和lock-wait telemetry |
| NZR-P1-012 | native crash/assert/hang会拖垮engine进程 | trust tier选择in-process或isolated VM worker，具备watchdog、kill与last-good recovery |
| NZR-P1-013 | optional export缺失依赖status或错误字符串包含`not found` | binding返回typed lookup result，禁止文本启发式改变lifecycle语义 |
| NZR-P1-014 | lifecycle仅靠`activate/saveState/...`约定名称 | manifest声明typed lifecycle interface、signature、required/optional、version与capability |
| NZR-P1-015 | 缺`saveState`会静默得到空state，state通过JSON字符串 | policy明确stateless/persistent，typed binary state带schema/digest/budget，不静默降级 |
| NZR-P1-016 | reload候选在激活路径重新编译且无artifact install receipt | compile/verify/stage在外部完成，reload只原子切换已验证generation并保留rollback artifact |

### 7.3 Value、Callsite 与 Reflection Contract

| ID | 差距 | 所需重构 |
|---|---|---|
| NZR-P1-017 | host ABI只支持六类基础值 | 定义versioned tagged value protocol，覆盖typed aggregate、optional/result、object ref与schema ID |
| NZR-P1-018 | 任意ZrVM Array都被解释为byte array | 数组携带element type/layout；非byte array明确走typed collection或拒绝 |
| NZR-P1-019 | bytes逐元素构造/读取，字符串跨边界复制 | 提供borrowed span/owned buffer lease、bulk copy和copy-byte telemetry |
| NZR-P1-020 | `HostHandle(u64)`借signed `i64`保留bit pattern | 使用独立handle tag、owner/generation/type，禁止普通整数伪造host resource |
| NZR-P1-021 | reflection每次读写序列化完整JSON字符串 | 编译typed field codec与batch column/buffer path，JSON仅留debug/compat入口 |
| NZR-P1-022 | callsite token是进程全局递增整数且不自描述owner | token包含table/package/world generation或使用不可伪造lease，耗尽与rebind可诊断 |
| NZR-P1-023 | Public可见性同时代表read/write authority | schema声明read/write/call权限、capability、thread/stage与server/editor限制 |
| NZR-P1-024 | 单字段调用没有同tick snapshot或批事务 | 定义`ScriptWorldTransaction`，批量输入、command buffer、event和presentation一次commit |

### 7.4 Host Interface、Schedule、GC 与 Memory

| ID | 差距 | 所需重构 |
|---|---|---|
| NZR-P1-025 | host module只按名称/version投影，没有BuildSet/ABI fingerprint | native module registration绑定generated interface hash、host build、capability和compatibility result |
| NZR-P1-026 | 四个extension channel缺统一prepare/publish/revoke receipt | registration形成generation-scoped transaction，失败零发布，reload/unload等待callback quiescence |
| NZR-P1-027 | callback合同全为同步调用 | 支持显式async ticket/continuation/cancel，禁止脚本在同步host frame内长期阻塞 |
| NZR-P1-028 | 7个plugin systems都声明conservative world access | 编译真实read/write access plan，按World/phase并行调度并验证冲突 |
| NZR-P1-029 | scene script仍依赖动态`script.bindings`和逐实体host调用 | Script Class/Component编译为typed ECS布局，生命周期批处理并稳定绑定generation |
| NZR-P1-030 | GC的microsecond budget只是传参和事后report | backend在safepoint可抢占/续跑，host deadline与backend pause分别形成enforced receipt |
| NZR-P1-031 | manifest memory soft/hard limit没有live bytes输入和执行策略 | 每slot采集heap/native/cross-boundary bytes，soft触发pressure，hard确定终止/隔离 |
| NZR-P1-032 | GC telemetry只有pause/root/cross-boundary计数 | 增加heap/committed/live/fragmentation/allocation rate/cycle/reason并绑定slot/world/build |

### 7.5 Editor、Diagnostics、Security 与 Operations

| ID | 差距 | 所需重构 |
|---|---|---|
| NZR-P1-033 | first-party editor catalog没有Zr语言provider | 增加Language Workspace、Source Editor、Build、Debugger、Profiler provider与生命周期owner |
| NZR-P1-034 | 外部LSP/compiler没有接入Editor document generation | 单一workspace owner同步open/change/save、dependency graph、diagnostic与artifact generation |
| NZR-P1-035 | 插件没有breakpoint/step/stack/locals/watch/evaluate接口 | 建立debug protocol、source map、safe-point和multi-session attach，Editor31负责产品面 |
| NZR-P1-036 | 没有script CPU/allocation/GC/host-call profiling与coverage | 统一trace event、clock、frame/tick、module/function/source identity和bounded capture |
| NZR-P1-037 | binding错误统一折叠成`VmError::Operation(String)` | 保留compiler/runtime/host/GC/category、code、source range、cause和retryability |
| NZR-P1-038 | compile/load/call diagnostic不绑定source/build/artifact/session | 接入Interface Diagnostic Envelope和`LanguageOperationReceipt` |
| NZR-P1-039 | Zircon只校验外层project path，import/dependency由外部workspace自行展开 | admission冻结完整source/dependency closure、trust、containment、digest与budget |
| NZR-P1-040 | 普通export/host-call没有fuel、bytes、rate、depth或package quota | 每tick/operation/package执行预算，超限typed fault并可隔离slot而非卡死主循环 |

### 7.6 Product、Platform 与 Evidence

| ID | 差距 | 所需重构 |
|---|---|---|
| NZR-P1-041 | 10个Vampire产品测试从Runtime移交后仍永久ignore | 插件owner建立真实backend generation测试后再删除旧ignore和support |
| NZR-P1-042 | 真实backend只有4个最小fixture测试 | 覆盖真实Vampire/WOC模块、class/array/field/helper、failure、reload与presentation事务 |
| NZR-P1-043 | feature-gated测试不属于默认required matrix | clean checkout构建外部toolchain，运行default/real/dist/App/Editor矩阵并归档receipt |
| NZR-P1-044 | 没有多package、多World、PIE、server/client并发证据 | 1/100/10k instance、双World、reload冲突、shutdown/reopen和contention模型测试 |
| NZR-P1-045 | wrapper缺malformed manifest/value/FFI/fuzz与fault injection | 对package、native callback、state/reflection codec、drop order和panic/crash建立敌对corpus |
| NZR-P1-046 | 没有长稳、heap、lock wait、frame-time和host-call基准 | 固定workload/hardware/BuildSet输出p50/p95/p99、RSS、heap、GC和throughput |
| NZR-P1-047 | 只列Windows/Linux/macOS且三者也无当前资格 | 明确Android/iOS/game-WASM策略；每个平台验证compiler、loader、ABI、sandbox和AOT/interp |
| NZR-P1-048 | 文档混合历史通过、当前限制和未来目标 | 文档声明current BuildSet/evidence expiry；开放failure和默认Unavailable必须优先展示 |

## 8. 四份开放Failure的实施映射

| Failure | 当前仍成立的最低事实 | 本篇对应 |
|---|---|---|
| deterministic bulk/cross-platform | JSON/state、逐元素bytes、无帧事务、无跨平台产品receipt，且外部CLI仍有具体语义失败记录 | NZR-P1-017..024、039..047 |
| ScriptCallTable hard-cut consumer | dense direct callsite源码已存在，但受管feature/lock验证未形成当前fixed return | 保留foundation，按NZR-P1-021..025复验 |
| VM hotpath | snapshot、GC schedule、deadline与lazy payload已有局部改进；内存bytes、watcher、World delta和产品测量仍缺 | NZR-P1-009..011、019、024、028..032、046 |
| Vampire behavior test owner | Runtime仍有10个ignore，plugin无对应generation-owned tests | NZR-P1-041..043 |

任何实施批次都应更新原failure owner，不在本篇创建平行“已修复”叙事。

## 9. P2：竞争性能力

| ID | 能力 | 工程目标 |
|---|---|---|
| NZR-P2-001 | Isolated VM worker/service | 不可信mod在独立进程或WASM sandbox运行，支持watchdog、resource quota和fast restart |
| NZR-P2-002 | Per-world VM shard与parallel scheduler | NUMA/worker-aware instance分片、deterministic merge和低contention host batch |
| NZR-P2-003 | Zero-copy typed host ABI | generated layout、borrow/lease、SIMD/column batch与跨DLL allocator provenance |
| NZR-P2-004 | Generated language/host SDK | 从同一schema生成Zr declarations、Rust adapter、docs、compat tests和Editor completion |
| NZR-P2-005 | Interp/Binary/AOT tiering | cook-time AOT、profile-guided tier、平台策略和三后端语义differential |
| NZR-P2-006 | Deterministic rollback/replay | tick input、RNG、state delta、host commands与presentation transaction可复现 |
| NZR-P2-007 | Time-travel debugger | snapshot/delta、reverse step、coroutine/task、GC/object graph与hot-reload generation视图 |
| NZR-P2-008 | Semantic authoring与refactor | LSP增量语义、跨package rename、code action、host schema currentness和semantic merge |
| NZR-P2-009 | Signed script package supply chain | dependency lock、SBOM、signature、policy、revocation、entitlement和artifact provenance |
| NZR-P2-010 | Multi-language backend SDK | 共享lifecycle/value/host/debug/GC certification，不把ZrVM细节固化进neutral runtime |
| NZR-P2-011 | Remote/distributed script build | hermetic worker、content-addressed cache、attestation、cancel和atomic artifact publication |
| NZR-P2-012 | Competitive language quality lab | reference corpus、fuzz、semantic differential、compile/runtime/GC性能和regression bisect |

这些能力只有在P1的provider truth、typed transaction、memory policy、debug/diagnostic和产品资格完成后才有意义。单独增加feature flag、空trait或演示benchmark不计进度。

## 10. 目标架构与硬切边界

~~~text
Zr Source Workspace
  documents + dependency lock + host schema generation
        |
        v
Hermetic Zr Toolchain BuildSet
  parse/type/SemIR/bytecode/AOT + diagnostics + debug maps
        |
        v
Signed Script Artifact
  module graph + interface/layout hashes + budgets + provenance
        |
        v
LanguageActivationPlan
  target + carrier + provider + native SDK + policy + compatibility
        |
        v
ZrVmWorld / isolated worker generation
  thread contexts + instance shards + scheduler + GC/memory owner
  immutable callsite/interface tables + reload transaction
        |
        +-> ScriptWorldTransaction: typed bulk input/output/commands/events
        +-> Diagnostic/Trace/Debug streams with budgets
        +-> Editor workspace/build/debug/profiler using same generations
~~~

硬切后，shipping runtime不得隐式编译源码；`dist`不得只导出metadata却宣称提供runtime能力；反射JSON与逐元素byte array不得留在帧热路径；全局mutex不得成为`Send/Sync`和多实例并发的永久证明；builtin scene system不得冒用缺失provider的owner名称。

## 11. 分层重构里程碑

### M0 · Truth Freeze与开放Failure复验

- 重取186路径、外部ZrVM clean revision、四份failure和target/carrier矩阵；
- required provider在App preflight fail-close，NativeDynamic降级为metadata-only或删除虚假能力；
- 禁止新增runtime compile、JSON reflection/state热路径与永久ignore。

### M1 · Toolchain、BuildSet 与 Artifact

- 实现Runtime21定义的source/build receipt、dependency lock、bytecode verifier和atomic publication；
- Editor/cook成为唯一compiler owner，Runtime只消费immutable signed artifact；
- dist/source/static carrier共享同一artifact和compatibility identity。

### M2 · Language Provider与Target Composition

- App/catalog输出`LanguageActivationPlan`，Client/Server/Editor逐target验证provider；
- editor catalog接入Zr language provider，缺backend/native SDK在启动前可操作地失败；
- builtin neutral systems与Zr plugin owner拆清，禁止伪装registration。

### M3 · Typed Host、Reflection 与 Transaction

- generated tagged value/handle/interface ABI替代数组猜测和JSON热路径；
- callsite绑定package/world/schema generation与字段权限；
- scene/frame调用批量化为可预算、可回滚的`ScriptWorldTransaction`。

### M4 · Runtime Context、Concurrency 与 Isolation

- 用正式thread/context/root API替代全局mutex安全模型；
- per-world shard、callback quiescence、deadline/cancel与可信度分级隔离；
- native assert/hang/OOM有watchdog、slot quarantine和last-good recovery。

### M5 · GC、Memory 与 Hot Reload

- 采集并执行soft/hard bytes policy、allocation rate与fragmentation；
- reload预编译候选、原子切generation、typed state migration和失败rollback；
- watcher/Editor build事件都携带source/artifact generation，拒绝旧结果覆盖。

### M6 · Editor、Debugger 与 Profiling

- Source Workspace、LSP、build status、diagnostic jump、breakpoint与multi-session debug闭环；
- stack/locals/watch/evaluate、script CPU/allocation/GC/host-call timeline接入产品；
- Script Class/Component/Field使用stable IDs并通过rename/save/reopen/cook。

### M7 · Product与跨平台资格

- 接走10个Vampire测试，增加WOC/真实class-array-field-helper和presentation transaction；
- clean Windows/Linux/macOS及明确的Android/iOS/game-WASM矩阵；
- fault、fuzz、1/100/10k scale、双World、reload、shutdown和长稳全部绑定BuildSet。

### M8 · 性能与竞争性验收

- 对照interp/binary/AOT、global-lock旧基线和per-world新架构；
- 报告compile latency、frame p50/p95/p99、host-call throughput、GC pause、heap/RSS和lock wait；
- correctness、determinism、failure recovery和同工作负载证据先通过，再声明优于参考引擎。

## 12. 产品资格门

| Gate | 验收内容 |
|---|---|
| G01 | 186个selected path存在且fingerprint可重建；外部toolchain为固定clean revision |
| G02 | Client/Server/Editor每个required Zr selection都解析唯一provider或preflight失败 |
| G03 | source/static/NativeDynamic carrier的effective capability真实且无metadata-only Ready |
| G04 | clean clone无需人工本机路径即可重建同一Zr toolchain与artifact digest |
| G05 | shipping runtime不解析/编译source，只装载已验证artifact |
| G06 | `.zro`通过header/section/checksum/CFG/type/layout/budget verifier后才执行 |
| G07 | package完整依赖闭包受containment、trust、bytes/items/depth/time预算 |
| G08 | lifecycle interface有versioned signature、required/optional和typed lookup result |
| G09 | missing lifecycle不会静默改变persistent/stateless policy |
| G10 | state snapshot是bounded typed binary并绑定schema/package/world/generation |
| G11 | hot reload候选先compile/verify/stage，commit失败恢复旧artifact和state |
| G12 | 多package、多World、PIE并发无进程全局串行瓶颈或跨world状态泄漏 |
| G13 | native thread attach/context/root/safepoint合同经模型与sanitizer测试 |
| G14 | trusted in-process和untrusted isolated模式都有crash/hang/OOM恢复receipt |
| G15 | value ABI无损覆盖标量、字符串、bytes、typed aggregate、optional/result和handle |
| G16 | bytes批量传输，1 MiB payload不发生逐元素VM allocation |
| G17 | host handle不可由普通整数伪造，stale/cross-owner/cross-generation确定拒绝 |
| G18 | reflection read/write按字段权限、capability、world和stage执行 |
| G19 | callsite在catalog/schema/reload A-B-A后不会错误命中新owner |
| G20 | 一帧脚本输入、state、command、event与presentation形成单一transaction receipt |
| G21 | callback async/cancel/quiescence明确，unload后无late callback或native registration |
| G22 | scheduler使用真实access plan并证明多实例吞吐与确定性merge |
| G23 | soft/hard memory limit由live bytes驱动且hard limit可隔离终止slot |
| G24 | GC deadline可执行，telemetry含heap、allocation、fragmentation、cycle和slot identity |
| G25 | compiler/runtime/host/GC diagnostic保留code、range、cause、build和correlation |
| G26 | Editor source/LSP/build/reload/debug/profiler消费同一workspace/artifact generation |
| G27 | 10个Vampire owner测试在插件真实backend中运行后，旧ignore被硬删除 |
| G28 | WOC/Vampire真实产品覆盖class/array/field/helper、reload、HUD/menu和failure recovery |
| G29 | malformed package/value/state/FFI和panic/crash fuzz不会越权、泄漏或挂死host |
| G30 | Windows/Linux/macOS及声明的移动/Web目标都有current BuildSet receipt |
| G31 | 1/100/10k instance与长稳报告compile、frame、GC、heap、RSS、host-call和lock wait |
| G32 | `git diff --check`、Markdown path/ID/count/currentness和四份open failure审计通过 |

## 13. 禁止的临时修补

1. 禁止把`backend-zr-vm`加入默认feature就宣称产品闭合；外部toolchain、artifact和平台仍须资格。
2. 禁止让NativeDynamic继续导出空behavior同时宣称`script.behavior.v1`可用。
3. 禁止为每种复杂值增加新的JSON字符串host函数。
4. 禁止把所有Array继续解释为bytes或用`i64`扩展更多host handle种类。
5. 禁止用更大的全局mutex、更多`.cloned()`或延长锁范围解决并发安全。
6. 禁止捕获native assert/hang为普通字符串错误并继续复用同一session。
7. 禁止在Runtime startup继续承担Editor/cook compiler职责。
8. 禁止复制Vampire测试到插件后仍保持ignore、mock backend或不验证行为结果。
9. 禁止以历史15/15、当前41个插件test attribute或descriptor smoke test替代BuildSet资格。
10. 禁止把Unity Graphics的debug transport误称为语言VM参考实现。

## 14. 本轮产出边界

本轮只完成静态review、纵向owner路由、48项P1、12项P2、8个里程碑和32个资格门；没有修改Rust/C/TOML/Zr生产实现，没有关闭四份failure，没有运行Cargo或真实VM。后续实施必须从M0重新冻结clean external toolchain和当前并发源码，再按failure最低owner修复。
