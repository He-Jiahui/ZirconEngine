# 构建入口迁移后的模块归属检查

## 1. 七项检查影响构建回归结果

在公共基线 `e89349ca` 上用 Python 3.14.4 运行七个构建工具测试模块，16 项中有 9 项通过、7 项失败。七项均在检查旧包名时失败。以资产暂存为例，两项实际资源复制用例都通过，负责检查职责划分的一项却在读构建入口时失败。[暂存模块的三项用例](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/tests/test_zircon_build_asset_staging_owner_boundaries.py#L13-L159)

## 2. 原始输入和对照

构建工具提供两种入口。程序可作为 `tools.build.zircon_build` 导入，也可直接执行。以暂存为例，包入口从同包模块导入 `stage_engine_assets` 和 `copy_resource_dirs`，脚本入口从 `tools.build` 的绝对路径导入这两个对象。测试对两种入口都要求源码包含 `from .build.zircon_build_asset_staging import (`。在独立的基线副本中，只把七项测试里的旧包前缀换成实际包前缀，同一组 16 项全部通过。[包入口](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/build/zircon_build.py#L19-L34) · [脚本入口](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/build/zircon_build.py#L97-L113)

## 3. 原因和涉及的职责

目录重构将构建入口和配套模块移入 `tools/build`。测试更新了入口文件的位置，却保留了对旧导入语句的预期。受影响的检查涵盖资产暂存、ZUI 校验、Hub 构建、插件资产根、插件目录、插件清单和着色器描述符。插件目录检查还分别匹配旧的相对与绝对包名。[目录迁移提交](https://github.com/He-Jiahui/ZirconEngine/commit/b082ecc28d8dbe5d3755cc9095240ce51244cdd9) · [ZUI 归属检查](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/tests/test_zircon_build_zui_asset_owner_boundaries.py#L22-L44) · [插件目录归属检查](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/tests/test_zircon_build_plugin_catalog_owner_boundaries.py#L28-L53)

## 4. Unreal 中的相邻职责与原测

UAT 的 `BuildCookRun` 编排 Build、Cook、Stage、Package，Stage 的部署上下文负责生成和应用清单。本机源码中对应 `Engine/Source/Programs/AutomationTool/Scripts/BuildCookRun.Automation.cs` 第 251–267 行和 `Engine/Source/Programs/AutomationTool/Scripts/CopyBuildToStagingDirectory.Automation.cs` 第 7150–7168 行。

已注册的 MSTest `DotnetProjectInfoTests.TestParseOwnProject` 从程序集位置找到实际工程，用生产 `DotnetProjectInfo.Read` 读取它，断言 `IsPackable` 存在且值为 `false`。它为 ZE 的回归提供了验证真实解析结果的参照。UE 的原测与断言见 `Engine/Source/Programs/Shared/EpicGames.Build.Tests/DotnetProjectInfoTests.cs` 第 17–35、53–74 行。

## 5. 历史归属

已核对 244 个全状态 PR 的标题、正文和改动文件。上述目录迁移早于 #1 的基线。#244 修复了正式归档中同名资产的来源选择，并在修前和修后各自记录了暂存归属测试的同一失败。#120、#136 和 #161 涉及过构建工具的其他工作。这七个测试文件由本项统一处理。[#244 的归档修复](https://github.com/He-Jiahui/ZirconEngine/pull/244) · [#161 的构建目标选择](https://github.com/He-Jiahui/ZirconEngine/pull/161)

## 6. 修复与验收

在两种入口分别加载真实定义，检查公开函数或类型绑定到各自的实现模块。资产暂存用例检查构建入口到暂存模块的绑定，ZUI 用例检查暂存模块到 ZUI 校验函数的绑定。验收复跑原来的 16 项。另用独立副本把单一路径绑定到另一个仍可导入的函数，要求对应检查报告失败。

## 7. 两个入口的实测结果

现有七个模块的 16 项测试在 Python 3.14.4 下由 9 项通过提升到 16 项通过。仅错绑脚本入口的资源复制函数时，暂存模块用例准确报告该符号和入口，其他 15 项通过。

测试逐一加载正式 Python 包与脚本入口，比较它们导出的函数或类型和相应实现对象。ZUI 测试核对暂存模块使用的校验函数。成功运行的代码提交为 `864bf4890699b57672036684396864af75fd4484`。[双入口检查](https://github.com/He-Jiahui/ZirconEngine/blob/864bf4890699b57672036684396864af75fd4484/tools/tests/zircon_build_owner_assertions.py)

另在 #244 的归档修复前后分别复跑原始暂存归属测试及本次修订。两版的原测试均在旧导入语句处失败，本次检查在两版上均通过。
