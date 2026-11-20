# macOS NativeDynamic 把 dSYM 内的同名 DWARF 文件列为运行库

## 1. 可观察的问题

向 ZE 的 NativeDynamic 导出阶段提供一份 macOS 动态库和配套 dSYM，导出成功，实际库只有一个，交付报告却列出两个。多出的条目是 `libanimation.dylib.dSYM/Contents/Resources/DWARF/libanimation.dylib`。PlatformBundle 读取交付报告时得到同样的列表，可选签名步骤选择输入文件时也包含这份调试信息。

## 2. 用户合同与影响

ZE 支持按目标平台复制预建插件。macOS 的 `.dylib` 属于运行库，`.dSYM` 是随包递归复制的调试符号。符号文件需要出现在完整文件清单中，运行库清单则用于向下游交付可加载的库。按叶文件后缀计算，两份文件在角色清单中变成了同一种产物。工具还会把调试文件作为外部处理命令的输入候选。[平台暂存合同](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/tooling/zircon-export-tool.md#L1960-L1974) · [签名输入合同](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/tooling/zircon-export-tool.md#L2251-L2278)

## 3. Unreal 的第三方规则与既有测试

本地 UE 参考树的 `Engine/Source/Developer/Apple/MetalShaderFormat/MetalShaderFormat.Build.cs:29–46` 为 Mac 显式登记 `libdxcompiler.dylib`，为 Windows 按架构选择对应目录的 `dxcompiler.dll`。`Engine/Build/Commit.gitdeps.xml:39248–39266` 同时列出了 ShaderConductor 和 DXC 的 Mac 动态库及 dSYM 内的同名 DWARF 文件。`Engine/Source/Programs/AutomationTool/AutomationUtils/DeploymentContext.cs:1013–1055` 分别处理 receipt 中的动态库和符号产物。

`Engine/Source/Programs/Shared/EpicGames.Core.Tests/FileSetTests.cs:20–25,54–65,85–122` 中的 `FileSetContainsAllFiles`、`FileSetExceptExcludesFileTypeWildcard` 和 `FileSetFilterContainsTripleDotPrefixAndSuffix` 使用含 PDB 和子目录的夹具，分别验证完整文件集和筛选结果。它们提供本次回归的分层依据。同名 DWARF 判例来自上述真实第三方物料布局。本轮验证使用本机 UE 文件和 ZE 导出工具。

## 4. ZE 定位与已有覆盖

预建包复制器已经识别 `.dSYM` 目录并递归复制其中的内容。包报告定稿后重新遍历文件，加载清单只比较最后一个后缀。可选签名/公证枚举和下游清单对账也按叶名判断，因而三个位置接受同一条错误记录。[复制目录](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/export/native_dynamic_materialize.py#L241-L279) · [报告定稿](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/export/native_dynamic_stage_payload_finalize.py#L83-L114) · [加载清单](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/export/native_dynamic_payload_file_manifest.py#L179-L207) · [签名枚举](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/export/native_signing.py#L279-L301) · [交付对账](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/export/native_dynamic_payload_directory.py#L145-L215)

现有 macOS 用例验证了 dSYM 内容的复制。夹具中的 DWARF 叶名没有 `.dylib` 后缀。Windows 用例覆盖 DLL/PDB 和目标平台后缀。逐项核对开放、已合并及已关闭 PR 后，最邻近的 #161 处理编译产物与符号的 target 来源，#244 处理引擎资产 ZIP 的来源。本案归属 NativeDynamic 预建包中产物角色的投影。[macOS 原用例](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/export/tests/test_native_dynamic.py#L120-L149) · [DWARF 夹具](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/export/tests/native_dynamic_test_support.py#L117-L146) · [Windows 原用例](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/export/tests/test_native_dynamic_stage.py#L176-L257) · [#161](https://github.com/He-Jiahui/ZirconEngine/pull/161) · [#244](https://github.com/He-Jiahui/ZirconEngine/pull/244)

## 5. 复现与对照

在 Windows 宿主上以 Python 3.14.4 使用版本 2 的本地 Validate 交付报告，向 `--stage native_dynamic` 传入已准备好的 `animation` 插件。采用 `macos-aarch64` 目标，`native/` 内放 `libanimation.dylib` 和它的 `.dSYM` 目录。在两组合成 Mac 样本间只改变 `Contents/Resources/DWARF/` 下符号文件的名称，字节完全一致。对比 stage 的 `materialized_packages[0].loadable_artifacts`、交付读取结果和签名候选。五组导出均返回 0，包级与 stage 的完整文件清单均保留了对应文件及其正确 SHA-256。

| 目标和输入 | stage 运行库数 | 交付读取数 | 可选签名候选数 |
|---|---:|---:|---:|
| Windows，DLL 与 PDB | 1 | 1 | 1 |
| Linux，SO 与 DBG | 1 | 1 | 1 |
| macOS，DWARF 叶名 `libanimation` | 1 | 1 | 1 |
| macOS，同一份 DWARF 改名 `libanimation.dylib` | **2** | **2** | **2** |
| macOS，本地 UE 的 ShaderConductor 动态库和 dSYM | **2** | **2** | **2** |

实际第三方库为 7,367,080 字节，符号为 52,724,892 字节。两者均含 x86_64 和 arm64 切片，库文件两片的 Mach-O 类型均为 `MH_DYLIB`，符号文件两片均为 `MH_DSYM`。本次实测截至预建包交付和外部工具的输入选择，目标机器上的实际签名、公证与启动留给 macOS 验证。

## 6. 等价回归与修复判据

用可重复创建的同名 DWARF 夹具覆盖上述两条 macOS 路径，断言完整清单同时包含库和符号，而加载清单与下游候选恰含一项真实库。补充只有符号时的失败用例，并复用 Windows DLL/PDB 和 Linux SO/DBG 对照。修复在角色判断时识别属于符号包的祖先目录，让暂存报告、下游读取和外部处理候选使用同一规则。验收时复核复制后的相对路径、字节数、摘要以及合法运行库的交付。
