# 项目资产搬迁恢复夹具混用了两层注册表

## 已观察到的失败

在 #256 的相邻复核中，Windows x64/MSVC 上精确运行 `interrupted_project_source_relocation_recovers_the_original_generation` 返回退出码 101。故障注入后确实留下了移动到新位置的源文件。独立打开项目后的原源文件、侧车和磁盘资产索引字节比较均已通过，新位置也已清除；随后[原测试](../../../zircon_runtime/src/asset/tests/project/manager/relocation.rs#L134-L149)在尚未扫描导入时断言 `reopened.registry().get_by_locator(&source).is_some()` 失败。

日志使用 `c229eb58`；报错夹具、项目打开和扫描导入源码与远端 `main` 的 `e89349ca5` 相同。没有把旧日志说成单独运行原样主干，也未把失败归咎于 Prefab 保存修复。#262 的场景首次写入问题不在本 PR 范围。

## UE 对照与 ZE 生命周期

UE 的 `FAssetRegistryState::Load`（`Engine/Source/Runtime/AssetRegistry/Private/AssetRegistryState.cpp:2723–2778`）加载并索引 `FAssetData` 元数据；元数据有效并不等于包里的 `UObject` 已加载。`FAssetData::FastGetAsset`（`Engine/Source/Runtime/CoreUObject/Public/AssetRegistry/AssetData.h:505–566`）找不到已加载包时，只有显式 `bLoad` 才发起加载。UE 原测 `System.AssetRegistry.AssetRegistryState.EnumerateAssetsPerformance`（`Engine/Source/Runtime/AssetRegistry/Private/AssetRegistryStateTest.cpp:14–60`）只建设 `FAssetData` 状态，用磁盘资产模式筛选并断言一个匹配条目；没有要求对应对象已加载。此处迁移该可观察边界，不迁移其百万资产规模，也不把 UE 原测当作 ZE 事务回滚证据。

ZE 使用 Rust 的 `AssetRegistryIndex` 保存 UUID 与源路径，用 `ResourceRegistry` 发布已导入的资源。`ProjectManager::open` [先恢复事务并加载资产索引，运行时资源表仍为空](../../../zircon_runtime/src/asset/project/manager/open.rs#L31-L63)；需要注册夹具的 importer，再走[扫描导入](../../../zircon_runtime/src/asset/project/manager/scan_and_import.rs#L161-L170)才发布运行时记录。两层索引、侧车和文件事务保持 ZE 实现，不用 UE 的 `UObject` 加载替代它们。

## 待验证

1. 保留搬迁中断后的文件、侧车与磁盘索引字节核对：旧地址恢复，新地址退出。
2. 独立打开后，在恢复的资产索引中读取同一 UUID 和旧地址；运行时不凭空产生资源。重新注册测试 importer 并扫描后，在运行时表验证原地址和稳定资源 ID，新地址仍不可见。
3. 在 Windows x64/MSVC 精确运行该 `zircon_runtime` 库用例（test profile，关闭默认特性，仅 `native-host`）。本首发 Draft 仅登记问题，尚未修改或验证夹具。
