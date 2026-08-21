---
related_code:
  - Cargo.toml
  - .github/workflows/ci.yml
  - tools/check_conventions.py
  - .codex/skills/zircon-project-skills/development-conventions.md
  - docs/plans/index.md
  - docs/zircon_app/prelude.md
  - docs/zircon_runtime/operation.md
  - docs/zircon_runtime_interface/plugin_api.md
  - docs/zircon_plugins/authoring-runtime-plugins.md
  - docs/zircon_hub/index.md
  - docs/engine-architecture/index.md
  - docs/editor-and-tooling/index.md
tests:
  - tools/tests/test_check_conventions.py
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_tooling/01-workspace-toolchain-ci-validation-and-developer-entrypoints-review.md
  - docs/plans/optimize/zircon_tooling/12-top-level-acceptance-archive-serialization-fixture-provenance-currentness-migration-review.md
  - docs/plans/optimize/zircon_tooling/13-repository-codex-skill-hook-structural-audit-governance-security-currentness-review.md
  - docs/plans/optimize/zircon_tooling/17-repository-content-source-set-ignore-generated-vendor-license-distribution-review.md
  - docs/plans/optimize/zircon_tooling/20-cargo-package-workspace-feature-dependency-target-graph-build-receipt-review.md
  - docs/plans/optimize/zircon_tooling/27-version-domain-schema-compatibility-support-window-migration-deprecation-upgrade-downgrade-review.md
reference_engines:
  - dev/UnrealEngine/README.md
  - dev/UnrealEngine/Engine/Documentation/Builds/CppAPI-HTML.tgz
  - dev/UnrealEngine/Engine/Documentation/Source/Programming/UnrealBuildSystem/ModuleFiles/ModuleFilesProperties/ModuleFilesProperties.INT.udn
  - dev/bevy/README.md
  - dev/bevy/Cargo.toml
  - dev/bevy/.github/workflows/docs.yml
  - dev/bevy/examples/README.md
  - dev/Fyrox/README.md
  - dev/Fyrox/fyrox/Cargo.toml
  - dev/Fyrox/fyrox/src/lib.rs
  - dev/godot/README.md
  - dev/godot/doc/tools/make_rst.py
  - dev/godot/doc/classes/@GlobalScope.xml
  - dev/Graphics/Packages/com.unity.render-pipelines.core/package.json
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Documentation~/index.md
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Documentation~/TableOfContents.md
  - dev/Graphics/Packages/com.unity.render-pipelines.core/CHANGELOG.md
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 28 · Documentation、API Reference、Plan Currentness、Link、Source Trace 与 Knowledge Publication 审查

## 1. 结论

Zircon并不是没有文档。Git索引下的`docs/`已有5,329个条目，其中5,328个当前存在，物理体积1,148,609,716 bytes；`docs/plans`有2,084个tracked条目，其中2,083个存在、约46.9 MB；`docs/tests`单独占1,216个文件和875.9 MB。在写入本报告前的工作树扫描基线中可读到3,240篇Markdown，2,649篇带frontmatter。Runtime、Editor、Interface、Plugins、Hub、App、architecture、tooling和大量编号计划都已经积累了可复用说明。规范还要求模块文档维护`related_code`、`implementation_files`、`plan_sources`与`tests`，`tools/check_conventions.py`也确实实现了仓库路径解析、缺失路径、绝对路径和越界路径检查，并进入required CI。这些基础必须保留。

问题不是“文档数量不够”，而是没有工程级Knowledge Publication控制面。仓库根没有`README.md`，`docs/`没有总入口，`docs/plans/index.md`只路由5组计划，却没有覆盖现存的optimize、MVP、performance、WOC和tooling/session工作；文档类型、owner、版本、适用BuildSet、source revision、复审时点、过期原因和发布状态没有统一合同。规范文档、模块说明、执行计划、某次验证结果、视觉证据和历史归档混在同一Markdown语义里，因此`Fresh`、`passed`、`current`之类文字可能在源代码变化后仍永久显示为真。

现有门禁也没有闭合。本轮只读执行`python tools/check_conventions.py --only docs --json`返回非零：2,649篇结构化文档、78,359个路径引用中有692项违规，影响242篇文档，包括667个missing path、22个repository escape和3个absolute path。普通Markdown链接又不在该validator的解析范围内；对当前3,240篇Markdown做词法下界扫描，得到2,477个local link candidate，其中61个唯一目标缺失、共64次、影响27篇文档。该数字不是完整Markdown AST或anchor检查，但已经证明“frontmatter路径门存在”不等于文档图可导航。

Rust API publication同样没有形成产品。162份Cargo manifest定义159个package，但`readme`、`documentation`、`repository`、`homepage`和`[package.metadata.docs.rs]`均为0；根`[workspace.package]`也只提供version、edition与license。CI没有`cargo doc`、doctest、broken intra-doc link、Markdown link、文档站构建或版本化发布job。对12,485个production-like Rust文件的严格词法下界扫描，23,636个精确`pub struct/enum/trait/fn/type/const/static/mod`定义中只有1,445个前邻`///`，约6.1%；170个`lib.rs/main.rs`中只有13个含`//!`。这不等于语义文档覆盖率，但足以否定“公共引擎和插件SDK已有可消费API reference”的假设。

本篇不重复Tooling01的通用CI/Cargo门禁、Tooling12的历史验收与fixture corpus、Tooling13的Codex skill/hook/结构审计、Tooling17的Git SourceSet与license、Tooling27的全产品Version Domain。Tooling28是文档内容分类、source trace、currentness、API reference、链接图、版本化文档构建、发布、索引与退役的canonical专项owner；共享身份与版本用O01/O03，claim/evidence用O00/O11，构建和发布由O14执行。登记 **1项P0、56项P1和12项P2**。

## 2. 审查边界与证据

| Evidence | 本轮结果 |
|---|---|
| E1 tracked docs inventory | `docs/` 5,329个tracked条目，5,328个存在，1,148,609,716 bytes；1个已索引plan当前物理缺失，本轮不修复相邻工作树 |
| E2 plan inventory | `docs/plans` 2,084个tracked条目，2,083个存在；2,079个tracked Markdown中2,078个存在；存在文件约46,892,915 bytes |
| E3 evidence footprint | `docs/tests` 1,216个tracked文件，875,927,659 bytes；`.png`、`.log`、`.txt`、`.rdc`等结果与普通文档同树但没有统一retention/publication class |
| E4 metadata scan | 写入本报告前当前工作树3,240篇Markdown、2,649篇frontmatter；仅130篇有`review_status/implementation_status/source_recheck_required`，且主要来自本优化计划 |
| E5 plan currentness | 2,078篇现存tracked plan Markdown中1,697篇有frontmatter，只有19篇有三项review/implementation/recheck状态，43篇提及source revision，314篇提及fingerprint |
| E6 structured path gate | `check_conventions.py --only docs --json`：2,649篇、78,359 path、692 violation、242篇受影响；667 missing、22 repository escape、3 absolute；exit 1 |
| E7 ordinary link lower bound | 3,240篇Markdown，2,477个local candidates，61个唯一missing target、64次occurrence、27篇受影响；regex scan，不含完整AST/anchor语义 |
| E8 package publication | 162份Cargo manifest、159个package；`readme/documentation/repository/homepage/docs.rs metadata`全部0 |
| E9 Rust API lexical lower bound | 12,485个production-like `.rs`；23,636个精确public item中1,445个前邻`///`，约6.1%；170个crate root中13个含`//!` |
| E10 entrypoint/workflow | 根`README.md`与`docs/index.md`不存在；3份GitHub workflow均无docs/rustdoc/doctest/Markdown publication lane |
| E11 temporal claim routing | 3,240篇Markdown中2,641篇至少命中date、current、fresh/latest或pass/green之一；archive和plan会合法命中，不能把词法量直接当stale缺陷 |
| E12 reference review | 逐读Unreal、Bevy、Fyrox、Godot与Unity Graphics的入口、API生成、class/package docs、example、TOC与发布工作流代表路径 |
| E13 dynamic scope | 仅运行文档validator；未重跑已知Editor、Hub、WOC或plugin build阻断，也未修改production、CI或manifest |
| Currentness | revision `ae2be3d865a937b9ed368bf965592045346c64e3`，branch `main`；29个关键文本输入clean；按路径ordinal排序，将每项编码为`path + LF + normalized UTF-8 content + LF`，fingerprint `cd37e22652d9cc0126a572761d5d8c063e11ae27a9eb764da14ae40ab349d528`；18,107 LF、918,714 content bytes |

统计解释必须保持克制：

1. 文档多、图片多或日期多都不是缺陷；缺陷是内容没有class、owner、版本与生命周期，consumer无法区分current truth和历史记录。
2. `///`下界会漏掉宏生成项、re-export说明、远距离attribute与模块级叙述，也会把低价值注释计入；它只证明门禁与发布面缺失，不给单个crate判刑。
3. regex链接扫描不会正确解释全部Markdown转义、reference link、HTML、fragment和生成路由；正式实现必须用parser和site graph。
4. 当前692项结构化路径违规来自dirty worktree快照，不能把每项都归因于主分支；但required validator返回红、错误类型跨多个已删除owner路径，是可复核事实。
5. 参考工程的文档规模不证明其架构、性能或每篇内容都正确；只提取可观察的入口、生成、版本、导航和发布边界。

## 3. 必须保留的工程基础

### 3.1 Frontmatter路径合同已有执行器

`check_conventions.py`不是空壳。它读取`implementation_files`、`related_code`与`tests`，规范化repository-relative路径，拒绝absolute path和repository escape，对missing target给出结构化Finding，并缓存resolution。后续应扩展schema和parser，而不是另写一个互不兼容的“文档检查脚本”。

### 3.2 模块文档已经形成较广的source mapping

Runtime 297个tracked文档、Editor 201个、Interface与Plugins等目录都已积累module-detail、overview、testing guide和workflow说明。大量frontmatter已经能反向定位实现文件和测试，说明source trace迁移可以渐进执行，不需要一次重写全部正文。

### 3.3 编号计划与优化报告已有状态字段雏形

本优化报告集合使用`doc_type/review_status/implementation_status/source_recheck_required`，部分计划还记录source revision、fingerprint、验证命令和产出状态。目标schema应吸收这些字段并定义状态机，而不是让另一套publication metadata并存。

### 3.4 参考源码与本仓证据可以离线复核

`dev/`中的Unreal文档source/API archive、Godot class XML和生成器、Bevy docs workflow、Fyrox crate docs、Unity Graphics package docs都能作为本地结构证据。这使review不依赖网页当前状态，也要求报告精确区分“本地观察”与“外部线上可用性”。

### 3.5 文档已经进入required CI

CI执行`tools/check_conventions.py --json`，说明仓库已经接受“文档合同可以阻断提交”的原则。当前需要修复红baseline、补足覆盖和产出可定位报告，不应把门降级为warning来换取绿色。

## 4. 已确认的结构断点

### 4.1 没有单一可发现入口

仓库根只有Cargo、license和policy类文件，没有README；`docs/`也没有index。新开发者、插件作者、引擎集成者和运维者不能从一个稳定入口选择“构建引擎、运行Editor、写plugin、理解Runtime API、执行测试、查看当前计划”。`docs/plans/index.md`只列5组计划，无法承担全仓文档入口。

### 4.2 内容类型是自由字符串，不是消费合同

当前frontmatter出现`module-detail`、`implementation-evidence`、`milestone-detail`、`review-and-refactor-plan`、`design-spec`、`testing-guide`、`plan-output-record`等30余种值，但没有中央registry、required fields、allowed transitions或retention policy。相同的`current`在normative reference、plan和run receipt中含义完全不同。

### 4.3 “结果”被写回长期文档

例如`docs/zircon_app/prelude.md`同时含某时点的`Fresh`通过结果与后续blocked描述，却没有review generation、source fingerprint或机器可判的superseded relation。模块说明因此兼任规范和运行日志；源变更后，消费者只能凭自然语言猜哪一段仍有效。

### 4.4 Frontmatter与普通链接形成两张不一致的图

现有validator只读取有限frontmatter字段，不检查`plan_sources`、reference fields、普通Markdown link、heading anchor、image/link fragment或跨文档symbol。当前结构化路径红与普通链接missing同时存在，说明修复其中一张图不能恢复导航完整性。

### 4.5 API surface没有文档产品身份

package metadata没有documentation/readme/repository，public facade也没有明确`PublicApiSet`。即使未来直接运行`cargo doc --workspace`，内部crate、生成crate、工具、examples、first-party plugin与对外SDK会被混在同一输出，无法定义semver、visibility、support和搜索优先级。

### 4.6 计划目录没有统一currentness状态机

现存tracked plan中绝大多数没有review、implementation与recheck字段；`completed/pending`多为正文词汇，不能可靠聚合。缺失的failure handoff、搬迁后的owner路径和绝对本机路径仍会被旧计划引用。计划需要保留历史，但历史保留不等于继续作为current implementation truth。

### 4.7 证据、fixture与文档发布混用同一存储策略

`docs/tests`承载大量PNG、log、RDC和文本结果，`docs/ui-and-layout`也有大体积视觉素材。Git路径可追踪不代表适合进入面向用户的文档站、source archive或永久保留。Tooling12/17已经定义archive和SourceSet问题，本篇拥有这些class进入或退出文档publication graph的规则。

### 4.8 没有版本化站点、搜索或退役语义

仓库未找到文档build manifest、site artifact、search index、canonical URL、version selector、redirect map、noindex preview、broken-link report或retired page tombstone。源码分支、engine release、plugin SDK版本和文档内容无法稳定绑定。

## 5. P0：Required文档真相门当前为红

### DOC-KNOWLEDGE-P0-001 · Required文档validator发现692项违规，但没有受控baseline与修复事务

CI要求运行的`check_conventions.py`在当前快照对docs返回exit 1，影响242篇文档。错误不是单一临时文件：样本包含已删除的`zircon_asset/zircon_scene/zircon_ui`等旧crate、旧Editor UI路径、缺失测试、绝对本机log和repository escape。若CI实际按当前树运行，required lane不可通过；若某入口没有运行或忽略退出码，又会发布已知断链文档。必须先冻结typed violation inventory，按canonical owner与内容class分批修复或显式retire；每次迁移生成before/after receipt，并保持新增违规预算为0。不得删除validator、放宽missing规则、批量删frontmatter或把required lane改成warning来“修复”。

## 6. P1：Content Model、Owner 与 Authority

| ID | 当前差距 | 工程级重构要求 |
|---|---|---|
| DOC-KNOWLEDGE-P1-001 | 没有canonical `DocumentId` | 使用稳定namespace/type/id，不以可变路径作为唯一身份；rename保留redirect与history |
| DOC-KNOWLEDGE-P1-002 | `doc_type`是自由字符串 | 建立versioned `ContentClassRegistry`与schema，未知类型fail validation |
| DOC-KNOWLEDGE-P1-003 | normative、tutorial、plan、receipt、archive、prototype混写 | 至少拆为Reference、Guide、Plan、EvidenceReceipt、Archive、DesignPrototype、GeneratedApi |
| DOC-KNOWLEDGE-P1-004 | 文档没有唯一owner/team或reviewer policy | 每个Document声明canonical owner、review authority与escalation route |
| DOC-KNOWLEDGE-P1-005 | 一篇文档可同时充当source truth和运行结果 | normative content与generation-bound result分离，正文只引用immutable receipt |
| DOC-KNOWLEDGE-P1-006 | frontmatter字段集合由脚本局部硬编码 | schema registry生成parser、validator、JSON schema和编辑器补全 |
| DOC-KNOWLEDGE-P1-007 | 文档目录位置暗示语义但不强制 | path policy由ContentClass投影，目录迁移不能改变class/identity |
| DOC-KNOWLEDGE-P1-008 | 没有machine-readable publication eligibility | `PublicationDecision`列出class、audience、visibility、version、gate与拒绝原因 |

## 7. P1：Source Trace、Review 与 Currentness

| ID | 当前差距 | 工程级重构要求 |
|---|---|---|
| DOC-KNOWLEDGE-P1-009 | 大部分文档没有review generation | 记录reviewed source revision、dirty digest、tool version和review timestamp |
| DOC-KNOWLEDGE-P1-010 | source path存在即可被视为current | 路径之外记录content digest、symbol identity与resolution generation |
| DOC-KNOWLEDGE-P1-011 | source变更不会自动使文档stale | SourceGraph增量计算affected documents并转入`recheck_required` |
| DOC-KNOWLEDGE-P1-012 | `source_recheck_required`只有布尔值 | 使用原因码、trigger set、first stale generation、owner与deadline |
| DOC-KNOWLEDGE-P1-013 | `current/passed/fresh`是自由文本 | claim必须引用EvidenceSet/BuildSet，过期时投影为stale而非继续显示通过 |
| DOC-KNOWLEDGE-P1-014 | superseded文档没有replacement edge | 明确`supersedes/superseded_by`并验证无环、目标可发布 |
| DOC-KNOWLEDGE-P1-015 | 同一主题可能有规范双真源 | topic authority registry只允许一个current normative owner，其余为adapter或archive |
| DOC-KNOWLEDGE-P1-016 | review完成与实现完成混为一谈 | Review、Implementation、Validation、Publication使用正交状态机与独立receipt |

## 8. P1：Navigation、Link Graph 与 Discoverability

| ID | 当前差距 | 工程级重构要求 |
|---|---|---|
| DOC-KNOWLEDGE-P1-017 | 根无README和角色入口 | 提供最小canonical入口，路由builder、engine user、plugin author、contributor、operator |
| DOC-KNOWLEDGE-P1-018 | `docs/`无总index | index由manifest生成，显示version、class、owner、currentness，不手抄全量列表 |
| DOC-KNOWLEDGE-P1-019 | plans index只覆盖5组 | 从PlanRegistry生成全量active/archive/blocked/superseded视图 |
| DOC-KNOWLEDGE-P1-020 | 普通Markdown link不进validator | 用CommonMark parser构建typed local/external/asset/symbol link graph |
| DOC-KNOWLEDGE-P1-021 | heading fragment没有稳定合同 | 生成稳定anchor或显式anchor ID，rename输出redirect并检测broken fragment |
| DOC-KNOWLEDGE-P1-022 | absolute本机路径进入文档 | 本机证据改用artifact URI/receipt；repository source只允许typed relative path |
| DOC-KNOWLEDGE-P1-023 | image、download与source link同等处理 | 按asset class校验mime、size、digest、license、alt text与publication policy |
| DOC-KNOWLEDGE-P1-024 | 没有搜索索引与结果优先级 | 生成versioned search index，current reference高于archive/result/prototype |

## 9. P1：Rust API、SDK 与 Runnable Example

| ID | 当前差距 | 工程级重构要求 |
|---|---|---|
| DOC-KNOWLEDGE-P1-025 | 没有`PublicApiSet` | 明确runtime facade、runtime interface、plugin SDK、tooling API与internal crate边界 |
| DOC-KNOWLEDGE-P1-026 | 159个package无documentation metadata | 只给可发布/可消费package补readme、repository、documentation和support identity |
| DOC-KNOWLEDGE-P1-027 | 根workspace无共享publication metadata | workspace声明canonical repository/homepage/readme/rust-version，package可受控覆盖 |
| DOC-KNOWLEDGE-P1-028 | CI不构建rustdoc | public API lane运行分层`cargo doc`，broken intra-doc link和warning按policy阻断 |
| DOC-KNOWLEDGE-P1-029 | 绝大多数crate root无`//!` | 每个public crate说明角色、边界、lifecycle、safety、feature与最小示例 |
| DOC-KNOWLEDGE-P1-030 | public item缺docs不受控 | 对PublicApiSet启用`missing_docs`预算，按crate逐步收紧，internal不机械追求100% |
| DOC-KNOWLEDGE-P1-031 | docs示例不执行 | doctest或compiletest绑定feature/target；无法运行的示例显式`no_run`原因与owner |
| DOC-KNOWLEDGE-P1-032 | examples没有文档反向映射 | ExampleManifest声明capability、prerequisite、command、expected artifact和API symbols |

## 10. P1：Build、Versioned Publication 与 Release Binding

| ID | 当前差距 | 工程级重构要求 |
|---|---|---|
| DOC-KNOWLEDGE-P1-033 | 没有`DocumentationBuild` | 文档构建绑定BuildSet、source graph、schema、generator、theme、locale和output digest |
| DOC-KNOWLEDGE-P1-034 | preview与release没有隔离 | PR preview使用不可猜测路径并noindex，release需promotion authority |
| DOC-KNOWLEDGE-P1-035 | 没有版本selector | docs version绑定EngineBuild/ProductVersion，不用branch名字冒充支持版本 |
| DOC-KNOWLEDGE-P1-036 | latest URL没有原子promotion | immutable version目录先验证，再原子切换channel pointer，失败保持旧版本 |
| DOC-KNOWLEDGE-P1-037 | 文档artifact没有manifest | 列出page、asset、API item、search shard、redirect、digest、size与content type |
| DOC-KNOWLEDGE-P1-038 | release notes/changelog未进入同一图 | 变更项链接API/schema/deprecation/migration guide和first supported build |
| DOC-KNOWLEDGE-P1-039 | site生成与source archive无关系 | PublicationReceipt引用Tooling17的SourceArchive和Tooling01的BuildSet |
| DOC-KNOWLEDGE-P1-040 | 没有rollback/retire操作 | 保留immutable generation，channel回退有审计；retire使用tombstone/redirect而非静默404 |

## 11. P1：Audience、Localization、Accessibility 与 Trust

| ID | 当前差距 | 工程级重构要求 |
|---|---|---|
| DOC-KNOWLEDGE-P1-041 | 文档没有audience/profile | 标注engine user、gameplay programmer、render engineer、plugin author、operator等受众 |
| DOC-KNOWLEDGE-P1-042 | internal和public内容无visibility gate | `Visibility`由manifest与publisher控制，敏感内部路径不因链接可达而发布 |
| DOC-KNOWLEDGE-P1-043 | code/log可能携带本机路径与secret | publication前做typed secret/path/credential扫描，命中即拒绝并留Finding |
| DOC-KNOWLEDGE-P1-044 | 外部链接没有trust policy | 区分official/reference/community/download，检查scheme、domain policy与last verified generation |
| DOC-KNOWLEDGE-P1-045 | 没有locale identity/fallback | 文档与fragment使用locale-independent ID，locale fallback和translation currentness显式化 |
| DOC-KNOWLEDGE-P1-046 | 翻译可能在source变化后仍标current | source digest变化自动标translation stale，不自动复制新claim |
| DOC-KNOWLEDGE-P1-047 | image/diagram无系统a11y门 | 发布页检查alt、caption、contrast语义和键盘可达；装饰资产显式标注 |
| DOC-KNOWLEDGE-P1-048 | 生成HTML trust boundary未定义 | 禁止默认执行不可信HTML/script；sanitize、CSP、asset integrity与generator pin进入Build |

## 12. P1：Operations、Metrics、Migration 与 Maintenance

| ID | 当前差距 | 工程级重构要求 |
|---|---|---|
| DOC-KNOWLEDGE-P1-049 | 692项违规没有typed owner队列 | FindingSet按rule/content class/owner/age分组，支持baseline burn-down但新增为0 |
| DOC-KNOWLEDGE-P1-050 | 修文档可与source rename分离提交 | owner rename transaction同时更新source graph、links、redirect和affected currentness |
| DOC-KNOWLEDGE-P1-051 | 没有orphan/unreachable报告 | gate检测无入口current docs、无consumer API pages、孤立assets和循环navigation |
| DOC-KNOWLEDGE-P1-052 | 没有freshness SLO | 按class定义复审触发和最大stale时间，不对archive施加错误的current SLO |
| DOC-KNOWLEDGE-P1-053 | 没有publication observability | 记录build duration、page/link/API counts、cache hit、warning、failure和artifact size |
| DOC-KNOWLEDGE-P1-054 | 文档schema升级无迁移器 | schema change提供deterministic migrator、dry run、diff、rollback与receipt |
| DOC-KNOWLEDGE-P1-055 | 大体积证据没有retention tier | EvidenceReceipt只发布摘要与artifact link，原始capture按Tooling12/17策略保留 |
| DOC-KNOWLEDGE-P1-056 | 文档质量可能靠总数量自认证 | KPI使用可达性、currentness、broken graph、API coverage和task success，不用page count代替质量 |

## 13. P2：后续质量与体验增强

| ID | 建议 |
|---|---|
| DOC-KNOWLEDGE-P2-001 | 提供离线versioned docs bundle，并验证从安装包打开时的相对链接与搜索 |
| DOC-KNOWLEDGE-P2-002 | 为Editor上下文帮助建立稳定symbol/topic URI，不把硬编码网页URL散落到panel |
| DOC-KNOWLEDGE-P2-003 | 从reflection/schema生成属性、枚举、console command与serialization reference |
| DOC-KNOWLEDGE-P2-004 | 生成API diff、新增/删除/deprecated清单，并链接Tooling27迁移指南 |
| DOC-KNOWLEDGE-P2-005 | 为示例建立可截取、可重放的expected output artifact，而不是只证明编译 |
| DOC-KNOWLEDGE-P2-006 | 搜索索引支持symbol alias、旧名称redirect和版本过滤 |
| DOC-KNOWLEDGE-P2-007 | 提供文档doctor，展示本地受影响文档、broken links与最小修复命令 |
| DOC-KNOWLEDGE-P2-008 | 对大型图片和capture生成受控派生格式，保留原始digest与可下载来源 |
| DOC-KNOWLEDGE-P2-009 | 维护术语表和跨模块名词authority，检测同名不同义与旧crate名称残留 |
| DOC-KNOWLEDGE-P2-010 | publication preview提供页面级视觉diff，但视觉变化不能绕过内容/schema gate |
| DOC-KNOWLEDGE-P2-011 | 采集匿名、最小化的search miss/404信号用于改进导航，不记录敏感查询正文 |
| DOC-KNOWLEDGE-P2-012 | 为release docs生成可打印/归档目录和长期校验manifest，支持未来工具迁移 |

## 14. 目标架构

### 14.1 控制链

```text
DocumentSource / Rust PublicApiSet / ExampleManifest / EvidenceReceipt
  -> ContentClassRegistry + DocumentSchema
  -> SourceGraph + LinkGraph + SymbolGraph
  -> CurrentnessEvaluator + VisibilityPolicy
  -> DocumentationBuild(BuildSetId, VersionDomain, Locale, GeneratorSet)
  -> Validators(path, link, anchor, API, example, secret, accessibility)
  -> immutable DocumentationArtifact + PublicationReceipt
  -> review / release promotion / rollback / retirement
  -> versioned site, offline bundle, search index, Editor context help
```

关键约束如下：

1. 文档source、生成API、example与evidence可以共享graph schema，但不能共享同一生命周期owner。
2. `DocumentId`稳定，path可迁移；每次迁移必须产生redirect或显式retired outcome。
3. Currentness是source/build/schema关系的计算结果，不是作者手写一句`current`。
4. Publication是O14操作，必须消费O01 BuildSet和O03 Version Domain；文档正文不能自行宣布release。
5. 验证结果是O11 EvidenceSet；页面只投影receipt，不能把一次passed文字固化成永久事实。
6. Public API只包含显式admitted `PublicApiSet`；internal docs可生成但默认不进入public publication。

### 14.2 ContentClass最小合同

| Class | Currentness输入 | 允许的consumer | 退役政策 |
|---|---|---|---|
| Normative Reference | source symbols/schema/version + owner review | developer、Editor help、SDK consumer | replacement + redirect，旧版本可保留 |
| Guide/Tutorial | referenced API/example BuildSet + task validation | user/contributor | 标stale或迁移，不能静默继续推荐 |
| Generated API | PublicApiSet + rustdoc generator + BuildSet | SDK/API consumer | 随version immutable保留 |
| Plan | review/implementation/validation状态与dependencies | maintainer/automation | complete/superseded/archive，不冒充当前实现 |
| Evidence Receipt | command/environment/source/artifact identity | qualification/dashboard | immutable，按retention tier归档 |
| Archive | provenance + original generation | historical audit | read-only、no current claim |
| Design Prototype | design generation + capability mapping | authoring review | 不进入产品capability truth |

### 14.3 物理模块建议

不建立一个巨型“docs database”。建议按职责拆分：

```text
tools/documentation/
  schema/             ContentClass、DocumentManifest、PublicationReceipt schema
  inventory/          repository/source/API/example discovery
  graph/              path、link、anchor、symbol、supersession graph
  currentness/        source/build/version drift evaluation
  validators/         typed rules，复用check_conventions resolver
  rustdoc/            PublicApiSet与crate selection adapter
  site/               page/search/redirect/offline artifact build
  publish/            preview/release promotion、rollback、retirement
  reporting/          JSON FindingSet与人类摘要
```

`tools/check_conventions.py`可以先成为兼容入口，调用新模块并保持现有JSON字段；完成消费者迁移后再决定是否保留薄wrapper。不要先复制resolver再长期双栈。

## 15. 参考实现的结构差异

### 15.1 Unreal

本地Unreal根README直接路由开发设置、编程/脚本和C++ API；`Engine/Documentation`包含833个文件，既有本地UDN source，也有229,361,318-byte的`CppAPI-HTML.tgz`生成artifact，并可观察到INT/CHN/JPN/KOR等locale变体。这证明大型引擎会区分入口、authoring source、生成API和locale；本报告不据此推断其线上发布系统或所有文档质量。

### 15.2 Bevy

Bevy根README把quick start、API docs和examples作为明确入口；Cargo metadata提供homepage/repository/documentation，workspace lint配置`missing_docs = "warn"`，docs.rs metadata声明构建策略。独立docs workflow固定action revision，用nightly rustdoc构建workspace/all-features文档、抓取example、上传站点artifact，并给development docs加`noindex`。Zircon缺少的是同类分层控制链，不是简单复制一条`cargo doc`命令。

### 15.3 Godot

Godot根README路由官方文档、class reference和demo；本地`doc/classes`有813个XML class source，`doc/tools/make_rst.py`约110 KB，负责生成索引、cross-reference并报告缺失description。它展示了“反射/class schema source -> validator -> generated reference”的可维护边界，Zircon可用于reflection、plugin API和console/schema reference。

### 15.4 Fyrox

Fyrox README路由docs.rs、book和example，`fyrox/Cargo.toml`含readme/homepage/documentation/repository，`fyrox/src/lib.rs`提供crate级说明和文档branding。本地checkout没有book source，不能假设其book发布细节；可确认的差异是Zircon公共package metadata与crate root说明尚未形成。

### 15.5 Unity Graphics

SRP Core package把`package.json`的name/version/description、`Documentation~/index.md`、`TableOfContents.md`与`CHANGELOG.md`放在同一package边界，当前本地`Documentation~`有101个文件。它说明package identity、TOC、docs和release history可以共同演进；Zircon plugin/SDK文档目前没有相同的package/version publication绑定。

## 16. 分层实施顺序

### M0 · 冻结红baseline并阻止新增

保存692项结构化path finding和61个lexical missing link下界的机器清单；给每项分配ContentClass、owner、canonical replacement或retire decision。required gate维持新增违规为0，历史baseline只能单调下降。

### M1 · Schema、Identity 与ContentClass

定义DocumentId、ContentClassRegistry、DocumentManifest、CurrentnessState、PublicationDecision和supersession graph。迁移现有frontmatter，不改写历史正文语义。

### M2 · 统一Graph与Validator

把现有resolver提取成共享库，用Markdown parser和Rust/Cargo adapter构建path/link/anchor/symbol/source graph；输出统一FindingSet、非零required exit和可定位JSON。

### M3 · 清理current documentation graph

优先修复或retire required/current normative docs中的missing、escape、absolute与旧owner路径；archive只保留历史并退出current consumer。生成根README和`docs/index`投影。

### M4 · Public API与Examples

定义PublicApiSet，对Runtime Interface、插件SDK和公开facade分批补crate/module/item docs；启用rustdoc、doctest/compiletest、example manifest和API diff。内部crate不因统一命令误承诺support。

### M5 · Versioned Build与Publication

构建immutable DocumentationArtifact、search index、redirect map、offline bundle与PublicationReceipt；PR preview noindex，release绑定BuildSet并原子promotion/rollback。

### M6 · Currentness与运营

source/API/schema变化自动标受影响文档，translation与guide进入复审队列；监控broken graph、stale age、task success、publication failure和artifact growth。

## 17. 验收门

| Gate | 验收条件 |
|---|---|
| G01 | ContentClassRegistry有schema/version/owner，未知class拒绝 |
| G02 | 所有current可发布文档都有稳定DocumentId和canonical owner |
| G03 | normative、plan、receipt、archive、prototype不再混用状态语义 |
| G04 | current claim绑定source revision/BuildSet/EvidenceSet，不使用自由文本自认证 |
| G05 | source/path/symbol变化能生成affected document set |
| G06 | supersession graph无环且replacement target可发布 |
| G07 | required structured path finding从692单调降到0，新增违规始终为0 |
| G08 | absolute repository-source path与repository escape为0 |
| G09 | 普通Markdown local link与fragment由parser验证，missing为0 |
| G10 | current graph没有orphan normative page和孤立required asset |
| G11 | 根README提供角色化入口且由manifest路由，不复制状态真相 |
| G12 | `docs/index`和plans index从registry生成并覆盖全部current scope |
| G13 | PublicApiSet明确列出对外crate/module和support policy |
| G14 | public package具有readme/repository/documentation/version identity |
| G15 | public crate root有角色、边界、lifecycle、feature和safety说明 |
| G16 | rustdoc required lane构建PublicApiSet且broken intra-doc link为0 |
| G17 | missing_docs预算按crate只降不升，waiver有owner和expiry |
| G18 | runnable examples绑定feature/target/command和expected artifact |
| G19 | doctest/compiletest结果绑定BuildSet并输出EvidenceSet |
| G20 | DocumentationBuild manifest覆盖全部page/asset/API/search/redirect digest |
| G21 | preview与release artifact隔离，preview默认noindex |
| G22 | release文档绑定EngineBuild/ProductVersion，不以可变branch作为版本 |
| G23 | version publication先验完整再原子promotion，失败保持旧generation |
| G24 | rollback与retire有typed operation receipt和审计记录 |
| G25 | search结果默认优先current reference，不让archive/prototype冒充答案 |
| G26 | external link标trust class并记录last verified generation |
| G27 | secret、本机路径、credential命中阻断public publication |
| G28 | locale使用稳定topic/fragment ID，source变化使translation stale |
| G29 | 图片与图表检查alt、license、digest、size和publication class |
| G30 | 生成HTML启用sanitize/CSP/integrity并固定generator/toolchain |
| G31 | schema migration支持dry run、deterministic diff、rollback和receipt |
| G32 | source rename transaction同步更新graph、redirect与currentness |
| G33 | evidence raw artifact按retention tier保存，站点只投影摘要与可信链接 |
| G34 | publication metrics包含duration/count/warning/failure/size且不可自报成功 |
| G35 | 全量validator有单元、fixture、integration和clean-checkout CI证明 |
| G36 | 同一revision重建文档artifact可复现，差异只能来自声明的非确定输入 |

## 18. 与既有报告的责任边界

| 依赖报告 | 本篇消费 | 仍由原报告拥有 |
|---|---|---|
| Tooling01 | BuildSet、CI lane、rustdoc执行入口 | 通用Cargo/toolchain/runner/release CI |
| Tooling12 | EvidenceReceipt、archive/fixture分离 | 顶层acceptance历史迁移与fixture provenance |
| Tooling13 | FindingSet、结构审计与required exit政策 | Codex skill/hook、权限、audit engine通用治理 |
| Tooling17 | SourceArchive、content class物理存储与license | Git source/generated/vendor/evidence集合和分发完整性 |
| Tooling20 | Cargo package graph与package identity | workspace feature/dependency/target graph |
| Tooling27 | Version Domain、support window、deprecation | 全产品schema/API/protocol兼容与迁移策略 |
| O00/O11 | capability claim和validation evidence | 事实是否可发布，不由文档正文自行决定 |
| O01/O03/O14 | BuildSet、version/schema、build/publish operation | 文档控制面只消费共享身份并产生Documentation receipt |

边界规则：

1. Tooling28可以报告required docs gate为红并拥有文档内容修复队列，但不复制Tooling13的通用audit engine P0。
2. Tooling28可以要求`cargo doc`与package documentation identity，但通用CI可达性和Cargo package admission仍由Tooling01/20拥有。
3. Tooling28定义archive是否进入publication graph，不改写Tooling12的历史证据provenance和retention事实。
4. Tooling28定义page version selector和currentness，不重新定义Tooling27的Engine/API/Schema支持窗口。
5. Tooling28不以“文档写了某feature”证明O00 capability available；必须链接实际provider、artifact和EvidenceSet。

## 19. 本轮产出与限制

本轮只新增审查文档并更新总索引，不修复692项违规，不新增README，不运行rustdoc，不修改CI，也不调整相邻session正在编辑或缺失的plan文件。已知Editor、Hub、WOC与plugin动态阻断保持原状态。29个fingerprint输入在取证时均为Git clean；后续任一输入变化都要求重新检查本报告的数量、路径、控制流和优先级。

在G01至G36完成以前，Zircon拥有的是大规模文档素材和若干局部门禁，不是可版本化、可检索、可验证、可退役的工程级知识产品。更不能用文档篇数、计划数量或一次validator通过来宣称功能完整、性能领先或达到Unreal级工程成熟度。
