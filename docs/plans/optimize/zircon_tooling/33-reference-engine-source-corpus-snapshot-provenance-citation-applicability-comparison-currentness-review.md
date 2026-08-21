---
related_code:
  - .gitignore
  - tools/check_conventions.py
  - .codex/skills/zircon-project-skills/zr-reference-engine-routing/SKILL.md
  - .codex/skills/zircon-project-skills/zr-reference-engine-routing/references/reference-engine-map.md
tests:
  - tools/tests/test_check_conventions.py
plan_sources:
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/coverage.md
  - docs/plans/optimize/zircon_tooling/07-performance-benchmark-profile-capture-symbol-crash-evidence-baseline-review.md
  - docs/plans/optimize/zircon_tooling/17-repository-content-source-set-ignore-generated-vendor-license-distribution-review.md
  - docs/plans/optimize/zircon_tooling/28-documentation-api-reference-plan-currentness-link-source-trace-knowledge-publication-review.md
  - docs/plans/optimize/zircon_tooling/32-hot-path-catalog-algorithmic-complexity-data-movement-batching-cache-locality-performance-governance-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Build/Build.version
  - dev/UnrealEngine/LICENSE.md
  - dev/UnrealEngine/Engine/Documentation/Builds/CppAPI-HTML.tgz
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManager.h
  - dev/bevy/Cargo.toml
  - dev/bevy/LICENSE-MIT
  - dev/bevy/LICENSE-APACHE
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/Fyrox/Cargo.toml
  - dev/Fyrox/fyrox/Cargo.toml
  - dev/Fyrox/LICENSE.md
  - dev/Fyrox/fyrox-impl/src/engine/executor.rs
  - dev/godot/version.py
  - dev/godot/LICENSE.txt
  - dev/godot/main/main.cpp
  - dev/Graphics/.gitattributes
  - dev/Graphics/LICENSE.md
  - dev/Graphics/Packages/com.unity.render-pipelines.core/package.json
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraph.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 33 · Reference Engine Source Corpus、Snapshot、Provenance、Citation、Applicability、Comparison 与 Currentness 审查

## 1. 结论

Zircon 的参考工程审查已经有相当扎实的离线基础。本报告写入前，`docs/plans/optimize` 七个分类目录中的 142 篇专项报告全部声明了 `reference_engines`；共 2,099 次引用、1,596 个唯一 locator，当前本机 0 missing。五个核心参考树的物理 checkout 合计 328,251 个文件、76,229,954,431 bytes；Bevy、Fyrox、Godot 和 Unity Graphics 都有可解析 Git HEAD，四个工作树当前均为 clean，前三者不是 shallow，Graphics 的 shallow commit 也通过 connectivity check。仓库级 reference-routing skill 已明确要求先读 Zircon owner、按约束选择 primary/secondary reference、禁止机械复制目录和 API。这些机制必须保留。

缺口不在“没有看别的引擎”，而在“看过什么、基于哪个代次、支持哪条差异主张”没有形成可复现合同。根 `.gitignore` 整体排除 `/dev/`，主 Git tree 对 `dev/` 的 tracked path 数是 0；`dev/` 根有 21 个目录和两个普通文件，却没有机器可读 corpus manifest。Unreal checkout 没有 `.git`，`Build.version` 虽写 `6.0.0`，但 `Changelist` 与 `CompatibleChangelist` 都是 0，只靠路径和版本文本无法唯一识别 57 GB 本地快照。Bevy/Fyrox/Godot 的 commit 可精确识别，但 tag describe 与产品内版本分别出现 `v0.16.0-rc.4-3096` 对 `0.20.0-dev`、`v1.0.1-145` 对 `2.0.0-rc.1`、`4.7-stable-953` 对 `4.8.0-dev`；这证明“最近 tag”“package version”和“source revision”必须分字段，不能择一冒充快照身份。Graphics 是 `rev-list --count=1` 的 shallow、detached/grafted commit，虽然有 `github` 远端，仍缺获取深度与对象可达性政策。

现有 142 篇报告引用 1,472 个文件和 124 个目录；文件字节共 275,849,543，路径加逐文件 digest 的当前聚合 SHA-256 为 `d1ba462cace4c62936a5dd411c1bd4af3d2afb9aaa6bb4d2e983de12bf5203ad`。其中单个 Unreal `CppAPI-HTML.tgz` 为 229,361,318 bytes。目录 locator 不能说明作者实际读了哪些文件，归档 locator 也不能与 source symbol 使用同一种证据语义。289 个唯一路径在多篇报告复用，共形成 792 次引用；复用本身合理，但同一路径可能分别被用于 plugin lifecycle、process host、性能或 editor workflow，不能让“路径出现过”自动给所有差异结论背书。WOC native client 报告还留下三处大小写为 `dev/fyrox/...` 的路径；Windows 当前可解析，Linux/case-sensitive corpus resolver 不成立。

元数据层面，142 篇报告中 0 篇声明 `reference_snapshot_id`、`reference_corpus_id`、reference revision/commit/version、structured citation 或 comparison claim。当前 `check_conventions.py` 只将 `implementation_files`、`related_code` 和 `tests` 纳入路径门，`plan_sources` 与 `reference_engines` 都不在执行合同中；本轮 0 missing 来自专门扫描，不是 required CI 证明。Tooling17 只拥有 `ContentClass::Reference`、source archive 与许可/notice；Tooling28 只拥有通用文档 SourceGraph/currentness；Tooling07 只拥有 benchmark/performance comparison receipt。没有报告拥有 `ReferenceCorpusManifest -> ReferenceSnapshotReceipt -> CitationSet -> ComparisonClaim -> Applicability/TranslationDecision -> ReviewReceipt -> Currentness` 的完整链。

本篇登记 **0 项 P0、48 项 P1、12 项 P2和 40 个验收门**。没有新增 P0 是因为当前问题会削弱 review 可复现性和未来工程决策，却不直接证明某个 shipping runtime 正在产生内存破坏、数据损失或安全越权；各领域已经确认的产品 P0 仍由原报告拥有。本篇只拥有参考 corpus、snapshot、citation、claim、applicability、translation 和 drift review 控制面，不接管领域架构、Tooling07 性能测量、Tooling17 分发许可、Tooling28 文档发布，也不要求跟踪、复制或重新分发受限制的 Unreal 源码。

## 2. 审查边界、口径与限制

### 2.1 当前物理账本

| Evidence | 本轮结果 | 可支持结论 |
|---|---:|---|
| E1 `dev/` 根 | 21 个目录、2 个普通文件；根无 corpus manifest | 参考资料存在，但 membership 与用途由本机目录约定决定 |
| E2 主仓 SourceSet | `.gitignore:9` 为 `/dev/`；`git ls-files -- dev` 为 0 | 主 commit 不能单独恢复本轮 reference universe |
| E3 五个核心 checkout | 328,251 文件、76,229,954,431 bytes | 这是含 Git object/生成归档的物理 footprint，不等于精确 source corpus |
| E4 报告引用 | 2,099 entries、1,596 unique locator、1,472 files、124 directories | 路径覆盖广，但 locator 粒度混合 |
| E5 引用文件内容 | 275,849,543 bytes；聚合 hash `d1ba462c...203ad` | 当前文件集合可冻结；目录和过去 review 代次仍不可复现 |
| E6 family entries | Unreal 806、Godot 446、Bevy 326、Graphics 262、Fyrox 255、Recast vendor 3、Slint 1 | 核心参考广泛出现；数量不能证明主张质量或必要性 |
| E7 report family breadth | 102 篇含五个核心 family、25 篇含四个、10 篇含三个、2 篇含两个、3 篇含一个 | routing 已广泛执行；不要求每篇机械凑齐五家 |
| E8 structured snapshot fields | 8 个候选字段全部 0/142 | 报告不能机器绑定 reference generation |
| E9 required validator | 只检查 implementation/related/tests；不检查 plan/reference fields | 当前 reference 0 missing 不是 required gate 结果 |
| E10 dynamic scope | 只做 read-only 文件/Git/hash/metadata 审计 | 没有构建参考引擎，也没有重跑已知 Zircon 动态阻断 |

五个核心 checkout 的物理 footprint：

| Family | Files | Bytes | 当前 snapshot identity |
|---|---:|---:|---|
| Unreal | 246,991 | 57,270,168,623 | 无 `.git`；`Build.version=6.0.0/UE5`，CL=0，文件 hash 只能局部冻结 |
| Bevy | 2,986 | 266,820,737 | `fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`，main，非 shallow，clean |
| Fyrox | 995 | 424,125,332 | `8d815db36494f1badb347547dfc7094bf4fbbdf8`，master，非 shallow，clean |
| Godot | 14,146 | 1,670,330,383 | `8c7e6c5877a78e8e61ea4fd42673219a9091dca7`，master，非 shallow，clean |
| Unity Graphics | 63,133 | 16,598,509,356 | `a7e4c051d256a781ab362c64316b125a1e104694`，detached/grafted，shallow，clean |

统计解释必须保持克制：

1. 物理文件数和字节包括 `.git` object、LFS materialization、生成文档、测试资产和归档，不能当成“引擎源码行数”或架构复杂度排名。
2. 当前工作树 clean 只证明嵌套 Git 仓库相对其本地 HEAD 没有普通变更；不证明 remote 是 canonical upstream、commit 永久可获取、submodule/LFS 完整或 license 允许任何再分发用途。
3. `git describe` 选择可达 tag；当前三处 tag/package version 不一致是身份字段混用证据，不代表仓库损坏。
4. 引用 family 数量只说明报告 frontmatter 的广度，不证明正文逐项读过、理解正确、适用于当前 Zircon 或足以支持“优于 Unreal”。
5. 当前路径存在性是在 Windows case-insensitive checkout 上计算；三处 `dev/fyrox` 仍是可移植性缺陷。
6. 本篇不提供法律意见。只记录 Unreal EULA pointer、Bevy MIT/Apache 双文本、Fyrox/Godot MIT 文本与 Unity Companion License 声明需要进入 access/use policy。

### 2.2 Snapshot 身份事实

| Family | VCS / origin | source 内版本 | 当前风险 |
|---|---|---|---|
| Unreal | 本地树无 Git identity | `6.0.0`、branch `UE5`、CL 0 | 版本可读但快照不唯一；同版本不同提交无法区分 |
| Bevy | GitHub origin、完整本地 history | workspace `0.20.0-dev` | nearest tag 是旧 0.16 RC；tag 不能替代 commit/package version |
| Fyrox | Gitee personal mirror origin | `fyrox` package `2.0.0-rc.1` | canonical upstream relation、mirror lag和获取政策未记录 |
| Godot | Gitee mirror origin | `4.8.0-dev` | describe 指向 4.7 stable ancestry；mirror currentness 未记录 |
| Graphics | GitHub `github` remote、shallow 1 commit | Core package `17.6.0` | detached/grafted；需要固定 acquisition recipe、LFS与commit fetch policy |

Snapshot ID 至少必须区分：VCS commit、engine/package semantic version、vendor branch/build/changelist、remote locator、mirror relation、shallow/sparse/submodule/LFS状态、dirty digest、license/access policy和取得时间。一个显示字符串不能覆盖这些正交维度。

### 2.3 当前 citation 形态

1. 1,472 个文件中 `.h` 533、`.cpp` 366、`.rs` 274、`.cs` 170；其余包括 Markdown、workflow、shader、TOML、JSON、XML、Python、license和archive。
2. 124 个 directory locator 跨 engine root、crate、renderer、editor、shader、test scene和文档生成目录。它们适合作为 search scope，不适合作为精确 claim evidence。
3. 229 MB Unreal C++ API tgz 是生成文档 archive，不是单一 source symbol；如果 claim 来自归档内部页面，必须记录 archive digest与member path。
4. 最常复用路径包括 Fyrox executor/plugin、Bevy plugin、Unity RenderGraph、Godot main和 Unreal ModuleManager。复用说明这些是共享 precedent，不说明不同报告的 claim 相同。
5. 当前 frontmatter 没有 symbol、member、line range、excerpt digest、claim ID、primary/secondary role、positive/negative precedent或translation verdict。
6. `reference-engine-map.md` 已明确 primary/secondary routing 和“不机械复制”，但 routing decision 没有固化到每份报告的机器字段。

### 2.4 License、access 与产品边界

参考源码只用于工程研究，不得静默成为产品 BuildSet 输入。`dev/` 当前整体被主仓忽略，这阻止了意外 commit，却也使 source review 不可复现；正确修复不是把 76 GB 内容全部纳入 Git，而是跟踪小型 manifest/receipt，按各 family 的许可与访问条件在授权环境中 materialize。Unreal 只提供 EULA URL，Graphics 使用 Unity Companion License，开源 family 又各有自身 third-party tree。任何 excerpt、generated summary、patch inspiration或 copied source 都必须走 Tooling17 的 NoticeGraph/derived-source admission；本篇只提供引用和访问元数据，不判定法律义务。

## 3. 必须保留的工程基础

### 3.1 离线源码可定位

现有 1,596 个唯一 locator 全部能在当前机器解析，关键架构判断不依赖网页即时状态。后续 manifest 应保留离线优先和 content digest，而不是把 review 改成易漂移的 URL 集合。

### 3.2 Reference routing 已有工程判断

repo-local skill 先按 Zircon owner 和任务约束选择 primary reference，再用 secondary reference补足 Rust、编辑器、渲染或系统规模约束，并显式禁止单一引擎机械决定本地目录/API。这是正确 policy 雏形，应转成 schema/validator 和 review receipt，不应删掉。

### 3.3 五家核心 reference 覆盖广

102/142 篇报告同时使用五个核心 family；剩余报告按主题选择较少 family，例如某些 Unreal 专有工作流。目标是证明 selection rationale 和 applicability，不是强制每篇达到五家计数。

### 3.4 开源 reference 有精确 commit 基础

Bevy、Fyrox、Godot、Graphics 当前 HEAD 都可解析，工作树 clean，Git connectivity check无报错。即使 remote/mirror/shallow政策仍欠缺，也可以先生成 immutable SnapshotReceipt，再补 acquisition currentness。

### 3.5 报告已经记录 source path

frontmatter 统一使用 `reference_engines`，迁移不需要从自由文本重新猜全部文件。可以将现有字段定义为 legacy locator list，生成 migration candidate，再由 owner补 claim-level citation。

## 4. 已确认的结构断点

### 4.1 Corpus membership 由本机目录决定

主 Git tree完全不记录 `dev/`，根又没有 manifest；换机、清理目录、切换镜像或更新某个嵌套仓库后，同一 Zircon commit会观察不同 reference truth。Tooling17 的通用 Reference content class不能替代具体 SnapshotReceipt。

### 4.2 Version、revision 和产品标识混成一个“版本”概念

三家开源引擎都出现 nearest tag 与 source/package version差异，Unreal又只有 CL=0 的 Build.version。后续如果报告只写“Bevy 0.20”“Godot 4.8”或“UE6”，无法解析到实际字节，也无法判断某条差异后来是否已变化。

### 4.3 Citation 只到路径，不到 claim

一篇报告可以列 20 个 reference files并提出 50 个差异，但机器不知道哪一文件支持哪一主张、是正例还是反例、引用哪个 symbol、是否只参考目录布局。路径全存在仍可能正文误读、版本不适用或引用范围过宽。

### 4.4 Directory 与 archive locator 不能冻结读取集合

124 个目录在 snapshot更新后会静默扩张/收缩；归档内部member也不在frontmatter。当前聚合 hash只覆盖 1,472 个明确文件，不能覆盖目录当时观察的内容。

### 4.5 Required docs gate 不读取 reference fields

`check_conventions.py` 的执行字段不含 `plan_sources` 或 `reference_engines`。路径删除、大小写错误、越界、目录误作文件、archive member缺失、snapshot drift均不会让 required docs lane失败。

### 4.6 Mirror、shallow、LFS 与访问策略没有统一记录

Fyrox/Godot使用Gitee mirror，Graphics是 shallow/grafted 且 `.gitattributes`声明大量 LFS pattern，Unreal受EULA约束且无Git identity。当前没有 acquisition command、expected commit、remote policy、object/materialization验证和授权principal。

### 4.7 Reference comparison 没有 applicability verdict

Unreal C++ ownership/GC、Unity C# managed/SRP、Godot C++ server/object、Bevy ECS/Rust和Fyrox Rust/editor的约束不同。仅发现相似类型或目录不能证明 API 可以复制、性能更好、failure semantics相同或适合 Zircon crate boundary。

### 4.8 Reference drift 不会使 report 自动 stale

报告普遍 `source_recheck_required: true`，但没有 reference old/new snapshot edge、affected citation、claim invalidation或review receipt。更新一个嵌套 repo后，所有旧路径可能仍存在，文档却继续显示 review_complete。

### 4.9 “当前 Unreal”与“本地 Unreal 快照”未分开

用户目标涉及当前 Unreal，而本机 Unreal只能确认 `6.0.0/CL0` local tree。没有公开/授权更新检查和FeatureParityReceipt时，报告只能说“相对本地快照观察到”，不能把它扩展为对当前公开版本的性能或能力结论。

## 5. P1：Corpus、Snapshot 与 Provenance

1. **TOOL-REFERENCE-P1-001**：定义 versioned `ReferenceCorpusManifest`，显式列 engine family、snapshot、root、content class、owner与允许用途。
2. **TOOL-REFERENCE-P1-002**：定义稳定 `ReferenceEngineId`，把 Unreal/Bevy/Fyrox/Godot/Unity Graphics与可选 Slint/Piccolo/Theatre等 family分开，不以本机目录名作身份。
3. **TOOL-REFERENCE-P1-003**：定义 `ReferenceSnapshotId`，由engine identity、VCS/build identity、manifest version和root tree digest导出。
4. **TOOL-REFERENCE-P1-004**：Git snapshot记录 full commit、branch仅作提示、dirty state/digest、shallow/sparse状态、submodule set和LFS materialization receipt。
5. **TOOL-REFERENCE-P1-005**：Unreal无Git快照生成 file manifest/tree digest，绑定 `Build.version`、CL、branch、acquisition channel和授权环境；CL=0不得宣称唯一官方build。
6. **TOOL-REFERENCE-P1-006**：mirror记录 canonical upstream、mirror URL、last fetched upstream identity、lag/availability状态；personal mirror不能静默冒充canonical origin。
7. **TOOL-REFERENCE-P1-007**：semantic/package version、nearest tag、branch、build/changelist和commit使用正交字段，resolver禁止择一代替完整snapshot。
8. **TOOL-REFERENCE-P1-008**：每个snapshot关联license/access policy ID、license text digest、授权principal/环境和禁止再分发标记；不在manifest复制credential。
9. **TOOL-REFERENCE-P1-009**：Reference corpus永不进入 Zircon product Cargo/asset/export SourceSet；build read audit发现依赖时fail closed并转Tooling17 admission。
10. **TOOL-REFERENCE-P1-010**：每个family提供幂等 acquisition/verify recipe，验证expected identity后才publish local root pointer。
11. **TOOL-REFERENCE-P1-011**：shallow、partial、sparse、submodule、LFS和generated docs分别声明 completeness profile；缺required object/blob时snapshot不可晋升。
12. **TOOL-REFERENCE-P1-012**：corpus更新采用staging root、验证和原子generation pointer；不得原地pull使进行中的review跨代读取。

## 6. P1：Citation、Evidence 与 Resolver

13. **TOOL-REFERENCE-P1-013**：定义稳定 `CitationId`，每条 citation绑定一个SnapshotId和一个精确 locator kind。
14. **TOOL-REFERENCE-P1-014**：locator区分`File`、`Symbol`、`LineRangeHint`、`DirectorySearchScope`、`ArchiveMember`、`GeneratedDocPage`和`ExternalUrl`。
15. **TOOL-REFERENCE-P1-015**：source claim优先引用language-aware symbol/member；line range只作同代展示提示，不作跨代唯一身份。
16. **TOOL-REFERENCE-P1-016**：每个文件或archive member记录content digest与bytes；可选 excerpt记录单独digest，不把大段受限源码复制进报告。
17. **TOOL-REFERENCE-P1-017**：124 个 legacy directory locator迁移为SearchScope，并把实际支持claim的文件冻结到ResolvedCitationSet。
18. **TOOL-REFERENCE-P1-018**：归档引用记录archive digest、member path/member digest和生成器/版本；229 MB tgz不能作为一个不透明“文件证据”。
19. **TOOL-REFERENCE-P1-019**：GeneratedDoc、source、test、example、workflow、license与package metadata标记EvidenceRole，不允许互相替代语义证明。
20. **TOOL-REFERENCE-P1-020**：扩展结构化文档parser和validator读取 `plan_sources/reference_engines/citations`，检查root、kind、存在性、case、digest和snapshot解析。
21. **TOOL-REFERENCE-P1-021**：path canonicalizer使用slash与repository-preserved casing；三处`dev/fyrox`迁移后在case-sensitive fixture上验证。
22. **TOOL-REFERENCE-P1-022**：同一路径跨报告复用保留独立CitationId/ClaimEdge；289个重复locator不能折叠为一条全局语义。
23. **TOOL-REFERENCE-P1-023**：resolver输出`ResolvedCitationSetReceipt`，包含tool/schema、snapshot、sorted locator、digest、unresolved原因和terminal状态。
24. **TOOL-REFERENCE-P1-024**：目录、glob和search query解析必须在冻结snapshot内执行并保存result set；未来新增文件不能回写改变旧receipt。
25. **TOOL-REFERENCE-P1-025**：引用图建立反向edge：snapshot/file/symbol变化可定位报告、claim、milestone与owner。
26. **TOOL-REFERENCE-P1-026**：禁止以“路径存在”“关键词命中”“五家都列出”直接生成工程差距；它们只能生成待人工语义review candidate。

## 7. P1：Comparison Claim、Applicability 与 Translation

27. **TOOL-REFERENCE-P1-027**：每条跨引擎结论赋 `ComparisonClaimId`，记录问题、Zircon当前证据、reference citation、结论与置信边界。
28. **TOOL-REFERENCE-P1-028**：claim声明类型：architecture、ownership/lifecycle、data model、algorithm、workflow、UI、toolchain、failure、security或performance。
29. **TOOL-REFERENCE-P1-029**：记录 primary/secondary reference选择理由，连接 routing rule；不要求无关family凑数。
30. **TOOL-REFERENCE-P1-030**：记录 positive precedent、negative precedent、counterexample和unknown，不把“实现不同”自动判成Zircon缺陷。
31. **TOOL-REFERENCE-P1-031**：`ApplicabilityDecision`列出共同约束、不同约束、规模、平台、语言/runtime、ownership与产品阶段。
32. **TOOL-REFERENCE-P1-032**：`TranslationDecision`明确吸收 invariant、boundary、algorithm还是workflow，并记录拒绝复制的API/layout/constant/unsafe机制。
33. **TOOL-REFERENCE-P1-033**：C++/GC/UObject、C# managed/SRP、Godot Object/Server、Bevy ECS和Fyrox Rust/editor差异必须作为适配输入，不得只做类型名映射。
34. **TOOL-REFERENCE-P1-034**：目录拓扑比较先映射owner/domain/visibility与增长压力，再决定 Zircon crate/module；文件数量或同名目录不构成架构证据。
35. **TOOL-REFERENCE-P1-035**：算法/容器比较记录complexity、allocation、data layout、threading、failure和workload前提；最终性能结论交Tooling07/32实测。
36. **TOOL-REFERENCE-P1-036**：UI/editor workflow比较记录用户任务、transaction、undo/save/recovery、latency和accessibility，不以截图相似替代产品行为。
37. **TOOL-REFERENCE-P1-037**：reference之间意见不一致时保留AlternativeSet与选择理由；禁止删除反例只留下支持既定设计的来源。
38. **TOOL-REFERENCE-P1-038**：每条accepted claim映射 Zircon canonical owner、finding/milestone和验收门；reference tooling不拥有领域实现。

## 8. P1：Currentness、Review 与 Comparison Qualification

39. **TOOL-REFERENCE-P1-039**：snapshot更新生成old/new diff summary，按CitationGraph标记受影响claim为`recheck_required`，不静默继承review_complete。
40. **TOOL-REFERENCE-P1-040**：`ReferenceReviewReceipt`绑定Zircon SourceSet/dirty digest、ReferenceCorpusId、ResolvedCitationSet、reviewer policy、tool/schema和时间。
41. **TOOL-REFERENCE-P1-041**：claim状态至少区分candidate、supported、qualified-with-limits、contradicted、superseded、unknown和incomparable。
42. **TOOL-REFERENCE-P1-042**：更新reference路径但未复核语义时只允许`locator_migrated`，不得自动保留supported结论。
43. **TOOL-REFERENCE-P1-043**：Tooling28 currentness evaluator消费reference drift reason code；本篇不另建文档发布状态机。
44. **TOOL-REFERENCE-P1-044**：性能/表现对比必须引用Tooling07 ComparisonReceipt与Tooling32 FeatureParity/Workload；source precedent不能替代同硬件实测。
45. **TOOL-REFERENCE-P1-045**：任何“当前 Unreal”主张记录外部版本核验渠道、核验时间、local snapshot relation和无法核验项；否则措辞限定为local snapshot。
46. **TOOL-REFERENCE-P1-046**：reference update lane区分scheduled refresh、security-triggered refresh与milestone-pinned snapshot；不得强迫历史receipt追随latest。
47. **TOOL-REFERENCE-P1-047**：waiver包含缺失family/object/access、owner、影响claim、期限和替代证据；inconclusive不记passed。
48. **TOOL-REFERENCE-P1-048**：required gate验证manifest/schema/case/digest/claim edge/currentness，输出机器可读receipt并保持新增违规预算为0。

## 9. P2：后续增强

| ID | 增强项 | 边界 |
|---|---|---|
| TOOL-REFERENCE-P2-001 | Language-aware symbol resolver | Rust/C++/C#/shader按各自parser解析；不能用一个regex伪装完整语义 |
| TOOL-REFERENCE-P2-002 | Snapshot diff browser | 展示old/new symbol和claim impact，不自动接受新设计 |
| TOOL-REFERENCE-P2-003 | Reference provenance panel | Editor/Hub只展示manifest/receipt，不取得受限源码authority |
| TOOL-REFERENCE-P2-004 | Archive member index | 为大型生成文档归档建内容寻址索引，原archive按access policy保留 |
| TOOL-REFERENCE-P2-005 | Call/type graph neighborhood | citation可附受控上下文，避免只读孤立函数；图只作review辅助 |
| TOOL-REFERENCE-P2-006 | Cross-engine concept registry | 维护概念而非强行统一类型名，允许一对多和无对应项 |
| TOOL-REFERENCE-P2-007 | Decision conflict visualization | 显示各reference约束、Zircon选择和被拒方案，不投票选多数 |
| TOOL-REFERENCE-P2-008 | Review workload prioritizer | 按affected owner、severity、snapshot drift和release风险排序，不按引用次数代替风险 |
| TOOL-REFERENCE-P2-009 | Reproducible search query | 保存parser/query/tool版本与结果digest，支持目录探索复核 |
| TOOL-REFERENCE-P2-010 | Restricted-source redaction scan | 阻止受限源码大段进入公开artifact；规则由Tooling17/security owner审批 |
| TOOL-REFERENCE-P2-011 | Reference health dashboard | 显示materialized/current/stale/inaccessible/partial，不把offline判失败 |
| TOOL-REFERENCE-P2-012 | Longitudinal architecture ledger | 记录同一概念跨snapshot演化，供迁移决策使用，不将latest自动视为best |

## 10. 参考实现自身的 provenance 对照

| Reference | 本轮可观察机制 | Zircon reference governance应吸收 | 不应机械推断 |
|---|---|---|---|
| Unreal | `Build.version`分major/minor/patch/changelist/compatible/branch，源码与生成C++ API archive共存，license指向EULA | non-Git vendor snapshot需要build identity、tree digest、archive/member role和access policy | `6.0.0`或目录名可唯一表示官方/current source；可以重新分发源码 |
| Bevy | full Git commit、GitHub origin、workspace package version、双license文本 | commit与package version分离，root identity可由clean Git receipt重建 | nearest tag与当前package version等价；其ECS/API必然适用Zircon |
| Fyrox | full Git commit、Rust crate/version、真实editor/runtime结构、MIT文本 | Rust-native architecture precedent仍要绑定commit和mirror relation | personal mirror天然等价canonical upstream或始终current |
| Godot | full Git history、`version.py`细分4.8 dev、MIT文本和大型server/editor source | source version与tag ancestry分字段；成熟C++ engine可作server/editor owner对照 | tag describe的4.7代表当前source version，或Object模型应直接复制 |
| Unity Graphics | package `17.6.0`、GitHub remote、shallow commit、LFS patterns、package/render graph source | package identity、commit、shallow/LFS completeness和license/access共同进入snapshot | source文件存在就证明全部LFS/test corpus完整，或C# pool/SRP实现可直接移植Rust |

这些参考只证明“成熟工程会显式携带多维版本、package、license、source/generated/test结构”以及当前本地文件中的具体设计。它们不证明 Zircon 已达到同等规模、正确性或性能，也不证明某一参考版本代表 2026-08-16 的全部公开最新能力。

## 11. 目标架构

### 11.1 证据链

```text
Authorized Acquisition Recipe
            |
            v
ReferenceCorpusManifest
  - EngineId / root / access / completeness policy
            |
            v
ReferenceSnapshotReceipt
  - commit/build/version/tree digest/dirty/shallow/LFS
            |
            v
ResolvedCitationSet
  - file/symbol/archive member/search scope + digest/role
            |
            v
ComparisonClaim
  - Zircon evidence + reference precedent + claim kind
            |
            v
ApplicabilityDecision -> TranslationDecision -> Domain Finding/Milestone
            |
            v
ReferenceReviewReceipt
  - Zircon SourceSet + snapshot set + reviewer/tool/schema
            |
            v
CurrentnessEvaluator / Recheck / Supersession / Qualification
```

Manifest和resolver属于build/review控制面，不能进入runtime frame loop、产品资源加载或shipping artifact。报告正文可保留人类可读对照，机器字段只负责身份、证据边和状态，不试图自动替代工程判断。

### 11.2 `ReferenceSnapshotReceipt`

最低字段：

- `ReferenceEngineId`、snapshot schema、root generation和acquisition recipe ID；
- VCS kind、remote/canonical upstream、full commit、branch hint、tag set、shallow/sparse/submodule/LFS状态；
- engine/package/build/changelist/compatibility version vector；
- sorted tree/file manifest digest、dirty path/content digest和materialization completeness；
- license/access policy、authorized environment/principal reference、redistribution class；
- created/verified generation、tool version、previous snapshot和terminal status。

### 11.3 `ComparisonClaim`

最低字段：

- stable ClaimId、owning report/domain/finding、question和claim kind；
- Zircon SourceSet evidence、current behavior与缺口；
- primary/secondary CitationId、precedent role和被观察机制；
- shared/different constraints、version/platform/workload/language/ownership applicability；
- translation target、accepted invariants、rejected mechanics和alternatives；
- claim state、reviewer、limitations、supersession与qualification receipt。

### 11.4 Currentness 传播

Reference snapshot变化不等于所有报告失效。resolver先计算changed file/symbol/archive member，再沿CitationEdge传播到claim；owner判断语义是否仍成立。纯注释/无关文件变化可产生`unaffected` receipt，symbol删除/行为变化进入recheck。Zircon source变化同样沿own-evidence edge触发。历史review固定旧snapshot永久可复核，但不继续投影为current recommendation。

## 12. 实施里程碑

### M0 · 冻结当前 corpus 与 legacy ledger

- 记录本报告中的五个核心snapshot、142篇legacy引用账本、1,472文件hash和124目录scope。
- 将三处`dev/fyrox`标为case migration candidate，不在review提交里偷改。
- 为Unreal生成non-Git tree manifest draft；为四个Git family生成snapshot receipt draft。
- 明确当前报告只相对local snapshot，不回写“current Unreal”结论。

### M1 · Schema、Manifest 与 Resolver

- 定义EngineId、SnapshotId、CitationId、locator kind、EvidenceRole和receipt schema。
- 让reference-routing map生成/校验family registry与primary/secondary role。
- 扩展文档parser读取legacy/new fields，先report-only输出，不立即阻断历史债务。
- 建case-sensitive、directory expansion、archive member、shallow/LFS和missing object fixture。

### M2 · Acquisition、Access 与 Atomic Snapshot

- 为Bevy/Fyrox/Godot/Graphics建立固定commit verify recipe和mirror/upstream policy。
- 为Unreal建立授权环境下的manifest/digest流程，不跟踪或上传受限源码。
- snapshot staging验证完成后原子发布root generation；review持有generation lease。
- Tooling17接入license/access/derived-source admission，产品SourceSet对reference read fail closed。

### M3 · Legacy Citation 迁移

- 先将1,472 file locator转换为content-bound Citation；124 directory locator保留为SearchScope。
- 按复用度和P0/P1领域风险迁移前50个高价值shared path。
- 每份报告把path list拆成claim edges，不要求一次重写全部正文。
- 迁移三处case错误并在Windows/Linux resolver fixture中证明相同canonical identity。

### M4 · Applicability 与 Translation Review

- 为architecture/runtime/editor/graphics/tooling建立最小claim模板。
- 选Module/Plugin、ECS/World、RenderGraph、Editor transaction和Build pipeline作为五个迁移样本。
- 强制记录不同语言、ownership、threading、failure、scale和product约束。
- accepted claim映射回原领域milestone；reference tooling不实现功能。

### M5 · Drift 与 Currentness

- snapshot更新生成typed diff和affected claim set。
- Tooling28消费reference stale reason，报告状态按claim而不是全篇path existence计算。
- historical receipt保持immutable；locator-only migration与semantic re-review分开。
- scheduled/latest与milestone-pinned snapshot并存，不强制历史追新。

### M6 · Required Gate 与 Review UX

- required docs lane验证新增citation无missing/case/digest/snapshot错误，历史债务采用typed baseline且预算为0。
- 提供claim/source/snapshot反向查询和受限excerpt策略。
- reviewer看到primary/secondary、alternatives、limitations和unresolved access，不只看路径清单。
- gate输出artifact/receipt，由Coordinator/CI消费，不把自由文本summary当passed。

### M7 · External Comparison Qualification

- Tooling07/32连接FeatureParity、workload、hardware与benchmark ComparisonReceipt。
- “current Unreal”类主张绑定可验证版本关系和授权/公开来源；缺项显示local-only/incomparable。
- 产品release只消费qualified domain finding与实测，不消费reference popularity。
- 定期审计corpus access、mirror lag、shallow/LFS completeness和claim currentness。

## 13. 验收门

1. `ReferenceCorpusManifest`有schema、EngineId、root、owner、access、completeness和用途政策。
2. 主 Zircon Git commit加manifest即可解析expected reference snapshot；不要求提交76 GB源码。
3. 五个核心family均生成terminal SnapshotReceipt，包含full identity和tree/file digest。
4. Unreal CL=0 snapshot使用tree manifest补足身份，且不宣称唯一官方build。
5. Git family区分commit、nearest tag、package/source version和branch hint。
6. Fyrox/Godot mirror记录canonical upstream relation与last verified generation。
7. Graphics shallow/grafted状态、remote、commit、LFS/materialization profile进入receipt。
8. snapshot dirty、partial、missing object/blob或access denial为typed状态，不填充clean/complete。
9. reference root更新采用staging与原子generation；并行review不会跨代读文件。
10. product build/cook/export读取reference root会被SourceSet read gate阻断。
11. 每个CitationId只绑定一个SnapshotId与一个locator kind。
12. file/symbol citation记录content digest；line range只作为同代hint。
13. 124个legacy directory locator全部分类为SearchScope或解析为冻结file set。
14. archive citation记录archive/member digest；CppAPI tgz不再是不透明单文件证据。
15. reference fields进入结构化parser，missing/escape/wrong-root能失败。
16. case-sensitive fixture会发现并在迁移后关闭三处`dev/fyrox`错误。
17. 289个复用locator保留独立claim edges，不被全局去重成一个语义。
18. resolver receipt记录tool/schema/snapshot/result set/unresolved reason和hash。
19. 新增目录文件不会改变旧SearchReceipt；只有新generation可产生新结果。
20. restricted source excerpt遵守access/size/publication policy，不泄露credential或大段源码。
21. 每个ComparisonClaim绑定Zircon evidence、reference citation、claim kind和owner。
22. primary/secondary selection有routing rationale；无关family不要求凑数。
23. positive、negative、counterexample、unknown和incomparable可机器区分。
24. ApplicabilityDecision记录语言/runtime、ownership、thread、failure、platform、scale和版本差异。
25. TranslationDecision列出吸收的invariant与明确拒绝复制的mechanic。
26. directory/API名称相似不能在没有owner/semantic mapping时自动生成finding。
27. algorithm/performance claim必须连接Tooling32 workload/cost和Tooling07 measurement receipt。
28. editor/UI claim包含task、transaction、undo/save/recovery与accessibility约束。
29. reference冲突保留AlternativeSet和选择理由，不删除反例。
30. accepted claim映射原领域finding/milestone；reference owner不接管实现。
31. snapshot变化只使受影响Citation/Claim进入recheck，不把全仓无差别标红。
32. locator迁移不能自动维持semantic supported状态。
33. historical ReferenceReviewReceipt固定旧snapshot且长期可复核。
34. Tooling28能显示reference drift reason、owner、first stale generation和replacement。
35. report的review_complete不能覆盖内部contradicted/unknown required claim。
36. waiver有owner、scope、影响claim、替代证据和expiry；inconclusive不记passed。
37. required CI对新增manifest/citation/claim违规保持0预算并发布机器receipt。
38. legacy迁移期间旧path list与新CitationSet有可审计映射，无silent drop。
39. “当前 Unreal”结论绑定外部/local版本关系、FeatureParity和ComparisonReceipt。
40. 没有同场景、同画质、同平台、同硬件、同采样证据时，不得宣称Zircon性能或表现优于当前Unreal。

## 14. Owner 边界

| Owner | 本篇要求 | 本篇不接管 |
|---|---|---|
| Tooling33 / Reference Review | corpus/snapshot/citation/claim/applicability/translation/drift receipt | 任何runtime/editor/plugin具体设计或实现 |
| Tooling17 / Repository Content | Reference content class、license/notice、derived source、archive/export policy | claim语义和snapshot citation图 |
| Tooling28 / Documentation | DocumentId、SourceGraph、publication与currentness UI/state machine | reference acquisition、engine identity和translation verdict |
| Tooling07 / Performance Evidence | benchmark、hardware、statistics、baseline、ComparisonReceipt | source precedent与架构适配判断 |
| Tooling32 / Hot Path | workload、cost、complexity、FeatureParity和product qualification | reference corpus materialization |
| Tooling13 / Repo Control | skill/rule/manifest hook与required CI policy | reference内容或领域结论authority |
| Security / Supply Chain | principal、credential、remote trust、restricted access和audit | 选择哪个架构precedent |
| Domain Runtime/Editor/Plugin/App owner | 解释claim、接受/拒绝translation、实现与验收 | 自行把reference path声明为资格通过 |

本报告映射全局 owner `O00 O01 O07 O08 O09 O10 O11 O14 O16`：O01/O14承载manifest、resolver、receipt和CI；O07/O11承载provenance/trust/evidence；O08/O09/O10/O16消费领域claim；O00禁止reference path越级投影为capability complete。

## 15. 验证与 Currentness

本轮对七个分类目录的142篇报告解析frontmatter，确认2,099次reference引用、1,596个唯一locator、1,472文件、124目录与0 missing；1,472文件共275,849,543 bytes，按ordinal path加逐文件SHA-256聚合得到`d1ba462cace4c62936a5dd411c1bd4af3d2afb9aaa6bb4d2e983de12bf5203ad`。该fingerprint不包含目录递归内容，因此只代表明确file locator set，不能冒充完整ReferenceCorpusId。

本轮读取五个核心tree的Git/build/package/license状态并执行四个Git checkout的connectivity、shallow、status检查。Bevy/Fyrox/Godot为非shallow clean；Graphics为shallow/grafted、rev-list count 1、clean，具有GitHub remote；Unreal无`.git`，Build.version hash为`3dcd8b6654872d013f2b00aa5f0ef1e0db89d4e8627ce820554675a27f228ea6`。这些命令证明本地事实，不证明remote网络可用、latest状态或法律许可结论。

当前 Zircon branch为`main`，source revision为`ae2be3d865a937b9ed368bf965592045346c64e3`，worktree含其他Session改动。30个frontmatter输入路径均存在且唯一；按canonical path ordinal排序，以`path + LF + raw file SHA-256 + LF`编码的输入fingerprint为`fb6c20d1534d3b2d6206449d1e14044560f98761be335cd0dc90c3d5f399e356`，总输入字节230,727,743，其中包含229 MB Unreal归档。本轮未修改production、test、manifest、workflow、`dev/`或既有报告引用，也未重跑Editor/Hub/WOC/plugin已知阻断。实施前必须重新解析分类报告数、reference locator、nested repo identity、dirty state、license/access policy与Zircon SourceSet；任何snapshot或报告引用变化都使本篇统计进入recheck。

## 16. Review 交接

首个实施切片是M0/M1，不是立即更新所有参考库：先将当前五个snapshot和142篇legacy locator冻结为小型manifest/receipt，定义locator kind与case-sensitive resolver，并让现有 `reference_engines` 作为legacy input产生typed diagnostics。没有原子snapshot与claim edge之前，直接pull参考库只会扩大不可复现窗口。

第二个切片选择五个高复用且跨域的样本：Unreal ModuleManager、Bevy/Fyrox plugin lifecycle、Godot main/server ownership、Unity RenderGraph，以及一个Unreal生成API archive member。每个样本完成SnapshotId、CitationId、ComparisonClaim、ApplicabilityDecision和TranslationDecision，再扩展到P0/P1领域报告。

禁止的捷径：把整个`dev/`提交进主Git、只记录“UE6/Bevy latest”、以nearest tag代替commit、把目录存在当成读过、按五家引用数量给报告评分、自动把reference diff生成Zircon缺陷、复制受限源码/常量/API、用路径hash证明语义正确、或在没有FeatureParity与同条件实测时宣称优于当前Unreal。
