---
related_code:
  - zircon_plugins/rendering
  - zircon_plugins/solari
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/entry_runner/editor/composition.rs
  - zircon_app/src/entry/export_bootstrap.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/rendering_features/rows.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_definition_collection.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_resolution.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/project_extension_report/feature_merge.rs
  - zircon_runtime/src/plugin/native_plugin_loader/registration_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/registration_replay.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09f1-environment-sky-ibl-reflection-probe-review.md
  - docs/plans/optimize/zircon_runtime/09f2-baked-lighting-lightmap-irradiance-volume-review.md
  - docs/plans/optimize/zircon_runtime/09g1-volumetric-fog-froxel-review.md
  - docs/plans/optimize/zircon_runtime/09g2-advanced-surface-lighting-review.md
  - docs/plans/optimize/zircon_runtime/09h2-exposure-color-bloom-dof-motion-blur-ssr-terminal-review.md
  - docs/plans/optimize/zircon_editor/22-render-pipeline-frame-capture-lighting-bake-reflection-probe-post-process-debug-authoring-review.md
  - docs/plans/optimize/zircon_tooling/03-export-preset-build-cook-pack-platform-bundle-release-review.md
  - docs/plans/optimize/zircon_tooling/08-shared-derived-data-cache-build-cache-remote-execution-artifact-reuse-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RendererInterface.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/ShaderCompiler/ShaderCompilerJobCache.cpp
  - dev/bevy/crates/bevy_render/src/lib.rs
  - dev/godot/servers/rendering/renderer_rd/renderer_compositor_rd.cpp
  - dev/godot/servers/rendering/renderer_rd/storage_rd/material_storage.cpp
  - dev/Fyrox/fyrox-impl/src/plugin/mod.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/BuildProcessors/CorePreprocessBuild.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderPipeline/RenderPipelineGlobalSettings.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: false
---

# 04 · Rendering Umbrella、Feature Bundles、Solari Native Provider 与产品装配工程化差距

## 1. 结论

`zircon_plugins/rendering`不是一个完整渲染器，而是一个 umbrella package、15 对 Runtime/Editor feature crate、一个 native dist 壳和少量 feature-owned helper 的组合。真正的 fog、OIT、light cookie、irradiance volume、planar reflection、subsurface scattering、SSAO 与 reflection capture 等机制大多位于`zircon_runtime::graphics`，部分实现已有真实 GPU pass、资源合同和产品测试基础；这批基础应保留。本报告不重复 Runtime09 系列已经拥有的算法、画质和 GPU lifetime finding，而是审查这些实现是否被插件 package、catalog、Editor preview、export bootstrap、native dist 和 capability truth 真实交付。

答案目前是否定的。普通 Editor 启动只收集 Rendering umbrella 的`RuntimePluginRegistrationReport`，没有收集任何一个 feature bundle 的`RuntimePluginFeatureRegistrationReport`。Catalog却先从 package manifest 的 optional feature metadata构造定义，只验证owner/provider plugin、target和依赖能力，随后把 feature记为available并发布capability；扩展合并阶段找不到具体registration时会静默跳过。因此用户可以得到“feature enabled/available/capability satisfied”的控制面结论，而RenderFeature、executor、component、shading model等数据面贡献为零。Generated export又走另一条会链接并提交feature registration的路径，导致Editor preview与导出产品可能使用不同渲染组合。

Rendering root还把package标为`stable`、根capability标为`complete`，默认启用post process、SSAO、reflection probes和baked lighting；其中post process与baked lighting executor明确只返回`Ok(())`，reflection probes descriptor没有pass/executor，只有SSAO进入generic compute路径。Solari source路径至少诚实发布`Unavailable` provider，但native projection声明的`runtime.render.solari_provider` extension不会被native replay消费：replay只处理`systems`，而Solari为零system，直接以空report成功返回。source与dist因此不是行为等价的交付形态。

本轮登记 **5项P0、56项P1、12项P2**。首要硬切是让feature可用性fail-closed：没有具体provider registration、可执行贡献、目标artifact和健康receipt时不得发布feature capability；然后统一source/export/native的同一canonical feature bundle，并把Rendering每项能力按`metadata-only / core-backed / feature-owned / executable / qualified`分级。未完成前，不能把15个feature crate、Rendering NativeDynamic包或Solari dist计为工程级可安装渲染能力，更不能据此宣称表现或性能超过Unreal。

## 2. 审查边界与物理证据

### 2.1 逐文件范围

| 集合 | 文件 / LF行 / bytes | 本轮证据 |
|---|---:|---|
| Rendering物理目录 | 173 / 9,037 / 395,923 | 151个tracked文件；15个feature的runtime/editor、root runtime/editor、dist、manifest及24个物理shader cache artifact |
| Solari物理目录 | 7 / 469 / 17,053 | runtime source provider、dist entry、两份manifest/Cargo与tests |
| 两域Rust | 118 / 8,021 / 289,261 | 68个test attributes；3个`#[ignore]`产品/GPU测试 |
| 产品组合 | first-party runtime/editor catalog、App Editor composition、generated export bootstrap | E3逐`manifest -> registration -> selection -> catalog -> extension merge`链 |
| Native交付 | registration manifest parser、live-host replay、source/dist declaration | E3确认extension可解析但不会重放，零system提前成功返回 |

行数按文件bytes中的LF计数，最后一行无LF时补1；二进制cache只用于物理账本，不冒充源码行。指纹按相对路径排序，对每个文件计算SHA-256，再对`path + space + hash + LF`清单计算SHA-256。Rendering物理快照为`d64fa9672379c97b53389c8c1d6278a7201ac5dc867ae91cd65dae661c4c657d`，Solari为`7f2c429532d1e145408b9bd4d4aa4263deb02d2c76978e81068fcf619baa246a`，合并范围为`049b747d3e08c280e4550fe6a0a4724ff5f7143c0d091a6a0be4c85996ef3794`。

成文时`git status --short --`在Rendering、Solari及相关catalog/source装配范围为空，因此`source_recheck_required: false`。Rendering目录内另有22个ignored cache文件，物理指纹仅固定本轮观察快照；它们不是tracked source currentness依据。两份已tracked的contact-shadow cache artifact仍纳入151个tracked文件，属于本报告finding而不是审查工具生成的新改动。

### 2.2 实际闭合的纵向链

本轮逐层检查了以下路径：

1. `plugin.toml`与Rust declaration -> package manifest -> builtin/first-party catalog；
2. project plugin/feature selection -> feature definition map -> dependency resolution -> available capability publication；
3. concrete feature registration -> extension registry merge -> RenderFeature/executor/component/shading model；
4. App Editor source bootstrap -> preview runtime；
5. generated export source -> linked feature provider -> exported runtime bootstrap；
6. native candidate -> dist entry -> V3 registration manifest -> live-host replay；
7. Rendering editor crates -> first-party Editor catalog -> view/command/document/controller/resource contribution；
8. feature tests -> Wgpu render framework -> cache root/artifact -> dynamic evidence。

这里必须区分package metadata与运行行为。`PluginFeatureBundleManifest`可以描述feature ID、依赖、target和capability，但它不能替代`RuntimePluginFeatureRegistrationReport`；后者也只有在实际合并到产品registry并执行pass/provider时才形成能力证据。单元测试手工调用`plugin_feature_registration()`或直接把descriptor/executor传给Wgpu framework，只证明局部实现可构造，不证明App产品会装配它。

### 2.3 可保留基础

- 15个feature具有统一package/owner命名和runtime/editor物理边界，export generator也已有按selection生成linked provider source的机制。
- contact shadow包含真实Wgpu compute pipeline、bind group、dispatch和validation；SSAO descriptor可进入通用compute执行路径。
- volumetric fog、OIT、light cookie、irradiance volume、planar reflection、subsurface scattering已有Runtime core-owned registration，plugin adapter可作为显式启用层而不是重写算法。
- reflection probe已有六面capture/persist helper与Editor trigger类型，可作为未来typed operation的底座。
- feature resolver已有依赖图、target过滤、blocked report和确定顺序；问题是admission输入缺少concrete provider gate，不是必须推倒整个solver。
- Solari source provider明确返回`Unavailable`与诊断，至少没有把未实现executor伪装为ready。
- native registration manifest已有schema版本、资源访问计划和external system replay，可以在同一事务框架内扩展typed extension/provider replay。

这些基础只能证明有可重构资产，不能抵消产品装配与truth contract缺口。

## 3. P0：控制面与实际渲染行为直接矛盾

### PLUGIN-RENDERING-P0-001 · Metadata-only feature会被解析为available并发布capability

`feature_definition_map()`先调用`merge_package_feature_definitions()`，把Rendering root manifest中的15个optional feature全部放入definition map。`feature_status()`只检查owner/provider plugin是否enabled、target是否支持和dependency capability是否存在，不要求具体`RuntimePluginFeatureRegistrationReport`。`resolve_pending_feature_dependencies()`一旦状态available，就把feature ID写入`available_features`并把feature capability加入全局集合。最后`merge_available_feature_extensions()`仅在feature registration数组里搜索匹配项，找不到就直接结束该iteration，没有fatal diagnostic。

普通Editor只提交umbrella runtime plugin registration，不提交15个feature registrations，所以当前路径可稳定产生“available feature + published capability + zero runtime contribution”。必须把concrete provider、artifact kind、registration digest和runtime initialization receipt纳入feature admission；缺失时应为`DeclaredButUnlinked`或`ProviderMissing`，禁止发布capability并输出可定位fatal diagnostic。feature capability只能在detached registry成功materialize并commit后发布。

### PLUGIN-RENDERING-P0-002 · Editor preview与generated export使用不同feature装配协议

App普通Editor路径调用`bootstrap_with_runtime_plugin_registrations`，first-party runtime catalog只链接Rendering umbrella；generated export bootstrap则接受`RuntimePluginFeatureRegistrationReport`，模板会为默认与选中的Rendering feature生成`ExportRuntimePluginFeatureRegistrationProvider`并提交registration。相同project manifest因此可能在Editor里只有metadata capability，在导出程序里真正安装某些descriptor/executor；反方向的link/profile差异也没有被preview提示。

这会破坏材质、shader、lighting、capture、cook和性能预览的基本等价性。必须生成唯一`ResolvedProductPluginGraph`，由Editor preview、Play、cook和export共同消费；每个节点绑定package/version/SDK/target/artifact/provider digest。CI必须对同一project selection比较四条路径的canonical extension inventory，并运行像素/资源/诊断等价测试，任何差异默认阻断导出。

### PLUGIN-RENDERING-P0-003 · Stable/complete与默认feature包含明确空执行或无执行面

Rendering root manifest标记`stable`，根capability状态为`complete`，默认启用post process、SSAO、reflection probes和baked lighting。post process与baked lighting都注册会被scheduler调用、但只返回`Ok(())`的executor；reflection probes注册的RenderFeature没有pass/executor，真实capture helper与Editor trigger没有产品consumer；只有SSAO包含可执行compute descriptor。当前状态会让用户、Plugin Manager和自动化把“默认启用”误解为可见渲染效果。

必须删除以空executor维持成功路径的合同。每个feature需要分离`declared`、`linked`、`initialized`、`executed_this_frame`、`visually_qualified`和`production_supported`状态；根package maturity由最低默认feature资格决定。未具备真实执行面时只能标`experimental/metadata-only`并从默认集移除。qualification必须绑定scene fixture、render graph节点、GPU marker/readback、golden image、错误注入和性能预算。

### PLUGIN-RENDERING-P0-004 · Solari native dist声明provider extension，但loader解析后完全丢弃

Solari source plugin注册`SolariRuntimeProviderRegistration`，provider返回明确`Unavailable`。native projection却只在registration manifest中声明`point = "runtime.render.solari_provider"`、`contribution = "plugin.solari.runtime"`、schema`zircon.runtime.solari-provider/1`，没有system。V3 parser接受modules/systems/resources/events/extensions/capabilities，但live-host replay在`manifest.systems.is_empty()`时立即返回空成功report；非空时也只遍历systems并填充`registered_systems`，没有消费extensions或events。

结果是source路径至少能向产品解释“provider不可用”，dist路径则静默没有provider、没有诊断、没有行为，仍可被视为成功加载。必须为native manifest建立版本化typed provider contribution replay，或禁止Solari以NativeDynamic交付；未知/未支持contribution kind必须fail-closed，不能解析后忽略。source/dist等价测试要比较provider ID、status、capabilities、diagnostics、lifecycle和卸载撤销。

### PLUGIN-RENDERING-P0-005 · Feature inventory与SDK identity存在多份相互矛盾的当前authority

production builtin catalog已列15个Rendering feature，当前static Rendering/Solari manifest和SDK常量为`0.2.0`；`PluginPackageManifest::new`默认仍为`0.1.0`，builtin Rendering/Solari builder没有显式覆盖SDK version。与此同时Runtime与Editor的多份测试仍断言Rendering恰有9个feature，遗漏volumetric fog、OIT、light cookies、irradiance volumes、planar reflections和subsurface scattering，并继续断言静态manifest为`0.1.0`。

这不是过期注释，而是同一package ID在static file、programmatic manifest、builtin catalog、source registration和测试中具有不同feature set/SDK identity，并已参与当前Editor test build的既有失败面。必须以canonical package schema生成static/source/builtin/export metadata，删除手写重复rows与固定旧集合测试；loader在identity不一致时拒绝admit。迁移前要为现有project selection提供显式versioned migration，不能靠接受两套值维持表面兼容。

## 4. P1：工程级完整性差距

### 4.1 Package owner、catalog、selection与admission

1. Rendering root runtime plugin只注册module descriptor和optional feature metadata，没有`register()`行为；根capability因此不能代表renderer provider已安装。
2. first-party runtime catalog的base组只依赖umbrella crate，没有依赖15个feature runtime crate，默认feature声明与link set由不同owner维护。
3. first-party editor catalog只链接Navigation和Neural，Rendering root editor与15个feature editor crate均不进入默认产品。
4. plugin workspace可独立编译所有crate，但主workspace与App产品只显式链接少数组合；workspace membership不能替代产品reachability。
5. project manifest selection没有记录解析后具体provider package/artifact，只保留feature ID和target policy，无法审计实际materialize来源。
6. package default feature、App Cargo feature、render profile overlay和export template各自维护默认集合，没有单一resolved graph。
7. feature provider selection允许manifest declaration充当definition provider，却没有要求该provider持有registration factory或artifact。
8. capability publication没有provider generation与health；热重载、device loss或初始化失败后旧capability无法被可靠撤销。
9. feature dependency只依赖字符串capability，不表达版本、quality tier、backend、shader model、resource format或mutual exclusion。
10. root`runtime.plugin.rendering`、各feature capability和Runtime core capability namespace混合，无法区分宿主机制与package提供能力。
11. feature disabled/blocked report不包含“linked artifact缺失”“registration缺失”“executor缺失”“visual qualification缺失”等关键状态。
12. Plugin Manager/Editor无法解释一个feature是core-backed adapter、feature-owned executor、metadata-only，还是仅在export中可用。

### 4.2 Source、export、dist与native行为等价

13. Rendering native dist只投影根module，`systems/events/extensions`为空；它无法重建15个feature的RuntimeExtensionRegistry贡献。
14. feature child package只有source/library形态，没有native artifact、serialized contribution或稳定feature-provider ABI。
15. root package默认声明`NativeDynamic`可用，会制造“动态包可交付完整Rendering feature”的错误预期。
16. export generator为每个feature写Rust source并静态链接crate，和native dist的动态选择不是同一artifact model。
17. dist behavior没有host-ready、device-created、device-lost、frame、save/restore或quiesce回调，无法承载真实渲染provider生命周期。
18. Rendering source注册的module descriptor在dist replay中也没有对应RuntimeExtensionRegistry module contribution，只存在registration metadata文本。
19. native manifest parser接受extensions/events但replay不消费，schema能力大于实现能力，其他渲染provider会重复Solari丢失问题。
20. native replay report只列registered systems，无法报告module/resource/event/provider/capability实际materialize inventory。
21. 空systems快速路径不会验证extension schema是否受支持，也不会将unsupported contribution标为fatal。
22. source/dist tests只验证entry指针、manifest字段或diagnostic字符串，不比较运行时registry与行为receipt。
23. 没有按platform/backend验证同一feature在linked source、generated export和NativeDynamic三种artifact kind的支持矩阵。

### 4.3 Feature bundle的真实执行层级

24. baked lighting默认feature只有空composite executor；真正lightmap/irradiance artifact与sampling链由Runtime09F2拥有但未被bundle证明交付。
25. post process默认feature只有空post-stack executor；真实exposure/bloom/DoF/motion blur/SSR chain由Runtime09H2拥有但未被bundle装配。
26. reflection probes默认feature descriptor无pass/executor，capture helper与persist流程没有连接到frame scheduler或Editor operation。
27. decals注册component descriptor与空projector composite，没有volume culling、DBuffer/GBuffer写入、material binding或可见像素资格。
28. shader graph只是少量enum/string WGSL拼接和空post executor，没有typed ports、拓扑、stage legality、binding layout、variant、diagnostic span、artifact或PSO集成。
29. VFX graph只有浅检查、字符串源、固定`[1, 1, 1]` dispatch和两个空executor，没有particle storage、spawn/update/compact/sort/draw/indirect/LOD/budget链。
30. ray tracing policy只注册零pass capability policy，没有BLAS/TLAS、pipeline/SBT、fallback、memory budget或backend provider。
31. contact shadow有真实compute executor，但产品测试直接构造Wgpu framework并手工传descriptor/executor，绕过App selection/catalog。
32. SSAO依赖Runtime内置WGSL和generic compute path，plugin自身不拥有shader artifact/version/quality settings；bundle升级与Runtime shader升级无法独立兼容。
33. volumetric fog只是把三项Runtime core registration转交registry，package capability没有证明资源初始化、frame demand、camera/world绑定和device-loss恢复。
34. OIT的两个core registration同样是thin adapter，没有按material/transparency mode证明真实产品selection会选中正确pass。
35. light cookies依赖core descriptor/executor，但bundle没有asset importer、texture residency、light-component binding与fallback合同。
36. irradiance volumes依赖core registrations，但没有从bake artifact到world probe volume、streaming、sampling和invalid data fallback的package级receipt。
37. planar reflections依赖core registrations，但bundle没有capture scheduling、recursion guard、visibility、resolution budget和multi-view admission。
38. subsurface scattering转交core descriptor/executor与shading model，bundle没有profile asset、material validation、kernel quality和fallback资格。
39. 15个feature没有统一`FeatureExecutionEvidence`，无法区分本帧被cull、未初始化、executor为空、GPU提交失败或视觉输出通过。

### 4.4 Editor authoring与产品UX

40. Rendering root editor crate只声明descriptor/capability，没有注册view、command、document、controller、resource或operation。
41. 15个feature editor crate整体上都是descriptor shell，没有进入first-party Editor catalog。
42. reflection probe editor虽有capture trigger/request/result类型，全仓没有产品consumer，plugin registration仍只返回descriptor。
43. Shader Graph与VFX Graph命名暗示完整authoring工具，但没有graph document、undo/redo、validation、compiler diagnostics、preview或artifact publication surface。
44. baked lighting、irradiance volume与reflection probe没有共享build job、cancel/progress、staging、atomic publish、last-good和scene revision合同。
45. post process、SSAO、fog、OIT、SSS等没有project/volume/camera override的统一settings document与Runtime preview bridge。
46. Editor22已有render pipeline/capture/bake/probe/post authoring差距；这些feature crate没有成为其typed provider，继续形成第二套空壳身份。
47. Plugin Manager无法从实际registration/execution evidence显示“此feature仅导出可用”“本Editor未链接”“native artifact不支持”或“executor为占位”。

### 4.5 Shader cache、artifact与源码卫生

48. feature产品测试使用默认project/cwd派生cache root，导致shader variant写入package源码目录，而不是隔离temp/project cache。
49. contact shadow目录已有6个`.zircon-cache`物理文件，其中2个metadata/WGSL compressed artifact已被tracked；generated DDC进入source authority。
50. volumetric fog目录已有18个ignored`.zircon/cache`artifact，重复测试会改变工作区物理状态且难以归因。
51. cache metadata包含`created_unix_seconds`等wall-clock字段和测试material ID，不能作为可复现source artifact或review fingerprint。
52. package tests没有显式temp root、cleanup receipt、cache key provenance、corruption/eviction/remote policy；Tooling08的DDC合同没有贯通feature测试。

### 4.6 Tests、CI与动态证据

53. 绝大多数feature测试手工调用registration函数，不能发现普通Editor没有收集feature reports。
54. 当前没有负向测试断言“manifest声明但registration缺失”必须blocked/fatal；现有静默跳过被稳定保留。
55. 3个ignored GPU/产品测试覆盖reflection probes与volumetric fog关键路径，默认CI不会执行；其余绿色单测不能替代这些动态证据。
56. 当前Editor共同test lane受既有239个compile errors与122个warnings阻断，本报告没有重复同一不可达命令；因此没有任何新的动态通过结论。

## 5. P2：长期产品化、生态与性能资格差距

1. 缺少跨版本Rendering feature provider compatibility、project migration和package rollback矩阵。
2. 缺少第三方RenderFeature/RenderPass executor的稳定SDK、sample package与ABI conformance suite。
3. 缺少feature组合冲突、order、resource alias、quality tier和backend fallback的SAT/constraint模型。
4. 缺少按scene/camera/view/world记录feature activation、GPU marker和可导出provider graph的诊断工具。
5. 缺少shader/artifact provenance UI，把source、compiler、defines、backend、driver、cache hit与loaded digest连成证据链。
6. 缺少feature安装、启用、热重载、device loss、shader compile failure和卸载的组合故障注入。
7. 缺少default profile在低端、主流、高端与headless target上的预算和自动降级策略。
8. 缺少同画质/同分辨率/同scene/同硬件下与Unreal、Godot、Bevy/Fyrox示例及Unity SRP的可重复性能对照协议。
9. 缺少视觉golden的颜色空间、曝光、随机种子、driver容差、遮罩与统计判定规范。
10. 缺少package体积、shader variant数量、pipeline warmup、startup与runtime memory预算。
11. 缺少Rendering/Solari provider的签名、publisher、entitlement和remote artifact trust证据展示。
12. 缺少废弃metadata-only feature、旧SDK identity与旧project selection的明确支持周期和迁移工具。

## 6. 参考引擎对照

| 参考 | 可借鉴责任边界 | Zircon应吸收 | 不应误推 |
|---|---|---|---|
| Unreal Renderer / Plugin Manager / Shader Job Cache | Renderer module拥有scene/view-family/render lifecycle与扩展点；plugin按loading phase显式装载；shader cache以input/output hash、DDC policy、异步query和memory budget治理 | feature selection必须安装真实module/provider并进入明确lifecycle；cache artifact脱离源码树并绑定完整build/input identity | Unreal庞大实现并不自动证明所有插件热卸载或shader缓存场景无缺陷，也不能照搬其内部类层级 |
| Bevy RenderPlugin | plugin显式创建Render SubApp、schedule、resources、asset loaders并在finish阶段完成GPU resource安装，含设备恢复路径 | “plugin enabled”必须可观察地改变schedule/resource/provider graph；GPU ready与device recovery是lifecycle状态 | Bevy没有Zircon目标中的完整商业Editor/package/native生态，不能补齐authoring和动态ABI结论 |
| Godot Renderer RD / shader cache | compositor明确初始化、逐帧begin/end与finalize GPU资源；shader cache位于user/project cache并区分可写cache与exported baked cache | feature/provider拥有初始化和finalize receipt；test/runtime cache使用隔离project/user root，source package保持immutable | Godot module模型不能直接证明Zircon DLL source/dist等价或动态卸载安全 |
| Fyrox Plugin | Plugin trait具有register/init/loaded/deinit/update/rendering与graphics-context create/destroy等显式hook | 渲染feature需要host/GPU context lifecycle和错误传播，不能只注册descriptor | Fyrox接口较轻，不是shader规模、第三方安全或分发治理上限 |
| Unity Graphics build processors / global settings | build preprocess持有build-scope data lifecycle；RenderPipelineGlobalSettings是与pipeline绑定的真实project asset | Editor、cook和runtime共享同一active settings/artifact graph，build stripping/validation消费真实配置 | 本地镜像只含Graphics packages，不代表Unity完整Editor、Package Manager或Player plugin loader实现 |

对照只用于owner、lifecycle、artifact和产品装配合同，不按代码量排名。性能目标必须在功能与画质等价、错误路径一致、统计方法固定后才有意义；当前Rendering package连preview/export/provider graph等价都没有达到。

## 7. 目标架构

### 7.1 Canonical Resolved Product Plugin Graph

由project selection、product profile、target、installed artifacts和host capabilities生成immutable `ResolvedProductPluginGraph`。每个feature节点至少包含：

- package/feature/provider qualified ID、version、SDK/ABI/schema；
- source artifact与resolved runtime artifact digest；
- requested/provided capabilities及版本约束；
- dependency、conflict、target/backend/quality约束；
- contribution inventory digest；
- admission、initialization、health、generation与qualification状态。

Editor preview、Play、cook、export与standalone runtime必须消费同一graph或提供机器可解释的差异报告。graph只在所有artifact和provider preflight通过后commit；不能从manifest metadata直接生成`available`。

### 7.2 Feature Bundle分层

建立统一`RenderFeatureProviderBundleV1`，明确四层：

1. `FeatureDeclaration`只描述可选能力和约束；
2. `FeatureContributionBundle`提供descriptor/executor/component/shading model/settings schema的canonical inventory；
3. `FeatureRuntimeProvider`负责device/world/frame lifecycle、health、quiesce与unload；
4. `FeatureQualification`绑定测试scene、artifact、GPU/visual evidence与支持等级。

core-backed feature允许贡献引用Runtime builtin implementation，但必须解析为具体implementation ID/version并获得初始化receipt；不能把“core中有同名代码”当作已启用。metadata-only feature不发布用户能力。

### 7.3 Source、Generated Export与Native统一交付

source和generated export可materialize同一canonical Rust bundle；native dist通过版本化serialized contribution加typed bridge materialize等价bundle。若某类executor无法安全跨ABI，则artifact manifest必须明确`NativeDynamic: unsupported`，而不是发布空root package。loader对parser支持但replayer不支持的contribution kind直接拒绝。

Native replay report扩展为modules/resources/events/extensions/providers/systems/capabilities逐项receipt，并支持detached registry事务、rollback、generation撤销和quiescent unload。Solari provider先保留`Unavailable`，但source/dist必须发布同一status与diagnostic；真实executor完成后再提升资格。

### 7.4 Editor与artifact边界

Editor feature crate必须注册真实typed provider，而不是只返回descriptor。graph类feature共享`GraphDocument -> semantic IR -> compiler -> immutable artifact -> preview install`；bake/capture类feature共享可取消job、scene revision、staging、atomic publish和last-good；settings类feature共享project/volume/camera override、validation与runtime generation。

shader/DDC只写显式temp、project DDC或user DDC，cache key包含source/compiler/defines/backend/target/schema；测试必须注入隔离root并验证cleanup。tracked generated cache从source hard-cut后由迁移脚本和CI policy防止回归。

## 8. 重构路线

### M0 · 关闭假能力并固定失败证据

- 增加“声明feature但无registration”的catalog测试，期望`ProviderMissing`且零capability。
- 增加普通Editor与generated export resolved graph diff测试，固定当前15项装配差异。
- 把Rendering root/default feature状态降为真实maturity，移除空executor feature的默认启用。
- 增加Solari native replay测试，固定extension被解析却未materialize的问题。
- 统一15项inventory与SDK `0.2.0` canonical fixture，删除旧9项/0.1.0成功合同。

### M1 · Canonical manifest、artifact与product graph

- 由一份schema生成static manifest、Rust declaration、builtin rows、catalog/export metadata和golden inventory。
- 建立`ResolvedProductPluginGraph`及fail-closed admission reason。
- first-party catalog显式链接被产品支持的feature provider；未链接项显示Unavailable而不是available。
- Editor preview、Play、cook和export共享resolved graph与diff receipt。

### M2 · Feature contribution与native provider ABI

- 引入`RenderFeatureProviderBundleV1`及canonical contribution digest。
- 扩展native replay或明确禁用NativeDynamic；完整处理extension/provider/event并事务化materialize。
- 为Solari建立source/dist同状态provider，再逐步接入真实GPU executor。
- 为core-backed feature固定implementation ID/version/lifecycle，不复制Runtime算法。

### M3 · 清除占位执行与接入Editor authoring

- 删除baked lighting、post process、decals、shader graph、VFX graph空executor成功路径。
- 将Runtime09已有实现通过真实provider接入bundle并补齐execution evidence。
- 把reflection capture、bake、post settings、Shader/VFX Graph接入统一document/compiler/job/artifact/preview合同。
- Plugin Manager展示provider graph、artifact kind、health、qualification和不可用原因。

### M4 · Cache、故障恢复、视觉与性能资格

- cache root全部注入，移除tracked/working-tree generated cache并加CI source hygiene gate。
- 覆盖device loss、shader failure、provider reload/unload、artifact corruption和fallback。
- 建立同语义visual golden、GPU capture、frame/memory/variant预算和跨backend矩阵。
- 只有满足正确性与证据合同的feature才能提升为stable/default，并进入与Unreal的性能比较。

## 9. 验收门

1. 任意selected feature缺少concrete registration/artifact/provider时，catalog返回结构化blocked reason，available capability集合中不存在该feature。
2. 普通Editor、Play、cook、generated export和standalone对同一selection产生相同canonical provider/contribution inventory；允许差异必须由target policy显式解释。
3. Rendering static/source/builtin/export manifest从同一schema生成，15项feature与SDK/version完全一致；CI无手写旧集合。
4. 每个default feature都至少有一次真实Render Graph/GPU执行、可见输出、failure propagation和qualification receipt；不存在空executor。
5. Solari source与native dist提供相同provider ID/status/diagnostic/lifecycle；native loader不再静默丢弃extension。
6. NativeDynamic只对真正可materialize的feature发布支持；source/dist contribution inventory digest可比较且卸载可撤销。
7. Editor Rendering feature具有真实command/document/controller/resource/provider，或明确标为Runtime-only；descriptor shell不计产品功能。
8. 所有shader/cache测试使用显式隔离root，工作区运行前后tracked与ignored package目录均不产生新artifact。
9. ignored GPU tests被纳入有硬件资格的required lane；无硬件时产生明确Skipped/Unavailable evidence而非假绿。
10. visual/performance报告绑定BuildSet、provider graph、scene、camera、quality、resolution、backend、GPU/driver、warmup、采样和artifact digest。

## 10. 验证边界与后续Owner

本轮只做静态E3 review与文档登记，没有修改生产代码，也没有声称Cargo、GPU或像素测试通过。当前Editor共同lane已由既有编译失败阻断，重复相同命令不会增加证据；3个ignored产品/GPU测试也不能作为当前动态资格。实施阶段先恢复M0的最小catalog/native replay测试可达性，再按M1-M4逐层扩大动态验证。

Finding owner分工如下：

- 本报告拥有Rendering/Solari package、catalog、source/export/native等价、feature bundle真实性和产品装配；
- Plugins01拥有通用SDK/package/native ABI、签名、loader admission与卸载安全；
- Runtime09系列拥有具体渲染算法、画质、Render Graph、RHI与GPU lifetime；
- Editor22拥有render authoring产品面，本报告要求feature crate成为其真实provider；
- Tooling03/08拥有export artifact与DDC基础设施，本报告拥有Rendering包对这些合同的接入。

因此后续不能在多个报告里分别修同一个solver、native replay或cache root。先由最低共享owner提供canonical合同，再由Rendering/Solari接入并用产品级证据验收。
