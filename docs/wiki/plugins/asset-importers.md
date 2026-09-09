---
related_code:
  - zircon_plugins/plugin_sdk/src/manifest/importer_runtime.rs
  - zircon_plugins/asset_importers
  - zircon_plugins/gltf_importer/runtime/src
  - zircon_plugins/obj_importer/runtime/src
  - zircon_plugins/texture_importer/runtime/src
  - zircon_plugins/audio_importer/runtime/src
  - zircon_plugins/shader_wgsl_importer/runtime/src
  - zircon_plugins/ui_document_importer/runtime/src
implementation_files:
  - zircon_plugins/plugin_sdk/src/manifest
  - zircon_plugins/asset_importers
  - zircon_plugins/gltf_importer/runtime/src
  - zircon_plugins/obj_importer/runtime/src
  - zircon_plugins/texture_importer/runtime/src
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/zircon_plugins/01
tests:
  - zircon_plugins/asset_importers
  - zircon_plugins/gltf_importer
  - zircon_plugins/obj_importer
doc_type: module-detail
title: 资产导入器插件
status: source-audited
---

# 资产导入器插件

资产导入器把源文件转换成 Zircon 资产及附属产物。选择过程以 importer ID、扩展名或完整后缀、priority、输出 kind、版本和 required capabilities 为依据。导入器清单可以存在而实现能力仍为 partial，因此工具链必须保留诊断与失败路径。

## 注册契约

`ImporterRuntimeManifestBuilder::new` 统一 importer 包的 ID、显示名、runtime crate、dist crate 与 runtime entry。默认面向 `client_runtime` 和 `editor_host`，平台为 Windows/Linux/macOS，distribution 使用 ABI v3、descriptor symbol `zircon_native_plugin_descriptor_v3`。通过 `with_asset_importers` 追加实际 importer 条目，最后 `build_package_manifest` 生成包清单。

每个 `asset_importers` 条目定义：

- `id` 与所属 `plugin_id`；
- `priority`，同一后缀候选的决策依据；
- `source_extensions` 或 `full_suffixes`；
- `output_kind` 和可选 `additional_output_kinds`；
- `importer_version`，用于派生缓存/重导入失效；
- `required_capabilities`，宿主缺失时不得选择该 importer。

导入阶段应把源内容、导入设置和 importer version 都纳入产物身份；一个模型 importer 可以产生 Model、Mesh、Material、Texture 等多种产物，调用方不能假定一输入一输出。

## 专用稳定包

| 包 | 输入 | 输出 | 当前状态 |
|---|---|---|---|
| `gltf_importer` | `.gltf`, `.glb` | Model + Mesh 等附属资产 | stable 包，具体 glTF capability partial |
| `obj_importer` | `.obj` | Model + Mesh | stable 包，能力 partial |
| `texture_importer` | 常见图片、DDS/KTX/ASTC、PSD、cubemap/array 路径 | Texture | stable 包，image capability partial |
| `audio_importer` | WAV、MP3/OGG/FLAC/AIF/AIFF | Sound | stable 包，WAV capability partial |
| `shader_wgsl_importer` | `.wgsl` | Shader | stable 包，WGSL capability partial |
| `ui_document_importer` | 完整后缀 `.zui` | UiWidget + UiLayout + UiStyle | stable 包，UI document capability partial，importer version 2 |

`opus_importer` 是 experimental 的 `.opus` 专用包。它与通用 audio importer 分开，便于按编解码器依赖独立分发。

## 聚合 importer family

`asset_importer.model`、`.texture`、`.audio`、`.shader`、`.data` 是 experimental family 包。它们提供统一目录与 optional native backend 插槽：

| Family | 内建声明 |
|---|---|
| Model | glTF/GLB、OBJ、PLY/STL、DXF；FBX/DAE/3DS/USD 系列要求 native backend |
| Texture | 常见图片、DDS/KTX/KTX2/ASTC、PSD；cubemap/DXGI 可要求 native backend |
| Audio | WAV、常用 codec、Opus；部分格式要求 native backend |
| Shader | WGSL、Naga 支持的 GLSL/SPIR-V；HLSL/CG/FX 可要求 native toolchain |
| Data | TOML、JSON、YAML、XML |

这些 family 清单描述的是路由能力集合，不表示每一种格式当前都有完整产品级转换器。对生产导入应优先检查对应 capability status 和 runtime 源码，而不是只看扩展名列表。

## 调用与选择流程

1. 规范化路径和后缀；`.zui` 等 full suffix 优先按完整后缀匹配。
2. 从已激活且 capability 满足的 importer 中筛选候选。
3. 按 priority 决策；专用包通常使用 120，family 通常为 100 或 110。
4. 读取源字节与导入选项，调用 importer。
5. 校验主输出 kind 与附属输出；提交产物和依赖边。
6. 保存 importer ID/version 与源摘要，供重导入和缓存失效。

同一种格式可能同时由专用包和 family 声明，例如 glTF、OBJ、图片和 WAV。项目不应无意中同时启用多个同优先级实现；若同时启用，应记录最终选择的 importer ID，避免跨机器结果漂移。

## 原生 importer

需要外部 SDK/编解码器的 importer 可通过 `runtime.asset.importer.native` 能力和 native dist 交付。原生边界只传输 ABI 定义的字节与 manifest，不传递 Rust 资产对象；宿主负责把命令输出解析成资产导入结果。动态库拒绝能力或 schema 不匹配时，应保留源文件并报告“无可用 importer”，不能生成空资产冒充成功。

## 错误与可重复性

导入器应区分不支持格式、源损坏、缺少 capability、转换失败和附属资源缺失。相同输入、设置和 importer version 应生成稳定结果。对浮点几何、色彩空间、音频声道布局、Shader stage 与 UI schema 的隐式默认值都应写入产物元数据，否则重导入可能改变结果。
