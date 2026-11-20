# Web 精简 cooked 资产库不应要求 Scene 模块

## 实际故障

在 Windows 宿主、Rust 1.94.1 上对 `wasm32-unknown-unknown` 执行以下库检查，只启用 `cooked-assets`，不启用 `native-host` 和完整浏览器产品：

```text
cargo check -p zircon_runtime --lib --no-default-features --features cooked-assets --target wasm32-unknown-unknown --locked --offline --jobs 1
```

首次发现于 [#233](https://github.com/He-Jiahui/ZirconEngine/pull/233) 的 `2c04b37f`。Cargo 退出 101，唯一编译错误为 `E0433`：[BrowserAssetSource::load_scene](../../../zircon_runtime/src/asset/runtime/browser/mod.rs#L207) 的返回类型引用了当前不启用的 `crate::scene`。本地回执 SHA-256 为 `0ab1160f439a5ae58002655f76162ca9e49f13d3a900ee6574879ea12b574f4b`。与基准 `e89349ca` 对比，资产浏览器源、便携入口、库入口和锁文件相同；[#233 的唯一生产改动](https://github.com/He-Jiahui/ZirconEngine/pull/233/files) 是任选的非 wasm zstd 边，不启用 Scene，也不是该项诊断。此时还没有从本独立分支实跑；下文将以无生产修改的首发提交复验。

[cooked-assets 进入便携资产运行时](../../../zircon_runtime/src/asset/portable.rs#L30-L34)，其中 [wasm 平台装配 BrowserAssetSource](../../../zircon_runtime/src/asset/runtime/mod.rs#L59-L69)，但 [Scene 的库入口条件](../../../zircon_runtime/src/lib.rs#L42-L49) 是 `native-host`、`graphics`、`animation` 或 `ui`。资产组件负责 Raw 目录、资源表、有限的资源驻留和读取；只有 [浏览器启动组合](../../../zircon_runtime/src/browser/open.rs#L56-L85) 在这里需要反序列化启动场景、建立 ECS 世界并安装 Level。正式 `target-web` 已满足 Scene 条件；这次诊断并不表示完整 Web 产品已失败或已通过。

## 与 UE 对齐的边界

本地 UE 参考树 6.0.0 / UE5 / CL 0。Core 的 `FIoBatch::Read` 接受 chunk ID 并返回 IO 请求，不反向引用 Engine 的 World（`Engine/Source/Runtime/Core/Public/IO/IoDispatcher.h:224–242`）；Engine `UEngine::LoadMap` 读取 package，再由 `UWorld::FindWorldInPackage` 解释，其 World 行为不属于 Core IO（`Engine/Source/Runtime/Engine/Private/UnrealEngine.cpp:15833–15881`）。`Core::IO::IoChunk`（`Engine/Source/Runtime/Core/Tests/IO/IoChunkEncodingTest.cpp:39–87`）直接对 Core 层进行编码、读取和字节断言，不需要创建 World。没有把这个 C++ 用例冒充 wasm 构建门禁：它提供应保持的依赖边界，ZE 的直接回归是通过最小 Cargo 目标编译真实资产库，而非启用 Scene 后假装最小构建通过。

ZE 保留 Rust Cargo 特性裁剪、浏览器 Raw 工件及 HTTP 读取和自研 ECS，不移植 Unreal package、压缩、World 或 C++ 构建规则。本问题不修改资源 ID、场景格式、资源图、浏览器启动行为或 #233 的原生依赖。

## 独立回归与验收

1. 在本 Draft 的单文档首发提交执行上面的最小库检查，记录退出码与诊断；不能用 #233 的日志替代本分支重现。
2. 保持低层资产来源可以编译；将依赖 `DynamicScene` 的方法和只为该方法保存的状态限制到已有 Scene 装配条件，不把图形、UI 或桌面宿主间接拉入 `cooked-assets`。保留目录校验和已有 Scene 组合的公共入口。
3. 以完全相同的最小命令重验；另以 `--no-default-features --features target-web --target wasm32-unknown-unknown --lib` 检查原有启动场景调用仍能编译。两者是库编译检查，不替代浏览器运行、WebGPU 帧或合并门禁。
