---
title: Editor Plugin Manifest、Provider Catalog、Authoring Contribution 与 App Admission Closure 当前工作树复审
category: zircon_editor
report_id: Editor259
review_date: 2026-08-31
baseline_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
verification_head: 14c89f9776bed828cc85e05e4b9914b3f8d1e784
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/199-runtime-plugin-profile-catalog-provider-resolution-current-working-tree-review.md
related_plugin_owner:
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
related_code:
  - zircon_plugins/*/plugin.toml
  - zircon_plugins/first_party_editor_catalog/Cargo.toml
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
  - zircon_plugins/first_party_editor_catalog/src/tests.rs
  - zircon_app/src/entry/first_party_editor_plugins.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_app/src/entry/product_host_config
  - zircon_editor/src/core/plugin
  - zircon_plugins/*/editor/src/plugin.rs
  - zircon_plugins/*/editor/**/*.zui
  - zircon_editor/assets/ui/editor/components/workbench
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorEngine.cpp
  - dev/UnrealEngine/Engine/Source/Developer/AssetTools/Private/AssetTools.cpp
  - dev/bevy/crates/bevy_app/src/plugin_group.rs
  - dev/Fyrox/editor/src/plugins
  - dev/godot/editor/plugins
  - dev/godot/core/extension/gdextension_manager.cpp
  - dev/Graphics/Packages/com.unity.shadergraph/Editor
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor259 · Plugin Provider / Authoring Admission 当前工作树差距

## 1. 结论

Editor 侧已经出现了真实的 authoring 代码，而不是所有插件都是空壳：AI、Animation Graph、Physics、Sound、Terrain、Tilemap、Timeline、UI Asset Authoring、Material、Neural、Navigation、Hybrid GI、Virtual Geometry 等 entry file 含有 command、asset importer/type、toolkit、graph palette、inspector customization、debug DTO 或 ZUI extension。尤其 `zircon_plugins/ai/editor/src/plugin.rs`、`animation_graph/editor/src/plugin.rs`、`physics/editor/src/plugin.rs`、`sound/editor/src/plugin.rs` 和 `timeline_sequence/editor/src/plugin.rs` 证明局部 authoring contribution 可继续复用。

但这些贡献没有被统一的 Editor provider admission 消费。当前 39 个 plugin manifest 中有 25 个 editor package id，扫描到 40 个 editor entry 文件（含 descriptor-only 和真实 contribution），而 `zircon_plugins/first_party_editor_catalog/src/catalog.rs:41-54` 仅编译 Navigation 与 Neural 两个 provider，实际 provider coverage 只有 2/25（约 8%）。其余插件即使存在 `plugin_registration`、package manifest 或 `.zui`，也不会从 `ResolvedProductHostConfig` 进入 Editor Host。Editor 目前不是“模块尚未全部完善”，而是 manifest、provider crate、catalog、App feature、authoring extension、runtime dependency 和 lifecycle 之间缺一个可拒绝、可审计的 admission contract。

入口还会静默丢失选择：`first_party_editor_catalog/src/catalog.rs:20-30` 对 parse 失败、未编译 provider、未知 provider 直接 `continue`，返回 bare `Vec<EditorPluginRegistrationReport>`；`zircon_app/src/entry/first_party_editor_plugins.rs:10-36` 只是把 resolved manifest 转成这条 Vec，没有 missing/blocked/duplicate receipt。当前 `first_party_editor_catalog/src/tests.rs:19-30` 的 source assertion 仍要求 projection 中存在 `registrations.push(registration);`，但生产代码已经改成 provider function pointer 后只调用 `registrations.push(provider());`，测试与实现已漂移，执行该测试会失败。这种 source-test drift 本身足以阻止把 catalog 当作工程级基础设施。

Editor Host 的 Cargo feature 也没有形成闭包：`zircon_app/Cargo.toml:136-156` 开启 advanced runtime、navigation runtime/editor、neural editor，却没有 `first-party-editor-catalog` 或 base runtime catalog 的统一 feature；`first_party_editor_plugins.rs` 因此可能只得到两个 editor provider，而 Runtime/Editor package manifest 中的 Sound、Rendering、Physics、AI 等 editor 面仍不可达。`entry_runner/editor.rs` 的变量名 `entry_config` 实际承载 `ResolvedProductHostConfig`，表明入口层仍把配置解析、provider admission 和 host composition 混在一起，难以审计 provider generation 与 shutdown ownership。

因此本轮登记 5 项 canonical P0（全部 Open），34 项 P1（30 Open / 4 Partial / 0 Closed），10 项 P2（10 Open），28 道资格门（25 Fail / 3 Partial / 0 Pass）。这些 finding 只覆盖 Editor provider/catalog/authoring admission，不把已有 Workbench UI substrate、单插件 authoring batch 或通用 EditorJobSystem 误报为插件产品闭环；Tooling 按用户要求排除。

## 2. 物理扫描范围与证据

### 2.1 Editor 选择集

本轮读取：

- `zircon_plugins/first_party_editor_catalog` 全部 source/Cargo/tests；
- `zircon_app/src/entry/first_party_editor_plugins.rs`、`entry_runner/editor.rs`、`product_host_config/**`、`builtin_modules.rs` 相关 Editor 生产调用点；
- 全部 `zircon_plugins/**/editor/src/**/*.rs` 与 editor `.zui`；
- `zircon_editor/src/core/plugin`、Workbench module/navigation/command bridge、Editor package manifest 相关测试；
- 全部 39 个 `plugin.toml` 的 editor/runtime/native package rows；
- Unreal/Godot/Bevy/Fyrox/Unity Graphics editor/plugin lifecycle 参考切片。

去重选择集为 259 个文件、24,685 行、22,511 非空行、893,108 bytes、176 个测试声明，其中 228 个 Rust editor 文件、30 个 `.zui` 文件。39 个 manifest 的 editor package id 为 25；当前 catalog provider 为 Navigation、Neural 两项。结构审计的 41/41 dist matrix 只证明发布结构行存在，不能证明 editor provider 已链接或可初始化。

### 2.2 真实 contribution 与 descriptor-only 边界

- AI editor（218 行）包含 import/open/validate/compile command、asset importer/type/toolkit、graph editor/palette、overlay；未进入 first-party catalog。
- Animation Graph editor（279 行）包含 graph/state authoring、palette、extension batch 与 `plugin_registration`；未进入 catalog。
- Physics editor（186 行）包含 authoring/debug/ragdoll command、template、asset/toolkit；运行时 editor feature 需要显式 provider admission。
- Sound editor（111 行）包含 mixer/acoustic authoring、binding、inspector customization 与 `plugin_registration`；当前 catalog 不提供它。
- Hybrid GI、Virtual Geometry、Neural、Navigation 有 extension/provider；其中只有 Neural/Navigation 由 catalog 路由。
- Rendering editor（44 行）主要是 descriptor、package manifest、capabilities，没有 `plugin_registration`；这不是“缺一行 cfg”可以修复的 authoring 产品，而是需要明确 descriptor-only policy、runtime-only policy 或实现真实 extension。
- Texture、Particles、Net、Terrain、Timeline、UI Asset Authoring、Prefab、Material、Native Window Hosting、Runtime Diagnostics 等均有不同程度 entry/contribution，但 manifest、editor crate、catalog 路由和 lifecycle status 未统一。

### 2.3 入口与测试证据

- `first_party_editor_catalog/src/catalog.rs:10-32` 只在 `EditorHost` 目标投影 selection；parse/provider/duplicate 失败均被压成空结果。
- `first_party_editor_catalog/src/catalog.rs:35-54` 用两个 cfg branch 返回 provider function pointer；没有 generated declaration matrix、feature provenance、provider ABI/build id 或 missing diagnostics。
- `first_party_editor_catalog/src/tests.rs:19-30` 的 source assertion 与当前 implementation 不一致；其余 tests 只覆盖 Navigation/Neural disabled、wrong target、duplicate。
- `zircon_app/src/entry/first_party_editor_plugins.rs:10-18` 从 config clone manifest 或 default，未合并 runtime profile manifest、mode baseline、render overlay，也没有 runtime/editor provider parity check。
- `zircon_app/src/entry/product_host_config/resolution.rs:314-348` 在无 request/profile/required/optional 时返回 `None`；Editor role 因此可能完全没有 project plugin manifest。
- `zircon_app/Cargo.toml:136-156` 的 target-editor-host 没有统一开启 editor catalog/base catalog；feature closure 依赖调用者另行补充。
- editor plugin entry 中的 command、asset、toolkit、ZUI extension 多数只在 crate 自己的 unit tests 或宏注册中可见，没有 Editor Host admission、reload、shutdown、runtime dependency、preview-world 或 error receipt 测试。

## 3. 当前 Editor 装配流程与工程化断点

```text
plugin.toml/editor crate/ZUI
  -> Cargo optional dependency + cfg (手写、分散)
  -> ResolvedProductHostConfig.project_plugin_manifest()
  -> first_party_editor_catalog (仅 Navigation/Neural)
  -> Vec<EditorPluginRegistrationReport>
  -> EditorModule / retained host / workbench
```

缺少以下中间 owner：

1. `EditorPluginDeclaration`：manifest、editor crate、runtime dependency、target、maturity、packaging、ABI、authoring capabilities、resource kinds 的单一声明。
2. `EditorProviderResolutionReceipt`：每个 manifest selection 的 selected/disabled/unknown/uncompiled/descriptor-only/runtime-mismatch/rejected 状态。
3. `EditorPluginGeneration`：catalog generation、provider source digest、runtime dependency generation、ZUI template revision、operation factory和reload boundary。
4. `EditorAdmissionTransaction`：prepare provider/module/asset/toolkit/commands，commit 到 Editor Host，失败回滚并保留 last-good generation。
5. `EditorRuntimeParityReceipt`：editor provider 与 runtime provider/package/module/capability/ABI 的双向依赖和版本证据。

## 4. P0 阻断项

| ID | 阻断 | 当前证据 | 必须重构 |
|---|---|---|---|
| ED-PC-01 | Editor catalog 只覆盖 2/25 editor declarations | `first_party_editor_catalog/src/catalog.rs:41-54` 只有 Navigation/Neural；manifest scan 有 25 editor ids | 从 manifest/Cargo metadata 生成全量 editor declaration/provider matrix；每个 id 必须有 provider、descriptor-only policy 或结构化 missing reason |
| ED-PC-02 | Provider selection 静默丢失 | `catalog.rs:20-30` parse/provider 失败 `continue`，App wrapper 返回 bare Vec | 返回 typed receipt；required/editor-host provider 缺失在 admission 前 fatal，optional 也必须可见 |
| ED-PC-03 | Editor Host feature closure 不完整 | `zircon_app/Cargo.toml:136-156` 没有统一 editor/base catalog feature；App 只能路由手动启用的两个 provider | 建立 target-editor-host 的生成 feature closure；profile、role、editor catalog、runtime catalog 共同校验 |
| ED-PC-04 | 真实 authoring contribution 不可达 | AI/Animation Graph/Physics/Sound/Terrain/Timeline/UI 等 entry 有 contribution，但没有 catalog route/host registration | provider admission 必须携带 commands、asset types、toolkits、extensions、runtime dependencies、lifecycle，不得仅因 crate 存在就宣称可用 |
| ED-PC-05 | Catalog source test 已与实现漂移 | `tests.rs:19-30` 要求 `registrations.push(registration);`，production 当前只 `push(provider())` | 修复测试与实现，改为行为/生成矩阵测试；禁止 source assertion 伪造 provider coverage |

## 5. P1 重构账本

| ID | 差距 | 目标验收 |
|---|---|---|
| ED-PC-06 | manifest/editor crate/Cargo feature 三份声明 | 生成单一 editor declaration matrix，crate、feature、symbol、manifest id 只定义一次 |
| ED-PC-07 | runtime/editor ID 字符串可能漂移 | typed canonical id、alias/redirect、schema migration 和 compile-time duplicate 检查 |
| ED-PC-08 | descriptor-only rendering policy 不明确 | Rendering 等 descriptor-only 插件明确 `DescriptorOnly`/`RuntimeOnly`/`AuthoringProvider` 类型与 UI 行为 |
| ED-PC-09 | provider registration 无 provenance | registration 携 package id、crate version、feature set、source digest、ABI/build id、target |
| ED-PC-10 | runtime dependency 没有 admission graph | editor provider 声明所需 runtime providers/capabilities；缺依赖在 resolve 阶段拒绝 |
| ED-PC-11 | target filtering 多层重复 | 单一 EditorHost/Client/Server target predicate，拒绝 reason 可回溯 |
| ED-PC-12 | required/optional/editor-only 语义不一致 | selection 记录来源、required、optional、role、profile、packaging，不靠 bool OR 猜测 |
| ED-PC-13 | App wrapper 不合并 effective manifest | Editor provider 入口消费与 Runtime 相同的 profile/mode/render-effective manifest 或明确 editor overlay |
| ED-PC-14 | no plugin generation pin | EditorModule、Workbench、asset index、operation registry、native host pin 同一 generation |
| ED-PC-15 | no admission transaction | provider、commands、asset/toolkit、ZUI extension prepare/commit/rollback，失败保留 last-good |
| ED-PC-16 | reload/unload 无 owner | provider reload 先 quiesce commands/jobs/watchers，撤销 contributions 后再卸载，所有 handles 收到 terminal receipt |
| ED-PC-17 | editor plugin lifecycle 不可观测 | initialize/ready/failed/deactivate/unload 状态、耗时、错误 code、generation 和 owner 可查询 |
| ED-PC-18 | commands 无 capability admission | 每个 command 绑定 document/world/selection/job capability，缺权限在执行前拒绝 |
| ED-PC-19 | asset/toolkit contribution 不可验证 | registration 通过 typed asset schema/importer/toolkit registry，支持 duplicate owner、priority、dispose |
| ED-PC-20 | ZUI extension 与 Rust provider 无 source map | extension id、template revision、provider contribution、callback route 有双向 provenance |
| ED-PC-21 | Workbench 仍可显示静态模块 | module navigation、status、Save/Compile/Preview/Play 必须读取 provider receipt/operation result，不从模块名推断可用 |
| ED-PC-22 | Editor plugin 与 runtime package parity 缺失 | editor/runtime package manifest、capability、resource kind、ABI version、target matrix 生成 parity report |
| ED-PC-23 | native dynamic editor provider 不在 catalog | dynamic editor crate 记录 artifact identity、trust、ABI、load state、shutdown/retry receipt |
| ED-PC-24 | externalized provider 策略缺失 | EditorHost 明确 builtin/linked/native/externalized provider ownership 和 security policy |
| ED-PC-25 | provider errors 被压成字符串/空结果 | structured error code、severity、source chain、repair action、telemetry 和 UI projection |
| ED-PC-26 | operation factory 不是 provider admission 的一部分 | command/operation descriptor 必须有 factory、typed input/output、cancellation、undo/redo scope |
| ED-PC-27 | authoring preview 与 runtime artifact 脱节 | preview world/asset compiler 使用 runtime artifact；禁止静态 fixture 作为成功证明 |
| ED-PC-28 | editor/runtime save schema 未对齐 | provider-owned data、manifest selection、asset references、ZUI state、migration version 统一存档 |
| ED-PC-29 | plugin contribution duplicate policy 缺失 | command/menu/inspector/asset/toolkit/extension 重复 owner 有 deterministic priority 或 fatal conflict |
| ED-PC-30 | editor shutdown 不收 provider jobs | EditorJobAuthority/TaskGraphScope 关联 provider owner，shutdown 等待 cancel acknowledgement 与 late publish fence |
| ED-PC-31 | host role 与 runtime profile 不一致 | `for_profile`/`for_runtime_profile` 统一 resolution，Editor provider看到同一 target/mode/profile |
| ED-PC-32 | no catalog cache provenance | cache key 包含 catalog generation、manifest fingerprint、Cargo features、provider source/artifact digest |
| ED-PC-33 | tests 只覆盖两个 provider | 生成 25-id matrix、missing/duplicate/wrong-target/descriptor-only/runtime mismatch/reload/shutdown tests |
| ED-PC-34 | no fault/security tests | provider panic、factory error、ZUI callback failure、native unload、untrusted artifact 可回滚且不污染 host |
| ED-PC-35 | no scale budget | 100+ provider、10k commands/assets/toolkits、cold start/reload/inspector lookup 有 P95/P99 budget |
| ED-PC-36 | no release provenance | export/editor package bundle 保存 declaration matrix、provider receipt、ABI、ZUI/template、source revision |
| ED-PC-37 | variable `entry_config` hides resolved type | 入口命名和 boundary 类型分离，禁止 Config/ResolvedConfig/Registration 混用 |
| ED-PC-38 | package rows 与 actual entry files drift | CI 对 manifest、entry file、plugin_registration、ZUI assets、Cargo feature 做 bidirectional audit |
| ED-PC-39 | feature-gated tests under-report coverage | 缺 feature 的 provider tests 仍生成 negative receipt，不能因 cfg 排除就从 coverage 消失 |

## 6. P2 产品化与性能

P2 共 10 项：1) Provider Browser/health dashboard；2) declaration-to-ZUI provenance viewer；3) startup admission waterfall；4) command/asset/toolkit contribution heatmap；5) generation/cache telemetry；6) hot-reload impact preview；7) editor package compatibility report；8) offline project validation；9) provider load cost budget；10) diagnostic bundle export。它们必须消费 typed receipt/generation，不能从 provider 数量或 Workbench 文案推导成功。

## 7. 资格门

| Gate | 当前 | 必须证明 |
|---|---|---|
| G1 editor declaration coverage | Fail | 25 editor manifest ids 均有 provider、descriptor-only policy 或 missing receipt |
| G2 catalog provider closure | Fail | Cargo feature、crate、symbol、manifest id、target 全部闭合 |
| G3 no silent drop | Fail | parse/unknown/uncompiled/blocked/duplicate 都有结构化 receipt |
| G4 App feature closure | Fail | target-editor-host 默认启用并校验 editor/base/runtime provider set |
| G5 authoring reachability | Fail | AI/Animation/Physics/Sound/Terrain/Timeline/UI/Material 等 contribution 可从 Host admission 到达 |
| G6 descriptor-only semantics | Partial | descriptor 可生成，但 policy 与 host UI 行为未统一 |
| G7 runtime/editor parity | Fail | provider dependency、capability、package/ABI/version、artifact 有一致报告 |
| G8 generation pin | Fail | EditorModule/Workbench/asset/operation/native host 使用同一 generation |
| G9 admission transaction | Fail | provider contribution prepare/commit/rollback 可证明 |
| G10 reload/unload | Fail | quiesce/cancel/revoke/unload/last-good/retry 有 terminal receipt |
| G11 command capability | Fail | command 在执行前检查 document/world/selection/job capability |
| G12 asset/toolkit ownership | Fail | duplicate/dispose/priority/registry closure 可回溯 |
| G13 ZUI source map | Fail | template/callback/provider/operation 有双向 provenance |
| G14 operation correctness | Fail | descriptor 有 factory、typed result、undo/redo、cancel、error |
| G15 preview/runtime parity | Fail | preview 使用 runtime artifact/compiled schema，不用静态成功 fixture |
| G16 persistence/migration | Fail | provider data、selection、ZUI state、asset refs save/reload/migrate 稳定 |
| G17 native/trust | Fail | dynamic provider ABI、signature/trust、platform、load state 可验证 |
| G18 diagnostics | Fail | Editor/CLI/debugger 显示真实 receipt/error，不显示静态 Available |
| G19 test matrix | Fail | 25-id 正反例、feature missing、duplicate、target、reload、shutdown 在 CI 可复算 |
| G20 fault containment | Fail | panic/factory/ZUI/native failure 不污染现有 generation |
| G21 shutdown | Fail | provider tasks/readers/watchers cancel 并确认，禁止 late publish |
| G22 scale | Fail | 大量 provider/contribution 的冷启动、查找、reload 有预算 |
| G23 release provenance | Fail | editor package/export bundle 可还原 provider/source/ABI/template |
| G24 source-test integrity | Fail | source assertions 与实现同步，关键行为有运行时证据 |
| G25 role/profile parity | Fail | Editor role 与 Runtime profile 生成相同 effective selection 语义 |
| G26 cache invalidation | Fail | manifest/feature/provider/ZUI/artifact 变化原子失效旧 consumer |
| G27 user recovery | Fail | missing provider 给出 repair/disable/last-good/retry action |
| G28 product UI truth | Fail | Workbench module/status/action 只来自 provider and operation state |

## 8. 参考引擎约束

Unreal 的 module/plugin manager 先验证 required plugin/module 是否可用，再加载并跟踪 module ownership；Editor asset/toolkit registration 具有明确 owner 和撤销边界。Godot GDExtension manager 将 load/initialize/deinitialize/reload/error 作为显式状态，不能用空列表代表失败。Bevy `PluginGroupBuilder` 对 add/set/disable/duplicate 有确定性构建语义；Fyrox 将静态/动态 plugin 与 editor plugin lifetime 分开。Unity Graphics 的 package.json、asmdef 和 editor/runtime assembly 共同描述版本与依赖，Editor package 不会因为有一个 C# 文件就自动可用。Zircon 应吸收这些约束：声明矩阵、provider receipt、唯一 generation、贡献事务、显式撤销和 runtime/editor parity 必须先于 Workbench 丰富化。

## 9. 重构顺序

先建立 `EditorPluginDeclarationMatrix`、`EditorProviderResolutionReceipt`、`EditorPluginGeneration` 和统一 diagnostic/error code，关闭 ED-PC-01/02/03/05；将 target-editor-host 的 Cargo feature closure 与 runtime profile resolution 统一。随后从 workspace/plugin metadata 生成 catalog，逐个接入已有真实 contribution（AI、Animation Graph、Physics、Sound、Terrain、Timeline、UI Asset、Material、Neural、Navigation 等），对 Rendering 等 descriptor-only 项目明确 policy。第三阶段接入 admission transaction、command/asset/toolkit/ZUI source map、runtime dependency、preview artifact、reload/shutdown。第四阶段补齐 native/trust、save/migration、fault/scale/reproducibility、25-id matrix、release provenance；最后才扩展更多 UI 和 Workbench 操作。兼容阶段允许 provider 缺失，但必须显示结构化 receipt，不得恢复 silent `continue`。

本轮仅完成 Editor259 review/index/coverage 文档，没有修改 Editor、Runtime、plugin、Cargo、ABI 或 ZUI 生产代码，也没有运行 Cargo、真实 Editor、UI automation、动态 provider、reload、fault、scale、soak 或 benchmark。Tooling 按用户要求排除；按用户要求未查询、轮询、等待或实时跟踪协调器。工作树中已有 catalog provider function pointer、UI importer、Windows 依赖和其他未提交修改均保留，报告评价的是当前合并边界。
