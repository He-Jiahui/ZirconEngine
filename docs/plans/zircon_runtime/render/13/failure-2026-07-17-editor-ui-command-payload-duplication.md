---
handoff_kind: failure
status: source_complete_dynamic_validation_pending
created_at: 2026-07-17
summary_slug: editor-ui-command-payload-duplication
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/render/13-texture-pipeline.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/render/13
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/recording/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/image
  - zircon_runtime/crates/zr_rhi/src/ui_surface.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/image.rs
tests:
  - UI image resource generation/upload deduplication
  - atlas multi-command CPU payload ownership
  - device-loss/reload and atlas UV parity
---

# Render13：Editor UI command按draw重复携带纹理字节

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：F4 host command stream 40文件与image primitive 9文件逐文件静态审查
- 修复责任计划：`docs/plans/zircon_runtime/render/13-texture-pipeline.md`
- 共同责任：`docs/plans/zircon_editor/editor_ui/08`
- 交接原因：Render13拥有texture resource registry、generation失效与GPU upload边界；EditorUI08只能提交changed handle，不能在Editor侧私建第二份纹理权威。

## 失败现象与复现证据

Editor UI图像在recording、Chrome stream和RHI draw list中使用owned `Vec<u8>`。同一atlas被多个draw command引用时，每条命令可各自深clone完整atlas RGBA；此前普通GPU present还会在stream到RHI终跳再clone一次，atlas recording也同时保留一份下一阶段必丢弃的普通RGBA。

本轮已让普通GPU present消费stream并移动终跳text/resource/RGBA分配，删除atlas recording中必丢弃的第二份bytes，并用回归测试冻结两处局部行为。这仍没有解决每command atlas clone和每次patch携带上传字节的owner问题。

PERF-MVP-179补充：Editor manual icon fallback把一个glyph展开为2–8个quad。若MVP shipped icon trace仍有asset miss，Render13应以有界`(glyph kind,size,tint,theme/resource generation)` mask/atlas resource承接fallback，Editor只提交一个handle/UV command；不得把多quad永久编译进每帧draw list。首选验收仍是补齐shipped assets并让产品fallback count为0。

PERF-MVP-180补充：Editor sprite atlas resolver在paint路径每次重新扫描目录、stat/clone manifest并`image::open`解码整张atlas。Render13 registry必须持有generation-owned decoded CPU image与GPU texture；同atlas generation parse/decode/upload各≤1，draw只带handle+UV。Editor10 file watcher提供source-key index与精确失效，Render13不得靠每draw stat文件维持新鲜度。

PERF-MVP-181补充：普通visual asset cache hit仍深clone完整`HostPaintImagePixels`，key按size/tint/path无限增长；retained preview每paint复制/hash全RGBA，missing/MUI/SVG variant仍会同步raster。Render13按`(resource,generation,size,tint)`提供有界raster/texture handle；能由shader tint表达的颜色不得生成RGBA副本。Stable draw的decoded/raster/pixel copied bytes与upload为0。

PERF-MVP-185/186补充：avatar在visual cache返回owned RGBA后每paint逐像素重做相同圆角alpha mask并格式化派生key；同一头像没有variant生命周期。Render13应按`(resource,generation,width,height,radius)`缓存有界mask/raster/texture，或在shader/command精确表达radius。Alert close/chip delete等复合glyph当前展开2至10个quad；产品命中证明需要时，以单个mask/atlas handle或typed compound primitive承接，paper shadow以typed effect保留三层语义并支持一次提交/批处理。

PERF-MVP-187补充：MUI X line/pie/sparkline/gauge在paint线程每帧生成最大约147KiB RGBA并逐像素栅格，key却只含kind+size。Render13优先接收typed line/arc/pie/gauge geometry；必须raster时按`(kind,size,data/state generation,theme generation)`持有有界CPU/GPU variant。缺value/theme/hole/data generation的key禁止进入registry。

## 最低共享层根因

`UiSurfaceImagePayload`把resource identity、upload metadata和owned CPU bytes合并在每条draw command中；Render13没有可由Editor引用的稳定UI texture handle/generation，也就无法把“资源变化时上传一次”和“每帧提交几何/UV”分成两个生命周期。

## 参考边界

- Bevy `ExtractedUiNode`保存`AssetId<Image>`，`ImageNodeBindGroups`按该ID缓存bind group。
- Slint femtovg renderer使用`TextureCacheKey -> Rc<Texture>`，item不复制解码像素。
- Godot canvas command保存`RID texture`，纹理存储独立管理资源生命周期。

## 架构修复验收

- `UiSurfaceCommandKind::Image`只携带稳定resource handle、UV、采样/颜色信息和generation；像素/纹理驻留Render13统一registry。
- 同一resource generation的CPU decoded/atlas payload副本数=1、GPU upload≤1；steady damage patch无resource change时upload bytes=0。
- changed/reloaded resource以generation失效并只重传一次；device loss可重建，不暴露悬空GPU对象或让Editor另建像素cache。
- 1/1k/10k共享atlas commands记录recording/stream/RHI allocated/copied bytes、upload count和main-thread p95。
- Atlas UV、dynamic viewport image、Softbuffer fallback、GPU pixels、clip/z-order与现有结果等价。
- Shipped MVP icon的asset resolve/fallback有计数；fallback=0以守卫锁定，或确需fallback时每glyph command=1、raster/upload≤1/key/generation且cache bytes有界。
- Stable atlas generation的paint-thread filesystem/parse/decode=0；同generation CPU decoded副本=1、GPU upload≤1，reload/device-loss精确重建且registry entries/bytes有界。
- Stable visual resource的pixel cache hit不clone RGBA；同variant decode/raster/upload≤1/generation，size/tint cache按bytes有界并可观测eviction。
- Stable avatar的RGBA copy/mask/key-format/upload=0；首次或changed `(resource,generation,size,radius)` variant的mask/raster/upload≤1，device loss精确重建且cache bytes有界。
- 常用material复合glyph的host/RHI command=1、raster/upload≤1/key/generation；typed paper shadow保持三层像素语义并证明提交/draw下降，clip/z/order/opacity不变。
- Stable MUI X chart的paint-thread RGBA alloc/raster/key-format/upload=0；changed chart raster/upload≤1/generation，typed geometry可批处理，cache identity区分theme/value/hole/data且entries/bytes有界。

## 禁止临时方案

- 不得仅把`Vec<u8>`替换成散落在每条command里的`Arc<Vec<u8>>`并称为最终资源系统。
- 不得按frame清空registry或用无界resource-key map隐藏泄漏。
- 不得删除Softbuffer/截图像素payload；它们必须通过显式CPU resource/capture路径取得。

## 修复结果与回传

Open state: `实现与独立静态复核已完成，待协调器受管动态验收`。

- Editor Chrome stream、`zr_rhi::UiSurfaceDrawList`、WGPU geometry/batching/upload/cache/render pass 均以精确 `(resource_key, generation)` 作为图像身份；同键的并存 generation 不会覆盖、错误批处理或错误采样。
- 资源表压缩只移除已驻留的同一资源身份，RHI 上传统计、WGPU admission/LRU/invalidations 和 render-pass bind-group 查询也使用同一对键；1/1k/10k 同 generation atlas 命令维持一次 source/upload 与一次批处理。
- 场景 UI 图像准备路径改为借用 `ResourceStreamer` 的已准备纹理并在绘制记录中保存紧凑 binding handle；缓存保留唯一资源所有权，稳定帧不再为每条命令克隆 `Arc<BindGroup>` 或 `Arc<GpuTextureResource>`，热重载仍由已准备资源的实际身份精确替换。
- 两次独立静态审查已完成；发现的 borrowed stream residency converter 重导出遗漏已前向修复。`rustfmt --check`、限定 `git diff --check` 与旧 key-only 语义扫描均已通过。
- 待协调器以可租赁 Session 身份运行受管 Cargo、真实 WGPU 1/1k/10k/热重载与 device-loss 帧、RenderDoc capture 和新 PNG 截图；截图必须写入 `docs/tests/runtime/render`。在该证据产生前不得关闭本 failure 或写为 accepted。
