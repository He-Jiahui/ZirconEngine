# Web 精简 cooked 资产库不应要求 Scene 模块

## 实际故障

在 Windows 宿主、Rust 1.94.1 上对 `wasm32-unknown-unknown` 执行以下库检查，只启用 `cooked-assets`，不启用 `native-host` 和完整浏览器产品：

```text
cargo check -p zircon_runtime --lib --no-default-features --features cooked-assets --target wasm32-unknown-unknown --locked --offline --jobs 1
```

首次发现于 [#233](https://github.com/He-Jiahui/ZirconEngine/pull/233) 的 `2c04b37f`。Cargo 退出 101，唯一编译错误为 `E0433`：[BrowserAssetSource::load_scene](../../../zircon_runtime/src/asset/runtime/browser/mod.rs#L207) 的返回类型引用了当前不启用的 `crate::scene`。本地回执 SHA-256 为 `0ab1160f439a5ae58002655f76162ca9e49f13d3a900ee6574879ea12b574f4b`。与基准 `e89349ca` 对比，资产浏览器源、便携入口、库入口和锁文件相同；[#233 的唯一生产改动](https://github.com/He-Jiahui/ZirconEngine/pull/233/files) 是任选的非 wasm zstd 边，不启用 Scene，也不是该项诊断。随后用独立首发提交 `f50064afa` 和完全相同的命令复跑：同一处 `E0433`，退出 101；该次回执 SHA-256 `8919758771b5328979b2408f81806cd0a335839901487c9d66ef8edbb0bf6ec9`。

[cooked-assets 进入便携资产运行时](../../../zircon_runtime/src/asset/portable.rs#L30-L34)，其中 [wasm 平台装配 BrowserAssetSource](../../../zircon_runtime/src/asset/runtime/mod.rs#L59-L69)，但 [Scene 的库入口条件](../../../zircon_runtime/src/lib.rs#L42-L49) 是 `native-host`、`graphics`、`animation` 或 `ui`。资产组件负责 Raw 目录、资源表、有限的资源驻留和读取；只有 [浏览器启动组合](../../../zircon_runtime/src/browser/open.rs#L56-L85) 在这里需要反序列化启动场景、建立 ECS 世界并安装 Level。正式 `target-web` 已满足 Scene 条件；这次诊断并不表示完整 Web 产品已失败或已通过。

## 与 UE 对齐的边界

本地 UE 参考树 6.0.0 / UE5 / CL 0。Core 的 `FIoBatch::Read` 接受 chunk ID 并返回 IO 请求，不反向引用 Engine 的 World（`Engine/Source/Runtime/Core/Public/IO/IoDispatcher.h:224–242`）；Engine `UEngine::LoadMap` 读取 package，再由 `UWorld::FindWorldInPackage` 解释，其 World 行为不属于 Core IO（`Engine/Source/Runtime/Engine/Private/UnrealEngine.cpp:15833–15881`）。`Core::IO::IoChunk`（`Engine/Source/Runtime/Core/Tests/IO/IoChunkEncodingTest.cpp:39–87`）直接对 Core 层进行编码、读取和字节断言，不需要创建 World。没有把这个 C++ 用例冒充 wasm 构建门禁：它提供应保持的依赖边界，ZE 的直接回归是通过最小 Cargo 目标编译真实资产库，而非启用 Scene 后假装最小构建通过。

ZE 保留 Rust Cargo 特性裁剪、浏览器 Raw 工件及 HTTP 读取和自研 ECS，不移植 Unreal package、压缩、World 或 C++ 构建规则。本问题不修改资源 ID、场景格式、资源图、浏览器启动行为或 #233 的原生依赖。

## 修复与验证

将[启动场景状态](../../../zircon_runtime/src/asset/runtime/browser/mod.rs#L52-L65)与 [`load_scene`](../../../zircon_runtime/src/asset/runtime/browser/mod.rs#L219-L258) 限制到已有的 Scene 装配条件；[目录校验](../../../zircon_runtime/src/asset/runtime/browser/mod.rs#L93-L110)仍在所有浏览器资产组合执行。未修改 Cargo 特性、锁文件、Raw 目录格式或浏览器调用方，也未为精简资产额外启用 Scene、图形或 UI。

Windows 宿主、Rust 1.94.1、`wasm32-unknown-unknown`、`zircon_runtime --lib`、dev profile，默认特性关闭、`--locked --offline --jobs 1`。以相同命令单独选择下列特性，不混入其他 Draft：

| 特性 | 实际结果 | 本地回执 SHA-256 |
|---|---|---|
| `cooked-assets`，纯文档首发 | 退出 101：`E0433`，未装配 `crate::scene` | `8919758771b5328979b2408f81806cd0a335839901487c9d66ef8edbb0bf6ec9` |
| `cooked-assets`，修复后 | 退出 0：最小浏览器资产库通过 | `9a8709456ccbf9243332d0af654657cef617adb14d09cf15eed3cc2bd129a348` |
| `animation`，修复后 | 退出 0：含 Scene、不含 WGPU；`load_scene` 方法实际参与编译 | `d187976afa0ae133d72f8379173799ce025330c91478ad57f3ad56c331664aa0` |
| `target-web`，修复后 | 退出 101：未改动的 `zr_rhi_wgpu` Canvas 格式赋值 `E0594`，对应 [#239](https://github.com/He-Jiahui/ZirconEngine/pull/239) | `2bd886b2da7da577d48ec955ebaa24958192ff5ef4367b882404760ab96dade5` |

这是库编译检查，不执行 UE 自动化、浏览器或 WebGPU 帧。完整 `target-web` 检查未到达 BrowserEngine；需要修复进入基准后重验，不能把最小库通过当作完整 Web 通过。GitHub 的必需检查仍独立生效。
