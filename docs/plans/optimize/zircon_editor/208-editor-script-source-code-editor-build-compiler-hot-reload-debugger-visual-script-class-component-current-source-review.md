---
title: Editor Script Source、Code Editor、Build、Compiler、Hot Reload、Debugger、Visual Script、Class 与 Component 当前源码复核
category: zircon_editor
report_id: Editor208
review_date: 2026-08-28
baseline_head: a2d8d811c4a3a1fc1db6f5375c491e7e4502533f
verification_head: a2d8d811c4a3a1fc1db6f5375c491e7e4502533f
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
canonical_owner: Editor31
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/31-script-source-code-editor-build-compiler-hot-reload-debugger-visual-script-class-component-authoring-review.md
  - docs/plans/optimize/zircon_editor/105-editor-script-source-code-editor-build-compiler-hot-reload-debugger-visual-script-class-component-current-source-review.md
  - docs/plans/optimize/zircon_editor/152-editor-script-source-code-editor-build-compiler-hot-reload-debugger-visual-script-class-component-current-source-review.md
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
  - examples/woc/scripts/woc_game/plugin.toml
plan_sources:
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_runtime/21-zr-language-parser-type-system-semir-bytecode-package-loader-vm-runtime-review.md
  - docs/plans/optimize/zircon_plugins/16-first-party-zr-vm-language-source-runtime-dist-catalog-reflection-callsite-host-interface-gc-hot-reload-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/99l-runtime-scene-reflection-type-schema-registry-dynamic-component-property-address-inspection-artifact-subscription-editor-product-integration-current-source-review.md
  - docs/plans/zircon_editor/editor/13-script-compilation-management.md
  - docs/plans/performance/01/2026-08-16-editor-core-script-build-generation-current-architecture-review.md
  - docs/plans/optimize/zircon_editor/31/2026-08-25-watch-budget-accounting.md
  - docs/plans/zircon_editor/editor/13/failure-2026-07-22-script-build-debounce-admission-backpressure.md
  - docs/plans/zircon_editor/editor/13/failure-2026-07-22-script-build-facade-validation-copy-closure.md
  - docs/plans/zircon_editor/editor/13/failure-2026-07-23-settings-registry-script-build-batch-window-migration.md
  - docs/plans/zircon_editor/editor/13/failure-2026-08-05-script-build-diagnostics-editor-log-source-bridge.md
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

# Editor208 · Script Source / Build / Hot Reload / Debugger / Visual Script 当前源码复核

## 1. 结论

Zircon当前仍没有工程级脚本创作、构建、安装、热重载和调试闭环。Runtime脚本域不是空壳：package discovery有深度、条目、路径、payload bytes、wall-time与取消预算；`VmPluginManager`有backend family、slot/generation、callback refresh和payload cache；`HotReloadCoordinator`有reflection prepare/commit、state snapshot/migration、activate/rollback及GC deadline。这些是真实底座，后续应保留而不是推倒重写。

Editor产品链仍未接上这些底座。`zircon_editor/src/core/script_build`保持5文件、1,696行，提供300 ms debounce、1,000 ms首事件截止、20路径/64 KiB预算、active + one queued request、`Watch < Command < Play`提升、typed dispatch identity和diagnostic cursor；但在模块外精确搜索`ScriptBuildOrchestrator`、`ScriptBuildDiagnosticsSink`、`.enqueue_play(`、`.enqueue_command(`和`.notify_watch_change(`全部为零。`CompileModules`、`ValidateLedger`、`RefreshBindings`没有production executor、artifact publication、install receipt、Play waiter或commandlet。

Runtime仍拥有另一套编译权威。Dynamic Session在加载level前同步执行`load_startup_scripts()`，逐root discover package，再由ZrVM real backend执行`ProjectWorkspace::open -> compile(incremental=true) -> start_session`。公开compile结果只给compiled/skipped/removed计数；虽然外部binding已可读module/source/zro hash、path和imports manifest，但Zircon不要求source/toolchain/dependency/target/debug-map齐备的immutable `ScriptArtifactSet`，Editor Build、Play、Client、Server和Cook没有共同产物真值。

默认装配比Editor152记录的差异更严重。WOC把`zr_vm_language`声明为Client、Server与EditorHost所需provider，并选择`backend = "zr_vm:project"`；`target-client`与`target-editor-host`仅启用generic `script`，没有provider或`backend-zr-vm`，而`target-server`连generic `script`都没有。ZrVM插件还明确`enabled_by_default(false)`、`experimental`且两项capability为`Partial`。这不是运行时降级策略，而是默认target与项目required声明互相矛盾。

脚本资源、Class/Component和编辑器产品仍为空。26项`ResourceKind`没有Script Source、Module、Class、Component、Visual Script、Debug Map或Artifact。扫描排除`dev/docs/tools/.codex/target`后的20,127个tracked/untracked Rust、ZUI和TOML文件，只出现5处无关的`UiCompiledAssetPackageSection::SourceDocument`，它表示UI模板编译包的source section；其余目标名为零，也没有任何脚本workspace/document/artifact/install/debug-map/class/component/editor产品合同。

因此Editor31/105/152的canonical结论维持：**5项P0全部Open；60项P1全部Open；12项P2全部Open；32个Gate为31 Fail、1 Partial、0 Pass**。目标链仍应是`ScriptWorkspace + transactional Source/VisualScript Documents -> canonical semantic build -> immutable ScriptArtifactSet -> qualified ScriptInstallReceipt -> VM slot generation`，并让文本、图、Class/Component、Build、Play、Runtime load、Hot Reload和Debug共享source/artifact/install/session identity。

## 2. 冻结范围与方法

本报告以`a2d8d811c4a3a1fc1db6f5375c491e7e4502533f`作为Zircon基线。共享工作树取证时有9,664项状态变化，本轮不回退、不覆盖、不暂存。外部`E:/Git/zr_vm`为`7c416d8ea362bb52cecd6a24c78f4e739fbcf87b`且有40项变化，只能作为本地snapshot证据，不能写成已发布稳定SDK。

Godot为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、Bevy为`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、Fyrox为`8d815db36494f1badb347547dfc7094bf4fbbdf8`、Unity Graphics为`a7e4c051d256a781ab362c64316b125a1e104694`，四个子仓库均为clean；Unreal没有独立`.git`，按Zircon基线工作树冻结。

选择集保留目录下全部文件及显式文件。物理行按逐文件LF/CRLF读取统计；tests统计Rust `#[test]`，ignored统计`#[ignore...]`。fingerprint由排序后的lowercase相对路径、`|`与逐文件SHA-256按LF连接后再次SHA-256；外部文件使用lowercase绝对路径。WOC完整源码只做规模清点，不把818个`.zr`全部加入selected fingerprint。

| 选择集 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---|
| Editor build/log/jump/command/catalog | **12 / 2,995 / 2,716 / 98,941 / 44 / 1** | `bda7d63cf336a0d2a8dd2b565245fc0957f24fc358b96fccfc56695c5734a530` |
| Runtime/Plugin/App/WOC纵切面 | **165 / 28,902 / 26,521 / 1,013,122 / 246 / 9** | `032d61bb4bcc1c877ef05f96b26e64835c86b3b8fad0da916238f1b0e0d67bb9` |
| Zircon selected union | **177 / 31,897 / 29,237 / 1,112,063 / 290 / 10** | `be567227df22e23970f1f0f004a427dabb22ec0e49d4eb3a9a93246e79af244d` |
| Unreal/Godot/Bevy/Fyrox/Unity Graphics | **121 / 37,748 / 31,438 / 1,299,471 / 3 / 0** | `df60d9a489e8b1c0beb468b8d67133d720481463114c3155bec0f82740d26b19` |
| 外部ZrVM adapter/LSP/debug snapshot | **6 / 3,533 / 3,140 / 138,353 / 11 / 0** | `fb68cc9e08d4968c29f079653bf512dd91e25c9890de18b7a0a08fd2fa80c3ea` |
| all selected | **304 / 73,178 / 63,815 / 2,549,887 / 304 / 10** | `e7deb1c1f966db7487cd19745fa67f3c8d759aab20c2fabfc8d290e939ab21fe` |

WOC当前有**818个`.zr`、247,408物理行、238,721非空行、10,002,975 bytes；355个`.zrp`、52,452 bytes；37个`.zro`、8,143,561 bytes；0个`.zri`**。规模与散落binary不能证明dependency closure、完整artifact set、debug map、Cook/install资格或默认Editor/Client/Server可运行。

## 3. 当前产品事实

### 3.1 Script Build只是有界状态机，不是已接通的构建产品

1. `ScriptBuildRequest::new`直接从request id构造`ScriptBuildGeneration`，source revision、compiler generation、artifact digest、install generation与runtime session没有独立identity。
2. request固定分配`CompileModules(Vec<PathBuf>) -> ValidateLedger -> RefreshBindings`，三个step没有production executor；当前只是状态名。
3. active step失败或取消会删除queued request和pending/debounced source facts，generation N失败仍可丢失N+1编辑。
4. watch admission已有300/1,000 ms、20路径/64 KiB、full-rebuild sentinel、one queued generation和O(1)去重路径字节记账，但没有source snapshot或dependency plan。
5. `ScriptBuildDiagnosticsSink`能拒绝stale/replay，却逐条format并同步`EditorLogService::emit`；retained log上限不等于producer CPU、I/O、allocation和fanout预算。
6. `ScriptDiagnostic`只有severity/code/module/message及可选path/line/column，没有position encoding、source revision/digest、range end、related span、fix-it、phase/backend/artifact identity。
7. 唯一Script command仍是显示`script_build`日志source，没有Build/Rebuild/Clean/Cancel/Open Script/Attach Debugger命令。

### 3.2 Runtime、profile和provider仍有三套不一致真值

1. Client/EditorHost有generic脚本框架但没有required ZrVM provider；Server连generic脚本框架也未启用。
2. WOC manifest仍对三个target声明required `zr_vm_language`，startup package为`woc_game`，package明确选择`zr_vm:project`。
3. first-party runtime catalog只在feature启用时注册ZrVM；first-party Editor catalog只有Navigation与Neural分支，没有ZrVM language/editor provider。
4. backend缺失直到package load才暴露`BackendUnavailable`，项目打开与target preflight不产生provider/toolchain/capability qualification receipt。
5. `load_startup_scripts`仍在Runtime Session关键路径discover和load source；real backend仍在加载时打开workspace、增量编译并启动session。
6. `hot_reload_discovered_slot`在Runtime模块外只命中ZrVM real-backend测试，没有production trigger。Editor build完成不会驱动Runtime reload。

### 3.3 Script Class/Component仍退化为动态JSON绑定

1. `SceneScriptBindingAsset`仍只有package、module、enabled、update、fixed_update与`BTreeMap<String, serde_json::Value>`，无class/component/schema/field ID/version。
2. Scene Project IO整体读写动态`script.bindings`；没有typed ECS component、reference graph、rename/cook closure或property transaction。
3. Runtime binding identity为`package::module#array_index`，数组插入、重排、rename、prefab override与merge都会改身份。
4. `onStart`只在首次Update路径调用，FixedUpdate可先于`onStart`发生；binding循环中的`?`使单个resolve/call错误中止当前phase剩余binding。
5. callback handle refresh、world-handle + dynamic-component-generation projection cache及package state migration是真实性能/热重载底座，但不提供per-component stable identity、field migration、default/override provenance或Editor preflight。

### 3.4 Source Editor、LSP、Debugger与Visual Script仍未接入

1. `ScriptLocation`保存path/line/column，host却只发通用`OpenAsset(path)`；成功后行列仅写入status line，没有caret/range/source revision。
2. 外部ZrVM LSP已声明incremental sync、diagnostics、completion、hover、signature、definition/reference/rename、semantic tokens、inlay hints、code actions、formatting、hierarchy与workspace operations。
3. 外部debug库已提供line/function/data/exception breakpoint、continue/pause/step、stack/scopes/variables、受effect policy约束的evaluate和profile。
4. Zircon没有versioned compiler/LSP/debug adapter、workspace/document sync、process lifecycle、request cancellation、debug session、source/artifact map或Editor UI。
5. Visual Script没有document schema、node provider、stable node/pin ID、type/effect/cycle validation、compiler lowering、artifact、debug map或runtime consumer；不得实现成第二解释器或第二类型系统。

## 4. Owner边界与目标合同

| 领域 | 唯一owner | Editor208要求 |
|---|---|---|
| document/dirty/save/recovery/conflict | Editor02 | `ScriptWorkspace`、`SourceDocument`、`VisualScriptDocument` adapter、revision/encoding/line map |
| asset identity/reference/dependency | Editor04 + Runtime Asset | Script Source/Class/Component/Visual Script正式resource、stable reference、rename/cook closure |
| language semantics/compiler/LSP | ZrVM owner + versioned Zircon adapter | canonical parse/type/query、structured diagnostics、symbol/schema、immutable artifact manifest |
| build admission/execution | Editor09/Runtime job owner | source generation、bounded job、cancel/progress、terminal build receipt；不得私建线程池 |
| runtime install/reload | Runtime07/21 | verified artifact、slot generation、safe-point commit、package/component migration与rollback |
| reflection/Inspector | Runtime reflection + Editor05 | compiled Class/Component/Field schema、typed property patch、default/override provenance |
| Play/session | Editor07 | required source/install generation waiter、failure/cancel/session replacement |
| debug/profiling | Runtime diagnostics + ZrVM adapter + Editor25 | artifact-qualified breakpoint/session/frame/value/profile receipt |

必须建立且不能合并冒充的identity包括`ScriptWorkspaceId`、`ScriptDocumentId`、`ScriptSourceGeneration`、`ScriptBuildTicket`、`ScriptArtifactSetId`、`ScriptInstallReceipt`、`VmSlotGeneration`、`ScriptClassId`、`ScriptComponentId`、`ScriptFieldId`、`ScriptDebugMapId`和`ScriptDebugSessionId`。Runtime内部DLL `RuntimeBuildSet`不是脚本产物合同，只能复用digest/target/capability校验思想。

## 5. 五套参考实现的直接差异

1. **Unreal Blueprint/Kismet**：`UBlueprint`保存compile status/system version、graphs、component templates、Generated/Skeleton Class和debug对象；Blueprint Editor提供compiler result、document navigation、搜索、断点与调试；Compilation Manager区分queue、dependency、skeleton/interface、class generation和reinstance。Zircon应吸收source/generated/debug identity与分阶段receipt，不复制UObject全局扫描。
2. **Unreal Hot Reload/Live Coding**：compile、patch load、reinstance和completion是不同阶段。Zircon VM更适合immutable artifact + slot generation + safe-point commit，但同样必须有candidate/commit/rollback/terminal receipt。
3. **Godot**：Script Editor拥有multi-document、history、autosave/save-all、reload、unsaved处理和breakpoint；`ScriptLanguage`定义validate/reload、completion、stack/locals/evaluate/profile；`ScriptDebugger`按source+line维护断点和step状态。Console filter与status line不能替代这些产品面。
4. **Fyrox**：Script Inspector以UUID选择script constructor，接reflection Inspector和外部IDE；Editor Build用command queue、child process、output window及play-after-build形成真实workflow。它不是Zircon终态，但产品可达性高于无caller状态机。
5. **Bevy**：`TypeRegistry`分离`TypeId`、full/short type path、ambiguous names和TypeData，并递归注册依赖，适合Class/Field schema参考；FileWatcher只提供debounced asset event基础。Bevy没有first-party script editor，不能为Zircon缺失产品辩护。
6. **Unity Graphics**：本地范围是ShaderGraph渲染图资产，`GraphUtil`覆盖asset创建、依赖环检查和graph traversal；它不是Unity通用C# Script/Visual Scripting权威，本报告不从该范围制造虚假脚本对标。

## 6. Canonical P0 currentness

| ID | 状态 | 当前证据 | 必须重构 |
|---|---|---|---|
| P0-1 | Open | required ZrVM与Client/Editor feature不相容，Server无generic script，插件disabled/Partial | target preflight、provider/toolchain/capability receipt；启动target前失败 |
| P0-2 | Open | Editor orchestrator无caller，Runtime startup同步另编source | 唯一shared semantic build与immutable ArtifactSet owner；Runtime只接受qualified artifact |
| P0-3 | Open | 无Script resource/document/editor；jump只OpenAsset并写status | transactional SourceDocument、真实code editor、revision/range navigation、LSP lifecycle |
| P0-4 | Open | `script.bindings`仍是package/module + JSON map | typed Class/Component/Field schema、stable ID、default/override、migration与Inspector transaction |
| P0-5 | Open | Visual Script、LSP、Debugger均无Zircon adapter/product consumer | versioned ZrVM adapter；text/graph共享artifact/debug map/runtime |

## 7. Canonical P1 currentness

| ID | 状态 | 当前差异 | ID | 状态 | 当前差异 |
|---|---|---|---|---|---|
| P1-01 | Open | project scripts无workspace identity/schema | P1-31 | Open | state migration无Editor preflight |
| P1-02 | Open | package/module/path/name混作identity | P1-32 | Open | rollback无产品projection/receipt |
| P1-03 | Open | `.zr`不在asset catalog | P1-33 | Open | lifecycle export依赖可选字符串名 |
| P1-04 | Open | 无transactional SourceDocument | P1-34 | Open | ZrVM关键工作由process-wide mutex串行 |
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

watch路径字节O(1)记账只减少受限集合的重复遍历，没有建立source generation、dependency snapshot、executor、artifact或diagnostic page，故P1状态不变。

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
| G01 | Fail | 默认三个target均不能满足required ZrVM provider，Server还未启用generic script |
| G02 | Fail | Source document未接Editor02 revision/save/recovery/conflict |
| G03 | Fail | 无LSP lifecycle与crash/restart/shutdown产品 |
| G04 | Fail | language query未绑定host reflection generation |
| G05 | Fail | diagnostic无position encoding/source revision/range，jump会伪定位 |
| G06 | Fail | fix-it/rename无preview/atomic transaction/undo/rollback |
| G07 | Partial | active+one queued、20路径/64 KiB、首事件截止与O(1)字节记账存在；job/diagnostic/artifact bytes/age/deadline与managed terminal evidence仍缺 |
| G08 | Fail | active失败仍删除queued与pending source fact |
| G09 | Fail | source/artifact/install/request identity不可追溯 |
| G10 | Fail | compiler只返回计数/字符串error，diagnostic ingress无bounded page |
| G11 | Fail | ArtifactSet非完整、非原子、非content-addressed、无toolchain/target/debug map |
| G12 | Fail | WOC 818源码与37 `.zro`没有完整qualified set证明 |
| G13 | Fail | incremental只接changed path/外部manifest，非canonical dependency snapshot |
| G14 | Fail | Build/Play/Runtime load没有共同compiler/cache/artifact/receipt |
| G15 | Fail | Runtime startup同步discover/open/compile/session start |
| G16 | Fail | Runtime不接受qualified artifact，仍重新编译source |
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
| G30 | Fail | compile/reload有局部rollback，但LSP/debug/fault统一terminal receipt缺失 |
| G31 | Fail | WOC typing/build/reload/debug没有动态预算分布或UI hitch/RSS/I/O证据 |
| G32 | Fail | Windows/Linux、Editor/Client/Server及Interp/Binary/AOT资格未闭合 |

## 10. 现存记录与currentness约束

1. `docs/plans/zircon_editor/editor/13-script-compilation-management.md`仍为`in_progress`。M1状态机/diagnostic source存在，M2真实VM/asset/install及M3 Play/job/commandlet仍开放。
2. 四份Editor13 failure handoff仍Open，分别约束watch admission/backpressure、facade validation copy、batch-window settings migration及diagnostic-to-log bridge；不得因局部测试存在而关闭父链。
3. `zircon_editor/31/2026-08-25-watch-budget-accounting.md`只推进G07，明确managed validation pending及产品层未接线。
4. `performance/01/2026-08-16-editor-core-script-build-generation-current-architecture-review.md`关于generation混用、失败删除新source和unbounded diagnostic ingress仍成立。
5. 外部ZrVM继续移动且dirty。实施adapter前必须冻结clean source/build/ABI receipt，禁止引用任意本机HEAD作为稳定依赖。
6. 本轮没有发现新的canonical finding，因此继承Editor31编号而不扩增总账。

## 11. 分层重构顺序

1. **M0 Truthfulness/Profile**：统一三个target的provider矩阵；required backend在project/target preflight验证；冻结ZrVM source/build/ABI revision，修正NativeDynamic/Partial能力表达。
2. **M1 Workspace/Document**：在Editor02建立ScriptWorkspace、SourceDocument、VisualScriptDocument、revision/encoding/line map、external conflict与真实range navigation；给Script资源正式asset identity。
3. **M2 Shared Build/Artifact**：分离source generation、ticket、artifact与install identity；接唯一shared job owner；compiler返回bounded structured diagnostic pages和immutable content-addressed ArtifactSet。
4. **M3 Class/Component/Reflection**：建立stable class/component/field schema、default/override provenance、typed Inspector patch、reference graph及legacy `script.bindings`迁移。
5. **M4 Runtime Install/Reload**：shipping Runtime只加载verified artifact；candidate在旧generation外prepare，safe point按expected session/slot commit；package与component state分别迁移，失败保留last-good。
6. **M5 Visual Script**：建立typed graph/node/pin/provider、type/effect/cycle validation和semantic diff；lowering到同一compiler module/interface/artifact/debug map。
7. **M6 LSP/Debugger/Profiler**：接versioned ZrVM adapter，提供workspace sync、symbol、breakpoint、attach/session/thread/frame/value/evaluate/profile/coverage及stale处理。
8. **M7 Product Qualification**：真实Build command、build-before-Play、Client/Server/Editor一致性、WOC cold/warm/incremental/reload/debug、fault/scale/soak和同workload性能验证。

## 12. 禁止的临时修补

- 禁止在Console旁加文本框或语法着色控件就宣称Script Editor完成。
- 禁止用regex、扩展名、stderr解析或另一套parser替代ZrVM canonical semantics/LSP/debugger。
- 禁止让Editor和Runtime分别编译source后比较时间戳；shipping Runtime不得重新编译。
- 禁止继续用request id同时表示source、artifact、binding/install generation，或失败时清除更新source fact。
- 禁止把bounded retained log当作bounded diagnostic ingress；海量错误必须在producer page admission截断并产出receipt。
- 禁止给JSON map增加hardcoded Inspector字段后命名为Script Component。
- 禁止把Visual Script实现为独立解释器、类型系统或shipping artifact。
- 禁止先画断点/stack面板，再用静态row冒充debug session。
- 禁止绕过`HotReloadCoordinator`直接替换VM instance，或把package state migration冒充component override migration。
- 禁止把WOC源码/`.zro`数量或测试marker写成默认产品可运行、工程完整或性能优于Unreal的证据。

## 13. 验证边界

本轮完成Editor build/log/jump/catalog、Runtime Script全目录、Dynamic startup、Scene binding、ZrVM plugin/App feature/WOC纵切面复核，并核对Unreal Blueprint/Kismet/HotReload/LiveCoding、Godot Script/Debugger、Fyrox Script Inspector/Build、Bevy Reflect/Watcher、Unity Graphics范围和外部ZrVM binding/LSP/debug API。没有修改Rust/C/TOML生产实现。

本轮没有运行Cargo、ZrVM CMake/CTest、LSP/debug protocol、WOC full build、Editor UI、Play、Client/Server、hot reload、fault、scale、soak或跨引擎benchmark。原因是用户本轮要求先做review，且MVP门仍未授权高级产品实现；静态审查不能声称动态资格通过。Tooling按用户要求排除；实施前必须重算selected manifest、外部revision/API、default feature组合和Runtime startup顺序。
