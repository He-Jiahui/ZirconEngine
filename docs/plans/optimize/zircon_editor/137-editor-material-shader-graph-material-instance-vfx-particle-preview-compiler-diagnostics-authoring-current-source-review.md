---
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/rendering
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/rendering
  - zircon_editor/src/core/commands
  - zircon_editor/src/core/editing
  - zircon_editor/src/core/extension/store
  - zircon_editor/src/ui/host/asset_editor_sessions
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_editor/src/ui/material_editor
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench
  - zircon_plugins/editor_support
  - zircon_plugins/first_party_editor_catalog
  - zircon_plugins/material_editor
  - zircon_plugins/particles
  - zircon_plugins/rendering/features/shader_graph
  - zircon_plugins/rendering/features/vfx_graph
  - zircon_plugins/shader_wgsl_importer
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/material
  - zircon_runtime/src/asset/assets/shader
  - zircon_runtime/src/graphics/shader/shader_assets.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/15-material-shader-graph-instance-vfx-particle-preview-compiler-diagnostics-authoring-review.md
  - docs/plans/optimize/zircon_editor/91-editor-material-shader-graph-material-instance-vfx-particle-preview-compiler-diagnostics-authoring-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/99d-runtime-particle-vfx-system-emitter-cpu-gpu-simulation-rendering-scalability-determinism-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_plugins/04-rendering-umbrella-feature-bundles-solari-native-provider-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/05-shader-wgsl-family-importer-compiler-artifact-native-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/09-first-party-particle-vfx-source-runtime-editor-dist-catalog-simulation-render-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/18-first-party-texture-source-importer-runtime-editor-dist-catalog-image-cubemap-array-volume-compression-streaming-product-integration-review.md
reference_engines:
  - dev/Graphics/Packages/com.unity.shadergraph
  - dev/Graphics/Packages/com.unity.visualeffectgraph
  - dev/UnrealEngine/Engine/Source/Editor/MaterialEditor
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/NiagaraEditor
  - dev/godot/editor/shader
  - dev/godot/modules/visual_shader/editor
  - dev/godot/editor/scene/particle_process_material_editor_plugin.cpp
  - dev/godot/editor/scene/particles_editor_plugin.cpp
  - dev/Fyrox/editor/src/plugins/material
  - dev/Fyrox/editor/src/scene/commands/material.rs
  - dev/Fyrox/editor/src/particle.rs
  - dev/bevy/crates/bevy_pbr/src/material.rs
  - dev/bevy/crates/bevy_shader/src
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
review_id: Editor137
---

# 137 · Editor Material / Shader Graph / Material Instance / VFX / Particle / Preview / Compiler / Diagnostics Authoring 当前源码复核

## 1. 结论

截至 `2026-08-26T15:14:52+08:00`、HEAD `8e56165c4c789416c328898d3d8937d934b52efa` 的物理工作树，Editor15/Editor91 的产品级结论没有关闭：Zircon 有可保留的 Material/Shader asset 合同、Material Graph TOML import、Naga WGSL parse/validate、Material/RendererData typed projection、共享 transaction/save/job/factory 框架和真实 Particles CPU/GPU runtime，但没有形成任何一条可交付的 Material、Shader Graph、Material Instance、VFX Graph 或 Particle authoring 闭环。

本轮确认的阻断不是“控件数量不够”，而是产品权威未接通：

1. 默认 first-party Editor catalog 与 `zircon_app` feature 闭包仍只装配 Navigation/Neural，没有 Material、Shader、Particles、VFX Editor。
2. Material plugin 声明的 `plugins://material_editor/editor/graph.zui` 与 `plugins://material_editor/templates/default_material_graph.toml` 仍无物理文件；Particles 的三份 ZUI 共 131 行，业务 event/command/binding 命中为 0，只含 13 个 `Space` 占位。
3. Core 已有 `OperationCommandFactoryRegistration`，Material 的 6 个、Particles 的 12 个 visible operation 却都只注册 descriptor；Particles 的全部菜单项继续 `with_enabled(false)`。
4. 四份 rendering Workbench 仍是 **884 行 / 103 nodes / 71 event routes / 0 providers**，固定展示 `M_Rock_Cliff`、`P_Bolt_01`、`lighting.wgsl`、`P_Sparks`，callback 直接写入“compiled”“persisted”“60 fps”“no errors”等静态成功文本。
5. runtime authoring、graphics shell 与 rendering feature 继续维护三套不兼容 Material/Shader graph DTO；Material compiler 只折叠 base color，Shader Graph 直接拼接未验证 WGSL，VFX 与 Shader Graph executor 仍是 `Ok(())` no-op。
6. Material/VFX/Particle 没有领域 DocumentId、source revision、transaction command、durable save receipt、last-good artifact、runtime preview session 或 generation-qualified telemetry。

因此 canonical 状态保持：Editor15 的 **5 项 P0 全部 Open**；60 项 P1 为 **21 Open / 39 Partial / 0 Closed**；12 项 P2 全部 Open；32 项资格门为 **21 Fail / 11 Partial / 0 Pass**。本报告刷新现状而不重复增加 canonical finding 总数。

```text
versioned source document
  -> reversible domain transaction
  -> one semantic compiler authority
  -> immutable target artifact + source map + diagnostics
  -> background build / dependency invalidation / last-good publication
  -> isolated runtime preview + generation-qualified telemetry
  -> real toolkit / inspector / graph-text-curve editor / debug view
```

## 2. 证据范围与 currentness

### 2.1 物理选择集

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 本轮证据 |
|---|---:|---|
| Editor 产品、command/document/extension 与 projection | **167 / 32,084 / 29,520 / 1,157,589 / 163 / 0** | 四份 Workbench、feedback/route、Material projection、asset session、command/factory、editing/extension store、catalog 与 App；fingerprint `c1576844084b250ce0f6031e2b1c35b2d059e780ad6fc539a7552ff514e14c00` |
| Material/Shader source、plugin、importer 与 runtime contract | **51 / 9,346 / 8,578 / 343,752 / 52 / 0** | Material Editor、WGSL importer、authoring graph、Material/Shader asset、readiness 与 loader/cache 接点；fingerprint `ef23847a1dd934ddbb4b049a095170bccbf3a79fac0b55d0a6b396bbd1fa364f` |
| Particle/VFX/Shader Graph package 与 runtime | **74 / 9,568 / 8,678 / 335,241 / 52 / 0** | Particles editor/runtime/templates/tests与Shader Graph/VFX Graph feature；fingerprint `9bca1c27b86ade465fcee1a3c573fa584087862853dbd92f84c5ffa189631282` |
| Zircon selected union | **292 / 50,998 / 46,776 / 1,836,582 / 267 / 0** | 上述三组去重物理集合；fingerprint `a2027aaf3aeafbe7d31edda1a3a4ed433465cb7bf4e6c535878c37e43393610e` |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics reference | **29 / 59,560 / 51,593 / 2,474,796 / 0 / 0** | graph model/validation/generation、undo/invalidation、compiled data、preview、runtime material与shader cache；fingerprint `14559f0615e33422264b89c94e6ec8a0cd9e1ddc322d80d1c80df64425cc8d9b` |
| Plan/docs evidence | **14 / 5,768 / 4,282 / 585,172 / 0 / 0** | Editor15/91、共享Editor owner、Runtime与Plugin parent reports；fingerprint `7403cd97c927f7883c9d03c0eb1cac6498fdb82063766292f63f4dc78405cae8` |
| 全部证据 union | **335 / 116,326 / 102,651 / 4,896,550 / 267 / 0** | 当前物理工作树去重集合；fingerprint `5857b03cadf51d3a9cd23738a2373ea2d12ce59decf7611d7dfc3d9658244e71` |

fingerprint 将 repo-relative path 排序后，以 `path + NUL + file SHA-256 + LF` 聚合再取 SHA-256。它只标识本轮静态证据集合，不是 shader permutation、PSO、DDC 或 cook key。

### 2.2 Editor91 之后的有效变化

本轮逐项比对 Editor91 捕获时间之后的相关物理文件。Shader/Material runtime 边界有一组应保留的真实进展：raw WGSL importer 改为产出 `ShaderAssetKind::Module`；readiness 增加 `kind/kind_diagnostic` 并使错误Surface、错误stage与非法entry fail-closed；Material validation拒绝非Surface shader；`.zshader v2` 明确拒绝直接声明Module。duplicate tracking同时从`BTreeSet`收敛到`HashSet`，token/stage/import entry projection增加release性能门。

这组变化修正了raw module冒充material surface的资产分类错误，强化 `ED-MSV-P1-007/028/036` 的Partial证据，但仍没有 importer reflection、graph compiler consumer、Editor effective readiness resolver、领域artifact publication或preview，不改变任何P0/P1状态。`zircon_runtime/src/asset/assets/authoring.rs` 的另一项变化属于Terrain direct-reference allocation；Workbench feedback的变化属于Blend Space，均不属于本报告闭环。当前选择集处于大规模共享脏工作树，本轮只读取并记录现状，没有回退或覆盖任何既有生产改动。

### 2.3 动态证据边界

按用户要求本轮只做 review，没有运行 Cargo、Editor/App、create/import/save/reopen/cook、backend shader compiler、GPU preview、particle parity、device-loss、fault/fuzz/scale/soak/profile 或跨引擎 benchmark。267 个静态 Rust test declaration 证明局部 contract 有测试，不证明 default bootstrap、factory dispatch、source-to-pixel 或 source-to-particle 产品闭环。

## 3. 当前实现纵向事实

### 3.1 产品装配、资源与命令真值

1. `ResourceKind` 已识别 Material、MaterialGraph、Shader，builtin toolkit 仍只覆盖 UI 与 Animation；缺插件时没有领域 fallback details/diagnostics/preview。
2. Material plugin 注册 view、drawer、toolkit、graph descriptor、palette、creation template 与 6 个 operation，但两个声明资源缺失，publication 也没有在 registry mutation 前验证 URI。
3. Particles 注册 create/open/add emitter/add module/edit curve/validate 及 preview transport descriptor；三份 ZUI 无业务绑定，菜单统一 disabled，`cpu_sprite_system.toml` 没有 create/import/save consumer。
4. Core registry、contribution store 和 dispatch 已支持 operation factory。精确 consumer 搜索显示 Material/Particles package 没有 `OperationCommandFactoryRegistration` 或 factory registration。
5. first-party catalog 与 App 默认 feature 仍没有四类 Editor。feature capability、manifest maturity、资源完整性、factory、runtime executor 与可见状态没有单一 effective readiness resolver。

### 3.2 Document、transaction、save 与可见 UI

1. Editor 的 transaction/history、dirty/save/autosave、CAS/refresh、job 与 toolkit/save 底座是真实代码；Material/VFX/Particle 没有领域 adapter。
2. `MaterialEditorProjection` 和 `RendererDataEditorProjection` 的生产引用为零，除定义外只出现在 focused tests；它们只能读，不能提交 typed edit。
3. 四份 Workbench 的 71 条 route 进入通用 template bridge，Change/Submit 只改控件属性；Save/Compile/Simulate/Preview 结果来自固定字符串，不带 document revision、job ticket、artifact generation 或 runtime receipt。
4. generic `asset_editor_sessions` 是 UI Asset document 产品，不能把 UI source schema、loader、mutation 和 preview 直接改名当 Material/VFX document。
5. 没有 unload/reload 时 unknown node/module、opaque payload、unsaved source、last-good artifact 与 preview session 的保持/恢复策略。

### 3.3 Material、Shader 与 compiler authority

1. canonical runtime authoring `MaterialGraphAsset` 可 serde/TOML round-trip并记录 URI、shader、node/link/parameter，但无 source schema version、stable edge/pin ID、unknown preservation、domain/target 或 migration envelope。
2. 节点域只有 Output、TextureSample、ScalarParameter、VectorParameter、Add、Multiply；output 只要求 `base_color`。
3. Material validator覆盖empty/duplicate node ID、output数量、missing node/pin、duplicate incoming 与required input；cycle只在递归求值时发现，缺typed pin、conversion、domain、finite、reachability与bounded topology。
4. compiler只编译base color；normal/metallic/roughness/occlusion/emissive/alpha/options/queue等全部写固定默认，texture-backed Add/Multiply直接以“v1”拒绝。
5. graphics `shader_assets.rs` 又定义只有 `name/output_domain` 的 MaterialGraph/ShaderGraph shell；Shader Graph feature再定义一套自由字符串连接的DTO，权威继续分裂。
6. Shader Graph把任意node ID/input直接插入WGSL identifier；TextureSample调用未声明的 `zircon_sample_texture_N()`；缺output时既报错又生成magenta函数；产物不经过Naga且executor为no-op。
7. WGSL importer真实调用Naga parse/validate并提取entry points，raw source现已正确标记为`Module`；但dependencies/source_files/imports/defs/property schema/options/texture slots/resources/layout/editor/pipeline等仍为空或默认。
8. Shader readiness现已按Module/Surface/Include/Compute/Fullscreen检查kind、shading model和entry stage，Material validation也拒绝非Surface shader。这是runtime fail-closed底座，不是Editor graph/text compiler或reflection产品。
9. Material/Shader runtime合同和readiness检查应作为artifact承载面；Editor不得另建平行PSO或shader cache authority。

### 3.4 Particle、VFX 与 runtime preview

1. Particles runtime已有System/Emitter、CPU/GPU simulation、pool/RNG、extract、GPU layout/program/planner/runtime owner、readback、transparent rendering及大量snapshot测试。
2. `ParticleSystemAsset` 仍只有 `Clone/Debug/PartialEq`，无serde/version/migration/unknown preservation；`particles.system`只存在于Editor registration/test，不在Runtime AssetKind/importer链。
3. VFX Graph只有SpawnRate/Lifetime/Velocity/ColorOverLife/ShaderGraphMaterial；compiler只检查capacity/spawn/material后返回固定pass名。
4. VFX workload固定 `[64,1,1]` workgroup与`[1,1,1]` dispatch；simulation/transparent executor都是no-op，不能更新particle state或产生像素。
5. Material没有runtime-real preview scene/mesh/camera/environment/target；Particle没有isolated world/clock/seed/warmup/rewind/generation gateway；Workbench“preview”文本不能代替这两条链。

## 4. 参考引擎逐源码差异

| 参考 | 已核实工程事实 | Zircon 应采用的边界 |
|---|---|---|
| Unity ShaderGraph | `GraphData`序列化property/keyword/node/edge/target；`GraphValidation`按Target/SubTarget验证；`GraphEditorView`对connect/move/remove统一登记Undo；`Generator`由target setup生成完整shader与preview target | source、validation、UI transaction和target generation分层但同源；palette、connection、validator、compiler不得各自解释pin语义 |
| Unity VFX Graph | `VFXModel`传播明确InvalidationCause；CompiledData构建expression/context/task/buffer；Undo区分backup/delta并决定recompile；code generator输出可执行program | 建立typed context/data/model、增量失效、compiled buffer/dispatch计划和undo/recompile currentness，不复制Unity对象模型 |
| Unreal Material | MaterialGraphSchema在连接前做兼容/loop检查并进入transaction；Material Editor把apply/preview/stats与graph编辑分工 | Material/Instance/Graph/Preview/Stats应是协作子系统；连接合法性与compiler必须共享规则，preview执行真实shader |
| Unreal Niagara | Graph维护compile hash/recompile notification；compiler job、HLSL translator、message manager和SystemViewModel分别拥有异步结果、typed pin、诊断与preview component | VFX/Particle需要compile identity、async result、element diagnostic与preview instance，不能只返回pass标签 |
| Godot | Shader Editor维护多文档、unsaved与错误定位；Visual Shader和Particles动作通过UndoRedo修改真实resource/node | text/visual shader共享runtime语义；所有可见编辑与particle工具动作必须可撤销并作用于真实资源 |
| Fyrox | Material editor由shader resource schema生成property editor并驱动真实preview；material command有execute/revert/save；particle preview保存并恢复scene node状态 | Rust实现可采用typed command、resource-driven inspector与隔离preview状态恢复 |
| Bevy | Material以typed bind group和pipeline specialization进入render；Shader记录imports/dependencies/defs；ShaderCache等待依赖、缓存processed shader并追踪dependent pipeline | Editor artifact必须接Runtime shader/material/cache/PSO权威；Bevy只作runtime contract参考，不作Editor UX基准 |

## 5. 父 P0 当前重判

| Canonical owner | 状态 | 当前证据与硬切要求 |
|---|---|---|
| `Editor15-P0-01` 默认产品入口、插件装配与资源包不闭合 | Open | 默认catalog仍无四类Editor，Material两个资源仍缺失。publication必须原子验证package/resource/factory/provider/backend。 |
| `Editor15-P0-02` authoring operation只有descriptor | Open | Core factory路径真实存在，Material/Particles贡献仍未注册factory，Particles菜单disabled。visible action必须产生owner-qualified terminal receipt。 |
| `Editor15-P0-03` Workbench伪造compile/simulate/preview | Open | fixed feedback继续宣称compile/persist/60 fps/no errors。删除业务固定成功分支，只接受typed job/compiler/runtime receipt。 |
| `Editor15-P0-04` graph authority分裂且输出不可证明执行 | Open | 三套graph继续并存，WGSL未Naga验证，Shader/VFX executor仍no-op。硬切到单一schema/compiler/artifact。 |
| `Editor15-P0-05` 无transactional document、durable save与runtime一致preview | Open | shared底座存在但领域consumer为零；补document/revision/history/save/LKG/preview generation闭环。 |

## 6. P1 当前差距账本

### 6.1 产品装配与 lifecycle

1. `ED-MSV-P1-001` [Partial] builtin识别资产但没有领域fallback toolkit：补只读details/diagnostics fallback，完整provider过资格门后才切换可编辑toolkit。
2. `ED-MSV-P1-002` [Open] 默认first-party Editor catalog没有四类产品：补package feature/dependency/registration矩阵与production bootstrap。
3. `ED-MSV-P1-003` [Open] Material插件资源合同损坏：补齐并版本化两份资源，publication在mutation前验证URI、owner和provenance。
4. `ED-MSV-P1-004` [Open] dist invocation/bridge没有执行fallback：明确native/editor-only边界；无执行路径的descriptor必须Unavailable。
5. `ED-MSV-P1-005` [Partial] shared operation factory存在但authoring batch没有领域factory：factory/document/compiler/preview provider须同descriptor原子prepare/commit/revoke。
6. `ED-MSV-P1-006` [Partial] graph descriptor/palette有registry但无产品consumer：建立owner-qualified graph session、selection、transaction与schema generation。
7. `ED-MSV-P1-007` [Partial] capability/maturity/readiness语义分裂：统一manifest、resource、factory、backend、target和executor effective readiness。
8. `ED-MSV-P1-008` [Open] Particles三份surface仍为空壳：以document-backed emitter/module/curve/viewport替换13个业务Space。
9. `ED-MSV-P1-009` [Open] `particles.system`没有Runtime AssetKind/importer：建立versioned source、bounded importer、artifact、save/reopen/cook/runtime handle。
10. `ED-MSV-P1-010` [Partial] shared document/save/autosave真实但领域未接入：补DocumentId/revision/dirty/history/savepoint/autosave/close adapter。
11. `ED-MSV-P1-011` [Open] plugin unload/reload没有领域保持策略：定义unknown/opaque/unsaved/LKG/preview状态机。
12. `ED-MSV-P1-012` [Partial] experimental/partial metadata没有形成产品门禁：navigation/command/toolkit/restore消费同一effective snapshot。

### 6.2 Material与Material Instance

13. `ED-MSV-P1-013` [Partial] Material projection只读且无production consumer：增加stable row ID/revision/edit descriptor/transaction route并接入toolkit。
14. `ED-MSV-P1-014` [Partial] property schema不足以生成完整widget：补range/unit/color/enum/resource/visibility/dependency/validation/editor factory。
15. `ED-MSV-P1-015` [Partial] override状态可见但不可操作：补enable/reset/revert/copy/paste/mixed value/origin和原子提交。
16. `ED-MSV-P1-016` [Partial] parent material存在但无Editor继承工作流：补chain/cycle/missing检查、resolved diff、reparent与override迁移。
17. `ED-MSV-P1-017` [Partial] runtime PBR/render options较强但Editor不可编辑：完整映射shading/blend/queue/cull/depth/alpha与高级PBR参数。
18. `ED-MSV-P1-018` [Partial] texture slot有合同但无资源工作流：补drag/drop、dimension/color-space/sampler/fallback/residency/missing引用。
19. `ED-MSV-P1-019` [Partial] shader schema变化不驱动Material inspector：建立revision订阅、compatible migration、orphan preservation与精确失效。
20. `ED-MSV-P1-020` [Partial] diagnostics可投影但缺稳定身份/修复链：补code/span/element/artifact/target/quick-fix/suppress。
21. `ED-MSV-P1-021` [Open] Material Graph节点域无法表达MaterialAsset：每个持久字段必须可表达或返回明确unsupported。
22. `ED-MSV-P1-022` [Partial] validator只有局部structural检查：共享pin type/cardinality/conversion/cycle/reachability/finite/resource/domain规则。
23. `ED-MSV-P1-023` [Partial] compiler只覆盖base color：输出完整Material IR/Shader/artifact/pipeline recipe，未实现字段不得静默默认。
24. `ED-MSV-P1-024` [Open] palette、validator、compiler语义冲突：float/vec广播、texture组合与conversion由同一schema定义。
25. `ED-MSV-P1-025` [Partial] runtime artifact/PSO底座未被graph compiler消费：输出reflection/layout/variant/recipe/source map/key并交Runtime09C安装。

### 6.3 Graph schema、Shader与diagnostics

26. `ED-MSV-P1-026` [Open] 多套graph authority并存：确定唯一versioned schema，迁移后删除graphics shell与feature-local DTO。
27. `ED-MSV-P1-027` [Open] node/edge/pin没有稳定身份：引入stable ID、namespace/version、opaque unknown payload与layout metadata。
28. `ED-MSV-P1-028` [Partial] importer与plugin validator规则冲突：收敛到共享compiler preflight，import/open/save/preview/cook共用corpus。
29. `ED-MSV-P1-029` [Open] 没有共享pin/resource/domain规则：建立typed lattice、conversion、binding、stage/domain、default/cardinality authority。
30. `ED-MSV-P1-030` [Open] palette声明与求值器广播不一致：删除隐式广播或显式生成可见conversion artifact。
31. `ED-MSV-P1-031` [Partial] cycle只在递归求值时暴露：补bounded cycle path、topological order、dead-node与side-effect排序。
32. `ED-MSV-P1-032` [Open] WGSL identifier可被任意字符串破坏：建立symbol table、sanitize/collision/reserved-word/source-map与长度预算。
33. `ED-MSV-P1-033` [Open] TextureSample生成未定义helper/layout：生成完整binding、sampler/texture类型与ShaderAsset layout。
34. `ED-MSV-P1-034` [Partial] Naga validator存在但graph生成不使用：抽成graph/text/import共享publication stage并加入backend gate。
35. `ED-MSV-P1-035` [Open] error graph与fallback artifact边界不清：invalid source不得admit；LKG/diagnostic preview/intentional fallback为互斥typed状态。
36. `ED-MSV-P1-036` [Partial] WGSL importer真实但reflection合同为空：补dependency/defs/binding/property/texture/render state/target provenance。
37. `ED-MSV-P1-037` [Open] 没有Shader text document产品：实现unsaved buffer、incremental parse、completion、format、include导航、target/entry与buffer compile。
38. `ED-MSV-P1-038` [Partial] importer错误只有字符串投影：升级为stable code、file/line/column/span/include stack/target/revision。
39. `ED-MSV-P1-039` [Partial] dependency字段存在但无authoring失效图：建立transitive dependency/watch/CAS/invalidation/generated-source导航。
40. `ED-MSV-P1-040` [Partial] runtime option/pipeline基础未形成compile matrix：补target/subtarget/quality/platform/backend/variant budget/stripping/cache可视化。

### 6.4 VFX、Particle与runtime bridge

41. `ED-MSV-P1-041` [Partial] Particle操作有descriptor但无factory：为12个操作定义typed payload、transaction/history与terminal receipt。
42. `ED-MSV-P1-042` [Partial] Particle runtime模型真实但无versioned serialization：为System/Emitter/Module建立serde envelope、migration、unknown preservation与source/artifact分层。
43. `ED-MSV-P1-043` [Partial] CPU sprite template存在但不可创建运行：接creation/import/document/save/reopen/runtime load E2E。
44. `ED-MSV-P1-044` [Open] 没有emitter/module/curve/gradient authoring产品：建立stack/context/attribute/parameter/curve/renderer/bounds/LOD编辑。
45. `ED-MSV-P1-045` [Open] Particle preview没有真实session：实现isolated world/clock/seed/camera/play/pause/step/rewind/warmup/generation fence。
46. `ED-MSV-P1-046` [Partial] CPU/GPU runtime没有Editor gateway：暴露backend/capacity/buffer/readback/fallback/stats与声明范围parity。
47. `ED-MSV-P1-047` [Partial] VFX Graph只有极小schema/compiler：建立typed context/data-flow/module与canonical simulation/render IR。
48. `ED-MSV-P1-048` [Partial] VFX有workload描述但executor no-op：由program/capacity推导buffer/dispatch并实际更新粒子、输出像素。
49. `ED-MSV-P1-049` [Partial] feature capability与Editor/runtime readiness未绑定：聚合optional feature/backend/executor/dependency/provider availability。
50. `ED-MSV-P1-050` [Open] 高级VFX/Particle语义缺失：补event/collision/ribbon/mesh/decal/light/sub-emitter、bounds/culling/LOD/scalability/migration。

### 6.5 Preview、jobs、性能与测试

51. `ED-MSV-P1-051` [Partial] 通用render/viewport存在但无Material preview authority：建立独立scene/mesh/camera/environment/path/target/reference-image控制。
52. `ED-MSV-P1-052` [Open] 没有last-good/current-source差异状态：同时显示source/requested/LKG/installed/preview generation。
53. `ED-MSV-P1-053` [Partial] shared jobs未承载领域compile：接Editor09 admission/dedup/progress/cancel/quota/revoke/shutdown。
54. `ED-MSV-P1-054` [Partial] shader/pipeline cache缺完整authoring key：纳入source、transitive dependency、schema/compiler/plugin、target/options/backend与miss reason。
55. `ED-MSV-P1-055` [Partial] RendererData projection无live consumer：连接真实preview renderer/pipeline/PSO snapshot并限定view/artifact/generation。
56. `ED-MSV-P1-056` [Partial] typed diagnostics无法定位graph/VFX元素：统一到property/node/edge/module/curve ID与跨pane jump。
57. `ED-MSV-P1-057` [Partial] runtime有snapshot但Editor无工程统计：显示instruction/resource/variant/pipeline/cache/compile及particle CPU/GPU/alive/drop/overdraw。
58. `ED-MSV-P1-058` [Partial] tests较多但产品闭环false-green：补default bootstrap、factory、transaction/save、真实preview和source-to-output E2E。
59. `ED-MSV-P1-059` [Open] fault/fuzz/large graph/GPU证据缺失：建立malformed/Nth-step/unload/device-loss/cancel/large graph/variant/particle soak/image parity。
60. `ED-MSV-P1-060` [Partial] shared UI accessibility基础未覆盖领域工具：完成graph/text/curve/preview keyboard/focus/UIA/high-contrast/reduced-motion/i18n/virtualization。

## 7. P2 成熟度差距

1. [Open] per-user graph grid、wire style、node density、preview background与lighting rig偏好。
2. [Open] 可命名Material preview camera/environment/mesh与VFX seed/time/debug bookmark。
3. [Open] recent asset、recent element与跨document navigation history。
4. [Open] minimap、reroute、sticky note、frame/group、alignment和distribution。
5. [Open] property/node/module/curve批量标签、颜色、comment与review note。
6. [Open] source revision、generated source与preview image结构diff/overlay。
7. [Open] source/diagnostics/artifact manifest/GPU capture引用的bounded support bundle。
8. [Open] opt-in compile latency/cache hit/preview FPS/particle budget趋势。
9. [Open] 批量shader compile、material reparent/override迁移、particle validation与dry-run。
10. [Open] Material function、subgraph/module preset、template library与dependency usage查询。
11. [Open] typed Editor scripting/remote Material/Shader/VFX command/query surface。
12. [Open] 多人asset lock、元素级冲突、review-only与revision annotation。

## 8. 重构里程碑

### M0 · Product truth与publication硬门

- 建立四类Editor package与effective readiness matrix，补default bootstrap。
- publication原子验证resource、factory、document/compiler/preview provider与runtime executor。
- 删除fixed success；不完整能力统一投影typed Unavailable。

### M1 · Canonical versioned source

- 收敛三套Material/Shader graph，定义stable node/edge/pin/parameter identity、migration与unknown preservation。
- 建立Material/Shader/VFX/Particle source envelope与owner generation、domain/target/context/module registry。

### M2 · Shared semantic compiler与artifact

- 统一bounded parse、structural/type/topology/reference/domain validation。
- Material/Shader输出IR/WGSL/reflection/layout/variant/pipeline recipe/source map并经Naga/backend验证。
- VFX/Particle输出真实simulation/render program、buffer/dispatch/resource计划。

### M3 · Transaction、durability、dependency与jobs

- 所有property/node/edge/module/curve操作变为可逆typed command。
- 接validated snapshot、atomic replace、CAS、autosave/recovery与LKG。
- compile接job、DDC、transitive invalidation、cancel/progress/quota/shutdown。

### M4 · Material/Instance与Shader toolkits

- projection接真实Inspector，完成property/texture/options、parent/override/reparent/multi-edit。
- 实现Shader text/graph、blackboard/function/subgraph/generated source与diagnostic navigation。
- 建立真实runtime material preview和source-to-pixel golden。

### M5 · Particle/VFX authoring与preview

- 完成system/emitter/context/module/attribute/parameter/curve/renderer/bounds/LOD。
- 连接Particles runtime与VFX compiler，删除no-op executor。
- 完成isolated preview、transport、seed/clock、backend parity与telemetry。

### M6 · Failure、规模、可访问性与hard cutover

- 建立fault/fuzz/device/driver/large graph/variant/particle soak与budget门。
- 完成keyboard/focus/UIA/i18n/high-contrast/reduced-motion。
- 同里程碑迁移first-party caller后删除旧DTO、fixed feedback、descriptor-only入口、假fallback与双compiler/preview authority。

## 9. 32项资格门

- [ ] 1. **Fail** 默认production bootstrap可打开Material/MaterialGraph/Shader，缺插件返回typed reason。
- [ ] 2. **Fail** publication在缺factory/template/schema/compiler/preview/provider时原子失败。
- [ ] 3. **Fail** Material graph/template URI物理可加载并带version/owner provenance。
- [ ] 4. **Fail** 每个visible action都有factory并产生transaction/job/preview receipt。
- [ ] 5. **Fail** Workbench不以固定字符串宣称compile/simulate/preview成功。
- [ ] 6. **Fail** canonical loader按budget拒绝oversized/deep/truncated/unknown/malformed source。
- [ ] 7. **Fail** 旧graph schema可迁移或typed拒绝，并保持stable identity/opaque data。
- [ ] 8. **Partial** Material validator覆盖局部ID/link/output，尚缺完整type/topology/domain。
- [ ] 9. **Fail** palette、connection UI、validator与compiler共享pin/conversion规则。
- [ ] 10. **Partial** compiler能产生最小MaterialAsset，但未覆盖完整Material合同。
- [ ] 11. **Fail** generated WGSL自包含、安全并通过Naga与required backend。
- [ ] 12. **Fail** invalid source不产生可admit artifact，fallback/LKG状态明确。
- [ ] 13. **Fail** Shader text editor支持unsaved compile、include/source map与准确定位。
- [ ] 14. **Partial** runtime有typed options/pipeline基础，authoring target/permutation未闭合。
- [ ] 15. **Fail** Material parent/override/reparent/schema变化可undo/redo并正确迁移。
- [ ] 16. **Fail** property/texture/option按revision原子提交，失败不改状态。
- [ ] 17. **Fail** graph undo/redo恢复source、selection、diagnostic与generation。
- [ ] 18. **Partial** shared save/CAS/autosave存在，Material/VFX领域未接入。
- [ ] 19. **Partial** dependency/cache字段存在，authoring transitive invalidation未接通。
- [ ] 20. **Partial** shared jobs有admission/cancel基础，领域compile未提交job。
- [ ] 21. **Fail** Material/Shader preview执行真实runtime pipeline并标识generation。
- [ ] 22. **Fail** reference material scene通过required backend image golden/GPU validation。
- [ ] 23. **Fail** `particles.system`可versioned create/save/reopen/cook/runtime load。
- [ ] 24. **Fail** Particle/VFX完整authoring操作均可undo/redo。
- [ ] 25. **Fail** VFX executor实际更新粒子并输出像素。
- [ ] 26. **Fail** preview transport驱动isolated runtime且固定seed/clock可复现。
- [ ] 27. **Partial** Particles runtime有snapshot/readback，Editor无live telemetry产品。
- [ ] 28. **Partial** extension lifecycle底座存在，领域unknown/reload/LKG恢复未定义。
- [ ] 29. **Fail** 10万node/edge、万级variant、批量compile与百万粒子有预算benchmark。
- [ ] 30. **Partial** 有局部validator/performance test，缺完整fault/fuzz/device/driver矩阵。
- [ ] 31. **Partial** shared retained UI有部分input/a11y基础，领域toolkit未验收。
- [ ] 32. **Partial** unit tests覆盖局部projection/compiler/runtime，缺default product E2E。

## 10. Owner边界与完成定义

1. Editor137刷新Editor15/91唯一父账，不新增重复P0；Editor02/04/05/08/09/11继续拥有共享document/asset/inspector/command/job/diagnostic基础。
2. Runtime09C拥有Material/Shader/Pipeline/PSO执行合同；Runtime99D拥有Particle/VFX simulation/render/scalability/determinism；Plugins04/05/09/18拥有package/importer/runtime distribution与texture依赖。
3. Editor137只拥有领域document/toolkit、typed edit、compile orchestration、preview session、diagnostic projection和product truth；Editor不得直接拥有GPU handle或另建runtime compiler。
4. graph schema必须hard cut。迁移期可读旧格式，写出只允许canonical version；迁移完成即删除旧DTO/re-export/compat facade/旧caller。
5. fixed Workbench数据只能存在于显式fixture/demo，不能挂在production capability或Save/Compile/Simulate成功状态上。

完成定义不是“crate存在”或“单元测试能构造DTO”，而是默认产品真实可达、visible action有factory、source由单一versioned schema与共享compiler生成可验证artifact、document可撤销且durable、preview执行真实runtime并带generation、diagnostic可定位，以及dependency/job/cache/failure/规模/backend/a11y全部通过资格门。在此之前，四类产品必须明确标记Partial/Experimental或Unavailable。
