---
related_code:
  - zircon_plugins/shader_wgsl_importer
  - zircon_plugins/asset_importers/shader
  - zircon_plugins/README.md
  - zircon_plugins/first_party_runtime_catalog/Cargo.toml
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/first_party_runtime_catalog/src/tests/generated_manifest.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/asset_rows/pipeline.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_validation/asset_importers
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/registration_replay.rs
  - zircon_runtime/src/asset/importer/contract.rs
  - zircon_runtime/src/asset/importer/registry.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_runtime/src/asset/importer/ingest/import_shader.rs
  - zircon_runtime/src/asset/assets/shader
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_plugins/04-rendering-umbrella-feature-bundles-solari-native-provider-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_editor/15-material-shader-graph-instance-vfx-particle-preview-compiler-diagnostics-authoring-review.md
  - docs/plans/optimize/zircon_tooling/08-shared-derived-data-cache-build-cache-remote-execution-artifact-reuse-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/ShaderCompilerCore.h
  - dev/UnrealEngine/Engine/Source/Developer/ShaderPreprocessor/Public/ShaderPreprocessor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/ShaderCompiler/ShaderCompilerJobCache.cpp
  - dev/bevy/crates/bevy_shader/src/shader.rs
  - dev/bevy/crates/bevy_shader/src/shader_cache.rs
  - dev/godot/servers/rendering/shader_language.cpp
  - dev/godot/servers/rendering/shader_compiler.cpp
  - dev/Fyrox/fyrox-material/src/shader/loader.rs
  - dev/Fyrox/fyrox-material/src/shader/mod.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/ShaderStripping/ShaderPreprocessor.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/ShaderStripping/ShaderStrippingReport.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: false
---

# 05 · Shader WGSL / Family Importer、Compiler Artifact、Native Dist 与产品链工程化差距

## 1. 结论

仓内不是只有一个shader importer，而是三份重叠实现：`zircon_runtime` core内建WGSL/GLSL/SPIR-V解析代码、较新的`shader_wgsl_importer` package，以及README明确称为迁移期declaration aggregator的`asset_importers/shader`旧family package。三者都使用Naga并构造同一种`ShaderAsset`，但package owner、capability、priority、产品链接和native交付并不一致。

当前默认产品尤其矛盾。Builtin catalog把`shader_wgsl_importer`标为stable/partial并投影WGSL capability，static manifest清单也同时收录新旧两包；实际first-party runtime provider catalog却没有两包的Cargo依赖或registration分支，App选择任一source package都得不到实现。AssetImporter core反而直接注册GLSL/SPIR-V函数handler，只把WGSL注册为“插件未安装”的DiagnosticOnly fallback。于是package/catalog声称的owner与真正能执行的owner相反，安装/禁用/选择package无法可靠决定行为。

旧family package又注册WGSL/GLSL/SPIR-V三个handler。它故意不发布WGSL capability，却让自己的WGSL descriptor要求该capability；当前registry并不执行required-capability admission，`FunctionAssetImporter`总是报告Available，所以该handler仍能执行。若新旧package同时手工注册，新WGSL priority 120覆盖旧family 100；若只注册旧包，它又绕过自己缺失的capability继续工作。required capability已经退化为说明文字，不是安全或可用性合同。

两份source parser至少会真正调用Naga，属于可保留基础；但输出还不是工程级shader artifact。所有WGSL、GLSL、compute和SPIR-V都固定写`ShaderAssetKind::Surface`，binding/resources/pipeline layout/import graph/defines/options/render state均为空；readiness只要存在WGSL文本且entry-point字符串合法就可报告ready，甚至不要求entry point或pipeline layout。编译又以`Capabilities::all()`验证，不绑定目标GPU/backend；generic `.glsl`无法识别stage时静默回退vertex。这样的asset可以在导入时成功、在material/PSO/device阶段才失败，且没有结构化source span和target artifact provenance。

本轮登记 **5项P0、54项P1、12项P2**。推荐硬切不是继续增加第四个parser，而是先选定唯一source frontend owner，删除core/plugin/legacy三套并行authority；再建立`ShaderSourceGraph -> validated IR -> reflection/interface -> target artifact -> PSO`合同。没有真实native importer bridge前，两份package都必须撤销NativeDynamic支持；没有target qualification和结构化reflection前，也不能把“parse通过”当作可渲染shader ready，更不能用其支持“超过Unreal”的性能或表现声明。

## 2. 审查边界与物理证据

### 2.1 逐文件范围

| 集合 | 文件 / LF行 / bytes | 本轮证据 |
|---|---:|---|
| `shader_wgsl_importer`全量 | 7 / 590 / 20,618 | plugin manifest、runtime/dist Cargo、4个Rust；7个test attributes、0 ignored |
| `asset_importers/shader`全量 | 7 / 879 / 30,243 | plugin manifest、runtime/dist Cargo、4个Rust；9个test attributes、0 ignored |
| 合并范围 | 14 / 1,469 / 50,861 | 8个Rust、1,278行Rust、45,032 bytes；16个test attributes |
| 产品装配 | first-party provider/static catalog、builtin catalog、App delegation | E3追踪selection -> registration -> importer registry -> import caller |
| 下游合同 | core importer、ShaderAsset/readiness、material/PSO、artifact cache、native replay | E3确认重复实现、假capability、假ready与dist空行为 |

指纹按相对路径排序，对每个文件计算SHA-256，再对`path + space + hash + LF`清单计算SHA-256。新WGSL包为`f44ed27a44a8c8a8a4c0db6f375f8ff11c5cdc71cc4ae5e1b749682cb4d37a1c`，旧family包为`5d4f78694a8d3719967c0b050030a7982ac92bc698339276e9aa6ad0a717347c`，合并范围为`80fe164a24575258412e744e08a40daf568d3d55f15b3856091428fb4f00b4cf`。成文时两包`git status --short --`为空，因此`source_recheck_required: false`；这只固定当前review输入，不表示实现完成。

### 2.2 实际闭合的链路

本轮逐层核对：

1. static `plugin.toml` -> Rust declaration -> programmatic package manifest；
2. static manifest inventory -> builtin metadata catalog -> project selection；
3. first-party provider catalog -> App registration collection -> RuntimeExtensionRegistry；
4. core builtin/DiagnosticOnly importer -> plugin function importer -> priority/availability selection；
5. source bytes/settings -> Naga parse/validate/emit -> `ShaderAsset`；
6. `ShaderAsset` -> readiness/material validation/artifact cache -> render pipeline/PSO consumer；
7. native dist entry -> registration manifest extension -> native live-host replay；
8. plugin tests/CI ->真实产品安装、选择、导入、target compile与GPU执行预期。

静态manifest出现在`STATIC_PLUGIN_MANIFESTS`只证明metadata文件可解析，不证明provider crate已链接。`AssetImporterDescriptor.required_capabilities`存在也不代表registry会执行它；当前availability来自handler的`capability_status()`，Function handler缺省恒为Available。必须以实际代码路径而不是字段名称判断能力。

### 2.3 可保留基础

- 新WGSL包会执行WGSL parse、完整Naga validation并提取entry point；旧family可解析WGSL/GLSL/SPIR-V并把GLSL/SPIR-V发射为WGSL。
- importer registry已有COW generation、ID与同priority matcher冲突检查、availability优先级和plugin owner撤销索引。
- `ShaderAsset`已有source language、entry points、dependency/import、definition、property/options、render state、resource与pipeline layout结构槽位。
- Runtime已有`.zmeta` shader package、readiness、material validation、artifact cache、shader prewarm与PSO基础；本报告要求raw frontend接入这些合同，不重复Runtime09C算法finding。
- package声明、dist entry与CI结构已经标准化，可在删除假NativeDynamic前复用manifest/scaffold机制。
- 参考实现表明不必复制Unreal体量：Bevy已展示source/import/dependency/defs/device capability组合，Fyrox展示shader资源/资源binding/pass边界，都是可渐进吸收的较小合同。

## 3. P0：当前控制面会把错误shader路径报告为可用或ready

### PLUGIN-SHADER-IMPORT-P0-001 · Catalog列出两包，但默认App无法链接任一source provider

`first_party_runtime_catalog/src/tests/generated_manifest.rs`同时收录`asset_importer.shader`与`shader_wgsl_importer`，builtin catalog还为新WGSL包建立stable/partial row。实际catalog Cargo没有两包依赖，`first_party_registration_for_runtime_plugin()`也没有对应分支；App只委托该函数。选择source package时registration被静默跳过，默认AssetImporter仍只有core GLSL/SPIR-V实现与WGSL DiagnosticOnly fallback。

必须让catalog metadata和linked provider从同一generated product graph产生。selected source package缺少编译provider时应返回`ProviderNotLinked`并阻断capability，不能跳过；若GLSL/SPIR-V决定永久内建，就删除其plugin owner宣称并明确core capability。默认产品集成测试从project selection开始，证明WGSL导入、禁用和缺包状态真实变化。

### PLUGIN-SHADER-IMPORT-P0-002 · required capability不参与importer admission，旧包可绕过自己的缺失能力执行

旧family declaration只发布`runtime.asset.importer.shader.naga`，测试明确断言不含WGSL capability；它的WGSL descriptor却要求`runtime.asset.importer.shader.wgsl`。package validation只检查capability namespace/重复，registry把Function handler一律标Available，selection按availability/priority选择，不检查available capability set。core GLSL/SPIR-V函数handler也要求相应capability但仍直接可用。

必须在detached registry materialization时以resolved product capability graph校验每个handler的requirements，并把provider ID/version/generation写入availability。缺少任何required capability时只能注册为Blocked或不注册；运行期capability撤销要原子发布新registry generation。禁止用descriptor字段冒充已执行的admission。

### PLUGIN-SHADER-IMPORT-P0-003 · 两份NativeDynamic dist都无法提供任何import行为

两份native projection都声明`runtime.asset.importer.shader` extension、零systems；dist无command/event/bridge/host-ready回调，诊断还明确说importer仍由runtime module托管。当前native replay在systems为空时直接返回空成功report，即使非空也不消费extensions。cdylib虽然链接runtime crate，Naga函数没有任何跨ABI入口，host无法提交source bytes/settings或取回typed outcome/diagnostics。

必须在typed native asset importer ABI完成前从default packaging和distribution forms撤销NativeDynamic。未来ABI需覆盖size/deadline/cancel、allocator provenance、source/settings schema、structured diagnostics、dependency/output manifest、artifact digest和unload quiescence；loader无法重放的extension必须fail-closed。source/dist等价测试要实际导入同一fixture并比较canonical artifact。

### PLUGIN-SHADER-IMPORT-P0-004 · importer把所有stage固定成Surface并生成结构空洞却可报告ready的ShaderAsset

两份plugin实现与core副本都无条件写`kind: Surface`，包括`.comp`与含compute entry的SPIR-V；同时把resources、pipeline layout、dependencies、source files、imports、shader defs、render state等全部置空。`ShaderReadinessReport::is_ready()`只检查runtime WGSL、entry point/definition字符串diagnostic和validation string为空，不要求至少一个entry point、不验证kind与stage组合，也不要求resource/pipeline interface。无entry-point module、compute-as-surface或有bindings但空layout的asset都可能成为ready。

必须从validated IR派生typed stage set、workgroup size、IO、binding/resource layout、push constants/overrides和required device features，并验证asset kind。readiness至少闭合source graph、entry point selection、interface reflection、target compile和pipeline-layout compatibility；library/include与surface/compute/fullscreen必须分型，不允许用默认Surface掩盖未知语义。

### PLUGIN-SHADER-IMPORT-P0-005 · 编译前端不绑定目标能力且会静默猜错GLSL stage

validator使用`Capabilities::all()`，没有把target backend、adapter features/downlevel flags、shader model或product policy传入。generic `.glsl`未提供`shader_stage`时从文件stem猜测；未知stem通过`.or(Ok(Vertex))`静默降为Vertex。导入可接受目标设备不支持的语言能力，或把语义可解析的generic shader以错误stage编译，失败直到Wgpu/PSO阶段才出现。

必须建立`ShaderCompileTarget`，包含backend/platform/device feature tier、validation policy、source language profile、stage/entry point和compiler version。无法确定stage时返回结构化AmbiguousStage，不得猜测。target-independent IR只能作为中间artifact；每个shipping target仍需独立compile/validate并记录required-vs-granted capability receipt。

## 4. P1：工程级完整性差距

### 4.1 Owner、迁移、catalog与capability

1. README称旧family只是迁移期aggregator，但没有删除日期、consumer清单、deprecation状态或hard-cut gate。
2. core、旧family和新WGSL包三份Naga frontend已发生代码复制，validation helper与错误文本可独立漂移。
3. 旧family priority 100与新WGSL priority 120形成隐式迁移策略，product graph没有解释为何选择其中一份。
4. registry只拒绝同priority重复matcher，允许不同priority无限叠加；没有明确override authority、trust或compatibility约束。
5. `slot`作为最终tie-break使同availability/priority/suffix选择依赖注册顺序，跨product link order可能改变owner。
6. `asset_importer.shader`有`RuntimePluginId`常量但没有builtin catalog row或provider branch，是孤立identity。
7. static manifest inventory与provider catalog分别维护，测试只验证前者generated header，未验证每个runtime-backed ID可materialize。
8. App source assertion列出两包crate名，却只断言App不直接依赖它们，反而没有断言catalog真实依赖和分支存在。
9. capability状态stable/partial、handler availability和target compile qualification是三套互不关联状态。
10. package enable/disable只移除plugin-ownedhandler；core复制的GLSL/SPIR-V行为仍存在，用户无法真正禁用shader frontend。
11. root WGSL package与family package均声明client runtime和Editor host，但没有解释为何shipping client需要source importer而不是cooked artifact。
12. importer/provider没有generation-qualified identity，reimport结果无法证明来自哪一份重叠frontend。

### 4.2 Source frontend、语义分析与diagnostics

13. `source_text()`克隆完整bytes后再UTF-8解码，导入峰值至少保留bytes与String两份source；包内没有shader专用bytes/token/depth预算。
14. SPIR-V原始bytes被hex编码存入`source`，体积翻倍，且artifact同时保留emitted WGSL。
15. importer version固定1，没有与Naga 29.0.1、frontend flags、emitter flags或schema revision绑定。
16. `Capabilities::all()`还忽略trusted/untrusted shader policy；没有bounds-check/robustness等级或沙箱来源区分。
17. WGSL/GLSL错误被压成带URI的String，缺code、severity、file/span、line/column、note/fix-it与include stack。
18. Naga validation错误使用Display而非source-mapped emission，Editor无法稳定跳转到原文件。
19. GLSL转WGSL不保存source map，后端validation/PSO错误无法映射回GLSL。
20. SPIR-V不保存debug/source mapping、entry point selection或specialization constant schema。
21. generic `.glsl`的stage只来自可变TOML string或文件名启发式，没有sidecar schema/version/validation receipt。
22. `.vs/.fs/.cs`短扩展没有明确语言/profile/version，可能与其他资产或平台约定冲突。
23. importer接受module中多个entry points，但没有选择策略、pipeline grouping或per-entry target artifact。
24. module无entry point仍可成功；library/include用途没有独立kind与dependency contract。
25. compute entry没有workgroup size、storage access、dispatch policy或required feature reflection。
26. vertex/fragment跨stage IO没有在同一pipeline组合上验证location/type/interpolation兼容。
27. binding group/index/type/visibility没有从Naga IR投影到`pipeline_layout`和`resources`。
28. override/specialization constants、push constants、early depth、subgroup/ray query等required features没有artifact字段。
29. source import/include graph始终为空；旧family没有接入`.zmeta` shader package的redirect/dependency机制。
30. validation只证明Naga module自洽，不证明Zircon material ABI、built-in bindings、render pass和vertex layout兼容。

### 4.3 Compiler、artifact、cache与Runtime安装

31. 输出主要是source DTO，不是带compiler/target/input graph digest的immutable compiled artifact。
32. `wgsl_source`与原source同时进入cache payload，缺少内容去重、compression和large-source budget。
33. artifact key是否失效依赖外层generic importer version；包自身没有显式compiler fingerprint或Naga ABI hash。
34. GLSL/SPIR-V转换在import时完成，却没有保存normalized IR或emission options，无法精确重现差异。
35. pipeline layout为空仍可ready，PSO阶段只能重新推断或晚期失败，source/artifact authority断裂。
36. 没有per-target WGSL/SPIR-V/MSL/HLSL/binary artifact、driver cache key或last-good generation。
37. 没有shader dependency invalidation、include cycle、dependent pipeline dirtying和incremental recompile接入证明。
38. 没有variant domain、define normalization、constraint、strip、usage tracing和cardinality budget。
39. HLSL/CG/FX只有DiagnosticOnly项，没有DXC/toolchain provider interface、process isolation、version与license/provenance。
40. cook/export没有证明source shader是否被剥离、目标artifact是否齐全或运行时是否仍依赖Naga frontend。
41. compiler crash/timeout/OOM、cache corruption与last-good rollback没有包级operation/receipt。

### 4.4 Dist、ABI与生命周期

42. dist entry要求的host capability正是package自身root capability，requested/provided/host-service语义混合。
43. native manifest的extension schema没有对应decoder/materializer，parser接受字段造成虚假兼容感。
44. dist声明`is_stateless=true`，但真实compiler service需要cache、in-flight job、toolchain process和diagnostic lifetime owner。
45. 没有save/restore可以接受，但也没有unload/quiesce，未来异步compiler扩展容易直接违反DLL寿命。
46. dist单测只查descriptor/report/manifest指针，不解码extension、更不调用import。
47. CI matrix只check/build cdylib，不从package目录安装、admit、import、卸载或比较source/dist artifact。

### 4.5 Tests、Editor与产品资格

48. source单测手工调用`plugin_registration()`，无法发现默认App provider catalog无分支。
49. 旧family测试把“不发布WGSL capability”固定为成功，却不测试handler应被blocked。
50. 测试只检查简单triangle WGSL/GLSL，没有compute kind、bindings、multiple entry points、imports、overrides或device capability。
51. 没有测试证明Naga升级会使importer/artifact cache失效。
52. 没有malicious/untrusted shader的token/depth/huge constant/diagnostic amplification/fuzz门。
53. Editor Material/Shader Graph产品面没有消费structured importer diagnostics、source map、artifact generation或target matrix。
54. 没有从实际项目文件经过AssetManager、catalog、cook、Wgpu pipeline到像素输出的端到端required test。

## 5. P2：长期生态、可观测性与竞争性资格差距

1. 缺少第三方shader frontend/provider SDK、conformance fixtures和兼容版本策略。
2. 缺少多语言source frontend到统一IR的feature parity矩阵与deprecation政策。
3. 缺少compiler worker pool、priority、deadline、cancel、memory/CPU quota和fairness治理。
4. 缺少shader compile farm/remote cache的artifact trust、tenant isolation和determinism协议。
5. 缺少每个shader/variant/pipeline的compile time、cache hit、binary size、instruction/resource统计。
6. 缺少Editor可导出的include/dependency/variant/provider graph与“为什么被选中”解释。
7. 缺少跨GPU vendor/driver/backend的compile、validation、pixel和crash corpus。
8. 缺少shader artifact签名、publisher、source trust和用户生成shader沙箱模型。
9. 缺少hot reload时旧pipeline retirement、in-flight frame、material generation和rollback组合故障测试。
10. 缺少variant stripping前后覆盖率、误删检测、warmup命中与stutter预算。
11. 缺少source/diagnostic本地化、IDE/LSP、fix-it和批量reimport UX资格。
12. 缺少同场景同画质下与Unreal shader compile/cache/warmup的可重复吞吐、延迟、峰值内存与运行时stutter对照。

## 6. 参考引擎对照

| 参考 | 可借鉴责任边界 | Zircon应吸收 | 不应误推 |
|---|---|---|---|
| Unreal ShaderCompilerCore / Preprocessor / Job Cache | versioned worker input/output、compile environment、input hash、structured source diagnostic、recursive include dependency、DDC/job cache与pipeline job | 把source graph、target、compiler、environment、reflection、artifact和diagnostic贯成同一job/identity链 | 不照搬所有宏、worker进程和platform abstraction层级，也不假设Unreal不存在compile stall或cache缺陷 |
| Bevy Shader / ShaderCache | source保留imports/additional imports/defs/file dependencies/validation policy；cache等待import、追踪dependent pipeline，并用device features映射Naga capability | 即使较轻量，也应让imports、defs、dependency、device capability和pipeline dirtying成为真实状态 | Bevy runtime shader模型不等于大型离线cook farm、商业Editor或native plugin ABI |
| Godot ShaderLanguage / ShaderCompiler | typed language/parser/semantic compiler与renderer-specific generated code分层，错误保留语言位置和mode语义 | frontend语义、shader kind/mode、generated target code和diagnostic不能压成一段WGSL字符串 | Godot自有shader语言与module模型不能直接替代Zircon多语言/第三方DLL设计 |
| Fyrox Shader resource | shader资源显式拥有resources、binding、passes、draw parameters与disabled passes，loader只负责形成完整resource | raw language importer必须接到render pass/resource interface，而不是默认空字段也ready | Fyrox当前格式和OpenGL取向不是跨backend编译上限 |
| Unity Graphics shader preprocessing/stripping | build scope按active pipeline/settings处理variant strip并生成report | cook需要真实active profile、variant usage和可审计strip report | 本地Graphics镜像不含Unity完整ShaderLab/HLSL importer与compiler service，不能用来断言其全部source pipeline |

对照重点是source-to-artifact责任链，而不是支持语言数量。Zircon当前能用Naga解析几个fixture不等于拥有工程级shader compiler；首先要证明相同source在Editor、cook、export和目标GPU上得到同一qualified artifact与可追踪诊断。

## 7. 目标架构

### 7.1 唯一Frontend Owner与硬迁移

选择一种明确模型：要么`shader_importers`单一family拥有WGSL/GLSL/SPIR-V及可选HLSL provider，要么每种语言独立package；两者不能同时存在。core仅保留`ShaderCompilerService`合同和无provider时的fail-closed诊断，不再复制parser。legacy package通过manifest migration把project selection映射到新ID后删除，priority不能充当迁移协议。

每个importer registration绑定`provider_package_id + artifact_kind + frontend_version + generation + requirements`。Registry materialization从resolved capability graph计算availability；selector返回provider receipt，tie必须由显式project policy解决，不能用注册slot。

### 7.2 Source Graph、IR、Reflection与Target Artifact

建立versioned链：

`ShaderSourceDocument -> ShaderSourceGraph -> ValidatedShaderIr -> ShaderInterfaceReflection -> TargetShaderArtifact -> PipelineArtifact`

SourceGraph记录canonical URI、content digest、imports/includes、defines、settings和source map。IR记录language/profile/stage/entry points与required capabilities。Reflection记录IO、bindings、resources、push constants、overrides、workgroup和material/pass ABI。Target artifact绑定backend/platform/device tier、compiler build、flags与binary/WGSL digest；pipeline artifact再绑定render pass、vertex layout、material variant和PSO state。

### 7.3 Compiler Operation与Native ABI

`ShaderCompileRequest`必须有bytes/items/token/depth/diagnostic预算、deadline/cancel、target和trust policy；`ShaderCompileReceipt`返回structured diagnostics、dependency manifest、reflection、artifacts、cache provenance和terminal status。大文件与worker在受控线程/进程执行，不阻塞AssetManager主调用链。

Native provider使用版本化buffer/handle ABI和typed compile bridge；host在任何source进入provider前完成signature/trust/capability admission。若不能实现跨ABI的安全诊断、artifact与cancel，就只支持SourceTemplate/LibraryEmbed，并从manifest删除NativeDynamic。

### 7.4 Editor、Cook与Runtime分层

Editor消费SourceDocument与structured diagnostics，preview安装last-good target artifact；cook从resolved product graph编译/strip全部required variants并原子发布manifest；shipping runtime默认只加载qualified artifact，不携带source frontend。Runtime hot reload使用generation、PSO compile状态、old-frame retirement和rollback，不直接覆盖当前pipeline。

## 8. 重构路线

### M0 · 固定假可用与重复owner

- 增加project selection测试，证明两包当前source provider不可达并改为显式`ProviderNotLinked`。
- 增加required capability缺失测试，Function importer必须Blocked。
- 固定compute-as-surface、empty-entry ready、empty-layout ready与silent vertex fallback失败fixture。
- 固定两份native dist零import behavior，并暂时标Unsupported。

### M1 · Canonical package/catalog硬切

- 决定单family或per-language owner，生成manifest/catalog/provider mapping。
- 迁移旧`asset_importer.shader` selection后删除legacy frontend与core复制。
- registry采用provider/generation/capability admission和显式override policy。
- static manifest、linked provider、builtin metadata与App source assertions使用同一generated inventory。

### M2 · Validated IR与reflection artifact

- 定义SourceGraph/IR/reflection/target artifact schema与migration。
- 从Naga投影stage、entry point、IO、bindings、resources、workgroup、overrides和required features。
- 建立structured source span/source map diagnostics与target capability校验。
- readiness改为artifact qualification，不再以WGSL字符串存在为充分条件。

### M3 · Compiler job、cache与cook

- 引入bounded/cancellable compile operation、worker isolation和last-good。
- cache key覆盖完整source graph、compiler build、target、flags、interface和schema。
- cook生成per-target artifact/variant manifest、strip report和shipping completeness gate。
- dependency变化只使受影响shader/pipeline generation失效。

### M4 · Native provider与Editor产品链

- 实现typed native asset importer/compile ABI或永久限制为linked forms。
- source/dist对同fixture产生相同canonical IR/reflection/artifact digest。
- Editor接入diagnostic navigation、target matrix、provider graph、compile progress/cancel与last-good preview。

### M5 · GPU、故障与竞争性资格

- 覆盖多backend/vendor/driver、device feature缺失、worker crash/timeout/cache corruption/hot reload rollback。
- 建立import -> cook -> Wgpu pipeline -> pixel golden端到端required lane。
- 固定compile throughput、cold/warm cache、peak RSS、artifact size、startup/warmup与frame stutter统计协议。

## 9. 验收门

1. 每个selected shader package都能materialize唯一provider；缺link/artifact/capability时结构化blocked且零handler。
2. core、legacy与new package不再并行拥有同一source frontend；project migration后旧ID不能继续注册。
3. importer selector结果与注册顺序无关，并可回溯provider/version/generation/override policy。
4. compute/fullscreen/include/surface kind与entry point集合正确；零entry point不能冒充可执行shader。
5. reflection覆盖IO/binding/resource/workgroup/override/push constant与required device features，并与pipeline layout一致。
6. generic GLSL缺stage时明确失败；target不支持能力在artifact publication前失败。
7. structured diagnostics可从generated WGSL/target compiler映射回原source URI/span/include stack。
8. cache/cook artifact绑定source graph、compiler、target、flags、schema和digest；Naga升级强制失效。
9. NativeDynamic要么真实完成source/dist等价import与安全unload，要么不出现在支持矩阵。
10. shipping产品无需source frontend即可加载全部required shader/variant/PSO artifact；缺失项阻断package。
11. Editor reimport/compile支持cancel、last-good、diagnostic navigation和真实GPU preview，不用字符串成功反馈。
12. 同一BuildSet与scene的Editor/Play/export在目标backend产生相同shader artifact inventory和pixel evidence。

## 10. 验证边界与Owner

本轮只做静态E3审查与文档登记，没有修改两包、Runtime、App、manifest、tests或CI，也没有声明Cargo/GPU测试通过。16个包内test attributes主要手工调用registration或检查native指针，不能覆盖默认产品不可达、required capability不执行、target artifact和GPU行为。实施时先用M0的窄catalog/registry/importer测试恢复可复核失败，再扩大到cook与GPU lane。

Owner边界如下：

- 本报告拥有shader importer package迁移、provider/catalog、source frontend、native dist与产品装配；
- Runtime04拥有通用AssetManager、source/artifact/cache/transaction合同；
- Runtime09C拥有shader/material/variant/pipeline/PSO与GPU执行；
- Plugins01拥有通用package/native ABI/trust/unload；Plugins04拥有native extension replay共享缺口；
- Editor15拥有Material/Shader Graph authoring消费面；Tooling08拥有共享DDC基础设施。

实施不能在core与两个package中分别修三次Naga parser。M1先硬切唯一owner，后续IR、artifact、Editor和GPU证据都只接该owner。
