# 场景崩溃恢复夹具未到达故障注入

## 已观察到的失败

`crash_publication_recovers_scene_and_opaque_workspace_before_reopen` 在 Windows x64/MSVC 的 `zircon_runtime` `native-host` 单测中返回 `MissingProjectAssetUri`，退出码为 101。失败发生在[夹具首次写入场景前](../../../zircon_runtime/src/asset/project/manager/tests/editor_document_transaction.rs#L182-L205)：它对尚未创建的 `res://scenes/main.scene.toml` 调用了要求源已存在的 `source_path_for_uri`，所以既没有写入初始场景，也没有进入事务的崩溃注入。

这项失败来自复核 #256 时记录的 `c229eb58` 源码；对照远端 `main` 的 `e89349ca5`，报错所在夹具和源路径解析器均未改动。该失败不能证明恢复算法有缺陷；没有把失败算作通过，也没有声称已单独运行原样主干二进制。#256 的 Prefab 路径修正发生在尚未运行的事务步骤中，不能修复该夹具的首次写入。另一个搬迁恢复用例的运行时资源表断言不在本 PR 范围。

## UE 对照与 ZE 边界

UE `UEditorLoadingAndSavingUtils::SaveMap`（`Engine/Source/Editor/UnrealEd/Private/FileHelpers.cpp:5873–5886`）在保存前把已挂载的逻辑包名转换成目标文件名；`FPackageName::TryConvertLongPackageNameToFilename`（`Engine/Source/Runtime/CoreUObject/Private/Misc/PackageName.cpp:1407–1434`）不要求目标文件已存在。原测 `System.Core.Misc.PackageNames` 在同文件第 4211–4305 行的 `TryConvertToMountedPath` 子项断言：已挂载的逻辑包名得到预期本地路径，未挂载的路径被拒绝。迁移的是创建目标解析与存在性查询分离的行为，不是 UE 的包格式或挂载实现。

ZE 使用 `res://`、项目资产根、TOML 场景以及场景和 opaque workspace 的日志事务。[项目路径入口](../../../zircon_runtime/src/asset/project/manager/source_path_for_uri.rs#L106-L165)已分别提供创建路径和查找既有源的操作。夹具应走创建入口，而不是放宽既有读取或修改事务恢复。

## 待验证

1. 仅创建目录、尚无场景源时，创建路径可解析到已登记根内的预期文件；既有源查找仍明确拒绝。
2. 写入初始场景后，既有源查找解析到同一文件；保留既有故障注入、日志恢复、原场景/工作区字节恢复及第二次提交后独立重开的断言。
3. 在 Windows x64/MSVC 上精确运行 `zircon_runtime` 库用例（关闭默认特性，仅 `native-host`，test profile）。此首发 Draft 仅登记问题；不包含修复，也不记录尚未执行的验收结果。
