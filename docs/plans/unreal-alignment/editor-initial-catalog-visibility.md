# Editor 首代资产目录：注册宿主未读到预置资产

**范围：**只调查真实工程首次激活后，依赖 Editor 目录的工具能否按精确 locator 读到首代资产。不涉及 Scene 后缀、工具分发实现、预览或一般 watcher 行为。本文记录已有测试与源码证据；不含引擎改动、新测试或产品验收结论。

## 1. 不一致：直接投影通过，注册宿主的首读缺指定行

ZE 固定基线为 [`e89349ca51b4e9d024c194cd3bdf645bd25b6c37`](https://github.com/He-Jiahui/ZirconEngine/commit/e89349ca51b4e9d024c194cd3bdf645bd25b6c37)。执行取自[目录分支快照 `9a271ce0`](https://github.com/He-Jiahui/ZirconEngine/commit/9a271ce02fa669d946d73a6997efc68cd57c255a)；该分支只承载审计目录，不含引擎差异。既有输入清点将选定源码、测试、manifest 和锁文件对齐到上述基线，三次执行前后未变。

三份已有本地命令回执及原始输出显示：Windows x64 MSVC，`zircon_editor` 单包 `--lib`、`test` profile，`--no-default-features`、无额外 feature、`--locked --offline --jobs 1`、`--test-threads 1`。每次单独启动 libtest；退出码 `101` 的两次均已跑到具名断言，不是编译失败。

| 既有 filter／用例 | 原生结果与首个失败点 |
|---|---|
| `::sync_from_project_` 的[五个直接投影用例](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/host/assets/manager/project_sync/tests/mod.rs#L17-L305) | **5 过、0 败；exit 0**。覆盖不变代不重复发布、`.zmeta` 包及复合 Shader 详情、无 artifact 的 Error 行、锁中毒恢复、导入后的 Shader IDE 输出。 |
| `::workbench_menu_open_ui_asset_opens_ui_asset_editor_for_shared_asset` | **1 败；exit 101**。[目录断言:167](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/tests/editor_event/runtime/animation_assets.rs#L158-L179) 未找到 `UI_ASSET_LOCATOR`（`res://ui/menu.zui`）；未进入 `dispatch_event`。 |
| `::asset_open_event_routes_animation_assets_to_animation_editor_views` | **1 败；exit 101**。[循环断言:256](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/tests/editor_event/runtime/animation_assets.rs#L246-L267) 依次要求 sequence、graph、state machine 三个精确 locator；至少一个缺项。panic 正文是循环共用断言，没有输出当前 locator，不能确定哪项首先失败；未进入循环后的 `dispatch_event`。 |

两条非 Scene 用例各自[在打开前真实写入源文件](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/tests/editor_event/runtime/animation_assets.rs#L10-L76)，但其输出不含目录总行数、Runtime `Ready` 或 catalog-input。不能由这两条失败推断整个目录为空、导入失败，或后续动画类型也缺行。

| 证据 | 回执 SHA-256 | 原始 stdout SHA-256 |
|---|---|---|
| 直接投影 | `0336cc9197ad442ef6e7ef5a3cac1caac72e95e4ea5fe24b4d278f01a9d67aa5` | `310cfd0c9c387d3e3d06786ce50bb80b36ded05af34062d3a92966b55cb05a35` |
| UI 宿主 | `b0759c5ee30359c07961bdc1f5ca7648620f14246eec49176ca9023c46c3ce00` | `e52ea9bb8ffd157257febe77c8262649585a929ba904433c15449d4293b07f9c` |
| 动画宿主 | `2d51fc5e20690277ab61b8afd34602d984b28bf482215d120fdf73908a1e7499` | `ab4c45d0bb9e5ad8c41278a0a7e8dc2d017d11ce2f6f8d76a9e9b750a0e8ba36` |

## 2. 影响：基于目录的首个工具请求缺前置

两个 headless 宿主用例均在分发前停住，证明该读取点缺指定行，不证明 GUI 首帧、实际窗口或后续维护仍不可见。[#157 固定记录:96–111](https://github.com/He-Jiahui/ZirconEngine/blob/7657c511d41cf261f4c78f5cd226bdd373a9090b/docs/plans/unreal-alignment/project-scene-suffix-case.md#L96-L111) 是独立的邻接证据：三条真实 Scene 的路径、Runtime `Ready` 和 catalog-input 均已核对；同一注册 Editor manager 初读及显式一次生产 refresh 后都是 `root="" / revision=0 / epoch=0 / assets=0`。这不是 #157 的后缀回归，也不指明某条 watch 分支。[#134](https://github.com/He-Jiahui/ZirconEngine/pull/134) 的[既有 Runtime registry 查询控制](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/tests/pipeline/manager/resource_records.rs#L8-L71) 不覆盖注册 Editor 首代；既有 [PERF-MVP-499/500](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/performance/01-mvp-performance-audit-and-optimization.md#L639-L640) 与[启动 watermark](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/_archive/performance/01/2026-07-30-editor-retained-host-startup-current-review.md#L37-L42) 是相邻工作，不是本次失败的已证原因。

**产品门另计。** F1 [staged create](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/mvp/Stage-MvpProducts.ps1#L1751-L1789) 读取同次创建诊断中[最后一个 `editor_project_open` 标记对应的记录](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/mvp/MvpProjectOpenEvidence.psm1#L29-L54)；记录的是[开项内的早期目录快照](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/host/project_access.rs#L27-L74)，不是后续 UI。`RenderableEmpty` 的[该门](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/mvp/MvpProjectOpenEvidence.psm1#L227-L255) 要求至少 4 个 Runtime starter Ready、无失败、Editor revision/epoch 非零且至少 4 个目录行；4 是模板验收最低数，不是通用目录门槛。晚帧补齐不会改写这一条记录。headless 不是正式验收；本报告没有 F1 实跑结果。`result=completed` 的可能误分类是另一项诊断问题，不纳入本题或修复范围。

## 3. 分歧根因与 UE 的处理

**能定位的是交接缺保证，不能定位这几次的具体拒绝原因。** 直接测试用 `sync_from_project` 喂入已准备的 `ProjectManager`，该入口[无 Runtime generation token](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/host/assets/manager/project_sync/sync_from_project.rs#L29-L46)。[真实 `EventRuntimeHarness`](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/tests/editor_event/support.rs#L189-L235) 则一次打开工程并立即读注册服务，`sync_asset_catalog` 只把已发布快照交给 shell，不负责发布，也不运行 retained 的事件消费。生产 [refresh](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/host/assets/manager/assets/mod.rs#L140-L151) 取得 active snapshot/token；[终端校验](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/host/assets/manager/project_sync/sync_from_project.rs#L236-L343) 可因 Editor 源请求/代过时或 Runtime token 不再匹配返回 `Ok(())` 而不提交；只有提交才换目录并广播 `CatalogChanged`。因此 Runtime 已可查询、refresh 返回成功、消费端已读到首代是三件事；直接投影全绿不覆盖后两者的交接。

[#157 的 0/0](https://github.com/He-Jiahui/ZirconEngine/blob/7657c511d41cf261f4c78f5cd226bdd373a9090b/docs/plans/unreal-alignment/project-scene-suffix-case.md#L96-L111) 是初始值，不是正常发布过一个零行目录：[构建时写入项目根](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/host/assets/manager/catalog_generation/build.rs#L51-L68)，[首代推进 revision/epoch](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/host/assets/generation.rs#L72-L91)。现有输出没有记录进入 refresh 时的 token 与终端分支，不能确认是未取得 active snapshot、Editor 请求失效还是 Runtime token 被拒；更不能指定 watcher 为换代写手。普通 `RenderableEmpty` 的 Shader IDE 生成输出在 watched `assets` 之外，同字节也不会重写文件；这排除的是该直接自激解释，不是所有通知时序。

**UE 生产源码对照（未执行）。** `IAssetRegistry` 区分首次 `OnFilesLoaded` 和附加挂载批也可触发的 `OnKnownGathersComplete`；`AssetRegistry.cpp` 在 game thread 先交付初扫末批资产事件再完成，不保证完成后新事件的相对顺序。Content Browser 的 `FCollectionAssetRegistryBridge` 在初扫中订阅 `OnFilesLoaded`，已就绪则立即做完成处理（用于集合 redirector 引用修正，析构退订）。这里可比的是消费者如何判断初次就绪，不是 UE 有一套同样的 Editor 目录代。

**UE 既有测试（只核源码）。** `TestFramework.CQTest.Helpers.CQTestAssetHelperTests` 的 Editor fixture 用 `AssetTools.CreateAsset` 建 `MyBlueprint`，显式 `AssetCreated` 注册到真实 registry，清退时删除：

| 原用例 | 原断言与可借鉴的控制 |
|---|---|
| `FindAssets_WithFilter_ReturnsAssetArray`（168–179） | 按精确包名查得恰一项，`GetAsset()` 对象有效；可借鉴精确身份的正控，不断言通知顺序。 |
| `FindAssetPackageByName_WrongName_ReturnsNullOpt`（120–130） | 同一有效包 filter 下用错误对象名得到 `NullOpt`；可借鉴身份错配的负控。 |

CQTest helper 在初扫仍 loading 时可同步 `SearchAllAssets(true)`，不是冷启动交付测试。`PlayMapInPIETests::GetTests` 也是辅助对照：启用自动查找时只在 `!IsLoadingAssets()` 分支枚举 on-disk `UWorld`；仅 Editor 构建的非 Editor（`-game`）分支可同步扫描一次。它不测试 Editor 目录首代。未找到这些源码中与 ZE 注册宿主首代完全等价的 UE 原测。

| 本地 Epic 参考树相对路径及范围 | SHA-256 |
|---|---|
| `Engine/Build/Build.version:1–10`（`6.0.0 / UE5 / CL 0`，非上游提交标识） | `3dcd8b6654872d013f2b00aa5f0ef1e0db89d4e8627ce820554675a27f228ea6` |
| `Engine/Source/Runtime/AssetRegistry/Public/AssetRegistry/IAssetRegistry.h:983–997,1029–1039` | `08c154ce35bced25eed8c415bbf0f6386905bcf978d35a92711fa1d1bf2c336f` |
| `Engine/Source/Runtime/AssetRegistry/Private/AssetRegistry.cpp:5330–5355,10979–11043` | `071f916324ff4e714329416559b3dce37be493b9df22a3dcb98988c58a4ca9fc` |
| `Engine/Source/Editor/ContentBrowser/Private/CollectionAssetRegistryBridge.cpp:142–177` | `52ae8765567cc2820fab888282ad038d3d042106597cbaec327406ae72c05e9c` |
| `Engine/Plugins/Tests/CQTest/Source/CQTestTests/Private/Helpers/CQTestAssetHelperTests.cpp:18–54,120–130,168–179` | `764cf587bfb5462d9c416f69b2a0739b0fd26e97d25a3eb0b2200f65bf0df567` |
| `Engine/Source/Developer/CQTest/Private/Helpers/CQTestAssetHelper.cpp:13–39,85–100` | `a02260f755b389fc11b81085b57476e43b9bd0feaf74ee90109e08cdb17ae0e3` |
| `Engine/Plugins/Tests/EditorTests/Source/EditorTests/Private/UnrealEd/PlayMapInPIETests.cpp:80–127` | `370a3f182121c52de38526078a6e560fa8c0c7d2318f93edfbe4d1232bb10836` |

## 4. Rust 做法：以提交结果和可读快照为准

保留不可变 `Arc` 快照和短终端 fence：按 operation root、activation 与 catalog-input 序号[校验 Runtime token](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/pipeline/manager/assets/runtime.rs#L114-L135)，校验通过才原子安装 Editor index/catalog，发布后再通知并读取快照。`Ok(())` 只表示无返回错误，不表示首代已发布。用发布证据区分仍未交付、当前代无需更新、候选失效与已提交；不把 Runtime 序号、事件游标和 Editor revision/epoch 当成同一计数器。

## 5. 接口与 ECS：可见性是服务契约，不是实体存储差异

UE 对照是进程级 registry 的包/对象路径查询及 game-thread 通知；ZE 用项目 Runtime owner 提供 `res://`/UUID 与 activation 下的源代，注册的 [EditorAssetManager](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/host/module.rs#L69-L98) 再向工具提供独立目录投影。Editor 世界/ECS、默认场景文档或 Runtime 中已有资源都不能代替该投影。应迁移的是「明确就绪点后，按身份检索」的行为，不复制 UE 全局等待、包对象模型或回调 ABI。

## 6. ZE 后续方案：先观测同一 activation，再决定修哪一层

1. 在同一真实、隔离的工程 activation 上，于开项前订阅 Runtime 与 Editor 的[实际变更流](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/host_lifecycle/startup/resources/events.rs#L19-L53)。在磁盘预置合法小写 Scene 与非 Scene 正控，只正常开项一次，使用已有事件消费路径。做一次有界的关联观测：核对精确 source/locator、Runtime `Ready` 与 catalog-input，记录 refresh 的 root/activation/token、终端分支、真实 `CatalogChanged` 与对应可读快照；无匹配交付则保留超时和各层证据。仅在同 activation 的 token 变化与实际 watch commit 对齐后，才归因该写手。
2. 若只缺夹具中的真实事件交付，修夹具，不改生产；若证实生产在同一 activation 丢了首代交付，才在现有注册宿主的发布或消费边界修复所证路径。保留过时/跨 activation 拒绝与不变代不重发；不靠重开、sleep、轮询/刷新循环、强扫、捏造目录行或绕过 fence 取得通过。
3. 后续先补上述真实交付回归与精确匹配正负控，再用相同用例验证修复；产品另跑原 staged create 的 F1，核同一次早期日志、呈现/捕获与进程清退。UE 对照仅是源码与测试设计依据，不报 UE 执行或通过。

## 7. 同一真实工程的首次打开与一次刷新：原生 RED（2026-11-18）

[测试提交 `194aa86f31fa276d58c80214cd0f6da7e3645e30`](https://github.com/He-Jiahui/ZirconEngine/commit/194aa86f31fa276d58c80214cd0f6da7e3645e30) 仅接入一个回归；原引擎、已有两个工具打开用例均不改。主机是 Windows x64/MSVC，Rust 1.94.1；`test -p zircon_editor --no-default-features --lib ::editor202_ --locked --offline --jobs 1 -- --test-threads 1 --show-output`。按名字子串选中**恰一例**，实际 **0 过、1 败、0 忽略、10,540 过滤，exit 101**。不是编译失败或静态推演。[一次开项、订阅和取样](https://github.com/He-Jiahui/ZirconEngine/blob/194aa86f31fa276d58c80214cd0f6da7e3645e30/zircon_editor/src/tests/editor_event/runtime/first_catalog.rs#L34-L174) 走真实 RenderableEmpty 创建、原 `.zui` writer、同一 Core 的注册服务，使用隔离用户目录。两个订阅在开项前建立，随后只额外调用一次生产 refresh；这与原来两个工具测试的惰性解析顺序不同，不把这几次运行当成完全相同的时序。

| 同一次执行中的观察点 | 实际观察 |
|---|---|
| Runtime 发布与源 | 收到 8 条 `Added`，含 `res://scenes/main.scene.toml` 和 `res://ui/menu.zui`；该次有界读取 `error=None, pending=0`。`Added` 不是 Ready、也不是 Editor 的 `CatalogChanged`。 |
| 标准小写场景正控 | 物理源、Runtime registry 和其不可变 catalog input 的路径/类型/状态对齐。`Scene/Ready`，制品实际解码为 `ImportedAsset::Scene`（raw 605 字节）。两个已取 Runtime 快照的[直接 Editor 索引](https://github.com/He-Jiahui/ZirconEngine/blob/194aa86f31fa276d58c80214cd0f6da7e3645e30/zircon_editor/src/tests/editor_event/runtime/first_catalog.rs#L231-L293) 均按该 URI 解析到同一资产 UUID。 |
| 同一 activation；旧 token 与 watch 计数 | 同一工程根、同一 activation；Runtime 输入序号 **12 → 19**。显式 refresh 前旧 token 是 `Current`，之后 `Superseded { newer_same_project_generation: true }`。watch 计数按 `(batch,reconciliation,committed,failed)`：开项前 `(0,0,0,0)`，显式 refresh 前 `(9,9,9,0)`，之后 `(16,16,16,0)`；refresh 自身返回 `Ok(())`。这里是**取样区间**的观测，不是逐事件逐提交的一一对应。 |
| 注册 Editor 与实际打开日志 | 首读和显式 refresh 后均为 `root="", revision=0, epoch=0, rows=0`；两个读取窗口中均没读到 `CatalogChanged`。同次 opener 的真实内存日志 sequence=1：`result=degraded`，registry 为 8 项/7 Ready/1 failed/1 diagnostic，Editor 三个数字为 0，`settings_source=persisted-v1`。不是 [#201](https://github.com/He-Jiahui/ZirconEngine/pull/201) 所定义的健康 Runtime 却误标 `completed` 的现场。 |
| UI 是另一个前置 | 真实 `.zui` 路径可解析，实际 importer 报 `UnsupportedFormat`（没有已注册的 `.zui` 导入器）；Runtime 有 `Data/Error` 行、无 artifact，有 catalog input。**不能把它算成 Ready UI 行的投影失败或已验证的工具路由**。本案不插入 mock 或为通过断言临时注册插件。 |

首先失败的是[首个已注册目录须属于本工程且已初始化](https://github.com/He-Jiahui/ZirconEngine/blob/194aa86f31fa276d58c80214cd0f6da7e3645e30/zircon_editor/src/tests/editor_event/runtime/first_catalog.rs#L328-L338)，不是 Scene 解析、默认场景加载、UI 导入器或后续工具操作。由此，在有真实 `Added` 和可解码小写 Scene 的前提下，**未发布首代可以复现**。外部已取 token 确实过期；未观测 `sync_from_project` 内部拿到的 token 及终端分支，也没有触发 reconciliation 的原始 OS 事件/路径。尚不能断言 7 次 watch commit 就是 7 次内部 fence 拒绝、某特定生成文件触发自激、目录永久为空或真实窗口有相同表现。

**下一步只补这一缺口：** 在同一开项的 open/显式 refresh 两段，获取已有 profiling 的实际 publish/superseded 分支；再将 scoped watcher 的事件种类、路径/无路径、overflow/错误与确切提交关联。保留同一 activation、旧代拒绝和标准 Scene oracle；不增加重试或用「没有等足」解释。不对本次源码未确认的写手做生产修复。真正的首帧及 staged F1/F5 仍是独立门。

本次普通命令回执 SHA-256 `42556af829fd1ceebe13fcfcb0e16ec370dcf3be015041d2691cccd6c3fdf8b6`；stdout 3,026 字节 `4aed4923614050eafd48d80f9ebebe2e47c2146b6f81d6a361bdd2cf0a2c7b5d`，stderr 992,263 字节 `abcd48d14b92e66f5a9f744e7e52ff2ae6048bf355380dbbbce0ecd3c0b9e1c4`。49 项选定输入运行前后复核，并非全源码或进程树的形式化封存。没有 UE、GUI、全包或产品验收结果。

## 8. 同一显式刷新前后：真实 Runtime 输入的净差分（2026-11-18）

[仅加观测的测试提交 `e385c60c5b6a37fcedd27b8676263f9eb0995cc0`](https://github.com/He-Jiahui/ZirconEngine/blob/e385c60c5b6a37fcedd27b8676263f9eb0995cc0/zircon_editor/src/tests/editor_event/runtime/first_catalog.rs)：原 402 行全部按原序保留，仍只有一次显式 refresh；复用已有的前后 Runtime 项目/令牌/输入快照，用原 fixture 的 operation root 核身份。原用例照常创建全新的 RenderableEmpty 工程；新增观测不另建工程，也不额外刷新、重开、等待、模拟日志或修改生产。Windows x64/MSVC，Rust 1.94.1，`test -p zircon_editor --no-default-features --lib ::editor202_ --locked --offline --jobs 1 -- --test-threads 1 --show-output`，实际 **0 过 / 1 败、0 忽略、10,540 过滤**，exit 101。原行为门结果：`[open catalog] no initialized generation for the created project`。

**本次观察：** 同 root/activation，输入代 11→17、1 Modified，两端内容仅两处 mtime 槽位不同。原外取 token `Some(Current)->Some(Superseded { newer_same_project_generation: true })`，same activation `Some(true)`，watch `(batch,reconcile,commit,failed)` 在 closed/open/refreshed 的三份快照为 `[(0, 0, 0, 0), (8, 8, 8, 0), (15, 15, 14, 0)]`，显式 refresh 返回 `Ok(())`。已注册 Editor 前/后 `(revision, epoch, rows)` 分别为 `(0, 0, 0)` / `(0, 0, 0)`，观测到的两段 `CatalogChanged` revisions `[]` / `[]`。

报告的 Runtime `delta_since`：added/removed/modified/renamed 分别为 0/0/1/0；manifest 与 package registry 均未变化。唯一变动的 shader 根 ID 是 `22da033c-63f7-874d-82ae-d29472fe0540`。真实打开日志仍为 `degraded`（8 个资产，7 Ready、1 Error）；小写 Starter Scene 制品解码为 605 B，两端同一 UUID。`.zui` 在本 headless 组成中仍为 `UnsupportedFormat`；它不是这两个端点之间发生净变化的那一行。

| 变化记录 | 端点中不同的字段 | 只忽略行 / 只忽略 meta / 同时忽略两时钟后行是否相等 |
|---|---|---|
| `modified` `res://shaders/pbr_shader` → `res://shaders/pbr_shader` | `source_mtime_unix_ms` 1795005919449 → 1795005920824；`meta.source_mtime_unix_ms` 1795005919449 → 1795005920824 | false / false / true |

独立按全部 ID 和全部记录/项目字段比较。`single_mtime_only=not_a_single_mtime_field`、`both_mtime_slots_explain_net_diff=true`。在这两份不可变快照之间，单独抹掉其中一个槽仍不等；两槽一起抹掉，其余所有输入才相等。

这里验证的是**两个端点的净变化**；不等于收到每一笔 OS 通知、抓到每次 generation 的内容或看到 Editor 内部丢弃候选的分支。即使 locator 与 [#204](https://github.com/He-Jiahui/ZirconEngine/pull/204) 的复合描述符机制相符，也不将另一独立工程的 writer 归到本次开项目；不削弱 fence，不从 `Ok(())` 推导已发布。

原生回执 `4b90c106146871242fd104742787a1d8e82fb546f1d55fdaf94737226ba76d48`；stdout `1f184fd312163d6d8cd79aeec62bbe21158607db08021d79abfd067da218ba04`（3,968 B）、stderr `506a51c4799e2c32b2c47b0a929c6dcd9dcde0f718d383bb829b14efcb3507a8`（992,262 B），51 项选定输入复核（覆盖上一见证的 49 项）。未执行 UE、GUI 首帧或 F1/F5 验收；没有产品修复。
