# Runtime 资源清单的来源目录

## 1. 问题与影响

Windows Runtime 的 shipping profile 规定 `asset_scope = "runtime"`。当运行时和编辑器资源目录各有一个同路径、同内容的图标时，最终 ZIP 中的 `staging_manifest.json` 会把来源记为编辑器目录。按清单追溯资源的维护者会找到另一份文件。归档中的图标字节和 SHA-256 均符合运行时输入。[产品配置](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/export/product_build_profiles.toml#L6-L24) · [清单的来源约定](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/tooling/zircon-size-optimized-builds.md#L42-L59)

## 2. 两个可直接比较的输入

本次对照用 `runtime-windows` profile 运行现有的配置解析、引擎资源复制、清单写入和确定性 ZIP 生成函数。两个资产根都放入 `icons/status-dot.svg`，编辑器另有一枚专属图标。第二组只改变编辑器图标的填充颜色。

| 编辑器同路径图标 | 实际复制来源 | ZIP 清单记录的 `source.path` |
|---|---|---|
| 与 Runtime 相同 | Runtime | `zircon_editor/assets/icons/status-dot.svg` |
| 颜色不同 | Runtime | `zircon_runtime/assets/icons/status-dot.svg` |

两份 ZIP 的运行时图标均为 111 字节，SHA-256 为 `dd3939cd4043c1e1cc5a46c20922e2cfa244c46f5ab4ff3bb8b3247fdf073c40`。两组均排除了编辑器专属图标。基线仓库的 643 个编辑器资源与 28 个运行时资源当前没有相同的相对路径，这两组输入用于验证工程增加合法资源后的行为。

## 3. ZE 的复制和清单生成路径

`stage_engine_assets` 从所选资源范围取根目录，Runtime 模式仅从 `zircon_runtime/assets` 复制。`_source_candidates` 为已复制文件查来源时则使用 `ENGINE_ASSET_ROOTS`，按 Editor、Runtime 顺序列出候选。`_find_source` 接受第一个 SHA-256 相符的文件。两棵树内容相同的时候，候选顺序决定了清单路径。[资源范围与复制](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/build/zircon_build_asset_staging.py#L15-L73) · [来源选择和 SHA 校验](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/build/zircon_build_staging_manifest.py#L142-L192)

## 4. Unreal 的来源映射和既有测试

本地 Unreal 6.0.0 源码中，`DeploymentContext.StageFile` 把实际 `InputFile` 和目标路径一起交给 `FilesToStage.Add`。`DumpManifest` 遍历已登记的映射，`CopyUsingStagingManifest` 依照该映射复制。对应位置为 `Engine/Source/Programs/AutomationTool/AutomationUtils/DeploymentContext.cs` 第 800–854 行，以及 `Engine/Source/Programs/AutomationTool/Scripts/CopyBuildToStagingDirectory.Automation.cs` 第 2993–3044 行。

已注册的低级测试 `ZenOplogUtils::Diff::DifferentPackageName` 固定包数据哈希、大小、文件名和 imports，更换包名后进行双向比较。测试要求两边缺失项均为 0、输出相同而元数据变化的项为 1、输出变化项为 0，并核对具体属性 `packagestoreentry/packagename`。位置为 `Engine/Source/Developer/ZenOplogUtils/Tests/ZenOplogUtilsTest.cpp` 第 487–511 行。它提供了分别验证载荷与元数据的判据。两棵资产根与 ZIP 的对照在 ZE 执行，UE 原测本轮核对了源码和断言。

## 5. 已有覆盖与相关 PR

现有 ZE 测试覆盖单一来源的文件哈希、缺少来源时的诊断和 shipping 排除项。此次检查了 243 个全状态 PR。同文件的 #161 处理显式 target triple 的编译产物和插件二进制候选，#230 处理项目 Cook 与 Pack 对显式资产清单角色的约定。本项对应引擎资源复制后的来源归属。[清单测试](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/tests/test_zircon_build_staging_manifest.py#L11-L111) · [资源范围测试](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/tests/test_zircon_build_asset_staging_owner_boundaries.py#L65-L159) · [#161](https://github.com/He-Jiahui/ZirconEngine/pull/161) · [#230](https://github.com/He-Jiahui/ZirconEngine/pull/230)

## 6. 等价回归与修复范围

回归从正式 profile 生成配置，走资源复制和 ZIP 入口，分别断言同字节和不同字节两组的来源、大小、SHA-256 和归档内清单。另以包含两个资产根的编辑器或开发打包方式核对其既有选择顺序。修复应让来源查找使用与复制相同的资源范围，并继续按实际输出校验内容哈希。[构建流程](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/build/zircon_build.py#L602-L614)

## 7. 修订前的正式回归

五个资产来源用例中四个通过。相同图标字节时，Runtime ZIP 的来源字段写入了 Editor 路径。六项已有清单及复制对照均通过。

测试以 Python 3.14.4 运行仓库中的 `unittest` 模块。两组分别生成 Runtime shipping ZIP，并打开归档核对载荷、清单内容与实际资源路径。其它输入包括双根合并、缺省范围、空范围和 Editor shipping 资源位置。六项对照覆盖已有的来源绑定、无来源诊断、生成与插件文件以及实际复制策略。

本次运行固定为 `2d11af2215768f99a87d84cf35ed0cd18b605544`。[可复跑用例](https://github.com/He-Jiahui/ZirconEngine/blob/2d11af2215768f99a87d84cf35ed0cd18b605544/tools/tests/test_zircon_build_asset_manifest_origin.py)

## 8. 修订后的 ZIP 清单

Runtime 归档的同路径图标现记录为选中的 Runtime 来源。相同与不同字节的两案均通过，另有三项资源范围用例和六项既有清单与复制测试通过，总计 11/11。

测试以 Python 3.14.4 运行仓库中的 `unittest` 模块。两组分别生成 Runtime shipping ZIP，并打开归档核对载荷、清单内容与实际资源路径。其它输入包括双根合并、缺省范围、空范围和 Editor shipping 资源位置。六项对照覆盖已有的来源绑定、无来源诊断、生成与插件文件以及实际复制策略。

本次运行固定为 `94c95094dfe15c6fa69341809f17831474945531`。[可复跑用例](https://github.com/He-Jiahui/ZirconEngine/blob/94c95094dfe15c6fa69341809f17831474945531/tools/tests/test_zircon_build_asset_manifest_origin.py)

另行执行了既有测试 `test_asset_staging_lives_in_asset_staging_owner`。它要求构建入口含有 `from .build.zircon_build_asset_staging import (`，在修前和修后均因这条字符串断言失败。相邻的 [#161](https://github.com/He-Jiahui/ZirconEngine/pull/161) 也编辑了清单文件，两份改动的三方文件合并成功。

## 9. 独立复核：载荷相同不能替代资源归属

| 源码或测试 | 等价点与差异 |
| --- | --- |
| UE `DeploymentContext.StageFile` → `FilesToStage` → 导出清单、按清单复制 | UE 保存显式的目标—来源映射；ZE 不照搬该映射，而是让复制和清单使用同一套产品 profile、资源根顺序与排除规则，在选中范围内用 SHA-256 校验已暂存文件。相同内容不能把排除的 Editor 根变成 Runtime 来源。 |
| UE `ZenOplogUtils::Diff::DifferentPackageName` | 将内容与元数据分别验证。ZE 的 [Runtime ZIP 回归](../../../tools/tests/test_zircon_build_asset_manifest_origin.py)独立断言归档字节、大小、SHA-256 和来源路径。选中范围外的文件即使字节相同，清单也报缺少来源，并不写出新的清单。 |

本模块是 ZE 的 Python 发布工具，不是 Rust 运行时；回归落在实际拥有复制、JSON 和确定性 ZIP 逻辑的 Python 单测。保留 Runtime/Editor profile 和可裁剪的资源范围；不改 UE 包格式、ZE 本体 Rust API 或插件 ABI。

Windows、Python 3.14.4。新增反例在远端基线 `e89349ca5` 的打包实现下实测失败（错误接受文件，写出清单）；本 PR 实现下通过。资产归属、清单与复制的定向行为用例 **12 通过、0 失败**。另有一个未改动的旧导入结构断言复核失败，已由独立 [#245](https://github.com/He-Jiahui/ZirconEngine/pull/245) 处理，未计入 12 项。未运行 UE 测试二进制或编译发布产品。
