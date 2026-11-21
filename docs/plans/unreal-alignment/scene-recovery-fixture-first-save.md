# 场景崩溃恢复夹具未到达故障注入

## 已观察到的失败

`crash_publication_recovers_scene_and_opaque_workspace_before_reopen` 在 Windows x64/MSVC 的 `zircon_runtime` `native-host` 单测中返回 `MissingProjectAssetUri`，退出码为 101。失败发生在[夹具首次写入场景前](../../../zircon_runtime/src/asset/project/manager/tests/editor_document_transaction.rs#L182-L205)：它对尚未创建的 `res://scenes/main.scene.toml` 调用了要求源已存在的 `source_path_for_uri`，所以既没有写入初始场景，也没有进入事务的崩溃注入。

这项失败来自复核 #256 时记录的 `c229eb58` 源码；对照远端 `main` 的 `e89349ca5`，报错所在夹具和源路径解析器均未改动。该失败不能证明恢复算法有缺陷；没有把失败算作通过，也没有声称已单独运行原样主干二进制。#256 的 Prefab 路径修正发生在尚未运行的事务步骤中，不能修复该夹具的首次写入。搬迁恢复用例的运行时资源表断言由 [#264](https://github.com/He-Jiahui/ZirconEngine/pull/264) 单独处理。

## UE 对照与 ZE 边界

UE `UEditorLoadingAndSavingUtils::SaveMap`（`Engine/Source/Editor/UnrealEd/Private/FileHelpers.cpp:5873–5886`）在保存前把已挂载的逻辑包名转换成目标文件名；`FPackageName::TryConvertLongPackageNameToFilename`（`Engine/Source/Runtime/CoreUObject/Private/Misc/PackageName.cpp:1407–1434`）不要求目标文件已存在。原测 `System.Core.Misc.PackageNames` 在同文件第 4211–4305 行的 `TryConvertToMountedPath` 子项断言：已挂载的逻辑包名得到预期本地路径，未挂载的路径被拒绝。迁移的是创建目标解析与存在性查询分离的行为，不是 UE 的包格式或挂载实现。

ZE 使用 `res://`、项目资产根、TOML 场景以及场景和 opaque workspace 的日志事务。[项目路径入口](../../../zircon_runtime/src/asset/project/manager/source_path_for_uri.rs#L106-L165)已分别提供创建路径和查找既有源的操作。夹具首次写入使用创建入口。重开直接经[源文档入口](../../../zircon_runtime/src/scene/world/project_io/source_document.rs#L77-L88)读取场景字节和版本，不把未扫描导入的文件当成制品；[制品读取接口](../../../zircon_runtime/src/scene/world/project_io/scene_asset.rs#L80-L95)及事务生产代码保持不变。

## 回归判据

1. 仅创建目录、尚无场景源时，创建路径可解析到已登记根内的预期文件；既有源查找仍明确拒绝。
2. 写入初始场景后，既有源查找解析到同一文件；保留既有故障注入、日志恢复、原场景/工作区字节恢复及第二次提交后独立重开的断言。
3. 在 Windows x64/MSVC 上精确运行 `zircon_runtime` 库用例（关闭默认特性，仅 `native-host`，test profile）。

## 独立实测：首次写入通过，末尾重开仍波及制品接口

在 Windows x64/MSVC、Rust 1.94.1，用 `cargo test -p zircon_runtime --lib --no-default-features --features native-host --locked --offline --jobs 1 asset::project::manager::editor_document_transaction::tests::crash_publication_recovers_scene_and_opaque_workspace_before_reopen -- --exact --nocapture --test-threads 1` 运行首轮夹具修正（该 Rust 文件 Git blob `5473f55c`）。验证已走到实际的场景目标替换后中断：日志非空、场景字节改变、工作区未被替换。独立打开后两个目标的旧字节恢复；第二次原子提交的场景和工作区新字节也比对成功。

**首轮完整用例失败（0 通过、1 失败，退出 101，7,774 项过滤）**。最后一个旧断言使用 `load_scene_from_uri` 读取未导入的场景，报告 `missing asset metadata for source uri res://scenes/main.scene.toml`。

## 独立复跑：首次写入、回滚与第二次重开全通过

同一命令在 Rust blob `513a76b6` 上得到 **1 通过、0 失败、0 忽略，退出 0，7,774 项过滤**。首次写入前，已挂载 URI 可以解析目标而不能查到既有源；写入后两个入口指向同一文件。故障采样确认日志非空、场景已替换而工作区仍旧；独立打开回滚到原场景节点和两份旧字节。第二次提交达到持久状态，另一次打开按源文档重建 World，节点和版本匹配新源，工作区也读回新字节。

验证仅覆盖 `zircon_runtime` 的 Scene/Project 文档恢复用例。没有改生产事务、ECS、矩阵变换或导入流程；没有运行 UE 自动化、编辑器窗口验收或全套资产用例。必需的 GitHub Actions `editor-compile-gate` 当前未上报，本地单测不替代合并门禁。
