# 开发构建按目标平台暂存 native_dynamic 插件

在 Windows 上指定 Linux triple 时，构建器接受目标并传给 Cargo，但随后要求暂存 `demo.dll`。Linux 对该 cdylib 的文件名是 `libdemo.so`。只改查找目录不能纠正这个文件名。

## 已复现的层级

在主干 `ce3adcc98292ee1a1470a69f459d67abb13051d8`，通过[参数解析](../../../tools/build/zircon_build_arguments.py)构造 `--targets plugins --plugins demo --mode debug --target-triple x86_64-unknown-linux-gnu`，调用[生产暂存逻辑](../../../tools/build/zircon_build.py)；仅替代 Cargo 执行，在正常的目标/profile 目录提供小型标记文件，按 Rust 返回的文件名创建。插件清单被复制，随后因找不到 `demo.dll` 而退出，正确命名的 `libdemo.so` 未交付。对 #161 的组合源码复跑亦如此：目录已正确，所求的 basename 仍错。

Rust 1.94.1 的 `rustc --print file-names --crate-name demo --crate-type cdylib --target <triple> -`（空 stdin，只查询命名，不编译）分别返回：Windows MSVC `demo.dll`、Linux GNU `libdemo.so`、macOS `libdemo.dylib`。没有创建或加载跨平台原生库，不能据此声称 Linux 或 macOS 引擎交付成功。

## 设计边界和验收

UE 的 `UEBuildTarget` 向 **目标** `BuildPlatform.GetBinaryExtension` 请求库后缀；例如 `UEBuildLinux.cs` 返回 `.so`。ZE 不移植 UBT：保留 Rust/Cargo 的 crate、target 和符号文件命名，保留插件清单、native_dynamic/rlib_static 与运行时平台准入。

- 这个入口随后会发布 `plugins/native_plugins.toml`，不是只把编译结果存入缓存。审查 [ZE 的导出平台策略](../../../zircon_runtime/src/core/framework/project/export_profile.rs)和[平台合同测试](../../../zircon_runtime/src/tests/plugins/export_build_plan_platform.rs) 后收窄验收：显式 Windows 目标按 DLL 命名，不回退到编译宿主的拼写；其余显式目标当前未获准加载，必须在调用该插件的 Cargo、复制文件与发布清单前诊断拒绝，而不是把错误的 `.dll` 简单替换为 `.so`。未提供目标时保留既有开发命名行为。
- 将“选对名字”和 #161 的“选对输出目录”分别验证；可从已产生的测试输入验证暂存/复制，不要求把主机二进制假称为另一个目标的产物。
- 此问题限开发构建的 native_dynamic 插件暂存。不借此扩展 Windows shipping profile、Wasm 原生插件加载能力、编辑器交叉发布或签名流程。

## 更改后验证

- 新增四项 Python 测试。原代码中两个控制方法通过；另外两项方法报告 11 个断言失败（五种不支持目标 × 正常/预演，加一项 POSIX 命名环境的 Windows 目标），零导入错误。修复后，四项连同插件载体、目标路径、暂存清单和来源用例共 **44/44** 通过。静态专用包仍交给其静态编译入口，不生成加载清单；未指定目标保持原行为。
- 在 Windows 用真实插件目录发现 Sound 和 `zircon_plugin_sound_dist`，对 Linux GNU、macOS ARM 两种显式目标运行 `build_plugins`；Cargo 指向不存在的程序以排除误编译。两次均直接得到平台准入诊断，产物目录与插件清单均未创建。Windows 目标的命名、复制和生成加载清单以标记文件验证，不是动态加载或 ABI 验收。
- 对现有同源 Windows 原生 `zircon_runtime` 测试程序（SHA-256 `f4f21db6a331c177649e8cb20e58def350f69104ef074512a262416432b3a1dc`）精确运行 `platform_target_policy_matches_host_resource_and_plugin_strategy`，不设平台过滤，1/1 通过。本次 Rust 生产和测试均未修改，也没有新增跨平台编译、链接或装载。