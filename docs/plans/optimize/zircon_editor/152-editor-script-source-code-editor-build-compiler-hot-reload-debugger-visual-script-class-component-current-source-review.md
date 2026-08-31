---
title: Editor Script Source、Code Editor、Build、Compiler、Hot Reload、Debugger、Visual Script、Class 与 Component 当前源码复核
category: zircon_editor
report_id: Editor152
review_date: 2026-08-26
baseline_head: d5d41037e080ecc948a3b13f3e8bab38b4cd708a
verification_head: d5d41037e080ecc948a3b13f3e8bab38b4cd708a
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
canonical_owner: Editor31
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/31-script-source-code-editor-build-compiler-hot-reload-debugger-visual-script-class-component-authoring-review.md
  - docs/plans/optimize/zircon_editor/105-editor-script-source-code-editor-build-compiler-hot-reload-debugger-visual-script-class-component-current-source-review.md
related_code:
  - zircon_editor/src/core/script_build
  - zircon_editor/src/core/logging/jump.rs
  - zircon_editor/src/ui/host/editor_activity_log.rs
  - zircon_editor/src/core/commands/defaults.rs
  - zircon_plugins/first_party_editor_catalog
  - zircon_runtime_interface/src/script_diagnostics/mod.rs
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime/src/asset/project/script_manifest.rs
  - zircon_runtime/src/asset/assets/scene/extensions.rs
  - zircon_runtime/src/scene/world/project_io/script.rs
  - zircon_runtime/src/dynamic_api/session/project.rs
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/script
  - zircon_plugins/zr_vm_language
  - zircon_plugins/first_party_runtime_catalog
  - zircon_app/Cargo.toml
  - examples/woc/zircon-project.toml
  - examples/woc/scripts/woc_game
plan_sources:
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_runtime/21-zr-language-parser-type-system-semir-bytecode-package-loader-vm-runtime-review.md
  - docs/plans/optimize/zircon_plugins/16-first-party-zr-vm-language-source-runtime-dist-catalog-reflection-callsite-host-interface-gc-hot-reload-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/63-runtime-scene-reflection-type-schema-registry-dynamic-component-property-address-inspection-artifact-subscription-editor-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/99l-runtime-scene-reflection-type-schema-registry-dynamic-component-property-address-inspection-artifact-subscription-editor-product-integration-current-source-review.md
  - docs/plans/zircon_editor/editor/13-script-compilation-management.md
  - docs/plans/performance/01/2026-08-23-editor-core-script-build-currentness-revalidation.md
  - docs/plans/performance/01/2026-08-24-plugin-zr-vm-language-current-source-performance-review.md
  - docs/plans/optimize/zircon_editor/31/2026-08-25-watch-budget-accounting.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/Blueprint.h
  - dev/UnrealEngine/Engine/Source/Editor/Kismet/Public/BlueprintEditor.h
  - dev/UnrealEngine/Engine/Source/Editor/Kismet/Public/BlueprintCompilationManager.h
  - dev/UnrealEngine/Engine/Source/Editor/Kismet/Private/BlueprintCompilationManager.cpp
  - dev/UnrealEngine/Engine/Source/Developer/HotReload
  - dev/UnrealEngine/Engine/Source/Developer/Windows/LiveCoding
  - dev/godot/editor/script/script_editor_plugin.h
  - dev/godot/editor/script/script_editor_plugin.cpp
  - dev/godot/core/object/script_language.h
  - dev/godot/core/debugger/script_debugger.h
  - dev/Fyrox/editor/src/plugins/inspector/editors/script.rs
  - dev/Fyrox/editor/src/lib.rs
  - dev/bevy/crates/bevy_reflect/src/type_registry.rs
  - dev/bevy/crates/bevy_asset/src/io/file/file_watcher.rs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Data/Util/GraphUtil.cs
reference_sources:
  - E:/Git/zr_vm/zr_vm_rust_binding/rust/zr_vm_rust_binding/src/lib.rs
  - E:/Git/zr_vm/zr_vm_language_server/stdio/stdio_initialize.c
  - E:/Git/zr_vm/zr_vm_language_server/stdio/stdio_initialize_capabilities.c
  - E:/Git/zr_vm/zr_vm_lib_debug/include/zr_vm_lib_debug/debug.h
  - E:/Git/zr_vm/zr_vm_lib_debug/include/zr_vm_lib_debug/profile.h
  - E:/Git/zr_vm/zr_vm_lib_debug/src/zr_vm_lib_debug/debug_protocol.c
finding_status:
  p0_open: 5
  p1_open: 60
  p1_partial: 0
  p1_closed: 0
  p2_open: 12
  p2_partial: 0
  p2_closed: 0
gate_status:
  fail: 31
  partial: 1
  pass: 0
---

# Editor152 · Script Source / Build / Hot Reload / Debugger / Visual Script 当前源码复核

## 1. 结论

Zircon当前仍没有工程级脚本创作与交付闭环。Runtime的VM底座继续增强：package discovery有深度、条目、manifest/path/payload bytes、wall-time、cancellation与runtime-owner准入；`VmPluginManager`有backend family、slot、generation、stable callback refresh和payload materialization cache；`HotReloadCoordinator`有reflection prepare/commit、package state snapshot/migration、activate/rollback、cooperative GC deadline与FIFO去重。最近在途修改又移除了discovery的process-global task-pool fallback，拆分了GC owner，并优化了state migration、state blob和payload cache的数据结构。这些都是真实基础，应保留。

Editor产品链没有因此闭合。`zircon_editor/src/core/script_build`当前5个文件、1,696行，拥有300 ms debounce、1,000 ms首事件截止、20路径/64 KiB预算、active + one queued request、`Watch < Command < Play`提升、线性dispatch ticket与diagnostic replay cursor；但全仓生产调用仍为零。`CompileModules`、`ValidateLedger`、`RefreshBindings`没有watch owner、Build command、shared job executor、VM compiler adapter、artifact receipt、binding/install receipt、Play waiter或commandlet。最近的Editor31子切片只把重复watch事件的字节预算从重复遍历改成O(1)增量记账，且记录明确标为managed validation pending；它只能让G07变为Partial，不能把状态机改写成产品闭环。

Runtime仍在`load_startup_scripts()`同步discover package，并由ZrVM real backend在同一加载路径中`ProjectWorkspace::open -> compile(incremental=true) -> start_session`。compile只返回compiled/skipped/removed计数，Runtime不要求immutable artifact set、compiler/toolchain identity、source digest、dependency lock、target/profile、debug map或qualified install receipt。Editor Build、Play、Client、Server与未来Cook因而没有共同的产物真值。

默认装配仍自相矛盾。WOC把`zr_vm_language`声明为required并选择`backend = "zr_vm:project"`；`target-client`与`target-editor-host`只启用generic `script`，没有`first-party-zr-vm-language-runtime-plugin`或`backend-zr-vm`。ZrVM插件自身`enabled_by_default(false)`，两项capability均为`Partial`，未启用backend时直到package load才返回`BackendUnavailable`。first-party Editor catalog仍只有Navigation和Neural，没有ZrVM language/editor provider。

脚本资源与场景创作也没有升级。`ResourceKind`没有Script Source、Module、Class、Component、Visual Script、Debug Map或Script Artifact；`SceneScriptBindingAsset`仍只有`package/module/enabled/update/fixed_update + BTreeMap<String, JSON>`，并整体写入动态`script.bindings`组件。Runtime binding key仍由`package::module#array_index`拼接，`onStart`只在Update首次运行，首个binding错误会中止余下binding。package级VM state migration不等于Scene中每个Script Component的stable class/field identity、default/override provenance和字段迁移。

语言工具能力存在于外部`E:/Git/zr_vm`，但没有进入Zircon产品。当前Rust binding的`CompileResult`仍只有三个计数；另一方面LSP直接声明incremental sync、diagnostics、completion、hover、signature、definition/reference/rename、semantic tokens、inlay hints、code actions、formatting、hierarchy与workspace operations，debug库直接提供line/function/data/exception breakpoint、continue/pause/step、stack/scopes/variables、effect-qualified evaluate和profile。Zircon既没有versioned adapter，也没有debug session、source/artifact map或Editor UI；不得再造regex parser、私有简化LSP或静态断点面板。

因此Editor31/105的canonical结论保持：**5项P0全部Open；60项P1全部Open；12项P2全部Open；32个Gate为31 Fail、1 Partial、0 Pass**。目标链仍是`ScriptWorkspace + transactional Source/VisualScript Documents -> canonical semantic build -> immutable ScriptArtifactSet -> qualified ScriptInstallReceipt -> VM slot generation`。文本、图、Class/Component、Build、Play、Runtime load、Hot Reload与Debug必须共享source/artifact/install/session identity。

## 2. 冻结范围与方法

本报告以`d5d41037e080ecc948a3b13f3e8bab38b4cd708a`作为Zircon基线。共享工作树中selected source已有23个modified/untracked条目，本轮不回退、不覆盖、不暂存。外部ZrVM在取证期间HEAD发生变化，最终计量时为`b1f6884794e74ff2c2a2149438dccc00effca4a4`且有29项worktree变化；因此外部能力只作为本地snapshot证据，不能写成已发布稳定SDK。

物理行按逐文件读取统计；tests统计`#[test]`，ignored统计`#[ignore...]`。fingerprint由排序后的lowercase相对路径、`|`与逐文件SHA-256按LF连接后再次SHA-256；外部文件使用lowercase绝对路径。WOC完整源码只做规模清点，不把818个`.zr`全部加入selected fingerprint。

| 选择集 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---|
| Editor build/log/jump/command/catalog | **14 / 3,169 / 2,882 / 104,828 / 45 / 1** | `2b3e23ac1a61dd8ada75e7588c000eec830cabd2b0ff878ea259eab35af30f3b` |
| Runtime/Plugin/App/WOC纵切面 | **166 / 28,719 / 26,376 / 1,006,255 / 251 / 5** | `69a1fe51f43f67624b6402b91b17ebddbdd8fe7e56fc2ea83911ac169e8e9cfe` |
| Zircon selected union | **180 / 31,888 / 29,258 / 1,111,083 / 296 / 6** | `a7a2d58cbe994407f03b5bb7c99b30c15f89f2054e8fd466b293d76431512699` |
| Unreal/Godot/Bevy/Fyrox/Unity Graphics | **115 / 37,538 / 31,257 / 1,292,143 / 3 / 0** | `bebf171a32ba3192191e8ff7ea6d2633ed2160b8a05ffa00635cf53677eeefb0` |
| 外部ZrVM adapter/LSP/debug snapshot | **6 / 3,533 / 3,140 / 138,353 / 11 / 0** | `fb68cc9e08d4968c29f079653bf512dd91e25c9890de18b7a0a08fd2fa80c3ea` |
| all selected | **301 / 72,959 / 63,655 / 2,541,579 / 310 / 6** | `7382365792a6f996b0bddb25c94fa3e36d414f85bbe79fcc2fe4f5f60cf5d17e` |

WOC当前有**818个`.zr`、247,408物理行、238,721非空行、10,002,975 bytes；355个`.zrp`、52,452 bytes；37个`.zro`、8,143,561 bytes；0个`.zri`**。源码规模、局部测试project和37份binary不能证明统一dependency closure、完整artifact set、debug map、cook/install资格或默认Editor/Client可运行。

## 3. 当前产品事实

### 3.1 Script Build状态机仍是无消费者的领域组件

1. `ScriptBuildRequest::new`直接从request id构造`ScriptBuildGeneration`，source revision、compiler generation、artifact digest、install generation与runtime session没有独立identity。
2. 每个request固定分配`CompileModules(Vec<PathBuf>) -> ValidateLedger -> RefreshBindings`，这三个step没有production executor；它们是状态名，不是已实现阶段。
3. active step失败或取消会删除queued request和全部pending watch fact；generation N失败仍可丢失N+1编辑，相关测试明确锁定这一行为。
4. watch admission已有300/1,000 ms、20条/64 KiB、full-rebuild sentinel和one queued generation。2026-08-25切片又保存deduplicated path byte total并复用checked merge，降低重复记账，但没有source snapshot或dependency plan。
5. `ScriptBuildDiagnosticsSink`按generation/request/step cursor拒绝stale/replay，却逐条format、构造jump并同步调用`EditorLogService::emit`。bounded retained log不限制1M diagnostics的producer CPU、I/O、allocation与fanout。
6. `ScriptDiagnostic`只有severity/code/module/message和optional path/line/column，没有position encoding、source revision/digest、range end、related spans、fix-it、symbol、phase、backend或artifact identity。
7. 模块外production caller搜索仍只命中导出、测试和日志投影；watch、command、Play、job、VM、artifact、install、commandlet均为零。
8. command registry唯一Script条目是`view.console.source.script_build`，只是“Show Script Build Logs”过滤器，没有Build/Rebuild/Clean/Cancel/Open Script/Attach Debugger命令。

### 3.2 默认profile与provider资格不闭合

1. `target-client`与`target-editor-host`包含`script`，但不包含first-party ZrVM provider或`backend-zr-vm`；generic feature只能编译Runtime脚本框架，不能满足项目声明的具体backend。
2. WOC manifest声明required `zr_vm_language`覆盖Client/Server/EditorHost，script package又选择`zr_vm:project`；默认App target无法据此获得真实backend。
3. runtime catalog有feature-gated ZrVM registration，但Editor catalog没有对应feature、crate或registration branch。
4. ZrVM plugin manifest诚实标注`experimental`和两个`Partial` capability；`RuntimePluginDescriptor`又明确`enabled_by_default(false)`。
5. 缺backend的错误延迟到`load_project_package`，项目打开/target preflight没有生成provider/capability/toolchain qualification receipt。

### 3.3 Runtime仍拥有第二编译权威

1. Dynamic Session构造顺序是open assets、load navigation、`load_startup_scripts`、load level；script load在Runtime会话启动关键路径同步执行。
2. `load_startup_scripts`逐root同步discover，再按package name过滤startup package并`load_discovered_package`；manifest没有workspace id、dependency lock、target profile、expected artifact或install generation。
3. real backend在process-wide ZrVM mutex下构造Runtime、注册host/reflection、open project、incremental compile并start session。每个普通export、state save/restore/schema、GC和drop也使用同一全局mutex。
4. Rust binding compile接口只返回compiled/skipped/removed；公开`ManifestEntry`虽有module/source/zro hash与paths/imports，却没有Zircon BuildSet、toolchain/target/host ABI、atomic publication或qualification合同。
5. package discovery、payload materialization cache、slot generation、stable callback与hot reload rollback是真实底座；它们接收的仍是discovered source package，不是qualified immutable artifact。
6. `hot_reload_discovered_slot`的非测试production caller仍为零。Editor build完成不会触发Runtime reload，Runtime file watch也没有唯一产品owner。
7. state/reflection migration增强的是package generation。它没有Scene document revision、component instance ID、field stable ID、default/override或Editor preflight，不能关闭Class/Component创作差距。

### 3.4 Script Class/Component仍退化为动态JSON绑定

1. `SceneScriptBindingAsset`字段仍为package、module、enabled、update、fixed_update和任意JSON properties；无class/component/schema/field ID与version。
2. artifact cache只逐字段复制这份结构；Scene Project IO从dynamic component clone JSON后反序列化，未形成typed ECS component。
3. Runtime projection按`script.bindings` dynamic-component generation缓存；这减少无关World mutation重建，但每次相关重建会重新创建`started`与callback cache。
4. binding identity使用`package::module#array_index`，数组重排、插入、prefab override、rename与merge都会改变身份。
5. lifecycle只有Update分支首次`onStart`和按phase调用`onUpdate/onFixedUpdate`；若FixedUpdate先发生，脚本会在`onStart`之前收到`onFixedUpdate`。
6. binding循环使用`?`，首个resolve/call错误会停止当前phase的后续binding；没有per-instance isolation、failure policy或terminal batch receipt。
7. gameplay health helper仍按JSON字段名`hp`解析和改写，证明复杂玩法状态继续绕过canonical reflection/property transaction。

### 3.5 Source Editor、LSP、Debugger与Visual Script均未接入

1. `ResourceKind`现有26类资源没有任何Script/Visual Script/Debug Map/Artifact kind；Editor asset registry、factory、toolkit、thumbnail、reference analyzer与open route没有脚本类型。
2. `LogJump`能保存path/line/column，但host最终只发通用`OpenAsset(path)`；成功后将行列写进status line，没有source revision、range、caret、selection或document navigation。
3. production搜索没有`ScriptEditor`、`CodeEditor`、`LanguageServer`、Visual Script、VM breakpoint/debug session；命中的Breakpoint属于UI响应式布局、Behavior Tree静态表面或Graphics Debugger。
4. 外部ZrVM LSP已经有incremental text sync、position encoding negotiation与丰富capability；Zircon没有process/library lifecycle、workspace/document sync、crash restart、request cancellation或bounded response owner。
5. 外部debug库已经有breakpoint、step、stack/scopes/variables、safe evaluate effect policy与profile；Rust binding和Zircon plugin没有把这些能力暴露成versioned debug adapter。
6. Visual Script没有document schema、node provider、pin type、control/data flow、cycle/effect validation、compiler lowering、artifact、debug map或runtime consumer。它不得做成第二解释器或第二类型系统。

## 4. Owner边界与目标合同

| 领域 | 唯一owner | Editor152职责 |
|---|---|---|
| document/dirty/save/recovery/conflict | Editor02 | `ScriptWorkspace`、`SourceDocument`、`VisualScriptDocument` adapter与revision |
| asset identity/reference/dependency | Editor04 + Runtime Asset | Script Source/Class/Component/Visual Script正式resource、stable reference与rename/cook closure |
| language semantics/compiler/LSP | ZrVM owner + versioned Zircon adapter | canonical parse/type/query、structured diagnostics、symbol/schema、immutable artifact manifest |
| build admission/execution | Editor09/Runtime job owner | source generation、bounded job、cancel/progress、terminal build receipt；不得私建线程池 |
| runtime install/reload | Runtime07/21 | verified artifact、session/slot generation、safe-point commit、package/component state migration与rollback |
| reflection/Inspector | Runtime reflection + Editor05 | compiled Class/Component/Field schema、typed property patch、default/override provenance |
| Play/session | Editor07 | required source/install generation waiter、failure/cancel/session replacement |
| diagnostics/logging | Editor11 | bounded diagnostic pages、visible generation、real range jump、truncation/continuation receipt |
| debugger/profiler | ZrVM debug adapter + Editor31 | target/session/thread/frame/value/breakpoint/profile UI，全部绑定artifact debug map |
| Visual Script | Editor31 + canonical compiler | typed graph source，lowering到同一module/interface/artifact，不建立独立Runtime |
| Cook/package工具 | 后续Rust tooling迁移 | 本轮只冻结ScriptArtifactSet输入/输出合同，不展开tooling优化 |

建议的最小身份链：

```text
ScriptWorkspaceId
  -> ScriptSourceGeneration(source closure + dependency snapshot)
  -> ScriptBuildTicket(observer/priority/cancel only)
  -> ScriptArtifactSetId(toolchain + target + source + dependency + host schema digests)
  -> ScriptInstallReceipt(runtime session + package slot + expected/installed generation)
  -> ScriptDebugMapId(artifact-qualified source/graph mapping)
```

request id不得继续承担content generation；Editor不能持有VM instance；Runtime不能在shipping/qualified load路径重新打开source workspace编译。

## 5. 参考实现的直接差异

1. **Unreal Blueprint/Kismet**：`UBlueprint`同时保存status/system version、Ubergraph/Function/Macro graphs、Component Templates、variables、Generated/Skeleton Class与debug object；`FBlueprintEditor`提供compiler results、graph/document navigation、find/replace、breakpoint/watch与debug object选择。Zircon应吸收source/generated/debug identity分离，不复制UObject全局扫描。
2. **Unreal Compilation Manager/Live Coding**：Compilation Manager有queued request、17阶段flush、skeleton/interface pass、dependency repair、bytecode/class generation与later reinstance；Live Coding区分compile、patch load、reinstance、completion。Zircon VM更适合immutable artifact、slot generation与safe-point commit，但必须同样分阶段并产出receipt。
3. **Godot**：`ScriptEditor`拥有history、autosave、reload、unsaved/save-all、method navigation和breakpoint；`ScriptLanguage`定义validate/reload、stack/locals/members/globals与profiling；`ScriptDebugger`维护source+line breakpoint与thread-local step/depth。Zircon不能用Console filter和status line替代这些适配面。
4. **Fyrox**：Script Inspector用UUID选择constructor、内嵌reflection Inspector并可打开external IDE；Editor Build以command queue、child process、output window和play-after-build组成真实workflow。其实现不是Zircon终态，但已高于当前“字符串binding + 无caller状态机”的产品可达性。
5. **Bevy**：`TypeRegistry`分离`TypeId`、full type path、short path与ambiguous names，递归注册field/variant依赖并以TypeData扩展能力。它适合Script Class/Field schema与reflection identity参考；Bevy没有first-party脚本Editor，不能被用作缺失产品的辩护。
6. **Unity Graphics**：本地树只有ShaderGraph等渲染编辑代码，直接搜索ScriptEditor/VisualScript/LanguageServer/ScriptDebugger只命中ShaderGraph查询external script editor。它不是Unity通用脚本/Visual Scripting权威源码，本篇不从该范围制造虚假对标。
7. **外部ZrVM**：LSP与debug库已提供可复用能力，但Rust binding compile surface仍只返回计数。正确方向是固定revision/ABI并发布structured compiler/LSP/debug adapter，不是在Zircon中重写语义。

## 6. Canonical P0 currentness

| ID | 状态 | 当前证据 | 必须重构 |
|---|---|---|---|
| P0-1 | Open | required `zr_vm:project`与默认Client/Editor feature不相容，插件disabled/Partial | target profile preflight、provider/toolchain/capability receipt；打开项目或启动target前失败 |
| P0-2 | Open | Editor orchestrator无caller，Runtime startup同步另编source | 唯一shared semantic build与immutable ArtifactSet owner；Runtime只接受qualified artifact |
| P0-3 | Open | 无Script resource/document/editor；jump只OpenAsset并写status | transactional SourceDocument、真实code editor、revision/range navigation、LSP lifecycle |
| P0-4 | Open | `script.bindings`仍是package/module + JSON map | typed Class/Component/Field schema、stable IDs、default/override、migration与Inspector transaction |
| P0-5 | Open | Visual Script、LSP、Debugger均无Zircon adapter/product consumer | versioned ZrVM compiler/LSP/debug adapter；text/graph共享artifact/debug map/runtime |

## 7. Canonical P1 currentness

| ID | 状态 | 当前差异 | ID | 状态 | 当前差异 |
|---|---|---|---|---|---|
| P1-01 | Open | project scripts无workspace identity/schema | P1-31 | Open | state migration无Editor preflight |
| P1-02 | Open | package/module/path/name混作identity | P1-32 | Open | rollback无产品projection/receipt |
| P1-03 | Open | `.zr`不在asset catalog | P1-33 | Open | lifecycle export依赖可选字符串名 |
| P1-04 | Open | 无transactional SourceDocument | P1-34 | Open | ZrVM所有工作由process-wide mutex串行 |
| P1-05 | Open | encoding/newline/position map合同缺失 | P1-35 | Open | backend unavailable延迟到load |
| P1-06 | Open | 无LSP lifecycle owner | P1-36 | Open | 无Script Class asset identity |
| P1-07 | Open | LSP sync未接Editor02 revision | P1-37 | Open | 无Script Component asset identity |
| P1-08 | Open | language query无reflection generation | P1-38 | Open | field无stable ID/redirect |
| P1-09 | Open | definition/reference/rename无跨资产事务 | P1-39 | Open | field type/default/constraint缺失 |
| P1-10 | Open | fix-it/code action无安全/undo模型 | P1-40 | Open | default与instance override不可区分 |
| P1-11 | Open | generated/source边界未定义 | P1-41 | Open | visibility/permission/effect缺失 |
| P1-12 | Open | symbol/search index无规模预算 | P1-42 | Open | update/fixed flags代替typed lifecycle |
| P1-13 | Open | request id冒充source/artifact generation | P1-43 | Open | dynamic binding绕过typed ECS |
| P1-14 | Open | build失败/取消删除更新source fact | P1-44 | Open | required sibling/conflict规则缺失 |
| P1-15 | Open | changed path不是dependency plan | P1-45 | Open | class/component不进reference/cook/rename closure |
| P1-16 | Open | `CompileModules`无executor | P1-46 | Open | 无Visual Script document/schema |
| P1-17 | Open | `ValidateLedger`只是状态名 | P1-47 | Open | Visual Script没有统一lowering |
| P1-18 | Open | `RefreshBindings`只是状态名 | P1-48 | Open | node catalog无typed provider contract |
| P1-19 | Open | compiler无structured diagnostics | P1-49 | Open | pin inference/conversion缺失 |
| P1-20 | Open | diagnostic ingress逐条同步无page budget | P1-50 | Open | control/data flow/cycle/effect语义缺失 |
| P1-21 | Open | 散落`.zro`无完整artifact manifest | P1-51 | Open | graph refactor/semantic diff缺失 |
| P1-22 | Open | `.zri`/debug map生命周期缺失 | P1-52 | Open | graph debug mapping缺失 |
| P1-23 | Open | artifact/cache/cleanup无唯一owner | P1-53 | Open | 无Script Workspace/editor toolkit |
| P1-24 | Open | Play build-before-run未闭合 | P1-54 | Open | 无真实Build命令/status projection |
| P1-25 | Open | headless/cook无同一build入口 | P1-55 | Open | 无breakpoint store/rebind |
| P1-26 | Open | startup同步discovery/compile | P1-56 | Open | 无debug session lifecycle |
| P1-27 | Open | Runtime startup与Editor build双authority | P1-57 | Open | 无stack/locals/watch/evaluate产品 |
| P1-28 | Open | hot reload无production trigger | P1-58 | Open | 无script profiling/coverage集成 |
| P1-29 | Open | package-name查找缺session-qualified identity | P1-59 | Open | 无multi-target/session/package debug model |
| P1-30 | Open | package state不等于component instance state | P1-60 | Open | 无fault/scale/performance/migration资格 |

watch路径字节记账优化不改变P1-13至P1-20的语义状态：它减少了已受限集合的重复遍历，但没有建立source generation、dependency snapshot、executor、artifact或diagnostic page。

## 8. Canonical P2 currentness

| ID | 状态 | 当前差异 |
|---|---|---|
| P2-01 | Open | language backend plugin SDK与capability negotiation缺失 |
| P2-02 | Open | text/graph round-trip与统一semantic view缺失 |
| P2-03 | Open | live value overlay/time-travel snapshot缺失 |
| P2-04 | Open | remote/distributed compile与cache qualification缺失 |
| P2-05 | Open | package dependency registry、lock与provenance产品缺失 |
| P2-06 | Open | sandbox/capability/permission可视化缺失 |
| P2-07 | Open | deterministic replay/rollback debugger缺失 |
| P2-08 | Open | semantic merge与协作冲突模型缺失 |
| P2-09 | Open | per-script/per-export budget annotation缺失 |
| P2-10 | Open | Interp/Binary/AOT semantic parity与sidecar策略缺失 |
| P2-11 | Open | 同workload跨引擎authoring/runtime基准缺失 |
| P2-12 | Open | signing/SBOM/provenance/quarantine/revoke供应链资格缺失 |

## 9. 32项资格门

| Gate | 状态 | 当前判定 |
|---|---|---|
| G01 | Fail | 默认profile不能满足required ZrVM provider，preflight也未提前失败 |
| G02 | Fail | Source document未接Editor02 revision/save/recovery/conflict |
| G03 | Fail | 无LSP lifecycle与crash/restart/shutdown产品 |
| G04 | Fail | language query未绑定host reflection generation |
| G05 | Fail | diagnostic无position encoding/source revision/range，jump会伪定位 |
| G06 | Fail | fix-it/rename无preview/atomic transaction/undo/rollback |
| G07 | Partial | active+one queued、20路径/64 KiB、1秒首事件截止与O(1)重复字节记账已存在；真实job/diagnostic/artifact resident bytes/age/deadline及managed terminal evidence仍缺 |
| G08 | Fail | active失败仍删除queued与pending source fact |
| G09 | Fail | source/artifact/install/request identity不可追溯 |
| G10 | Fail | compiler只返回计数/字符串error，diagnostic ingress无bounded page |
| G11 | Fail | ArtifactSet非完整、非原子、非content-addressed、无toolchain/target/debug map |
| G12 | Fail | WOC 818源码与37 `.zro`没有完整qualified set证明 |
| G13 | Fail | incremental只接changed paths/外部manifest，非canonical dependency snapshot |
| G14 | Fail | Build/Play/Runtime load没有共同compiler/cache/artifact/receipt |
| G15 | Fail | Runtime startup明确同步做discover/open/compile/session start |
| G16 | Fail | Runtime不接受qualified artifact，仍重复编译source |
| G17 | Fail | hot reload不消费qualified artifact/expected session-slot generation |
| G18 | Fail | package reflection/state rollback存在，但component schema与产品receipt缺失 |
| G19 | Fail | package state和component instance state没有分离迁移合同 |
| G20 | Fail | Class/Component/Field stable ID不存在 |
| G21 | Fail | Inspector没有compiled typed schema patch |
| G22 | Fail | legacy `script.bindings`仍是唯一读写格式，无迁移删除门 |
| G23 | Fail | Visual Script与文本没有共同module/artifact/interface |
| G24 | Fail | graph stable ID/type/cycle/effect/semantic diff不存在 |
| G25 | Fail | text/graph breakpoint与artifact debug map不存在 |
| G26 | Fail | debug attach/continue/step/stop与Runtime session无Zircon合同 |
| G27 | Fail | stack/value/evaluate无Zircon depth/count/bytes/time/effect预算 |
| G28 | Fail | process-wide mutex与package-name入口不能证明multi-PIE/target隔离 |
| G29 | Fail | profiling/coverage没有artifact/debug map/clock-domain绑定 |
| G30 | Fail | compile/reload底座有局部rollback，但LSP/debug/fault统一terminal receipt缺失 |
| G31 | Fail | WOC typing/build/reload/debug没有动态预算分布或UI hitch/RSS/I/O证据 |
| G32 | Fail | Windows/Linux、Editor/Client/Server及Interp/Binary/AOT资格未闭合 |

## 10. 现存记录与currentness约束

1. `docs/plans/zircon_editor/editor/13-script-compilation-management.md`仍为`in_progress`。M1纯状态机与diagnostic投影存在，M2真实VM/asset/install、M3Play/job/commandlet仍开放。
2. `editor/13/failure-2026-07-22-script-build-debounce-admission-backpressure.md`虽已有部分修复，但仍要求shared job ticket的entry/bytes/oldest-age/cancel、production caller和managed Cargo evidence；不得因子问题关闭而关闭父链。
3. `docs/plans/optimize/zircon_editor/31/2026-08-25-watch-budget-accounting.md`明确只推进G07，implementation complete但managed validation pending，并明确列出所有未接产品层。
4. 2026-08-23 performance currentness仍准确指出request/source/artifact/binding generation混用、失败删除新source和unbounded diagnostic ingress；本轮只更新文件规模与watch byte-accounting delta。
5. 2026-08-24 ZrVM performance报告中的外部revision已经过期；本轮观察外部repo继续移动且dirty。实施adapter前必须冻结clean source/build/ABI receipt，不能引用任意本机HEAD。
6. `zircon_runtime_interface/src/runtime_build_set`的新Runtime DLL BuildSet合同是Host/Runtime内部ABI artifact identity，不是ScriptArtifactSet。可以复用digest/target/capability验证思想，不能把它直接改名冒充脚本编译产物。

## 11. 分层重构顺序

1. **M0 Truthfulness/Profile**：统一target provider矩阵；required backend在project/target preflight前验证；冻结ZrVM source/build/ABI revision，修正NativeDynamic/Partial能力表达。
2. **M1 Workspace/Document**：在Editor02下建立ScriptWorkspace、SourceDocument、VisualScriptDocument、revision/encoding/line map、external conflict与真实range navigation；给Script资源正式asset identity。
3. **M2 Shared Build/Artifact**：分离source generation、ticket、artifact、install identity；接唯一shared job owner；compiler返回bounded structured diagnostic pages和immutable content-addressed ArtifactSet。
4. **M3 Class/Component/Reflection**：建立stable class/component/field schema、default/override provenance、typed Inspector patch、reference graph与legacy `script.bindings`迁移。
5. **M4 Runtime Install/Reload**：shipping Runtime只加载verified artifact；candidate在旧generation外prepare，safe point按expected session/slot commit；package与component state分别迁移、失败保留last-good。
6. **M5 Visual Script**：typed graph/node/pin/provider、type/effect/cycle validation和semantic diff；lowering到同一compiler module/interface/artifact/debug map。
7. **M6 LSP/Debugger/Profiler**：接versioned ZrVM adapter，提供workspace sync、symbols、breakpoint、attach/session/thread/frame/value/evaluate/profile/coverage，并处理stale source/artifact。
8. **M7 Product Qualification**：Build command、build-before-Play、Client/Server/Editor一致性、WOC 818-file cold/warm/incremental/reload/debug、fault/scale/soak与同workload性能验证。Tooling实现留待Rust迁移，但ScriptArtifactSet合同不得再改变。

## 12. 禁止的临时修补

- 禁止在Console旁增加文本框或语法着色控件就宣称Script Editor完成。
- 禁止用regex、扩展名、stderr解析或另一套parser替代ZrVM canonical semantics/LSP/debugger。
- 禁止让Editor和Runtime分别编译source后比较时间戳；Runtime不得在qualified/shipping load路径重新编译。
- 禁止继续用request id同时表示source、artifact、binding/install generation，或在失败时清除更新source fact。
- 禁止把bounded retained log当作bounded diagnostic ingress；1M错误必须在producer page admission处截断并有receipt。
- 禁止给JSON map增加几个hardcoded Inspector字段后命名为Script Component。
- 禁止把Visual Script实现为独立解释器、独立类型系统或独立shipping artifact。
- 禁止先画断点/stack面板，再用固定row或静态反馈冒充debug session。
- 禁止绕过`HotReloadCoordinator`直接替换VM instance，也禁止把package state migration当作component override migration。
- 禁止把818个`.zr`、355个`.zrp`、37个`.zro`或测试marker数量写成默认产品可运行或性能优于Unreal的证据。

## 13. 验证边界

本轮完成当前共享工作树的Editor build/log/jump/catalog、Runtime Script全目录、Dynamic startup、Scene binding、ZrVM plugin/App feature/WOC纵切面复核，并直接核对Unreal Blueprint/Kismet/HotReload/LiveCoding、Godot Script/Debugger、Fyrox Script Inspector/Build、Bevy Reflect/Watcher、Unity Graphics范围和外部ZrVM binding/LSP/debug API。没有修改Rust/C/TOML生产实现。

本轮没有运行Cargo、ZrVM CMake/CTest、LSP/debug protocol、WOC full build、Editor UI、Play、Client/Server、hot reload、fault、scale、soak或跨引擎benchmark。Tooling按用户要求暂不纳入优化。共享Zircon与外部ZrVM均存在在途变更，实施前必须重算selected manifest、外部revision/API、default feature组合和Runtime startup顺序。
