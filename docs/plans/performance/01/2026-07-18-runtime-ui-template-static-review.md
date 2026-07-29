---
related_code:
  - zircon_runtime/src/ui/template
  - zircon_runtime/src/ui/tests/asset_compile_cache.rs
  - zircon_runtime/src/ui/tests/asset_mui_web_style.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
  - docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Editor/UMGEditor/Private/WidgetBlueprintCompiler.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationWidgetList.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Styling/SlateStyleRegistry.cpp
  - dev/slint/internal/compiler/llr/lower_to_item_tree.rs
  - dev/slint/internal/compiler/passes/apply_default_properties_from_style.rs
  - dev/Fyrox/fyrox-ui/src/build.rs
  - dev/Fyrox/fyrox-ui/src/widget.rs
tests:
  - five source-level RED to GREEN performance guards passed
  - rustfmt and scoped git diff checks passed
  - current-source Windows template_asset_hot_paths passed 5/5 through shared Cargo coordinator
  - compile/style/hot-reload/authoring scale counters pending
  - F4 asset preview, edit and hot-reload trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI template逐文件性能静态审查（2026-07-18）

## 范围与覆盖

本批逐文件完整阅读`zircon_runtime/src/ui/template/**` 83/83：root pipeline 5/5、`build/**` 10/10、`asset/**` 68/68。累计UI tracked source从371/783增至454/783；物理current UI仍因外部未跟踪`tests/v2_asset/performance_guards.rs`为784。该文件不属于本批，随后已在v2 tests切片完成静态读取，但仍未动态验收。

动态门禁已于2026-07-18通过shared Cargo coordinator job `3303f080105640b296f969377c5e1611`：`cargo test -p zircon_runtime --lib template_asset_hot_paths --locked --jobs 1 --color never -- --test-threads=1`，target为`F:/cargo-targets/zircon-engine/performance-mvp-audit`，5 passed、0 failed、8380 filtered out；22m14s为共享target当前增量编译，测试执行0.00s。构建产生587条既有warning，本次5项门禁无新增失败。规模counter与产品验证仍pending，因此文档状态保持`static_complete_dynamic_pending`。

静态扫描命中346处clone、34处collect、6处sort、423处format、434处to_string、20处filesystem access、425处BTreeMap/BTreeSet使用和70处递归形态，未发现线程调度或后台编译/落盘边界。集中热点为prototype instancer、component contract、schema tree materialization、style application、resource/localization collection、cache fingerprint与surface hot reload。

## PERF-MVP-304/305：重复校验与authority subtree复制

`compile_with_cache`在miss时先完成全部preconditions，再调用公开`compile`重复shape/localization/component/binding全图校验；package compile同样重复。`validate_node_tree`还把每个节点的完整subtree clone进seen map，深链趋近O(N²)复制。

本轮五组止损中的前三组以RED→GREEN守卫抽出`compile_validated`供cache/package复用，并让authority seen map只保存borrowed id/node。更大的authoring问题仍在：node/parent/child操作每次DFS，style insert/replace/move复制全部stylesheets后重新解析所有selectors，已交接EditorUI05。

## PERF-MVP-306/312：prototype环境复制与第三份runtime tree

prototype显式frame stack避免Rust递归，但每个child frame复制node、mounts、token/param maps；slot读取clone整组expanded nodes。最终`UiTemplateTreeBuilder`又为每node构造祖先path String并clone所有template metadata，layout self/slot先合并复制TOML表，source/compiled/tree形成三份owned payload。

EditorUI05既有validated prototype DAG/canonical arena交接已扩展到`template/asset/compiler/prototype_instancer.rs`和tree builder；EditorUI02负责typed layout/slot contract。Slint在compiler lowering阶段建立`LoweringState`、component/item/property mappings并输出indexed item tree，说明Zircon也应让runtime surface持generation artifact handle，而不是逐层重物化authoring maps。

## PERF-MVP-307：样式按节点全规则扫描

`apply_styles_to_tree`为每node复制祖先StylePathEntry、物化path snapshot、扫描全部rules并排序命中项；每rule又复制同sheet token map。原实现还深clone每个matched rule和child slot owner全attributes。

本轮两组RED→GREEN止损将matched集合改为`&ParsedStyleRule`并借用owner attributes。EditorUI04既有handoff继续负责selector候选索引、interned path/state、sheet-level token owner与computed-style delta。UE Slate保留indexed invalidation widget list/range并只修复受影响父子索引；Slint把style defaults作为lowering pass写入binding，而不是在每次runtime traversal重新匹配全表。

## PERF-MVP-308/310/311：cache、schema与validator多轮全扫

compile key每次TOML序列化根文档和compiler注册的全部imports，再重走component/resource graph；hit返回完整compiled clone。persistent eviction递归读目录并反序列化每个候选文件。schema migrator为tree/flat source先parse Value判型，再parse header，再parse typed document。localization/resource/binding/component validation各自递归全树、反复format path和parse selector/expression；同一imported component被多次reference时反复建privacy index。

EditorUI05需产出single-parse source arena、declared dependency closure fingerprint、Arc compiled artifact、persistent asset manifest/LRU，以及共享node/path/selector/expression/component indexes。UE generated widget variables用于runtime快速字段查找，Fyrox `BuildContext`直接把builder结果插入retained pool并持共享style resource；两者都不支持在稳定命中后再次序列化或复制完整UI文档。

## PERF-MVP-309：hot reload与resource resolver放大

watch batch对每change分别cascade并复制String集合；compile eviction扫描entries。resource resolver对每URI分别retain全cache，cached placeholder为了恢复diagnostic indices扫描全部历史diagnostics，历史又无界增长。缺少精确asset-to-node ownership时template变化退化为surface root全dirty。

EditorUI05新handoff要求watch generation、dependency DAG、resolver reverse index、generation-bounded diagnostics和asset→surface/node ownership统一在一个authority中；1/100/10k changes/dependencies/cache/surfaces记录edge visits、cache scans、diagnostic bytes、dirty nodes和reload p95。

## 责任计划与验收

EditorUI04收到asset compiler style补充；EditorUI05三份既有v2 handoff已补template/asset路径，并新增authoring index与hot-reload/resolver两份failure；EditorUI02联动typed layout/slot。动态验收需覆盖1/100/10k nodes/rules/imports/instances/assets/changes，记录parse/validation/index builds、tree/rule/edge visits、clone/serialized bytes、filesystem calls、RSS与CPU p50/p95/p99。current-source Cargo、F4 asset preview/edit/hot-reload产品trace和像素完成前继续留`pending.md`，不得进入`review.md`。
