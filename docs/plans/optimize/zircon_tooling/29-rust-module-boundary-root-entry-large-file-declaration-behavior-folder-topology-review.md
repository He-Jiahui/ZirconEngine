---
related_code:
  - Cargo.toml
  - .codex/skills/zircon-project-skills/development-conventions.md
  - .codex/skills/zircon-project-skills/zr-module-boundary-discipline/references/binding-rs-anti-pattern.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/large_file_ownership.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_convention_gate.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_editor_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/editor_structure_audits/module_convention_boundary.py
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/global_budget.rs
  - examples/woc/native/crates/woc_protocol/src/command_payload.rs
  - examples/woc/native/apps/woc_client/src/input/intent.rs
  - zircon_runtime/src/core/framework/render/view_family.rs
  - zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/app.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/asset/importer/environment_ibl.rs
  - zircon_runtime/src/asset/project/paths.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/ui_hotspot.rs
  - zircon_runtime_interface/src/profiling.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/scene.rs
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/lib.rs
  - examples/woc/native/crates/woc_contract_codegen/src/lib.rs
  - zircon_runtime/src/graphics/visibility/static_index/mod.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/mod.rs
  - tools/cargo-zircon/src/plugin/scaffold/mod.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/binding.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/editing/binding.rs
tests:
  - zircon_runtime/src/tests/runtime_absorption/structure_convention.rs
  - tools/tests/test_editorui10_test_file_budget_contract.py
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_app/02-pbr-viewer-tool-runtime-evidence-renderdoc-review.md
  - docs/plans/optimize/zircon_app/04-woc-native-client-window-input-shell-ui-presentation-frame-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_runtime/09f1-environment-sky-ibl-reflection-probe-review.md
  - docs/plans/optimize/zircon_runtime/09h1-temporal-aa-velocity-history-upscaling-review.md
  - docs/plans/optimize/zircon_runtime/19-woc-command-protocol-payload-codec-admission-movement-outcome-runtime-review.md
  - docs/plans/optimize/zircon_runtime/25-filesystem-path-uri-vfs-mount-watch-sandbox-atomic-io-review.md
  - docs/plans/optimize/zircon_tooling/02-cargo-zircon-plugin-scaffold-manifest-validation-native-probe-review.md
  - docs/plans/optimize/zircon_tooling/13-repository-codex-skill-hook-structural-audit-governance-security-currentness-review.md
  - docs/plans/optimize/zircon_tooling/17-repository-content-source-set-ignore-generated-vendor-license-distribution-review.md
  - docs/plans/optimize/zircon_tooling/20-cargo-package-workspace-feature-dependency-target-graph-build-receipt-review.md
reference_engines:
  - dev/bevy/crates/bevy_app/src/lib.rs
  - dev/bevy/crates/bevy_render/src/lib.rs
  - dev/Fyrox/fyrox/src/lib.rs
  - dev/Fyrox/fyrox-impl/src/lib.rs
  - dev/godot/core/config/project_settings.h
  - dev/godot/core/config/project_settings.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraph.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 29 · Rust Module Boundary、Root Entry、Large File、Declaration/Behavior 与 Folder Topology 审查

## 1. 结论

Zircon已经有“按owner拆文件”的明确规范，也有Runtime、Editor和若干计划专属的结构检查器；因此当前问题不是完全没有模块治理。`GEN-S3`要求`lib.rs/mod.rs/main.rs`只保留模块声明、curated re-export与最小入口接线，`GEN-S4`要求生产Rust文件严格小于1000行，Runtime 15又有800行production budget测试。现有Runtime审计能够发现大文件并给出owner class，Editor审计能够发现超大测试、UI根文件、重复测试树与禁用命名。这些基础应保留并收敛，而不是另起一套互不兼容的行数脚本。

当前实现仍不是全仓、可阻断、理解源码语义的工程级Source Architecture Gate。本轮对`zircon_*`、`examples/`与`tools/`下17,263个tracked Rust路径做词法分层，排除明确测试目录/文件、fixture/bench与显式generated路径或`// @generated`首行后，得到11,958个manual production-like候选、1,315,926个物理行。该口径中有1,217个文件不少于300行、548个不少于500行、87个不少于800行、32个不少于900行、13个不少于1000行、5个不少于1100行。它是需要AST/Cargo校正的候选集，不是11,958项缺陷；但13个1000行热点都逐文件复核过，确有手写行为或内联测试边界，不能归因于纯generated/vendor输出。

现有Runtime审计本轮运行发现其中11个并正确返回`migration-debt-present`，但进程exit 0；其SourceSet只解析根`Cargo.toml`中名字以`zircon_`开头的member，漏掉`examples/woc`的1986行`command_payload.rs`、1418行`input/intent.rs`、独立`zircon_plugins`工作区和`tools/cargo-zircon`。Editor审计也返回`migration-debt-present`与30项债务，但同样exit 0；它只扫描`zircon_editor/src`，当前28项是测试文件预算、1项是UI根文件、1项是重复测试树，未检查根入口行为、声明/行为混装或目录扇出。局部报告红但命令绿，不能构成required admission gate。

源码结构风险也不等于“文件越短越快”。13个热点中9个含内联`mod tests`，物理行预算同时揭示测试owner未拆，但不能把测试行误报成热路径实现；`zircon_runtime/src/core/framework/render/mod.rs`虽有447行，却主要是44个模块声明与44个use，是反例，不能按行数机械切碎。相反，305行的Editor `editing/binding.rs`只有一个`EditorUiHost` impl却承载16个binding mutation命令，532行的profiling `mod.rs`同时挂9个子模块并保留capture/env/global recorder/counter行为，已经违反root/binding结构职责，即使没有碰到1000行。

本篇不重复App02/04、Runtime03/09f1/09h1/19/25中的功能正确性、协议、渲染、IBL、profiling或路径语义缺陷，也不接管Tooling13的通用required-exit治理、Tooling17的SourceSet/license、Tooling20的Cargo graph。Tooling29是Rust物理源码角色、root facade、declaration/behavior、folder topology、large-file ownership、例外与结构迁移receipt的canonical专项owner；功能owner仍负责拆分后的行为与性能。登记 **0项P0、48项P1和12项P2**。

## 2. 审查边界、方法与限制

| Evidence | 本轮结果 |
|---|---|
| E1 tracked Rust universe | `zircon_*`、`examples/`、`tools/`共17,263个tracked `.rs`路径 |
| E2 production-like lower bound | 排除测试/fixture/bench、generated路径/文件与精确`// @generated`首行后，11,958个候选、1,315,926物理行 |
| E3 size distribution | `>=300` 1,217；`>=500` 548；`>=800` 87；`>=900` 32；`>=1000` 13；`>=1100` 5 |
| E4 root/binding candidates | `lib.rs/main.rs/mod.rs/binding.rs`候选中24个不少于300行；逐读8个高风险/反例样本 |
| E5 flat folder lower bound | 163个目录有至少12个direct manual `.rs`；最大目录81、60、47、42个；该计数只用于路由人工审查 |
| E6 Runtime audit | `audit_runtime_structure.py --json`用时88.6s、exit 0；module/large-file gate均`migration-debt-present`，11个hotspot、3个owner-class debt、0 unclassified |
| E7 Editor audit | `audit_editor_structure.py --json`用时8.5s、exit 0；`migration-debt-present`，4814个production文件，0个`>1000` production、28个超800行测试、1个UI root violation、1个duplicate test tree |
| E8 Runtime 800-line guard | `global_budget.rs`只扫`zircon_runtime/src`并排除目录`tests`、`tests.rs/_tests.rs`；同口径静态复算4,870个文件中37个`>=800`、7个`>=1000`，内联测试仍计入 |
| E9 semantic spot reads | 逐读13个`>=1000`文件及contact-shadow、WOC codegen、static-index、profiling root、plugin scaffold、render facade、GPU binding、Editor binding |
| E10 reference review | 逐读Bevy/Fyrox root facade、Godot ProjectSettings、Unreal ModuleManager、Unity RenderGraph代表文件与相邻目录 |
| E11 dynamic scope | 运行两套只读Python结构审计；文档落盘后运行docs validator，仍为692项既有违规且Tooling29自身0项；未编译Rust、未重跑已知Editor/Hub/WOC/plugin阻断、未修改production/test/CI/manifest |
| Currentness | branch `main`，revision `ae2be3d865a937b9ed368bf965592045346c64e3`；55个frontmatter取证输入按path ordinal排序，每项编码为`path + LF + normalized UTF-8 content + LF`，fingerprint `9303e0a10a7346602c8420ba2d0f36e1b274f091aef36112c590602797916860`，36,026个normalized content LF、1,709,801 content bytes；`source_cubemap.rs`取证时已有相邻工作树修改，9篇输入优化报告处于本轮既存untracked状态，其余超限热点clean |

统计解释：

1. “production-like”来自路径、文件名和generated首行的词法分类，不理解Cargo target、宏展开、`include!`或条件编译；正式门必须消费Cargo metadata与Rust syntax tree。
2. 300/500行只是审查触发器，不是失败条件；失败依据是职责混装、root行为、声明与编解码耦合、目录不可路由或明确违反`GEN-S4`。
3. 内联测试仍是物理边界债务，但不能用总行数推断生产热路径复杂度；正式报告必须同时给physical、non-test syntax和test-owner三组数。
4. 平铺目录中的许多文件可能是刻意的一类型一文件；只有路径无法表达域、修改耦合持续升高或root fanout失控时才应重构。
5. 参考文件的长度不构成质量或性能排名；本篇只提取可观察的facade、声明/实现、cohesive subsystem与相邻文件边界。

## 3. 必须保留的工程基础

### 3.1 仓库已经定义不可退让的结构规则

`GEN-S3`与`GEN-S4`是MUST，不是建议。root entry只做结构接线，production file严格低于1000行；拆分按owner，不允许`part1/part2`式机械切块。后续应让自动门准确执行这些规则，不能因为当前baseline为红就把阈值上调或把MUST改成warning。

### 3.2 Runtime审计已经产出owner-aware finding

`large_file_ownership.py`不是只打印行数：它把hotspot分为runtime-framework-render、runtime-other、editor-retained-host、editor-ui、support/plugin等owner class，输出decision group、migration debt和risk。应复用其Finding结构，并替换粗粒度SourceSet与owner taxonomy。

### 3.3 Editor审计已经区分production与test budget

Editor结构审计分别给production/test阈值，要求test exemption有非空修复原因，还检查dead-code suppression、禁用命名、UI根owner与重复test tree。这证明按FileRole设不同policy可行；需要扩展到全仓并修正边界语义，而不是抹平成单一行数限制。

### 3.4 Runtime 15已经把物理文件预算写成可执行测试

`runtime_15_no_oversized_production_files`递归扫描、排序并列出所有超限路径，失败信息可定位。它适合作为兼容入口，但扫描逻辑应调用统一SourceUnitClassifier，避免测试、审计脚本和文档各自维护不同的“production”定义。

### 3.5 大量目录已经体现细粒度owner

Runtime与Editor中已有许多folder-backed split、child-owner测试和thin facade，说明迁移不需要推翻现有布局。应从13个明确违规和高耦合root开始，保留已经单一职责的小文件，不为追求平均行数制造路径噪声。

## 4. 已确认的结构断点

### 4.1 三套“结构真相”口径不一致

Runtime Python审计使用1000行且扫描根workspace中`zircon_`成员；Editor Python审计使用production `>1000`、test `>800`且只扫Editor；Runtime Rust测试使用`>=800`且只扫`zircon_runtime/src`。对`tests.rs/_tests.rs`、内联`#[cfg(test)]`、generated、fixture、examples、plugin workspace和tool target的分类各不相同。消费者无法回答“当前全仓到底有几个production违规”而不先选一个脚本。

### 4.2 finding状态没有绑定进程失败

本轮两个入口都在JSON中明确输出`migration-debt-present`，却返回exit 0。CI或人工脚本若只看进程状态，会把红报告当成功。Tooling13拥有通用required-exit修复，本篇要求结构FindingSet声明severity、baseline与admission policy并被该执行器消费。

### 4.3 root entry检查停留在路径/行数，未读语法职责

当前门不能识别`lib.rs/mod.rs/main.rs`中的函数体、impl、文件IO、环境解析、GPU创建或业务路由。contact-shadow `lib.rs`在测试前已经同时拥有descriptor、resource contract、WGPU pipeline/executor和validation；WOC codegen `lib.rs`拥有DTO/catalog/source identity/JSON读取/validation/hash；profiling `mod.rs`保留25个以上行为函数。它们不是curated facade。

### 4.4 declaration与behavior仍频繁共居

`zircon_runtime_interface/src/profiling.rs`在一个公开边界中放23个struct/enum/trait家族及其配置、retention、span/counter、UI hotspot、control request/response、runtime input/device和asset reload语义。WOC `command_payload.rs`把约49个payload/catalog/point声明与各自encode/decode/validation放进1986行。修改一个wire family会重开整个协议表面，难以做按域兼容、fuzz和owner review。

### 4.5 大文件报告没有区分实现与内联测试

13个1000行文件中，9个存在内联`mod tests`：例如`view_family.rs`测试模块从921行开始，IBL staging从716行、entry runtime从639行、render pipeline从542行、interface profiling从680行开始。它们仍违反物理预算，也说明test owner没有拆；但报告若只给1381/1324/1111，会误导读者把全部行都视作production behavior。

### 4.6 平铺目录使路径无法表达子域

`paint_template_nodes`有81个direct Rust文件，Editor retained-host `app`有60个、`ui/host`有47个、workbench template bridge有42个；Runtime的surface input、scene world、runtime plugin catalog各33个。它们不自动等于缺陷，但名称前缀代替目录层级后，新增修改容易反复触碰同一`mod.rs`/host context，reviewer也不能从路径判断paint node、state projection、command routing或platform adapter owner。

### 4.7 generated与handwritten协议没有统一SchemaSource边界

WOC已有带`// @generated`头的`generated.rs`与`generated_command_payloads.rs`，也有`woc_contract_codegen`；但手写`command_payload.rs`继续持有大批wire code、length/range验证和转换。现有功能报告已经记录协议数量与类型漂移，本篇只补结构结论：schema declaration、generated leaf、handwritten semantic adapter、codec test corpus必须有唯一authority和目录边界。

### 4.8 “大文件重构”没有事务与性能护栏

当前owner decision多为自然语言。没有机器可读的before symbols、public re-export、callsite set、allocation profile、compile unit、test owner、after paths与rollback receipt。若直接拆文件，可能只移动文本、制造跨模块private访问和clone，甚至让渲染热路径更慢；结构改善不能靠文件数量自证成功。

## 5. P0：无新增独立项

本轮没有发现需要另立P0 owner的结构问题。required finding返回exit 0的通用门禁缺陷由Tooling13拥有；WOC协议、render/IBL、profiling、path与app功能正确性由既有App/Runtime报告拥有。这里登记可阻断后续扩张的P1结构债务与重构合同，避免把同一根因重复计为多个P0。

## 6. P1：SourceSet、Classifier 与 Enforcement

| ID | 当前差距 | 工程级重构要求 |
|---|---|---|
| RUST-MODULE-P1-001 | 没有全仓canonical `RustSourceSet` | 从Cargo metadata、显式auxiliary workspace/example/tool registry和Git SourceSet生成稳定清单 |
| RUST-MODULE-P1-002 | Runtime审计用regex读取根members并只保留`zircon_`前缀 | 解析workspace/exclude/default-members、nested workspace与target，禁止用名称猜scope |
| RUST-MODULE-P1-003 | production/test/generated/fixture分类分散 | 建立versioned `SourceUnitClassifier`，每个路径输出role、reason、package、target与cfg |
| RUST-MODULE-P1-004 | 800、`>1000`、`>=1000`三种预算并存 | policy registry按FileRole/owner定义阈值，`GEN-S4`统一为production physical `<1000` |
| RUST-MODULE-P1-005 | 内联测试计入总量却不可单独观察 | syntax tree同时输出physical、production syntax、test syntax与generated expansion统计 |
| RUST-MODULE-P1-006 | `migration-debt-present`仍exit 0 | 通过Tooling13 required runner把新增/非基线finding映射为非零退出与typed result |
| RUST-MODULE-P1-007 | 当前JSON没有完整scan manifest | 报告source revision、dirty digest、tool/policy version、included/excluded paths与分类理由 |
| RUST-MODULE-P1-008 | 没有结构例外生命周期 | `BoundaryWaiver`必须有FindingId、owner、cohesion理由、scope、expiry与移除gate |

## 7. P1：Root Entry 与 Binding Boundary

| ID | 当前差距 | 工程级重构要求 |
|---|---|---|
| RUST-MODULE-P1-009 | root规则不检查函数体/impl/副作用 | 用Rust parser识别root中的behavior，允许mod/use/re-export与声明过的最小delegation |
| RUST-MODULE-P1-010 | plugin `lib.rs`常兼任descriptor与完整runtime | descriptor/registration留root，pipeline/resource/validation/executor进入命名子域 |
| RUST-MODULE-P1-011 | Runtime `mod.rs`仍保留capture/index/registration行为 | root只装配children；mutation/query/parse/recovery各自归owner文件 |
| RUST-MODULE-P1-012 | `binding.rs`可成为命令行为仓 | binding root只描述contract与路由，16个Editor mutation按selection/event/route/payload拆分 |
| RUST-MODULE-P1-013 | root fanout没有架构预算 | 记录direct child count、domain count与curated export；高fanout先按子域分层而非改短名字 |
| RUST-MODULE-P1-014 | structural root与mixed root被同一行数规则处理 | root policy看语法角色；447行纯facade可审查豁免，305行多行为binding必须报告 |
| RUST-MODULE-P1-015 | 最小入口接线没有可验证定义 | delegation只允许构建一个owner并转交；env/file/GPU/network/business orchestration归child owner |
| RUST-MODULE-P1-016 | root拆分缺少public surface事务 | 先冻结symbol/re-export/callsite图，同批迁移并删除旧行为路径，禁止兼容shim长期存活 |

## 8. P1：Declaration、Codec 与 Behavior Family

| ID | 当前差距 | 工程级重构要求 |
|---|---|---|
| RUST-MODULE-P1-017 | declaration文件可无限吸收parse/validate/route | 每个公开concept有声明owner；非平凡codec、mapping、mutation、query移到behavior family |
| RUST-MODULE-P1-018 | profiling interface混合多个协议域 | 拆capture、frame/span、counter、hotspot、control、input/device、asset reload contract并curated export |
| RUST-MODULE-P1-019 | WOC payload声明与49组codec同文件 | 按command domain生成wire declaration/codec，semantic validation与admission由手写adapter拥有 |
| RUST-MODULE-P1-020 | 151项client intent映射集中在一个match | 按movement/combat/social/instance/admin等域分路由表，并从canonical command schema生成覆盖检查 |
| RUST-MODULE-P1-021 | generated ID与手写payload可能形成双真源 | 定义`ProtocolSchemaSource`，generator输出leaf，手写层只能引用stable IDs与semantic rules |
| RUST-MODULE-P1-022 | IBL staging混合store/codec/journal/recovery/report | 拆read/write、snapshot retry、bundle journal/recovery、validation与receipt owner |
| RUST-MODULE-P1-023 | project paths混合layout/identity/OS canonicalization | project layout、URI/path identity、Windows adapter与transaction error分别归属 |
| RUST-MODULE-P1-024 | declaration移动没有ABI/API检查 | 输出before/after symbol map、visibility/re-export diff、serde/wire schema diff与downstream compile set |

## 9. P1：Large Production File Decomposition

| ID | 当前差距 | 工程级重构要求 |
|---|---|---|
| RUST-MODULE-P1-025 | 当前有13个manual production-like文件`>=1000` | 冻结清单与owner，不允许新增；按本篇target family逐个降至`<1000`并保留行为 |
| RUST-MODULE-P1-026 | 当前有87个文件`>=800`但没有统一预警 | 800作为warning/review trigger，1000作为MUST failure；趋势按owner与generation发布 |
| RUST-MODULE-P1-027 | 9个超限文件内联大型test module | 测试移入folder-backed test owner，保留private seam所需的最小`cfg(test)` adapter |
| RUST-MODULE-P1-028 | `view_family.rs`混合resolution policy/controller/history/pipeline | 拆dynamic resolution、resolution plan、target/history、phase pipeline与viewport math |
| RUST-MODULE-P1-029 | render-frame文件混合submit/target/capture/timing/report | 按frame preparation、graph execution、presentation/capture与GPU timing receipt拆分 |
| RUST-MODULE-P1-030 | PBR viewer app混合window/surface/load/render/input/evidence | 保留ApplicationHandler壳，拆lifecycle、async scene load、present/retry、input与capture evidence |
| RUST-MODULE-P1-031 | PBR viewer scene混合project/assets/IBL/prewarm/frame timing | 拆scene acquisition、environment preparation、prewarm、frame owner与startup evidence |
| RUST-MODULE-P1-032 | cubemap/IBL文件混合sampling/build/persist/recovery | 分离纯sampling math、GPU/CPU build、artifact codec/store与transaction recovery，逐层做golden/perf gate |

## 10. P1：Folder Topology、Generated 与 Test Ownership

| ID | 当前差距 | 工程级重构要求 |
|---|---|---|
| RUST-MODULE-P1-033 | 163个目录至少12个direct manual Rust文件 | 用domain/fanout/change-coupling审查，不设机械文件数失败；高风险目录给出subsystem map |
| RUST-MODULE-P1-034 | paint-template目录81个direct文件 | 按node family、style resolution、geometry/text/image与host projection建立子目录 |
| RUST-MODULE-P1-035 | retained-host app目录60个direct文件 | 按lifecycle、command、projection、persistence、window/platform adapter划分owner |
| RUST-MODULE-P1-036 | Editor `ui/host`有47个direct文件 | 把asset session、workbench/window、dialog/menu、runtime bridge暴露为可导航子系统 |
| RUST-MODULE-P1-037 | WOC protocol目录26个direct文件且payload独占1986行 | 以schema family组织command/event/snapshot/error，生成leaf与手写adapter物理分区 |
| RUST-MODULE-P1-038 | generated检测依赖路径/字符串启发 | generated artifact必须有精确header、generator ID、schema digest、禁止行为token与regen test |
| RUST-MODULE-P1-039 | generated leaf与手写support可同名同层 | generated目录只接纳leaf data/table；semantic adapter、validation与migration进入稳定手写owner |
| RUST-MODULE-P1-040 | test tree与production tree映射不统一 | `TestOwnerId`反向绑定production owner；大测试按scenario/contract/fixture拆分且不复制实现逻辑 |

## 11. P1：Migration、Validation 与 Performance Safety

| ID | 当前差距 | 工程级重构要求 |
|---|---|---|
| RUST-MODULE-P1-041 | 可按行数机械切成`partN` | splitter必须提交DomainOwner图和moved-symbol清单；禁止数字分片与无语义`misc/utils` |
| RUST-MODULE-P1-042 | private访问可能诱导新增clone/Arc/锁 | 拆分前后比较allocation、clone、lock、channel与hot-path call graph，新增成本需显式批准 |
| RUST-MODULE-P1-043 | 每个split没有分层验证模板 | 先static symbol/re-export，再focused unit/contract，再package compile，最后产品/性能证据 |
| RUST-MODULE-P1-044 | 结构变短可被误宣称性能提升 | StructureReceipt只证明owner/size；性能必须引用独立benchmark/frame capture与统计置信度 |
| RUST-MODULE-P1-045 | 没有change-coupling基线 | 从Git历史生成co-change矩阵，辅助识别混域文件；历史信号不能覆盖当前架构owner决定 |
| RUST-MODULE-P1-046 | hotspot清单没有source currentness | 每次审计绑定revision、dirty digest、policy/tool version；输入变化自动使receipt stale |
| RUST-MODULE-P1-047 | 没有全仓机器artifact | 产出deterministic JSON/SARIF，包含SourceUnit、role、owner、finding、waiver与suggested boundary |
| RUST-MODULE-P1-048 | 可能照抄参考工程文件尺寸 | 只复用facade/cohesion/decl-impl原则；Zircon阈值由自身MUST和compile/runtime数据决定 |

## 12. P2：后续增强

| ID | 增强项 | 价值 |
|---|---|---|
| RUST-MODULE-P2-001 | IDE显示SourceUnit role/owner | 新代码落地前即可发现错误目录 |
| RUST-MODULE-P2-002 | PR自动附module graph diff | reviewer可见新增边、fanout与root behavior |
| RUST-MODULE-P2-003 | 生成folder topology热图 | 辅助发现长期平铺与单owner过载 |
| RUST-MODULE-P2-004 | 生成public re-export地图 | 便于控制facade与内部路径泄漏 |
| RUST-MODULE-P2-005 | co-change趋势看板 | 发现反复一起修改但分属多域的文件 |
| RUST-MODULE-P2-006 | compile-unit timing按模块归因 | 防止拆分造成编译时间退化 |
| RUST-MODULE-P2-007 | macro expansion/source map审计 | 避免generated或宏展开行为逃逸预算 |
| RUST-MODULE-P2-008 | test-to-owner coverage图 | 识别巨大测试树与无人负责的public contract |
| RUST-MODULE-P2-009 | waiver expiry提醒与自动升级 | 防止结构例外永久化 |
| RUST-MODULE-P2-010 | safe split codemod dry-run | 只生成候选move/re-export diff，不自动决定架构 |
| RUST-MODULE-P2-011 | docs.rs模块树与内部owner图联动 | 公共消费者看到稳定API，内部细分不外泄 |
| RUST-MODULE-P2-012 | historical architecture playback | 对比里程碑间模块边界，而非只看当前快照 |

## 13. 13个当前`>=1000`候选的owner级处置

| Lines | Path | 结构读数 | 目标边界/既有功能owner |
|---:|---|---|---|
| 1986 | `examples/woc/native/crates/woc_protocol/src/command_payload.rs` | 约49组声明+codec/validation，无内联test module | command domain schema/generated codec/semantic validator；功能路由Runtime19、App03/04 |
| 1418 | `examples/woc/native/apps/woc_client/src/input/intent.rs` | 151项intent及集中mapper，无内联test module | movement/combat/social/instance等intent router；功能路由App04 |
| 1381 | `zircon_runtime/src/core/framework/render/view_family.rs` | production到920行，test从921 | dynamic resolution/plan/history/phase pipeline；功能路由Runtime09h1 |
| 1324 | `zircon_runtime/src/asset/artifact/ibl_source_cubemap_staging.rs` | test从716；store/codec/journal/recovery混合 | artifact read/write/recovery/validation/receipt；功能路由Runtime09f1与Editor asset pipeline |
| 1111 | `zircon_app/src/entry/entry_runner/runtime.rs` | test从639；env/project/session/frame/teardown混合 | startup config/runtime session/frame exit/teardown；功能路由App01 |
| 1086 | `zircon_app/src/bin/zircon_shader_pbr_viewer/app.rs` | 约1083行app行为，仅尾部test cfg | lifecycle/load/present/input/evidence；功能路由App02 |
| 1086 | `.../render_frame_with_pipeline.rs` | test从542且test cfg散布；两组renderer impl | graph execution/present/capture/timing；功能路由render reports |
| 1085 | `zircon_runtime/src/asset/importer/environment_ibl.rs` | test从731；restore/stage/build/write混合 | restore/stage/build/persist；功能路由Runtime09f1 |
| 1070 | `zircon_runtime/src/asset/project/paths.rs` | test从556；layout/canonicalization/Windows混合 | layout/path identity/platform adapter；功能路由Runtime25 |
| 1053 | `zircon_runtime/src/core/runtime/diagnostics/profiling/ui_hotspot.rs` | test从668；accumulator/report/alert rules混合 | metric collection/report/rule registry；功能路由Runtime03 |
| 1017 | `zircon_runtime_interface/src/profiling.rs` | test从680；23个contract family | interface contract子域与curated facade；功能路由Runtime03/Interface |
| 1010 | `zircon_runtime/src/core/framework/render/environment/source_cubemap.rs` | 约1008行sampling/build math，尾部test cfg | quality/timing/mip/sampling/SH/build；功能路由Runtime09f1；取证时dirty |
| 1002 | `zircon_app/src/bin/zircon_shader_pbr_viewer/scene.rs` | test从695；project/assets/IBL/prewarm/timing混合 | acquisition/environment/prewarm/frame/evidence；功能路由App02 |

`render_frame_with_pipeline.rs`的缩写路径只用于表格可读性，canonical path已列入frontmatter。任何拆分先重查相邻session修改；特别是`source_cubemap.rs`当前dirty，不允许按本报告快照直接覆盖。

## 14. Root/Binding代表样本

| Path | 读数 | 结论 |
|---|---|---|
| contact-shadow `runtime/src/lib.rs` | 757行，4 mod、12 use、5 declaration、4 impl，test从470 | root持有descriptor、resource、pipeline/executor与validation，必须下沉行为 |
| WOC codegen `src/lib.rs` | 699行，3 mod、10 use、18 declaration、1 impl，test从679 | catalog/identity/JSON/validation/hash应分域，root只暴露codegen API |
| static-index `mod.rs` | 620行，1 child、5 declaration、2 impl，test从436 | declaration、update/query与ray traversal混装 |
| profiling `mod.rs` | 532行，9 child、14 use、33 lexical fn，test从305 | child已存在但capture/env/global control仍滞留root |
| cargo-zircon scaffold `mod.rs` | 459行，2 child、4 declaration、9 impl，test从441 | transaction、TOML、marker insertion、rollback需分owner |
| render `mod.rs` | 447行，44 mod、44 use、1 trait、近零行为 | 主要是structural facade；应审查fanout/trait归属，不能机械按行数拆 |
| GPU scene `binding.rs` | 370行，单一GPU binding域，test从236 | 当前较cohesive；先拆test，只有descriptor/build families继续耦合时再拆行为 |
| Editor editing `binding.rs` | 305行，一个impl、16个public mutation | 虽未超限，binding root已是selection/event/route/payload命令仓，应变thin route |

这张表说明行数只负责触发阅读，最终Finding必须携带语法职责与owner证据。24个`>=300`根/绑定候选不能一键判红；正式AST审计应输出`structural`、`cohesive exception`、`mixed behavior`或`needs domain review`。

## 15. 目标架构与核心合同

```text
Cargo metadata + auxiliary workspace registry + Git SourceSet
    -> WorkspaceSourceSetManifest
    -> SourceUnitClassifier
         {RootFacade, Declaration, Behavior, GeneratedLeaf,
          TestOwner, Fixture, BuildTool}
    -> Rust syntax / module / public-symbol graph
    -> ModuleBoundaryGraph + DomainOwnerRegistry
    -> policy evaluation + BoundaryWaiverRegistry
    -> BoundaryFindingSet (JSON/SARIF, required exit)
    -> RefactorTransaction
    -> StructureReceipt + BehaviorReceipt + PerformanceReceipt
```

### 15.1 `SourceUnitId`

不能只用路径。至少包含repository source identity、Cargo package/target、Rust module path、cfg set、file role与content digest；rename通过transaction映射旧新identity。

### 15.2 `FileRole`

`RootFacade`只允许声明/curated export/minimal delegation；`Declaration`允许数据与trivial constructor，不吸收复杂codec/router；`Behavior`声明明确domain owner；`GeneratedLeaf`必须可重建且无架构行为；`TestOwner/Fixture`有独立预算和production映射。

### 15.3 `ModuleBoundaryGraph`

节点是SourceUnit/DomainOwner/PublicSymbol，边至少区分declare、re-export、call、mutate、construct、register、generate与test。root行为、跨域私有访问、facade fanout和循环依赖从图上判定，不能靠文件名包含`mod`或`binding`。

### 15.4 `BoundaryWaiver`

大型cohesive subsystem可以例外，例如经证明单一职责且拆分会损害局部性的实现；waiver必须说明为什么cohesive、谁批准、阈值、性能/编译证据、expiry。Unreal/Unity存在大型cohesive文件不意味着Zircon可以给任意聚合文件永久豁免。

### 15.5 `RefactorTransaction`

每次拆分记录before/after paths、moved symbols、public surface diff、callsite migrations、test moves、generated authority与rollback point。同批删除旧行为路径；不保留`legacy/compat/shim`，不使用`part1/part2`。

## 16. 参考实现的结构差异

### 16.1 Bevy

本地`bevy_app/src/lib.rs`71行，由13个module与12个public re-export构成，没有本地声明、函数或impl，是清晰facade。`bevy_render/src/lib.rs`592行则有30个module并保留RenderPlugin/RenderSystems等初始化编排，说明复杂subsystem root可以有经命名的owner行为，但应保持域单一且相邻模块明确。Zircon应学习facade与subsystem root的区别，不照抄文件长度。

### 16.2 Fyrox

`fyrox/src/lib.rs`36行，主要把`fyrox_impl`与脚本表面转出；`fyrox-impl/src/lib.rs`114行，以9个module和12个public re-export暴露实现域。它展示了消费者facade与内部owner树分离。Zircon多个plugin `lib.rs`仍把注册表面和完整实现堆在一起。

### 16.3 Godot

ProjectSettings以295行header声明contract、1900行cpp承载实现，并由相邻平台/配置设施协作。该文件仍大，但声明与实现、公共API与内部算法有语言级边界。Zircon的profiling interface和WOC payload目前把声明、codec、validation混在同一Rust source unit。

### 16.4 Unreal

ModuleManager是大型cohesive subsystem：1189行public header与2249行private implementation，不符合Zircon自己的1000行MUST，却有明确Public/Private、module manager domain和相邻实现边界。可借鉴的是可见性和owner，不是取消Zircon门禁，也不能据此推断性能领先。

### 16.5 Unity Graphics

RenderGraph.cs为1740行大型核心类，但位于完整RenderGraph包目录，周围有builder、resource、compiler、native pass等owner。它说明某些cohesive algorithm可通过waiver保留局部性；同时也证明单看行数无法评价架构。Zircon需先证明cohesion与性能，再批准例外。

## 17. 分层实施顺序

### M0 · 冻结清单与统一口径

保存本轮13个`>=1000`、87个`>=800`和24个root/binding候选；定义Cargo-resolved SourceSet与FileRole。现有红baseline只允许下降，新违规required fail。

### M1 · 统一执行器

Runtime/Editor/Rust test改为消费同一classifier与policy；JSON/SARIF包含scan manifest。通过Tooling13 runner让required finding非零退出，保留focused compatibility入口。

### M2 · 先拆测试owner

对9个内联大test module迁到folder-backed tests，验证private seam与behavior不变；这一步不宣称production架构或性能已经改善。

### M3 · Root与Declaration边界

优先处理contact-shadow、codegen、profiling、static index、scaffold与Editor binding；冻结public symbols/re-export，按domain hard cut。

### M4 · 13个热点逐域拆分

从WOC协议/intent、view family、IBL/artifact、entry runtime、PBR viewer、render execution、paths与profiling逐个事务迁移。每一批由其功能报告owner验证，不做全仓同时搬家。

### M5 · Folder topology

对81/60/47/42等高fanout目录结合co-change和domain map分层；cohesive flat families可保留，必须有review decision。

### M6 · 性能与持续治理

结构receipt与benchmark/frame capture分开发布；监控新增root behavior、budget趋势、waiver expiry、compile time与hot-path allocation。

## 18. 验收门

| Gate | 验收条件 |
|---|---|
| G01 | WorkspaceSourceSetManifest覆盖根workspace、nested plugin workspace、examples与tools |
| G02 | 每个SourceUnit都有package/target/module/file role与分类reason |
| G03 | generated leaf由精确header/generator/schema digest识别，不靠目录猜测 |
| G04 | test、fixture、bench、generated与production分类有fixture tests |
| G05 | production physical `<1000`语义与GEN-S4完全一致，恰好1000也失败 |
| G06 | 800 warning与1000 failure按policy registry单源，不在三处硬编码 |
| G07 | 内联test行与production syntax行分别报告，physical总量仍可审计 |
| G08 | 13个当前`>=1000`有稳定FindingId、owner与target boundary |
| G09 | 新增非基线大文件使required command非零退出 |
| G10 | Runtime/Editor现有入口与统一FindingSet计数一致 |
| G11 | required runner不再出现`migration-debt-present`且exit 0 |
| G12 | root AST policy能识别函数体、impl、副作用与跨域orchestration |
| G13 | structural facade与mixed behavior root可被稳定区分 |
| G14 | `lib.rs/mod.rs/main.rs`只保留允许的结构与最小delegation |
| G15 | `binding.rs`只保留contract/route，复杂mutation/codec按域下沉 |
| G16 | contact-shadow/codegen/profiling/static-index/scaffold root均完成owner split |
| G17 | public re-export与downstream path在hard cut后有symbol diff证明 |
| G18 | 不出现`partN`、`misc`、`utils`或迁移语义shim |
| G19 | declaration文件不再持有非平凡parse/codec/route/mutation |
| G20 | profiling public contract按子域拆分且facade保持curated |
| G21 | WOC schema/generated/semantic adapter只有一个authority chain |
| G22 | 151项intent mapper按domain路由且exhaustiveness由schema gate覆盖 |
| G23 | IBL staging的store/codec/journal/recovery/report边界可独立测试 |
| G24 | project path的layout/identity/platform adapter可独立替换与验证 |
| G25 | 9个大型内联test module迁入可导航TestOwner |
| G26 | TestOwner反向绑定production owner且不复制实现逻辑 |
| G27 | `view_family`拆分保留resolution/history/phase行为golden |
| G28 | render-frame拆分保留submit/present/capture/timing顺序与错误语义 |
| G29 | PBR viewer app/scene拆分保留真实窗口、加载、截图与timing证据 |
| G30 | cubemap/IBL拆分保留artifact digest、sampling golden与GPU/CPU parity |
| G31 | 高fanout目录有domain map、review decision与curated facade |
| G32 | 每个RefactorTransaction有before/after paths、symbols、callsites和rollback |
| G33 | 每次split通过static、focused、package与产品级分层验证 |
| G34 | split没有未批准的allocation/clone/lock/channel成本增长 |
| G35 | StructureReceipt不冒充performance或feature-complete证据 |
| G36 | 同revision与policy重跑FindingSet deterministic，输入变化自动stale |

## 19. 与既有报告的责任边界

| 依赖报告 | 本篇消费 | 仍由原报告拥有 |
|---|---|---|
| Tooling13 | required runner、FindingSet、非零exit政策 | 通用skill/hook/audit治理与权限安全 |
| Tooling17 | Git SourceSet、generated/vendor/evidence分类 | 分发、license、ignore与archive source truth |
| Tooling20 | Cargo package/target/feature graph | workspace admission与依赖/target构建语义 |
| Tooling02 | cargo-zircon scaffold功能合同 | plugin模板、manifest验证与native probe正确性 |
| App02 | PBR viewer真实产品/RenderDoc证据 | viewer功能、画面、交互、capture与性能结论 |
| App04 | WOC native client/input/presentation | intent功能映射、window/input/UI与产品闭环 |
| Runtime03 | profiling/config/diagnostics语义 | capture、counter、hotspot、retention与运营能力 |
| Runtime09f1 | environment/IBL/reflection probe | sampling、bake、artifact与视觉/性能正确性 |
| Runtime09h1 | temporal/view-family渲染能力 | dynamic resolution、history、upscaler与画质 |
| Runtime19 | WOC command protocol/codec/admission | wire compatibility、validation、fuzz与命令结果 |
| Runtime25 | path/VFS/watch/atomic IO | 路径身份、sandbox、watch与transaction语义 |

边界规则：

1. Tooling29拥有“代码放在哪里、root是否含行为、文件/目录如何分类、拆分如何留receipt”，不重新定义各功能域的正确行为。
2. 大文件若同时有功能P0/P1，本篇只登记结构Finding；功能报告仍是修复优先级与验收authority。
3. Tooling13修复通用exit/runner，本篇提供Rust结构policy与当前baseline，不能另造第二个CI truth。
4. Tooling17决定source/generated/vendor集合的仓库级身份，本篇消费后投影Rust FileRole，不复制license/distribution逻辑。
5. 任一结构拆分必须在功能owner可验证时进行；不能为了让行数门变绿而破坏ABI、序列化、渲染顺序或性能局部性。

## 20. 本轮产出与限制

本轮只新增审查文档并更新优化索引，不修改Rust、Cargo、测试、CI或现有审计器。Runtime/Editor审计只读执行结果如E6/E7；Runtime 800行测试未编译执行，E8是对其源码口径的静态等价复算。`check_conventions.py --only docs --json`返回exit 1、2,651篇文档、78,405条路径、692项违规、242篇受影响，其中Tooling29为0项；这是既有docs红baseline未恶化，不是全仓文档门通过。已知Editor、Hub、WOC、plugin和其它动态阻断保持原状态。

在G01至G36完成前，Zircon拥有的是多套局部结构检查、许多已拆分的小owner和一批明确债务，不是覆盖全workspace、能阻断新增、理解root/declaration/behavior/generated/test语义的工程级模块架构门。文件数量、平均长度或一次“classified-and-clear”都不能证明功能完整、性能领先或达到Unreal级工程成熟度。
