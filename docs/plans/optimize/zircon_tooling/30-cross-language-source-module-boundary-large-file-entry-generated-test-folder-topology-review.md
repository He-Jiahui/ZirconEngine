---
related_code:
  - .github/workflows/ci.yml
  - tools/check_conventions.py
  - examples/woc/tools/package.json
  - examples/woc/scripts/woc_game/src/world/state.zr
  - examples/woc/scripts/woc_game/src/instances/delve_state.zr
  - examples/woc/scripts/woc_game/src/progression/inventory_vendor_state.zr
  - examples/woc/scripts/woc_game/src/combat/auto_attack_state.zr
  - examples/woc/scripts/woc_game/src/combat/effect_numeric_dispatch_state.zr
  - examples/woc/scripts/woc_game/src/combat/effect_world_dispatch_state.zr
  - examples/woc/scripts/woc_game/src/combat/damage_state.zr
  - examples/woc/scripts/woc_game/src/world/delve_collision_content.zr
  - examples/woc/scripts/woc_game/src/protocol/command_payloads.zr
  - examples/woc/tools/command_payload_codegen.mjs
  - examples/woc/tools/m3_delve_collision_content_codegen.mjs
  - examples/woc/tools/m8_eastbrook_encounter_codegen.mjs
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/git_finalize.py
  - tools/session_coordinator/workspace_copy.py
  - tools/session_coordinator/cli.py
  - tools/session_coordinator/cleanup.py
  - tools/session_coordinator/failures.py
  - tools/session_coordinator/workflows/milestones.py
  - tools/session_coordinator/workflows/failure_closeouts.py
  - tools/session_coordinator/supervision/service.py
  - tools/session_coordinator/web/package.json
  - tools/session_coordinator/web/tsconfig.json
  - tools/session_coordinator/web/tsconfig.test.json
  - tools/session_coordinator/web/src/api/contracts.ts
  - tools/session_coordinator/web/src/api/validation.ts
  - tools/session_coordinator/web/src/pages/OverviewPage.tsx
  - tools/session_coordinator/web/scripts/run-tests.mjs
  - tools/session_coordinator/web/scripts/verify-dist.mjs
  - zircon_hub/package.json
  - zircon_hub/tsconfig.json
  - zircon_hub/web/src/types/hub.ts
  - zircon_hub/web/src/data/hubData.ts
  - tools/editor-workbench-preview/package.json
  - tools/editor-workbench-preview/design.js
  - tools/editor-workbench-preview/design.css
  - tools/editor-workbench-preview/verify-designs.mjs
  - tools/ui-profile-capture.ps1
  - tools/mvp/Invoke-MvpAcceptance.ps1
  - tools/mvp/Stage-MvpProducts.ps1
  - tools/mvp/MvpAcceptanceStagingSnapshot.psm1
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
  - tools/zircon_validate_shader_pbr_viewer_evidence.py
  - zircon_plugins/navigation/native/build.rs
  - zircon_plugins/navigation/native/native/recast_bridge.h
  - zircon_plugins/navigation/native/native/detour_query.cpp
  - zircon_plugins/navigation/native/native/detour_tile_cache.cpp
  - zircon_plugins/navigation/native/native/recast_bake.cpp
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process_screen_space_reflection.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/shaders/trace_schedule_handoff.wgsl
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/shaders/update_completion.wgsl
tests:
  - tools/session_coordinator/tests/test_server.py
  - tools/session_coordinator/tests/test_cargo_runner.py
  - tools/session_coordinator/tests/test_git_finalize.py
  - tools/session_coordinator/web/src/__tests__/contracts.test.ts
  - tools/session_coordinator/web/src/__tests__/components.test.tsx
plan_sources:
  - docs/plans/optimize/zircon_app/03-woc-product-role-host-zrvm-transaction-state-client-server-integration-review.md
  - docs/plans/optimize/zircon_hub/02-web-shell-catalog-settings-team-cloud-accessibility-performance-review.md
  - docs/plans/optimize/zircon_runtime/08d-navigation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/12-woc-zrvm-package-kernel-world-state-schedule-serialization-runtime-review.md
  - docs/plans/optimize/zircon_runtime/18-woc-generated-content-catalog-buildset-install-query-runtime-review.md
  - docs/plans/optimize/zircon_runtime/19-woc-command-protocol-payload-codec-admission-movement-outcome-runtime-review.md
  - docs/plans/optimize/zircon_runtime/20-woc-package-root-world-api-facet-registry-snapshot-command-publication-review.md
  - docs/plans/optimize/zircon_runtime/21-zr-language-parser-type-system-semir-bytecode-package-loader-vm-runtime-review.md
  - docs/plans/optimize/zircon_tooling/05-woc-content-codegen-build-scripts-generated-artifact-incremental-review.md
  - docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md
  - docs/plans/optimize/zircon_tooling/07-performance-benchmark-profile-capture-symbol-crash-evidence-baseline-review.md
  - docs/plans/optimize/zircon_tooling/13-repository-codex-skill-hook-structural-audit-governance-security-currentness-review.md
  - docs/plans/optimize/zircon_tooling/14-editor-workbench-design-spec-screenshot-export-visual-evidence-prototype-governance-review.md
  - docs/plans/optimize/zircon_tooling/15-mvp-build-staging-product-process-acceptance-evidence-resource-baseline-control-plane-review.md
  - docs/plans/optimize/zircon_tooling/17-repository-content-source-set-ignore-generated-vendor-license-distribution-review.md
  - docs/plans/optimize/zircon_tooling/19-script-interpreter-entrypoint-command-registry-cli-operation-receipt-review.md
  - docs/plans/optimize/zircon_tooling/21-unsafe-rust-ffi-native-memory-thread-affinity-panic-unload-safety-governance-review.md
  - docs/plans/optimize/zircon_tooling/27-version-domain-schema-compatibility-support-window-migration-deprecation-upgrade-downgrade-review.md
  - docs/plans/optimize/zircon_tooling/29-rust-module-boundary-root-entry-large-file-declaration-behavior-folder-topology-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/AutomationUtils/BuildCommand.cs
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/BuildGraph/BuildGraph.cs
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/BuildGraph/Tasks/CompileTask.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Unity.RenderPipelines.Core.Runtime.asmdef
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraph.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraph.Compiler.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/Debug/RenderGraph.DebugData.cs
  - dev/godot/pyproject.toml
  - dev/godot/.pre-commit-config.yaml
  - dev/godot/methods.py
  - dev/godot/modules/navigation_3d/SCsub
  - dev/godot/platform/windows/detect.py
  - dev/bevy/crates/bevy_pbr/src/render/pbr_fragment.wgsl
  - dev/bevy/crates/bevy_pbr/src/render/pbr_functions.wgsl
  - dev/bevy/crates/bevy_pbr/src/render/pbr_lighting.wgsl
  - dev/Fyrox/fyrox-impl/src/renderer/shaders/ssao.shader
  - dev/Fyrox/fyrox-impl/src/renderer/shaders/deferred_directional_light.shader
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 30 · Cross-Language Source Architecture、Large File、Entry、Generated/Test 与 Folder Topology 审查

## 1. 结论

Rust并不是当前唯一的源码结构风险。Zircon仓库还用Zr承载产品级世界状态与玩法，用Python承载Session Coordinator和大量仓库工具，用PowerShell承载Windows采集/验收/分发流程，用TypeScript/JavaScript承载Hub、控制台和原型，用C++承载Recast/Detour桥，用WGSL承载渲染pass。Tooling29只拥有Rust物理边界；如果其规则不能扩展为按语言、角色和构建目标校正的统一策略，巨型owner只会从`.rs`转移到另一种扩展名。

本轮对排除`dev/`、`docs/`、vendor/thirdparty、target和node_modules后的2,684个tracked跨语言路径做了归一化逐文件扫描，共684,736行、29,022,345个UTF-8内容字节。先按generated/dist/bin-tests路径和前12行锚定generated注释，再按精确测试路径/文件名分类，得到1,496个manual non-test/non-generated保守候选、383,212行、15,848,919字节；其中297个不少于300行、131个不少于500行、50个不少于800行、23个不少于1000行、13个不少于2000行。该分类是待正式parser/build graph校正的lower bound，不把每个长文件自动判为缺陷。

最严重的已确认结构断点是`examples/woc/scripts/woc_game/src/world/state.zr`：68,730行，占本轮manual行数约17.9%，直接导入538个模块，词法下界有约1,430个函数候选、232个public函数；它同时承载WorldState、命令执行、战斗/社交/进度编排、序列化、反序列化和从46,133行开始的巨型`selfTest`。`decodeState`约2,487行、`encodeStateVersion`约2,202行。已有Runtime12至20拥有功能与协议正确性，本篇只把package root/facet/codec/test的物理切分登记为结构owner。

Session Coordinator不是“完全未拆分”：它已经有62个root Python文件、workflows/control_plane/codex_sync/supervision子包和102个测试文件。但7个核心服务仍各自聚合1,662至2,971行class body，`server.py`的`CoordinatorApplication._command_unlocked`约1,056行，`cli.py`的`_run`约959行。其问题是command/application/service owner仍过宽，不是简单把每300行切一个文件。`migrations.py`的2,834行则是65个顺序migration函数，必须以不可变迁移单元、schema catalog和回放门治理，不能按行数重写历史。

仓库的跨语言检查链也不闭合。根CI会运行Python编写的约定测试和Rust约定门，但`tools/check_conventions.py`的命令只覆盖Rust structure/fmt/clippy及文档/例外规则；CI没有Hub或Coordinator Web的`npm`安装/typecheck/test/build，没有全仓Python lint/typecheck、PowerShell analyzer、JS lint、C++ static analysis或WGSL全变体结构门。局部`package.json`脚本、Hub strict TypeScript和Coordinator Web测试是可保留基础，不能被当成required CI已经覆盖。

本篇不重复Tooling05/06/07/14/15/17/19/21/27的codegen、协调器语义、性能证据、原型、MVP、SourceSet、脚本操作、FFI安全或迁移正确性，也不接管App03、Hub02、Runtime08d/09c/12/18/19/20/21的产品与运行时功能。Tooling30是非Rust源码物理角色、entry/service/schema/generated/test边界、folder topology和跨语言结构门的canonical专项owner；登记 **0项P0、64项P1和16项P2**。

## 2. 审查边界、口径与限制

| Evidence | 本轮结果 |
|---|---|
| E1 tracked universe | 2,684个`.c/.cpp/.h/.cs/.ts/.tsx/.js/.mjs/.py/.ps1/.psm1/.sh/.html/.css/.wgsl/.glsl/.hlsl/.metal/.lua/.zr`路径，684,736行 |
| E2 conservative manual lower bound | generated优先、其后test分类；1,496个manual候选、383,212行、15,848,919字节 |
| E3 size distribution | manual `>=300` 297、`>=500` 131、`>=800` 50、`>=1000` 23、`>=2000` 13 |
| E4 role exclusions | 158个generated；其余1,030个test/fixture候选；30个generated Zr leaf位于非`generated/`目录但有锚定文件头 |
| E5 language lower bound | manual Python 503/135,014行；Zr 285/128,908；MJS 377/46,185；PowerShell脚本/模块64/28,271；WGSL 127/16,839 |
| E6 folder fanout | WOC tools 368个direct manual MJS；`zircon_export` 246个direct Python；WOC combat/world/progression/instances分别85/59/47/40个direct manual Zr；Coordinator root 62个Python |
| E7 semantic spot reads | 逐读23个千行热点；对7个Coordinator巨型服务做Python AST class/method跨度检查；对5个PowerShell入口做AST parse；抽查Hub/控制台TS、导航C ABI与5个大型WGSL |
| E8 TS boundary | Hub `hub.ts` 776行/47 interfaces/7 aliases，`hubData.ts` 681行；Coordinator `validation.ts` 553行/44函数，`OverviewPage.tsx` 507行，`contracts.ts` 449行/41 interfaces |
| E9 native/shader boundary | `detour_query.cpp` 886行、`recast_bridge.h` 329行/23 structs；`post_process.wgsl` 910行/53函数/7 entrypoints，SSR 854行/51函数/5 entrypoints |
| E10 dynamic/static validation | 本轮只做只读清单、AST/词法和文件审查；未编译生产代码，未重跑已知Editor、Hub、WOC、plugin阻断；文档落盘后执行路径、ID、链接、计数与docs validator |
| Currentness | branch `main`，revision `ae2be3d865a937b9ed368bf965592045346c64e3`；100个frontmatter输入按path ordinal排序，每项编码为`path + LF + normalized UTF-8 content + LF`，fingerprint `95f0a0c34d689cda4e94b9d436a04f36a8e53c83ffc84204bf83545d3f605f65`，175,418个normalized content LF、8,048,542 content bytes；`.codex/.../validate-matrix.ps1`取证时已有相邻修改，18篇输入优化报告处于本轮既存untracked状态，本篇不修改任何production/test/CI/manifest |

统计解释：

1. 行数按CRLF/CR归一为LF后计数；byte是归一化UTF-8内容字节，不是磁盘allocation。
2. test/generated分类来自路径、文件名和锚定文件头，不理解Zr package manifest、Node生成清单、Python packaging或shader assembly；正式门必须记录分类reason和generator authority。
3. 300/500行只触发审查。大型cohesive shader函数库、schema migration history或数据声明可申请有期限waiver；混合entry、跨域service和内嵌测试不能仅凭“还能工作”豁免。
4. `design.js`/CSS是prototype实现，不能按production runtime要求重构；但它必须被明确隔离，不能继续作为Editor功能真相或复制源。
5. 本轮不根据文件长度推断帧性能、编译性能或功能成熟度；拆分同样可能增加动态dispatch、序列化复制、shader重复编译和模块加载成本。

## 3. 必须保留的工程基础

### 3.1 局部语言工具链不是空白

Hub和Coordinator Web都启用TypeScript `strict`，前者build先typecheck，后者`check`串联typecheck、test、build和dist可达性/敏感材料检查。Coordinator的test入口会用独立`tsconfig.test.json`编译测试后交给Node test runner。问题是这些入口没有进入根required CI，也没有统一receipt。

### 3.2 Coordinator已经形成部分子域

`workflows/`、`control_plane/`、`codex_sync/`、`supervision/`及大量focused tests证明按owner拆分可行。后续应把巨型application/service中的command routing和领域事务迁入现有子域，不另造`helpers2.py`或`service_partN.py`。

### 3.3 WOC生成物已有generator和`--check`模式

collision、command payload、content catalog等生成脚本会记录来源或生成头，若干入口支持check模式。这是建立`GeneratedSourceReceipt`的基础。需要补的是统一manifest、输出路径策略、generator digest与CI全覆盖，而不是删掉codegen回到手写复制。

### 3.4 导航native桥已有Rust安全表面和focused测试

native crate已经把Rust侧bake/query/crowd/tile cache拆为命名模块，并通过`build.rs`显式列出Zircon-owned C++和vendored Recast/Detour。结构迁移应保留C ABI、ownership和结果释放语义；Tooling21与Runtime08d继续拥有安全和行为验收。

### 3.5 Shader源码已按render feature放入相邻目录

后处理、mesh、environment、Hybrid GI等多数WGSL已有feature owner路径。问题集中在单个文件混装多个pass/entrypoint、公共函数复制和缺少全变体source graph，而不是把127个WGSL全部迁到单一共享目录。

## 4. 已确认的结构断点

### 4.1 全仓只有Rust结构规则形成了明确门

`GEN-S3/S4`、Runtime/Editor结构审计和Tooling29均以Rust为中心。Python class、PowerShell command root、TS contract barrel、C ABI header、shader entrypoint和Zr package root没有同等FileRole/owner规则。不同语言无法汇总成一份可阻断的SourceArchitectureFindingSet。

### 4.2 `world/state.zr`已经成为第二套应用/运行时总线

538个import横跨combat/progression/social/world/instances/generated；WorldState构造会连续填充大量default column；同一文件含`applyCommands`、序列化/反序列化、offline规则和近2.3万行`selfTest`尾部。它既不是薄package root，也不是单一cohesive算法。

### 4.3 Coordinator的包拆分没有穿透到核心application/service

`CargoJobService`约2,971行/69 methods，`GitFinalizeService`约2,160行/69 methods，`WorkspaceCopyService`约2,257行/69 methods，`MilestoneWorkflowService`约2,389行/42 methods，`SupervisionService`约1,662行/38 methods。它们把admission、persistence、process lifecycle、recovery、projection和policy组合在一个mutation surface，增加并发与事务审查面积。

### 4.4 CLI与HTTP command routing仍是巨型条件分派

`cli.py`的parser约543行、`_run`约959行；`server.py`的`CoordinatorApplication`约2,255行，`_command_unlocked`约1,056行。新增一个命令会同时触碰parse、auth/admission、service call、response projection和error mapping，无法由命令注册表检查exhaustiveness。

### 4.5 PowerShell入口把orchestration与implementation放在同一脚本

`ui-profile-capture.ps1`有60个函数/87个top-level statements，MVP acceptance有43个函数，staging有34个函数，snapshot module有23个函数，skill validation matrix有38个函数。AST均可解析，但主入口仍持有采集、进程、文件、证据、报告和清理实现，缺少module contract与平台适配边界。

### 4.6 WOC codegen目录以里程碑前缀代替领域包

`examples/woc/tools`有368个direct manual MJS，混合source extract、catalog contract、codegen、asset check、static guard、parity和M3/M4/M5/M8里程碑命名。`command_payload_codegen.mjs`单文件2,052行；路径无法稳定表达generator family、输入authority或输出owner。

### 4.7 Web contract与demo fallback再次集中

Hub `types/hub.ts`聚合47个interface和7个alias，覆盖project/build/catalog/settings/team/cloud等页面；`hubData.ts`把681行fallback shell/demo数据放入production source。Coordinator Web又独立维护449行contract和553行手写validation。跨Rust/Python/TS的schema没有机器生成或版本绑定，结构上容易出现字段漂移。

### 4.8 Prototype和generated dist进入tracked source universe

workbench preview把212个函数候选放入9,183行`design.js`，CSS有约499个selector block；Coordinator Web的`dist/assets`有28个tracked构建输出。它们可以作为可复核证据或嵌入资源存在，但必须有明确ArtifactRole、source digest和regeneration gate，不能与手写production source同口径维护。

### 4.9 C ABI header和桥实现横跨多个native domain

`recast_bridge.h`把23个struct及bake/query/crowd/tile-cache函数放在一个public header；query/tile-cache/crowd实现重复`set_message`、`finite3`、`distance3`、result reset/status等桥接逻辑。单一header使任一子域变化扩大ABI审查面，也缺少机器生成的layout/version清单。

### 4.10 WGSL文件按“最终管线”而非pass owner聚合

`post_process.wgsl`同时实现FXAA、DOF、motion blur、chromatic aberration、fog、vignette、grain/dither、LUT和SSR composite，暴露7个entrypoint；SSR文件又把depth/reflection pyramid、ray trace、temporal history和5个entrypoint放在一起。`fallback_mesh.wgsl`含6个entrypoint。现有文件都没有源码级`#import`，共享/组装关系由宿主侧承担，缺少可审计ShaderSourceGraph。

## 5. P0：无新增独立项

本轮没有创建新的P0。WOC产品真实性、协议/状态兼容、Coordinator并发与授权、native FFI安全、shader正确性均已有P0/P1功能owner。本篇的结构债务会扩大这些风险，但不能重复计数或用文件拆分替代功能修复。

## 6. P1：Cross-Language SourceSet、Classifier 与 Required Gate

| ID | 当前差距 | 工程级重构要求 |
|---|---|---|
| CROSS-SOURCE-P1-001 | 没有全仓跨语言SourceSet | 从Git、workspace/package/build manifests和显式auxiliary registry生成versioned清单 |
| CROSS-SOURCE-P1-002 | 语言集合靠临时扩展名列表 | 每种语言声明parser、build target、generated/test规则、owner和quality commands |
| CROSS-SOURCE-P1-003 | test/generated/production分类靠路径猜测 | `SourceUnitClassifier`输出role、reason、authority、target和content digest |
| CROSS-SOURCE-P1-004 | generated header只在部分Zr生效 | 统一锚定header/schema/generator manifest；30个目录外generated leaf均可追溯 |
| CROSS-SOURCE-P1-005 | 没有非Rust结构baseline | 固化本轮23个千行、50个800行热点和高fanout目录，新增required fail |
| CROSS-SOURCE-P1-006 | 根CI不运行跨语言质量矩阵 | 增加Python/Node/PowerShell/C++/shader/Zr按平台可用性的required jobs和typed skip |
| CROSS-SOURCE-P1-007 | 局部脚本成功不等于CI覆盖 | 每个command产出FindingSet/receipt，required runner验证实际执行而非package script存在 |
| CROSS-SOURCE-P1-008 | 行数策略无法表达语言语义 | policy按FileRole和syntax role触发，cohesive例外必须有owner、证据、expiry |

## 7. P1：Zr Package Root、World Facet、Codec 与 Test Owner

| ID | 当前差距 | 工程级重构要求 |
|---|---|---|
| CROSS-SOURCE-P1-009 | `world/state.zr`为68,730行聚合根 | 建立thin world facade，只装配state store、system registry与公开facet |
| CROSS-SOURCE-P1-010 | 538个import直接扇入root | 按combat/progression/social/instance/world facet提供curated dependency boundary |
| CROSS-SOURCE-P1-011 | WorldState声明与default-column初始化共居 | schema declaration、column defaults、entity factory和migration分别归owner |
| CROSS-SOURCE-P1-012 | `applyCommands`仍在state root | command admission/dispatch按protocol family路由，并有exhaustive registry |
| CROSS-SOURCE-P1-013 | encode/decode各跨两千行 | 按schema version和facet拆codec，保留canonical order、bounds与golden digest |
| CROSS-SOURCE-P1-014 | offline gameplay helper回流root | feature实现下沉原domain module，root不保留第二份规则或compat helper |
| CROSS-SOURCE-P1-015 | `selfTest`尾部近2.3万行 | 迁入反向绑定owner的Zr test package，按facet/codec/command分suite |
| CROSS-SOURCE-P1-016 | `delve_state`及多个combat/progression文件继续膨胀 | 用同一policy逐文件做cohesion review，不以拆完root为终点 |

## 8. P1：Python Application、Service、Migration 与 CLI Boundary

| ID | 当前差距 | 工程级重构要求 |
|---|---|---|
| CROSS-SOURCE-P1-017 | `CoordinatorApplication`聚合41个methods | application只做composition/auth boundary，命令交给typed handler registry |
| CROSS-SOURCE-P1-018 | `_command_unlocked`约1,056行 | 每个command family独立request/handler/result/error mapper并可穷尽检查 |
| CROSS-SOURCE-P1-019 | CargoJobService混合admission/process/reconcile | 拆lane admission、reservation ledger、process observer、reconciler和projection |
| CROSS-SOURCE-P1-020 | GitFinalizeService混合preview/index/commit/recovery | 按intent、scope validation、index transaction、commit writer和recovery拆分 |
| CROSS-SOURCE-P1-021 | WorkspaceCopyService混合plan/materialize/run/cleanup | 分离planner、materializer、runner、terminal lifecycle和managed-root policy |
| CROSS-SOURCE-P1-022 | Milestone/Supervision巨型事务owner | 将gate/review/manifest/reconcile与state transition/proof/lifecycle按事务边界拆分 |
| CROSS-SOURCE-P1-023 | CLI parser与执行器均为巨型函数 | declarative command spec生成parser，handler只调用application port并渲染结果 |
| CROSS-SOURCE-P1-024 | migrations按单文件累积且可被随意编辑 | 每个migration不可变、独立identity/digest，catalog排序，fresh/upgrade/downgrade回放 |

## 9. P1：PowerShell、JavaScript、Codegen 与 Prototype Boundary

| ID | 当前差距 | 工程级重构要求 |
|---|---|---|
| CROSS-SOURCE-P1-025 | PowerShell command root含大量实现函数 | root只解析参数/加载module/调用operation/映射exit，逻辑进入`.psm1` owner |
| CROSS-SOURCE-P1-026 | capture脚本混合窗口、输入、证据、报告 | 分离platform adapter、interaction driver、artifact collector、evidence evaluator |
| CROSS-SOURCE-P1-027 | MVP acceptance混合stage/run/assert/package | 由typed acceptance plan描述阶段，PowerShell只执行有receipt的operation |
| CROSS-SOURCE-P1-028 | validation matrix脚本承担协调器策略 | skill入口消费canonical validation service，不复制lane/target/path政策 |
| CROSS-SOURCE-P1-029 | WOC 368个MJS平铺 | 按extractors/generators/contracts/checks/parity/assets及领域子包迁移 |
| CROSS-SOURCE-P1-030 | milestone前缀成为长期owner | M3/M5/M8只留历史metadata，稳定路径使用domain和artifact identity |
| CROSS-SOURCE-P1-031 | 2,052行payload generator混合schema/render/check | 拆schema reader、semantic validation、renderer、manifest writer和check adapter |
| CROSS-SOURCE-P1-032 | 9,183行prototype JS可能成为复制源 | 冻结为versioned design evidence，production实现只消费规范/asset，不复制代码 |

## 10. P1：TypeScript Contract、Validation、Page 与 Artifact Boundary

| ID | 当前差距 | 工程级重构要求 |
|---|---|---|
| CROSS-SOURCE-P1-033 | Hub类型集中于单一776行barrel | 按project/build/catalog/settings/team/cloud/action拆schema，root只curated export |
| CROSS-SOURCE-P1-034 | 681行fallback demo data位于production data owner | 移入显式demo fixture/provider，真实空态与demo态由capability选择 |
| CROSS-SOURCE-P1-035 | Python/Rust/TS手工复制控制面contract | 从versioned API schema生成DTO和runtime validator，保留semantic adapters |
| CROSS-SOURCE-P1-036 | 553行validation含44个手写函数 | 按contract family拆validator或由schema生成，cross-field policy单独命名 |
| CROSS-SOURCE-P1-037 | Overview page混合25个派生/格式函数 | page负责composition；selector/view-model/format/action分别归owner |
| CROSS-SOURCE-P1-038 | Hub package没有test script或web测试路径 | 建立contract/component/a11y focused tests并纳入Hub `check` |
| CROSS-SOURCE-P1-039 | Coordinator production tsconfig排除tests | 保留独立test config，但required入口必须同时证明prod/test typecheck与执行 |
| CROSS-SOURCE-P1-040 | tracked dist没有source binding receipt | dist manifest记录source tree、lockfile、toolchain、base path、asset graph和digest |

## 11. P1：Native C ABI 与 WGSL Source Graph

| ID | 当前差距 | 工程级重构要求 |
|---|---|---|
| CROSS-SOURCE-P1-041 | 单一header暴露bake/query/crowd/tile cache | 以稳定umbrella facade包含按domain ABI header，符号仍保持hard cut一致 |
| CROSS-SOURCE-P1-042 | 23个ABI struct没有机器可读layout catalog | 生成size/alignment/offset/version清单并与Rust FFI双向断言 |
| CROSS-SOURCE-P1-043 | native结果/error/finite helper重复 | 建立内部非ABI bridge support owner，禁止复制ownership/status语义 |
| CROSS-SOURCE-P1-044 | build.rs手列source但无Zircon-owned source receipt | 输出compiler/flags/source/vendor revision/ABI fingerprint并进入artifact provenance |
| CROSS-SOURCE-P1-045 | post-process文件混装多种独立pass | 拆共享math/resource、effect kernels和entrypoint leaf，由manifest组装pipeline |
| CROSS-SOURCE-P1-046 | SSR文件混合pyramid/trace/temporal/resolve | 按pass graph拆分并保留binding layout、dispatch顺序和history golden |
| CROSS-SOURCE-P1-047 | fallback mesh含6个entrypoint与完整PBR fallback | 分离vertex deformation、material sampling、lighting和velocity/reactive entry leaf |
| CROSS-SOURCE-P1-048 | 没有canonical ShaderSourceGraph | 每个entrypoint声明imports、bindings、permutation、host owner、digest和compile receipt |

## 12. P1：Generated/Test Ownership、Folder Topology 与 Migration Safety

| ID | 当前差距 | 工程级重构要求 |
|---|---|---|
| CROSS-SOURCE-P1-049 | generated leaf可散落任意domain目录 | manifest决定就地生成或`generated/`，两者都必须有header和generator identity |
| CROSS-SOURCE-P1-050 | 30个目录外generated Zr缺统一索引 | 建立GeneratedSourceCatalog，列input/output/schema/tool/digest/check command |
| CROSS-SOURCE-P1-051 | test role仅靠路径/文件名 | parser识别Zr selfTest、Python/TS test constructs、PowerShell test harness和shader fixture |
| CROSS-SOURCE-P1-052 | `world/state`内嵌测试不可独立选择 | TestOwner manifest按facet绑定production symbols和focused command |
| CROSS-SOURCE-P1-053 | 368/246/85/62等高fanout目录无decision | 结合domain/co-change/import graph形成FolderTopologyDecision，不机械限制文件数 |
| CROSS-SOURCE-P1-054 | tools root仍有52个跨域文件 | 按build/profile/MVP/export/repository governance设命名package与thin entry |
| CROSS-SOURCE-P1-055 | 拆分没有跨语言symbol/artifact事务 | RefactorTransaction记录paths、exports、CLI/API/ABI/entrypoint、generated输出和rollback |
| CROSS-SOURCE-P1-056 | 结构迁移可能破坏性能/确定性 | 对codec、shader、process、filesystem和FFI分别绑定golden/benchmark/trace，不以行数变小验收 |

## 13. P1：Quality Toolchain、CI 与 Ownership Convergence

| ID | 当前差距 | 工程级重构要求 |
|---|---|---|
| CROSS-SOURCE-P1-057 | 根仓没有Python quality config | 建立pinned formatter/linter/type policy，按production/tool/test渐进baseline |
| CROSS-SOURCE-P1-058 | 根仓没有JS/TS lint/format policy | 统一Node toolchain、lockfile和lint规则，局部package只扩展不复制 |
| CROSS-SOURCE-P1-059 | 没有PowerShell analyzer policy | 固定PSScriptAnalyzer版本/规则，Windows required job覆盖command/module/tests |
| CROSS-SOURCE-P1-060 | Zircon-owned C++没有clang-tidy/format contract | 建立与vendored source分离的compile_commands、warning和static analysis gate |
| CROSS-SOURCE-P1-061 | WGSL只随具体pipeline间接编译 | 对127个tracked shader和声明permutation做standalone parse/validate/reflect矩阵 |
| CROSS-SOURCE-P1-062 | Zr没有source architecture command | 复用真实parser/package loader输出imports/symbols/test/generated roles并阻断新债务 |
| CROSS-SOURCE-P1-063 | CI不能证明局部package check被执行 | required matrix记录command、tool version、input digest、duration、result和skip reason |
| CROSS-SOURCE-P1-064 | 结构owner与功能owner可能互相覆盖 | central owner map保持一项canonical结构Finding，功能报告提供行为/性能acceptance |

## 14. P2：后续增强

| ID | 增强项 | 目标 |
|---|---|---|
| CROSS-SOURCE-P2-001 | IDE source-role projection | 在编辑器中显示owner、role、generator和waiver状态 |
| CROSS-SOURCE-P2-002 | 跨语言import graph | 观察Zr/TS/Python/shader/C ABI依赖与环 |
| CROSS-SOURCE-P2-003 | co-change heatmap | 为高fanout目录提供实际拆分证据 |
| CROSS-SOURCE-P2-004 | semantic diff summary | 报告schema/API/ABI/entrypoint变化而非只看文本 |
| CROSS-SOURCE-P2-005 | compile-time trend | 量化拆分对Python startup、TS build、C++ compile和shader compile影响 |
| CROSS-SOURCE-P2-006 | waiver dashboard | 追踪大型cohesive文件例外、expiry与复审 |
| CROSS-SOURCE-P2-007 | generated drift dashboard | 展示dirty output、tool/input版本和重生成成本 |
| CROSS-SOURCE-P2-008 | test-owner coverage map | 由生产symbol反查focused suite与缺口 |
| CROSS-SOURCE-P2-009 | PowerShell platform parity | 对Windows-only operation记录可替换port和非Windows typed skip |
| CROSS-SOURCE-P2-010 | Python import/startup profile | 防止模块拆分造成控制台冷启动退化 |
| CROSS-SOURCE-P2-011 | shader include cache | 复用内容寻址模块并观察重复编译/缓存命中 |
| CROSS-SOURCE-P2-012 | ABI header generation | 从schema生成C/Rust声明和layout tests，减少手工镜像 |
| CROSS-SOURCE-P2-013 | web schema documentation | 从同一contract生成API reference和示例payload |
| CROSS-SOURCE-P2-014 | Zr package browser | 按facet浏览imports、public API、codec和test owner |
| CROSS-SOURCE-P2-015 | topology budget suggestions | 只建议审查，不自动移动文件或创建`misc`目录 |
| CROSS-SOURCE-P2-016 | architecture receipt viewer | 关联refactor、validation、performance和rollback证据 |

## 15. 23个当前`>=1000`候选的owner级处置

| 行数 | 路径 | 处置 |
|---:|---|---|
| 68,730 | `examples/woc/scripts/woc_game/src/world/state.zr` | P1，按world facet/schema/command/codec/test做hard cut |
| 9,183 | `tools/editor-workbench-preview/design.js` | prototype冻结；按design scene/component/action拆证据，不成为production依赖 |
| 3,620 | `tools/editor-workbench-preview/design.css` | 与prototype component/token owner对齐，保持视觉golden |
| 3,332 | `tools/session_coordinator/cargo_jobs.py` | admission/reservation/process/reconcile/projection拆分 |
| 3,046 | `tools/session_coordinator/server.py` | thin application + typed command handlers |
| 2,834 | `tools/session_coordinator/migrations.py` | 保留顺序历史，迁为不可变migration unit/catalog，不机械重写 |
| 2,644 | `tools/ui-profile-capture.ps1` | command root与capture/evidence/platform modules分离 |
| 2,571 | `tools/mvp/Invoke-MvpAcceptance.ps1` | acceptance plan、operation、assertion和packaging分离 |
| 2,465 | `tools/session_coordinator/git_finalize.py` | intent/scope/index/commit/recovery拆分 |
| 2,448 | `tools/session_coordinator/workflows/milestones.py` | validation/review/gate/manifest/reconcile/close拆分 |
| 2,358 | `tools/session_coordinator/workspace_copy.py` | planner/materializer/runner/terminal/cleanup拆分 |
| 2,052 | `examples/woc/tools/command_payload_codegen.mjs` | schema/validator/renderer/manifest/check拆分 |
| 2,038 | `tools/session_coordinator/cli.py` | declarative spec/parser/handler/renderer拆分 |
| 1,816 | `tools/session_coordinator/supervision/service.py` | transition/proof/reservation/recovery/lifecycle拆分 |
| 1,802 | `tools/mvp/Stage-MvpProducts.ps1` | stage planner/copier/runner/receipt拆分 |
| 1,409 | `examples/woc/scripts/woc_game/src/instances/delve_state.zr` | 按instance lifecycle/affix/reward/reset owner复核 |
| 1,382 | `tools/mvp/MvpAcceptanceStagingSnapshot.psm1` | snapshot lease/copy/hash/restore owner拆分 |
| 1,338 | `.codex/skills/zircon-dev/scripts/validate-matrix.ps1` | skill adapter消费canonical validation API |
| 1,290 | `tools/session_coordinator/cleanup.py` | cleanup与retention owner已可见，继续分离policy/operation |
| 1,279 | `tools/session_coordinator/workflows/failure_closeouts.py` | closeout transaction、artifact、graph mutation拆分 |
| 1,239 | `tools/session_coordinator/failures.py` | graph query/mutation/import/projection拆分 |
| 1,220 | `tools/zircon_validate_shader_pbr_viewer_evidence.py` | PNG/metadata/profile/evidence validators按格式拆分 |
| 1,026 | `examples/woc/tools/m8_eastbrook_encounter_codegen.mjs` | 迁入encounter generator family并绑定input/output manifest |

注意：该表是当前snapshot，不是要求把所有文件降到相同长度。`migrations.py`、CSS、code generator与service class需要不同的syntax/owner政策。

## 16. 目标架构与核心合同

### 16.1 `CrossLanguageSourceUnit`

稳定字段至少包括repository path、language、package/build target、entry role、owner、test/generated/vendor/prototype role、parser/toolchain version和content digest。任何“未识别”都必须显式输出，不能默认为production或忽略。

### 16.2 `GeneratedSourceReceipt`

记录generator id/version/digest、ordered inputs、schema version、outputs、normalization、check command和reproducibility status。允许generated leaf就地靠近domain，也允许集中目录；禁止没有receipt的“看起来像生成”文件。

### 16.3 `CommandOperation`

Python/PowerShell/Node入口统一遵守parse -> authorize/admit -> plan -> execute -> collect -> publish result。入口不持有领域实现，operation有typed id、risk、inputs、side effects、receipt和exit mapping。

### 16.4 `ShaderSourceGraph`

每个shader module和entrypoint有稳定id、imports、binding layout、specialization/permutation、host pipeline owner、source digest和compile artifact。拆文件不应导致共享函数复制或未受控permutation爆炸。

### 16.5 `CrossLanguageRefactorTransaction`

记录before/after paths、public symbols、commands/API/ABI/entrypoints、generated outputs、test owners、performance baselines和rollback point。同批迁移消费者并删除旧owner，不保留长期compat/shim或`partN`。

## 17. 参考实现的结构差异

### 17.1 Unreal Engine

本地Unreal源码中AutomationTool有521个C#文件、UnrealBuildTool有322个C#文件，Engine Source下有744个`*.Build.cs`。AutomationTool按`AutomationUtils`、`BuildGraph`、platform和`BuildGraph/Tasks`分owner；`BuildCommand.cs`、`BuildGraph.cs`与`CompileTask.cs`分别代表command contract、graph orchestration和leaf task。参考点是程序集/目录/任务边界和显式build module，不是认为Unreal没有大文件。

### 17.2 Unity Graphics

本地Graphics有4,601个C#、120个asmdef和1,498个shader相关文件。Core RenderGraph相邻目录把compiler、debug、resource、pass、builder拆为命名文件；`RenderGraph.cs`本身仍有1,740行，证明cohesive core可保留例外，但周围有assembly、runtime/editor/test和subsystem边界。Zircon应学习assembly/schema owner，不照抄行数。

### 17.3 Godot

本地Godot有132个Python和213个`SCsub`，每个module/platform用`SCsub`或`detect.py`声明局部build owner；根`pyproject.toml`与pre-commit配置提供Python/仓库质量入口。它同样存在大型build helper，但module/platform边界比Zircon的tools root和WOC平铺脚本更可导航。

### 17.4 Bevy

本地Bevy有205个WGSL/WESL/GLSL文件，其中84个在`bevy_pbr`。PBR把bindings、types、mesh、lighting、fragment、prepass、shadow和feature pass分开；`pbr_functions.wgsl`/`pbr_lighting.wgsl`也超过千行，说明shader拆分必须以职责和compile graph为依据，而非统一800行门。

### 17.5 Fyrox

本地Fyrox有41个shader文件，renderer按bloom、SSAO、directional/point/spot light等pass命名，material标准shader另归`fyrox-material`。其文件较小不自动代表功能更强；可借鉴的是renderer pass与material owner的路径边界。

## 18. 分层实施顺序

### M0 · 冻结SourceSet与分类

保存2,684路径清单、23个千行候选、30个目录外generated leaf和高fanout目录；以Tooling17 SourceSet为仓库真相，增加语言/build target projection。

### M1 · 统一FindingSet和required matrix

先实现classifier、policy、receipt和baseline；把现有Hub/Coordinator Web checks接入CI，再逐步加入Python、PowerShell、C++、WGSL和Zr，缺工具链时typed skip而非静默绿。

### M2 · Zr world hard cut

冻结public API、schema order、command registry和snapshot golden；先迁内嵌tests，再按facet/codec/command/state construction拆分，每批由Runtime12至20验收。

### M3 · Coordinator application/service split

从`server.py`和`cli.py`的command registry开始，再拆cargo/finalize/workspace/milestone/supervision事务；每批保留数据库原子性、并发和recovery evidence，由Tooling06拥有功能验收。

### M4 · Generator、PowerShell 与 Web schema

重组WOC tool package、建立GeneratedSourceReceipt；让PowerShell入口变薄；收敛Hub/Coordinator API schema和validator，tracked dist绑定reproducible receipt。

### M5 · Native/shader source graph

先生成ABI layout inventory和ShaderSourceGraph，再拆header/bridge和post-process/SSR/fallback mesh；由Tooling21、Runtime08d/09c及渲染视觉/性能报告共同验收。

### M6 · Folder topology和waiver

对368/246/85/62等目录结合co-change/import graph做decision；cohesive平铺family可保留，但必须记录owner、理由和复审点。

### M7 · 持续性能与当前性

监控新增巨型owner、entry behavior、waiver expiry、generated drift、test owner、compile/startup/shader permutation和hot-path性能；结构receipt不冒充功能或性能证据。

## 19. 验收门

| Gate | 验收条件 |
|---|---|
| G01 | CrossLanguageSourceSet覆盖本轮扩展名、nested package、examples、tools、Hub与plugin native |
| G02 | 每个source unit有language/package/target/owner/role/reason |
| G03 | 同revision/tool policy重跑清单和FindingSet deterministic |
| G04 | 30个目录外generated Zr均绑定generator/input/output digest |
| G05 | test/generated/vendor/prototype/production分类有fixture tests |
| G06 | 本轮23个千行候选有稳定FindingId、owner和处置 |
| G07 | 新增未批准的巨型mixed owner使required gate失败 |
| G08 | waiver包含cohesion/performance证据、owner、scope和expiry |
| G09 | 根CI实际运行Hub和Coordinator Web完整check并保存receipt |
| G10 | Python quality gate覆盖production/tools/tests且baseline只降不升 |
| G11 | PowerShell analyzer在Windows执行，无法执行时不是隐式成功 |
| G12 | Zircon-owned C++与vendor分开lint/compile/static analysis |
| G13 | 127个WGSL及声明permutation可独立parse/reflect/compile |
| G14 | Zr parser输出imports/public symbols/test/generated roles |
| G15 | `world/state.zr`成为thin facade且不再含feature behavior |
| G16 | world facet依赖不再由538个root import直接扇入 |
| G17 | WorldState schema/default/entity factory/migration有独立owner |
| G18 | command registry exhaustively映射protocol family和handler |
| G19 | codec拆分保持canonical byte order、bounds和schema golden |
| G20 | `selfTest`迁为按facet可独立执行的test owner |
| G21 | Coordinator application只做composition/auth和handler dispatch |
| G22 | CLI parser由declarative spec生成且handler没有巨型分派 |
| G23 | cargo/finalize/workspace/milestone/supervision事务各有明确owner |
| G24 | migration identity/digest不可变，fresh/upgrade/downgrade回放通过 |
| G25 | PowerShell root只做参数、module、operation、result与exit映射 |
| G26 | WOC tools按稳定domain/artifact owner分包，不再靠M数字路由 |
| G27 | 每个generator都有check command和GeneratedSourceReceipt |
| G28 | prototype代码不能被production package导入或复制为runtime truth |
| G29 | Hub类型按domain拆分且curated facade保持consumer兼容 |
| G30 | demo fallback成为显式fixture/provider，不冒充真实能力 |
| G31 | Python/Rust/TS API schema有唯一authority和version |
| G32 | generated validator与handwritten cross-field policy边界清晰 |
| G33 | tracked dist可由pinned toolchain/lockfile/source digest复现 |
| G34 | navigation ABI按domain声明且umbrella facade保持稳定 |
| G35 | C/Rust size/alignment/offset/symbol inventory在CI一致 |
| G36 | bridge support消除重复且不改变ownership/status语义 |
| G37 | ShaderSourceGraph覆盖imports/bindings/permutations/host owner |
| G38 | post-process、SSR、fallback拆分保持entrypoint与binding兼容 |
| G39 | shader拆分没有重复编译/permutation/allocation性能退化 |
| G40 | 高fanout目录都有保留或拆分decision及证据 |
| G41 | 每个RefactorTransaction记录symbol/API/ABI/entry/artifact diff |
| G42 | 不产生`partN`、`misc`、`legacy`或长期compat owner |
| G43 | 结构变绿不被当成功能完整、画质或性能领先证明 |
| G44 | source/input/tool变化自动把本报告及相关receipt标记stale |

## 20. 与既有报告的责任边界

| 依赖报告 | 本篇消费 | 仍由原报告拥有 |
|---|---|---|
| Tooling05 | WOC generators、inputs/outputs与drift | codegen正确性、增量构建和artifact语义 |
| Tooling06 | Coordinator服务/事务/测试拓扑 | 并发、lease、resource、finalize、supervision正确性 |
| Tooling07/15 | capture/MVP脚本与证据链 | 性能采集、产品staging和acceptance真实性 |
| Tooling13/17 | required runner、SourceSet和generated/vendor分类 | 通用执行治理、license、ignore与分发source truth |
| Tooling14 | workbench prototype和视觉证据 | design spec、截图、原型当前性与产品映射 |
| Tooling19 | script/CLI operation identity | 命令语义、权限、side effect和operation receipt |
| Tooling21 | native FFI/ownership/thread/panic安全 | C ABI安全与unload correctness |
| Tooling27 | migration/version/support policy | schema兼容、升级/降级和deprecation语义 |
| Tooling29 | Rust root/large-file/folder policy | Rust parser、Cargo SourceSet和Rust结构迁移 |
| App03 | WOC产品role和真实host闭环 | client/server/ZrVM transaction与产品资格 |
| Hub02 | Hub Web功能与体验 | catalog/settings/team/cloud/a11y/performance |
| Runtime08d/09c | navigation/shader功能 | nav行为、material/pipeline/PSO/渲染正确性 |
| Runtime12/18/19/20/21 | WOC state/content/protocol/root/Zr language | 运行时语义、兼容、VM与package能力 |

边界规则：

1. Tooling30回答“非Rust源码是什么角色、放在哪里、entry/service/schema/generated/test边界如何受控”；不重新定义业务行为。
2. Tooling29与Tooling30应消费同一repository SourceSet/FindingSet骨架，但各语言parser和policy不同；不能用Rust 1000行MUST直接套shader、migration或CSS。
3. 功能报告中的P0/P1仍决定修复优先级。结构拆分只有在原功能owner可验证时才能实施。
4. 任何source move都必须hard cut并更新build/package/import/manifest；禁止为了短期兼容保留双authority。
5. 性能目标必须用真实产品、frame/capture/benchmark和资源预算证明；文件更短、目录更多或lint全绿都不是性能领先证据。

## 21. 本轮产出与限制

本轮只新增审查文档并更新优化索引，不修改Zr、Python、PowerShell、JS/TS/CSS、C++、WGSL、测试、CI、package或manifest。所有语言扫描、AST和参考源码读取均为只读。`check_conventions.py --only docs --json`返回exit 1、2,653篇结构化文档、78,468条checked path、692项既有违规、242篇受影响，Tooling30自身0项；frontmatter、Finding ID、汇总计数、coverage连续性、内部链接、fingerprint和`git diff --check`均另行复核。该红baseline未恶化，不能描述为全仓通过。

在G01至G44完成前，Zircon拥有的是若干局部strict/typecheck/test入口、部分清晰子包和大量可用实现，不是覆盖所有source language、能识别entry/service/schema/generated/test语义、可阻断新增结构债务并保留行为/性能证据的工程级源码架构体系。尤其`world/state.zr`和Coordinator核心服务必须作为真实owner重构，而不能用新的wrapper、compat路径或更多临时代码遮住现有聚合。
