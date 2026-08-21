---
related_code:
  - zircon_editor/src/ui/material_editor
  - zircon_editor/src/core/asset/type_registry/builtin.rs
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_command_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/reference_menu_actions.rs
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/rendering
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/rendering
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/asset/assets/shader/shader_asset.rs
  - zircon_runtime/src/asset/importer/ingest/import_authoring_asset.rs
  - zircon_runtime/src/graphics/shader/shader_assets.rs
  - zircon_plugins/editor_support/src/lib.rs
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
  - zircon_plugins/material_editor
  - zircon_plugins/shader_wgsl_importer
  - zircon_plugins/particles
  - zircon_plugins/rendering/features/shader_graph
  - zircon_plugins/rendering/features/vfx_graph
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/MaterialEditor
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/NiagaraEditor
  - dev/godot/editor/shader
  - dev/godot/modules/visual_shader/editor
  - dev/godot/editor/scene/particle_process_material_editor_plugin.cpp
  - dev/godot/editor/scene/particles_editor_plugin.cpp
  - dev/Fyrox/editor/src/plugins/material
  - dev/Fyrox/editor/src/scene/commands/material.rs
  - dev/Fyrox/editor/src/particle.rs
  - dev/Graphics/Packages/com.unity.shadergraph
  - dev/Graphics/Packages/com.unity.visualeffectgraph
  - dev/bevy/crates/bevy_pbr
  - dev/bevy/crates/bevy_render/src/render_resource
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: false
---

# 15 · Material、Shader Graph/Instance、VFX/Particle、Preview、Compiler 与 Diagnostics Authoring 工程化差距

## 1. 结论

Zircon已经具备一批应当保留的底层基础：`MaterialAsset`和`ShaderAsset`表达了PBR参数、父材质、property schema、texture slot、render state、queue、pass/resource/layout及validation diagnostic；WGSL importer确实使用Naga解析和验证源码；`MaterialEditorProjection`能把材质/着色器合同投影为typed property、texture和diagnostic rows；Particles runtime也已有CPU/GPU simulation、extract、buffer、shader、transparent render与readback基础。结论不能简化为“材质、着色器和粒子系统完全不存在”。

但当前Editor产品不能称为工程级Material/VFX authoring，更不能用现有界面证明这些能力已成立。最严重的五个断点是：

1. 默认linked first-party editor catalog只暴露Navigation和Neural；builtin asset type registry虽识别Material、MaterialGraph和Shader，却不给toolkit。Material Editor插件即使由外部动态发现加载，其声明的`graph.zui`和default graph template也不在包内，默认产品入口和插件资源合同都不闭合。
2. Material和Particles注册的大量operation只是descriptor。authoring contribution batch没有operation factory字段，注册路径调用`register_command`而非`register_operation`；Material graph descriptor/palette没有产品consumer，Particles菜单显式disabled且三份ZUI只含`Space`。可见入口无法形成document mutation。
3. Core Material/VFX workspace和extension Shader/Particle workspace使用硬编码资产、节点、warning和编译结果；callback只改status/output字符串，却显示“compiled”“simulation running”“queued”等成功语义。它们没有compiler、job、preview world或runtime回执，构成能力过度声明。
4. 仓内并存三套互不兼容的material/shader graph schema。authoring importer只检查存在Output；Material插件compiler只折叠常量base color；Shader Graph直接拼WGSL且render executor为no-op；VFX Graph只返回固定pass名且两个executor均为no-op。Editor、import、cook、runtime没有共享semantic authority或canonical artifact。
5. 没有Material/Shader/VFX/Particle transactional document、durable save、dependency compile、last-good artifact和runtime一致preview闭环。已有projection只读且生产无consumer，无法编辑、提交或证明屏幕结果来自当前source revision。

本报告记录5个P0、60个P1、12个P2，给出M0-M7重构路线与32个验收门。目标是建立`RenderAuthoringDocument + Versioned Graph Schema + Semantic Compiler + Transaction Command + Derived Artifact + Isolated Preview Session`，不是继续扩充只注册ID、写固定字符串或生成不可执行近似WGSL的表面功能。本轮只做静态review，没有修改production代码。上一轮同一工作树的`zircon_editor --lib`测试编译在617.2秒后被239个既有test-build错误和122个warning阻断，本轮没有重复相同Cargo lane；67个test attributes只作静态inventory，不得表述为动态通过。

## 2. 审查边界与可复验证据

### 2.1 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Editor projection、registry、operation dispatch、Workbench/ZUI产品面 | 12 / 3,917 / 183,586 | E3代表链、E2全量inventory；fingerprint `07128120...f67fe92e` |
| Material Editor、WGSL importer及runtime material/shader/graph合同 | 24 / 3,414 / 125,764 | E3：registration、schema、validator/compiler、import与projection；fingerprint `be9e4e79...843f2f3e` |
| Particles editor/runtime/package | 57 / 8,733 / 306,329 | Editor E3、runtime接点E2：authoring声明、模板、simulation/render基础及tests；fingerprint `121a10a8...fd48254b` |
| Rendering Shader Graph / VFX Graph features | 16 / 592 / 19,711 | E3：schema/compiler/feature/executor/editor registration；fingerprint `9f88f84a...0794a0b8` |
| selected combined scope | 109 / 16,656 / 635,390 | 本轮取证时去重集合，67个test attributes、0 ignored；fingerprint `520c48ec...44c96c5` |

fingerprint按相对路径排序，将`path + NUL + per-file SHA-256 + LF`拼接后计算SHA-256。它只标识本轮证据集合，不是shader permutation、pipeline cache、DDC或cook key。

取证时上述109个source/asset/manifest/test文件没有未提交修改；工作树其余区域仍有大量并行变化。`source_recheck_required=false`只表示本轮没有已知的同范围在途冲突，不免除实施前重新取source、catalog、feature defaults和动态测试结果。

67个静态test attributes主要覆盖Material projection行生成、插件registration metadata、最小graph validator/compiler helper、Particles CPU/GPU/extract/registration及模板/control ID。它们没有覆盖默认Editor bootstrap、operation factory dispatch、真实document mutation/undo/save、graph source到runtime pipeline、live preview、shader/VFX GPU output、Nth-step failure、malformed/large graph或产品能力真实性。

### 2.2 证据等级与未覆盖边界

- E3：builtin asset registry、default first-party linked catalog、plugin registration、authoring contribution batch、operation dispatch失败分支逐函数闭环。
- E3：Material Graph source、import validator、plugin validator/compiler、Shader Graph compiler和VFX Graph compiler/executor逐字段对照。
- E3：Workbench ZUI action到module/extension feedback逐action核对，确认结果来源是静态文本而非compiler/job/runtime。
- E3：Material/RendererData projection到production consumer全局搜索；Particles operation、asset kind、importer和ZUI事件全局搜索。
- E3：Unreal Material Editor/Niagara、Godot Shader/Visual Shader/Particles、Fyrox Material/Particle、Unity ShaderGraph/VFXGraph对应源码对照。
- E2：Particles runtime只确认Editor必须消费的CPU/GPU基础与边界，完整simulation/render正确性不在本报告重复审查。
- E2：Bevy只作为runtime material、shader asset与render-resource分层参考，不是Editor graph/VFX UX基准。
- 未覆盖：真实GPU厂商/驱动编译、离线shader compiler farm、完整PBR reference image、复杂Niagara级simulation语义、移动/主机平台以及十万节点/百万粒子实测。它们进入验收门或Runtime09系列，不冒充已验证能力。

### 2.3 本轮追踪的生产链

1. builtin registry注册Material、MaterialGraph和Shader resource kind，但`builtin_toolkit()`只为UI类资产返回toolkit；没有插件时Asset Browser只能进入“No toolkit”路径。
2. default linked first-party editor catalog只按feature返回Navigation和Neural registration；Material、Particles、Shader Graph和VFX Graph不在默认linked catalog。外部native动态发现可能加载其他package，因此这里不推断“所有部署方式都绝对不可加载”。
3. Material Editor插件声明experimental/editor-only，注册Material/MaterialGraph toolkit、view、drawer、graph descriptor、palette、creation template和六个operations。
4. 其UI template定位`plugins://material_editor/editor/graph.zui`，creation template定位`plugins://material_editor/templates/default_material_graph.toml`；两份物理文件均不存在。
5. `EditorAuthoringContributionBatch`承载command descriptor、menu、asset type、graph/timeline等元数据，但不承载operation factory；publication调用`register_command`，不是`register_operation`。
6. operation dispatch在descriptor没有event且registry找不到factory时返回`MissingFactory`并记录control failure。六个Material operation和Particles authoring/preview operation只在descriptor/tests中出现。
7. `GraphEditorDescriptor`和`GraphNodePaletteDescriptor`进入registry/snapshot/query API，但production consumer搜索只落在声明、注册和其他插件producer，没有graph canvas/toolkit消费它们。
8. Particles插件的ZUI与CPU sprite template物理存在；但菜单明确`enabled(false)`，authoring/preview/drawer ZUI只含`Space`，没有event，`particles.system`也没有runtime importer/AssetKind消费。
9. Particles runtime asset是普通Rust struct，当前没有serde derive；模板TOML没有加载成该结构的执行链。
10. Core Material和VFX workspace以固定`M_Rock_Cliff`、`P_Bolt_01`、参数/曲线/warning填充界面。command feedback只把控件文字改成“Material compiled”“compile complete, 2 warnings”或“VFX simulation running”。
11. Extension Shader Editor和Particle Library同样通过固定feedback显示open/compile/preview/simulate queued及预设warning，没有job ticket、compiler report或runtime snapshot。
12. `MaterialGraphAsset` importer只做`validate_output_node()`，因此重复ID、multiple output、坏pin、悬空edge、cycle和non-finite仍可进入资产系统。
13. Material Editor validator比importer稍强，但仍不验证pin type/cardinality/topology；compiler只计算base color常量/texture，把其他Material字段写成defaults，且拒绝texture-backed Add/Multiply。
14. `zircon_runtime::graphics::shader::shader_assets`、runtime authoring asset和rendering Shader Graph feature各自定义不同Material/Shader Graph；没有schema version、migration或权威转换。
15. rendering Shader Graph按vector顺序直接拼WGSL，任意ID进入identifier，TextureSample调用未定义helper；missing output仍生成magenta函数。生成物不经过Naga validation，render executor是no-op。
16. VFX Graph compiler只检查`max_particles`、SpawnRate和ShaderGraphMaterial是否存在，返回固定simulation/transparent pass；两个render executor均为no-op，simulation dispatch也没有来自graph的真实workload计划。
17. WGSL importer是正确基础：通过Naga parse/validate并提取entry point。但导入后的dependency/source file/import/definition/property/texture/option/render contract大多为空或default，插件能力也明确标记partial。
18. `MaterialEditorProjection`和`RendererDataEditorProjection`能形成typed read-only rows与diagnostics；production消费者为零，只有tests调用。

## 3. 已有工程基础，重构时必须保留

### 3.1 Material与Shader合同

- `MaterialAsset`已有shader/parent、PBR参数、property values、texture slots、options、queue、diagnostics以及version 2 `.zmaterial`持久化和project reference转换，应成为authoring document与runtime artifact之间的稳定合同，而不是被简单graph compiler重新缩减。
- `ShaderAsset`已表达source/WGSL、entry point、source dependency、property schema、option、texture slot、shading model、render state、queue、disabled pass、resource/layout/generated WGSL/editor/pipeline layout与validation diagnostics，足以承接更严格的compiler结果。
- shader/material readiness和contract diagnostic已有typed基础；重构应补stable code、source location和revision，而不是继续用Workbench字符串另建一套“诊断”。

### 3.2 Import、projection与运行时基础

- WGSL importer使用Naga parser和validator，说明source validation无需从零实现；应把它提升为共享compiler stage并输出可定位diagnostic/source map。
- Material/RendererData projection保留了group、label、default/override、texture和diagnostic path/source，是构建typed inspector和renderer-data详情视图的起点。
- Particles runtime已有CPU/GPU simulation、extract、pool、RNG、buffer/layout/program/shader/transparent/readback与manager/service结构；Editor preview应通过受控gateway消费它，而不是另写假simulation。
- Runtime09C已经审查material/shader/pipeline/PSO合同；Editor不能另建不经过runtime compiler与pipeline cache的成功语义。

### 3.3 插件与产品接点

- asset type、toolkit、view/drawer、command/menu、graph descriptor、palette和creation template的注册模型已经存在，适合扩展owner lease、capability state和factory绑定。
- operation dispatch能够明确返回`MissingFactory`，可作为capability truth和bootstrap invariant的失败基础。
- Workbench已有Material、VFX、Shader和Particle导航位置，可以在真实toolkit完成后替换为document view；当前不应删除位置合同，只应撤销虚假成功反馈。

## 4. 目标架构

### 4.1 Authoring source、compiler与artifact分层

| 层 | 应持有内容 | 不得持有内容 |
|---|---|---|
| `RenderAuthoringDocument` | asset ID、source revision、stable element ID、typed property/node/edge/parameter、layout metadata、dependency reference | live GPU handle、UI control ID、固定status文本 |
| `RenderSchemaRegistry` | material domain、node/module descriptor、pin/property type、owner/version、migration、target capability | 当前document实例或selection |
| `RenderSemanticCompiler` | bounded parse、name/type/topology/reference/domain校验、canonical IR、stable diagnostics/source map | UI副作用、磁盘写入、preview world |
| `CompiledRenderArtifact` | source/dependency/options/target key、validated IR/WGSL、reflection/layout、pipeline recipe、debug map | mutable Editor draft |
| `RenderAuthoringTransaction` | document/base revision、typed before/after、merge key、dirty effect、compile request | 裸字符串operation和不可逆vector mutation |
| `RenderPreviewSession` | isolated scene/world、mesh/system subject、artifact generation、clock、camera、environment、runtime snapshot | authoritative source资产 |
| `RenderAuthoringProjection` | stable row/node/edge/diagnostic IDs、incremental diff、selection/focus/accessibility语义 | 从display string反向猜命令 |

Canonical pipeline必须是：

```text
bounded source bytes / imported source
  -> version dispatch + migration
  -> structural schema validation
  -> reference / pin / type / domain resolution
  -> canonical IR + deterministic diagnostics
  -> target-specific WGSL / simulation program / pipeline recipe
  -> Naga + backend validation
  -> immutable derived artifact keyed by source/dependencies/options/target
  -> isolated runtime preview + typed debug/profiling snapshot
```

任一阶段失败时保留last-good artifact，但UI必须明确显示“source revision invalid / previewing artifact generation N”，不能把旧画面、magenta fallback或固定文本当作当前source成功。

### 4.2 统一graph与module schema

Material Graph、Shader Graph和VFX Graph可以拥有不同domain，但必须共享稳定element identity、versioned envelope、owner-scoped descriptor、typed pin/cardinality、edge identity、default value、parameter/blackboard、layout metadata、unknown-node preservation、migration和diagnostic定位合同。领域compiler在共享structural layer之上处理material output、shader stage/target和VFX spawn/update/output context；不得继续维护三套同名但无法迁移的孤立struct。

Material instance不应复制完整parent材质。authoring document应记录显式override及parent revision，projection显示inherited/default/override origin，compiler按确定顺序解析parent chain、static option/permutation和texture/resource binding，并把结果连接Runtime09C的pipeline/PSO key。

### 4.3 Transaction、保存与依赖编译

所有property、texture、node、edge、parameter、module、curve、emitter和preview setting修改都形成可逆typed command；prepare阶段完成schema/reference/dirty external effect，commit阶段原子发布document revision、history和projection。保存必须基于validated snapshot，采用同目录temp、flush、atomic replace、CAS/external edit检测、LKG/autosave和import acknowledgement。

compiler由Editor09 background job拥有admission、dedup、priority、cancellation acknowledgement、progress、resource budget和shutdown fence。artifact key至少包含source revision、transitive dependency revisions、schema/plugin owner versions、compile options、target/backend/profile和compiler version。修改一个共享function/include/material parent时，dependency graph应精确失效消费者并保持未受影响artifact可用。

### 4.4 Preview与产品真实性

Material/Shader preview建立隔离scene、真实mesh、camera、environment、light、render path和pipeline；VFX/Particle preview建立隔离simulation world、seed、clock、bounds、event/collision source和真实runtime manager。play/pause/step/rewind/warmup必须驱动同一runtime实现，并返回artifact generation、frame/simulation counters、errors、GPU/CPU timing、draw/particle statistics和capture入口。

所有可见command只有三种合法状态：绑定了真实factory并能产生transaction/job/preview result；因capability/target/plugin状态disabled且给typed reason；或完全不出现在生产导航。禁止以更换control text模拟compile、save、preview、simulate或validate成功。

## 5. P0：产品不可达、能力过度声明或source到runtime合同断裂的问题

### P0-1 · 默认产品入口、插件装配和Material资源包合同不闭合

builtin registry不给Material/MaterialGraph/Shader toolkit，默认linked catalog又不包含Material/Particles/Shader Graph/VFX Graph。即便外部动态发现加载Material插件，它声明的graph ZUI和default graph template也不存在。必须建立默认/可选deployment的明确capability matrix，在同一registration transaction中验证asset type、toolkit、view、template、factory和physical resources；真实production bootstrap测试不得靠helper或假fixture补齐断路。

### P0-2 · Authoring operation只有descriptor，没有可执行factory或产品consumer

Material六个operation和Particles十余个authoring/preview operation没有factory；authoring contribution只发布command descriptor。Graph descriptor/palette没有canvas consumer，Particles菜单disabled且ZUI为空。必须让每个enabled operation在owner lease下绑定typed factory/document command，并由bootstrap invariant拒绝“visible + enabled + missing factory/template/schema”的插件publication。

### P0-3 · Material/VFX/Shader/Particle Workbench伪造编译、模拟和预览结果

Core/extension workspace以硬编码资产、warning和结果填充，callback只改字符串却显示compiled、running或queued。用户无法区分demo、请求已受理、编译完成与runtime输出。必须立即建立capability truth：未接真实job/runtime的action隐藏或disabled；状态必须来自带ticket/generation/provenance的typed result，并能跳转真实diagnostic。

### P0-4 · 三套Graph authority分裂，compiler输出不能证明可执行

runtime authoring graph、graphics shader assets和rendering feature各自定义Material/Shader Graph。import validator、plugin compiler、WGSL generator和VFX compiler规则互不一致；生成WGSL未经Naga复验，Shader/VFX executor还是no-op。必须硬切到versioned canonical schema、共享structural validator、领域semantic compiler和immutable artifact；import/save/preview/cook/runtime load消费同一diagnostic corpus与target validation。

### P0-5 · 没有transactional document、durable save和runtime一致preview闭环

已有projection只读且生产无consumer；Material/VFX/Particle没有authoritative Editor session、history、dirty、autosave、dependency compile或preview generation。即使helper返回MaterialAsset，也没有产品路径保存并展示其真实pipeline结果。必须接入Editor02 transaction/save和Editor09 job authority，以Runtime09C compiler/pipeline及Particles runtime为唯一preview执行源；完成前不得公开“完整Material/VFX Editor”能力。

## 6. P1：工程级完整性差距

### 6.1 产品装配、toolkit、document lifecycle与capability

1. Material、MaterialGraph和Shader虽进入builtin resource registry，却没有内建只读详情/preview fallback；插件缺失时只得到“No toolkit”。
2. default first-party linked catalog只拥有Navigation/Neural feature，Material/Particles/Shader/VFX的发行装配、依赖和默认profile没有可验证矩阵。
3. Material插件引用的`editor/graph.zui`和`templates/default_material_graph.toml`缺失，registration不会在publication前验证URI可解析和资源可读。
4. Material dist registration的`invoke_command`为None且bridge methods为空，descriptor没有native command fallback或远程authoring能力。
5. authoring contribution batch没有operation factory、document provider、compiler provider或preview provider字段，command metadata与执行所有权分离。
6. Graph editor/palette snapshot只有query API，没有生产graph canvas/toolkit consumer、selection owner或command route。
7. Particles authoring/preview菜单显式disabled，但asset type/view/operations仍被注册，capability state没有统一解释“声明存在但不可用”。
8. Particles authoring、preview和drawer ZUI只含`Space`且无event，测试只验证control/template ID，不验证交互或data binding。
9. `particles.system`仅存在于Editor常量/测试；runtime没有对应AssetKind/importer，asset browser创建结果无法解析为runtime particle system。
10. Material/Shader/VFX/Particle没有统一document ID、source revision、dirty state、history context、autosave payload或close decision。
11. 插件unload/reload没有定义open document、unknown node/module、last-good artifact、preview session和unsaved source的迁移/保持策略。
12. experimental/partial capability只在manifest/descriptor层出现，产品导航没有按maturity、target、backend、factory和resource readiness动态disable或解释原因。

### 6.2 Material、Material Instance与property authoring

13. `MaterialEditorProjection`只生成read-only rows，production无consumer；没有从stable row ID到typed command的反向编辑合同。
14. property row的kind、group、label、default和override已有数据，但没有editor widget factory、range/unit/color/enum/resource picker或validation commit policy。
15. override只能被投影为布尔状态；没有enable/reset/revert/copy/paste override、multi-edit mixed value和origin breadcrumb。
16. parent material字段存在，但Editor没有parent chain、cycle、missing parent、revision pin、inherited value diff和reparent transaction。
17. standard PBR、render queue、shading model、blend/cull/depth/stencil等选项没有typed authoring、domain constraint或target capability提示。
18. texture slot没有drag/drop、sampler/color-space/normal-map语义、fallback、streaming residency、thumbnail、missing reference和批量替换workflow。
19. Material选择Shader时没有基于`ShaderAsset`property/texture/option schema增量重建Inspector，也没有保留/迁移兼容override。
20. shader contract diagnostics虽可投影，但没有stable diagnostic code、source/material property定位、quick fix、suppress policy或revision provenance。
21. Material Graph只有Output、TextureSample、Scalar/VectorParameter、Add和Multiply六种node，无法表达已有MaterialAsset绝大多数PBR和render option字段。
22. plugin validator只检查部分duplicate/output/link/input，不检查pin name/type/cardinality、multiple incoming、cycle、reachability、finite value、resource/domain或shader schema。
23. Material compiler只求值base color，并把metallic/roughness/normal/emissive/alpha/options等写为固定default；产物不能等价表达source graph。
24. Add/Multiply遇到texture-backed input直接报错，palette却把pin都声明为`float`，而compiler又允许scalar/vector广播到base color `vec4`，descriptor与语义不一致。
25. compiler不生成canonical shader IR、WGSL、reflection/layout、variant/permutation、pipeline recipe、artifact key或source debug map，也不连接Runtime09C的PSO authority。

### 6.3 Graph schema、Shader authoring与compiler diagnostics

26. 仓内三套Material/Shader Graph struct同名不同义，缺少单一schema owner、版本、migration、conversion和deprecation路线。
27. graph node/edge只用自由字符串ID和pin名，没有stable UUID/element ID、edge identity、namespace、schema version或unknown node opaque payload。
28. builtin authoring importer只验证“至少一个Output”，与Material插件“恰好一个Output”等规则冲突，坏graph可先进入catalog再在另一层失败。
29. 没有共享pin type、conversion、cardinality、default value和domain/stage规则；多个incoming、非法swizzle、texture/sampler/resource binding都无法在source层判定。
30. palette将Add/Multiply pin声明为`float`，compiler却以vec4计算并允许scalar splat，UI连接合法性无法预测compiler行为。
31. 没有cycle detection、topological order、reachability/dead node、constant folding边界、side-effect ordering和deterministic diagnostic order。
32. Shader Graph generator直接把任意node ID拼入WGSL identifier，没有identifier sanitize、collision table、source element map或恶意/Unicode输入规则。
33. TextureSample生成对`zircon_sample_texture_{binding}`的调用，但compiler不定义helper、binding declaration或resource layout，字符串生成物不是自包含shader module。
34. 生成WGSL不经过Naga parse/validate和target backend validation，Editor无法保证展示的“compile”结果甚至是合法WGSL。
35. 缺Output时generator仍发出magenta返回并同时记录diagnostic；artifact validity、fallback用途和runtime admission没有区分，错误source可能继续被当作可运行结果。
36. WGSL importer虽真实验证源码，却把dependency/source file/import/definition/property schema/options/texture slots及多数render contract置空/default，无法完成工程级shader reflection。
37. Editor没有Shader text document、incremental parse、syntax/semantic completion、stage/entry选择、format、find/replace、include navigation和unsaved buffer compile。
38. Naga/import错误没有进入Editor统一的stable code、file/line/column/span、include stack、target、revision和quick-fix projection。
39. 没有include/source dependency graph、macro/definition/profile authoring、external file watch/CAS、generated-source查看和source map跳转。
40. 没有target/subtarget、quality/platform/backend permutation、keyword/static option预算、variant stripping/usage和compile matrix可视化。

### 6.4 VFX Graph、Particle System与runtime bridge

41. Particles create/add/open/emitter/module/curve/validate/play/pause/stop/rewind/warmup operation只有descriptor，没有factory、handler或typed payload decoder。
42. runtime `ParticleSystemAsset`/`ParticleEmitterAsset`没有serde合同，Editor template与runtime对象之间不存在versioned load/save/migration链。
43. `cpu_sprite_system.toml`物理存在但没有importer/creation consumer；创建测试只断言template URI，不证明能产生可运行asset。
44. Editor没有真实emitter/module stack、spawn/update/output context、parameter/attribute blackboard、curve/gradient editor、renderer/bounds/LOD authoring。
45. preview ZUI没有playback事件或viewport；warmup、rewind和play没有isolated world、seed、clock、camera、collision/event source及generation隔离。
46. CPU/GPU runtime基础没有通过Editor gateway暴露simulation mode、capacity、buffer/readback、fallback、diagnostic和live stats，authoring无法验证两种backend parity。
47. VFX Graph schema只有SpawnRate、InitialVelocity、Gravity和ShaderGraphMaterial；compiler只检查三个存在性条件并返回固定pass名，没有typed attributes、contexts、data flow或program IR。
48. VFX simulation/transparent executor均为no-op，dispatch不由graph、capacity或workgroup计划推导；注册render feature不能证明产生任何粒子或像素。
49. rendering Shader Graph/VFX Graph editor crate只注册plugin descriptor/capability，没有authoring extensions；feature又是optional且默认关闭，Editor声明与runtime可用性未绑定。
50. 没有event/collision/ribbon/mesh/decal/light/sub-emitter、GPU event/readback、deterministic seed、bounds/culling、LOD/scalability和module migration的authoring合同。

### 6.5 Preview、jobs、diagnostics、性能与测试

51. Material/Shader没有隔离preview scene、mesh/LOD、camera、environment/light、render path、platform/quality和reference image控制。
52. compile失败没有last-good artifact/generation提示；固定magenta或旧画面可能被误认为当前source结果。
53. Material/Shader/VFX compile不进入background job admission、dedup、cancellation acknowledgement、progress、resource quota或shutdown fence。
54. 没有基于source/dependency/schema/plugin/compiler/target/options的DDC key和transitive invalidation，批量编辑将无法可靠复用artifact。
55. `RendererDataEditorProjection`没有live renderer/pipeline/PSO/runtime snapshot consumer，不能证明stage/feature诊断对应当前preview generation。
56. diagnostic没有从WGSL/backend/pipeline/simulation错误映射到stable property/node/edge/module/curve元素，也没有跨paneselection和jump owner。
57. 没有展示shader instruction/resource/variant、pipeline creation、compile duration/cache hit、particle CPU/GPU time、alive/spawn/drop和overdraw等工程统计。
58. Material/Particles tests主要证明descriptor、字符串、模板和最小helper；没有默认bootstrap、operation dispatch、document transaction/save和product projection测试。
59. 没有graph fuzz/malformed corpus、Nth-step failure、plugin unload、driver/device loss、large graph/variant/particle soak、真实GPU像素或simulation parity证据。
60. graph/canvas/curve/preview缺keyboard/focus/screen-reader/high-contrast/reduced-motion和i18n；可见状态又大量硬编码英文。

## 7. P2：不阻断正确性但影响成熟度的差距

1. 缺少per-user graph grid、wire style、node density、preview background和lighting rig偏好。
2. 缺少可命名的Material preview camera、environment、mesh、VFX seed/time和debug bookmark。
3. 缺少recent material/shader/VFX asset、最近element和跨document navigation history。
4. 缺少node minimap、edge routing、reroute、sticky note、frame/group、alignment和distribution工具。
5. 缺少property/node/module/curve的批量标签、颜色、comment和review note。
6. 缺少两个material/shader/VFX revision的结构diff、generated source diff和preview image overlay。
7. 缺少将source、compiler diagnostics、artifact manifest、GPU capture引用导出为bounded support bundle。
8. 缺少opt-in authoring telemetry摘要，包括compile latency、cache hit、preview FPS和particle budget趋势。
9. 缺少批量shader compile、material reparent/override迁移、particle validation和命令行dry-run报告。
10. 缺少可共享的material function、subgraph/module preset、template library和依赖使用查询。
11. 缺少Editor scripting/remote automation的typed Material/Shader/VFX command与query surface。
12. 缺少多人协作下的asset lock、元素级冲突提示、review-only与revision annotation。

## 8. 参考引擎对照

| 参考 | 逐源码确认的原则 | Zircon当前差距 | 采用边界 |
|---|---|---|---|
| Unreal Material Editor / Material Instance Editor | Material Editor拥有preview material/viewport、graph actions、apply/compile、validation、stats、find、diff和实例详情；独立文件覆盖MaterialGraph、MaterialStats、MaterialInstance与viewport | Zircon只有read-only projection、缺失plugin UI/template、descriptor-only operation和常量base-color compiler | 吸收toolkit分层、preview/apply、实例override、stats/diagnostic/diff原则，不复制UObject/Slate结构 |
| Unreal Niagara Editor | 编译器、HLSL translator、graph digest、message manager、debugger、baker、system/emitter editor data与stack/parameter authoring分层 | Zircon VFX compiler只返回固定pass，executor no-op，Particle Editor没有module stack/preview command | 吸收source/digest/compiler/message/debug/bake分层及system/emitter/context模型，不把Niagara所有节点作为M0前置 |
| Godot Shader / Visual Shader / Particles | text shader editor处理错误、completion与include；Visual Shader有专用graph plugin；Particles提供restart、convert/generate等真实动作并通过UndoRedo提交场景变化 | Zircon没有shader document/graph consumer，Particles操作未绑定factory且ZUI为空 | 吸收真实动作可撤销、text/visual shader共用runtime语义、particle工具连接场景资源原则 |
| Fyrox Material / Particle | Material plugin、property editor和scene material commands分离，material command可execute/revert；particle editor连接实际scene/node数据 | Zircon projection与transaction断开，Particles asset/template与runtime不相通 | 吸收command inverse、property editor integration和runtime resource绑定原则 |
| Unity ShaderGraph | `GraphData`、validation、generator、GraphEditorView、target/subtarget和blackboard形成versioned graph到generated shader的完整层次 | Zircon graph schema分裂、pin/type/topology不足，且生成WGSL未经validation | 吸收graph model/validation/generation/target/UI分层；不复制Unity serialization和UIElements细节 |
| Unity VFXGraph | VFXGraph/VFXModel、data/context、blackboard、validation、undo controller、compiled data和compiler passes独立 | Zircon只有四类node、固定pass和no-op executor，没有context/data/undo/compiler artifact | 吸收context/data/model/controller/compiler结构及undo/validation原则 |
| Bevy render/PBR | Material/Shader asset、render resource、pipeline specialization与runtime execution边界清晰 | Zircon Editor helper output未连接runtime pipeline/PSO authority | 只作为runtime合同和specialization参考；Bevy不是本轮Editor UX基准 |

共同结论不是“复制某个节点界面”，而是：source schema、transaction、semantic compiler、target artifact、runtime preview和typed diagnostics必须各有单一权威；所有可见能力必须能沿这条链追到真实数据与执行结果。

## 9. 重构里程碑

### M0 · Capability Truth、产品入口与插件资源硬门

- 建立Material/Shader/VFX/Particle deployment capability matrix和production bootstrap测试。
- publication前验证toolkit、template、asset type、factory、compiler/preview provider及physical resource URI。
- 隐藏/disable所有静态feedback和missing factory action，状态改为typed unavailable reason。
- 明确builtin fallback与可选插件边界，插件缺失时提供诚实的只读详情或不可用诊断。

### M1 · Canonical Schema Registry 与 Versioned Authoring Document

- 收敛三套Material/Shader Graph模型，定义共享stable element/edge identity、typed pin和versioned envelope。
- 建立material domain、shader target、VFX context/module descriptor registry及owner lease/migration。
- 建立Material/Shader/VFX/Particle document ID、source revision、dirty、autosave、unknown element preservation。
- 定义Material instance parent/override origin和Particle system/emitter/module serialization。

### M2 · Semantic Compiler、Target Artifact 与 Diagnostics

- 实现bounded parse、structural/type/topology/reference/domain validation和deterministic diagnostics。
- Material/Shader输出canonical IR、WGSL、reflection/layout、variant/pipeline recipe并经Naga/backend validation。
- VFX/Particle输出真实simulation/render program、dispatch/buffer计划和runtime resource合同。
- import、save、preview、cook与runtime load共享compiler corpus、artifact key和source debug map。

### M3 · Transaction、Durable Save、Dependency Graph 与 Jobs

- 所有property/node/edge/module/curve操作迁移为可逆typed command和history context。
- 实现validated snapshot、atomic replace、CAS/external edit、LKG、autosave恢复和import acknowledgement。
- compiler接入Editor09 job authority、DDC、transitive invalidation、cancel/progress/quota/shutdown。
- plugin unload/reload保持document opaque data、last-good artifact和明确stale generation。

### M4 · Material、Instance、Shader Text/Graph 完整authoring

- 将现有projection接入typed Inspector，完成property/texture/option/parent/override编辑与multi-edit。
- 实现graph canvas、palette、typed pins、blackboard、subgraph/function、layout和diagnostic navigation。
- 实现Shader text editor、include/dependency、completion、entry/target/profile/permutation和generated source查看。
- 建立真实Material preview、compile stats、variant/pipeline/PSO状态和source-to-pixel验证。

### M5 · VFX Graph、Particle System、Curve 与 Runtime Preview

- 完成system/emitter/context/module/attribute/parameter/curve/gradient/renderer/bounds/LOD authoring。
- 将Particles CPU/GPU runtime和VFX compiler连接隔离preview world、clock、seed、collision/event及warmup/rewind。
- 实现simulation program、dispatch/buffer、renderer pass和shader/material dependency，不允许no-op executor进入可用capability。
- 显示alive/spawn/drop/bounds/CPU/GPU timing、overdraw和backend parity diagnostics。

### M6 · Product Integration、Diagnostics、性能与可访问性

- Workbench替换为真实document/toolkit projection和ticketed job/preview状态。
- 接入Editor11 diagnostic journal、jump/source map、quick fix、artifact manifest与GPU capture引用。
- 建立大graph、variant explosion、批量compile、百万粒子和malformed/fuzz基准及budgets。
- 完成keyboard/focus/screen reader/i18n/high contrast/reduced motion和虚拟化大列表/canvas。

### M7 · 兼容、故障、跨平台与发布门

- 建立schema/plugin node/module migration、unknown placeholder和cross-version golden。
- 覆盖插件缺失/重载、磁盘满、外部编辑、cancel race、device loss、driver reject与crash recovery。
- 对目标backend/profile执行shader compile matrix、reference image和particle simulation/render parity。
- 所有production capability通过默认bootstrap与end-to-end资产闭环后，才提升maturity并进入发行profile。

## 10. 验收门

- [ ] 1. 默认production bootstrap能按声明打开Material/MaterialGraph/Shader；未安装可选插件时给出typed capability reason，不依赖test helper。
- [ ] 2. plugin publication在缺factory、template、schema、compiler/preview provider或physical resource时原子失败，不留下半注册entry。
- [ ] 3. Material插件的graph/template URI均有真实可加载资源、版本与owner provenance；卸载后open document不丢source。
- [ ] 4. 每个visible/enabled Material/VFX/Particle/Shader command都有可执行factory，并产生transaction、job或preview result。
- [ ] 5. Workbench不再通过固定字符串宣称compile/simulate/preview成功；所有状态带ticket、generation、revision和provenance。
- [ ] 6. canonical graph loader在明确budget内拒绝oversized、深层、截断、未知版本和malformed source，无panic/OOM。
- [ ] 7. 三套旧graph schema有明确migration或拒绝diagnostic，迁移保持stable element、layout、opaque unknown data和reference。
- [ ] 8. structural compiler拒绝空/重复ID、悬空edge、坏pin、cardinality、cycle、unreachable required node、non-finite和invalid domain。
- [ ] 9. pin type/conversion与palette、connection UI、compiler完全一致，scalar/vector/texture/sampler规则有golden corpus。
- [ ] 10. Material compiler覆盖MaterialAsset全部声明字段或明确unsupported diagnostic，不再静默写固定default。
- [ ] 11. generated WGSL自包含、identifier安全、resource layout完整，并通过Naga与每个required backend/profile验证。
- [ ] 12. invalid shader不会生成可admit artifact；fallback/last-good有独立状态且UI明确标示source/artifact generation差异。
- [ ] 13. shader text editor支持unsaved buffer compile、include/source map和file/line/column diagnostic，跳转定位准确。
- [ ] 14. shader target/subtarget/entry/option/permutation变化确定性进入artifact/PSO key并触发精确失效。
- [ ] 15. Material parent/override增删、reparent和shader schema变化可undo/redo，并正确保留/迁移兼容override。
- [ ] 16. property/texture/option编辑失败时document/history/dirty/projection/artifact均不改变；成功时单revision原子提交。
- [ ] 17. node/edge/parameter/layout操作undo/redo恢复完全相同的source bytes、selection、diagnostics与compile generation。
- [ ] 18. save使用validated revision snapshot、durable atomic replace和CAS；磁盘满/外部编辑/import失败不静默标clean。
- [ ] 19. dependency/include/function/parent/plugin schema变化精确失效消费者，未受影响artifact保持cache hit。
- [ ] 20. compile job支持dedup、priority、progress、cancel acknowledgement、quota与bounded shutdown，无stale result覆盖新revision。
- [ ] 21. Material/Shader preview使用真实runtime pipeline/PSO并显示artifact generation、target、compile/pipeline diagnostics。
- [ ] 22. reference material场景在required backend/profile下通过image golden、容差、GPU validation和capture provenance。
- [ ] 23. `particles.system`具有versioned importer/save/migration，CPU sprite template能创建、重新打开并运行真实asset。
- [ ] 24. Particle/VFX Editor可编辑emitter/context/module/attribute/parameter/curve/renderer/bounds/LOD并全部可undo/redo。
- [ ] 25. VFX compiler产生非空simulation/render program、dispatch和buffer计划；enabled executor实际更新粒子并输出像素。
- [ ] 26. play/pause/step/stop/rewind/warmup驱动隔离runtime preview，固定seed/clock下CPU/GPU结果满足声明parity。
- [ ] 27. preview显示alive/spawn/drop/bounds、CPU/GPU timing、buffer/dispatch和overdraw，device loss/backend fallback有typed状态。
- [ ] 28. plugin unload/reload、unknown node/module、missing dependency和schema migration保留source并提供可恢复workflow。
- [ ] 29. 10万node/edge、万级variant、批量compile和百万粒子场景有声明内存/延迟/frame budgets及可复现benchmark。
- [ ] 30. fuzz/malformed、Nth-step failure、cancel race、磁盘/crash/device loss和driver reject均有failure-injection/golden证据。
- [ ] 31. Material/Shader/VFX/Particle toolkit完成keyboard/focus/screen-reader/i18n/high-contrast/reduced-motion验收。
- [ ] 32. tests同时覆盖default production registry、真实operation dispatch、document/save/compiler/preview端到端；禁止descriptor/string/template-only false-green。

## 11. 非目标与边界

- 本报告不重复实现Runtime09A/09C拥有的RHI、Render Graph、GPU lifetime、material/shader/pipeline/PSO内部；Editor要求消费并验证其真实合同。
- 本报告不把Particles runtime已有CPU/GPU基础判为零，也不在本轮证明其所有算法正确；这里只审查authoring、serialization、preview与产品桥接。
- 本报告不要求M0-M3一次达到Niagara全部module或Unity VFXGraph全部context规模，但要求schema/compiler可扩展且当前能力声明真实。
- 本报告不要求复制Unreal/Unity/Godot/Fyrox界面；要求复制的是transaction、compiler、artifact、preview、diagnostic和capability truth工程原则。
- 本报告不把67个静态test attributes当成动态通过，也不因当前Editor test-build基线失败而降低验收标准。
- 本轮只做review和重构计划，不修改production代码、tests、asset schema、plugin manifest或ZUI。

## 12. 完成定义

Material/Shader/VFX/Particle Editor只有在以下事实同时成立时才能从“注册表/投影/运行时基础与UI prototype”提升为“工程级authoring”：默认入口与插件装配真实可达；所有可见操作有factory且可撤销、可保存；source由单一versioned schema和共享compiler形成目标artifact；diagnostic可定位、job可取消、依赖可失效；preview执行真实runtime pipeline/simulation并标识generation；插件、迁移、故障、大资产、跨平台和可访问性都有可复验证据。在此之前，必须把当前Material/VFX/Shader/Particle工作区和插件描述为实验性骨架，不能当作完整制作套件。
