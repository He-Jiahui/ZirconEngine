---
title: Editor Material、Material Instance、Shader/Material Graph、Toolkit、Preview 与 Compiler 当前工作树工程化差距
category: zircon_editor
report_id: Editor249
review_date: 2026-08-30
baseline_head: working-tree
related_code:
  - zircon_plugins/material_editor/plugin.toml
  - zircon_plugins/material_editor/editor/src
  - zircon_plugins/material_editor/dist/src
  - zircon_plugins/rendering/features/shader_graph/editor/src
  - zircon_plugins/rendering/features/shader_graph/runtime/src
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/material
  - zircon_runtime/src/asset/assets/shader
  - zircon_editor/src/core/asset
  - zircon_editor/src/core/editing/operation
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/commands
plan_sources:
  - docs/plans/optimize/zircon_editor/15-material-shader-graph-instance-vfx-particle-preview-compiler-diagnostics-authoring-review.md
  - docs/plans/optimize/zircon_editor/137-editor-material-shader-graph-material-instance-vfx-particle-preview-compiler-diagnostics-authoring-current-source-review.md
  - docs/plans/optimize/zircon_editor/248-editor-asset-workspace-catalog-provider-preview-import-reimport-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/189-runtime-material-shader-artifact-variant-pipeline-pso-cache-publication-current-working-tree-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/MaterialEditor/Private/MaterialGraph/MaterialGraphSchema.cpp
  - dev/UnrealEngine/Engine/Source/Editor/MaterialEditor/Private/MaterialEditor.cpp
  - dev/UnrealEngine/Engine/Source/Editor/MaterialEditor/Private/MaterialInstanceEditor.cpp
  - dev/UnrealEngine/Engine/Source/Editor/MaterialEditor/Private/MaterialStats.cpp
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Data/Graphs/GraphData.cs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Generation/Processors/Generator.cs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Drawing/MaterialGraphPreviewGenerator.cs
  - dev/godot/scene/resources/material.h
  - dev/godot/servers/rendering/shader_language.h
  - dev/bevy/crates/bevy_render/src/render_resource/pipeline_cache.rs
  - dev/Fyrox/fyrox-material/src/lib.rs
  - dev/Fyrox/editor/src/scene/commands/material.rs
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
tooling_scope: excluded_by_user_request
---

# Editor Material、Material Instance、Shader/Material Graph、Toolkit、Preview 与 Compiler 当前工作树工程化差距

## 1. 结论

主 Editor 已经具备可复用的 AssetTypeRegistry、toolkit descriptor、GraphEditorDescriptor、operation descriptor/factory registry、document/save/job/diagnostic等通用底座；Material 插件也确实注册了 Material/MaterialGraph toolkit、六条operation path和六类palette node，runtime `MaterialGraphAsset`有最小validator与constant evaluator。因此这里不是“完全没有代码”。

但产品链仍不可执行。`zircon_plugins/material_editor`与optional Shader Graph选集只有17个Rust/TOML文件、1,483行、约57.7 KiB；Material插件引用的`plugins://material_editor/editor/graph.zui`和`plugins://material_editor/templates/default_material_graph.toml`在包内物理不存在。六个operation只有descriptor，没有`OperationCommandFactoryRegistration`；native dist明确`is_stateless: true`、`state_schema_version: 0`、`invoke_command: None`、`bridge_methods: []`。Material Graph compiler只把base color常量或单张texture折叠到传统`MaterialAsset`，其余PBR字段全部默认。另一个`rendering.shader_graph`又声明第二套非序列化graph DTO，按Vec顺序拼接未消毒WGSL，TextureSample调用未定义helper，render pass executor永远`Ok(())`且不编码任何命令。

所以 Editor137 的五项父 P0仍全部Open：默认产品入口与package完整性、descriptor-only operation、fixed/false success、graph authority分裂、transactional document/save/runtime preview均未关闭。Editor249不重复P0，登记34项P1（30 Open / 4 Partial）、12项P2和26个资格门（22 Fail / 4 Partial / 0 Pass），并把实现边界收敛为单一Material Authoring产品，而不是继续给两个graph模型各补一层UI。

## 2. 审查范围与证据边界

### 2.1 当前owner链

| 层 | 关键文件 | 当前职责 | 本轮核验 |
|---|---|---|---|
| Package/registration | `material_editor/{plugin.toml,editor/src/plugin.rs,dist/src/lib.rs}` | capability、surface、toolkit、command、native entry | package资源、factory、bridge、state、unload、默认可达性 |
| Source schema/compiler | `material_editor/editor/src/lib.rs`、Runtime `asset/assets/authoring.rs` | graph DTO、validation、constant evaluation | ID、pin/type/topology、version、artifact、diagnostic |
| Duplicate feature | `rendering/features/shader_graph/{editor,runtime}` | optional capability、second graph/WGSL/pass | schema authority、生成安全、runtime执行、false-ready |
| Shared Editor host | `core/asset`、`core/editor_extension.rs`、`core/commands`、`core/editing/operation` | type/toolkit/graph/operation factory infrastructure | 是否有Material production consumer、是否可撤销/保存/恢复 |
| Runtime contract | Material/Shader asset、Runtime189 artifact/publication | typed property、shader ABI、generation/LKG/PSO | Editor输出是否可被Runtime证明安装和预览 |

### 2.2 证据等级

- **E3**：当前工作树逐文件读取，验证URI物理存在性、operation factory、native bridge和runtime executor。
- **E2**：对照Editor137/Runtime09C/Runtime189与本地Unreal、Unity Graphics、Godot、Bevy、Fyrox源码。
- **E1**：测试和descriptor只证明注册意图；本轮未运行Editor host、Cargo、UI automation、preview、WGPU或save/reopen E2E。
- **E0**：不能据此宣称Material Editor可用，更不能宣称与Unreal/Unity的工作流、诊断或性能相当。

## 3. 当前可保留底座

1. Core operation registry已经能注册`OperationCommandFactoryRegistration`并从descriptor路由到typed command；Material必须消费这条链，不需要再发明私有dispatch。
2. AssetTypeRegistry能够augment `ResourceKind::Material`/`MaterialGraph`并挂toolkit/creation template；正确修复是让package publication验证资源和factory后原子生效。
3. `MaterialGraphIndex`一次建立node/incoming-link索引，validator能发现空/重复ID、缺失node、多incoming、缺required input，compiler能检测递归cycle；这些可作为canonical graph validator的早期测试资产。
4. Runtime Material/Shader已有typed schema、Naga reflection、candidate/LKG和pipeline admission底座；Editor不应复制GPU/compiler authority，只应提交versioned source并消费artifact receipt。
5. Editor共享document、history、save/CAS/autosave、job、diagnostic和PreviewScene基础已经存在；缺口是领域adapter和真实consumer，而不是底层类型完全缺失。

## 4. 父 P0 当前重判

| Canonical owner | 状态 | 当前证据与硬切要求 |
|---|---|---|
| `Editor15-P0-01` 默认产品入口、插件装配与资源包不闭合 | Open | 两个Material URI资源物理缺失；默认catalog/App readiness也不能原子证明package/resource/factory/backend。 |
| `Editor15-P0-02` authoring operation只有descriptor | Open | Core factory能力真实存在，但Material六条operation与ShaderGraph均没有factory/handler，dist也无invoke bridge。 |
| `Editor15-P0-03` Workbench/反馈伪造compile、preview、save成功 | Open | 可见命令与静态文案没有typed compiler/runtime/save receipt；所有成功状态必须来自实际owner。 |
| `Editor15-P0-04` graph authority分裂且输出不可证明执行 | Open | `MaterialGraphAsset`与`ShaderGraphAsset`继续并存；一个只折叠base color，另一个拼接坏WGSL且executor no-op。 |
| `Editor15-P0-05` 无transactional document、durable save与runtime一致preview | Open | shared底座存在但Material领域consumer为零；没有DocumentId/revision/history/savepoint/artifact/LKG/preview generation闭环。 |

## 5. P1 工程化差距（34 项）

### 5.1 Package、registration 与产品可达性

| ID | 状态 | 当前证据与差距 | 必须重构为 |
|---|---|---|---|
| EDM4-P1-001 | Open | Material Editor未形成默认产品catalog/App可达性，capability存在不能证明可打开。 | first-party catalog/profile声明package、feature maturity、dependencies和明确Unavailable reason。 |
| EDM4-P1-002 | Open | `graph.zui`和`default_material_graph.toml`物理缺失，registry mutation前不验证URI。 | package activation先解析所有resource/template/schema/factory，再一次性发布或全部拒绝。 |
| EDM4-P1-003 | Partial | Core operation factory链完整，Material只注册六个descriptor。 | 每条open/create/validate/compile/preview注册typed factory、preflight、apply/result和stable receipt。 |
| EDM4-P1-004 | Open | native dist无`invoke_command`、state、restore、unload和bridge method。 | dist具备命令bridge、state schema、session save/restore、revoke/unload drain，或不宣称native完整形态。 |
| EDM4-P1-005 | Open | package声明stateless，但Material document/preview/selection/compile job天然有状态。 | state归Document/Session owner，dist只持可序列化handle；reload后能恢复或明确降级。 |
| EDM4-P1-006 | Open | optional Shader Graph editor只实现descriptor，不注册surface/toolkit/palette/operation。 | 在single-schema hard cut中吸收或删除；不可继续作为独立“已注册feature”。 |

### 5.2 Graph schema、编辑语义与transaction

| ID | 状态 | 当前证据与差距 | 必须重构为 |
|---|---|---|---|
| EDM4-P1-007 | Open | Runtime authoring MaterialGraph和rendering ShaderGraph是两套不兼容DTO。 | 一个versioned `MaterialGraphDocument`，target/domain/stage/function/subgraph均由同一schema表达。 |
| EDM4-P1-008 | Open | `MaterialGraphAsset`没有schema version、document ID、source revision、compiler/profile。 | header携带schema、document GUID、revision、target、compiler settings、migration history。 |
| EDM4-P1-009 | Open | node/link/pin均是任意String；rename/copy/paste/merge后identity不稳定。 | stable node/pin/link GUID，display name与symbol分离，serialization顺序不参与identity。 |
| EDM4-P1-010 | Open | palette仅Output/Texture/Scalar/Vector/Add/Multiply，Output只有base_color。 | 由Runtime material domain/schema生成完整stage/domain/output/node/function palette与capability过滤。 |
| EDM4-P1-011 | Open | palette pin type、validator和compiler各自写规则；Add/Multiply descriptor甚至只声明float。 | 共享type lattice、conversion、resource dimension、stage availability和required/default规则。 |
| EDM4-P1-012 | Open | 连接时没有production schema handler做方向/type/cycle/cardinality检查。 | `CanConnect`/`ConnectCommand`共享compiler validator，非法连接在transaction apply前拒绝并定位。 |
| EDM4-P1-013 | Open | node/link/position/parameter修改没有Material领域command/history。 | 每种编辑都是reversible command，包含selection/focus/diagnostic invalidation与revision receipt。 |
| EDM4-P1-014 | Open | graph没有function/subgraph、reroute、comment/group、parameter collection、static switch或custom code边界。 | 先定义模块/函数调用与dependency contract，再扩node；禁止用字符串include绕过schema。 |

### 5.3 Compiler、artifact 与diagnostic

| ID | 状态 | 当前证据与差距 | 必须重构为 |
|---|---|---|---|
| EDM4-P1-015 | Partial | `compile_material_graph`能验证后产出最小MaterialAsset，但只处理base color。 | compiler产出canonical Runtime189 source/program request，覆盖完整material domain与typed parameters。 |
| EDM4-P1-016 | Open | texture-backed Add/Multiply直接返回“v1不能组合”，不是shader编译。 | 构建typed IR/SSA或expression DAG，texture/math在shader中执行，constant folding只是优化。 |
| EDM4-P1-017 | Open | canonical MaterialGraph不生成WGSL/program artifact，无法证明graph语义被GPU执行。 | graph -> typed IR -> target code -> reflection -> artifact receipt；MaterialAsset只引用artifact/schema。 |
| EDM4-P1-018 | Open | duplicate ShaderGraph按Vec顺序发出let，任意ID直接成为WGSL identifier。 | canonical topological schedule + symbol table + sanitize/collision/reserved word + deterministic output。 |
| EDM4-P1-019 | Open | TextureSample生成`zircon_sample_texture_{binding}()`，没有helper、binding或sampler声明。 | resource allocator生成完整texture/sampler/bind-group layout并与reflection对拍。 |
| EDM4-P1-020 | Open | missing output仍生成magenta函数；MaterialOutput把roughness塞进alpha，语义与Runtime material ABI不一致。 | invalid graph不得发布admittable artifact；diagnostic preview/error bundle与真实output ABI分离。 |
| EDM4-P1-021 | Open | 两个graph compiler都不调用Naga/WGPU validation。 | Runtime189 compiler service是唯一publication gate，Editor只投影job与diagnostic。 |
| EDM4-P1-022 | Open | compile report只有`Vec<String>`和optional material/WGSL，无artifact ID、source map、dependency、target或generation。 | typed `MaterialCompileReceipt`携带request/source/artifact/install/LKG generation、provenance和cache outcome。 |
| EDM4-P1-023 | Open | graph diagnostic无stable code、node/pin/span/include stack、target/profile。 | structured diagnostic定位document/node/pin/file/span，支持fix-it、dedup、localization和stale过滤。 |
| EDM4-P1-024 | Open | compile在调用线程同步运行，无job priority/cancel/supersede/owner revoke。 | 接共享Editor job和Runtime189 compiler ticket；快速validation与完整compile分阶段，旧revision结果拒绝发布。 |

### 5.4 Toolkit、Material Instance 与preview

| ID | 状态 | 当前证据与差距 | 必须重构为 |
|---|---|---|---|
| EDM4-P1-025 | Open | Open/Preview命令没有handler，声明surface不能产生document或viewport。 | toolkit factory打开qualified document/session，返回view/document/generation receipt。 |
| EDM4-P1-026 | Open | 没有独立Material Instance toolkit；Material和Graph被同一view descriptor概括。 | Material、Instance、Graph、Shader Text、Stats/Diagnostics是协作但独立的toolkit mode。 |
| EDM4-P1-027 | Open | parent字段存在但无chain/cycle/missing/reparent/override migration UI。 | instance resolver显示origin/effective value，reparent先预览diff并以一个可撤销transaction提交。 |
| EDM4-P1-028 | Partial | Runtime shader property schema存在，Editor没有由其生成完整inspector。 | schema驱动widget、range/unit/color/enum/resource/visibility/dependency/validation和mixed values。 |
| EDM4-P1-029 | Open | base color之外的normal/metallic/roughness/AO/emissive/alpha/render state/queue/texture slots没有完整编辑面。 | 所有Runtime material字段和shader custom properties按同一typed commit route编辑。 |
| EDM4-P1-030 | Open | 没有独立preview scene、mesh/environment/camera/path/quality/target控制。 | MaterialPreviewSession拥有隔离scene、真实Runtime artifact、2D/3D mesh、lighting/IBL、target/profile和capture。 |
| EDM4-P1-031 | Open | preview没有current source/requested/LKG/installed/drawn generation可见性。 | viewport标题、diagnostic和stats绑定同一`PreviewGenerationReceipt`，禁止显示错代成功。 |
| EDM4-P1-032 | Open | 没有per-node preview、preview budget、visibility priority或stale texture retirement。 | node/graph/material preview统一调度、cache和GPU budget；只更新可见dirty subgraph。 |

### 5.5 Document、save、stats 与产品真相

| ID | 状态 | 当前证据与差距 | 必须重构为 |
|---|---|---|---|
| EDM4-P1-033 | Partial | shareddocument/save/CAS/autosave存在，Material领域没有adapter；没有dirty/savepoint/reopen/migration测试。 | MaterialDocument adapter接revision/history/savepoint/autosave/recovery/CAS，save只写canonical current schema。 |
| EDM4-P1-034 | Open | 没有shader text document、compile stats、permutation/PSO cost、dependency/recompile reason或平台对照。 | text/graph共享compiler与diagnostic；Stats显示真实target artifact、instruction/resource/permutation/PSO/cache/compile数据。 |

## 6. P2 性能、质量与维护（12 项）

1. **EDM4-P2-001**：graph/document使用大量String/BTreeMap；建立1k/10k/100k nodes的edit/validate/compile/undo/serialize/RSS基线。
2. **EDM4-P2-002**：增量validation按changed nodes和dependency frontier运行，禁止每次pointer move或selection重编完整graph。
3. **EDM4-P2-003**：node/pin/link使用arena/stable index只作当前generation加速，跨save/merge仍以GUID为权威。
4. **EDM4-P2-004**：per-node preview按可见性、viewport distance、dirty cost、GPU bytes调度；后台tab不得争抢主preview预算。
5. **EDM4-P2-005**：大型palette使用分类/搜索/usage/favorites索引，未知plugin node可opaque显示并阻止不可逆保存。
6. **EDM4-P2-006**：source map与diagnostic index支持node/pin/file双向定位，避免每次错误点击全图扫描。
7. **EDM4-P2-007**：material instance多选和mixed values必须按schema批量提交，逐对象事件风暴需要coalesce与per-item failure。
8. **EDM4-P2-008**：preview render target、mesh、environment和artifact缓存需entry/byte/device budget、LRU、lease与project teardown。
9. **EDM4-P2-009**：compile debounce是可配置policy，必须与cancel/supersede结合；简单固定毫秒不能掩盖慢compiler。
10. **EDM4-P2-010**：graphdiff/merge按GUID和semantic property工作，不能依赖serialization顺序或position。
11. **EDM4-P2-011**：keyboard、screen reader、high contrast、zoom/pan、large graph navigation和localization需要动态UI验收。
12. **EDM4-P2-012**：Editor interaction、compiler job、Runtime artifact、preview frame和save receipt共享correlation ID，才能分析端到端延迟。

## 7. 参考引擎对照

| 参考 | 可吸收的工程合同 | Zircon当前差异 |
|---|---|---|
| Unreal Material Editor | `MaterialGraphSchema`在连接前检查same-node、loop、pin type并用`FScopedTransaction`修改；Material Editor分离graph、apply、preview、feature/quality、stats和instance编辑。 | Zircon只有descriptor palette和事后字符串validator，无transaction、真实apply/preview/stats/instance产品。 |
| Unity Shader Graph | `GraphData`保存asset GUID、target、unknown target、preview data并执行ValidateGraph；Generator按target/pass生成vertex/surface代码；PreviewGenerator创建独立scene/camera/material。 | Zircon没有versioned target graph或真实preview，两个compiler都未形成可安装artifact。 |
| Godot Material/Shader | ShaderMaterial根据shader暴露参数、revert和RID，shader语言提供typed AST/uniform hint，runtime version/variant负责compile/cache。 | Zircon Runtime schema有基础，但Editor未由schema生成inspector，graph与runtime compile/version断开。 |
| Fyrox Material Editor | Material由shader resource schema定义property/texture binding；Editor material command具有execute/revert，资源可共享保存。 | Zircon有通用command底座却无Material factory，property/texture/parent修改不可撤销且无save receipt。 |
| Bevy Pipeline/Shader Cache | Runtime明确Queued/Creating/Ok/Err并等待shader dependency；适合作为Editor状态投影的底层合同。 | Zircon Editor command/preview无法显示requested/artifact/installed/drawn generation，成功状态无Runtime receipt。 |

## 8. 目标架构

```text
MaterialAuthoringPackage
  -> MaterialDocumentService
       { DocumentId, revision, history, savepoint, migration }
  -> MaterialGraphSchemaService
       { stable IDs, typed pins, connect rules, functions, target/domain }
  -> MaterialCompileController
       { Editor job -> Runtime189 compiler ticket -> artifact receipt }
  -> MaterialToolkit
       { graph, instance, inspector, shader text, diagnostics, stats }
  -> MaterialPreviewSession
       { isolated scene + exact Runtime artifact/LKG/draw generation }
  -> Asset/Package Publication
       { resources + factories + templates + state/unload all-or-nothing }
```

Editor拥有source document、交互transaction和结果投影；Runtime189拥有compiler artifact、pipeline、GPU resource和draw publication。任何Editor preview都只能消费Runtime receipt，不能在插件里创建第二套shader/pipeline authority。

## 9. 重构顺序

1. **M249.0 产品真相**：package activation对资源/factory/dist/backend原子preflight；缺失项时Material Editor明确Unavailable，删除no-op ShaderGraph执行器。
2. **M249.1 单一graph schema硬切**：冻结version/GUID/target/domain/type规则，给两个旧DTO做read migration，write只允许canonical格式，完成后删除旧类型和re-export。
3. **M249.2 Document/operation**：实现open/create/edit/connect/delete/parameter/reparent/save factories，全部接history/CAS/autosave/recovery与typed receipt。
4. **M249.3 Compiler/artifact**：graph生成typed IR并调用Runtime189 compiler；诊断具node/pin/span/source map，compile可cancel/supersede并保留LKG。
5. **M249.4 Toolkit/instance/inspector**：schema-driven property/texture/options/render state，instance origin/effective/override/reparent完整且可撤销。
6. **M249.5 Preview/stats**：隔离PreviewScene执行真实artifact，2D/3D/mesh/environment/target切换，显示exact generation、stats、cache和failure。
7. **M249.6 产品验收**：save/reopen/migrate/plugin reload/project switch、1k/10k graph、multi-select、fault、device loss、a11y、visual capture与性能报告。

## 10. 资格门（26 个）

| Gate | 状态 | 完成条件 |
|---|---|---|
| ED-MAT-G01 | Fail | package内所有ZUI/template/schema资源存在且activation前验证。 |
| ED-MAT-G02 | Fail | 默认catalog/profile可打开Material、Instance和Graph或显示typed unavailable。 |
| ED-MAT-G03 | Fail | 六条可见operation都有production factory和terminal receipt。 |
| ED-MAT-G04 | Fail | native dist bridge/state/restore/unload与声明maturity一致。 |
| ED-MAT-G05 | Fail | plugin unload/project switch退休document/job/preview/command，无旧回调。 |
| ED-MAT-G06 | Fail | Material/Shader Graph只剩一个canonical schema/compiler。 |
| ED-MAT-G07 | Fail | graph有version/GUID/revision/target/domain/migration。 |
| ED-MAT-G08 | Fail | node/pin/link stable identity在copy/save/reopen/merge后保持。 |
| ED-MAT-G09 | Fail | palette/connect/validator/compiler共享typed pin与conversion规则。 |
| ED-MAT-G10 | Fail | same-node/direction/cardinality/type/cycle在transaction apply前拒绝。 |
| ED-MAT-G11 | Fail | 所有graph/material/instance编辑可undo/redo并恢复selection/diagnostic。 |
| ED-MAT-G12 | Fail | compiler覆盖完整Material contract，而非只折叠base color。 |
| ED-MAT-G13 | Fail | generated code自包含、deterministic、symbol安全，无未定义helper。 |
| ED-MAT-G14 | Fail | Naga/reflection/pass ABI fail-closed，invalid source不产生admittable artifact。 |
| ED-MAT-G15 | Fail | compile receipt带source/dependency/compiler/target/artifact provenance。 |
| ED-MAT-G16 | Fail | compile job支持priority/cancel/supersede/stale rejection和bounded result。 |
| ED-MAT-G17 | Fail | UI同时显示current/requested/LKG/installed/drawn generation。 |
| ED-MAT-G18 | Fail | PreviewScene执行真实Runtime artifact，支持mesh/environment/target/capture。 |
| ED-MAT-G19 | Fail | instance parent/override/reparent/cycle/schema migration完整可撤销。 |
| ED-MAT-G20 | Partial | Runtime property schema可投影，但尚未驱动完整Inspector。 |
| ED-MAT-G21 | Partial | shared save/CAS/autosave存在，Material adapter/save-reopen尚未闭合。 |
| ED-MAT-G22 | Partial |已有字符串diagnostic，仍需stable code/node/pin/span/fix-it。 |
| ED-MAT-G23 | Partial | Runtime有material LKG基础，Editor尚未消费exact artifact/draw receipt。 |
| ED-MAT-G24 | Fail | stats来自真实artifact/target/permutation/PSO/cache，不是固定文案。 |
| ED-MAT-G25 | Fail | 1k/10k graph、多选、preview churn、keyboard/a11y/localization动态通过。 |
| ED-MAT-G26 | Fail | create->edit->compile->preview->save->reopen->reload E2E和fault矩阵通过。 |

## 11. Review-only 交付边界

本轮只写review、索引和coverage，没有修改Editor/Runtime/plugin/Cargo/ABI/ZUI，也没有补缺失资源或运行UI/Cargo/GPU测试。下一步实现必须优先关闭父P0与single-schema/product-truth边界；在资源、factory、compiler artifact和preview receipt闭合之前，不应增加更多按钮、palette node或静态成功反馈。
