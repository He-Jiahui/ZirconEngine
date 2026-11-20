# 移动 Prefab 源后，当前场景应能保存已有编辑

## 1. 用户操作与影响

项目内有一个 Squad 预制体，场景中的多个实体引用它。打开场景，修改一个普通实体的位置，在 Asset Browser 中把 Squad 拖到同项目的另一文件夹，等移动完成后执行 File → Save。搬迁保留 Squad 的 UUID，场景保存仍用旧地址查询注册表。按公开主干的代码，这条路径会在写入场景前返回 `MissingPath`，同一场景中的位置编辑也无法写盘。[Asset Browser 的移动命令](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/host/assets/manager/assets/project_relocation.rs#L24-L83) · [File Save 对场景的派发](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/host/editor_event_execution/menu_action.rs#L75-L175)

## 2. 引用怎样变旧

`relocate_project_source` 在文件提交后装入新项目代，发布包含旧、新地址的 `Renamed`。新目录保留每个 UUID，删除旧路径条目，登记新地址，并更新依赖边。已经打开的 World 按实体保留 Prefab 引用、局部变换和覆盖值。编辑器会保留有未保存修改的 World。下一次保存从这份 World 数据读取移动前的引用。[搬迁提交](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/pipeline/manager/assets/relocation.rs#L54-L88) · [注册表更新](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/registry/relocation.rs#L41-L86) · [World 的实例记录](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/world/project_io/prefab.rs#L6-L24) · [脏场景的切换条件](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/workbench/startup/editor_state_project.rs#L37-L52)

## 3. 保存和重开为何表现不同

场景写入器对每个正式引用调用 `persist_runtime_reference`。UUID 查找能找到搬迁后的条目，随后按旧 locator 查找会返回 `MissingPath`。这个步骤位于 `PreparedFileWrite` 和事务提交之前。独立重开时，读取器先按稳定 UUID 找条目，使用条目的当前 locator 构造引用，并将过时的路径提示标为可修复。用户需要能够在第一次普通保存时完成这次地址更新。[保存事务](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/editor_document_transaction.rs#L255-L285) · [保存时的双重查找](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/persisted_reference.rs#L28-L75) · [重开时解析 UUID](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/reference_resolver.rs#L151-L220)

## 4. Unreal 的对应实现与原测试

本地 UE 参考树为 `6.0.0 / UE5 / CL 0`。`Engine/Source/Developer/AssetTools/Private/AssetRenameManager.cpp:1576–1603` 按明确的旧、新资产地址登记重定向，再遍历选定包中的对象，更新软引用。`Engine/Source/Runtime/CoreUObject/Private/UObject/SoftObjectPath.cpp:320–337,374–381` 在适用的编辑器保存档案中执行 `PreSavePath`，把已登记的资产地址写入软引用。

`Engine/Source/Developer/AssetTools/Private/AssetHeaderPatcher.cpp:4975–5003` 中的 `DoPatch(FSoftObjectPath)` 测试检查顶层包和对象移到目标路径，也检查子对象 `Some.SrcPackage.Subobject` 中同名的 `SrcPackage` 原样保留。`Engine/Source/Runtime/Engine/Private/Tests/WorldPartition/WorldPartitionEditorSoftObjectPathTests.cpp:153–180` 分别断言两个不同顶层实例中的同名 Actor 和组件应解析到各自的运行时路径。ZE 回归沿用这两项可观察行为，检查类型化引用的更新和每个实例数据的归属。UE 对照来自测试源码。

## 5. 已有覆盖与本题范围

ZE 的资产搬迁测试检查同一 ID 的新地址、旧地址失效及 `Renamed` 事件。已有场景测试检查实例覆盖值的往返。两者相接的边界是本题的复现位置。#133 保留 World 和缓存中的实例数据。#210 修复顶层空覆盖值的序列化。本题选用非空字符串，并移动同 UUID 的源文件。已有 Runtime87 计划登记了脏文档搬迁风险，此处把它落实为可单独验收的保存操作。[已有搬迁回归](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/pipeline/manager/assets/tests/relocation.rs#L12-L66) · [实例数据 #133](https://github.com/He-Jiahui/ZirconEngine/pull/133) · [空覆盖值 #210](https://github.com/He-Jiahui/ZirconEngine/pull/210) · [Runtime87 的引用迁移规划](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/optimize/zircon_runtime/87-runtime-asset-reference-identity-locator-guid-subasset-redirector-rename-move-resolution-repair-migration.md#L217-L228)

## 6. 等价回归与修复条件

通过正式 importer 建立 Leaf、嵌套引用 Leaf 的 Squad、引用 Squad 的三个场景实例。先保存普通实体的 X 坐标 `1.25` 并回读。再在同一 World 改为 `4.25`，移动 Squad，检查同 UUID、新目录代及 `Renamed`。在该 World 上调用正常项目保存。基线要同时观察具体的旧路径错误、原磁盘字节和新建 ProjectManager 的独立重开。首发已核对两端源码、原测试断言和 TOML 输入结构，运行时测量待补入本 PR。

修复后的保存应写出三个实例的新路径提示。独立重开后，X 应为 `4.25`，实例各自的变换和覆盖值都能读回。其中一个普通字符串刻意等于旧 URI，仍应保持原值。建议在构造待写场景引用时，用可确认的同项目搬迁记录或唯一稳定 UUID 解析根 Prefab 的新地址，再执行现有物理源与版本检查。UUID 缺失、旧地址被其他资源占用以及物理源不唯一时应保留明确错误。
