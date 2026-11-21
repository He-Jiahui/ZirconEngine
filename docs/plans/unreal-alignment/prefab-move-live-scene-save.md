# 移动 Prefab 源后，当前场景应能保存已有编辑

## 1. 用户操作与影响

项目内有一个 Squad 预制体，场景中的多个实体引用它。打开场景，修改一个普通实体的位置，在 Asset Browser 中把 Squad 拖到同项目的另一文件夹，等移动完成后执行 File → Save。搬迁保留 Squad 的 UUID，场景保存仍用旧地址查询注册表。按公开主干的代码，这条路径会在写入场景前返回 `MissingPath`，同一场景中的位置编辑也无法写盘。[Asset Browser 的移动命令](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/host/assets/manager/assets/project_relocation.rs#L24-L83) · [File Save 对场景的派发](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/host/editor_event_execution/menu_action.rs#L75-L175)

## 2. 引用怎样变旧

`relocate_project_source` 在文件提交后装入新项目代，发布包含旧、新地址的 `Renamed`。新目录保留每个 UUID，删除旧路径条目，登记新地址，并更新依赖边。已经打开的 World 按实体保留 Prefab 引用、局部变换和覆盖值。编辑器会保留有未保存修改的 World。下一次保存从这份 World 数据读取移动前的引用。[搬迁提交](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/pipeline/manager/assets/relocation.rs#L54-L88) · [注册表更新](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/registry/relocation.rs#L41-L86) · [World 的实例记录](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/world/project_io/prefab.rs#L6-L24) · [脏场景的切换条件](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/workbench/startup/editor_state_project.rs#L37-L52)

## 3. 保存和重开为何表现不同

场景写入器对每个正式引用调用 `persist_runtime_reference`。UUID 查找能找到搬迁后的条目，随后按旧 locator 查找会返回 `MissingPath`。这个步骤位于 `PreparedFileWrite` 和事务提交之前。独立重开时，读取器先按稳定 UUID 找条目，使用条目的当前 locator 构造引用，并将过时的路径提示标为可修复。用户需要能够在第一次普通保存时完成这次地址更新。[保存事务](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/editor_document_transaction.rs#L255-L285) · [保存时的双重查找](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/persisted_reference.rs#L28-L75) · [重开时解析 UUID](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/reference_resolver.rs#L151-L220)

## 4. Unreal 的对应实现与原测试

本地 UE 参考树为 `6.0.0 / UE5 / CL 0`。`Engine/Source/Developer/AssetTools/Private/AssetRenameManager.cpp:1576–1603` 按明确的旧、新资产地址登记重定向，再遍历选定包中的对象，更新软引用。`Engine/Source/Runtime/CoreUObject/Private/UObject/SoftObjectPath.cpp:320–337,374–381` 在适用的编辑器保存档案中执行 `PreSavePath`，把已登记的资产地址写入软引用。

`Engine/Source/Developer/AssetTools/Private/AssetHeaderPatcher.cpp:4975–5003` 的 `DoPatch(FSoftObjectPath)` 验证只更新类型化引用的顶层地址，子对象中同名文本不被替换；对应 ZE 保存后只有 Prefab 引用的路径提示改变，等于旧 URI 的覆盖字符串保持原值。`Engine/Source/Runtime/Engine/Private/Tests/WorldPartition/WorldPartitionEditorSoftObjectPathTests.cpp:153–180` 验证不同顶层实例中的同名 Actor 和组件各自解析；对应 ZE 的三份独立场景实例和两份嵌套实例各自保留局部变换与覆盖值。这里迁移的是可观察语义，不是 UE 对象或包结构。

**有意保留的差异：** UE 在适用的保存档案中对 `FSoftObjectPath` 就地应用已登记的重定向。ZE 以 UUID 为身份，以 locator/路径提示为位置；仅当旧位置确实空出且现行项目代的唯一 Prefab 源及侧车匹配时，在待发布快照中更新类型化引用，不改活动 ECS World、不做全局字符串替换。同一场景共享的旧引用只需核对一次物理搬迁。ZE 的 TOML 事务、矩阵式层级变换与运行时模块边界不变。这里未运行 UE 自动化。

## 5. 已有覆盖与本题范围

ZE 的资产搬迁测试检查同一 ID 的新地址、旧地址失效及 `Renamed` 事件。已有场景测试检查实例覆盖值的往返。两者相接的边界是本题的复现位置。#133 保留 World 和缓存中的实例数据。#210 修复顶层空覆盖值的序列化。本题选用非空字符串，并移动同 UUID 的源文件。已有 Runtime87 计划登记了脏文档搬迁风险，此处把它落实为可单独验收的保存操作。[已有搬迁回归](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/pipeline/manager/assets/tests/relocation.rs#L12-L66) · [实例数据 #133](https://github.com/He-Jiahui/ZirconEngine/pull/133) · [空覆盖值 #210](https://github.com/He-Jiahui/ZirconEngine/pull/210) · [Runtime87 的引用迁移规划](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/optimize/zircon_runtime/87-runtime-asset-reference-identity-locator-guid-subasset-redirector-rename-move-resolution-repair-migration.md#L217-L228)

## 6. 等价回归与修复条件

通过正式 importer 建立 Leaf、嵌套引用 Leaf 的 Squad、引用 Squad 的三个场景实例。先保存普通实体的 X 坐标 `1.25` 并回读。再在同一 World 改为 `4.25`，移动 Squad，检查同 UUID、新目录代及 `Renamed`。在该 World 上调用正常项目保存。基线要同时观察具体的旧路径错误、原磁盘字节和新建 ProjectManager 的独立重开。首发已核对两端源码、原测试断言和 TOML 输入结构，运行时测量待补入本 PR。

修复后的保存应写出三个实例的新路径提示。独立重开后，X 应为 `4.25`，实例各自的变换和覆盖值都能读回。其中一个普通字符串刻意等于旧 URI，仍应保持原值。建议在构造待写场景引用时，用可确认的同项目搬迁记录或唯一稳定 UUID 解析根 Prefab 的新地址，再执行现有物理源与版本检查。UUID 缺失、旧地址被其他资源占用以及物理源不唯一时应保留明确错误。

## 修前原生回归：搬迁后活场景保存仍查询旧地址

同一个已打开的真实项目中，未搬迁对照先写盘并重开成功（探针 x=1.25）。在同一活动场景把探针编辑为 x=4.25，再走正常搬迁入口，把 Squad Prefab 从 `res://prefabs/squad.prefab.toml` 移到 `res://archive/squad.prefab.toml`。旧源和侧车退出，目录保留同一 UUID 并登记新地址，但活动 World 仍持旧地址。此时普通保存返回针对旧地址的 `MissingPath`。

失败后场景文件与未搬迁的对照逐字节相同。关闭原项目代后独立重开：三个场景实例仍引用同一 Prefab，顶层覆盖行数为 1／1／0，两个嵌套实例各保留一行覆盖，未移动的 Leaf 依赖保持不变。解析出的实际地址为新位置，磁盘探针仍是 x=1.25，证明失败的保存没有写入活动场景的 x=4.25。已回读场景、Prefab、侧车和目录字节，不把读路径成功说成写路径已修复。

Windows x64/MSVC，`zircon_runtime` 库测试目标（关闭默认特性，仅请求 `native-host`，test profile）。单个精确用例在有界 Job 中为 **0 通过、1 失败（退出 101），7,775 项被过滤**。PE 原位使用、执行前后核对指纹，未留独立副本。未运行修后版本、普通桌面窗口或 UE 自动化。本次只增补同一条回归，不包含生产代码。

[原生回归源码](https://github.com/He-Jiahui/ZirconEngine/blob/f84bf72a92c1716859f99401521dfd5cbbe34654/zircon_runtime/src/asset/pipeline/manager/assets/tests/relocation.rs#L416)

## 修后原生回归：搬迁后保存并独立重开

Scene writer 仅在本次保存的类型化 Prefab 实例上更新路径提示。它用 UUID 核对当前项目目录中的 Prefab、唯一物理源和侧车，确认旧源和侧车均消失后更新 locator，再经过原有的严格引用写入校验。 [Scene 保存](https://github.com/He-Jiahui/ZirconEngine/blob/52f49b7daa15cc50e9a3ed33b32128d150b52346/zircon_runtime/src/asset/project/manager/editor_document_transaction.rs#L272) · [源身份核对](https://github.com/He-Jiahui/ZirconEngine/blob/52f49b7daa15cc50e9a3ed33b32128d150b52346/zircon_runtime/src/asset/project/manager/persisted_reference.rs#L110)

同一条 Windows x64/MSVC 回归在修前收到旧地址的 `MissingPath`，磁盘保留了无搬迁对照的 x=1.25。修后使用同一 `zircon_runtime` libtest 目标、`native-host` 和 test profile。普通保存后，独立新开项目读回 x=4.25。三个实例的 `path_hint` 都是 `assets/archive/squad.prefab.toml`，GUID 与搬迁前一致。磁盘场景的覆盖行数仍为 1／1／0。嵌套的两个 Leaf 实例各保留一行覆盖与原引用。覆盖值中的字面量 `res://prefabs/squad.prefab.toml` 原样保留。 [原生回归源码](https://github.com/He-Jiahui/ZirconEngine/blob/fc72fd7d5f6d56f55ad60630e538701f265c676d/zircon_runtime/src/asset/pipeline/manager/assets/tests/relocation.rs#L416)

精确选择一个用例。修前 0 通过、1 失败（退出 101），修后 1 通过、0 失败、0 忽略（退出 0）。两次均过滤 7,775 项。修后使用与编译产物字节一致的保留 PE，并回读原始测试输出、Job 和物理文件。验证范围是 Runtime 项目搬迁、保存及重开。

## 相邻测试的验收边界

先前运行记录基于 `c229eb58`，和本轮开始时的 `78d54420` 仅有本文档差异。八个定向用例中六个通过，另有两项失败，**均不计为通过**：

- [源搬迁中断恢复](../../../zircon_runtime/src/asset/tests/project/manager/relocation.rs#L134-L149)：恢复后的源、侧车和磁盘索引字节检查已通过，随后在只调用 `ProjectManager::open` 后期待运行时资源表已有该 locator 的断言失败。打开路径只加载磁盘索引，不扫描导入运行时对象。
- [场景与工作区中断恢复](../../../zircon_runtime/src/asset/project/manager/tests/editor_document_transaction.rs#L182-L205)：夹具在首次写入场景前使用只接受已存在源的 `source_path_for_uri`，先返回 `MissingProjectAssetUri`，未进入本 PR 改动的保存事务。

这两项保持原断言与原失败记录，单独复核和修复。首次写入夹具已另开 [Draft #262](https://github.com/He-Jiahui/ZirconEngine/pull/262)。本 PR 不声称已验证完整崩溃恢复或编辑器产品验收。

## 独立复核：快照边界和旧路径占用

[扩展后的同一实际保存用例](../../../zircon_runtime/src/asset/pipeline/manager/assets/tests/relocation.rs)还验证：成功保存不改活动 World 中的旧 locator。关闭 watcher、用独立项目管理器重开后，在旧位置创建尚未入库的另一源文件，再试图保存活动 World 中 `x=7.0` 的新编辑；保存返回旧 locator 的 `MissingPath`，磁盘场景逐字节不变，新编辑仍在内存。正常搬迁的对照独立重开仍读回 `x=4.25`、同一 UUID 和新位置，实例覆盖及其字面量保持原值。

Windows x64/MSVC，Rust 1.94.1，`zircon_runtime` 库测试、test profile，只启用 `native-host`（默认特性关闭）。实际命令为 `cargo test -p zircon_runtime --lib --no-default-features --features native-host --locked --offline --jobs 1 asset::pipeline::manager::project_asset_manager::relocation::tests::relocating_a_prefab_keeps_the_open_scene_savable -- --exact --nocapture --test-threads 1`。一次精确执行：1 通过、0 失败、0 忽略，7,775 项过滤。未运行 UE 自动化、完整资产套件或编辑器 GUI。主干的 GitHub Actions 必需检查 `editor-compile-gate` 未上报，本地结果不替代合并门禁。
