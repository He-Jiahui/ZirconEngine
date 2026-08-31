---
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/rendering
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/rendering
  - zircon_editor/src/ui/material_editor
  - zircon_editor/src/ui/host/asset_editor_sessions
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/render_asset_vfx.rs
  - zircon_editor/src/core/asset/type_registry/builtin.rs
  - zircon_plugins/editor_support
  - zircon_plugins/first_party_editor_catalog
  - zircon_plugins/material_editor
  - zircon_plugins/shader_wgsl_importer
  - zircon_plugins/particles
  - zircon_plugins/rendering/features/shader_graph
  - zircon_plugins/rendering/features/vfx_graph
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/material
  - zircon_runtime/src/asset/assets/shader
  - zircon_runtime/src/graphics/shader/shader_assets.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/15-material-shader-graph-instance-vfx-particle-preview-compiler-diagnostics-authoring-review.md
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
review_id: Editor91
---

# 91 · Editor Material / Shader Graph / Material Instance / VFX / Particle / Preview / Compiler / Diagnostics Authoring 当前源码工程化差距

## 1. 当前结论

冻结于 `2026-08-25T17:37:22+08:00`、HEAD `8ee9411db24b7b4bdaf3fe028194642a7557c0b6` 的物理工作树仍没有可交付的工程级 Material、Shader Graph、VFX Graph 或 Particle authoring 产品。Zircon 已有强于原型的底座：`MaterialAsset`、`ShaderAsset`、材质合同验证、Naga WGSL importer、Material/RendererData typed projection、Particles CPU/GPU runtime，以及 Editor 的 transaction、operation factory、document/save/autosave/job 框架都是真实代码。它们应被保留，但当前没有形成从 source document 到 compiler artifact、runtime preview、durable save 和产品入口的闭环。

四份可见 rendering Workbench ZUI 合计 **884 行、103 个 node、71 条 event route、0 个 provider**。Material 固定展示 `M_Rock_Cliff`，VFX 固定展示 `P_Bolt_01`，Shader Editor 固定展示 `lighting.wgsl`，Particle Library 固定展示 `P_Sparks`。对应 callback 仍直接写入 `preview persisted`、`compile queued`、`compile complete, 2 warnings`、`simulation running, no errors`、`60 fps preview` 等结果，没有 compiler ticket、source revision、artifact generation、preview session 或 runtime receipt。

产品装配仍不闭合。first-party Editor catalog 只有 Navigation 与 Neural；Material Editor 没有进入默认 linked catalog，且声明的 `plugins://material_editor/editor/graph.zui` 与 `plugins://material_editor/templates/default_material_graph.toml` 两个物理资源仍不存在。Material 和 Particles 注册了 command descriptor，但 `EditorAuthoringContributionBatch` 不携带 operation factory；真实 dispatch 在缺失 factory 时会返回 `MissingFactory`。Particles 菜单仍显式 disabled，Shader Graph/VFX Graph Editor crate 只注册 capability/plugin descriptor，没有 authoring extension。

编译权威仍分裂。仓内同时存在 runtime authoring `MaterialGraphAsset`、graphics shell 同名 graph、rendering feature `ShaderGraphAsset` 和独立 VFX schema。Material plugin compiler 只求值 base color；Shader Graph 直接拼 WGSL、调用未定义 texture helper、未经过 Naga，且 executor 为 no-op；VFX compiler 只检查少量节点并返回固定 pass，两个 executor 仍为 no-op。真实 WGSL importer 虽能 Naga parse/validate，却没有把完整 dependency/reflection/layout/property/texture contract 填入 artifact。

因此 Editor15 的 5 项父 P0 当前仍为 **5 Open / 0 Partial / 0 Closed**。本报告不重复登记 P0；60 项 canonical P1 重判为 **21 Open / 39 Partial**，12 项 P2 均为 Open；32 项验收门为 **21 Fail / 11 Partial / 0 Pass**。Partial 只表示共享底座或局部数据结构已存在，不表示产品可用。目标链固定为：

```text
versioned Material / Shader / VFX / Particle source assets
  -> transactional domain documents with stable element identity
  -> one shared structural and semantic compiler authority
  -> immutable target artifacts + diagnostics + source maps
  -> background build, dependency invalidation and last-good publication
  -> isolated runtime preview sessions with generation-qualified telemetry
  -> real asset toolkits, inspectors, graph/text/curve editors and debug views
```

## 2. 当前物理范围与证据边界

### 2.1 冻结范围

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 当前证据 |
|---|---:|---|
| Editor 产品面、projection、operation、UI asset session 与 catalog | **81 / 17,067 / 16,079 / 700,047 / 82 / 3** | 四份 Workbench、route/binding/feedback、Material/RendererData projection、UI document/save/refresh、asset registry、catalog 与 App 接线；fingerprint `b6367ea175e514bd6aee6db1db496cd0f2da5e004180638ffb6cea6cb733e7b0` |
| Material/Shader source、asset、plugin、importer 与 compiler | **42 / 7,284 / 6,659 / 258,125 / 37 / 6** | Material plugin、WGSL importer、authoring graph、Material/Shader asset 全模块及未跟踪 `default_pbr.rs`；fingerprint `d387556b4e922854bdb764d019b328084861244c4448d43ab1f58e7c9da6da4d` |
| Particle/VFX/Shader Graph package 与 runtime | **74 / 9,568 / 8,678 / 335,241 / 52 / 1** | Particles editor/runtime/templates/tests、Shader Graph/VFX Graph editor/runtime feature；fingerprint `9bca1c27b86ade465fcee1a3c573fa584087862853dbd92f84c5ffa189631282` |
| Zircon selected 合计 | **197 / 33,919 / 31,416 / 1,293,413 / 171 / 10** | 当前物理文件去重集合；fingerprint `b6a57b2828ae79c8bc40e84c3a7e3f3726697e7628c7e24b26b367a501427838` |
| Unreal、Unity Graphics、Godot、Fyrox、Bevy 参考 | **43 / 64,283 / 55,428 / 2,642,860 / 约 63 / 0** | graph model/validation/generation、target、undo、compiled data、preview、stats、shader cache 与 focused tests；fingerprint `68cf86e052a72c0c0fbe89329bd14c40c93c52b2500e17116ada610c7c0320fd` |

fingerprint 按 repo-relative path 排序，将 `path + NUL + file SHA-256 + LF` 聚合后再计算 SHA-256。它只标识本轮证据集合，不是 shader permutation、pipeline cache、DDC 或 cook key。

### 2.2 Currentness 与并发工作树

selected 范围当前包含 **49 个既有 tracked 修改文件和 1 个未跟踪文件** `zircon_runtime/src/asset/assets/material/default_pbr.rs`。变化覆盖 Workbench ZUI、Material projection、UI asset session、operation dispatch、Material compiler/tests、WGSL importer、Particles runtime，以及 Material/Shader asset 合同。本轮读取并评价的是这些物理内容，没有回退、格式化或覆盖任何生产修改。

这些在途变化带来若干真实局部进展，例如材质默认 PBR 数据开始抽出、projection/tests 和 asset session 更丰富、Particles planner/runtime owner 继续演进。但它们没有改变默认 catalog、缺失插件资源、无 operation factory、无领域 document、固定 Workbench feedback、graph authority 分裂和 no-op VFX/Shader Graph executor等结论。因此 `source_recheck_required=true`，实施前必须重新冻结同一选择集。

### 2.3 动态证据边界

按用户要求本轮只做 review，没有运行 Cargo、Editor/App、asset create/import/save/reopen/cook、shader backend compiler、GPU preview、particle simulation parity、device loss、fault/scale/soak/profile或跨引擎 benchmark。171 项静态 test declaration 主要覆盖 shared UI asset session、Material projection、最小 graph compiler、WGSL importer、Particles runtime 与 registration metadata；10 项 ignored 中包含 release performance gate。它们不证明默认产品装配、真实 Material/VFX document mutation、source-to-pixel、source-to-particle、跨 backend compile matrix 或发布资格。

## 3. 当前实现的纵向事实

### 3.1 产品入口、资源与 capability truth

1. `ResourceKind` 已识别 Material、MaterialGraph 和 Shader，但 builtin toolkit 仍只覆盖 UI 与 Animation；插件缺失时没有领域只读详情或 preview fallback。
2. `zircon_plugins/first_party_editor_catalog` 仍只按 feature 装配 Navigation 与 Neural，`zircon_app/Cargo.toml` 也没有把 Material、Particles、Shader Graph 或 VFX Graph Editor 纳入默认 host feature 闭包。
3. Material plugin 注册了 view、drawer、toolkit、graph descriptor、palette、creation template 与六个 operation，但两个声明资源物理缺失，publication 前也没有 URI 可解析性硬门。
4. Particles 的三份插件 ZUI 与 `cpu_sprite_system.toml` 物理存在，但 authoring/preview/drawer surface 没有业务绑定，菜单显式 disabled，模板没有 importer/creation consumer。
5. Shader Graph/VFX Graph Editor package 只有 descriptor/capability；runtime feature 默认可选且 executor 不工作。manifest 声明、runtime readiness 与 Editor visible state 没有单一 capability resolver。

### 3.2 Operation、document、save 与 UI truth

1. `EditorAuthoringContributionBatch` 可注册 command、menu、asset、inspector、scene mode、graph/timeline descriptor，但没有 operation factory、document、compiler或preview provider字段。
2. Editor core 已有真实 `OperationCommandFactoryRegistration`、transaction engine、dirty/save/autosave、CAS/refresh 和 typed failure；Material/Particles 插件没有消费这些路径。
3. Material 六个 operation 与 Particle create/open/add/validate/play/pause/stop/rewind/warmup operation 只在 registration/tests 中出现，没有 factory registration。
4. generic `asset_editor_sessions` 是 UI asset document 产品，包含 robust save/refresh/dependency/transaction底座，但 source 类型、route、loader 和 mutation 都是 UI 专属，不能直接冒充 Material/VFX document。
5. Workbench field edit 仍只改控件 `value/value_text` 并刷新；Submit 与 Change 没有 domain validation、transaction、dirty generation 或 save acknowledgement差异。
6. 四份 Workbench command feedback 仍以固定成功文本替代 operation/job/runtime result，且 71 条 event route 均没有业务 provider。

### 3.3 Material、Shader 与 graph compiler

1. runtime authoring `MaterialGraphAsset` 只有 Output、TextureSample、ScalarParameter、VectorParameter、Add、Multiply 六类节点；importer 只验证至少存在一个 Output。
2. Material plugin validator 增加 duplicate ID、exactly-one output、悬空 node/pin、重复 incoming 和 required input 检查，但没有完整 pin typing、domain、finite、reachability或预先 cycle诊断。
3. Material compiler 只折叠 base color；normal、metallic、roughness、emissive、alpha 和 render options 被写成固定默认，texture-backed Add/Multiply 被拒绝。
4. runtime `ShaderAsset` 和 `MaterialAsset` 已有 source、entry、dependency、property/texture schema、render state、resource/layout、parent/PBR/options/queue/diagnostics 等强合同，这是未来 artifact 的正确承载面。
5. `zircon_runtime::graphics::shader::shader_assets` 又定义了只有 `name/output_domain` 的同名 Material/Shader graph shell；rendering Shader Graph feature再定义第三套不兼容 schema。
6. Shader Graph generator直接把 node ID 和 input name插入WGSL identifier；TextureSample调用未定义 `zircon_sample_texture_*` helper；缺Output时生成magenta函数同时报错；生成物不经过Naga。
7. WGSL importer确实执行Naga parse与validate并提取entry point，但导入后的dependency/source/import/definition/property/texture/options/render contract大多为空或默认。
8. `MaterialEditorProjection` 与 `RendererDataEditorProjection` 能生成 typed property、texture、feature 与 diagnostic row，focused tests覆盖较充分；生产 consumer 精确搜索仍为零，projection 不能反向提交 typed command。

### 3.4 VFX、Particle 与 preview/runtime bridge

1. Particles runtime 已有 `ParticleSystemAsset`、Emitter、CPU/GPU simulation、pool/RNG、extract、buffer/layout/program/readback、transparent render、service/manager 与 snapshot tests。
2. 该 asset 仍不是 versioned serde/import contract，`particles.system` 只在 Editor registration/test语境出现，普通 AssetKind/importer/save/reopen链不存在。
3. VFX Graph schema只有 SpawnRate、InitialVelocity、Gravity 和 ShaderGraphMaterial；compiler 只检查 `max_particles`、spawn 与 material 后返回固定 pass 名。
4. VFX descriptor现有固定 compute workload label、`[64, 1, 1]` workgroup与`[1, 1, 1]` dispatch，但 simulation/transparent executor 均为 `Ok(())` no-op。
5. Material/Shader没有隔离 preview scene、mesh/camera/environment/target/quality控制；Particle/VFX没有隔离 world、clock、seed、warmup/rewind、generation和runtime telemetry bridge。
6. Runtime99D 已拥有 Particles/VFX simulation/render/scalability/determinism父问题；Editor91只拥有source document、toolkit、transaction、compiler触发、preview orchestration与产品projection，不重复计算runtime算法差距。

## 4. 参考引擎差异与采用边界

| 参考 | 本轮逐源码确认的工程事实 | Zircon 必须吸收的边界 |
|---|---|---|
| Unity ShaderGraph | `GraphData` 持有稳定序列化图与 target；`GraphValidation` 检查 target/subtarget兼容；`GraphEditorView` 对 connect/move/remove 统一注册 undo；`Generator` 从 target setup 生成完整 shader；tests覆盖graph、serialization与generator | source、validation、UI transaction、target generation必须分层但同源；Zircon不得让palette、validator和compiler各自解释pin语义 |
| Unity VFXGraph | `VFXModel` 以明确 invalidation cause传播dirty；Context区分Spawner/Init/Update/Output；CompiledData构建expression/context/task/buffer；compiler pass输出asset descriptor；Undo stack区分full backup与delta并决定recompile | 建立typed context/data/model、增量失效、compiled program/buffer plan、undo/recompile currentness；不复制Unity对象模型 |
| Unreal Material Editor | Material graph schema在创建/连接时进入transaction并拒绝loop；Material Editor、Instance Editor、Viewport与Stats各自拥有apply/preview/override/platform shader统计职责 | Material/Instance/Graph/Preview/Stats应是协作子系统；连接合法性必须在编辑时与compiler一致，preview必须执行真实shader |
| Unreal Niagara | Graph维护compile hash与recompile notification；compiler job有异步结果；HLSL translator做typed pin validation；message manager按asset/topic维护diagnostic；SystemViewModel拥有preview component、compile状态与undo | VFX/Particle需要compile identity、async result、typed diagnostics、preview instance和system/emitter authoring，不得只返回固定pass字符串 |
| Godot | Shader Editor维护多文档/unsaved/error状态；Text Shader有preprocessor、错误定位与preview；Visual Shader有完整node/connection/undo插件；Particles动作通过UndoRedo修改真实resource/node | text与visual shader必须共享runtime语义；所有可见编辑和particle工具动作都应可撤销并作用于真实资源 |
| Fyrox | Material property editor绑定真实resource/preview；scene material command提供execute/revert并保存；particle preview保存并恢复scene node状态，驱动play/stop/rewind | Rust实现可采用typed command、resource editor与隔离预览状态恢复，不需要复制Unreal/Unity复杂对象系统 |
| Bevy | Material通过typed bind group与pipeline specialization进入render；Shader记录imports/dependencies/defs/validation；ShaderCache等待依赖、缓存processed shader并追踪dependent pipeline | Editor artifact必须连接runtime shader/material/cache/PSO权威；Bevy只作runtime contract参考，不作Editor UX基准 |

## 5. 可保留底座与禁止误判

1. 保留 `MaterialAsset`、`ShaderAsset`、`.zmaterial/.zshader`、readiness/validation与新抽出的默认PBR合同；不要新建只供Editor使用的平行runtime DTO。
2. 保留 Naga importer，但将 parse/validate/reflection/source map提升为共享compiler stage，供文本shader、graph、import、preview与cook共同调用。
3. 保留 Material/RendererData projection，补 stable row ID、edit descriptor、revision、provenance和production consumer，不从显示文本反推写命令。
4. 保留 Editor transaction、operation factory、document toolkit、dirty/save/autosave、CAS/refresh、job与extension生命周期底座；Material/VFX只能通过领域adapter接入。
5. 保留 Particles CPU/GPU runtime、planner、resource与snapshot基础；Editor preview通过runtime gateway消费，不能在UI层另写一套simulation。
6. 保留四个 Workbench位置合同，但在真实 provider 就绪前改为 typed Unavailable；固定业务数据与成功反馈不能继续作为产品能力证据。

## 6. Editor15 父 P0 当前重判

| Canonical owner | 状态 | 当前证据与硬切要求 |
|---|---|---|
| `Editor15-P0-01` 默认产品入口、插件装配与资源包不闭合 | Open | first-party catalog仍无四类Editor，Material两个声明资源缺失。先建立manifest/resource/factory/provider publication硬门，再开放入口。 |
| `Editor15-P0-02` authoring operation只有descriptor | Open | core factory路径真实存在，但Material/Particles贡献仍未注册factory，Particles菜单还disabled。所有visible action必须绑定owner-qualified执行合同。 |
| `Editor15-P0-03` Workbench伪造compile/simulate/preview | Open | fixed feedback仍直接写成功结果，字段仍只改控件。删除业务固定成功分支，结果只接受typed operation/job/runtime receipt。 |
| `Editor15-P0-04` graph authority分裂且输出不可证明执行 | Open | 三套Material/Shader graph继续并存，Shader/VFX executor仍no-op，生成WGSL未Naga验证。必须hard cut到单一schema/compiler/artifact。 |
| `Editor15-P0-05` 无transactional document、durable save与runtime一致preview | Open | shared UI document底座存在但领域consumer为0；Material/VFX/Particle无document ID、source revision、save/reopen或preview generation。 |

## 7. P1 当前源码差距账本

### 7.1 产品装配、toolkit、document lifecycle 与 capability

### `ED-MSV-P1-001` [Partial] builtin识别资产但没有领域fallback toolkit

Material、MaterialGraph和Shader有type/presentation基础；补只读details/diagnostics fallback，并仅在完整插件provider通过资格检查后切换到可编辑toolkit。

### `ED-MSV-P1-002` [Open] 默认first-party Editor catalog没有四类产品

增加Material、Shader、Particles、VFX的package feature/dependency/registration矩阵与production bootstrap测试，App只从统一catalog装配。

### `ED-MSV-P1-003` [Open] Material插件资源合同仍损坏

补齐并版本化`graph.zui`和default graph template，publication必须在任何registry mutation前解析所有URI并验证owner/provenance。

### `ED-MSV-P1-004` [Open] dist invocation与bridge没有执行fallback

Material dist的command invocation和bridge methods仍为空；明确native/editor-only执行边界，未提供执行路径的descriptor不得标记available。

### `ED-MSV-P1-005` [Partial] shared operation factory存在但authoring batch无法贡献

扩展contribution transaction以携带factory/document/compiler/preview provider，并和descriptor在同一prepare/commit/revoke中原子发布。

### `ED-MSV-P1-006` [Partial] graph descriptor/palette有registry但没有产品consumer

建立owner-qualified graph canvas/session，消费descriptor/palette、selection、transaction和schema generation；禁止snapshot只供测试查询。

### `ED-MSV-P1-007` [Partial] capability/maturity存在但readiness语义分裂

统一manifest maturity、feature enable、resource、factory、backend、target和runtime executor readiness，输出typed unavailable reason。

### `ED-MSV-P1-008` [Open] Particles插件authoring/preview/drawer仍为空壳

将三份`Space` surface替换为document-backed emitter/module/curve/viewport投影；tests必须验证真实binding与mutation，不只验证control ID。

### `ED-MSV-P1-009` [Open] `particles.system`没有Runtime AssetKind/importer

建立versioned source extension、bounded importer、artifact role、save/reopen/cook和runtime handle，模板只能引用已注册schema。

### `ED-MSV-P1-010` [Partial] shared document/save/autosave真实但领域未接入

为Material/Shader/VFX/Particle建立DocumentId、source revision、dirty/history/savepoint/autosave/close decision adapter，不能复用UI DTO语义。

### `ED-MSV-P1-011` [Open] plugin unload/reload没有领域保持策略

定义open document、unknown node/module、opaque payload、last-good artifact、preview session和unsaved source在owner revoke/reload时的状态机。

### `ED-MSV-P1-012` [Partial] experimental/partial标记没有形成产品门禁

现有maturity字段只作metadata；产品navigation、command、toolkit和restore必须消费同一effective capability snapshot。

### 7.2 Material、Material Instance 与 property authoring

### `ED-MSV-P1-013` [Partial] Material projection只读且无production consumer

保留typed rows，增加stable row ID、source revision、edit descriptor与transaction route，并接入真实Material toolkit。

### `ED-MSV-P1-014` [Partial] property schema元数据不足以生成完整widget

当前已有kind/group/label/default；补range、unit、color、enum、resource、visibility、dependency、commit validation和custom editor factory。

### `ED-MSV-P1-015` [Partial] override状态可见但不可操作

增加enable/reset/revert/copy/paste、multi-edit mixed value、origin breadcrumb和单revision原子提交。

### `ED-MSV-P1-016` [Partial] parent material存在但没有Editor继承工作流

补parent chain/cycle/missing revision检查、resolved value diff、reparent transaction、override迁移和依赖失效。

### `ED-MSV-P1-017` [Partial] runtime PBR/render option完整度高但Editor不可编辑

将shading、blend、queue、cull、depth/stencil、alpha、clearcoat、anisotropy、transmission等映射为target-aware typed authoring。

### `ED-MSV-P1-018` [Partial] texture slot有合同但没有资源工作流

补drag/drop、dimension/color-space/normal-map、sampler、fallback、streaming residency、thumbnail、missing reference与批量替换。

### `ED-MSV-P1-019` [Partial] shader schema已有数据但Material inspector不随其增量更新

建立shader revision订阅、compatible override migration、orphan preservation和exact invalidation，避免整个Inspector无身份重建。

### `ED-MSV-P1-020` [Partial] diagnostics可投影但缺稳定身份和修复链

为诊断补code、severity、source span、material/property/slot element ID、artifact revision、target和quick-fix/suppress policy。

### `ED-MSV-P1-021` [Open] Material Graph节点域无法表达MaterialAsset

扩展domain、output和function/subgraph模型，使每个可持久Material字段都能被表达或返回明确unsupported诊断。

### `ED-MSV-P1-022` [Partial] validator只有局部structural检查

补共享pin type/cardinality/conversion、cycle、reachability、finite/resource/domain/stage和deterministic diagnostic order。

### `ED-MSV-P1-023` [Partial] compiler只覆盖base color

把全部Material output编译到canonical IR、ShaderAsset/material artifact与pipeline recipe；禁止对未实现字段静默写固定default。

### `ED-MSV-P1-024` [Open] palette、validator与compiler类型语义冲突

Add/Multiply的float声明、vec4广播和texture拒绝必须由同一schema descriptor定义，连接UI应预先预测compile结果。

### `ED-MSV-P1-025` [Partial] runtime artifact/PSO底座未被graph compiler消费

输出reflection、resource/layout、variant、pipeline recipe、source map和artifact key，交由Runtime09C authority安装，不在Editor创建第二套PSO路径。

### 7.3 Graph schema、Shader authoring 与 diagnostics

### `ED-MSV-P1-026` [Open] 多套graph authority继续并存

确定唯一versioned schema owner，提供一次性migration或typed rejection，完成后删除graphics shell和feature-local平行DTO，不留compat facade。

### `ED-MSV-P1-027` [Open] node/edge/pin没有稳定身份

引入stable element/edge ID、namespace、schema version、unknown opaque payload和layout metadata，禁止自由字符串承担全部identity。

### `ED-MSV-P1-028` [Partial] importer与plugin validator规则冲突

两条入口已有验证函数但标准不同；收敛到共享compiler preflight，使import、open、save、preview和cook使用同一corpus。

### `ED-MSV-P1-029` [Open] 没有共享pin/resource/domain规则

建立typed pin lattice、conversion table、resource binding、stage/domain restriction、default value和cardinality authority。

### `ED-MSV-P1-030` [Open] palette声明与求值器广播行为不一致

删除隐式、未声明的scalar-to-vector语义，或将其作为显式conversion node/edge artifact并由UI显示。

### `ED-MSV-P1-031` [Partial] cycle只在递归求值时暴露

现有compiler能因递归失败报错，但缺preflight topology；补有界cycle path、topological order、dead node和side-effect排序。

### `ED-MSV-P1-032` [Open] WGSL identifier生成可被任意字符串破坏

建立symbol table、sanitize/collision规则、保留字处理和source element map；对Unicode/恶意/超长ID设明确budget。

### `ED-MSV-P1-033` [Open] TextureSample生成未定义helper与layout

compiler必须生成完整binding declaration、sampler/texture类型、helper或内联调用，并与ShaderAsset resource/layout完全一致。

### `ED-MSV-P1-034` [Partial] Naga validator存在但graph生成不使用

将WGSL importer的Naga stage抽成共享服务，所有graph/text/import产物在publication前执行同一parse/validate与backend gate。

### `ED-MSV-P1-035` [Open] error graph与fallback artifact边界不清

缺Output等错误不得产生可admit artifact；last-good、diagnostic preview和intentional fallback必须是互斥typed状态。

### `ED-MSV-P1-036` [Partial] WGSL importer真实但reflection合同为空

补include/import dependency、definitions、entry visibility、binding reflection、property/texture schema、render state和target provenance。

### `ED-MSV-P1-037` [Open] 没有Shader text document产品

实现unsaved buffer、incremental parse、completion、format、find/replace、include navigation、entry/target选择和buffer compile。

### `ED-MSV-P1-038` [Partial] importer错误只有字符串级投影

保留现有validation diagnostics，升级为stable code、file/line/column/span、include stack、target/revision和jump/quick fix。

### `ED-MSV-P1-039` [Partial] dependency字段存在但没有authoring失效图

建立source/include/function/parent/plugin schema dependency graph、watch/CAS、transitive invalidation和generated-source导航。

### `ED-MSV-P1-040` [Partial] runtime option/pipeline基础未形成compile matrix

增加target/subtarget、quality/platform/backend、keyword/static option、variant budget/stripping/usage和cache命中可视化。

### 7.4 VFX Graph、Particle System 与 runtime bridge

### `ED-MSV-P1-041` [Partial] Particle操作有descriptor但没有factory

core factory和transaction路径可复用；为create/open/emitter/module/curve/validate/transport定义typed payload、history与terminal receipt。

### `ED-MSV-P1-042` [Partial] Particle runtime模型真实但没有versioned serialization

为System/Emitter/Module建立serde envelope、schema version、migration、unknown preservation与source/artifact分层。

### `ED-MSV-P1-043` [Partial] CPU sprite template存在但不可创建可运行资产

把template接到creation operation、importer、document、save/reopen与runtime load E2E，URI测试不再作为完成证据。

### `ED-MSV-P1-044` [Open] 没有emitter/module/curve/gradient authoring产品

建立system/emitter stack、spawn/update/output context、attribute/parameter blackboard、curve/gradient、renderer/bounds/LOD/scalability编辑。

### `ED-MSV-P1-045` [Open] Particle preview没有真实session

实现isolated world、clock、seed、camera、collision/event source、play/pause/step/stop/rewind/warmup与generation fence。

### `ED-MSV-P1-046` [Partial] CPU/GPU runtime能力没有Editor gateway

暴露backend selection/capacity/buffer/readback/fallback/diagnostic/live stats，并证明同source在声明范围内的parity。

### `ED-MSV-P1-047` [Partial] VFX Graph只有极小schema/compiler

保留现有DTO作迁移输入，建立typed attribute/context/data-flow/module与canonical simulation/render program IR。

### `ED-MSV-P1-048` [Partial] VFX已有workload描述但executor仍no-op

workgroup/dispatch元数据只是起点；由compiled program和capacity推导buffer/dispatch，executor必须实际更新粒子并输出像素。

### `ED-MSV-P1-049` [Partial] feature capability存在但Editor/runtime readiness未绑定

把optional feature、backend、executor、shader/material dependency与authoring provider聚合为单一effective availability。

### `ED-MSV-P1-050` [Open] 高级VFX/Particle语义整体缺失

补event/collision/ribbon/mesh/decal/light/sub-emitter、GPU event/readback、deterministic seed、bounds/culling/LOD/scalability和module migration。

### 7.5 Preview、jobs、diagnostics、性能与测试

### `ED-MSV-P1-051` [Partial] 通用render/viewport存在但无Material preview authority

建立独立preview scene、mesh/LOD、camera、environment/light、render path、platform/quality和reference image控制，复用Runtime真实pipeline。

### `ED-MSV-P1-052` [Open] 没有last-good/current-source差异状态

UI必须同时显示source revision、requested build、last-good artifact、installed generation和preview generation，禁止旧画面冒充当前成功。

### `ED-MSV-P1-053` [Partial] shared background jobs未承载领域compile

接入Editor09 admission、dedup、priority、progress、cancel acknowledgement、quota、owner revoke和shutdown fence。

### `ED-MSV-P1-054` [Partial] shader/pipeline cache存在但缺完整authoring key

artifact/DDC key纳入source、transitive dependency、schema/plugin/compiler、target/options/backend，并输出可审计miss reason。

### `ED-MSV-P1-055` [Partial] RendererData projection无live consumer

连接当前preview renderer/pipeline/PSO snapshot，以renderer、view、artifact和generation限定feature diagnostics。

### `ED-MSV-P1-056` [Partial] typed diagnostics存在但无法定位graph/VFX元素

统一WGSL/backend/pipeline/simulation错误到property/node/edge/module/curve ID，并支持跨pane selection与jump owner。

### `ED-MSV-P1-057` [Partial] runtime已有部分snapshot但Editor无工程统计

显示shader instruction/resource/variant、pipeline create/cache、compile duration、particle CPU/GPU、alive/spawn/drop/bounds/overdraw。

### `ED-MSV-P1-058` [Partial] tests数量较多但产品闭环仍false-green

保留projection/compiler/runtime unit tests，新增default bootstrap、factory dispatch、document transaction/save、真实preview和source-to-output E2E。

### `ED-MSV-P1-059` [Open] fault、fuzz、large graph与GPU证据缺失

建立malformed corpus、Nth-step failure、plugin unload、device loss、cancel race、large graph/variant/particle soak与image/simulation parity。

### `ED-MSV-P1-060` [Partial] shared UI可访问性基础没有覆盖领域工具

graph/text/curve/preview完成keyboard/focus/screen-reader/high-contrast/reduced-motion/i18n与大列表/canvas虚拟化资格。

## 8. P2 成熟度差距

1. [Open] 缺per-user graph grid、wire style、node density、preview background和lighting rig偏好。
2. [Open] 缺可命名Material preview camera/environment/mesh与VFX seed/time/debug bookmark。
3. [Open] 缺recent asset、recent element和跨document navigation history。
4. [Open] 缺minimap、reroute、sticky note、frame/group、alignment和distribution工具。
5. [Open] 缺property/node/module/curve批量标签、颜色、comment和review note。
6. [Open] 缺source revision、generated source和preview image的结构diff/overlay。
7. [Open] 缺source、diagnostics、artifact manifest与GPU capture引用的bounded support bundle。
8. [Open] 缺opt-in compile latency、cache hit、preview FPS和particle budget趋势摘要。
9. [Open] 缺批量shader compile、material reparent/override迁移、particle validation和dry-run报告。
10. [Open] 缺Material function、subgraph/module preset、template library和dependency usage查询。
11. [Open] 缺Editor scripting/remote automation的typed Material/Shader/VFX command/query surface。
12. [Open] 缺多人协作asset lock、元素级冲突、review-only和revision annotation。

## 9. 重构里程碑

### M0 · Capability Truth、默认装配与资源publication硬门

- 建立四类Editor package和effective readiness matrix，补default bootstrap。
- publication原子验证resource、factory、document/compiler/preview provider与runtime executor。
- 删除fixed success；不完整能力统一投影typed Unavailable。

### M1 · Canonical Versioned Source 与 Schema Registry

- 收敛Material/Shader graph，定义stable node/edge/pin/parameter identity和migration。
- 建立Material/Shader/VFX/Particle versioned source envelope与unknown preservation。
- 建立owner generation、domain/target/context/module descriptor registry。

### M2 · Shared Semantic Compiler 与 Immutable Artifact

- 统一bounded parse、structural/type/topology/reference/domain validation。
- Material/Shader输出IR/WGSL/reflection/layout/variant/pipeline recipe/source map，并经Naga/backend验证。
- VFX/Particle输出真实simulation/render program、buffer/dispatch/resource计划。

### M3 · Transaction、Durable Save、Dependency 与 Jobs

- 所有property/node/edge/module/curve操作改为可逆typed command。
- 接入validated snapshot、atomic replace、CAS、autosave/recovery和LKG。
- compile接入job、DDC、transitive invalidation、cancel/progress/quota/shutdown。

### M4 · Material 与 Material Instance Toolkit

- 将projection接入真实Inspector，完成schema-driven property/texture/options编辑。
- 实现parent/override/reparent/multi-edit与diagnostic navigation。
- 建立真实runtime material preview和compile/pipeline stats。

### M5 · Shader Text 与 Shader Graph Toolkit

- 实现text document、include/dependency、completion、entry/target/profile和generated source。
- 实现graph canvas、typed connection、blackboard、function/subgraph、layout和undo。
- 建立source-to-artifact-to-pixel golden链。

### M6 · Particle 与 VFX Authoring

- 完成system/emitter/context/module/attribute/parameter/curve/renderer/bounds/LOD编辑。
- 连接Particles runtime与VFX compiler，移除no-op executor。
- 完成isolated preview、transport、seed/clock、backend parity和telemetry。

### M7 · Diagnostics、性能、故障与可访问性

- 接入Editor11 journal、source map、jump/quick fix与artifact manifest。
- 建立large graph/variant/particle budgets、fault injection、device loss与soak。
- 完成keyboard/focus/screen-reader/i18n/high-contrast/reduced-motion。

### M8 · Hard Cutover 与发布资格

- 同里程碑迁移first-party callers后删除旧graph DTO、fixed feedback、descriptor-only入口与假fallback。
- 通过default bootstrap、round-trip、backend matrix、reference image、particle parity和failure gates后再提升maturity。
- 不保留旧路径re-export、compat module、双compiler或双preview authority。

## 10. 验收门当前状态

- [ ] 1. **Fail** 默认production bootstrap能按声明打开Material/MaterialGraph/Shader，缺插件时返回typed reason。
- [ ] 2. **Fail** publication在缺factory/template/schema/compiler/preview/provider时原子失败。
- [ ] 3. **Fail** Material graph/template URI物理可加载并有version/owner provenance。
- [ ] 4. **Fail** 每个visible action都有factory并产生transaction/job/preview receipt。
- [ ] 5. **Fail** Workbench不再以固定字符串宣称compile/simulate/preview成功。
- [ ] 6. **Fail** canonical loader按budget拒绝oversized/deep/truncated/unknown/malformed source。
- [ ] 7. **Fail** 旧graph schema有迁移或typed拒绝且保持stable identity/opaque data。
- [ ] 8. **Partial** Material validator覆盖一部分ID/link/output错误，尚缺完整type/topology/domain。
- [ ] 9. **Fail** palette、connection UI与compiler共享pin/conversion规则。
- [ ] 10. **Partial** compiler能产生最小MaterialAsset，但未覆盖完整Material合同。
- [ ] 11. **Fail** generated WGSL自包含、安全并通过Naga和required backend。
- [ ] 12. **Fail** invalid source不产生可admit artifact，fallback/LKG状态明确。
- [ ] 13. **Fail** Shader text editor支持unsaved compile、include/source map与准确定位。
- [ ] 14. **Partial** runtime有typed options/pipeline基础，authoring target/permutation key未闭合。
- [ ] 15. **Fail** Material parent/override/reparent/schema变化可undo/redo并正确迁移。
- [ ] 16. **Fail** property/texture/option编辑按revision原子提交，失败不改状态。
- [ ] 17. **Fail** graph操作undo/redo恢复source、selection、diagnostic与generation。
- [ ] 18. **Partial** shared save/CAS/autosave存在，Material/VFX domain尚未接入。
- [ ] 19. **Partial** dependency/cache字段存在，authoring transitive invalidation尚未接通。
- [ ] 20. **Partial** shared jobs支持admission/cancel等基础，领域compile没有提交job。
- [ ] 21. **Fail** Material/Shader preview执行真实runtime pipeline并标识generation。
- [ ] 22. **Fail** reference material scene通过required backend image golden/GPU validation。
- [ ] 23. **Fail** `particles.system`可versioned create/save/reopen/cook/runtime load。
- [ ] 24. **Fail** Particle/VFX完整authoring操作均可undo/redo。
- [ ] 25. **Fail** VFX executor实际更新粒子并输出像素。
- [ ] 26. **Fail** preview transport驱动isolated runtime且固定seed/clock可复现。
- [ ] 27. **Partial** Particles runtime已有部分snapshot/readback，Editor尚无live telemetry产品。
- [ ] 28. **Partial** extension lifecycle底座存在，领域unknown/reload/LKG恢复未定义。
- [ ] 29. **Fail** 10万node/edge、万级variant、批量compile与百万粒子有预算benchmark。
- [ ] 30. **Partial** 有若干validator/performance tests，尚无完整fault/fuzz/device/driver矩阵。
- [ ] 31. **Partial** shared retained UI有部分输入/a11y基础，领域toolkit尚未验收。
- [ ] 32. **Partial** unit tests覆盖局部projection/compiler/runtime，缺default product E2E。

## 11. Owner 边界与 hard-cut要求

1. Editor91刷新Editor15唯一父账，不新增重复P0；Editor02/04/05/08/09/11继续拥有共享document/asset/inspector/command/job/diagnostic基础。
2. Runtime09C拥有Material/Shader/Pipeline/PSO执行合同；Runtime99D拥有Particle/VFX runtime simulation/render/scalability/determinism；Plugins04/05/09/18拥有package/importer/runtime distribution与texture依赖。
3. Editor91拥有领域document/toolkit、typed edit、compile orchestration、preview session、diagnostic projection和product truth。Editor不得直接拥有GPU handle或另建runtime compiler。
4. graph schema收敛必须硬切。迁移窗口可以读取旧格式，但写出只能使用canonical version；迁移完成后删除旧DTO、re-export、compat facade和旧caller。
5. fixed Workbench内容只能作为显式fixture/demo存在，不能继续挂在production capability、Save/Compile/Simulate动作或成功状态上。
6. tooling按用户要求排除；未来Rust迁移工具只消费canonical schema/artifact，不在本报告建立新的tooling owner。

## 12. 本轮验证与完成定义

本轮逐文件核对了四份可见surface、71条route、fixed feedback、field edit、operation dispatch、authoring contribution、catalog/App装配、Material/Shader source与asset、Material compiler、WGSL importer、Particles全package、Shader/VFX Graph feature和selected tests，并对照43个参考文件。没有运行动态构建或产品验证，因此任何静态test declaration都不写成“通过”。

Material/Shader/VFX/Particle Editor只有在以下事实同时成立时才可从“底座+样机”提升为工程级authoring：默认装配真实可达；所有可见操作有owner-qualified factory；source由单一versioned schema和共享compiler生成可验证artifact；document可撤销、durable save、CAS与恢复；preview执行真实runtime pipeline/simulation并带generation；diagnostic可定位；dependency/job/cache/failure/规模/backend/a11y都有资格证据。在此之前，当前四个Workspace和可选插件必须被描述为Partial/Experimental或Unavailable，不能作为完成能力展示。
