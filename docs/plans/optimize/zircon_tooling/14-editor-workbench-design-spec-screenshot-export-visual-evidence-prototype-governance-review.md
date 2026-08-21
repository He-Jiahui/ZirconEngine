---
related_code:
  - tools/editor-workbench-preview/app.js
  - tools/editor-workbench-preview/design-manifest.mjs
  - tools/editor-workbench-preview/design.css
  - tools/editor-workbench-preview/design.html
  - tools/editor-workbench-preview/design.js
  - tools/editor-workbench-preview/export-designs.mjs
  - tools/editor-workbench-preview/export-evidence.mjs
  - tools/editor-workbench-preview/export-options.mjs
  - tools/editor-workbench-preview/index.html
  - tools/editor-workbench-preview/package-lock.json
  - tools/editor-workbench-preview/package.json
  - tools/editor-workbench-preview/preview-sheet.js
  - tools/editor-workbench-preview/server.mjs
  - tools/editor-workbench-preview/styles.css
  - tools/editor-workbench-preview/verify-designs.mjs
  - tools/editor-workbench-preview/verify-reference-negative-guard.mjs
  - docs/ui-and-layout/editor-workbench-design-export.md
  - docs/ui-and-layout/editor-workbench-designs
  - docs/ui-and-layout/editor-workbench-designs/EXPORT-EVIDENCE.json
  - docs/ui-and-layout/editor-workbench-designs/STYLE-NOTES.md
  - docs/ui-and-layout/workbench.png
  - zircon_editor/assets/ui/editor/reference/workbench.png
  - zircon_editor/fixtures/workbench/default-layout.json
  - zircon_editor/fixtures/workbench/editor-data.json
  - zircon_editor/fixtures/workbench/view-descriptors.json
  - zircon_editor/fixtures/workbench/view-instances.json
  - zircon_editor/src/ui/workbench/fixture/constants.rs
  - .github/workflows/ci.yml
tests:
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/layout_routes.rs
  - zircon_editor/tests/integration_contracts/workbench_retained_shell.rs
  - zircon_editor/tests/integration_contracts/workbench_window_template.rs
plan_sources:
  - docs/plans/performance/01/fixed-2026-08-02-workbench-design-export-freshness.md
  - docs/plans/zircon_editor/editor_layout/index.md
  - docs/plans/zircon_editor/editor_ui/09-editor-modules-and-design-parity.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Developer/FunctionalTesting/Public/AutomationScreenshotOptions.h
  - dev/UnrealEngine/Engine/Source/Developer/FunctionalTesting/Private/ScreenshotFunctionalTest.cpp
  - dev/UnrealEngine/Engine/Source/Developer/ScreenShotComparison/Private/Models/ScreenComparisonModel.cpp
  - dev/UnrealEngine/Engine/Source/Developer/ScreenShotComparison/Private/Widgets/SScreenShotBrowser.cpp
  - dev/Graphics/TestProjects/PostProcessing_Tests/Assets/CommonAssets/Scripts/PostProcessingGraphicsTests.cs
  - dev/Graphics/TestProjects/PostProcessing_Tests/Assets/CommonAssets/Scripts/PostProcessingGraphicsTestSettings.cs
  - dev/Graphics/.yamato/postprocessing-win-dx12.yml
  - dev/Graphics/.yamato/postprocessing-linux-vulkan.yml
  - dev/Graphics/.yamato/all-postprocessing.yml
  - dev/bevy/tools/example-showcase/src/main.rs
  - dev/bevy/.github/workflows/send-screenshots-to-pixeleagle.yml
  - dev/Fyrox/editor/src/asset/preview/cache.rs
  - dev/Fyrox/editor/src/preview.rs
  - dev/godot/editor/editor_node.cpp
  - dev/godot/editor/run/game_view_plugin.cpp
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Tooling 14：Editor Workbench DesignSpec、截图导出、视觉证据与 Prototype 治理审查

## 1. 结论

`tools/editor-workbench-preview`不是一个小型截图脚本，而是当前Editor视觉计划的事实控制面：16个tracked文件共15,483行、692,040 bytes，驱动270个design entry、271张PNG和一份约26.37 MB的tracked输出集。`docs/plans/zircon_editor/editor_layout/index.md`又把这些PNG称为布局与设计语言的视觉权威，多份Editor计划直接以其中的layout/state/content截图定义后续实现目标。因此它必须按DesignSpec、baseline审批、artifact publication和产品可追溯性系统审查，不能只按“能打开的HTML草图”处理。

本轮确认现有工具保留了若干有价值机制：Playwright版本由lockfile固定；manifest与renderer ID/output做双向检查；完整导出才写`EXPORT-EVIDENCE.json`；文本输入做LF canonicalization；输出、style note与部分source计算SHA-256；unknown selection fail closed；双reference PNG做byte identity；Windows会检查遗留preview进程。270个页面在真实Edge加载时没有page error、没有viewport overflow，10个JS/MJS文件也全部通过`node --check`。这些机制可以迁入正式Visual Evidence Service，不需要全部删除。

但当前证据结论不能成立。2026-08-16在dirty状态为空的tracked current source上运行`npm run design:verify`，15.8秒后因`view-descriptors.json` digest不匹配退出1：evidence仍记录4,311 canonical bytes与`CF50...9918`，当前文件为4,134 bytes与`BFE3...4832`。更关键的是，固定截图入口`design.html/design.js/design.css`根本不请求四份workbench fixture或31个Ionicon；evidence却把它们作为source dependency，反而不纳入使用fixture的legacy `app.js/index.html/styles.css`。这不是单纯“忘记重导出”，而是artifact dependency graph把两个产品混在一起。

270页浏览器扫描又显示：共72,540个DOM节点、14,775个视觉控件、1,805个原生交互元素，但没有一个`role`/ARIA/tabindex注解；页面中位节点数266，只有140种结构签名，最大同构组含34个名义不同的工程工具。它们是可读的静态设计意图，不是Command、Document、Transaction、Job、Runtime Bridge或Accessibility行为证据。当前manifest有192个`editor-page`，却只有`id/output/kind`，没有capability owner、实现状态、产品入口、artifact、acceptance test或source revision。继续扩充截图数量会扩大“名义覆盖”，不会使Editor更接近Unreal级工程能力。

现有`EXPORT-EVIDENCE`还是self-certifying candidate lock：完整导出会同时重写candidate PNG与其hash，没有独立approved baseline、incoming/diff、reviewer、approval operation或promotion receipt。验证器只证明“当前文件与当前自写清单一致”，再用平均亮度、暗像素比例、亮像素比例、teal比例和采样颜色数拒绝空图；它不能发现按钮语义错误、字段串线、domain workflow缺失、文本截断、错误状态缺失或整体页面被另一个同样深色的静态模板替换。仓库required CI也没有调用`design:verify`。本轮登记 **6项P0、40项P1、12项P2**；所有生产修复仍为pending。

## 2. 物理清单与动态证据

### 2.1 tracked source与artifact规模

| 物理面 | 文件/条目 | 行数或规模 | 事实 |
| --- | ---: | ---: | --- |
| `tools/editor-workbench-preview` | 16 files | 15,483 lines / 692,040 bytes | 10 JS/MJS、2 HTML、2 CSS、package与lockfile |
| `design.js` | 1 | 9,183 lines / 527,876 bytes | 206 full、20 focus与其余spec/window定义；单文件承担registry、domain data、renderer与DOM helper |
| `design.css` | 1 | 3,620 lines / 59,990 bytes | 22 custom-property定义、297个color literal/125种颜色、949个px literal、36个`!important`、0个media query |
| manifest | 270 designs | 6 full、12 focus、8 tool、192 editor-page、8 tool-focus、44 spec/workflow/window | 每项只有`id/output/kind` |
| tracked output | 273 files | 26,370,084 bytes | 271 PNG、`STYLE-NOTES.md`、`EXPORT-EVIDENCE.json`；无Git LFS/filter属性 |
| evidence source set | 46 entries | 11 tool files、31 icons、4 fixtures | 固定design capture不请求icons/fixtures，dependency graph过宽且归属错误 |

行数统一使用portfolio既有的UTF-8 LF record口径；`design.js`函数落点已到9183行，不能再用约8.9K的非空行口径评估模块预算。

### 2.2 current-source命令结果

| 命令/检查 | 结果 | 结论 |
| --- | --- | --- |
| `node --check`全部10个JS/MJS | 10/10通过 | 语法可解析，不代表证据语义正确 |
| `npm run design:verify` | exit 1，15.8秒 | clean tracked source与tracked evidence已漂移；`view-descriptors.json` digest mismatch |
| 270 design Edge DOM扫描 | 270/270加载，无page error、无根viewport overflow | 基础渲染可用 |
| DOM节点 | min 37 / p50 266 / max 516 / total 72,540 | 固定页面体量较大但有界 |
| DOM结构签名 | 140 unique / 18 duplicate groups | 名义domain数显著高于实际layout family数；最大同构组34页 |
| 控件 | 14,775 visual / 1,805 native interactive / 0 explicit a11y annotation | 静态视觉控件未形成交互与accessibility合同 |
| legacy `/` preview | 137 nodes / 12 buttons / 7 draggable / 0 role-ARIA | fixture-driven prototype可加载，但仍是mouse-first内存模型 |
| required CI检索 | 0 invocation | `.github/workflows/ci.yml`、顶层Cargo/tools/tests均未要求`design:verify` |

`design:verify:reference-negative`内部连续运行三次完整verifier；基础verifier已因同一digest漂移失败，因此本轮没有重复执行这个不可能转绿且代价更高的路径。也没有运行`design:export`：它会覆盖271张PNG、style note与evidence，超出review-only授权，并会抹去当前失败现场。

### 2.3 历史currentness

`EXPORT-EVIDENCE.json`与整套tool最后在commit `ee461fe8`（2026-08-03）更新；`view-descriptors.json`在commit `266d305e`（2026-08-07）把`preferred_drawer_slot/preferred_host`迁到`workbench_slot/default_presets`，但没有触发任何required lane或evidence owner。此前`docs/plans/performance/01/fixed-2026-08-02-workbench-design-export-freshness.md`已经把同类问题标记为fixed，并宣称clean checkout双重验证271 PNG通过。四天后的普通fixture变更即可再次使证据失效，证明修复只替换了mtime机制，没有建立依赖选择、owner通知与required gate。

## 3. P0：必须先修复的真实性与发布边界

### WBP-P0-001 · Design inventory被当作Editor capability inventory

192个`editor-page`与其静态输出被权威计划用于结构目标，但manifest不表达`EditorCapabilityId`、owner crate/module、Document/Command/Operation、runtime provider、实现状态或验收test。不存在的Localization、LiveOps、Online、Render Queue等能力可以和真实功能获得同样的“有图”状态。必须建立DesignSpec到Capability Registry的一对一或明确many-to-one映射，并强制`concept/prototype/implemented/verified/retired`状态；只有`implemented+verified`才能进入产品coverage。

### WBP-P0-002 · 当前视觉权威证据为RED且不在required CI

tracked clean source上的标准命令已exit 1，失败在8月7日提交后持续存在；`.github/workflows/ci.yml`没有任何设计验证入口。当前计划仍把输出集称为视觉权威，形成“文档GREEN、可执行证据RED”。required lane必须按变更选择运行schema/provenance验证，定期GPU lane运行真实capture/diff；失败应阻断提升视觉基线或声称current。

### WBP-P0-003 · candidate与approved baseline由同一操作自我签署

完整`design:export`顺序覆盖PNG、写style note，再把这些新candidate hash写进`EXPORT-EVIDENCE.json`。没有独立approved revision、incoming/diff、review operation、reviewer identity或promotion签名，因此错误renderer只要完成一次导出就成为新事实。必须分离`capture candidate`、`compare`、`review`、`promote approved baseline`四个权限与artifact；普通capture不得修改approved集合。

### WBP-P0-004 · artifact dependency graph连接了错误产品

固定design pipeline不读取fixture与Ionicon，却为35个无实际request的文件建hash；使用这些资源的legacy app三件套又不进evidence。当前digest失败正是false dependency造成的全库RED。必须由浏览器request trace、静态module graph和显式asset declaration共同生成per-design dependency closure；legacy prototype要有独立manifest/evidence，不能与固定DesignSpec互相污染。

### WBP-P0-005 · 导出发布不是事务，且server admission可捕获错误进程

exporter直接写最终目录，任一中途失败都会留下新旧PNG混合集；evidence最后才写且也非atomic rename。server readiness只对固定端口任意HTTP 2xx成立，不校验spawned child、nonce、build ID或manifest fingerprint；端口被已有服务占用时child可退出而exporter继续连接错误服务。必须使用随机受管端口、nonce health contract、child PID/exit监督、staging目录、全量验证与单次atomic publication。

### WBP-P0-006 · capture environment未进入identity，无法声称deterministic

lockfile只固定Playwright library，实际使用系统`msedge` channel；evidence不记录Edge binary/version/hash、OS image、font inventory、DPR、locale、timezone、color profile、GPU/software renderer、CSS engine或capture worker identity。相同source可在环境变化后产生不同PNG，而新export仍自写hash通过。必须建立`VisualCaptureEnvironmentId`与可复现worker image，并把platform/RHI/theme/DPI/locale作为baseline key，不可再用“HTML/CSS所以deterministic”替代环境身份。

## 4. P1：工程化重构清单

| ID | 差距 | 当前证据 | 目标 |
| --- | --- | --- | --- |
| WBP-P1-001 | `design.js`单文件混合inventory、domain mock data、layout config、DOM renderer与helper | 9,183 lines / 527,876 bytes，`additionalEditorConfig`约2.3K行 | 按DesignSpec schema、shared shell renderer、domain specimen、workflow/state family拆分；单文件预算与owner明确 |
| WBP-P1-002 | manifest与renderer双份手写registry | `DESIGNS`和`ALL_DESIGNS`都重复270个id/output；verifier事后比对 | 单一typed DesignSpec生成renderer registry、style note与export selection |
| WBP-P1-003 | manifest schema只有`id/output/kind` | 无title、owner、capability、viewport、theme、locale、status、approval policy | versioned schema + validator + migration，字段缺失fail closed |
| WBP-P1-004 | 192个domain页面压缩为少量generic结构 | 270页只有140种结构，最大同构组34页 | 共享shell允许复用，但每个domain必须声明独有workflow、state、control semantics与验收，而非换标签 |
| WBP-P1-005 | config lookup有silent prefab fallback | `additionalEditorConfig/detail/output`未知key均回退prefab | build时exhaustive map；unknown/遗漏直接失败并报告DesignSpec ID |
| WBP-P1-006 | 设计说明是大段生成字符串，不是可消费合同 | `writeStyleNote`硬编码大量批次段落 | DesignSpec字段生成简洁catalog；design token/layout/state规则进入独立versioned schema |
| WBP-P1-007 | 文档数量和权威口径可漂移 | layout index仍写“约250张”，真实为270+sheet；verifier只查少量marker | 文档计数、分类与链接由manifest生成，禁止手写近似数作current事实 |
| WBP-P1-008 | DesignSpec没有生产owner链接 | 无`.zui` root、pane descriptor、CommandId、DocumentKind、JobKind、ArtifactKind、RuntimeBridge、test ID | capability mapping gate逐项验证owner存在、状态一致、实现/测试路径current |
| WBP-P1-009 | evidence source closure包含35个false dependency | design三件套没有`/fixtures`、`/assets`或`url()`引用 | 记录每次capture真实request set并与declared dependency closure比对 |
| WBP-P1-010 | legacy prototype实际依赖未被绑定 | `app.js/index.html/styles.css`不在46项source，四fixture却在 | prototype独立package/manifest/test/evidence；或明确retire并删除入口 |
| WBP-P1-011 | capture identity只记录channel/尺寸/selector/timeout/wait | 缺浏览器、worker、字体、DPR、locale、theme、GPU等 | 完整`VisualCaptureEnvironmentId`和baseline dimension tuple |
| WBP-P1-012 | readiness依赖固定250 ms sleep | 未等待`document.fonts.ready`、renderer-ready generation、资源request quiescence | 页面发布typed ready receipt，包含font/assets/layout generation；timeout输出未满足条件 |
| WBP-P1-013 | exporter不收集page/console/request失败 | selector可见后即截图 | JS exception、console error、4xx/5xx、failed request、missing font均fail capture |
| WBP-P1-014 | server health没有nonce与manifest身份 | `waitForServer(baseUrl)`只检查`response.ok` | `/health`返回nonce、PID、ControlPlaneBuildId、manifest digest，必须匹配spawn receipt |
| WBP-P1-015 | 手工端口合同分裂 | `start`默认5173，exporter默认5187，env解析无范围/占用治理 | OS分配ephemeral port，由parent传递handle/nonce，不公开固定端口 |
| WBP-P1-016 | 输出直接覆盖最终路径 | 每页`page.screenshot({path: final})`，evidence也直接`writeFile` | capture到staging、fsync/validate/compare后atomic promote；失败保留candidate artifact而不污染approved |
| WBP-P1-017 | partial export没有独立receipt | 会改PNG与style note，但明确不更新evidence | partial candidate进入session目录并产partial manifest；最终目录只接受完整promotion |
| WBP-P1-018 | source dependency只有全局集合 | 任一source变化使全部271输出待重建 | per-design transitive dependency与affected-set selection；shared token变化才触发全量 |
| WBP-P1-019 | evidence没有revision与workspace状态 | 无commit、tree hash、dirty paths、capture session、producer build | source snapshot与producer identity必须进入不可变receipt |
| WBP-P1-020 | 26.37 MB binary corpus没有生命周期策略 | 271 PNG直接进Git、无LFS/filter、无retention/delta budget | approved baseline按用途与platform分层；candidate/diff放artifact store；Git只保留必要权威spec或受预算baseline |
| WBP-P1-021 | baseline只有单一1672x941环境 | 0 media query；无DPI、compact、wide、locale、theme、OS/RHI tuple | design intent至少覆盖compact/standard/wide与100/150/200% DPI；真实截图按支持矩阵选基线 |
| WBP-P1-022 | current failure没有owner notification与过期状态传播 | fixture commit晚于evidence四天，计划仍称fixed/current | dependency owner变更自动标记affected baselines stale，通知owner并阻断current claim |
| WBP-P1-023 | verifier比较candidate hash而非approved pixels | `buildExportEvidence`重算当前outputs后与同目录JSON比 | incoming必须与独立approved artifact比较，报告pixel/structure/a11y diff与阈值 |
| WBP-P1-024 | broad color profile能被错误深色页面轻易满足 | 只查luma/dark/bright/teal/unique colors | per-design approved diff、mask/tolerance、region semantic anchors与blank/solid检查并存 |
| WBP-P1-025 | 不验证文字与几何语义 | 无clipping、overlap、offscreen、duplicate ID、control label、focus order检查 | DOM/layout snapshot检查bounds、overflow、text fit、landmark与expected controls；真实Editor另做pixel diff |
| WBP-P1-026 | 手写PNG parser替代成熟图像库 | 自行读取chunk/inflate/filter，不校验CRC/完整结构，只支持RGB8 non-interlaced | 使用受维护PNG/image-diff library，明确色彩空间、alpha、ICC与错误报告 |
| WBP-P1-027 | evidence schema不拒绝重复entry | `Map(path -> entry)`会折叠重复path | schema validator检查unique path、排序、类型、hash格式、byte范围、完整dimension tuple |
| WBP-P1-028 | output目录只拒绝额外PNG | 其它陈旧JSON/MD/临时文件不受manifest治理 | manifest声明全部允许artifact；未知文件、缺失receipt与临时发布残留均失败 |
| WBP-P1-029 | 文档验证依赖substring marker | 三份Markdown检查固定短语 | 文档从schema生成或读取结构化frontmatter；文案变化不应破坏产品gate |
| WBP-P1-030 | negative guard重复三次全量验证 | ignored override、negative、restore各扫描271 PNG | 将reference comparison抽成focused unit/contract test；一次完整suite只做最终集成 |
| WBP-P1-031 | 已知provenance失败仍扫描全库像素 | 本轮15.8秒后才报告单一digest mismatch | 分phase fail-fast：schema/source identity先行，pixel scan只在identity有效时运行；结果仍输出phase receipt |
| WBP-P1-032 | 人工视觉QA没有 durable approval receipt | fixed handoff只写“representative visual QA passed” | reviewer、review set、approved/rejected decision、diff artifact、timestamp、source/producer identity可查询且不可变 |
| WBP-P1-033 | pinned runtime reference不是当前Editor输出 | 两份同一1.526 MB PNG仅做byte copy测试 | reference intent与real product capture分离；后者必须由当前Editor binary/project/interaction生成 |
| WBP-P1-034 | 270张design output没有生产acceptance consumer | production tests只检查单个`workbench.png`与fixture include | 每个implemented capability至少一个真实Editor interaction/capture gate；concept图不得冒充该gate |
| WBP-P1-035 | 缺incoming/approved/diff/triage/promotion工作流 | verifier只有pass/fail文本 | 提供comparison report、overlay/heatmap、new/missing/different分类、owner review与source-control/artifact promotion |
| WBP-P1-036 | legacy bootstrap无HTTP/schema错误处理 | 四个`fetch(...).then(r => r.json())`，任一失败即unhandled blank page | typed fixture loader、HTTP status、schema version、error surface与retry/diagnostic |
| WBP-P1-037 | prototype每次操作全量重建DOM并线性查找 | `render()`清空`#app`；多次`Array.find`与完整tree traversal | 若保留prototype，使用immutable state+incremental projection并记录交互结果；否则明确retire |
| WBP-P1-038 | prototype直接改fixture projection且JSON深拷贝tree | dock/split直接mutate，`JSON.parse(JSON.stringify(node))` | typed layout commands、transaction、undo/redo、schema-preserving clone与validation |
| WBP-P1-039 | 可见menu多数没有command | Save/Reset/Undo/Delete等只是无handler的`div.menu-item` | 删除伪命令，或接入真实prototype command registry并显示disabled/reason/outcome |
| WBP-P1-040 | drag/dock为mouse-only且不持久 | draggable div、无keyboard/ARIA、无save/load、floating不可继续管理 | keyboard parity、focus/announcement、drop semantics、persistence/migration；否则不宣称interactive workbench |

## 5. P2：一致性、维护性与局部可靠性

| ID | 差距 | 建议 |
| --- | --- | --- |
| WBP-P2-001 | 直接访问未知`?design=`静默回退scene | 页面显示显式unknown design错误；CLI继续fail closed |
| WBP-P2-002 | port用`parseInt`但不校验NaN/范围 | typed config parser校验1..65535或只用OS分配端口 |
| WBP-P2-003 | 404把完整Node error与本机路径返回浏览器 | 记录server-side诊断，client只返回typed 404，不泄露absolute path |
| WBP-P2-004 | localhost server没有method allowlist、CSP、nosniff与明确root containment check | 只允许GET/HEAD与三个受管roots，resolve后校验containment并加安全header |
| WBP-P2-005 | spawned server stdout/stderr设为pipe但从不消费 | 使用受限ring buffer并纳入failure artifact，避免未来日志量导致backpressure |
| WBP-P2-006 | CSS存在36个`!important` | 通过layer/specificity/token component contract收敛，避免截图修补优先级 |
| WBP-P2-007 | 只有22个变量却散布297个颜色literal/125种颜色 | 从生产design token schema生成CSS；只允许审计过的semantic token |
| WBP-P2-008 | 949个px literal且无media query | 固定baseline可保留canvas尺寸，但组件metric、DPI与compact variant必须来自token/variant schema |
| WBP-P2-009 | icon/select/mini button常用`div`模拟 | DesignSpec DOM用真实或`inert`语义组件，避免参考图掩盖交互状态缺失 |
| WBP-P2-010 | 固定设计页包含1,805个可聚焦button但没有handler | 静态capture root设`inert`；要演示行为的页面进入独立interactive scenario |
| WBP-P2-011 | `design:export`与`design:export:only`脚本完全相同 | 合并为一个有清晰`capture candidate`语义的CLI，selection由参数表达 |
| WBP-P2-012 | 270-entry串行capture至少包含67.5秒固定sleep | 分片并行要按worker/environment identity隔离；先移除固定sleep并用ready receipt，不能只加并发 |

## 6. 参考引擎对照

### 6.1 Unreal Engine

Unreal的`AScreenshotFunctionalTest`从真实world/viewport捕获，截图前执行loading flush、camera cut与第二次flush，并用time delay和frame delay等待；capture metadata携带context、test label、variant、RGB/A/brightness tolerance、local/global error、anti-aliasing/color规则与frame trace。`ScreenShotComparison`再区分incoming、approved与delta/report路径，浏览器允许筛选、审查、add/replace，并把promotion接到source control。Zircon应吸收的不是UObject/Slate实现，而是actual product capture、settle contract、comparison policy、variant/environment identity和独立approval operation。

### 6.2 Unity Graphics

Unity Graphics的测试由`SceneGraphicsTest`枚举真实scene，按scene内`GraphicsTestSettings`等待指定帧数，再以`ImageAssert.AreEqual(reference, camera, settings)`比较。reference目录显式分Color Space、Platform、Editor、Graphics API；Yamato分别在Windows DX11/DX12/Vulkan、Linux Vulkan、macOS Metal及player/editor lane运行，上传test-results/log/player artifacts。Zircon当前单一Edge/单尺寸self-hash没有同等级的环境维度、实际renderer入口或required GPU matrix。

### 6.3 Bevy

Bevy example showcase把`fixed_frame_time`、screenshot frame、stop frame、WGPU backend、example selection和CI mode显式化，分别记录success/failure/no-screenshot；Pixel Eagle workflow以上传artifact、commit、branch和OS建立run，先按hash去重，再与同OS main branch比较并因missing/diff退出失败。它仍不等同完整Editor视觉审批系统，但证明capture scheduling、backend identity、结果分类和跨revision comparison必须在artifact协议中，而不是隐藏在250 ms sleep里。

### 6.4 Fyrox与Godot

Fyrox的Editor preview直接拥有真实`Scene`、camera、render target、resource UUID cache与有界throughput，preview交互通过UI message更新；它说明资源预览证据应来自实际engine resource/render path。Godot Editor的截图路径从Editor/game embedded viewport请求并读回真实image，处理嵌入进程rect与保存错误；当前参考树没有与Unreal/Unity同等完整的Editor screenshot comparison harness，因此本篇只吸收它的真实Editor capture ownership，不虚构其baseline审批能力。

## 7. 目标架构

### 7.1 三种产品必须分离

```text
DesignSpec Registry
  -> intent/layout/state/token references
  -> concept/prototype/implemented/verified lifecycle
  -> no product-completion claim by itself

Interactive Prototype
  -> scenario state + command/outcome + keyboard/a11y
  -> disposable or versioned independently
  -> never shares evidence graph with fixed spec capture

Visual Product Test
  -> real zircon_editor binary + project fixture + scripted interaction
  -> platform/RHI/DPI/theme/locale baseline key
  -> incoming/approved/diff + test result + artifacts
```

### 7.2 typed contracts

`DesignSpecV1`至少包含：`design_id`、`kind`、`title`、`capability_ids`、`owner`、`lifecycle_status`、`renderer_family`、`scenario_id`、`viewport_variants`、`theme_variants`、`locale_variants`、`token_schema_version`、`expected_control_roles`、`reference_policy`、`retirement`。

`VisualCaptureReceiptV1`至少包含：source snapshot、dirty state、producer build、browser/editor binary、OS image、font set、GPU/RHI、DPR、locale、theme、viewport、scenario inputs、ready generation、request/console summary、output hashes和artifact URI。

`VisualComparisonReportV1`至少包含：incoming/approved identities、policy/tolerance、missing/new/different/identical分类、pixel metrics、semantic/layout/a11y diff、diff/overlay artifacts、owner与required action。`VisualApprovalReceiptV1`再单独记录reviewer、decision、scope与promotion revision。

### 7.3 publication flow

```text
SourceSnapshot + CaptureEnvironment
  -> affected DesignSpec selection
  -> isolated worker + nonce health
  -> staged candidates + capture receipts
  -> schema/semantic/layout checks
  -> compare against immutable approved baselines
  -> CI report + diff artifacts
  -> explicit owner review
  -> atomic approved-baseline promotion
```

## 8. 实施顺序

1. **M0 Truth freeze**：把现有271张PNG标记为`design-intent/currentness=RED`，不得据此提升产品能力；保留失败evidence，不批量重导出掩盖根因。
2. **M1 Schema split**：建立DesignSpec、PrototypeScenario、VisualProductTest三个registry；迁移270 ID并补owner/lifecycle，删除silent fallback和双registry。
3. **M2 Evidence graph**：用真实request trace生成per-design依赖；把legacy app拆包或retire；source/producer/environment identity进入capture receipt。
4. **M3 Transactional capture**：ephemeral port+nonce、typed ready、console/request gate、staging与atomic candidate publication；partial capture不触碰approved目录。
5. **M4 Comparison/approval**：引入成熟image codec/diff，建立incoming/approved/diff、policy、review与promotion receipts；迁移当前PNG为首次人工审定baseline，而非自动自签。
6. **M5 Product bridge**：对`implemented` capability逐项建立真实Editor binary interaction与capture；concept/prototype页面只能作为设计输入。
7. **M6 Required lanes**：PR跑schema/affected-set/headless semantic checks；受管GPU lane跑平台/RHI产品截图；nightly做全270 intent capture与real product matrix抽检，所有artifact可追溯。

## 9. 验收门

1. clean tracked current source执行schema/provenance gate为GREEN，且不会因未被页面请求的fixture/icon变更全量失效；
2. 修改一个domain specimen只选择其受影响输出，共享token变更才选择声明依赖的全量集合；
3. 端口被占用、child提前退出、nonce不匹配、字体未ready、request 404、console error均fail closed且不修改approved目录；
4. capture receipt完整记录source、producer、browser/editor、OS、font、GPU/RHI、DPR、locale、theme和viewport；
5. candidate生成不能自行成为approved，promotion必须存在独立review receipt；
6. 错误但仍为深色/teal的整页替换会被per-design diff或semantic anchor拒绝；
7. manifest遗漏renderer/config、unknown key、重复evidence path、未知artifact均在schema phase失败；
8. 270个ID全部有owner与lifecycle，`implemented/verified`必须有真实产品入口和acceptance test；
9. concept/prototype截图不会计入Editor capability完成度或release readiness；
10. compact/standard/wide、100/150/200% DPI、支持的theme/locale不发生文本截断、重叠和不可达控件；
11. real Editor screenshot按platform/RHI baseline key比较并输出incoming/approved/diff与日志；
12. legacy interactive prototype若保留，Save/Undo/Reset/Dock具有typed command outcome、keyboard/a11y和persistence；否则入口与fixture evidence一并retire；
13. CSS颜色与metric来自生产token schema，禁止新的裸色值和截图专用`!important`补丁；
14. required CI能因source/evidence drift、missing screenshot、visual diff或approval缺失退出非零；
15. full verifier先做identity fail-fast，不在已知source mismatch后浪费271张PNG解码；
16. candidate/diff/历史artifact有retention与存储预算，Git中只保留明确批准的最小权威集合。

## 10. 本轮边界

本篇拥有DesignSpec registry、web prototype隔离、截图capture/provenance/comparison/approval/publication、visual required lane与设计证据到产品能力的映射。Editor01继续拥有真实retained UI/layout/input/accessibility与性能实现；Editor各domain报告继续拥有具体工具是否存在、能否编辑和runtime闭环；Tooling10继续拥有跨语言测试架构与result service。本篇不因静态网页画出某个工具而重复登记该工具的功能缺口。

本轮只新增review与索引，未运行会写文件的`design:export`，未修改271张PNG、`EXPORT-EVIDENCE.json`、fixture、web prototype、production Editor、tests或CI。
