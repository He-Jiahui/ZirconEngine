---
title: Runtime Plugin Profile、Manifest、Catalog、Provider Resolution 与 App/Export/Native Closure 当前工作树复审
category: zircon_runtime
report_id: Runtime199
review_date: 2026-08-31
baseline_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
verification_head: 14c89f9776bed828cc85e05e4b9914b3f8d1e784
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/99zk-runtime-builtin-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-current-source-review.md
related_editor_owner:
  - docs/plans/optimize/zircon_editor/259-editor-plugin-provider-catalog-current-working-tree-review.md
related_plugin_owner:
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
related_code:
  - zircon_runtime/runtime-feature-presets.toml
  - zircon_runtime/build.rs
  - zircon_runtime/src/plugin/runtime_profile
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog
  - zircon_runtime/src/core/framework/project/project_plugin_manifest
  - zircon_runtime/src/builtin/runtime_modules/manifest.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly
  - zircon_runtime/src/plugin/export_build_plan
  - zircon_plugins/first_party_runtime_catalog/Cargo.toml
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/product_host_config
  - zircon_app/Cargo.toml
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/PluginManager.h
  - dev/bevy/crates/bevy_app/src/plugin_group.rs
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/Fyrox/fyrox-core/src/plugin.rs
  - dev/godot/core/extension/gdextension_manager.cpp
  - dev/godot/core/config/project_settings.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/package.json
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime199 · Profile / Catalog / Provider Closure 当前工作树差距

## 1. 结论

当前 Zircon 的插件选择链已经有不少可复用的 typed substrate：`ProjectPluginManifest` 能表达 target、required、packaging、runtime/editor crate 和 feature；`RuntimePluginCatalog` 能生成 immutable project plan；`RuntimeModuleCompositionCompiler` 能在 fatal diagnostic 时拒绝 composition；profile preset build script 能校验 schema、顺序、模块与重复项；export build plan 也会保留 source selection 和 provider feature。问题在于这些能力没有收敛为一个“声明 -> 编译 provider -> runtime/editor registration -> module/artifact -> lifecycle”唯一 authority。

当前结构审计显示 39 个 plugin manifest，30 个 runtime package id、25 个 editor package id；`first_party_runtime_catalog` 只对 15/30 个 runtime 声明提供编译 registration（约 50%），`first_party_editor_catalog` 只对 2/25 个 editor 声明提供 provider（约 8%）。这不是“可选插件尚未打开”，而是 manifest 已声明的 package、Cargo feature、Rust provider 和产品目标之间没有可证明的闭包。

最危险的默认路径是 profile：`runtime-feature-presets.toml` 中 Client2D/Client3D/Editor/Dev 的 required plugins 包含 `ui`、`sound`、`rendering`，但 Client profile 的 `app_features` 没有 `first-party-runtime-plugins`，Editor/Dev 只有 advanced-render、navigation 和 editor provider feature，没有 base runtime catalog。`zircon_app/Cargo.toml:115-156` 的 target feature 组合因此无法从 profile 声明推出可链接 provider。反过来，`EntryConfig::new(Editor/Runtime)` 不附带 `RuntimeProfileDescriptor`，而 `EntryConfig::for_runtime_profile` 才附带 profile manifest；同一个产品角色存在两套不同的 plugin closure。

当前 first-party catalog 还把 invalid id、未知 id、未编译 provider 统一变成 `continue`/空结果：`zircon_plugins/first_party_runtime_catalog/src/lib.rs:13-31` 和 `first_party_editor_catalog/src/catalog.rs:10-32` 返回 bare `Vec`，没有 missing/unknown/feature-blocked/duplicate receipt。下游 plan 虽能发现部分 feature provider 缺失，但无法区分“用户未选择”“声明存在但 provider 未链接”“catalog 自己漏路由”。这会让 required plugin 在入口处消失，直到更晚的 module availability 或 capability 阶段才以不完整信息失败，optional plugin 则可能静默丢失。

此外，`RuntimeProfileDescriptor::availability_report` 使用 `require_external_provider = false`（`availability.rs:18-30`），在没有实际 linked/native/builtin provider 时仍可给成熟 descriptor `Available`；只有 provider-aware 调用才检查 `builtin_runtime_domain_is_available`，而该函数当前只把 `Ui` 当 builtin（`evaluation.rs:248-275`）。可见状态与真实 App composition 因调用入口不同而不一致。export 生成的 provider 列表、App 手写 cfg catalog、native dynamic discovery 也没有共享 receipt、catalog generation 或 source digest，无法证明运行时和发行包装使用的是同一组 provider。

因此本轮登记 6 项 canonical P0（全部 Open），34 项 P1（31 Open / 3 Partial / 0 Closed），12 项 P2（12 Open），32 道资格门（29 Fail / 3 Partial / 0 Pass）。这些 finding 只评价 provider closure，不把已有的 module graph、feature dependency、manifest schema 或 cache substrate误判为完成；Tooling 继续按用户要求排除。

## 2. 物理扫描范围与证据

### 2.1 Runtime 选择集

本轮逐文件读取 profile/build/catalog/manifest/assembly/export 相关 production、tests、Cargo、build script 与 App 生产调用点：189 个去重文件、22,298 行、20,328 非空行、846,912 bytes、195 个测试声明。选择集包含：

- `zircon_runtime/build.rs`、`runtime-feature-presets.toml`；
- `src/plugin/runtime_profile/**` 全部 profile、availability、generation、selection、projection；
- `src/plugin/runtime_plugin/runtime_plugin_catalog/**` 全部 projection、feature resolution、project plan、selection、extension report、cache；
- `src/core/framework/project/project_plugin_manifest/**` 全部 selection、hydration、defaults、validation；
- `src/builtin/runtime_modules/assembly/**` 和 `load_report/**`；
- `src/plugin/export_build_plan/**`；
- App `entry` 中 runtime catalog wrapper、product host resolution、builtin module composition、engine entry 与 target Cargo features。

静态审计 `python tools/audit_plugin_structure.py --json --repo-root .` 当前通过既有 manifest/dist 结构门：39/39 manifest、29 个 runtime descriptor root、41/41 dist build matrix；但只投影 2 个 editor provider package，脚本没有 catalog coverage 或 profile closure 指标。对 39 个 `plugin.toml` 的声明扫描得到 runtime 30、editor 25；runtime catalog 当前路由 15 个，editor catalog 当前路由 2 个。该审计没有把“manifest 存在”当作“provider 可执行”。

### 2.2 关键源码路径

- `first_party_runtime_catalog/src/lib.rs:13-31` 解析目标 manifest、去重后查找 provider；20-27 的 parse/lookup 失败直接跳过，34-100 是手写 cfg/ID 分支，只有 15 个 provider。
- `first_party_editor_catalog/src/catalog.rs:10-32` 只接受 `EditorHost`，同样静默跳过 unknown/uncompiled provider；41-54 只有 Navigation 与 Neural cfg 分支。
- `first_party_editor_catalog/src/tests.rs:19-30` 的 source assertion 仍要求 `registrations.push(registration);`，而当前 production projection 只有 `registrations.push(provider());`，测试与实现已漂移，执行测试时会失败。
- `zircon_app/src/entry/first_party_runtime_plugins.rs:16-45` 分别处理 resolved config、render overlay 和 runtime profile；这些入口对 effective manifest 的构造不同。
- `zircon_app/src/entry/builtin_modules.rs:42-98,134-200` 先从 registration reports 构造 catalog/plan，再编译 module composition；render profile 只额外注入 VirtualGeometry/HybridGi/Solari。
- `zircon_app/src/entry/engine_entry.rs:252-279` 的 `for_profile` 使用 `EntryConfig::new`，而非 `for_runtime_profile`；profile 默认 plugin 与 role 默认配置因此不是同一条路径。
- `zircon_app/src/entry/product_host_config/resolution.rs:298-370` 仅在 runtime profile/request/required/optional 任一项存在时生成 manifest；无 profile 的 role 返回 `None`，required/optional 只修改选择，不验证 provider closure。
- `zircon_runtime/src/builtin/runtime_modules/manifest.rs:8-57` 的 mode baseline 只注入 UI/UiDocumentImporter；不会注入 profile 中 required Sound/Rendering。
- `zircon_runtime/src/plugin/runtime_profile/availability.rs:18-44` 暴露 provider-agnostic 与 provider-aware 两套报告；`availability_projection/evaluation.rs:248-275` builtin provider 判断只识别 Ui。
- `runtime_plugin_catalog/project/selection.rs:69-126` 只选择有 registration 的 enabled id；enabled id 无 registration 会被 `retain` 丢弃，未形成 unresolved selection receipt。
- `runtime_plugin_catalog/project.rs:173-257` cache 以 catalog generation、manifest fingerprint、target 复用 plan，但没有保存 provider declaration matrix、cargo feature provenance 或 unresolved set。
- `zircon_runtime/build.rs:127-264` 验证 schema、profile/module/plugin key 与重复项，却没有读取 workspace Cargo feature、manifest provider crate、cfg 路由或 required closure。

## 3. 当前装配流程与断点

```text
profile TOML
  -> build.rs 形状校验/生成 descriptor
  -> EntryConfig / product host resolution
  -> effective manifest + mode baseline + render overlay
  -> 手写 first-party cfg catalog (bare Vec)
  -> RuntimePluginCatalog / compiled project plan
  -> feature dependency + module composition
  -> native/export provider paths (另有一套 authority)
```

这条流程缺少四个必须显式存在的中间产物：

1. **Declaration matrix**：每个 manifest id 的 runtime/editor/native crate、Cargo feature、target、maturity、ABI、module/capability 和 provider symbol 的生成映射。
2. **Resolution receipt**：每个 selection 必须记录 selected/disabled/unknown/feature-blocked/uncompiled/linked/native/duplicate 结果及 required 语义。
3. **Provider closure**：profile、App、editor、export、native 五个消费者必须消费同一个 catalog generation 和 provider set。
4. **Lifecycle ownership**：registration report 只是静态描述，仍缺 initialize/activate/ready/deactivate/unload 的 provider-specific evidence 与 shutdown result。

## 4. P0 阻断项

| ID | 阻断 | 当前证据 | 必须重构 |
|---|---|---|---|
| RT-PC-01 | Profile required plugins 与 Cargo/App provider closure 不可证明 | presets required `sound/rendering`；Client 没有 base catalog feature，Editor/Dev 没有 base runtime catalog（`runtime-feature-presets.toml`、`zircon_app/Cargo.toml:115-156`） | 生成 profile-to-feature/provider closure；required provider 缺失在 build/resolve 阶段 fail-closed，不能进入 composition |
| RT-PC-02 | Runtime catalog 只覆盖 15/30 声明 | `first_party_runtime_catalog/src/lib.rs:34-100` 手写 15 个 cfg branch；manifest scan 有 30 runtime ids | 从 manifest/Cargo 元数据生成 declaration/provider matrix；每个 declared id 必须是 linked、native、externalized 或有结构化 missing reason |
| RT-PC-03 | Required/unknown/uncompiled selection 在入口静默消失 | runtime `continue`（13-27）；editor 同形；compiled selection 69-118 只保留有 registration 的 id | catalog API 返回 `ProviderResolutionReport`/fatal diagnostics；required unknown、feature blocked、provider absent 永远不可被 `Vec` 丢弃 |
| RT-PC-04 | Availability 可在无 provider 时报告 Available | `availability.rs:18-30` 传 false；`evaluation.rs:248-275` builtin 只识别 Ui | 统一 provider-qualified availability；任何 UI、profile、editor、export 状态均来自同一 receipt，不允许 optimistic Available |
| RT-PC-05 | App、export、native、editor 使用多套 provider authority | App 手写 cfg catalog；export 生成 provider selection；native discovery 另有 package/ABI 路径；无共同 generation/digest | 建立 `ProviderCatalogSnapshot`，包含 declaration hash、Cargo/build feature、registration ABI、native artifact、source revision；所有消费者校验相同 snapshot |
| RT-PC-06 | Profile role 默认入口不一致，导致同角色不同 plugin set | `engine_entry.rs:253-255` `EntryConfig::new(profile)`；261-263 才走 runtime profile；resolution 314-327 无 profile 时返回 None | 合并 `for_profile`/`for_runtime_profile` 为单一 profile resolution；角色、profile、mode baseline、render overlay 必须生成一份可审计 manifest |

## 5. P1 重构账本

| ID | 差距 | 目标验收 |
|---|---|---|
| RT-PC-07 | build.rs 不读取 Cargo feature | 生成时解析 workspace metadata；未知/拼写错误 feature 阻断构建 |
| RT-PC-08 | build.rs 不验证 provider crate/symbol | 每个 runtime/editor/native declaration 都有 crate、feature、symbol、ABI version 检查 |
| RT-PC-09 | 手写 cfg 分支重复 ID 字符串 | 由单一 manifest metadata 生成 Rust lookup，禁止第二份手写路由 |
| RT-PC-10 | canonical ID 与 `key()`/字符串比较混用 | 所有路径使用 typed ID、canonical alias table 和 redirect migration；未知 alias 有诊断 |
| RT-PC-11 | registration report 无 provider provenance | 报告携带 package id、crate version、feature set、source digest、ABI/build id、linkage |
| RT-PC-12 | target filtering 在多层重复 | 统一 target/platform predicate，并把拒绝原因记录为 TargetUnsupported，而不是空 Vec |
| RT-PC-13 | required/optional merge 只 OR bool | 合并保留来源、顺序、priority、owner、conflict；重复 required 只生成一条 receipt |
| RT-PC-14 | completed manifest 自动补 disabled catalog rows | 区分 user declaration、catalog default、hydrated metadata；不得用 disabled row 隐藏漏声明 |
| RT-PC-15 | feature provider 与 base provider 状态分离 | feature/base/native/external provider closure 有统一 graph、cycle 和 missing diagnostics |
| RT-PC-16 | runtime profile optional plugin 没有 readiness contract | optional 记录 Available/Deferred/Blocked，并明确是否允许降级；禁止无原因消失 |
| RT-PC-17 | profile required capability 未绑定 provider | capability -> module/provider owner 反向索引；required capability 缺失直接 fatal |
| RT-PC-18 | render overlay 只覆盖三种 advanced renderer | 所有 RenderProductFeature 都由 metadata 映射 plugin/capability/feature，overlay 可解释且可测试 |
| RT-PC-19 | UI baseline 与 importer baseline 由 cfg 隐式注入 | baseline 作为版本化 profile declaration；Ui 与 UiDocumentImporter 的 required/optional 语义来自 manifest |
| RT-PC-20 | editor provider catalog 不能消费 runtime snapshot | Editor registration 绑定同一 package declaration、runtime dependency、target、generation |
| RT-PC-21 | export build plan 未复用 App resolution receipt | export 与 App 使用同一 completed manifest、provider set、diagnostic codes、source digest |
| RT-PC-22 | native dynamic discovery 无 selection receipt | native candidate 必须声明 artifact identity、ABI/build id、platform、trust、load state，并回写 resolution |
| RT-PC-23 | linked/native/builtin 重复装配没有唯一 owner | catalog 先决定 owner/linkage，再由 module composition 只消费决定结果；重复 provider 变成 fatal/explicit override |
| RT-PC-24 | plan cache key 缺 Cargo/provider provenance | cache key 纳入 catalog generation、manifest fingerprint、target、feature closure、artifact/build digest |
| RT-PC-25 | catalog generation 没有跨消费者 pin | App/editor/export/native 在一个 generation 上 pin；reload/feature change 原子发布新 generation |
| RT-PC-26 | registration 只有静态 descriptor | provider lifecycle 记录 initialize/ready/deactivate/unload、failure action、drain deadline |
| RT-PC-27 | module composition diagnostics 不能回指 selection 来源 | 每个 ModuleProposal 携 selection id、provider package、feature、source location、receipt id |
| RT-PC-28 | availability categories 与 composition result 可能漂移 | composition 完成后回写 qualified availability；未 compose 的报告不能显示 runtime Available |
| RT-PC-29 | role target 与 profile target mode 可产生隐式冲突 | resolution 统一 target mode/role check，错误包含 source、expected、actual、selection |
| RT-PC-30 | server profile 没有明确 externalized policy | headless/server 对每个 plugin 记录 no-provider、externalized、server-safe 或 blocked；禁止默认空 manifest 掩盖需求 |
| RT-PC-31 | maturity 检查先于 provider 检查但无诊断聚合 | 将 target/maturity/linkage/provider/ABI 诊断合并为稳定 code、severity、action、source chain |
| RT-PC-32 | feature cfg 组合缺少矩阵测试 | 生成 Client2D/3D/Editor/Dev/Server × target × base/advanced/editor/native 矩阵，required closure 全部正反例覆盖 |
| RT-PC-33 | provider order 只在 selected registration 上排序 | 由 declaration order + dependency graph 生成 deterministic order；missing provider 不改变其他节点静默顺序 |
| RT-PC-34 | no transactional publication across App startup | resolution、catalog、plan、composition、lifecycle 采用 prepare/commit/rollback，失败不留下半装配 catalog |
| RT-PC-35 | runtime/editor package versions 可漂移 | package manifest、runtime/editor crate、artifact ABI version 统一 compatibility range 与 lockfile evidence |
| RT-PC-36 | source manifest fingerprint 不含 generated metadata | fingerprint 包含 normalized selections、hydrated metadata、catalog declaration hash、feature set |
| RT-PC-37 | no provider admission security/trust boundary | native/external provider 经过 signature/trust/capability policy；不可信 provider 不能被 profile 标为 Available |
| RT-PC-38 | diagnostics 只在 late composition 才可见 | resolve boundary输出 machine-readable report，可由 Editor/CLI/export 共同展示与存档 |
| RT-PC-39 | no migration for old catalog IDs/features | alias/redirect/schema migration 生成 deterministic diff，旧 id 不被 `parse_key` 失败后静默抛弃 |
| RT-PC-40 | test source drift 已存在 | 修复 editor catalog source assertion并增加 runtime/editor generated matrix tests；测试必须锁定声明覆盖率与失败原因 |

## 6. P2 产品化与性能

P2 共 12 项：1) provider closure dashboard；2) profile diff viewer；3) startup trace waterfall；4) catalog cache hit/miss telemetry；5) declaration-to-artifact provenance export；6) feature flag audit；7) plugin load cost budget；8) parallel provider discovery；9) deterministic order visualization；10) hot reload impact preview；11) offline package validation；12) release compatibility report。它们必须读取 P1 的 receipt/snapshot，不得从静态 manifest 数量推断健康度。

## 7. 资格门

| Gate | 当前 | 必须证明 |
|---|---|---|
| G1 declaration coverage | Fail | 39 manifest 的 runtime/editor/native 声明都有唯一 provider/absence reason |
| G2 profile closure | Fail | 每个 built-in profile 的 required plugin/capability 在目标 feature 下可解析 |
| G3 no silent drop | Fail | unknown/duplicate/uncompiled/target blocked selection 都有 receipt，required 为 fatal |
| G4 provider-qualified availability | Fail | Available 只能由 linked/native/builtin/externalized policy 证明 |
| G5 App/editor/export parity | Fail | 三条路径消费相同 manifest/provider snapshot 与 hash |
| G6 role/profile identity | Fail | `for_profile` 与 `for_runtime_profile` 结果一致且可审计 |
| G7 generated catalog | Fail | 无手写 ID/cfg 漏路由，生成物可重建并有 source digest |
| G8 feature validation | Fail | Cargo feature、provider crate、registration symbol 全部 build-time 校验 |
| G9 target/platform | Partial | target predicate 已存在，但多层重复且拒绝原因未统一 |
| G10 required capability | Fail | capability 缺 owner/provider 时在 resolution 阶段失败 |
| G11 linkage ownership | Fail | builtin/linked/native/externalized 唯一 owner 与重复策略可证明 |
| G12 cache correctness | Partial | plan cache 有 generation/fingerprint/target，缺 provider feature/artifact provenance |
| G13 lifecycle | Fail | provider initialize/ready/deactivate/unload 有 receipt 和 drain |
| G14 ABI/trust | Fail | dynamic provider ABI/build/trust 与 selection 锁定 |
| G15 deterministic order | Partial | selected registrations 可排序，但 unresolved set 影响不可见 |
| G16 transactional startup | Fail | 失败不发布半完成 catalog/module composition |
| G17 matrix tests | Fail | profile/target/feature/native/editor 正反例矩阵在 CI 可复算 |
| G18 source migration | Fail | alias/redirect/schema upgrade 不丢 selection |
| G19 observability | Fail | startup、provider、diagnostic、cache、load cost 可查询 |
| G20 scale | Fail | 1,000+ provider declarations、冷启动与热 reload 有 P95/P99 budget |
| G21 release | Fail | export/native package closure 与 runtime/editor snapshot 一致 |
| G22 recovery | Fail | provider load/ready failure 有 rollback、last-good 与 retry policy |
| G23 security | Fail | untrusted external provider 在 admission 前被拒绝 |
| G24 test source integrity | Fail | source assertions 与 production implementation 同步，不能以字符串测试伪造覆盖 |
| G25 user-facing status | Fail | Editor/CLI 显示真实 resolution receipt，不显示静态 Available |
| G26 server/headless | Fail | Server/Headless profile 明确 provider policy，不以空 manifest 隐藏需求 |
| G27 plugin ABI compatibility | Fail | runtime/editor crate 与 artifact ABI version range 有正反例 |
| G28 generation invalidation | Fail | feature/catalog/artifact 变化原子失效所有旧 consumer |
| G29 reproducibility | Fail | 同一 lockfile/source/profile 产生相同 provider set/order/hash |
| G30 fault injection | Fail | provider missing/duplicate/panic/timeout/native unload 可验证 |
| G31 cold startup | Fail | resolution/registration/composition 不因重复 manifest parse 或 full clone 超预算 |
| G32 release audit | Fail | 发行前生成可归档的 declaration/provider/artifact provenance bundle |

## 8. 重构顺序

第一阶段建立 `ProviderDeclarationMatrix`、typed `ProviderResolutionReceipt`、`ProviderCatalogSnapshot` 和统一 diagnostic codes；同时把 `for_profile` 与 `for_runtime_profile` 合并成一个 profile resolution，先关闭 RT-PC-01/03/04/06。第二阶段由 workspace metadata 生成 runtime/editor/native catalog，删除手写 cfg/ID 路由，加入 profile/target/Cargo feature closure build gate。第三阶段让 App runtime、Editor catalog、export build plan、native discovery 全部消费同一 snapshot，并把 linked/native/externalized ownership 与 lifecycle receipt 接到 composition。第四阶段补齐 generated matrix tests、fault/scale/reproducibility、cache invalidation、migration、trust 与 release provenance；最后才扩展更多 first-party providers。任何兼容层都必须保留 unresolved receipt，不能恢复 `continue` 静默语义。

本轮仅完成 Runtime199 review/index/coverage 文档，没有修改 runtime、editor、plugin、Cargo、ABI 或生成代码，也没有运行 Cargo、Editor、动态 DLL、native load、fault、scale、soak 或 benchmark。Tooling 按用户要求排除；按用户要求未查询、轮询、等待或实时跟踪协调器。工作树中已有的 UI importer、catalog provider 函数和 Windows 依赖修改均保留，报告评价的是它们合并后的当前边界。
