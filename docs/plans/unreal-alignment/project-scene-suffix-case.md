# 项目打开/保存：已导入 Scene 的大小写后缀被打开边界拒绝

> 当前状态：已在 Windows 原生测试复现并修复两处后缀校验。精确 5 案由 **1 过 4 败 → 4 过 1 败**；真实宿主目录前置仍失败，不能宣称从窗口可见、整个 Editor 已通过。第 7–8 节给出结果；下列六点保留原始调查时的判断。UE 的本机路径与文件 SHA256 只标识参考树，未执行 UE；私有运行日志不随 PR 发布。

- 初版裁决（历史）：一条源码可证的窄缺陷，建议优先级 P2；不是架构差异或缺失能力。当时只完成调查，尚无 Draft PR、测试或引擎改动。
- 本地只读基线：`cb495bb1c952a24c86f4513b8099bc4942c3a87d`。按 lead 核定，ZE 引用使用已公开祖先 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`；只读 `git diff --exit-code e893… -- zircon_runtime zircon_editor zircon_runtime_interface zircon_plugins/prefab_tools` 无差异。本地后续流程退役与外来技能修改没有夹带。
- UE 身份：[Build.version:1–10](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Build/Build.version#L1-L10>) 是 6.0.0、BranchName UE5、Changelist 0；不推断发布版或提交身份。

## 六段问题说明

## 1. 差异、符号、精确位置与可达性

ZE 内建 `zircon.builtin.toml.scene` 在 [asset_importer.rs:132–135](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/importer/ingest/asset_importer.rs#L132-L135>) 注册 `.scene.toml`；`AssetImporterRegistry::best_full_suffix_match` / `MatcherIndex::slots` 在 [registry.rs:169–184、407–412](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/importer/registry.rs#L169-L184>) / [相等比较:407–412](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/importer/registry.rs#L407-L412>) **忽略 ASCII 大小写**。
扫描通过 [sources.rs:238–245](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/scan_and_import/sources.rs#L238-L245>) 建立源 URI，[source_uri_for_path.rs:63–74、96–104](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/source_uri_for_path.rs#L63-L74>) / [组件保留:96–104](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/source_uri_for_path.rs#L96-L104>) 没有把文件名后缀转小写；编辑器 [record.rs:13–17](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/host/assets/manager/catalog_generation/record.rs#L13-L17>) 继续保留 locator 和 kind。
于是合法空场景源 `assets/levels/Upper.SCENE.TOML` 可以被选择为 Scene importer，而 `scene_entries_from_catalog` 经 `parse_project_scene_uri` 的区分大小写 `ends_with(".scene.toml")` 在 [scene_picker_session.rs:250–261、309–320](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/scene_picker_session.rs#L250-L261>) / [校验:309–320](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/scene_picker_session.rs#L309-L320>) 静默滤掉它。
绕过 picker 也无济于事：`EditorManager::open_scene_document` → `SceneDocumentRoute::open_while_routed` → `ProjectAuthority::open_scene` 的 [manager:288–306](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/host/editor_manager_project.rs#L288-L306>) / [route:113–120](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/core/document/scene_route.rs#L113-L120>) / [authority:336–368](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/core/project/scene_document.rs#L336-L368>) 又作同一大小写敏感拒绝。
UE 实际生产链是 [FileHelpers.cpp:3271–3294](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/FileHelpers.cpp#L3271-L3294>) `FEditorFileUtils::LoadMap` → [PackageName.cpp:2042–2051](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/Misc/PackageName.cpp#L2042-L2051>) `TryConvertToMountedPathComponents` → [PackagePath.cpp:129–148](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/Misc/PackagePath.cpp#L129-L148>) `ParseExtension`，以 `IgnoreCase` 识别 map 扩展名。

## 2. 有界实际影响

输入限于本来可由 ZE Scene importer 接受、后缀大小写不是严格小写的项目内 `res://` 场景，例如 `.SCENE.TOML`、`.ScEnE.ToMl`。Open Scene 会把已经存在的 Scene catalog 行当作无效 URI 丢弃，可能显示“无场景”；通过 typed open API 提交则返回“must end in .scene.toml”，不会安装该场景。当前小写场景不受影响；本问题没有证明数据损坏、跨项目读写、崩溃或所有项目打不开。特别是 [editor_project_document_load.rs:39–46](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/workbench/project/editor_project_document_load.rs#L39-L46>) 的默认场景启动读取与 [source_document.rs:103–125](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/world/project_io/source_document.rs#L103-L125>) 不依赖该 suffix gate，不能把 picker/authority 拒绝扩大成所有启动入口都拒绝。创建/Save As 共用同一个敏感校验，也表现为输入接受范围不一致，但不另计第二问题。

## 3. 根因与 UE 实际解决

根因是 Scene importer、retained picker、core authority、持久化 writer 对同一“是否 Scene 后缀”的谓词不一致。持久化 writer 已在 [editor_document_transaction.rs:372–383](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/editor_document_transaction.rs#L372-L383>) 使用 ASCII 小写后的后缀识别；不是 AssetUri 自动规范化成小写后又出错。[locator.rs:340–348、405–423](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime_interface/src/resource/locator.rs#L340-L348>) / [组件处理:405–423](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime_interface/src/resource/locator.rs#L405-L423>) 保留普通组件大小写。UE 通过上段生产调用链共享 `FPackagePath::ParseExtension` 的不敏感识别，避免每个打开入口重新用敏感后缀判断。这里只借鉴格式分类一致性；没有声称 UE 的 Linux 文件系统会把任意大小写物理文件当同一文件，也不把 UE `.umap` 二进制或 package 命名搬入 ZE。

## 4. Rust 方案，不造架构

保持已有 AssetUri、ProjectAuthority、SceneDocumentRoute 和 source-revision/路径租约，把两处现有 Scene 后缀 gate 收窄地改为 **只对 ASCII 格式后缀**作 `eq_ignore_ascii_case` 等价判断（可用安全 `str::get` 尾切片，或与现存 writer 一致的本地判断）。保留原 locator 的全部字节、UUID 和物理路径；禁止把整个 URI 转小写，否则会改变大小写敏感文件系统上的身份。`res://` 所有权、无 label、真实父目录/链接检查、dirty/session admission 均不放松。无需新增注册中心、文档服务或场景架构。

## 5. ABI / ECS 差异

UE map 是 UObject/UPackage；ZE 是保留大小写的 AssetUri + 项目 catalog + Scene/World，二者身份不可互换。本草案只调整 Rust 内部校验谓词与测试，不改变 RuntimeInterface DTO/ABI、资源 UUID、ECS entity ID、component codec、序列化布局或持久化版本。UE original test 的 package extension 枚举只能作分类契约参照，不是 ZE 文件格式或 ECS 测试的原样迁移。

## 6. ZE 改法仅草案及顺序

先由 lead 在已核定公开基线建立本问题独立 Draft（问题说明先行；本轮不执行）。之后只在当前 picker 测试和 ProjectAuthority 场景测试补 `.scene.toml` / `.SCENE.TOML` / `.ScEnE.ToMl` 对照，证明“真实扫描后 kind 为 Scene → picker 保留原 locator → authority 打开相同源”，同时保留 `.scene.toml.backup`、非 res、label、逃逸路径拒绝与 dirty/stale ticket 控制。原入口先 RED 后仅修改两处敏感 gate，再复跑同一组；后续是否把既有 writer 的本地谓词统一命名属于实现时小选择，不是本轮新架构任务。**测试、Cargo、UE 可执行程序及编辑器产品操作均未运行；当前是确定分支的源码推导，不是已执行复现。**

## UE 原有用例优先的测试表

| 类型 | UE 路径/行、原测试名 | 输入 | 原期望 | ZE 既有落点 / 本问题使用方式 |
|---|---|---|---|---|
| UE 已有控制 | [PackagePathTest.cpp:39–63、131–135](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/Misc/PackagePathTest.cpp#L39-L63>)；`System.Core.Misc.PackagePath` / `RunTestParseExtension` | 基础路径 + `LexToString(Map)`，即标准 `.umap`；并遍历其它既有扩展名 | Map 分类；`ExtensionStart` 等于无扩展路径长度 | [scene_picker_session 测试:34–60](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/tests/scene_picker_session.rs#L34-L60>) `scene_picker_catalog_exposes_only_valid_project_scene_assets` 已保留合法小写 Scene/非法伪 Scene 对照。只复用分类意图，不声称已有 upper-case UE 测试。 |
| UE 已有控制 | [PackagePathTest.cpp:77–102](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/Misc/PackagePathTest.cpp#L77-L102>) 同一测试 / `RunTestIsPackageExtension` | `.umap` / `umap`；`.umapBarglWargl`、空、点等反例 | 标准 map 后缀有效，拼接伪后缀无效 | [scene_document 测试:194–219](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/core/project/tests/scene_document.rs#L194-L219>) `project_authority_rejects_a_non_scene_asset_before_loading_it`；追加伪后缀控制而不放松类型边界。 |
| **源码推导契约，非迁移** | 原测试没有显式 `.UMAP` 变体；依据 [PackagePath.cpp:141–147](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/Misc/PackagePath.cpp#L141-L147>) 的 IgnoreCase 与 ZE importer 自己的 IgnoreCase | 实际项目合法空 Scene 源分别用小写、大写、混合后缀；原 locator 不变 | 都由同一个 Scene importer 接受并能从 picker/authority 打开；拒绝 backup/label/非 res/逃逸路径 | [picker 测试:34–95](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/tests/scene_picker_session.rs#L34-L95>) 与 [authority 测试:15–54](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/core/project/tests/scene_document.rs#L15-L54>)；后续 Draft 内补，不在本轮写测试。 |
| ZE 既有保存控制 | 无对应 UE uppercase 保存原用例 | 打开次场景再显式保存 | 默认场景源字节不变 | [document_roundtrip.rs:367–401](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/tests/workbench/project/document_roundtrip.rs#L367-L401>) `saving_an_explicit_scene_target_never_overwrites_the_manifest_default_scene`；防止为解决后缀问题误改保存目标。 |

搜索范围：完整精读本地 PackagePath 原测试；查看 PackageName 内嵌 `TryConvertToMountedPath` 用例、UnrealEd/Private/Tests，以及 Plugins/Tests/EditorTests 中 LoadMap/SavePackage 调用。没有找到显式 uppercase map-extension 的原用例，不发明测试名、不把上述新变体写成“已迁移 UE 单测”。

## 模块 / owner / 去重 / 未清点

| 项 | 已读结论 |
|---|---|
| UE 模块声明 | [Projects.Build.cs:12–31](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Projects/Projects.Build.cs#L12-L31>)：Core/Json，特定 Editor receipt 路径有 DesktopPlatform；[CoreUObject.Build.cs:26–44](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/CoreUObject.Build.cs#L26-L44>)：Core/Json/Projects；[UnrealEd.Build.cs:10–15、58–69](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Editor/UnrealEd/UnrealEd.Build.cs#L10-L15>) / [依赖:58–69](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Editor/UnrealEd/UnrealEd.Build.cs#L58-L69>)：Editor-only，依赖 Engine/CoreUObject/Projects。 |
| 项目级调用链 | UE [ProjectManager.cpp:28–48](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/ProjectManager.cpp#L28-L48>) 成功加载 descriptor 后安装 CurrentProject；[ProjectDescriptor.cpp:257–279](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/ProjectDescriptor.cpp#L257-L279>) 保存 descriptor。ZE [open_project.rs:25–33](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/core/project/authority/open_project.rs#L25-L33>) → [manager/open.rs:31–67](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/open.rs#L31-L67>)；场景/工作区保存用既有 durable transaction，不把 manifest/save 与 scene/save 混为一谈。 |
| ZE owner | 行为修复限定 `zircon_editor` 的 scene picker 与 `core/project`；`zircon_runtime` importer/writer 是对照依赖，不建议修改。现有 [scene_save.rs:78–123](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/core/document/scene_save.rs#L78-L123>) 的 source revision、durability、Saved gating 保留。 |
| 去重 | 已读 [PR 快照](<D:/CodexData/worktrees/ze-ue-audit-20261117/reports/ue-module-audit-20261117/github-pr-snapshot.json#L113-L130>)、#133/#134合入文件、[Editor61:140–195](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/optimize/zircon_editor/61-editor-scene-document-authoring-world-open-new-reload-save-close-dirty-transition-autosave-recovery-multi-document.md#L140-L195>)。旧“Save默认场景/dirty切换无保护”等报告不能直接照抄为当前缺陷，已有相应显式保存/dirty/stale ticket控制。后缀大小写是不同根因；在已查本地文档和近期PR快照中没有同根因项。 |
| 未清点 | 未审查 Hub 的所有创建/安装/分发路径、所有平台文件系统别名、UE SavePackage 完整事务/原子性、全部 autosave/recovery、所有项目插件/版本迁移。此条不代表这些领域通过。 |

## 验证门与依赖（均未执行）

1. 当前仅完成源码、行号、模块依赖、Git 基线等价性和 UE SHA-256 取证；**没有 Cargo/UE/GUI/测试运行结果**。
2. 后续最小门：`zircon_editor --lib`、defaults off，过滤 `scene_picker_catalog_`、`project_authority_opens_` / 新增 mixed-case case、上述显式保存控制；Windows/MSVC 下跑现有 Windows-only authority fixture。[编辑器 manifest:28、40–51](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/Cargo.toml#L28-L51>) 会依赖 runtime `target-editor-host` 与 test-support，不能声称是无引擎依赖的纯 parser 测试。只有重新获准执行时才声明确切命令；本轮禁止 Cargo。
3. 用现有真实 project fixture扫描合法空 Scene，分别验证导入 kind、catalog原 locator、picker可见性、authority读同一源；依赖本机工具链、已锁依赖与合法 `D/E/F:/cargo-targets` 输出根。真实 UI Open Scene 是随后产品门，不以静态 if 分支代偿。
4. UE 原测试需 WITH_DEV_AUTOMATION_TESTS；本机 UE 源没有可核实发布身份，本轮不配置、不编译、不执行它。

## UE 源文件 SHA-256（仅短说明，无 UE 实现复制）

| 文件 | SHA-256 |
|---|---|
| [Build.version](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Build/Build.version>) | `3DCD8B6654872D013F2B00AA5F0EF1E0DB89D4E8627CE820554675A27F228EA6` |
| [Projects.Build.cs](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Projects/Projects.Build.cs>) | `5A15D34F1450FD8006A41B3124F06F589FF1189DA759A53D5C16B00B55ED6302` |
| [ProjectManager.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/ProjectManager.cpp>) | `102DAD6796F1334ECAE7D4207AD9CBC5C7393E894BA252591F51523F029DD3BA` |
| [ProjectDescriptor.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/ProjectDescriptor.cpp>) | `284891BAE78D7355EACCB43D6C093EE23C43DEE4F2F910653CF20178BC34A4AC` |
| [CoreUObject.Build.cs](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/CoreUObject.Build.cs>) | `CB5FAB6F1B508E0ED8C2D712E24D097B91D317ED18A72F8CD36074BC763B20FE` |
| [UnrealEd.Build.cs](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Editor/UnrealEd/UnrealEd.Build.cs>) | `B5D1654A06CE299E7B156CFCA3C554F330F9D53ED17496728999E74909442B42` |
| [FileHelpers.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/FileHelpers.cpp>) | `511FF28988F15917EADD419E5EC04841FA54576E04AC138AA6596C471E1D418B` |
| [PackageName.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/Misc/PackageName.cpp>) | `B9BD03BFDBBD0CD78FDB9F12E38A7E4FA789DA548E30ACF6C94A77D3AECFF2A7` |
| [PackagePath.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/Misc/PackagePath.cpp>) | `154953D54E649B5DCF1B5D2515C5C4837CD0AA39758FB28747513B61694129CF` |
| [PackagePathTest.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/Misc/PackagePathTest.cpp>) | `B85AD0E7BC8D3DBDAB69BF11F4DD8C1A486EA28172A82D16F662E032772B73A9` |

## 7. 首次原生复现：导入成功，Editor 拒绝（2026-11-18）

以上第 1–6 节保留问题 Draft 初版。本节在原实现 `029b2fbea00c3daa1e2c028aab79a14eeba0a0e9` 上只追加 5 个用例，没有生产改动。本机重新核过 10/10 份 UE 源文件。UE 原测验证标准 `.umap` 和伪后缀；大写是从分类源码推导的 ZE 变体，不是 UE 实跑。UE 的 Editor 打开流程还可从包名重新构造标准后缀，不据此承诺所有文件系统的大小写别名。

| 用例 | 在原实现上实际走到的位置 |
|---|---|
| [真实项目导入 → typed open](../../../zircon_editor/src/core/project/tests/scene_document.rs#L260-L354) | 三个不同文件名各有一个不同标记。一轮扫描后三个原 URI 均为 `Scene`、`Ready`，磁盘产物可解码并读出各自标记；小写从同一源打开并读回正确 World。`MiXeD.ScEnE.ToMl`、`UPPER.SCENE.TOML` 都得到 `SceneTarget { reason: "scene asset URI must end in .scene.toml" }`。有效源字节的备份及带 label 的 URI 均被拒绝。 |
| [受控目录 → Open 请求](../../../zircon_editor/src/ui/retained/app/tests/scene_picker_session.rs#L311-L335) | 小写保留原 URI；目录行虽是 `Scene`，混合、全大写和长文件名却被漏列。受控行不是导入证据。 |
| [Create 请求](../../../zircon_editor/src/ui/retained/app/tests/scene_picker_session.rs#L391-L402) | 小写通过；`res://Levels/É.ScEnE.ToMl` 在构造请求时被后缀校验拒绝。没有在此用例中创建或保存文件。 |
| [负控](../../../zircon_editor/src/ui/retained/app/tests/scene_picker_session.rs#L338-L389) | 通过：不同资源类型、备份、label、非项目 URI、短路径及 UTF-8 边界均不能被选作完整场景。 |
| [首次真实宿主目录 → Open](https://github.com/He-Jiahui/ZirconEngine/blob/5d2f18ec35bab25a93c836992262a4093a000b8a/zircon_editor/src/ui/retained/app/tests/scene_picker_session.rs#L404-L449) | **未到后缀断言。** 工程打开后，夹具取得的目录连 `res://Levels/Cobalt.scene.toml` 都没有，在查小写原 locator 时失败。不能称作第四个后缀复现，也不能声称真实宿主 picker 已经验证。 |

Windows x64/MSVC，Rust 1.94.1，`cargo test -p zircon_editor --no-default-features --lib editor157_ --locked --offline --jobs 1 -- --test-threads 1`，`test` profile。该过滤词还命中既有 Inspector 的 3 案：2 过、1 性能忽略。完整输出 **3 过 / 4 败 / 1 忽略 / 10537 过滤，exit 101**；其中新增 5 案为 **1 过 / 4 败**，后者仅 3 项证明后缀行为，1 项是目录前置。原生命令回执 SHA-256：`f546f55b27516f93cd0b0ae6df839491f18f24c4dc3019a31aa54fa113485c42`。未运行全 Editor、用户窗口、UE 或跨平台产品测试。

## 8. 修复与同一组回归：后缀行为已纠正，宿主目录仍未通过

[第二次测试基线](https://github.com/He-Jiahui/ZirconEngine/commit/cc0f798f3deb59f37db91fabd982286b02edf80e)未改引擎，只加强了[真实宿主用例的前置及诊断](../../../zircon_editor/src/ui/retained/app/tests/scene_picker_session.rs#L430-L529)：核实三个物理文件的拼写与内容、同一活动 Runtime 的三个精确 `Ready` 记录及其目录输入，然后对注册的 Editor 服务做一次正常刷新。其余四案和共享的选择/提交断言保持原样。

[生产提交](https://github.com/He-Jiahui/ZirconEngine/commit/c5109c91f49ce02fe9eb9fe805f51c9ca5bde542)仅修改 Editor 内部：[ProjectAuthority 的完整后缀比较](../../../zircon_editor/src/core/project/scene_document.rs#L349-L380)按字节安全截取末尾，只对 ASCII `.scene.toml` 忽略大小写；[picker](../../../zircon_editor/src/ui/retained/app/scene_picker_session.rs#L309-L320)复用同一判定。文件名与目录名不作转换。`res://`、无 label、kind、文件保护与提交票据未放松；其余改动只是一处 crate-private 导出。没有改导入器、目录发布、保存格式、ABI 或 ECS。

| 实测范围（通过 / 失败 / 忽略） | 原引擎 | 修复后 |
|---|---:|---:|
| 相同 5 案，精确过滤 `::editor157_` | **1 / 4 / 0** | **4 / 1 / 0** |
| 既有 picker 测试模块（含 4 项新增用例） | 未整组复跑 | **12 / 1 / 0** |
| 既有工程场景文档模块（含 1 项新增用例） | 未整组复跑 | **7 / 0 / 0** |
| 显式保存不覆盖 manifest 默认场景 | 未单独复跑 | **1 / 0 / 0** |

- **3 项由失败转为通过**：同一真实工程三种拼写的场景均可按原名从源读出正确标记；受控目录可保留大小写提交 Open 请求；Create 可构造混合或全大写目标请求。1 项拒绝条件在两侧均通过。显式保存控制验证目标隔离，不等于已经做过大写路径的窗口 Save As。
- **第 5 项仍失败，未忽略。** 精确对照两侧均于[同一查小写行断言](../../../zircon_editor/src/ui/retained/app/tests/scene_picker_session.rs#L496-L529)失败：活动 Runtime 的三条输入已就绪（两次均观察到 generation 12），Editor 初次读取及刷新后的快照均为空 root、revision 0、epoch 0、0 行。候选的 picker 整组又命中同一断言，该次 Runtime generation 10。用例都未走到 live picker 的后缀判断；没有从这些观测确定目录为何未发布，也不重试到绿。宿主的正常[打开时刷新](../../../zircon_editor/src/ui/host/project_access.rs#L21-L39)与[事件刷新](../../../zircon_editor/src/ui/retained/app/assets/refresh.rs#L106-L119)仍应独立调查。
- 准确的配对命令为 `cargo test -p zircon_editor --no-default-features --lib ::editor157_ --locked --offline --jobs 1 -- --test-threads 1`。工作树、工具链、特性、输入测试字节相同；两侧均完成 5 案、10540 项被过滤，均 **exit 101**。两个原生回执 SHA-256 分别为 `d3297ce981f7b19ea025e8327f87fbae70609931db50c549e7a62df0eacf7244`、`a4b99aa9d37222450839a93a19493371ad6b85deea77a1dc3f07af853c9140f0`。四次候选运行之间有重叠；按完整用例名去重，共 21 个不同用例：20 过、1 败。**本次只确认后缀行为及负控，不是整个 Editor、跨 OS、UE 或产品验收。**
