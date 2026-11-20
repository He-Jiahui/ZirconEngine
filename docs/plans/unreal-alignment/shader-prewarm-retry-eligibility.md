# 着色器预热遇到坏材质后，原样重试会报告成功

## 1. 复现输入和读数

在 Windows x64 上，从公开 `main` 的 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37` 编译普通 `zircon_shader_prewarm`。在同一个 `Assets` 目录放入合法的 Surface shader 与 `materials/invalid.zmaterial`。材质的首行是未闭合的 `[options`。连续两次启动程序，传相同的 `--project-root`、`--asset-root` 和 `--cache-dir`，仅改变 `--report` 的输出文件名。

| 调用 | 退出码 | 诊断 | 报告请求 / 写入 / 失败 | WGSL 制品数 |
|---|---:|---|---|---:|
| 首次调用 | 1 | 材质 TOML 第 1 行第 9 列，`unclosed table` | 未生成 | 0 |
| 用同一份输入与缓存重试 | 0 | stderr 为空 | 0 / 0 / 0 | 0 |

两次使用的资产文件保持相同。两次调用后得到的库存索引 SHA-256 均为 `85327c23b7c6a8f7d8fb4eab27b94e193bfbd5c83a86a40480bba11f9b04dd4b`。普通程序的 SHA-256 为 `5b16ed830a2c50d569a9bf29929643445ea8a740f4753a0fa7ffdf98d5cea12b`，构建选择 `zircon_runtime`、`target-server,dynamic-api`、`dev` profile。

## 2. 对用户和构建调用者的影响

第一次运行已指出需要修改的文件。用户若立即重试，会从返回值和新的报告同时得到成功结果，原来的坏材质仍在项目中。成功预热后的零工作重试则是现有增量行为，应当继续支持。调用者需要能够从返回值分辨这两种情况。[命令行返回值](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/bin/zircon_shader_prewarm/main.rs#L12-L19)

## 3. ZE 写入索引和处理错误的顺序

程序先收集路径、元数据与源文本，并发布库存和索引。随后，它才把根目录内的材质转换为预热请求。材质 TOML 错误在这一步终止了首次调用。默认重试检查到库存仍然新鲜，于是跳过该根，把空请求列表交给执行器。执行器返回零请求的成功报告。该索引能证明源文件未变，当前格式缺少上一次预热是否完成的信息。[库存读取与更新](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/bin/zircon_shader_prewarm/manifest/asset_inventory.rs#L50-L93) · [投影、执行和最终返回值](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/bin/zircon_shader_prewarm/run.rs#L119-L207) · [坏材质的解析入口](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/bin/zircon_shader_prewarm/manifest/material_sources.rs#L120-L167)

## 4. Unreal 的对应实现与原测试

本地参考树版本为 `6.0.0 / UE5 / CL 0`。在 `Engine/Source/Editor/UnrealEd/Private/Cooker/CookRequestCluster.cpp:2417–2438`，`ProcessPlatformAttachments` 读出上次的 commit status。遇到 `Error` 时，它撤销“内容未变，可跳过”的决定，让下次 Cook 仍报告错误。`Engine/Source/Runtime/Engine/Private/ShaderCompiler/ShaderCompilerJobCache.cpp:1648–1653` 也只把成功且允许使用缓存的 job 交给 `AddJobOutput`。

UE 原有 `DerivedData::Cache::Hierarchy::MemoryCacheCleansUpOnCancel` 在存储处理前取消 `PutValue`，随后检查同 key 的内存条目没有可用数据，取消计数恰为一。代码位于 `Engine/Source/Developer/DerivedDataCache/Tests/DerivedDataCacheStoreHierarchyTest.cpp:400–410、547–572`。Gauntlet `IncrementalCookByTheBook` 提供成功路径的对照，它在首轮检查 Cook 包数大于零，重启后检查第二轮为零，并检查客户端启动。代码位于 `Engine/Source/Programs/AutomationTool/Gauntlet/Unreal/Automation/UE.IncrementalCookByTheBook.cs:35–63、116–120`。本次核对了 UE 测试源码，实际执行数据来自 ZE 的普通预热程序。

## 5. 既有测试和相邻问题

ZE 已有一次调用的坏材质测试，会检查 `ParseZMaterial` 和错误路径。本次回归增加第二个进程，让它读取首次失败留下的缓存。相邻的 #152 处理材质选项的 32 位预算。此处的复现使用合法的 32 位 shader。#230 处理同一次 Cook 与 Pack 之间的清单交接。当前修复位置是普通预热程序判断是否跳过资产根的逻辑。[一次材质解析的回归](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests/asset_scan_errors.rs#L67-L84) · [#152](https://github.com/He-Jiahui/ZirconEngine/pull/152) · [#230](https://github.com/He-Jiahui/ZirconEngine/pull/230)

## 6. Rust 修订与验收

给一次默认预热成功完成的库存代次记录成功状态。库存仍可在扫描后立即保存，跳过后续转换则需要同一代次的成功记录。记录应在整批转换、合并、制品写入和报告成功后提交。原缓存只有库存时，先做一次完整转换。旧程序使用的库存和新格式可以独立读取。

验收用同一程序、同一缓存连续读取坏材质，要求两次均返回 1 并指明该文件。随后改好输入，验证首次写出有效变体、第二次零请求。补充已成功的材质随后改坏、两个根中途失败后单独恢复、旧缓存升级和 `--quality-tier high` 的用例。

## 默认重试与旧成功缓存的验证结果

每个资产根保存一份 16 字节的成功代次。整批默认预热完成后才写入这份状态。下一次运行检查当前库存代次与成功记录，决定能否跳过已完成的工作。

| 已执行的普通程序 | 结果 |
|---|---|
| 同一份损坏材质连续预热两次 | 主干退出码依次为 1、0。修复后两次均为 1，均报告原材质的语法错误。 |
| 合法库存、已成功后引入错误、多个资产根、旧版失败库存 | 修复分支独立运行的五项用例全部通过。合法 32 位输入首次写出三个变体，默认热运行请求数为零。 |
| 对同一资产根进行八次有效编辑 | 九个不同库存代次共生成 27 个变体键。每次均只有一个代次状态文件和两份库存文件。最后一次未编辑的调用请求数为零。早期候选曾留下九份成功标记。 |

本分支的独立验证共启动 27 次 CLI。旧版成功库存的验证另外组合了 [#255 的磁盘写入修复](https://github.com/He-Jiahui/ZirconEngine/pull/255)。旧程序先写出三个变体，新程序跨过首批清单的创建秒后重新投影，三个变体的键和源身份一致。新版默认热运行得到零请求，旧版随后能重新导出同一资源。四次命令均返回 0。首批六个 v3 制品和两份旧库存文件均保留原来的字节。

此前保留的首个候选通过六项中的五项。旧库存重投影时只有两个变体成功，一个变体因清单冲突失败。这个观测可以用来区分资产根的重试资格和已有变体的写入兼容性。

[成功记录的读写](https://github.com/He-Jiahui/ZirconEngine/blob/dbd2ef1e6133b5371f5a69774acf44724284d43a/zircon_runtime/src/bin/manifest/asset_inventory/snapshot.rs) · [普通预热回归](https://github.com/He-Jiahui/ZirconEngine/blob/dbd2ef1e6133b5371f5a69774acf44724284d43a/tools/tests/test_zircon_shader_prewarm_retry.py)
