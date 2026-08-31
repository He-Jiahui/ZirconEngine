---
related_code:
  - Cargo.toml
  - .github/workflows/ci.yml
  - .github/workflows/profile-feature-contract.yml
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/entry/runtime_library/runtime_session/operation.rs
  - zircon_app/src/entry/runtime_library/wake_registry.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/renderdoc.rs
  - zircon_editor/src/core/gateway/session/gateway.rs
  - zircon_editor/src/core/gateway/session/operations.rs
  - zircon_editor/src/core/gateway/session/output.rs
  - zircon_editor/src/core/process.rs
  - zircon_hub/src/process/editor_focus/probe.rs
  - zircon_hub/src/projects/shared_recent_projects.rs
  - zircon_runtime_interface/src/buffer.rs
  - zircon_runtime_interface/src/runtime_api/abi/api_table.rs
  - zircon_runtime_interface/src/runtime_api/session/operation.rs
  - zircon_runtime_interface/src/runtime_api/frame/highlight_set.rs
  - zircon_runtime_host/src/foreign_output/state.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/dynamic_api/bounded_json.rs
  - zircon_runtime/src/dynamic_api/session/ffi.rs
  - zircon_runtime/src/dynamic_api/session/operation.rs
  - zircon_runtime/src/plugin/native_plugin_loader/abi_declarations.rs
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_calls.rs
  - zircon_runtime/src/plugin/native_plugin_loader/ffi_panic_guard.rs
  - zircon_runtime/src/plugin/native_plugin_loader/load_discovered.rs
  - zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_abi.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter/abi_decode/read.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter/ecs_registration/mod.rs
  - zircon_runtime/src/scene/ecs/archetype/table/column.rs
  - zircon_runtime/src/scene/ecs/component/table_column.rs
  - zircon_runtime/src/scene/ecs/commands/inline_command_arena.rs
  - zircon_runtime/src/scene/ecs/query/query_state/many_item_array.rs
  - zircon_plugins/plugin_sdk/src/native.rs
  - zircon_plugins/plugin_sdk/src/dist.rs
  - zircon_plugins/native_dynamic_fixture/native/src/lib.rs
  - zircon_plugins/navigation/native/build.rs
  - zircon_plugins/navigation/native/src/crowd.rs
  - zircon_plugins/navigation/native/src/tile_cache.rs
  - zircon_plugins/navigation/native/native/recast_bridge.cpp
  - zircon_plugins/physics/runtime/src/backend/jolt/native_world.rs
  - zircon_plugins/physics/runtime/src/backend/jolt/runtime.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/runtime_owner.rs
  - tools/cargo-zircon/src/plugin/validate/native_artifact.rs
  - tools/tests/test_frameworks_06_ci_toolchain_contract.py
  - tools/tests/test_frameworks_06_dependency_governance_contract.py
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/08a-physics-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08d-navigation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/21-zr-language-parser-type-system-semir-bytecode-package-loader-vm-runtime-review.md
  - docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
  - docs/plans/optimize/zircon_runtime_interface/05-runtime-host-foreign-output-safe-api-ownership-admission-budget-fuse-observability-review.md
  - docs/plans/optimize/zircon_tooling/01-workspace-toolchain-ci-validation-and-developer-entrypoints-review.md
  - docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
  - docs/plans/optimize/zircon_tooling/17-repository-content-source-set-ignore-generated-vendor-license-distribution-review.md
  - docs/plans/optimize/zircon_tooling/20-cargo-package-workspace-feature-dependency-target-graph-build-receipt-review.md
reference_engines:
  - dev/bevy/crates/bevy_ecs/src/world/unsafe_world_cell.rs
  - dev/bevy/crates/bevy_ecs/src/bundle/writer.rs
  - dev/Fyrox/fyrox-core/src/pool/mod.rs
  - dev/godot/core/extension/gdextension.cpp
  - dev/godot/core/extension/gdextension.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleInterface.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Common/CoreUnsafeUtils.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/CoreDataSystems/InstanceDataSystem.Jobs.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 21 · Unsafe Rust、FFI、Native Memory、Thread Affinity、Panic 与 Unload Safety 治理差距

## 1. 结论

ZirconEngine的`unsafe`并非少量边缘代码。当前tracked、非`dev/`、非`docs/`范围有17,263个Rust文件；其中234个文件出现1,656行`unsafe`相关文本。按路径排除`tests`、`tests.rs`、`*_test.rs`等明显测试后，仍有约810个production-like `unsafe {}` block、131个`unsafe fn`候选、231个`unsafe extern`声明或定义。另有212处`repr(C)`、663行裸指针类型使用，以及18个手工`unsafe impl`，其中13个在production-like路径。真实编译表面还会因39个plugin dist package的宏展开增加，源码文本统计不是最终边界上限。

这些低层实现大多有合理存在理由。Runtime DLL、native plugin、RenderDoc、Windows Job Object、Jolt、Recast/Detour、ZrVM和高性能ECS都无法仅靠安全Rust完成现有边界；`LoadedRuntime`持有DLL直至session销毁，Runtime export有`catch_unwind`总入口，foreign output已有预算与release owner，ZrVM owner明确全局锁和drop顺序，ECS列存储也对部分allocation/move/drop操作写了局部`SAFETY`说明。这些机制应保留并加强，不能为了“unsafe数量归零”退回复制型、全局锁型或低性能实现。

真正的工程缺口是仓库没有把unsafe当作受治理的资源。159个Cargo package中只有WOC的4个crate声明`#![forbid(unsafe_code)]`；没有workspace `unsafe_op_in_unsafe_fn`、`undocumented_unsafe_blocks`或`missing_safety_doc`策略，没有UnsafeUnit清单、owner、hazard class、线程/卸载合同和source-bound evidence。约810个production-like unsafe block只有72行可由文本识别为`SAFETY:`/`# Safety`等说明；73个实际production public/restricted-public `unsafe fn`中，只有8个在声明前18行内有`# Safety`章节。数字不能直接证明UB，但足以证明安全前提大多只存在于调用者默契和局部代码形状中。

具体soundness和ABI缺陷已经被专项报告拥有：Plugins01登记`NativePluginStatic<T>` blanket `Sync`、可复制/可伪造owned buffer、任意生命周期slice、callback panic和load-before-admission等P0；Runtime Interface01/05拥有DLL ABI、foreign ownership和safe-host carrier；Runtime03拥有ECS aliasing/parallel schedule；Runtime08A/08D拥有Jolt与Recast owner；Tooling01/10拥有Miri、sanitizer、fuzz和CI可达性。本篇不重复这些finding，也不把同一裸指针再计一次。

本篇新增的是横跨owner的安全控制面：**没有新增P0，登记28项P1和10项P2**。目标不是建立一套脱离代码的“安全文档”，而是让每个unsafe unit都能回答：谁拥有它、为何必须unsafe、调用前提是什么、指针/内存/线程/代次/卸载由谁保证、panic如何封闭、哪组Miri/sanitizer/child-process/soak证据验证了当前source与build。

## 2. 审查边界与计数方法

### 2.1 全仓物理盘点

| 根目录 | `unsafe`相关行 | 涉及文件 | production-like unsafe block | production-like block文件 |
|---|---:|---:|---:|---:|
| `zircon_runtime` | 724 | 98 | 304 | 48 |
| `zircon_plugins` | 501 | 67 | 342 | 54 |
| `zircon_editor` | 162 | 31 | 57 | 18 |
| `zircon_app` | 136 | 10 | 67 | 6 |
| `zircon_runtime_interface` | 90 | 16 | 8 | 3 |
| `zircon_hub` | 19 | 4 | 13 | 4 |
| `tools` | 18 | 4 | 16 | 4 |
| `zircon_runtime_host` | 6 | 4 | 3 | 3 |
| **合计** | **1,656** | **234 unique** | **810** | **140 unique** |

计数使用tracked Rust source的全文匹配。`production-like`只按路径排除明显测试，因此会把`wgpu_product_tests`等命名异常目录算入production，也可能把production文件内的`#[cfg(test)]`算入；反过来，macro展开、C/C++、系统库和外部ZrVM实现不在Rust token统计中。它是风险inventory基线，不是证明安全或不安全的语法分析器。

补充物理事实：

- `extern "C"`/`extern "system"`、`repr(C)`和裸指针集中在Runtime DLL、plugin SDK/loader、App/Editor gateway、native physics/navigation及平台API；
- 54行`no_mangle`/link-export相关源码只出现在15个文件，但dist macro会为大量package生成导出函数；
- 唯一tracked C/C++产品子树是Navigation native：34个`.cpp`、23个`.h`，连同27个Rust bridge及vendor共91文件、约34,797行source；
- 13个production手工Send/Sync实现分布在`LoadedRuntime`、Windows `JobObject`、plugin fixture、Recast Crowd/TileCache、Jolt world/backend、Plugin SDK static、ZrVM owner和ECS column；
- 全仓没有真实`static mut`定义；文本命中来自`&'static mut`测试类型。这是应保留的正面事实。

### 2.2 高风险cluster

| cluster | 代表代码 | 当前安全边界 |
|---|---|---|
| Runtime DLL | App `LoadedRuntime/RuntimeSession`、Runtime Interface、dynamic API | 函数表/owned output/session teardown已有局部owner，跨build、bad pointer与abort仍由专项报告治理 |
| Native plugin | SDK macros、loader、host callback、bridge/ECS registration | manifest/capability/behavior ABI存在，但safe API soundness、pre-admission load和unload已是Plugins01 P0 |
| ECS | archetype column、TableColumnLayout、inline command arena、query/system | 高性能raw storage真实；前提分散在局部注释、scheduler检查和`TypeId`判断中 |
| Native runtime | Jolt、Recast/Detour、ZrVM | wrapper和Drop存在；线程、全局初始化、worker quiescence和library generation没有共同schema |
| Platform/diagnostic | Windows Job Object、focus probe、recent-project lock、RenderDoc | OS handle与动态符号各自包装，没有统一platform unsafe owner/qualification |
| Tooling | `cargo-zircon` native probe | validation本身会执行`Library::new`与export，需要与产品loader共享trust和crash isolation |

### 2.3 明确继承而不重复计数

| 已有owner | 本篇只引用的事实 |
|---|---|
| Plugins01 | 5项native SDK/loader P0及其具体修复 |
| Runtime Interface01/05 | ABI layout、wire type、foreign allocator、bad carrier、host safe API、fuse和unload矩阵 |
| Runtime03 | ECS aliasing、query conflict、parallel schedule、storage generation和性能结构 |
| Runtime08A/08D | Jolt/Recast具体native owner、thread/lifetime、artifact与规模测试 |
| Runtime21 | 外部ZrVM C/Rust实现、process-global owner、PIC/GC和real backend资格 |
| Tooling01/10 | MSRV、Miri、ASan/UBSan/TSan、fuzz、child process和required test architecture |
| Tooling03 | generated platform host callback为placeholder、产品loop与lifecycle未闭合 |

本篇的Finding只拥有“这些owner如何进入同一Unsafe Safety Control Plane”，不能用一个全局wrapper替代domain修复，也不能把已有P0降格成文档问题。

## 3. 必须保留的工程基础

### 3.1 DLL与foreign output已有owner骨架

`LoadedRuntime`将`Library`与API table放在同一owner，`RuntimeSession`在destroy之后才释放library；required function slot在构造时验证，optional tail按table size读取。App wake trampoline用`catch_unwind`阻止panic跨C ABI。Runtime host foreign-output path已有byte/item/time/depth预算、release回调、fuse和metrics。重构应把这些局部机制提升为统一lease/receipt，不应重新暴露裸function pointer给产品层。

### 3.2 Runtime export和plugin loader已有panic guard入口

Runtime API总入口及native plugin behavior调用已有`catch_unwind(AssertUnwindSafe(...))` helper；loader还区分descriptor、entry、behavior和host callback。问题是guard并非由ABI schema生成，generated platform host、SDK host-ready、tool probe和系统callback仍可形成不同策略。应收敛覆盖面，而不是删除现有guard。

### 3.3 ECS的unsafe用于真实dense storage与并行访问

Archetype column保留allocation layout、ZST、move/drop和change tick；query/system层通过访问描述建立并行冲突图。这是性能基础。Bevy参考也大量使用unsafe，但把`UnsafeWorldCell`的aliasing前提、mutable-access来源和每个unsafe method的Safety章节写在同一抽象上。Zircon需要同等级proof-carrying access witness，而不是退回`Box<dyn Any>`全路径或单线程World锁。

### 3.4 Native wrapper已有Drop与部分线程说明

ZrVM owner明确全局锁、session/registration/runtime逆序drop；Recast TileCache说明mutation经`&mut self`且可在线程间移动；ECS column对Send/Sync写出布局与exclusive owner前提。这些是可迁移的安全论证样板，后续要变成可审计manifest与test link。

## 4. P1差距：Inventory、Policy 与 Source Currentness

### UNSAFE-P1-001 · 没有canonical UnsafeUnitManifest

234个文件的unsafe unit只能靠搜索发现，无法知道哪个是FFI、aliasing、allocator、thread、platform handle、SIMD、GPU lifetime或性能特例。建立`UnsafeUnitManifest`，每项至少记录稳定ID、owner package/module、source span、hazard class、justification、inputs/outputs、invariant、reviewer、introduced revision和qualification set。

### UNSAFE-P1-002 · unsafe没有owner与风险等级

同一统计里混有`NonNull` owner、raw slice decode、C export、OS handle、ECS type erasure和test allocator。当前没有P0/P1 risk、untrusted/trusted caller、process-fatal/isolated、hot-path/cold-path和public/internal维度，无法决定review深度或release gate。由O01 Tooling维护catalog，domain owner维护语义与修复。

### UNSAFE-P1-003 · 159个package没有deny-by-default策略

只有WOC codegen/parity/protocol/runtime四个crate显式`forbid(unsafe_code)`；其余package都可在任意模块新增unsafe。workspace默认deny，只有manifest列出的module或crate可`allow(unsafe_code)`；generated/native boundary通过独立crate隔离，普通domain crate保持forbid。

### UNSAFE-P1-004 · 缺少unsafe专项lint合同

未发现`unsafe_op_in_unsafe_fn`、`undocumented_unsafe_blocks`或`missing_safety_doc`的workspace policy。启用Rust lint与Clippy并不足以证明soundness，但可阻止unsafe fn内部隐式扩大范围、无说明block和public unsafe API无Safety章节。lint版本和suppression必须进入resolved package receipt。

### UNSAFE-P1-005 · Safety说明覆盖不足且格式不可机读

约810个production-like block只有72行可识别Safety说明；73个实际production public/restricted-public unsafe函数中65个在近邻没有`# Safety`。为block、unsafe fn和unsafe impl定义最小模板：validity、alignment、aliasing、lifetime、thread、unwind、ownership、count budget和postcondition；不是写“caller guarantees safety”空话。

### UNSAFE-P1-006 · 13个production unsafe impl没有统一Send/Sync proof

`LoadedRuntime`、Recast、Jolt、ZrVM、SDK static、ECS column和JobObject各自决定线程可移动/共享。只有部分实现有明确SAFETY说明，且没有negative compile-fail、thread-affinity或unload test链接。每个unsafe impl必须拥有具体字段级proof和`ThreadCapability`；blanket generic impl一律由专项owner消除。

### UNSAFE-P1-007 · macro展开后的真实unsafe surface不在inventory

39个dist package使用SDK macro生成static、entry、descriptor和behavior export；源码`no_mangle`统计看起来只有15文件。Unsafe inventory必须在`cargo expand`或等价编译IR层记录每个最终target的export、function pointer和unsafe impl来源，关联macro版本与package feature closure。

### UNSAFE-P1-008 · 34个C++与23个header不进入Rust unsafe预算

Navigation bridge/vendor是唯一tracked native C/C++产品树，但Rust token门看不到其allocation、array bound、exception、thread和ABI风险。建立`NativeSourceUnit`同级catalog，区分first-party bridge与vendored upstream，记录compiler flags、exception/RTTI、sanitizer支持、patch provenance和symbol surface。

### UNSAFE-P1-009 · unsafe inventory没有source/currentness fingerprint

现有报告可以标`source_recheck_required`，但unsafe单元没有body hash、dependency ABI hash或新增/删除diff。每次PR生成UnsafeDelta：新增、扩大visibility、增加caller、改变layout/thread/drop、删除Safety证据都需owner review；纯行数变化不能自动通过。

## 5. P1差距：FFI、Memory、Panic 与 Dynamic Library

### UNSAFE-P1-010 · raw pointer admission分散在107个文件

null/len、alignment、count、UTF-8、enum、owner token和generation检查由Runtime Interface、Runtime loader、Plugin SDK、App、Editor及native backend分别实现。建立小型、domain-neutral的`ForeignInputAdmission` primitive和生成规则；它只负责shape/provenance/budget，不吞掉各ABI的semantic validation。

### UNSAFE-P1-011 · unsafe输入可在safe wrapper中获得与source无关的生命周期

具体`bytes_from_slice<'a>`缺陷由Plugins01 P0拥有。本篇新增的控制面要求所有raw-to-reference helper声明borrow source，并由API形状把返回lifetime绑定到`ForeignCallScope`/owner lease；inventory禁止出现自由`<'a>`返回foreign slice而没有scope参数。

### UNSAFE-P1-012 · out parameter没有统一initialized witness

Runtime Interface01已拥有失败初始化语义；当前App与Editor operation poll仍以`MaybeUninit`接收foreign success再`assume_init`。横向修复应生成`ForeignOut<T>`或先写valid sentinel，并让callee成功状态与`Written<T>`不可分离；lint/contract test扫描所有73个production public unsafe入口，不能逐函数靠约定。

### UNSAFE-P1-013 · foreign allocator provenance没有统一receipt

Runtime owned result、plugin owned buffer、Recast结果、Jolt object和ZrVM binding使用不同free策略。具体double-free/forgery由专项owner修复；全局建立`ForeignAllocationReceipt { allocator_id, module_generation, allocation_id, bytes, alignment, release_fn, state }`，禁止仅靠data/len/capacity可逆hash证明owner。

### UNSAFE-P1-014 · count与byte arithmetic没有共同上限/overflow policy

raw slice、frame pixels、FFI arrays、native bake/result和ECS layout都执行`count * element_size`或offset arithmetic；部分用`checked_mul`，部分依赖先前validation或debug assertion。所有untrusted/count-derived allocation先通过`ForeignExtent`检查max items/bytes/alignment/overflow，再允许创建slice或layout。

### UNSAFE-P1-015 · no-unwind trampoline不是ABI生成物

Runtime export与部分plugin call已有guard，SDK host-ready缺口已由Plugins01 P0拥有，generated platform host又是另一条路径。每个export/callback descriptor生成统一trampoline，明确`C`还是`C-unwind`、panic映射、diagnostic budget和process-fatal policy；不得靠reviewer记住哪些函数需要`catch_unwind`。

### UNSAFE-P1-016 · Cargo profile没有panic/ABI policy

四个workspace没有为产品、plugin、tool、test和sanitizer target冻结panic策略。unwind穿过C边界、callback内部abort、`panic=abort`和catch行为会随profile改变。ResolvedPackageGraphReceipt必须包含panic strategy；ABI schema声明允许/禁止unwind，测试覆盖debug/release同一故障分类。

### UNSAFE-P1-017 · 动态库lease在四条路径重复实现

Runtime DLL、native plugin、RenderDoc和`cargo-zircon` probe各自调用`Library::new/get`。App的`LoadedRuntime`保活关系是正确基础，但没有公共`NativeModuleLease`来限制symbol/function pointer逃逸、调用计数、thread affinity、trust admission和unload。工具probe也必须在child process隔离，不能让“验证”直接污染控制进程。

### UNSAFE-P1-018 · unload quiescence没有跨domain证明

Unreal明确`PreUnloadCallback -> ShutdownModule -> FreeDllHandle`和逆依赖顺序；Godot按initialization level deinitialize后close。Zircon的Runtime DLL、plugin、Jolt worker、Recast query、ZrVM callback和Editor session没有共同`StopAdmission/Drain/Destroy/Unload`状态及outstanding lease count。建立`NativeModuleGeneration`和quiescence receipt，任何raw handle/symbol/callback存活时禁止unload。

### UNSAFE-P1-019 · thread affinity只存在于注释或`&mut self`

Recast/Jolt owner可Send、LoadedRuntime可Send+Sync、Windows handles跨task移动，但native库是否允许create/call/drop跨线程没有target-specific合同。`ThreadCapability`区分MainThread、CreatorThread、SerializedAnyThread、ConcurrentRead和WorkerSafe；executor在调度前验证，Drop在错误线程时形成fatal/cleanup policy而非静默调用。

### UNSAFE-P1-020 · process-global native初始化没有统一owner

Jolt使用`OnceLock`注册allocator/factory/types，ZrVM使用process-wide mutex，RenderDoc与plugin loader也依赖进程全局模块状态。建立`NativeRuntimeService`拥有init generation、configuration、refcount、shutdown policy和fork/test isolation；单个world/plugin不能自行初始化或假定永不反初始化。

### UNSAFE-P1-021 · native handle缺少统一owner/generation identity

Jolt body/shape pointer、Recast Crowd/TileCache handle、DLL API pointer和plugin function table各自封装，但跨reload/world replacement时没有共同generation。所有可持久/跨task raw handle使用`OwnerId + Generation + Slot`，不把地址或第三方整数直接暴露为可复用identity。

### UNSAFE-P1-022 · destructor/free失败无法进入terminal receipt

Rust `Drop`不能返回错误，native free、DLL unload、worker join和foreign release失败有的abort、有的log、有的忽略。高风险owner提供显式`close()`/`shutdown()`生成receipt；Drop只作最后防线并记录leak/fatal state。release必须幂等或线性消费，不能在Drop中猜测foreign状态。

### UNSAFE-P1-023 · Runtime session的abort路径缺少统一crash envelope

App和Runtime在reentrant destroy/release等不变量破坏时调用`process::abort()`，避免use-after-free方向是正确的，但abort前没有保证写出source/build/session/operation/native lease的低分配crash envelope。把不可恢复invariant映射到Crash Service emergency writer，保持立即终止，不允许改成继续运行。

## 6. P1差距：ECS 与 High-Performance Unsafe Core

### UNSAFE-P1-024 · ECS没有单一Safety Model文档与owner

Archetype column、TableColumnLayout、query fetch、system param、parallel schedule和command arena共同决定aliasing，却没有类似Bevy `UnsafeWorldCell`的统一模型说明“谁可创建、何时可复制、访问如何分区、哪些metadata可共享、world replacement如何失效”。建立`EcsSafetyModel`并由Runtime ECS owner维护，不散落成函数注释集合。

### UNSAFE-P1-025 · access validation与raw access没有proof-carrying witness

Scheduler先计算read/write conflict，system/query随后在另一层执行raw fetch；二者靠相同ID和代码路径保持一致。生成不可伪造的`ValidatedWorldAccess<'world>`/`QueryAccessWitness`，包含world generation、component set、mutability和batch scope；unsafe fetch必须消费witness，不能只收raw pointer与index。

### UNSAFE-P1-026 · erased move/drop callback只有plain unsafe fn pointer

`TableColumnLayout`保存`write_box/take_box/drop_value`，正确性依赖TypeId、Layout、live-row count和allocation owner同时匹配。把它收敛为`ComponentVTable`，构造时绑定type/layout/drop strategy/schema；调用需要column generation与slot state witness，debug/qualification记录double-drop、uninitialized-read和layout mismatch。

### UNSAFE-P1-027 · inline command arena的payload pointer缺少线性消费合同

arena用`MaybeUninit<u8>`和raw offset换取无堆分配命令，方向合理；但payload ptr、drop callback、block reuse、panic unwind和deferred apply跨多个类型。建立`CommandArenaLease`与slot状态`Free -> Initialized -> Moved/Applied -> Dropped`，panic/fault test验证每个命令恰好drop一次且不会在arena reset后访问旧pointer。

### UNSAFE-P1-028 · UnsafeEvidence没有绑定到source、target和hazard

Tooling01/10已经登记Miri/sanitizer/fuzz lane缺失；本篇新增`UnsafeEvidenceReceipt`合同：记录UnsafeUnitId、source/build/target/toolchain、features、test/fuzz corpus、sanitizer、seed、duration、result和artifact digest。Miri通过不能证明C++，ASan通过不能证明data race，TSan通过不能证明Rust aliasing；每个hazard必须选择匹配证据。

## 7. P2长期能力

| ID | 能力 | 价值与边界 |
|---|---|---|
| UNSAFE-P2-001 | Unsafe delta risk scoring | 按public ABI、untrusted input、allocator、thread、unload和hot path评估review深度，不以行数排名代替语义 |
| UNSAFE-P2-002 | 编译IR/MIR级inventory | 记录macro monomorphization与最终export，不能依赖文本扫描作唯一真相 |
| UNSAFE-P2-003 | ECS provenance/model checking | 对query split、deferred command、panic/drop、world replace做Miri/Loom或小状态模型，不声称形式化证明整个World |
| UNSAFE-P2-004 | C/C++静态分析与hardened flags | clang-tidy、warnings-as-errors、CFI/CFG、stack protector和exception policy进入native target receipt |
| UNSAFE-P2-005 | 第三方native进程隔离 | 不可信importer/validator优先worker process；不是用IPC掩盖同进程核心backend的owner缺陷 |
| UNSAFE-P2-006 | capability pointer/handle table | 长期减少裸地址跨层传播，支持permission、generation和revocation |
| UNSAFE-P2-007 | hot-reload safe-point verification | 栈、callback、job、GPU/native allocation都退出旧generation后才允许卸载 |
| UNSAFE-P2-008 | memory tagging/guard-page canary | 在支持平台对foreign buffer、arena和native object做抽样资格，不作为所有平台required前置 |
| UNSAFE-P2-009 | cross-language differential fuzz | C/C++/Rust consumer对同一header、bad input和ownership trace给出一致状态与释放行为 |
| UNSAFE-P2-010 | unsafe性能收益receipt | 每个高风险unsafe优化证明相对安全baseline的CPU/alloc/cache收益；无收益或收益消失时收回unsafe |

## 8. 目标架构

```text
RepositoryContentManifest + ResolvedPackageGraphReceipt
                         |
                         v
                 UnsafeUnitManifest
      (owner / source span / hazard / invariant / target)
                         |
        +----------------+----------------+
        |                |                |
        v                v                v
 ForeignCallScope  ThreadCapability  EcsAccessWitness
 pointer/extent/   creator/main/     world/generation/
 allocator/out     serialized/worker component access
        |                |                |
        +----------------+----------------+
                         |
                         v
         NativeModuleLease + ModuleGeneration
        admit -> call -> drain -> destroy -> unload
                         |
                         v
              UnsafeEvidenceReceipt
       source/build/tool/hazard/result/artifact
                         |
                         v
          BuildSet / ValidationSet / Release Gate
```

边界原则：

1. `UnsafeUnitManifest`只拥有横向身份、风险、证据与currentness；domain报告继续拥有真实语义。
2. safe wrapper必须使错误调用在类型或admission层不可表达；仅在注释里要求caller小心不算闭合。
3. Thread、owner、generation、allocator和library lease是正交identity，不能合成一个opaque token。
4. unsafe不是性能证明。保留unsafe必须有benchmark与failure semantics；移除unsafe也不能接受数量级性能退化。
5. native第三方源码、macro展开和generated platform host都进入最终target inventory。

## 9. 参考引擎对照

### 9.1 Bevy：unsafe集中在可解释抽象，aliasing前提贴近API

Bevy ECS同样大量使用unsafe；本地`bevy_ecs`对`# Safety`/UnsafeWorldCell/unsafe Send-Sync的匹配有716行、76文件。`UnsafeWorldCell`明确说明world数据放在`UnsafeCell`、system initialize验证不相交访问、mutable whole-world borrow的排他条件，并在Send/Sync实现旁写字段级SAFETY。Zircon应学习这种“unsafe核心 + safe system surface + access witness”，不应只比较unsafe数量。

### 9.2 Fyrox：参考源码也可能是负面样本

Fyrox pool的`RefCounter(UnsafeCell<isize>)`直接unsafe实现Send/Sync，计数增减没有原子或近邻安全论证。它说明参考引擎不能自动当作完成标准；Zircon要以自己的并发模型、Miri/TSan和owner证据判断，不复制一个已有项目的unsafe形状。

### 9.3 Unreal：native module有显式pre-unload与逆依赖shutdown

`IModuleInterface`区分Startup、PreUnload、Shutdown、PostLoad和dynamic-reload policy；ModuleManager在FreeDllHandle前执行相应阶段并保留逆依赖顺序。Zircon不需要复制UObject/module类层次，但所有DLL function pointer、callback、allocation和worker都必须归入可证明的module generation与quiescence流程。

### 9.4 Godot：extension初始化级别与library open状态是显式状态机

GDExtension在loader initialize失败时立即close；后续initialize/deinitialize按level单调推进，并在调用前验证library open。Zircon应吸收状态机与level/owner显式性，不能只用`Option<Library>`或函数指针非空推导安全可调用。

### 9.5 Unity Graphics：unsafe性能核与job/native container绑定

Unity Graphics把指针算法集中在`CoreUnsafeUtils`、NativeArray/UnsafeUtility、GPU-driven job和明确buffer size中。它同样不是soundness证明，但展示了unsafe应贴近数据布局、job schedule和capacity合同，而不是散布到产品逻辑。Zircon还需补上Rust特有的aliasing/lifetime和DLL unload证据。

## 10. 重构里程碑

### M0 · Truth Freeze

- 生成17,263 Rust文件的UnsafeUnit baseline，纳入macro展开、C/C++和generated host；
- 将既有Plugins/Runtime Interface/ECS/native P0/P1映射到UnsafeUnitId，不复制finding；
- required gate禁止新增未登记unsafe、unsafe impl、export和native source。

### M1 · Policy 与 Documentation

- workspace启用`unsafe_op_in_unsafe_fn`及分阶段undocumented/missing safety门；
- 普通crate默认forbid，边界crate按manifest局部allow；
- 13个production unsafe impl补字段级proof、negative compile-fail与thread contract。

### M2 · FFI 与 Panic Boundary

- 生成ForeignCallDescriptor、extent/out/allocator wrapper与no-unwind trampoline；
- Runtime DLL、plugin SDK/loader、App/Editor gateway和tool probe共享shape primitive；
- 具体carrier/ABI P0仍按原专项里程碑硬切。

### M3 · Native Module Lifetime

- 建立NativeModuleLease/Generation/ThreadCapability；
- Jolt、Recast、ZrVM、RenderDoc、runtime DLL和plugin接入admit/drain/destroy/unload；
- tool validation转child process，bad DLL不能终止Coordinator/CLI主进程。

### M4 · ECS Safety Core

- 发布EcsSafetyModel，access conflict生成witness；
- TableColumnLayout与command arena使用proof-carrying vtable/slot state；
- Miri/Loom/fault验证ZST、panic/drop、query split、deferred apply和world replacement。

### M5 · Qualification

- 实现UnsafeEvidenceReceipt并接入Tooling10 TestPlan；
- Windows先跑Miri可用子集、ASan/UBSan native、child-process malformed carrier和unload stress；
- Linux/macOS补calling convention、sanitizer和loader矩阵，unsupported必须有owner/expiry。

### M6 · Performance 与 Release

- 为每个hot unsafe unit建立安全baseline与收益证据；
- unsafe delta、evidence currentness和native module receipt进入BuildSet/Release gate；
- 与Unreal或其他引擎比较时冻结场景、配置、失败语义和硬件，不以删除检查换取“更快”。

## 11. 验收门

1. 最终Cargo target的所有unsafe unit、export、native source和macro expansion都能映射稳定owner与hazard。
2. 普通crate新增unsafe默认编译失败；allow只存在于manifest登记的最小module。
3. 每个public/restricted-public unsafe fn有准确`# Safety`，每个unsafe block/impl有近邻字段级说明。
4. 13个production unsafe impl都有Send/Sync proof、negative test和thread/unload qualification。
5. raw pointer转换必须消费ForeignCallScope/Extent；自由任意生命周期slice构造不可编译。
6. 所有out parameter在任何返回状态下可安全检查/释放，success不能伴随未初始化输出。
7. 所有foreign allocation有allocator/module generation与线性release receipt，复制/伪造/double-free被隔离拒绝。
8. 所有C ABI export/callback经过生成的no-unwind trampoline；panic得到typed result或低分配fatal envelope。
9. NativeModule在outstanding symbol/callback/job/allocation/handle非零时不能unload。
10. Jolt/Recast/ZrVM/runtime DLL/plugin的create/call/drop线程符合ThreadCapability。
11. ECS raw access只能消费同world/generation、已验证component access witness。
12. command arena在normal、cancel、panic和world teardown路径每个payload恰好drop一次。
13. Miri、ASan/UBSan、TSan/Loom、fuzz、child-process与unload soak按hazard映射，不用单一green替代全类证据。
14. UnsafeEvidenceReceipt绑定source、resolved package graph、target、toolchain、features和artifact digest。
15. unsafe hot path有CPU/alloc/cache收益与failure behavior；无收益的unsafe被移除。
16. 本篇既有owner映射不重复计数，专项P0在原报告未关闭前仍阻断qualification。

## 12. 本轮证据与限制

- 完成tracked Rust、C/C++、export、unsafe impl、Safety说明、lint、dynamic loader和高风险owner的静态横向盘点；
- 读取App/Runtime DLL、Runtime Interface/Host、plugin SDK/loader、ECS、Jolt、Recast、ZrVM、Hub/Editor平台FFI和tool probe代表实现；
- 读取Bevy ECS、Fyrox pool、Unreal ModuleManager、Godot GDExtension和Unity Graphics unsafe utility/job参考；
- 没有修改production、manifest、workflow或tests；
- 没有运行Cargo/Miri/sanitizer/fuzz/native build、DLL unload、C++ analyzer或性能测试；既有Editor/Hub/WOC/plugin lock阻断不在本轮重复执行；
- 当前Runtime与Editor source有大量其他Session在途修改，unsafe inventory实施前必须重取source fingerprint，故`source_recheck_required: true`。

本轮结论是：Zircon已经拥有值得保留的高性能unsafe基础，但还没有工程级安全控制面。任何“性能优于Unreal”的目标都必须建立在同等功能、同等失败处理、无未定义行为和可重放native lifetime证据上；unsafe可以是性能工具，不能成为省略生命周期、验证和恢复语义的许可证。
