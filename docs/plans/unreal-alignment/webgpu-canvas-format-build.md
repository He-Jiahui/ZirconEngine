# [WebGPU/Wasm] Canvas 交换链格式写入阻断 Web 库构建

## 1. 现象

在 [#151 的实测提交][run] 上完整执行 `cargo build -p zircon_app --no-default-features --features target-web --lib --target wasm32-unknown-unknown --profile dev --locked --offline --jobs 1`，退出 101，未生成该次 `zircon_app` 库的 `.wasm`。`zr_rhi_wgpu` 在 [surface_bootstrap.rs:107:13][error] 报 **E0594**：``cannot assign to `descriptor.swapchain.format`, as `descriptor` is not declared as mutable``。该文件与[公开基线 B][base] 同为 Git blob `42b671edf08678d425b563c4482b7037c08a761b`；实测不依赖 #151 的输入修改。来源：[B 的解构与 Wasm 分支][assignment]。

## 2. 用户入口与产物

`target-web` 的 `zircon_app` 以 Wasm `cdylib` 提供 `WebEngine.load(canvas, assetRoot)`，由嵌入页传入画布与预处理场景目录。浏览器加载的是 `wasm-bindgen --target web` 输出的 `zircon_app.js` 和 `zircon_app_bg.wasm`；此次失败发生在生成绑定前。不涉及产品目录里尚未交付的 `WebClient` 项，也不是读取 `zircon-export.manifest.json` 的另一套导出页。来源：[库目标][manifest] · [特性][feature] · [Web 入口][entry] · [浏览器产物使用方][consumer] · [产品目录][catalog] · [导出页][export]。

## 3. 生产格式如何选择

Zircon 首先以 `Rgba8Unorm` 构造 Canvas 描述符；Wasm 分支向这张 surface 请求相容适配器，按 capabilities **原顺序**取第一个可映射的 BGRA/RGBA 8-bit（sRGB 或 UNORM）格式，写回描述符后才交给共同的相容性检查及设备接管。来源：[Canvas 调用点][canvas] · [选择与交接][select]。

相邻的 UE 6.0.0 路径是 **UnrealGame / Linux / Vulkan**。Vulkan 枚举当前 surface 的 `VkSurfaceFormatKHR`；优先保留请求的像素格式并匹配颜色空间，找不到该格式时尝试将同一颜色空间的已报告格式反查成 UE 格式。选出的格式与颜色空间实际写入 `VkSwapchainCreateInfoKHR`。这是原生交换链协商的类比，不是 UE 的 Wasm 编译门。来源（UE 树内路径、行、文件 SHA-256）：

```text
Engine/Source/UnrealGame.Target.cs:6–16  01a0387d4c1b583c07ff7e424aebf3d49df25456e8bc2780fc1af45442d95492
Engine/Source/Programs/UnrealBuildTool/Platform/Linux/UEBuildLinux.cs:764–781  d5c01da3133faef46e306da467f8da119c0b676a7827309802eb14aca93c6e61
Engine/Source/Runtime/RHI/Private/Linux/LinuxDynamicRHI.cpp:69–79,88–115  8aadc3af0651c76f4e42961ee67e60e3b01441d2e21fe241cd5734434689aed3
Engine/Source/Runtime/VulkanRHI/Private/VulkanSwapChain.cpp:227–268,270–336,529–537,594–595  dd5bb512689daaeec8667e516043ea64aae3c3564ac89cfd8eca5d9e5f65d396
```

## 4. UE 既有自动化

`Rendering.RHI` 在非 NullRHI 下注册 **Test RHI Pixel Format → RHI Target Formats**：按能力检查 `PF_R32_FLOAT` 渲染目标及若干深度格式，创建纹理、按需创建 SRV，断言返回成功。**Test RHI Copy → RHICopyTexture** 对 `PF_B8G8R8A8` 的 UNORM/sRGB 四种源、目标组合执行 GPU copy，再读回逐像素比对原值。这两条验证的是格式资源与数据，不创建窗口交换链。来源（注册、用例、读回，文件 SHA-256）：

```text
Engine/Plugins/Tests/RHITests/Source/RHITests/Private/RHITests.spec.cpp:18–20,67–74,91–98  062e7dbe9286457baed67522045e41a667bc39679216e926821fb767e88f1bc5
Engine/Plugins/Tests/RHITests/Source/RHITests/Public/RHITextureTests.h:257–355,918–943,981–1021  04d813c69313160e68442ec1ad15cbcf95aab0fc951ed1ab60982e961304e497
Engine/Plugins/Tests/RHITests/Source/RHITests/Private/RHITextureTests.cpp:18–64  52b442d6ae04a256b4a47273a36fc4cbbc99aeae84e69a83eac2b1f7d9aae9e3
```

## 5. 最小修法与独立范围

在解构之后仅用 `#[cfg(target_arch = "wasm32")] let mut descriptor = descriptor;` 建立可变局部绑定。保留原格式选择顺序、错误、原生枚举和描述符接口。[#235][dx12] 虽触及同文件，但只增加 Windows 测试的适配器跟踪，真正修的是已呈现帧的退休引用；[#151][key] 和 [#238][focus] 分别处理键值配对和画布失焦。来源：[建议插入点][assignment] · [#235 同文件改动][dx12-source]。

## 6. 回归与验收

用第 1 节命令复跑完整 Web 库，从 Cargo 产物事件取得这次链接的 `zircon_app.wasm`，再以 `0.2.122` 版 `wasm-bindgen --target web --out-name zircon_app --out-dir <新目录> <本次链接的Wasm>` 生成 JS 和 `_bg.wasm`，记录两件实物的大小与 SHA-256。现有原生 profile `cargo check` 和键盘 Wasm example 均不能替代这一门。若验收到能运行，再用同一对新产物和已知可用的 cooked 场景调用 `WebEngine.load`、等待正尺寸画布首帧并读取提交回执；构建、绑定和浏览器结果分别记录。来源：[现有检查范围][ci] · [锁文件][lock] · [现有浏览器调用方式][consumer]。

[base]: https://github.com/He-Jiahui/ZirconEngine/tree/e89349ca51b4e9d024c194cd3bdf645bd25b6c37
[run]: https://github.com/He-Jiahui/ZirconEngine/commit/89fe5c004285ec32d68b9aa2992a888c19166a97
[error]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/surface_bootstrap.rs#L107
[assignment]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/surface_bootstrap.rs#L73-L109
[manifest]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/Cargo.toml#L10-L12
[feature]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/lib.rs#L78-L83
[entry]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/web/mod.rs#L15-L49
[consumer]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/tools/validation/browser/validate-web-memory.mjs#L23-L64
[catalog]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/entry/product_host_config/product_role_catalog.rs#L86-L105
[export]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/plugin/export_build_plan/platform_host_files/browser.rs#L157-L180
[canvas]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/graphics/backend/backend/render_backend_new_offscreen.rs#L73-L106
[select]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_rhi_wgpu/src/production/surface_bootstrap.rs#L79-L144
[dx12]: https://github.com/He-Jiahui/ZirconEngine/pull/235
[dx12-source]: https://github.com/He-Jiahui/ZirconEngine/compare/e89349ca51b4e9d024c194cd3bdf645bd25b6c37...38cbb4c860169c2aaf8b7fc41a3702451b63eada
[key]: https://github.com/He-Jiahui/ZirconEngine/pull/151
[focus]: https://github.com/He-Jiahui/ZirconEngine/pull/238
[ci]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/.github/workflows/profile-feature-contract.yml#L49-L109
[lock]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/Cargo.lock#L8072-L8075

## 7. 独立草稿的产品回归

在 `d8d6d88120568dc1ae0b9dfe27f82c88e40e7fae` 执行 `cargo build -p zircon_app --no-default-features --features target-web --lib --target wasm32-unknown-unknown --profile dev --locked --offline --jobs 1`。Windows 上的 Rust 1.94.1 编译器以 101 退出，定位到 `zr_rhi_wgpu` 的画布交换链格式赋值。Cargo JSON 的唯一错误代码为 `E0594`。完整产品构建停在生成 Wasm 库之前。[赋值位置](https://github.com/He-Jiahui/ZirconEngine/blob/d8d6d88120568dc1ae0b9dfe27f82c88e40e7fae/zircon_runtime/crates/zr_rhi_wgpu/src/production/surface_bootstrap.rs#L107)

## 8. 完整 Web 产品库构建结果

在 `7ee0081901559eeb5d3cedfb6a5ce2e00d71ccdf` 上，相同包、目标、`target-web` 配置和依赖锁文件的构建退出码由 101 变为 0。Wasm 分支将局部描述符绑定为可写，随后仍按原有顺序选择适配器报告的画布格式。[格式选择](https://github.com/He-Jiahui/ZirconEngine/blob/7ee0081901559eeb5d3cedfb6a5ce2e00d71ccdf/zircon_runtime/crates/zr_rhi_wgpu/src/production/surface_bootstrap.rs#L73)

Rust 1.94.1 链接的 `zircon_app.wasm` 为 101,019,794 字节，SHA-256 为 `1c77cc2cc105f78a4c1479ead1fac3aa09c554cdd6c34cbd4161b6dd7e612656`。`wasm-bindgen 0.2.122 --target web` 从该文件生成下列产品。

| 文件 | 字节 | SHA-256 |
|---|---:|---|
| `zircon_app.js` | 91,172 | `025581fb2ba5be1575b4890c3a4d49a8a4e7ec3d5cedc4bd5089eb26a1be49da` |
| `zircon_app_bg.wasm` | 90,396,136 | `0a7fca86748bc794ca07eebe5c87c28838c95d4df2a7b3d6832c0b53b62e8658` |
