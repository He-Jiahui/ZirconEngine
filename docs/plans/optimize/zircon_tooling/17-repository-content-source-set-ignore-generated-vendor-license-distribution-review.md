---
related_code:
  - .gitignore
  - .rustc_info.json
  - Cargo.toml
  - Cargo.lock
  - deny.toml
  - LICENSE
  - .github/workflows/ci.yml
  - .github/workflows/mvp-editor-windows.yml
  - tools/mvp/MvpProductInputManifest.psm1
  - tools/session_coordinator/baselines.py
  - tools/session_coordinator/artifact_receipts.py
  - tools/session_coordinator/web/package.json
  - tools/session_coordinator/web/package-lock.json
  - tools/session_coordinator/web/dist
  - tools/session_tray/dist
  - zircon_hub/package.json
  - zircon_hub/package-lock.json
  - examples/woc/LICENSES.md
  - examples/woc/contracts/m8_assets.json
  - examples/woc/assets/m8/licenses/source-CREDITS.md
  - examples/woc/tools/package.json
  - examples/woc/tools/package-lock.json
  - examples/vampire/LICENSES.md
  - examples/vampire/assets/models/kenney_graveyard/KENNEY_GRAVEYARD_LICENSE.txt
  - zircon_plugins/navigation/native/NOTICE.md
  - zircon_plugins/navigation/native/vendor/recastnavigation/License.txt
  - zircon_plugins/rendering/features/contact_shadow/runtime/.zircon-cache/shader_variants/v1/11/1171588ec74b24db4d0fa1713feaa266d70f856d5a50466b4d91de94ae01a58e.meta
  - zircon_plugins/rendering/features/contact_shadow/runtime/.zircon-cache/shader_variants/v1/11/1171588ec74b24db4d0fa1713feaa266d70f856d5a50466b4d91de94ae01a58e.wgsl.zst
tests:
  - tools/tests/mvp-product-inputs.Tests.ps1
  - tools/session_coordinator/tests/test_baselines.py
  - tools/session_coordinator/tests/test_artifact_receipts.py
  - tools/session_coordinator/tests/test_manifest_retention.py
plan_sources:
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_app/03-woc-product-role-host-zrvm-transaction-state-client-server-integration-review.md
  - docs/plans/optimize/zircon_app/06-vampire-roguelite-example-project-asset-script-gameplay-evidence-product-integration-review.md
  - docs/plans/optimize/zircon_tooling/01-workspace-toolchain-ci-validation-and-developer-entrypoints-review.md
  - docs/plans/optimize/zircon_tooling/05-woc-content-codegen-build-scripts-generated-artifact-incremental-review.md
  - docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md
  - docs/plans/optimize/zircon_tooling/07-performance-benchmark-profile-capture-symbol-crash-evidence-baseline-review.md
  - docs/plans/optimize/zircon_tooling/09-release-channel-artifact-repository-install-update-rollback-operations-review.md
  - docs/plans/optimize/zircon_tooling/12-top-level-acceptance-archive-serialization-fixture-provenance-currentness-migration-review.md
  - docs/plans/optimize/zircon_tooling/13-repository-codex-skill-hook-structural-audit-governance-security-currentness-review.md
  - docs/plans/optimize/zircon_tooling/15-mvp-build-staging-product-process-acceptance-evidence-resource-baseline-control-plane-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Extras/ThirdPartyNotUE
  - dev/UnrealEngine/Engine/Source/Programs/UnrealBuildTool/System/TargetReceipt.cs
  - dev/bevy/.gitattributes
  - dev/bevy/.gitignore
  - dev/bevy/LICENSE-APACHE
  - dev/bevy/LICENSE-MIT
  - dev/bevy/CREDITS.md
  - dev/bevy/Cargo.toml
  - dev/godot/.gitattributes
  - dev/godot/.gitignore
  - dev/godot/LICENSE.txt
  - dev/godot/COPYRIGHT.txt
  - dev/godot/thirdparty/README.md
  - dev/Fyrox/.gitattributes
  - dev/Fyrox/.gitignore
  - dev/Fyrox/LICENSE.md
  - dev/Fyrox/Cargo.toml
  - dev/Graphics/.gitattributes
  - dev/Graphics/.gitignore
  - dev/Graphics/Packages/com.unity.render-pipelines.core/package.json
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Third Party Notices.md
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Tooling 17：Repository Content、SourceSet、Ignore、Generated/Vendor、License 与分发完整性审查

## 1. 结论

本轮审查根级Git/Cargo/license控制文件、当前全部tracked path、tracked-but-ignored集合、跨语言package metadata、checked-in generated/vendor/evidence、MVP与Session Coordinator的source枚举、CI license gate，以及WOC/Vampire/navigation的notice/provenance。当前仓库有27,432个tracked path、checkout字节约1,335,024,765；其中2,849个文件、93,877,865 bytes同时被当前`.gitignore`匹配。这个状态本身不丢失已跟踪修改，但会让同一owner下新增源码、asset、license、fixture或skill在普通`git status`、MVP fingerprint和Coordinator baseline中静默消失。

根因是Git ignore被迫承担了四个不同职责：本机垃圾排除、source membership、evidence retention和product asset selection。MVP与Coordinator均用`git ls-files --others --exclude-standard`纳入untracked input，所以ignore规则事实上进入BuildSet信任边界。根规则`examples/*`命中全部1,967个tracked WOC文件，`examples/vampire/*`命中172个tracked Vampire文件，`/.codex/*`命中242个tracked repo-control文件，`*.log`/`*.txt`命中461个历史证据和5个vendored/license文件，`.zircon-cache/`还命中2个checked-in contact-shadow shader cache artifact。一个新文件是否属于产品source由文件名和历史`git add -f`决定，不由versioned content manifest决定。

分发许可闭包也不成立。159个Cargo package全部通过workspace声明`MIT OR Apache-2.0`，但仓库根只提供MIT文本，没有Apache-2.0文本；WOC `LICENSES.md`明确说`source-LICENSE.txt`已materialize，当前文件确实存在于本机，却被`examples/*`忽略且不在Git tree；仓库没有根级NOTICE/THIRD_PARTY_NOTICES或按产品生成的notice bundle。`cargo deny`能检查依赖声明和source policy，不能证明实际产品包携带了应分发文本、asset授权、native vendor notice和SBOM。本报告不提供法律意见，只记录机器可验证的声明/文件/产品包不闭合。

仓库也有可保留基础：Cargo package全部有license字段，`deny.toml`拒绝未知registry/git且advisory零忽略；WOC asset manifest记录hash/bytes/role/license id，Vampire和navigation有局部license/notice；4个可执行Node项目有lockfile且5个package均为private；27,432个tracked path没有case-fold collision、Windows非法segment、特殊Git mode或当前workspace绝对路径>=240字符。这些正向机制应迁移进Repository Content Manifest，而不是被一刀清除。

本篇登记 **3项P0、54项P1、12项P2**。它只拥有RepositoryContentManifest、SourceSet/SourceArchive、checked-in generated/vendor分类、license/notice closure和package publication policy；Cargo workspace/toolchain由Tooling01拥有，WOC generator由Tooling05拥有，Coordinator artifact由Tooling06拥有，历史证据由Tooling07/12拥有，release artifact由Tooling09拥有，repo-local Codex由Tooling13拥有，immutable BuildSet由Tooling15拥有。本轮没有修改ignore、license、cache、source、test或workflow。

## 2. 物理清单与动态证据

### 2.1 Repository content规模

| 根 | tracked files | checkout bytes | 本轮解释 |
|---|---:|---:|---|
| `docs` | 5,327 | 1,148,469,890 | 大量历史evidence；由Tooling07/12拥有retention |
| `examples` | 2,140 | 82,491,193 | WOC 1,967、Vampire 173；两者均被宽ignore覆盖 |
| `zircon_runtime` | 7,781 | 43,706,660 | production source，由Runtime报告拥有 |
| `zircon_editor` | 6,412 | 26,015,079 | production/source/assets，由Editor报告拥有 |
| `tools` | 1,327 | 16,086,900 | source、lock与checked-in web dist混合 |
| `zircon_plugins` | 3,140 | 10,205,215 | source、vendor、notice与cache artifact混合 |
| 其余根合计 | 1,305 | 8,049,828 | App/Hub/interface/host/template/control files |
| **合计** | **27,432** | **1,335,024,765** | Git tree不是自动合格的source或release archive |

没有`.gitattributes`，因此没有line-ending/binary/export-ignore/LFS policy；没有`.gitmodules`、symlink或submodule mode。若直接`git archive HEAD`，默认会包含全部tracked evidence/cache/dist，而不会按Source/Fixture/Evidence/Vendor/Generated/Release分类。

### 2.2 Tracked-but-ignored集合

| 根/规则 | files | bytes | 风险 |
|---|---:|---:|---|
| `examples` / `examples/*`与Vampire子规则 | 2,139 | 82,483,176 | 新脚本、asset、license默认不进入source fingerprint |
| `docs` / `*.log`、`*.txt` | 461 | 9,820,382 | 历史证据新增项被文件扩展名决定retention |
| `.codex` / `/.codex/*` | 242 | 1,559,149 | 新skill/hook/rule默认不可见，Tooling13控制面可漂移 |
| `zircon_plugins` / `*.txt`与`.zircon-cache` | 7 | 15,158 | vendor license/CMake和generated shader cache混合 |
| **合计** | **2,849** | **93,877,865** | 已跟踪内容靠历史force-add存活，新增同类内容fail-open消失 |

代表性`git check-ignore --no-index -v`结果精确落在`.gitignore:71/116/124/126/128/130`。当前MVP source fingerprint读取tracked diff及`ls-files --others --exclude-standard`；Coordinator baseline构造与增量候选也使用`--exclude-standard`。因此ignored untracked文件即使被编译器、脚本或产品读取，也不会进入这两条source receipt。

### 2.3 Package与license元数据

| 检查 | 结果 | 解释 |
|---|---:|---|
| Cargo manifests/packages | 162 / 159 | 三个workspace root无`[package]` |
| Cargo package license | 159/159 | 全部继承`MIT OR Apache-2.0` |
| Cargo repository/readme | 0/159、0/159 | 发布消费者不能从package metadata追溯source/docs |
| Cargo explicit publish policy | 2/159 | 仅`zr_rhi`和`zr_rhi_wgpu`为`publish=false` |
| Node package | 5/5 private | private是正向边界，但均无license/repository/engines/packageManager |
| Node lockfile | 4/5 | docs Workbench prototype无lock；domain由Tooling14拥有 |
| root dual-license texts | MIT 1 / Apache 0 | manifest选项与可分发文本不闭合 |
| local notice roots | WOC、Vampire、navigation | 没有根级/产品级聚合和artifact inclusion gate |

WOC声明的`examples/woc/assets/m8/licenses/source-LICENSE.txt`在当前机器存在，但`git ls-files --error-unmatch`失败，且`git check-ignore`指向`examples/*`。这说明当前工作树可表现出比clean clone更完整的许可材料，不能用本机存在性证明source或产品分发闭包。

### 2.4 Generated、cache与dist边界

1. 根`.rustc_info.json`是354-byte Cargo/rustc探测缓存，记录本机rustc 1.94.1、commit、host和LLVM；没有schema owner/currentness，却作为普通tracked source进入Git tree。
2. contact-shadow runtime跟踪一对`.zircon-cache/shader_variants` meta/zstd artifact；没有发现按其固定hash消费的source owner。Shader key/并发正确性由Tooling07拥有，本篇只判定content class未声明。
3. Session Coordinator跟踪29个hashed web dist文件、949,104 bytes，tray另跟踪一个dist HTML；source与bundle并存，但没有checked-in bundle manifest绑定package-lock、Node/tool版本、build command和source hashes。Tooling06拥有其产品加载/更新。
4. `docs/tests`的历史日志/图像是证据，不应因扩展名规则被当作source；同样不能在source archive中无条件携带1.148 GB docs tree。
5. WOC generated Zr/ZRO、asset closure和reference catalogs是产品输入还是可再生artifact，当前只能由路径惯例推断；Tooling05拥有generator transaction，本篇要求明确content class。

## 3. P0：先恢复SourceSet与分发闭包真实性

### TOOL-REPOCONTENT-P0-001 · Git ignore进入BuildSet信任边界，required新source可以完全不被fingerprint观察

MVP和Coordinator均将`--exclude-standard`后的untracked列表当作live source universe；根ignore又覆盖整个WOC、Vampire、`.codex`、所有`.txt/.log`和任意`.zircon-cache`。当前tracked文件仍因index身份被看到，但同目录新增的Zr module、asset sidecar、license、hook或generated contract可以被产品读取而不改变source fingerprint，也不会出现在普通未跟踪变更清单。Tooling05已发现clean clone缺WOC入口，Tooling15已证明fingerprint不是snapshot；本篇拥有的硬切是禁止ignore规则定义SourceSet membership。

建立versioned RepositoryContentManifest，按owner显式列根/模式与class；BuildSet只消费manifest解析出的冻结SourceSet，并对“被实际read但不在SourceSet”的输入fail closed。ignore只处理本机未拥有输出，不再作为产品source排除政策。

### TOOL-REPOCONTENT-P0-002 · 声明的license/notice与Git tree及产品artifact不闭合

全部Cargo package声明`MIT OR Apache-2.0`，Git tree只有MIT文本；WOC文档声称materialized的source license不在Git tree；局部asset/native notice没有根级NoticeGraph和按产品聚合器。CI `cargo deny`检查依赖元数据，不检查root dual-license文本、asset license closure、native vendored notice或最终package内容。当前任何source/release“许可材料完整”结论都不可成立。

硬切要求由项目owner确认实际授权策略并补齐匹配文本；构建NoticeGraph，将source/vendor/asset/dependency的license identity、原文hash、obligation和产品使用边连接到PackageReceipt；缺文本、未知授权、声明/文件冲突或未携带required notice时阻断分发。本篇不替代法律审查。

### TOOL-REPOCONTENT-P0-003 · Git tree混合source、generated、vendor、fixture、evidence、cache与developer state，没有可验证SourceArchive

27,432个tracked path既含production source，也含1.148 GB docs/evidence、93.9 MB tracked-ignored内容、本机rustc缓存、shader cache、generated web bundle和vendored native code。仓库没有`.gitattributes`/export policy、content class schema、generator receipt、retention或source archive workflow。Git commit能固定字节，却不能证明哪些字节是构建输入、哪些可再生、哪些必须随source/release分发，也不能证明checked-in generated产物与source同代。

建立`SourceArchiveReceipt`，绑定Git tree、RepositoryContentManifest、生成器/工具链、vendor/notice closure、included/excluded tree manifest和archive hash；source archive、SDK、plugin source bundle与release artifact采用不同profile。未分类path和tracked cache/developer state阻断archive promotion。

## 4. P1：Repository Content控制面重构

### 4.1 Content identity、class与Git policy

1. **TOOL-REPOCONTENT-P1-001**：定义`RepositoryPathId`和规范UTF-8 slash路径；identity大小写规则按目标文件系统显式记录。
2. **TOOL-REPOCONTENT-P1-002**：定义`ContentClass::{Source, GeneratedSource, Vendor, Fixture, Evidence, Reference, ReleaseAsset, DeveloperState}`，禁止仅靠目录名推断。
3. **TOOL-REPOCONTENT-P1-003**：每条manifest规则携带owner、consumer、required profile、generator、retention、export和license policy。
4. **TOOL-REPOCONTENT-P1-004**：RepositoryContentManifest规则必须无重叠歧义；同一路径多class时要求显式优先级并由validator报出。
5. **TOOL-REPOCONTENT-P1-005**：`SourceSetId`由排序path、mode、content digest和manifest version计算，不以Git ignore结果代替。
6. **TOOL-REPOCONTENT-P1-006**：Build reader通过sandbox/read audit或声明input graph验证实际读取集合是SourceSet子集。
7. **TOOL-REPOCONTENT-P1-007**：tracked、untracked、ignored和generated四种Git状态只作为诊断维度，不决定业务class。
8. **TOOL-REPOCONTENT-P1-008**：将`examples/*`改为精确local-output规则；WOC/Vampire产品source不得依赖历史force-add。
9. **TOOL-REPOCONTENT-P1-009**：将`/.codex/*`改为明确state/cache排除；repo-owned skill/hook/rule由Tooling13 manifest纳入。
10. **TOOL-REPOCONTENT-P1-010**：取消全局`*.txt/*.log`业务政策，按evidence/temp目录和owner声明排除，license/fixture文本默认可见。
11. **TOOL-REPOCONTENT-P1-011**：`.zircon-cache`只允许DeveloperState/DerivedData；若需golden fixture则迁入fixture root并绑定digest/provenance。
12. **TOOL-REPOCONTENT-P1-012**：validator对tracked-but-ignored、manifest-unclassified和ignored-but-read path fail closed，并输出规则来源。

### 4.2 Generated、vendor、fixture与evidence

13. **TOOL-REPOCONTENT-P1-013**：GeneratedSource记录generator binary/source digest、argv、environment allowlist、input tree和determinism version。
14. **TOOL-REPOCONTENT-P1-014**：checked-in generated文件必须有reproduce-and-diff gate；生成后dirty tree或缺文件均失败。
15. **TOOL-REPOCONTENT-P1-015**：`.rustc_info.json`移出source class并由Cargo target policy管理，不携带开发机探测结果进入archive。
16. **TOOL-REPOCONTENT-P1-016**：contact-shadow cache pair迁出production source或转为具名golden fixture，禁止DerivedData直接跟踪。
17. **TOOL-REPOCONTENT-P1-017**：checked-in web dist生成bundle manifest，绑定source/lock/Node/package-manager/build command及每文件hash。
18. **TOOL-REPOCONTENT-P1-018**：web dist发布采用新generation目录和原子指针，避免source与旧hashed chunks混合。
19. **TOOL-REPOCONTENT-P1-019**：vendor entry记录upstream URL/commit/tag、patch series、license hash、更新policy和安全owner。
20. **TOOL-REPOCONTENT-P1-020**：Recast窄扩展形成可重放patch，不只在NOTICE中描述“有修改”。
21. **TOOL-REPOCONTENT-P1-021**：Fixture与Evidence分class；fixture是测试输入，evidence是运行输出，二者currentness/retention完全不同。
22. **TOOL-REPOCONTENT-P1-022**：大文件使用CAS/LFS或artifact repository policy；Git tree只保留manifest与小型canonical fixture，具体迁移由Tooling07/12拥有。

### 4.3 License、notice与provenance

23. **TOOL-REPOCONTENT-P1-023**：由owner确认MIT-only或MIT/Apache dual-license，manifest、根文本、package metadata和发布页面必须一致。
24. **TOOL-REPOCONTENT-P1-024**：每个license text赋稳定LicenseTextId与SHA-256，禁止只靠自由文本label判定闭包。
25. **TOOL-REPOCONTENT-P1-025**：WOC `source-LICENSE.txt`进入受控Git/generated artifact closure，文档不能声称本机only文件已分发。
26. **TOOL-REPOCONTENT-P1-026**：asset manifest的license id解析到实际文本或明确外部授权receipt；未知/商业授权不得只留描述。
27. **TOOL-REPOCONTENT-P1-027**：商业asset授权receipt不泄露凭据，但需记录owner、scope、产品、地域/平台限制、到期/撤销政策。
28. **TOOL-REPOCONTENT-P1-028**：native vendor、Rust、Node、font、audio、model、texture和reference-derived source进入统一NoticeGraph。
29. **TOOL-REPOCONTENT-P1-029**：source archive、Editor/Runtime/Hub安装包、plugin SDK和example bundle各自生成精确notice subset。
30. **TOOL-REPOCONTENT-P1-030**：notice generator验证原文未被normalize/truncate，保留upstream attribution和本地patch说明。
31. **TOOL-REPOCONTENT-P1-031**：dependency SBOM与asset/vendor NoticeGraph通过PackageReceipt关联，不把Cargo-only SBOM称为产品SBOM。
32. **TOOL-REPOCONTENT-P1-032**：license/notice waiver具备owner、法律review reference、scope、期限和撤销原因。
33. **TOOL-REPOCONTENT-P1-033**：CI检查manifest声明、文本存在、hash、产品closure和archive inclusion；`cargo deny`继续作为依赖层子门。
34. **TOOL-REPOCONTENT-P1-034**：新增/升级vendor或asset先通过provenance admission，再允许generator/cook消费。

### 4.4 Package metadata与publication policy

35. **TOOL-REPOCONTENT-P1-035**：所有内部Cargo package默认`publish = false`，仅经ReleaseProvider登记的包显式开放。
36. **TOOL-REPOCONTENT-P1-036**：可发布crate补repository、homepage/readme、documentation、rust-version和owner metadata。
37. **TOOL-REPOCONTENT-P1-037**：159个package不能永久共享`0.1.0`而无兼容政策；版本由package family和ABI/schema变化驱动。
38. **TOOL-REPOCONTENT-P1-038**：Cargo package include/exclude显式列source/license/readme/generated requirements，并用`cargo package --list`验收。
39. **TOOL-REPOCONTENT-P1-039**：Node private package固定`packageManager`与Node `engines`，lockfile生成环境进入ToolchainSet。
40. **TOOL-REPOCONTENT-P1-040**：Node package license字段与是否分发区分建模；private不能替代第三方dependency notice。
41. **TOOL-REPOCONTENT-P1-041**：docs prototype若保留为可执行工具，补lock或改为明确non-product artifact并由Tooling14治理。
42. **TOOL-REPOCONTENT-P1-042**：跨workspace package name/version/source digest进入Release Catalog，防止同名不同root发布。

### 4.5 Archive、平台与Git字节语义

43. **TOOL-REPOCONTENT-P1-043**：新增`.gitattributes`，明确LF/CRLF文本、binary、merge策略、LFS/CAS和`export-ignore`。
44. **TOOL-REPOCONTENT-P1-044**：line-ending normalization变更先做一次有审计的tree migration，避免把全仓字节漂移混入功能提交。
45. **TOOL-REPOCONTENT-P1-045**：archive path验证case-fold collision、Windows reserved/invalid/trailing字符、Unicode normalization和目标path limit。
46. **TOOL-REPOCONTENT-P1-046**：symlink/submodule若未来引入，manifest记录link target/commit并按目标平台政策验证；当前为零是正向基线。
47. **TOOL-REPOCONTENT-P1-047**：source archive固定mtime、uid/gid、mode、entry order和compression实现，支持byte-for-byte复现。
48. **TOOL-REPOCONTENT-P1-048**：archive extraction做path traversal、case collision、symlink escape和disk quota预检。

### 4.6 CI、release与currentness

49. **TOOL-REPOCONTENT-P1-049**：required CI运行tracked-ignored/unclassified/ignored-read/reproduce-generated四类content gate。
50. **TOOL-REPOCONTENT-P1-050**：clean checkout lane不挂载本机ignored overlay，验证WOC/Vampire/skills/license真实闭包。
51. **TOOL-REPOCONTENT-P1-051**：SourceArchive build从commit/tree object读取，不从可变worktree事后采样；绑定Tooling15 BuildSet。
52. **TOOL-REPOCONTENT-P1-052**：release gate解包实际artifact，核对SBOM/notice/license/source-offer与PackageReceipt，不只检查仓库文件。
53. **TOOL-REPOCONTENT-P1-053**：content manifest和notice graph支持currentness；vendor、license或generator变化使旧receipt失效。
54. **TOOL-REPOCONTENT-P1-054**：建立repository size、large blob、generated churn和archive closure预算，但预算超限不得自动删除canonical source。

## 5. P2：可观测性与开发体验

1. **TOOL-REPOCONTENT-P2-001**：提供`repo content explain <path>`，显示class、owner、ignore rule、archive/profile和license edge。
2. **TOOL-REPOCONTENT-P2-002**：pre-commit提示新增ignored-but-owned文件，并给出正确manifest修改入口，不建议盲目`git add -f`。
3. **TOOL-REPOCONTENT-P2-003**：生成tracked-but-ignored趋势图，按root/rule/bytes展示收敛进度。
4. **TOOL-REPOCONTENT-P2-004**：source archive生成可浏览tree manifest与大小top list，便于review异常大文件。
5. **TOOL-REPOCONTENT-P2-005**：NoticeGraph输出人类可读第三方声明和机器可读SPDX/CycloneDX投影。
6. **TOOL-REPOCONTENT-P2-006**：package metadata由workspace模板生成并允许少量显式override，避免159份手工漂移。
7. **TOOL-REPOCONTENT-P2-007**：vendor更新工具生成upstream diff、license diff和安全影响摘要。
8. **TOOL-REPOCONTENT-P2-008**：generated diff在review中关联输入/generator变化，避免审查者阅读无来源的大块输出。
9. **TOOL-REPOCONTENT-P2-009**：archive/notice receipt可由Hub与Editor About页查询，但UI不拥有source truth。
10. **TOOL-REPOCONTENT-P2-010**：建立跨Windows/Linux/macOS的checkout/archive smoke，记录实际filesystem语义。
11. **TOOL-REPOCONTENT-P2-011**：对license label、URL和text hash做去重，保留别名而不复制多份漂移文本。
12. **TOOL-REPOCONTENT-P2-012**：文档索引链接到canonical content owner和迁移状态，避免README成为第二manifest。

## 6. 目标架构

### 6.1 Repository Content Manifest

```text
Git tree + explicit owned overlay
            |
            v
RepositoryContentManifest --classify--> Source / Generated / Vendor / Fixture
            |                            Evidence / Reference / Release / State
            v
       Frozen SourceSet ------> BuildSet / Generator Actions
            |
            +------> SourceArchiveReceipt
            +------> NoticeGraph ------> Product Notice Bundle
```

manifest规则至少包含path matcher、owner、class、required profiles、consumer、generator/action digest、license/provenance、retention、archive inclusion和maximum size。Git tracked/ignored状态作为validator输入而非class source。最终SourceSet必须是冻结tree，不是在构建前后重复读取活动worktree。

### 6.2 NoticeGraph

NoticeGraph节点包括ProjectSource、CargoDependency、NodeDependency、NativeVendor、Asset、Font、Audio、GeneratedCombination与LicenseText；边记录使用、派生、组合、修改和分发。ProductReceipt列实际包含的artifact/content identity，notice generator据此求闭包，不能从整个repository依赖清单猜测。

### 6.3 Archive profiles

至少区分Developer Source、Public Source、Plugin SDK Source、Example Source、Evidence Export和Installed Product。每个profile有精确include/exclude、许可闭包、大小预算、重现方式和安全策略；`git clone`、`git archive`、Cargo package和产品zip不是同一种artifact。

## 7. 与既有报告的非重复边界

| 事实 | Canonical owner | 本篇只拥有 |
|---|---|---|
| 双workspace、lock、MSRV、cargo-deny | Tooling01 | package publication metadata与content分发闭包 |
| WOC ignored生成输入、clean clone缺文件 | Tooling05、App03 | ignore不得定义SourceSet与WOC license进入archive |
| Coordinator baseline/artifact/Web | Tooling06 | repository content class和checked-in bundle receipt |
| 大量历史PNG/RDC/log retention | Tooling07、Tooling12 | Evidence与SourceArchive分class |
| release repository/install/rollback | Tooling09 | notice/source archive作为PackageReceipt输入 |
| `.codex`权限、skill/hook currentness | Tooling13 | repo-owned control files不应被宽ignore隐藏 |
| mutable worktree fingerprint/ProductReceipt | Tooling15 | explicit SourceSet和ignored-read admission |
| Vampire asset/runtime/evidence缺口 | App06 | asset/license closure进入NoticeGraph |

## 8. 分层实施路线

### M0 · 冻结错误分发声明

- 不再宣称当前source/release license closure完整。
- 导出27,432 path的content classification候选、2,849 tracked-ignored账本和missing WOC license事实。
- required gate先禁止新增unclassified ignored source、tracked cache和无notice vendor。

### M1 · Manifest与Git policy

- 建RepositoryContentManifest和validator，按owner确认Source/Generated/Vendor/Fixture/Evidence等class。
- 精确收敛`examples/*`、`.codex/*`、`*.txt/*.log`、`.zircon-cache`规则。
- 引入`.gitattributes`并完成一次有审计的normalization migration。

### M2 · SourceSet与generated/vendor receipts

- MVP/Coordinator改消费冻结SourceSet，不再用ignore枚举定义业务输入。
- WOC/web dist/shader等checked-in generated接reproduce-and-diff；vendor接upstream/patch/license receipt。
- developer cache从Git tree移除，canonical fixture迁入具名root。

### M3 · License与NoticeGraph

- owner确认root license策略并补齐文本，修复WOC materialized license漂移。
- 汇聚Cargo/Node/native/asset/font/audio notice与授权receipt。
- 为source/example/plugin SDK/产品生成不同notice bundle。

### M4 · Package与archive

- 内部package默认不可发布，可发布包补metadata/include policy。
- 构建可复现SourceArchive与PackageReceipt，执行解包/notice/source-offer检查。
- 大evidence迁移依赖Tooling07/12 CAS与retention实现。

### M5 · Promotion与运营

- CI从clean tree验证content、generated、vendor、notice、archive与平台可移植性。
- ReleaseProvider只promote绑定SourceSet/Archive/Notice/SBOM receipt的artifact。
- 持续监控size、ignored debt、vendor/license currentness和generated drift。

## 9. 验收门

1. 全部tracked与所有实际build-read path均有唯一ContentClass和owner，0 unclassified/ambiguous。
2. required source新增后即使匹配ignore也使gate失败，不能被MVP/Coordinator fingerprint漏掉。
3. WOC/Vampire clean checkout不依赖本机ignored overlay；源码、asset sidecar和required license完整。
4. root package license声明与实际提供文本由owner确认且一致。
5. 每个产品/SDK/source archive的NoticeGraph闭包可重建，缺文本/未知授权fail closed。
6. checked-in generated source/dist可由固定toolchain重建且tree diff为零。
7. cache、developer state和历史evidence不进入SourceSet；必要fixture有独立identity/provenance。
8. internal Cargo package默认`publish=false`；可发布包通过`cargo package --list`和安装smoke。
9. Node package固定Node/package-manager，lock与bundle receipt同代。
10. SourceArchive固定entry/mode/mtime/order/compression并可byte-for-byte重建。
11. Windows/Linux/macOS检查无case/Unicode/reserved/path/symlink问题；当前零碰撞基线保持。
12. 实际release artifact解包后包含精确license/notice/SBOM/source-offer，不以仓库存在性替代。
13. ignored debt、large blob和generated churn有预算与owner，但工具不会擅自删除canonical source。
14. source、archive、package与notice receipt绑定同一BuildSet，旧generation/currentness不能复用。

## 10. 本轮限制与下一步

本轮执行的是Git索引、路径、文件大小、package metadata、ignore解释和source-enumerator静态/只读探测，没有运行`git archive`生成1.3 GB tar，也没有运行Cargo/Node package或产品release。没有修改`.gitignore`或补license文本，因为实际授权选择、archive profile和大文件迁移必须由owner确认并分层实施。

下一步应先做M0 machine-readable inventory和P0-001 gate：用当前Git tree生成候选manifest，但不得把现有ignore规则自动抄成source policy。随后由Tooling01/05/06/07/09/12/13/15各owner确认其path class，再实施license/notice和archive闭包。
