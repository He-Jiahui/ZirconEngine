---
related_code:
  - zircon_plugins/plugin_sdk
  - zircon_plugins/plugin_sdk_examples
  - zircon_plugins/first_party_runtime_catalog
  - zircon_plugins/first_party_editor_catalog
  - zircon_plugins/native_dynamic_fixture
  - zircon_plugins/editor_contribution_fixture
  - zircon_plugins
  - zircon_runtime/src/plugin/native_plugin_loader
  - zircon_editor/src/ui/host/editor_manager_plugins_export
  - zircon_editor/src/ui/host/editor_manager_project.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - tools/cargo-zircon/src/plugin
  - tools/zircon_export/plugin_build.py
  - tools/zircon_export/plugin_validate.py
  - tools/zircon_export/plugin_build_signature.py
  - .github/workflows/ci.yml
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
  - docs/plans/zircon_runtime/runtime/06/failure-2026-07-31-native-plugin-v4-surface-inventory-drift.md
  - docs/plans/zircon_runtime/runtime/06/failure-2026-07-22-native-sdk-callback-global-panic-hook.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/PluginDescriptor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/ModuleDescriptor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/Interfaces/IPluginManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
  - dev/godot/core/extension/gdextension_interface.json
  - dev/godot/core/extension/gdextension_interface.schema.json
  - dev/godot/core/extension/gdextension_interface_header_generator.cpp
  - dev/godot/core/extension/gdextension_library_loader.cpp
  - dev/godot/core/extension/gdextension_manager.cpp
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/bevy/crates/bevy_app/src/plugin_group.rs
  - dev/Fyrox/fyrox-impl/src/plugin/mod.rs
  - dev/Fyrox/fyrox-impl/src/plugin/dylib.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 01 · Plugin SDK、包合同、Catalog、发行链与 Native ABI 工程化差距

## 1. 结论

当前插件域不是“完全没有工程化”。39 份 `plugin.toml` 都由 Rust declaration 同步生成，结构审计没有发现重复 package id、重复全局 module name、缺失声明 crate、capability projection 漂移或 skeleton debt；`cargo-zircon` 已有 scaffold、manifest sync/check、manifest/artifact validate，`zircon_export plugin build/validate` 已能构建独立包、收集 loadable artifact、计算 SHA-256，并可调用外部签名与公证命令。CI 也会检查插件 workspace、运行全部插件测试、验证全部清单并在 Ubuntu 上逐个构建 39 个 dist crate。这些基础应该保留。

但它还不是可交付给第三方、可安全装入产品、也不能保证静态/动态功能等价的插件平台。最严重的五项问题都在边界本身：公开 `NativePluginStatic<T>` 对任意 `T` 无条件实现 `Sync`；owned byte carrier 为 `Clone + Copy` 且可用公开算法伪造 owner token、重复 free；`bytes_from_slice<'a>` 可制造与输入无关的任意生命周期并把坏 null shape 当空；SDK 入口宏允许 `on_host_ready` panic 穿越 `extern "C"` 直接中止进程；Editor/App 又在应用项目 selection 之前先 `Library::new` 并调用目录内每个 editor native candidate 的入口。后者意味着“未选择”或 `enabled = false` 不能阻止 DLL 代码执行，而 runtime loader 不读取构建链生成的 hash/signature sidecar。

发行形态也存在实质性空壳。当前 39 个 dist crate 全部声明 `invoke_command: None`、`bridge_methods: []`、`on_host_ready: None`；只有 glTF fixture-like dist 携带 save/restore state。39 个第一方 native projection 的业务声明又全部 `systems: []`，唯一非空 system 来自 `native_dynamic_fixture`。因此 AI、Physics、Sound、Hybrid GI、Virtual Geometry 等 package 虽统一广告 `native_dynamic`，其 dist DLL 只发布 manifest、module 和 capability metadata，并没有携带对应静态 Rust 插件的算法与系统执行能力。CI 的“39 个 dist 构建成功”证明 DLL 壳可编译，不证明动态安装后功能存在。

版本真值同样未闭合。生成的 39 份 `plugin.toml` 都写 `version = 0.1.0`、`sdk_api_version = 0.2.0`、ABI 3、`engine_compat = ">=0.1, <0.2"`；但 `RuntimePluginDescriptor::package_manifest()` 和 standalone editor descriptor 仍从 `PluginPackageManifest::new()` 得到 SDK API `0.1.0`。`plugin_sdk_examples` 的源码测试还明确断言 `0.1.0`。loader 只检查 native ABI 和 engine range，runtime registration 只检查 `sdk_api_version` 是三段数字，从不确认它受当前 engine 支持。所谓 SDK API version 目前是可漂移、不可执行的 metadata。

本轮登记 5 项 P0、30 项 P1、8 项 P2。最低共享层修复顺序必须是：先封闭 native ABI soundness 和“先加载后选择”的供应链入口；再建立单一生成式 ABI/manifest/version authority 与宿主验证；然后选择一个真实插件完成 static/source/native 三形态行为等价、安装锁定和跨平台发布；最后才把 `native_dynamic` 扩到全部第一方 package。不能继续批量复制 98 行 dist 壳来增加“支持插件数”。

## 2. 审查边界与证据

### 2.1 本轮物理范围

| 集合 | 文件 / 物理行 | 证据等级与边界 |
|---|---:|---|
| `plugin_sdk` 全部 | 21 / 5,676 | E3：declaration、runtime/editor builder、native ABI、dist macros、test support；33 个 test attributes，未运行 |
| `plugin_sdk` production | 18 / 4,842 | E3：排除三个 `tests.rs`；fingerprint `9c5ccf5c3a0a39f4e4f0e83f5141edf29c46759dade16c6e55769eb427b237d7` |
| SDK examples | 7 / 604 | E3：editor source 与 native dist 示例；fingerprint `3b10f51a35ce4255ef990b48936ccce234a05434ee06e14e6d489faa8aa6d770` |
| 39 个 dist crate `lib.rs` | 39 / 4,248 | E3：宏参数、entry、behavior、state 与 unit tests；fingerprint `c1bf3303042d70c77378a8393609513df6bf086efb6447e17f0fd7ebfec13a64` |
| first-party runtime/editor catalog | 8 / 1,424 | E3：catalog wiring 与 source-manifest parity tests；fingerprint `51062a23619b6e357b805e7b2b0798fb947364713d27ac69148188ebd6c57344` |
| native/editor contribution fixtures | 2 / 884 | E3：真实 callback、V2 debt、panic/state/hot reload与editor contribution；fingerprint `d4cb4890704831e498f3c8475458fd83ada6cd872f244d13aba9838d4c4497ad` |
| `cargo-zircon` plugin owners | 9 / 3,143 | E3：check/sync/scaffold/validate/native artifact probe；fingerprint `4394f5bdca242c6c6f5bf427d66c33d8194e40abf2797f4019f8b406a745b21b` |
| `zircon_export/plugin_*.py` | 87 / 9,522 | E2-E3：standalone validate/build/package/hash/signing；不代表完整 export pipeline；fingerprint `b282b0d37fa3f5851fbc9103df87336087aae3f9cf26a89b3e1eccc7c1ba67c8` |
| host ABI/load admission focused set | 9 / 3,862 | E3：host ABI mirror、descriptor/entry、candidate/compat/load、Editor/App selection；fingerprint `fa2ee015e35c6b23227b1266bee3551208c16e6cc9004221bed227585d324c36` |
| structured repository inventory | 39 plugin manifests / 140 Cargo manifests / 39 dist Cargo manifests | E3：TOML parse、required fields、identity、version、dependency row与dist形态统计 |

指纹算法与前序报告一致：相对路径排序，逐文件 SHA-256，再对 `path<TAB>hash<LF>` 清单取 SHA-256。上述 focused paths 成文前不在工作区修改列表；`hybrid_gi` 的四个算法实现文件有其他 Session 修改，但不属于本轮 SDK/package/dist/loader 合同集合。本文仍标记 `source_recheck_required`，因为实施前必须重新读取 ABI 两端、selection/load 顺序和生成清单。

本轮没有逐审 AI、Physics、Sound、Rendering、Hybrid GI 等插件内部算法；这些由既有 runtime/graphics 系统报告及后续 plugin-family 报告拥有。本文只判断同一业务实现能否通过 SDK、manifest、catalog、dist artifact 和 host admission 被正确发布与装载。

### 2.2 结构与动态证据

本轮实际运行 `python tools/audit_plugin_structure.py --json`，结果为 `classified-and-clear`：39 份预期/生成 manifest、0 schema violation、0 generated-header violation、0 skeleton migration debt、29 runtime descriptor roots、0 single-source violation、0 capability mismatch、41 个 dist build matrix entry、0 dependency boundary violation。该结果证明结构门工作正常，不证明 native ABI soundness、签名准入、动态行为等价或产品加载安全。

本轮没有运行 Cargo、真实 DLL、Editor/App、外部 signer、Windows/macOS 构建、sanitizer、fuzzer、旧新 engine/plugin skew 或崩溃隔离测试。现有 CI 和历史计划记录只作为“已有门”证据，不被本文重新宣称通过。静态源码已经足以确认无条件 unsafe impl、free 算法、入口 panic 路径、load-before-select 顺序、version 默认值和 39 个 dist macro 参数。

### 2.3 纵向调用链

本轮逐项追踪了：

1. `declare_plugin!` -> manifest sync -> 39 份 `plugin.toml` -> runtime/editor source descriptor；
2. project `ProjectPluginSelection` -> Editor/App native discovery -> compatibility -> `Library::new` -> descriptor/entry -> selection projection；
3. `plugin_sdk::native/dist` -> 39 个 cdylib -> `cargo-zircon` native artifact probe -> CI dist matrix；
4. dist entry report -> registration manifest/bridge methods/callback/state -> native live host materialization；
5. `zircon_export plugin build` -> loadable artifact -> SHA/signing sidecar/native loader manifest -> product runtime loader；
6. package dependency/capability/version fields -> runtime validation/compatibility -> catalog selection；
7. SDK tests/examples -> 实际缺失的非 Rust consumer、跨版本、跨平台、malicious carrier 和 static/native parity gate。

### 2.4 参考源码边界

- Unreal 的 plugin manager 把 discovered 与 enabled 分开，按 project/target/command-line/default policy 配置 enabled set 后，再按 module loading phase 装载 enabled plugin modules。descriptor 还拥有 EngineVersion、SupportedTargetPlatforms、sealed/disallowed dependency、explicit loading、content/config/localization 与 pre/post build policy。它不是 sandbox，也不能替代 Zircon 的签名/隔离设计；可借鉴的是“发现不等于执行”和 package/module policy 分层。
- Godot 的 GDExtension 从 JSON/schema 生成 C header，以 get-proc-address 发布接口，携带 engine version 和 initialization level，并由 manager 执行 load/unload/reload 生命周期。它同样在进程内执行、不提供天然安全边界；可借鉴的是单一生成接口、非 Rust consumer 和分级初始化/重载合同。
- Bevy `Plugin` 的 `build/ready/finish/cleanup/is_unique` 是静态 Rust app 组合合同，不是第三方二进制 ABI。Zircon 的 source plugin 可借鉴完整生命周期，但不能把 Bevy 风格 trait object 当稳定 DLL surface。
- Fyrox 的 dynamic plugin 明确警告 Rust trait-object 动态链接受编译器版本影响、不适合作为稳定生产 ABI，同时保留 library owner、prepare/reload 和文件复制。它直接反证“同 workspace Rust DLL 能加载”不等于公开 SDK。
- 仓内 Unity Graphics 是渲染 package/test 源码，不包含 Unity Player/native plugin manager 的权威实现；本文不推断其闭源发行与信任行为。

## 3. 已有可保留基础

1. `declare_plugin!` 已把 package id、target、platform、capability、maturity、packaging 和 native entry 集中到声明 owner，manifest sync/check 会拒绝生成文件漂移。
2. 39 份 manifest 当前均可解析且 identity 唯一；module crate 存在，catalog capability projection 和 workspace dependency boundary 有自动审计。
3. SDK feature 已至少区分 declaration/runtime/editor/native；dist crate 使用 `default-features = false`，避免把完整 editor/runtime 无条件带入纯 ABI 壳。
4. command manifest 已使用 `deny_unknown_fields`，slot dense、payload schema 与 output limit 有校验；V4 output sink 改成 host-owned writer，方向优于跨库返还任意 `Vec`。
5. native loader 会先检查 distribution ABI 与 engine range，再验证 descriptor id、embedded manifest、capability 和 entry symbol；loaded library owner 被 report/host 保留。
6. callback panic guard 已不再替换 process-global panic hook，普通 command/state callback 可映射为 typed panic status。
7. native live host 已有 registration replay、bridge binding、state save/restore、hot reload、generation 和 access-plan 基础；这些应成为真实 plugin parity gate，而不是删除。
8. `zircon_export plugin validate/build` 已按 owner 拆分大量 manifest、distribution、asset、interface 和 package checks；build 可生成 loadable file manifest、SHA-256、外部签名/公证 audit。
9. CI 会检查 root/plugin workspace、manifest/catalog、全部插件 test、standalone validate 和逐 dist build；这比“只在主程序里能编译”强得多。
10. `plugin_sdk_examples` 提供 editor window/importer/inspector source 示例，native fixtures 覆盖 command、panic、state 和 editor contribution，为后续真正的兼容矩阵提供了种子。

## 4. 差距清单

### 4.1 P0：必须先封闭的内存安全、进程可用性与加载准入风险

#### P0-01 · `NativePluginStatic<T>` 对任意 `T` 无条件实现 `Sync`

`plugin_sdk/src/native.rs` 公开 `#[repr(transparent)] pub struct NativePluginStatic<T>(T)`、公开构造/`get()`，随后 `unsafe impl<T> Sync`。任何第三方都能构造 `static NativePluginStatic<Cell<_>>` 或其他 `!Sync` 类型，再经共享引用跨线程访问，形成 safe API 可触发的数据竞争/UB。不要给泛型 wrapper blanket unsafe impl；为每个只读 ABI carrier 提供私有 newtype 和逐类型 invariants，或让生成器产出具体 static holder。若 raw pointer 使 carrier 自动不为 `Sync`，证明应落在具体 ABI 类型而非任意 `T`。

#### P0-02 · owned byte carrier 可复制，owner token 可推导并允许 double free/forged free

`NativePluginOwnedByteBufferV2` 派生 `Clone + Copy`，暴露 pointer/len/capacity/token/free。`owned_bytes` `forget(Vec)` 后用公开常量 salt 与 pointer/len/capacity XOR 生成 token；free 重新计算后直接 `Vec::from_raw_parts`。复制 carrier 后两次 free 都会通过 token；foreign caller 也能推导 token、篡改 capacity 或伪造另一块内存。必须改为不可复制的 opaque allocation ID/registry 或 host allocator，release 只消费一次性 ID，foreign metadata 不能决定 Rust allocator 的 capacity。该问题与 [runtime interface owned buffer](../zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md) 同根，应共用一种所有权协议。

#### P0-03 · `bytes_from_slice<'a>` 可伪造任意生命周期并接受坏 carrier shape

公开 `unsafe fn bytes_from_slice<'a>(slice) -> &'a [u8]` 的输出生命周期与任何输入借用无关，调用方可将 callback-scoped pointer 绑定为 `'static` 并在回调后保留；null + nonzero length 又被静默返回空，非空路径未检查 `isize::MAX` 就 `from_raw_parts`。它虽标成 `unsafe`，却没有可满足、可审计的 safety contract，并被 fixture/SDK callback 直接复用。改为 callback scope token 绑定的 borrowed view，先统一验证 null/len/alignment/range/budget，再只在闭包作用域内暴露 slice；禁止返回自由泛型生命周期。

#### P0-04 · `on_host_ready` panic 会穿越 `extern "C"` 并中止宿主

`export_native_plugin_entry_v3!` 生成普通 `pub extern "C" fn`，直接调用 `NativePluginEntryPointV3::entry_report()`；后者直接调用第三方 `fn` 类型的 `on_host_ready`，没有 `catch_unwind`。fixture 已实际使用 `Some(emit_host_v3_*_signals)`，所以这不是死 API。任何 hook panic 都会穿越 non-unwind C ABI，Rust 通常直接 abort 整个 Editor/App。entry/descriptor/host-ready 与所有 callback 必须经过统一 no-unwind trampoline；panic 应形成 entry failure report，child-process gate 必须证明宿主不被终止。

#### P0-05 · Editor/App 在 selection 与信任校验之前执行目录内原生 DLL

`selected_native_editor_plugin_registration_reports()` 先调用 `load_discovered_native_editor_plugins(project_root/zircon_plugins)`，loader 对每个 editor candidate 先 `Library::new`、probe descriptor、调用 entry，返回后才用 `ProjectPluginManifest.enabled/target` 过滤 registration。`EditorManager::apply_project_plugin_manifest()` 和 App editor startup 都重复该顺序。因此未选择、disabled、依赖失败或仅被放入目录的 DLL 已经执行。manifest/selection 没有 hash、publisher、signature/trust pin；build 生成的 `.sig`/SHA sidecar也未被 loader读取。必须拆成纯数据 discovery -> schema/identity/dependency/selection/trust/signature admission -> child-process probe -> approved code load，disabled package 必须用测试证明零 code execution。

### 4.2 P1：公开或产品化前必须闭合的工程合同

#### P1-01 · 39 个 `native_dynamic` dist 只是 metadata shell

全部 39 个 dist entry 都是 `invoke_command: None`、空 bridge table、空 host-ready；38 个连 state callback 也没有。AI/Physics/Sound/Hybrid GI 等业务算法只在 source runtime crate 的 `runtime` feature 中，dist 通常只依赖 declaration projection。为每个 package advertising `native_dynamic` 定义行为能力表；没有真实执行路径的 package 必须先移除该 packaging 选项，不能把“DLL 可加载”显示成“插件功能可用”。

#### P1-02 · static/source/native 三形态没有行为等价合同

catalog/manifest parity 只比较 metadata。没有相同 project、scene、input、asset 和 workload 下 static、LibraryEmbed、SourceTemplate、NativeDynamic 的 system registration、输出、state、diagnostics 和 teardown equivalence。先选一个小而真实的 runtime plugin 做 golden parity；通过后再扩展，不允许批量复制 dist 模板。

#### P1-03 · source descriptor 与生成 manifest 的 SDK API version 已发生真实漂移

39 份生成 TOML 为 `sdk_api_version = 0.2.0`，而 `PluginPackageManifest::new()` 默认 `0.1.0`；`RuntimePluginDescriptor::package_manifest()` 没有覆盖它，SDK example test 也固定期待 `0.1.0`。first-party runtime catalog parity test 理论上应阻止该漂移，但当前代码事实说明至少部分 catalog/feature lane未把它闭合。version 必须由一个 generated constant 注入所有 source/native/file projection，禁止构造器藏旧默认值。

#### P1-04 · `sdk_api_version` 只检查格式，不参与兼容准入

`cargo-zircon validate` 和 runtime package validation 只确认三段数字；native loader只检查 distribution ABI与engine range。`99.0.0` SDK metadata 仍可能被接受。建立 SDK API compatibility table、minimum/maximum/feature epoch，并在 source registration、standalone validate、artifact probe 和 product loader 四处消费同一 policy。

#### P1-05 · package、SDK API、descriptor ABI、behavior ABI 与 schema epoch 缺少统一发布矩阵

当前同时存在 package 0.1、SDK 0.2、descriptor V3、behavior V4、entry report epoch 5、command schema V4、registration schema V3。版本各自合理不等于组合可兼容。发布物需要 machine-readable compatibility manifest，列出每个 interface family、schema、engine build set、target/data model 和 deprecation window。

#### P1-06 · 默认 Rust SDK 是完整 engine/editor 的锁步内部 API

`plugin_sdk` 默认 `runtime` 直接依赖 `zircon_runtime`；`editor` 再依赖完整 `zircon_editor`。这适合作为同仓 first-party source authoring helper，不是稳定第三方 SDK。明确拆成 public declaration/native contract crate 与 internal lockstep source integration crate；不要假装 full engine Rust types 跨 minor version 可稳定。

#### P1-07 · SDK 没有独立可发布 artifact、支持窗口和迁移记录

当前依赖全部通过 workspace/path 闭合，CI 没有 `cargo package`/detached consumer/published-crate dry run，也没有 SDK changelog、support window、compat fixture registry。需要可下载的 SDK bundle、license/header/schema/examples、版本迁移指南和 N-2 consumer gate。

#### P1-08 · SDK 与宿主手写两套 ABI，开放的 V2 hard-cut debt仍在

SDK `native.rs` 与 runtime `abi_declarations.rs` 分别维护相同 layout；host 仍有 V2 descriptor/report/host table/behavior/byte实体和 V3 alias，fixture仍输出 V2。开放 failure 已明确要求删除这些物理类型/alias。先完成 hard cut，再从单一 IDL生成两端；禁止通过继续复制 V5/V6 struct 维持一致。

#### P1-09 · 没有 generated C/C++ header、非 Rust consumer 或 ABI schema

`plugin_sdk` 下 C/C++/C#/Zig/Swift 文件数均为 0，没有 cbindgen/IDL/header generator。所谓 C ABI 只能被相同 Rust workspace 实际消费。至少生成 C11 header、C++ RAII wrapper、layout JSON、calling convention/export macro和一个独立 C consumer；Godot 的 generated interface 是更合适的参考。

#### P1-10 · ABI table/carrier 缺少 size、reserved、build identity与feature fingerprint

descriptor、entry report、host table、behavior、bridge table大多只带 version/epoch，没有 `struct_size`、reserved flags、target triple、pointer width、build id、SDK hash和capability schema fingerprint。相同数字但不同构建语义会被误接受，尾扩展又只能 hard cut。内部/公开 ABI 策略需显式分开并可测试。

#### P1-11 · 公共 ABI 使用 `usize`，却没有 data-model handshake

byte len/capacity、output max、bridge method count 使用 `usize`；32/64 bit、alignment 和 endian 不在握手或 snapshot 中。公共 contract 优先固定宽度并规定最大值，至少建立 x86_64/aarch64/32-bit layout compile gate。

#### P1-12 · C string 与 pointer table读取无长度和结构预算

capability list、plugin id、manifest、diagnostics、schema 和 entry names 依赖 `CStr::from_ptr`；SDK会在验证 table size 前解引用 host pointer并读取未受限终止字符串。改用 pointer+length/validated string view和 per-field cap；任意地址有效性无法在同进程内证明，malformed plugin必须在隔离 probe中验证。

#### P1-13 · registration manifest 的 SDK parser 与宿主 parser 严格度不一致

command manifest两层都 `deny_unknown_fields`，但 SDK 的 registration root/nested DTO 未启用；runtime host mirror却多处启用。作者可在 SDK round-trip成功后被宿主因未知字段拒绝。schema应单源生成两端 parser、JSON/TOML schema和negative corpus。

#### P1-14 · `plugin.toml` root/nested未知键可被 serde/validator静默忽略

`PluginPackageManifest` 和多数 nested manifest没有 `deny_unknown_fields`；`cargo-zircon validate`枚举并检查已知字段，却没有完整 allowed-key gate。`capabilites`、`supported_platfoms` 等 optional typo可能默认为空。对稳定 schema启用严格未知字段；预留扩展应使用显式 namespaced extension table，而非全局宽松解析。

#### P1-15 · package dependency没有版本、来源、artifact或interface version约束

dependency只有 id/required/capability/interfaces。39 份 manifest 的 8 条跨包依赖均无法表达 `VersionReq`、source registry/git/path、artifact digest、interface ABI/schema version或平台 variant。Runtime 07 已登记 solver缺口；本域必须先扩充可求解、可锁定的 package contract。

#### P1-16 · project selection没有 package version/source/hash lock

`ProjectPluginSelection` 只有 id/enabled/required/target/packaging/crate/features，且 `enabled` 默认 true。工程在另一台机器打开时无法确认选中的是同一 package/artifact。需要 project plugin lockfile，记录 resolved version/source/digest/signing identity/target artifact和dependency closure。

#### P1-17 · build hash/signature sidecar没有进入 runtime admission

`plugin_build_signature.py` 会记录 loadable artifact SHA 和外部 signing audit，但 runtime/editor/app 搜索不到对应 sidecar contract。将 package payload manifest、hash、platform signature/notarization result 和 loader admission绑定；验证必须发生在 `Library::new` 前，并防 TOCTOU。

#### P1-18 · export `native_plugins.toml` 与产品 loader是两套事实

export pipeline生成并严查 `native_plugins.toml`，产品 native loader却递归发现 `plugin.toml`并自行推导动态库文件名，不消费 staged loader manifest。统一 staged artifact index；发布时确定的路径、hash、package identity 和target mapping不能在运行时重新猜测。

#### P1-19 · native artifact validator只 probe descriptor/symbol，不调用 entry和behavior

`cargo-zircon` artifact validate会 `Library::new`、调用 descriptor、比较 embedded manifest/capabilities/entry symbol，但不执行 entry negotiation、解析 report/behavior/registration或验证callback。CI artifact check因此可让“descriptor正确、entry崩溃/空壳”的DLL通过。probe应在child process中覆盖完整admission并产出结构化receipt。

#### P1-20 · 插件发行CI只有Ubuntu host lane

39 个 standalone dist全部在 `ubuntu-latest` check/build；没有MSVC DLL、macOS dylib、aarch64、debug/release CRT、符号可见性或目标架构矩阵。native package至少要覆盖Windows/MSVC、Linux GNU、macOS arm64/x64和产品实际支持目标。

#### P1-21 · SDK feature powerset没有独立 consumer gate

workspace all-targets会受Cargo feature unification影响，不能证明 `declaration`、`native`、`editor_contribution`、`runtime`、`editor` 各自最小依赖能独立编译。为每个公开 feature建立 detached fixture和 `--no-default-features` matrix，禁止无意把完整 runtime/editor带回ABI SDK。

#### P1-22 · 现有dist测试只证明report形状，不证明插件业务存在

39 个dist合计79个test attributes，大多检查descriptor id、entry非空和registration manifest pointer。AI/Physics/Sound测试没有运行任何同名系统或算法。增加package-level executable scenario；若某形态不支持业务能力，状态必须是unsupported而非loaded/ready。

#### P1-23 · capability只是无版本字符串，缺少limits、owner和接口协商

capability通过NUL字符串列表与`host_has_capability`查询，不能表达version range、quantitative limits、dependency/conflict、permission provenance或denial reason。建立typed capability descriptor与limit table；协商结果必须绑定entry report、manifest和host policy。

#### P1-24 · capability negotiation不是加载权限边界

loader在调用entry之前已经执行DLL初始化代码，entry拒绝capability只能决定返回哪个report，不能保护宿主免受未授权代码。权限只对可调用host API生效；code execution trust必须在加载前独立完成，敏感/第三方插件还需进程或WASM/VM隔离。

#### P1-25 · 缺少install/update/uninstall/rollback与registry产品链

当前工具可scaffold、validate、build、package，但没有repository index、download、transactional install、atomic update、rollback、quarantine、revocation、license/publisher UI或磁盘清理。Hub/Editor后续应消费统一package service，不能直接让用户复制目录。

#### P1-26 · 所有第一方package共用0.1.0和宽泛engine range

成熟度从experimental到stable不同，功能和ABI变化频率也不同，却全部version 0.1.0并接受整个engine 0.1 minor range。建立package独立版本/compat policy与release automation；“stable”必须有兼容和退役承诺，不能只是UI标签。

#### P1-27 · 除glTF样例外没有真实state schema与迁移实现

38/39 dist无state callback；hot reload对复杂plugin的manager/resource/GPU handle/world registration无法迁移。每个可reload package需声明state schema、prepare/quiesce/save/restore/commit/rollback和不可迁移原因，默认应不可reload而非假设stateless。

#### P1-28 · module loading phase、explicit load、sealed/disallowed policy没有进入file contract

source runtime descriptor有init level/system stage，但`plugin.toml` module主要表达kind/target/capability，无法声明pre-default/post-engine/editor-only loading phase、explicit load、sealed package或disallowed dependency。参考Unreal把package policy与module phase分层，并映射到Core lifecycle，不要在host调用点硬编码。

#### P1-29 · 单一dist crate/filename推导不足以表达平台与配置artifact

distribution只有dist crate、symbol、entry和engine range；loader按当前OS拼接库名。没有arch、configuration、CRT、CPU/GPU feature、minimum OS、debug symbol、dependent library和per-platform hash映射。发行manifest必须精确选择artifact，不得靠目录碰撞和文件名猜测。

#### P1-30 · 进程内native插件没有崩溃/挂起/资源耗尽隔离等级

Runtime 07 已登记in-process native isolation与callback budget；本轮补充package/admission层证据：manifest没有trust tier/isolation mode/resource budget，dist默认即native_dynamic。定义 first-party trusted lockstep、signed native child process、WASM/ZrVM等tier；未知第三方不能与Editor主进程共享故障域。

### 4.3 P2：应在主合同闭合后收敛的质量与维护债

#### P2-01 · 默认command output上限256 MiB过大且不是协商值

`NATIVE_COMMAND_MAX_OUTPUT_BYTES_V4` 固定256 MiB，单次plugin command即可造成显著内存压力。按command/host/profile协商较小上限，并加入累计、并发和时间预算。

#### P2-02 · V2物理名、V3 alias、V4 behavior与epoch 5命名增加作者误用概率

当前类型名不能直接回答“这是当前还是退役合同”。hard cut后只保留唯一current物理类型，并在生成compat manifest中表达family version，避免源码alias成为事实兼容层。

#### P2-03 · `dist.rs` 853行与declaration macro 623行承担过多代码生成职责

大型`macro_rules!`复制runtime/editor/combined三套static table和report，错误常落在展开后内部符号。拆成小型typed builders/proc macro或IDL generator，提供span准确的compile-fail tests；不是为了减少行数，而是减少ABI真值复制。

#### P2-04 · standalone build文档已退化为约928 KiB的追加式历史台账

`docs/zircon_plugins/plugin-standalone-build.md` 约2,305行但接近1 MiB，混入大量会话状态、历史命令与未声明验收，难以作为作者手册。将稳定合同、操作指南、failure history和benchmark evidence拆开；历史记录归plan output，不进入主用户文档。

#### P2-05 · SDK examples偏Editor authoring，没有最小第三方runtime/native产品样例

现有example对window/importer/inspector较完整，但dist仍为空行为。新增不依赖仓内私有crate的detached C与Rust样例，包含真实runtime system/command、state migration、package build、签名、install和产品load。

#### P2-06 · “stable/beta/experimental”没有验收定义

8份manifest标stable、7份beta、24份experimental，但没有与API stability、测试矩阵、性能、平台支持、deprecation和security response绑定。将maturity变成可计算release gate，否则只是描述性字符串。

#### P2-07 · diagnostics多为自由文本，跨build/load/live-host缺少统一correlation

部分loader有typed internal error，最终report和Editor status大量压成`String`。定义stable diagnostic code、stage、package/artifact/build id、generation、cause chain和correlation id，UI再本地化展示。

#### P2-08 · 测试数量被source-shape与fixture覆盖放大

SDK/39 dist已有不少unit test，但安全边界仍没有blanket-Sync misuse、double-free child process、entry panic process survival、disabled-zero-execution、signed-admission、cross-version或real-plugin parity。coverage看板应按风险场景而非`#[test]`数量计分。

## 5. 结构收敛与责任边界

### 5.1 不新增另一套平行`IPlugin`

仓库现有Core `PluginDescriptor`、runtime `RuntimePlugin`、editor `EditorPlugin`、native entry/behavior和未来`EnginePlugin`路线代表不同生命周期/执行域。最小收敛不是把它们塞进一个巨型trait，而是建立一条authoritative package identity和四个明确adapter：source runtime、source editor、native ABI、VM/WASM。package/selection/diagnostic身份共享，生命周期和故障域保持分层。

### 5.2 public contract与internal lockstep API分开

- public declaration：无runtime/editor依赖的package、capability、interface、version schema；
- public native/VM ABI：generated C/Rust bindings、固定宽度carrier、host API、budget、state和lifecycle；
- internal source SDK：允许依赖完整runtime/editor，明确与engine build set锁步；
- host adapters：在selection/trust通过后，把public contract投影到Core module/system/editor extension；
- package service：负责resolve/lock/install/verify/update/rollback，不归loader临时拼路径。

### 5.3 与既有报告的所有权

- [Runtime 07](../zircon_runtime/07-script-plugin-runtime-review.md) 继续拥有generation authority、transactional publication、dependency solver、VM budget、native isolation、debugger和scene script lifecycle；本文不复制其实现计划。
- [Runtime Interface 01](../zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md) 拥有通用byte carrier、owned result、status、handle、build identity与FFI验证模式；plugin ABI应复用，不再发明第二套token/free。
- 本文拥有SDK author contract、manifest/package/version、dist artifact、catalog parity、build/signing到loader admission及static/native行为等价。
- 后续Editor plugin UX报告拥有browser、enable/disable、permission prompt、restart/reload交互；Hub报告拥有repository/install/update UI，但底层package service contract必须与本文一致。

## 6. 重构路线

### M0 · 先阻断不安全执行

1. 删除blanket `Sync`，为具体static ABI类型建立可证明holder。
2. 替换owned buffer协议和free，实现allocation registry/host allocator与一次性release。
3. 删除自由生命周期`bytes_from_slice`，统一validated callback-scoped view。
4. 给descriptor/entry/host-ready/callback全部加no-unwind trampoline和child-process crash gate。
5. 将discovery、selection、dependency、trust/hash/signature verification移到`Library::new`之前；临时产品策略应默认禁用未验证native plugin。

M0完成前，不得把新的第三方native plugin入口接入Editor/Hub，也不得把现有`native_dynamic`宣传为安全扩展机制。

### M1 · 单一合同与版本收敛

1. 完成V2实体/V3 alias hard cut，关闭两个open failure handoff。
2. 定义IDL/schema source，生成Rust host/SDK、C header、manifest schema、layout snapshot和docs。
3. 统一package/source/file/embedded manifest的SDK API version；loader真正执行compat policy。
4. 建立build set/data model/interface-family compatibility manifest。
5. 严格化unknown fields、dependency version/source/interface constraints和project lockfile。

### M2 · 一条真实插件发行闭环

1. 选择一个有实际system/command/state但规模可控的runtime plugin。
2. 实现source/library/native三形态行为等价和reload rollback。
3. 生成跨平台package payload、hash/signature、loader manifest与detached SDK consumer。
4. package service完成resolve/install/verify/activate/update/rollback/uninstall。
5. Editor/App只消费resolved lock和verified artifact，不再递归猜测目录。

### M3 · 扩展到第一方catalog与第三方生态

1. 每个package按能力决定支持的packaging；无动态实现的先移除`native_dynamic`。
2. Windows/Linux/macOS/arch/configuration矩阵验证真实DLL和产品load。
3. N-2 engine/plugin、old/new SDK/schema skew与deprecation矩阵。
4. trusted in-process、signed child-process、VM/WASM tier及资源预算。
5. registry/publisher/revocation/license/security response和长期telemetry。

## 7. 验收门

1. `NativePluginStatic<Cell<_>>` 等compile-fail；SDK public safe API不能构造跨线程data race。
2. copied/forged/malformed owned carrier在child process、ASan/Miri/fuzz下不double free、不按foreign capacity重建allocator object。
3. borrowed bytes的null+nonzero、`isize::MAX`、dangling/expired scope全部typed reject；API无法返回任意`'static`。
4. descriptor、entry、host-ready和每类callback panic均不终止宿主，返回结构化failure。
5. disabled/unselected/incompatible/untrusted插件的DLL initializer与entry计数始终为0。
6. artifact在verification后被篡改、sidecar缺失、签名撤销或hash不符时，`Library::new`前失败。
7. generated Rust/C header在MSVC/Clang/GCC、x64/arm64和至少一个32-bit layout lane一致。
8. SDK API、descriptor/behavior/schema/build set不兼容组合全部有matrix expected result，未知组合fail closed。
9. 39份file manifest与source/embedded/catalog projection逐字段一致，包括version、SDK API、dependency和distribution。
10. unknown root/nested field、typo、duplicate和unsupported extension全部给出stable diagnostic code。
11. project lock在离线另一台机器解析到相同package closure和artifact hash；未锁变体不能静默替换。
12. chosen pilot plugin在static/source/native三形态运行同一scenario，system/output/state/diagnostic/teardown golden一致。
13. `cargo-zircon` artifact probe在child process调用完整entry/report/behavior，不只查symbol。
14. CI真实构建并加载Windows DLL、Linux SO和macOS dylib；覆盖debug/release及依赖库缺失/错误架构。
15. hot reload在save、unload、new load、restore、commit任一步失败时可回滚，旧generation不再被调用。
16. package install/update/uninstall断电/崩溃注入后目录和lock保持原子一致，可恢复到上一版本。
17. capability version/limit/permission denial在host和plugin两端一致，未授权host API不可调用。
18. crash/hang/OOM plugin按isolation tier被终止或熔断，Editor主进程和project data保持可用。

## 8. 禁止的临时方案

- 禁止继续复制空`dist/src/lib.rs`并仅增加descriptor/report测试。
- 禁止通过注释为blanket unsafe impl、自由生命周期slice或token free“补安全说明”。
- 禁止先`Library::new`再弹权限/签名提示；DLL initializer已经执行。
- 禁止把build sidecar存在等同于runtime已验证，或把外部signer返回0等同于可信publisher。
- 禁止用`engine_compat`宽泛range替代SDK/ABI/schema/build identity矩阵。
- 禁止保留V2实体/V3 alias作为“以后兼容”，现有计划已要求hard cut。
- 禁止以Rust workspace同时编译通过替代C ABI稳定性、detached consumer与跨compiler验证。
- 禁止让Hub、Editor和runtime各自实现一套package resolve/install/trust真值。
- 禁止将first-party trusted plugin策略默认套到未知第三方package。
- 禁止在没有真实行为等价gate时把`native_dynamic`标记为stable或完整。

## 9. 本轮状态

本报告完成plugin SDK/package/catalog/dist/native admission纵向首轮静态深审，实施尚未开始。结构审计clean、manifest生成、standalone build/signing和现有CI均被登记为可保留基础；它们不抵消5个P0 soundness/admission风险，也不证明39个dynamic package具有业务行为。

下一轮可转向`zircon_editor`插件管理与authoring workflow，或继续审`zircon_runtime_interface`其余公共DTO；进入任何native plugin实现修复前，必须先重取本文focused fingerprints并确认两个Runtime06 open failure的current source状态。
