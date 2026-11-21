# 开发构建按目标平台暂存 native_dynamic 插件

在 Windows 上指定 Linux triple 时，构建器接受目标并传给 Cargo，但随后要求暂存 `demo.dll`。Linux 对该 cdylib 的文件名是 `libdemo.so`。只改查找目录不能纠正这个文件名。

## 已复现的层级

在主干 `ce3adcc98292ee1a1470a69f459d67abb13051d8`，通过[参数解析](../../../tools/build/zircon_build_arguments.py)构造 `--targets plugins --plugins demo --mode debug --target-triple x86_64-unknown-linux-gnu`，调用[生产暂存逻辑](../../../tools/build/zircon_build.py)；仅替代 Cargo 执行，在正常的目标/profile 目录提供小型标记文件，按 Rust 返回的文件名创建。插件清单被复制，随后因找不到 `demo.dll` 而退出，正确命名的 `libdemo.so` 未交付。对 #161 的组合源码复跑亦如此：目录已正确，所求的 basename 仍错。

Rust 1.94.1 的 `rustc --print file-names --crate-name demo --crate-type cdylib --target <triple> -`（空 stdin，只查询命名，不编译）分别返回：Windows MSVC `demo.dll`、Linux GNU `libdemo.so`、macOS `libdemo.dylib`。没有创建或加载跨平台原生库，不能据此声称 Linux 或 macOS 引擎交付成功。

## 设计边界和验收

UE 的 `UEBuildTarget` 向 **目标** `BuildPlatform.GetBinaryExtension` 请求库后缀；例如 `UEBuildLinux.cs` 返回 `.so`。ZE 不移植 UBT：保留 Rust/Cargo 的 crate、target 和符号文件命名，保留插件清单、native_dynamic/rlib_static 与运行时平台准入。

- 对显式桌面目标按该目标寻找实际 cdylib 文件名，不回退到宿主系统的拼写；无显式目标保持既有宿主命名行为。
- 将“选对名字”和 #161 的“选对输出目录”分别验证；可从已产生的测试输入验证暂存/复制，不要求把主机二进制假称为另一个目标的产物。
- 此问题限开发构建的 native_dynamic 插件暂存。不借此扩展 Windows shipping profile、Wasm 原生插件加载能力、编辑器交叉发布或签名流程。