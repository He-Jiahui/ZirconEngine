---
title: Editor04 Activity 可见行虚拟化与 F1 资产投影优化记录
category: zircon_editor
date: 2026-09-30
implementation_status: component_candidates_reviewed_shared_visibility_support_applied_union_pending
validation_status: managed_tests_not_run
performance_status: native_product_acceptance_open
---

# Editor04 Activity 可见行虚拟化与 F1 资产投影

Activity/F1 合并候选尚未应用。本记录区分已经写入的测试支持修复与待移交的优化候选，不表示 F1 项目与资产、测试或性能验收完成。

## 优化候选及审查

Activity 只为视口及 overscan 创建实际文件夹和资产行，保留完整逻辑滚动范围；指针按逻辑行定位文件夹 ID 或资产 UUID，并保留不可变 Arc 行代际及投影生命周期。F1 从不可变输入在后台准备目录、类型/详情与 Workbench 投影，按请求和来源代际拒绝过期结果，同键暂时失败采用有界退避。locale 和 projection_pending 参与呈现身份，使用已有国际化准备中和空状态标签。

| 冻结组件 | 范围及身份 | 静态审查 |
|---|---|---|
| Activity v1 | 14 个修改目标；patch `167b86a558b1b1eafd27563f76088c8c14df186765d65fbbd29df86453374948` | `1ae569a602fd4b5b432d7018391b0ce06bb5b2fa82a7e835689348aced39ad04`，无该切片 must-fix |
| F1 v4 | 59 个候选来源、43 个修改目标；patch `acf94525161ee4556c4bdbc0787623b127c549a939087c8b579cfacc56f7f6c6` | `c517a6e120e2de2bdd7fe74221ea55c35d75051e020d53967498415e04c5f602`，增量审查保留前序边界 |
| Activity/F1 integration v2 | 18 个来源、15 个修改目标；manifest `5efe806b8aa358e278ad3441249fe4ad8bb2e1cddaab2ce49c96502f9d825a3d` | `5bff3d71faa35ff7b9826ced73a7463e7f9f960a59f442b34cba5a4a3408b663`，无该增量 must-fix |

最终 union v1 manifest 为 `c15f3d16903d82dde5b05a7038b8588d61a62e98dec0c69bcee082e3313a268f`：65 个唯一修改目标，82 个候选来源加 6 个只读依赖，共 88 项模块/测试来源。14 是组件计数，不与 65 相加；88 不是完整 Cargo 编译闭包。旧 v1 没有合并 patch；当前 v2 的精确字节补丁和隔离重放见后续记录，源码尚未应用。Runtime09 v4 typed shutdown 是独立、未应用的依赖，manifest `34e2e3cfbd19c121d2bb834f08cf6c8ea59c5304ee495a8b7076c76f6e2a05ff`，不包含在 88 项中。上述审查未运行 Cargo。

## 已应用的最低支持层修复

inline 场景视口回归使用真实 `crate::tests::editor_event::support`，但父模块原来私有。仅将 `zircon_editor/src/tests/mod.rs` 的 `mod editor_event;` 改为 `pub(crate) mod editor_event;`；既有 support 和 EventRuntimeHarness 保持原实现。

- 原始 SHA：`fe7d89aba080c35d391c553f98a2a1055983c212655523d753c4352fe1242405`。
- 已写入并登记的 SHA：`394a6950b5d0ff29234963fbeafe9b2bb14c60f6c6225e2b879f1008052e3578`。
- 独立审查 SHA：`8a3ae1997c9ce745cad24dfbe90cd1f29f6c7e92f96f3031910cf06f9bdc7aae`。
- 写入日志：`.codex/state/session-coordinator/async-validation-batches/2026-09-30-editor-shared-event-test-support-visibility-guarded-apply.json`，SHA `18c851b19eaebb3eaf6d01467e2ef3525c1ebe4192440a28281aa1dad2d0d0c7`。
- public 精确移交、live lease、原始字节保护及 baseline.attribute 完成；rustfmt 和 scoped diff check 为 0。Cargo/typecheck、动态测试和性能仍未验证。

这是单独的依赖 overlay；旧 union 的只读 tests/mod.rs hash 已成为历史，不证明 65 目标合并候选已应用。

## 剩余修复与来源准入

两个真实场景视口测试使用 `let _guard = env_lock();`，仅保留锁对象引用，随后 Harness 修改进程全局 `ZIRCON_CONFIG_PATH`。候选在 Harness 创建前取得 `.lock().expect(...)` 的实际 guard，并保持至 Harness 销毁。prepared SHA `7f53f9eb13a432f3a5b221649ae73c0edfeaa1940e2afead86d24475bb2364a7`；独立审查 SHA `688652740ee0156a61b22d38cb903b410f5d7dbe0cad44bce214812b406eae46`。候选 rustfmt 与 patch check 通过，尚未应用。

该 snapshot.rs preimage 为 `9ecfd6b825f04fc4566aadd86ff2a35139fba851a2765ddfc116a669224ba0ed`，包含外部会话后续工作区回归。public 预览返回 registered 所有者 `astra-editor-leaves-20260929-01a0f033` 和 `source_owner_executable`；不得用旧 union 覆盖或强制移交。

旧 88 项所有权审计 SHA `fd09188326a06cadb73fa8e03240d7a0ad289d5246d50322fd0d48a704444a07` 是时间点记录，脚本 inactive 分类不证明 public 可转移。新67路径public preview请求 `f8da80fa8c194a508f1a23bdbbd5e1a5` 初次client超时，随后在v4记录修复与真实小工程输入准备完成的工作检查点核对原请求一次。终态completed：56条eligible、11条ineligible；其中10条为 `source_owner_executable`（8条active M1及2条registered Editor叶节点会话），1条为本会话已持有的tests/mod.rs（`path_already_owned_by_target`），后者不构成外部阻塞。核对日志 `.codex/state/session-coordinator/async-validation-batches/2026-09-30-editor04-exact67-public-preview-single-reconciliation-at-record-repair-v4.json`，SHA `55a35004d347d581c8f79027cb1092cafcfe2df5658109c10ce548e968786c5c`；这是17:59 UTC时点的预览，未转移所有权或写入union源码。应用前仍需当前字节、live lease和合法移交；不重放旧fingerprint或据eligible结论覆盖后续外部修改。

remaining-delta 审查 `da8e4da6ccdfaaca6e98096f6b69f4d45df5bc6e174d08d320774ad2f051fa01` 确认四处截图 fixture Arc 适配类型一致，但旧冻结包缺三个 fixture 的原始字节副本；当前 v2 已补齐并精确重放65目标，完整编译准入仍开放。Projector 审查 `363f30e6c7a9ae428a8fad95bb0f487d2aa7b9e287edddaf55f4394cfd3a853d` 仅覆盖冻结增量。后续必须保留外部修改，补当前原始字节、所有权/lease/hash 准入与完整编译来源。

## 真实产品验收

产品已有 `zircon_editor.exe --project <root>` 入口，当前版本 workspace 可保存/恢复 `editor.assets#1` Activity。真实 asset roots 经 runtime 目录输入进入 Editor；路由静态证据 SHA `934993b3be798c0ad6884c2659cdb15837bca135f065d8aef2f1d42dd2dba9a2` 不证明 100k 加载、命中或性能。现有工具 runner 的 10k 上限不能作为 100k 结果；后续使用真实工程输入，保持 tooling 不改。

按 Optimize01/04 执行产品 gate：每个 measured PID 内完成 warmup、measured、quiescence；click1000、pointer move1000、resize200 为独立场景，各至少3轮，不要求各轮共用一个 PID。p95 input-to-damage≤1ms、damage-to-submit≤8ms、input-to-successful-present≤9ms；帧预算16.67ms单独检查。warm peak RSS 增长≤96MiB、quiescence-end≤64MiB且无单调增长。记录 P50/P95/P99、分配、RSS和来源代际；没有批准的 P99 数值上限。

真实1/100/10k/100k打开、关闭、重开及交互证据，分组 managed Debug/Release 验证和最终产品验收全部开放。完成列表：`docs/plans/astra/features/editor/1067-editor04-activity-visible-row-virtualization-completion-list.md`。

## 2026-10-01 当前源码合并与精确重放

当前保留源码的 union v2 位于 `offline-candidates/root-editor04-current-preserving-union-v2`。manifest SHA `f4721f8e3cfeabf2a5c7d4e4e0e76afa43d527586dd7fd64f36f7c8c6ada999c`；65 个修改目标、17 个未修改支持来源、7 个当前只读依赖，snapshot 同时属于后两组，去重为 88 项。它仍不是完整 Cargo 闭包，Runtime09 typed shutdown 仍为独立未应用依赖。

全部既有目标的原始字节已归档，包括旧包缺失的三个 fixture。63 个修改 Rust 文件通过 `rustfmt --check`；17 个不修改支持来源通过 parser/emit 检查并保留原字节。16 个格式差异经 Root 静态核对；这不等于编译或语义测试通过。

project_access 的当前来源 SHA `b7db12ab9ae323161a64f338fbf5f97bb6430089bc6c717f0678e133e08f1eee`。旧 F1 候选遗漏了当前 `save_active_scene_with_workspace` 入口，实际 `editor_manager_workspace.rs` 仍调用它。私有 rebase 保留当前 `save_active_scene`/`save_active_scene_with_workspace` 完整实现和 `ProjectEditorWorkspace` 引入，保留 F1 后台打开逻辑；候选 SHA `1f49f9b4ab354def0bb5ec6014ac8ebb9b37491c8076761a861844697df9665a`。独立增量审查及合法 owner 移交仍开放，未写入共享源码。

精确字节 patch `candidate-exact-bytes.patch` SHA `ccc9be547c75e56c89d3900746e203b8d917bc3a40be814e5a997a41b909ee06`；从 65 个归档 preimage 进行隔离 `git apply --check` 和实际 replay 均为 0，65 个结果与候选字节逐项一致。回执 `exact-replay-v2-receipt.json` SHA `8b1c8350d9893d50a48eea0dc37f2af58fbd100d6d3e41db9a28485bb05be272`。先前规范化文本补丁丢失 CRLF，且深路径触发 Git 路径限制；仅修正私有补丁，按命令启用 longpaths，不改 tooling 或仓库配置。

snapshot 保留实际来源 SHA `9ecfd6b825f04fc4566aadd86ff2a35139fba851a2765ddfc116a669224ba0ed`，包含外部新增的 workspace 回归；两处环境 guard 候选仍未移交/应用。既有父模块可见性与 Harness 配置恢复修复均保留在当前只读来源。不能部分写入跨 API 合并候选来替代完整准入。

真实 1/100/10k/100k 工程输入已物理生成并完成文件系统核验，回执 `offline-candidates/editor04-native-activity-scale-project-inputs-v1/receipt.json` SHA `e1fc485cfc33e3041a4b265d540bafe06bb0ebc7466451c47c096237299bb1e6`；100k 输入有 200009 个文件、8304611 字节。引擎目录计数、产品打开/关闭/重开、交互帧延迟和 RSS 尚未执行。工程生成和 patch replay 不关闭产品性能 gate。

## 2026-10-01 · 完整 v3 候选与来源准入

v3 在 worker 完成的 `AssetWorkspaceSnapshot` 构造补 `projection_pending: false`，在同步测试构造补 `projection_pending: false` / `projection_lifetime: None`。两处缺字段会阻断类型检查，现已在私有候选修复；其余 63 目标字节保留，真实 pending → ready、lease 及保存入口保持原契约。[editor04_v3](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-editor04-current-preserving-union-v3/manifest.json) SHA `1f9d63788914694e7921922662c6072b453cc5f4f0fa0e402e99c375f61f73d5`；[editor04_v3_review](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-editor04-union-v3-incremental-review-v1/review-bindings.json) SHA `31cf0c1b12d1952ae2ecdcd6c78d3294d17b793889f94e9ba642662bd1cfc153`。

- [x] 完成 65 路径候选和两文件增量的精确重放、格式检查及独立静态复核。
- [ ] 合法取得 `zircon_editor/src/ui/host/project_access.rs` 的完整来源交接后应用全契约；当前 registered foreign owner 保留，未拆开应用。
- [ ] 将 F1/Activity、Runtime09 全 42 终态契约和 Runtime02/Editor63 的新 API 合成，再执行兼容 Check、正常回归及 Release 批次。
- [ ] 执行真实 Browser / Activity 规模、交互和原有 latency/RSS 门；当前未取得性能达标证据。

[editor172_v5](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-editor172-current-preserving-terminal-runtime09-v5/manifest.json) SHA `2f9c2d477315d985120035f7881a67a4c56ddd0e61b30d7a29b588131d196863` 保留全部 42 typed shutdown / observer / absolute-deadline 契约，41 候选与上游字节一致，更新模块保留当前视觉证据抽取。它是合成输入，尚未完成 Runtime02 V2 / Editor63 V4 新 API 适配，也未执行 Cargo。


## 2026-10-01 · 完整 terminal 依赖的当前源码合成

[完整 42 路径 V5 增量独审](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-editor172-v5-independent-delta-review-v1/review.json) SHA `15e758e0f27026833a52a11dbfd0868452b69dec58e380d90cbd0829ff9d7431` 核对完整 42 candidate/preimage 与 28 readonly dependencies，41 个上游候选字节保持一致。唯一 mod.rs 增量保留当前 zui_visual_evidence 提取模块与五个 public export，并加入 pending terminal shutdown export，未留下旧 inline 重复实现。

- [x] 完成这一明确 mod 增量的独立静态复核；复用同 SHA 的格式与精确 replay 证据。
- [ ] 合成 Runtime02 V2、Editor63 V4 及 Editor04 companion 的完整来源；保留 Runtime09 completion_pending、barrier/lost-wakeup 与同一绝对 deadline。

Editor04 整波 65 路径继续受 project_access 外部归属约束，未应用；这一依赖复核不代表新 API 已编译兼容或产品性能通过。

## 2026-10-01 Editor UI 支持修复记录（私有候选）

- Workbench A37 已独立核对 37 个源码/前像、21 项 API、217 个自有调用及完整补丁回放；旧 711 个断言和 125 个测试名称均保留。实际 admitted base 后发生更新拒绝时，已提交 entry/cache 保留，pending dirty 与 Unchanged 重试源码成立；未执行成功恢复测试，18 条外部 caller 和整批类型/性能门仍开放。
- Asset preview 已冻结 17 个增量路径、168 个候选源文件：same-Core owner、Send leaf、source/selection/cursor/palette/history 与 Undo/Redo 暂存后提交，以及真实 retained-weight 拒绝后同 owner 重试回归。独立审查、外部 registry/test caller 迁移与行为执行尚未完成。
- Scene viewport 独审新增首 miss 的接纳原子性缺陷：接纳前 cache 查询会消耗 Stale marker，临时拒绝后同 owner 成功重试变为 Preparing，宿主可能失去 redraw 通知。V1 不可当作已修复；新 support-first successor 与真实预算拒绝/重试回归待完成。
- 多窗口 presentation 必须先准备全部窗口再通过单个 registry/native publication gate；toolbar/window producers 正在私有迁移。共享 action stage 同 pane 重复 bind 的 epoch/token 缺陷也在最低支持层修复。
- 原 Editor04 activity cohort 尚未绕过 foreign host/project-access 边界；本记录没有将任何上述私有候选计为共享源码或运行验收完成。
- [ ] 原测试、Editor95 与 Editor172 whole42 当前基线组合验收通过。
- [ ] 实际 UI 回归及原计划性能预算达标。

## 2026-10-01 后继独审与统一提交边界

- Scene viewport V2 的首次 Stale marker 拒绝/重试和唯一借用 reset prepare/commit 已经五文件增量独审静态闭合，真实 6 源 / 55 项 Runtime API 绑定也替换了旧错误依赖。五个回归仍未执行，外层 authoring/borrow/type/native/性能门开放。
- Retained B68 已独审 68 个源文件、两个完整原始字节回放、Rust 2021 格式与旧断言保留。新的 geometry/window-metrics 暂存后继正在接入真实全部窗口统一 publication；单个窗口成功不能替代后续窗口接纳失败时的整体旧 frame/cache/dirty 保留。
- Asset 的 31 个外部测试 fixture 已迁移 179 个构造调用并保留 1558 个断言与 185 个测试名，私有 replay/格式通过；源码独审、整批正常测试与数值性能未完成。
- Editor63 checkpoint3 的旧持久化 RAII、跨 catch uncertainty 与 Scope 权限传递已静态复审；新增普通 cancel 的 OwnerKey 依赖及 gizmo setter 错误转换仍是明确 P1 后继修复。实际 activation/world registry/native terminal wake 与绝对跨 DLL clock/ABI 后继仍待组合与验收。
- 本次只准备记录与已独审 EventBus contention 两源 profile，不写未冻结 Editor 大 cohort，也没有编译/测试/性能通过结论。

## 2026-10-01 补充：实际调用方、窗口计数与生命周期审查

- [x] 记录独立静态审查：窗口成功 pane 发布与非空 geometry addon 发布补回原有 rebuild 计数；拒绝、纯 geometry 和空 addon 不计数，旧 wrapper 不重复计数。
- [x] 记录剩余 11 个 Editor 测试调用方的私有迁移：485 条旧断言和 37 个旧测试保留，构造与执行借同一个实际保留的 Core。测试尚未执行。
- [x] 记录 UiSurface 专用 projection checkpoint 与 actual manager secure-store rebind 的私有修复；普通 Clone 的跨 surface 隔离保持。实际 pointer caller 后继、Scene mode/world/focus 的准入顺序及整体类型闭合仍待完成。
- [x] 记录 Engine checkpoint5 与 clock3 私有增量独审：V3 receipt 验证、activation terminal wake、held permit 转发和错误阶段缓存 deadline 的已确认问题已静态修复。
- [ ] Retained actual host 失败清理、安装与 hydration 顺序、fresh watch proof、EditorState 终态 metadata acknowledgment、startup Pending owner retention 完成整体组合并通过运行验证。
- [ ] 完成整批 managed 编译与行为测试；执行原计划 Release latency、allocation 和 RSS 数值验收。上述私有源码和静态重放不能作为测试通过或性能达标证明。

审查依据：离线候选目录中的 `root-runtime02-editor-ui-retained-pane-counters-v2-independent-review-v1`、`root-runtime02-editor-ui-remaining-caller-tests-independent-review-v1`、`root-runtime02-surface-projection-checkpoint-independent-review-v2`、`root-editor63-v4-engine-checkpoint5-independent-review-v1`、`root-editor63-native-abi10-clock3-independent-review-v1`。最新归属与字节绑定见本次公开准入及实际写入收据。

## 2026-10-01 补充：pointer 身份、拒绝恢复与同窗统一发布

- [x] 记录 pointer lower11 V2 的具体身份修复：projection checkpoint 保持 origin，实际 InputManager 同 origin 切换到新 secure store，Drop/foreign activation 清理实际当前 store；普通独立 surface Clone 保持既有隔离。[lower11 独审](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-editor-ui-pointer-lower11-v2-independent-review-v1/review.json) SHA `306dab3b1b5553b7c0db2be5f0275485de26c76d46146331c432f2f6ad18ef70` 与 [Runtime 七源支持独审](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-surface-projection-checkpoint-independent-review-v2/review.json) SHA `fb2e833ce4e8e1f62c2760516d36ce8082aef34a7029f0da687cbcd35fbb10c1` 均为精确私有源码证据，因果测试尚未执行。
- [x] 完成同窗多视图发布的私有支持修复：所有视图借同一不可变 snapshot，先准备全部 fallible leaf 与一个 aggregate PanePatch，再生成一次 native Rc/cursor receipt；单视图调用委托同一路径，结果借用保留 receipt，成功 generation/rebuild count 只递增一次。[四目标冻结](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-editor-ui-retained-scoped-aggregate-successor-v1/manifest.json) SHA `269a0f6ec9e1de5eb2e6e57148e94c6206177d437172afdd158599bc9bcd50a2`；[非作者独审](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-editor-ui-scoped-aggregate-independent-review-v1/review.json) SHA `b88c4090d67d2f387f1a8697b0ab01d077d22d985fb0da3bd234ecbd61a1680a` 的双前像八次回放与四文件 Rust 2021 检查一致，保留七个旧测试和 20 条旧断言，新增两个因果源码用例。拒绝用例为首个视图纯暂存、后一个真实 leaf ScopeClosed 后整窗旧 cursor/frame/index/pane/count 保留，未观察 red/green 执行。
- [x] 记录 [实际 53 调用方检查点](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-editor-ui-pointer-callers-successor-v1/caller-checkpoint-v1/manifest.json) SHA `79f1336437e32f8596b7bca663c4d9d9dc6efc1e33bbfeb6313539b3f3a45fe5`：私有后继保留 typed host 错误，focus 拒绝阻断后续调用，drag 拒绝恢复本 attempt 投影及旧 target group；floating header 只在真实 host focus 成功后推进 last focus。[53 源非作者独审](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-pointer53-independent-review-v1/review.json) SHA `bc386a29a656b55336f3b7ec1628f66264bc75dfaeab70794080a9886d1375a4` 核对双前像 106 个 postimage、220 个支持文件、152 个实际签名锚点及原七个测试名/17 个断言 token，无新增自有必修项。Welcome receipt 当前仅为 DTO，真实 project helper 在 UI commit 后执行，typed/staged 上层接纳继续修复；Scene pointer_event 与 toolbar saved focus 的真实准入边界尚未闭合。
- [ ] 将同窗支持接入全部窗口统一 Registry/action epoch/native publication，完成实际 Scene/World/Focus 的先准入后变更、16 个外部测试调用方与 partial startup owner carrier 的整体组合并合法应用。
- [ ] 执行整批类型与正常行为回归；真实 1/100/10k/100k 工程、click1000、pointer move1000、resize200 各至少三轮，验收 p95 input-to-damage <=1ms、damage-to-submit <=8ms、input-to-successful-present <=9ms、16.67ms 帧预算、warm peak RSS 增长 <=96MiB 与 quiescence-end <=64MiB 且无单调增长，并保留实际分配与 P50/P95/P99 原始证据。

上述 caller 与窗口修复尚未作为完整共享 Rust cohort 应用，测试和产品性能仍开放。声明 retention、实际 heap 分配与 RSS 分别核验；单窗静态证据不足以关闭全部窗口和 native 门。

## 2026-10-01 补充：实际 buffer 支持与上下文身份

- [x] 已写入共享 `MaterialOverrideSet::retained_payload_bytes()`：按实际 `Vec` capacity 与 Copy slot 元素表示 checked 计费，资源对象 referent 不属于这个 buffer。[实际单源写入和归属收据](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-material-slot-helper-current-admission-v1/one_material_slot_helper_guarded_actual_apply_and_attribution.json) SHA `ba049db7251f942e6db2e1547f1c813ea24d28e85cda0a08edd7df58d7ac90ab` 确认公开 eligible 移交、fresh lease、原始字节归档、diff check 为 0 和 attribution 完成；[独立静态审查](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-material-override-retained-payload-independent-review-v1/review.json)已核对实际类型与 buffer，未执行 Rust 测试或性能测量。
- [x] 私有 same-activation validator 已独审：真实 ScopeInner Arc 创建 token Weak lineage，JobScheduler/UiDriver 纯身份检查拒绝 foreign Core 和同 Core 不同 activation；旧 admitted context 在 Closing/Cancelled 后继续有效，不进行第二次准入。[七源独审](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-activation-context-validator-independent-review-v1/review.json) SHA `58015cb780befe547c5effd52765c953bd25951713461b3dcd6c53bec4c45802`；四项因果源码回归未运行，旧 same-pin retirement body 保持原始字节。实际 CoreV3 TaskNode provider 与 shared 观察依赖分别记录，最终组合仍须保留真实 provider。
- [x] Scene 私有 Accounting4 将 captured mesh material override slots 的实际容量加入 fixed-token charge；真实非空 published cache、quota 拒绝前 Mode 不变与同 Core 释放 holder 后重试源码已补齐。[四增量独审](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-scene-input-accounting-independent-review-v1/review.json) SHA `dd6076f3ffc92ecff8829c5f3173037b9ba44293313a870a8c3455f7ac960f11` 关闭旧 slots 遗漏与 token-weight 文档问题；计量声明不等于 allocator/RSS 实测。
- [x] reset fixture 先完整 `.err().expect(...)` 消费 probe Result 再重借 controller；生产 reset body 保持旧 reset83 的原始字节。[增量独审](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-scene-reset-fixture-independent-review-v1/review.json) SHA `1c87bbefb758161e1127c5e5317d046de3225fae65617b6a32fc0e0fbe2b58db` 核实旧断言保留，amendment 如实区分 raw 不同与 LF-normalized 相同。没有观察到 compiler E0499 或测试 red/green。
- [x] State/host Completion20 V2 删除 GizmoTransactionError 中不存在的 variant 分支，将 typed Eq 移入实际 EditorStateOperationError；完整 completion 保留到 effects/journal，业务结果与 canonical terminal 分开处理。[单文件后继独审](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-scene-host-completion20-independent-review-v2/review.json) SHA `22a4840c9501e2cfc4e932e4a330b0c9363aef6f83906aee3e9e73ec2a8fd9f5`，复用其余 19 源；未编译运行。
- [ ] 完成 Scene/State/Pointer 在首次 focus/world/Mode 变更前采用纯 validator、真实 Welcome 业务接纳、10 个 State 调用方的 receipt 析构顺序及完整 upper owner carrier。
- [ ] 合法应用完整 caller cohort 后统一提交兼容 Check、普通回归、ABI 与隔离 Release 批次；取得原计划 latency、allocation、retained quota 与真实 RSS/产品性能数值达标证据。

本节只有 material buffer helper 是新增共享 Rust 写入；其余明确标注为私有候选。旧冻结、foreign owner、原测试与性能门保留；异步编译提交后继续功能修复，不持续查询或等待编译。


## 2026-10-01 · 当前源码组合进展（执行与性能验收待完成）

- Scene 四源与 State 两源采用纯 same-activation validator，在首次 mode/world/policy 变更前检查；已接纳的 Closing/Cancelled context 保留原业务结果，无第二次准入。[独立增审](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-editor-ui-scene-context-validation-independent-review-v1/review.json)（SHA `fb10631bb2421ffc3b59b668fcb05dc5cb71ec200f38d3e68da16fcd10ea2b03`） 核对精确 source/dependency，新增真实 context 用例未执行。
- State external10 的真实 receipt Drop 顺序 P2 已由私有 V2 与[非作者修复审查](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-state-completion-external10-independent-review-v2/review.json)（SHA `81cfc22826c1b42f92a82cd39ba304e37fca55408b35ad53c479615b373539af`）关闭；旧 83 个测试名、396 条断言保留，未宣称任何测试执行。Pointer admitted7 已由[独立审查](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-pointer-admitted-leaf-independent-review-v1/review.json)（SHA `6d280b92f3e1a2a54aadec5af9f1c633b0a6776b3d02e0145275e697b04a78c7`）确认 supplied context 的叶子路径，无第二次准入；App full-event 两源仍待完整 Surface heap 和 Play/caller 合成。
- Whole FailedSceneHostOperation 保留真实 inner/outer error、业务 body、TaskTerminal 与计费 handle，成功业务加 Cancelled 不被伪装成失败。[Carrier 生产审查](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-failed-scene-host-completion-carrier-independent-review-v1/review.json)（SHA `ccdcac741541abeed842cc489915fc84de7724731f721a971ca388536963050a`）；[四项因果源码回归增审](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-failed-scene-host-completion-regression-independent-review-v1/review.json)（SHA `642f137e87ed43122e0a65733f5e5d28b3c58bd41c0e97651300f14fdace484f`） 核对 31 项依赖和 62 条断言。前三项覆盖保留 receipt 拒绝及 Drop 后等待实际退休再重试，第四项真实 UI owner 先 Drop 后再次 prepare；均未运行。
- Welcome 五源 business 操作与一源 capture 后继保留真实 installed/failed activation 及原有 projection；[最终 V2 独审](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-welcome-business-operation-independent-review-v2/review.json)（SHA `b1055187d7cf705d2d82eb800699a3b210a78761b1f8d15e1b4b897617a195c9`） 不把表格观察、DTO/noop 或文本诊断当业务成功。此批未新增 business 回归，实际失败恢复和完整窗口发布仍待执行。
- C8 State project 两源 join 目前明确 Applicable=false，缺实际 World/empty-reset payload provider；[不可应用骨架](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime02-editor-state-project-authority-join-successor-v1/manifest.json)（SHA `4825c67c47e7469dee5a99a31be32b918008303f693b528a87f9f661950deab1`） 的临时拒绝路径不作为成功修复。真实字体/Surface opaque 容量、startup no-State 终态证据、独立 ProductCore module/graph 清理、原计划延迟/分配/RSS 门仍开放。

本节 `[x]` 仅表示明确范围的私有源码修复或独立静态审查；`source_reviewed_execution_pending` 不表示已写入共享 Rust、测试通过或性能达标。兼容任务合并提交后继续功能修复，不逐项编译、不阻塞等待或持续查询编译状态。


## 2026-10-03 当前修复与批量验收

本节更新到 V49。Resource51 的三个文件已实际应用到当前 Main；其余修复按私有源码及独立审查范围记录。编译、原测试和性能达标继续待验收。

- [x] Resource51 无过滤查询直接复用当前有序 entries，去掉额外筛选和同序排序；类型过滤分支与其他过滤行为保留。三个原顺序测试补上真实查询对照，另加独立 10K／100K／1M listing profile。实际三文件写入、原字节保护和限定差异检查通过；原索引、资源管理器及 22／24 性能场景保留，未移植 Source6。普通测试与测量尚未执行，输出 Vec 分配没有标为零。
- [x] Core supplied-scope V2 保留原执行器关闭／停止原因优先级，再读取目标 scope；Loader V2 精确绑定该真实 API，pending 发布前保留原 typed refusal。独立源码审查关闭已列依赖问题，相关原测试仍待执行。
- [x] App／Editor 三文件 V3 修复全部 19 个真实配置出生点，包括两处隐藏默认配置的便捷调用；保留 155 个断言表达式、41 个 selector、原 Runtime／Minimal 配置和生产缺失 policy 的拒绝。
- [x] Font／B2 的 791 文件已完成列明范围的独立源码审查与实际来源核对，保留同一个 cooked Header／Vec／adapter；容量拒绝先释放原字节再退款。审查仅覆盖明确列出的用途与调用体，Arc 弱引用尾部、DLL／native、allocator、RSS 和原性能门继续开放。
- [x] World 暂存复制使用同一原 policy 的可失败 try_clone，保留原大小错误优先级、运行容器恢复及真实活跃 schedule；JSON 原定义导出已恢复。Resource／World 两表事务先准备再变更 epoch／事件，拒绝保留原行与 owner，原资源在 mutex 外释放；完整 schedule 交换保留真实绑定。各限定源码复审通过，新增原用例仍未运行。
- [x] Operation 八文件使用 tick 传入的原 Core Compute 调度器自动准入；保留真实 TaskHandle、原错误 Arc、取消／过期原因与一次 slot 释放。独立源码复审通过；操作完成事件没有等同于 Core 账本或 allocator 归零，原 32 并发／4 MiB 编码限制没有当成物理堆证明。
- [x] 三个真实原生收尾用例已封存通过，历史失败保留；自然排空与超时强制清理分别判定。首次八项验证在 Cargo 前被严格清单拒绝，0 项编译／测试执行；补齐两个 cc 版本的 8 个真实 src/target 模块后，29649 项源码清单与八项配置通过限定复审，原 worker／命令保持。
- [x] 修正后的八项异步验证已由一个隐藏 worker 一次启动：两项 all-targets 检查、RHI／Wgpu 各 Debug 与 Release 测试、两项 Cosmic 测试。启动前 D 盘实际余量 37.944 GiB，原 35 GiB 门槛通过；启动日志已封存，0 次启动后结果读取、0 次等待。结果保持 pending，原首次清单失败与容量拒绝历史保留。旧 build 目录压缩仍待完整收据，禁止新编译使用，未把当前余量归因于尚未验收的压缩。
- [x] Editor 的 31 文件修复包完成独立源码复审：84 处旧默认构造移除，保留 203 个 selector、1,076 个断言表达式、原 Compute 构造参数和已加载 World 的 policy。DynamicScene schema／adapter 与同 owner 出生接口也通过限定源码复审；原调用体、目录别名和资源转移寿命保留，测试尚未执行。
- [ ] Native Schedule V1 发现真实 P1：回滚时再次分配 header 可能掩盖最初的拓扑错误；修复新版本进行中。字体 805 文件及 root／standalone Cargo／lock 候选按真实用途合并，保留当前其他域修复；10 处后续调用重叠继续处理。解析器、全套 Runtime／Editor 原测试、DLL／弱引用物理寿命及原性能验收仍开放。
- [ ] Resource51 原三次 6666 对样本的 P95／P99 高于基准分组仍为 4／7、1／3、2／5，原尾延迟没有标成已修复。R9 独立用途版本仍排序，不能套用当前 Main 有序索引的免排序收益。继续按原 count／bytes／join、CPU、RSS、I/O、allocation、p50／p95／p99 及 Editor 原性能门槛验收。

[V49 实际代码、修复与验收状态](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime-editor-current-source-results-and-records-v49/current-source-and-results.json)；[源码复审与异步动作来源](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime-editor-current-source-results-and-records-v49/qualified-source-review-and-dispatch-bindings.json)；[Main 三文件实际写入](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-runtime51-current-main-unfiltered-listing-actual-guarded-apply-20261003-v1/actual-guarded-source-apply.json)；[最近压缩结果离散读取](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-main-resource51-and-font791-qualified-source-milestone-existing-compression-result-v1/existing-compression-milestone-capture.json)。

累计 30 次离散编译结果读取、26 次 terminal 封存、0 次编译等待。勾选只对应明确列出的实际代码应用、私有源码复审或已封存原生用例；普通测试与性能完成标准保持。本次记录写入没有改变 Rust、索引或 tooling；本里程碑已实际应用的 Main Rust 为三个文件。未提交、推送或外部通知，Goal 保持 active。

[本次八项真实异步启动日志](../../../../../.codex/state/session-coordinator/async-validation-batches/offline-candidates/root-actual-native-eight-focused-check-test-one-shot-launch-20261003-v2/actual-hidden-worker-launch-receipt.json)。
