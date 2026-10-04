---
handoff_kind: failure
status: open
created_at: 2026-08-31
summary_slug: retained-swash-native-scale-bypasses-physical-raster
origin_plan: docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md
fixing_plan: docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/12
fixing_child_dir: docs/plans/zircon_runtime/text/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/framework/text/glyph_raster
  - zircon_runtime/src/text/atlas/raster_key
  - zircon_runtime/src/text/native_bitmap_atlas/source_cache/worker.rs
  - zircon_runtime/src/text/raster/service
  - zircon_runtime/src/text/raster/swash/request.rs
  - zircon_runtime/src/ui/surface/text_artifact.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs.rs
tests:
  - powershell -NoProfile -Command "Select-String over Editor paint_text and Cargo for removed backend symbols must return no matches"
  - py -m unittest discover -s tools/tests -p 'test_runtime_text_*.py' -v
  - managed focused zircon_runtime glyph_raster_service and native bitmap atlas request tests
  - managed focused zircon_editor paint_text raster tests after the current shared compile blockers clear
  - current-source Editor WGPU screenshots at 100%, 125%, 150%, and 200% effective scale
---

# Text04: retained Swash主路径绕过物理像素栅格尺度

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md`
- 来源执行切片：UI12 M6 device-pixel AA、local supersampling与当前产品视觉验收审计
- 修复责任计划：`docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md`
- 交接原因：Text04已定义`physical_px = logical_px x scale_factor`、scale变化即重栅格且atlas/cache identity包含scale；retained Editor的Swash/fontdue raster owner也列在Text04责任路径。UI12不能在`.zui`样式层或上层paint调用点补偿字形采样密度。

## 失败现象与复现证据

UI12针对用户报告的圆角、矢量与文字像素感审计了实际产品绘制链。WGPU圆角使用analytic SDF + `fwidth` + 4x4 local samples，SVG使用2x/4x source supersampling和linear-light premultiplied resolve；但retained文字的主Swash路径仍固定为native 1x：

- `paint_text/draw/glyphs/metrics.rs`声明`TEXT_RASTER_SUPERSAMPLE = 8.0`，调用方把该值传给glyph raster cache。
- `paint_text/raster.rs::rasterize_cached_font_glyph(...)`优先调用Swash；只有Swash返回`None`时，fontdue fallback才消费传入的raster scale。
- `paint_text/raster.rs::rasterize_swash_glyph(...)`的scaler使用`.size(logical_px)`，没有消费物理scale或传入的8x local scale；返回值又固定`raster_scale: NATIVE_SWASH_RASTER_SCALE`。
- `paint_text/raster/metrics.rs`把`NATIVE_SWASH_RASTER_SCALE`定义为`1.0`；`paint_text/raster/tests.rs::retained_text_raster_uses_swash_for_ui_face`明确断言主路径`raster.raster_scale == 1.0`。
- Text04计划本身要求“栅格输入按物理像素”以及`physical_px = logical_px x scale_factor`，当前retained Swash消费者与该合同不一致。

当前源码SHA-256证据：

- `paint_text/raster.rs`: `DD205B227F7AB49574703782EAE60570EA1170DFAF4F0DE0CADE33AFFB6B8A86`
- `paint_text/draw/glyphs/metrics.rs`: `915A0602F1ECD60C272105BF215660CE2AA7132ED6E440C43C2349FE15185FCD`
- `paint_text/draw/glyphs.rs`: `D15DD9640B0EB3679D60107374EC6903E99D0F879E3115EA8841FFA7CEDD9F06`

这只证明主路径采样策略缺口，不声称当前产品视觉失败已通过截图量化。UI12的当前源码Editor构建仍被共享Runtime编译错误阻断，因此没有把旧WGPU截图或HTML设计预览冒充当前产品验收。

## 最低共享层根因

retained glyph cache入口虽然接收local raster scale，但Swash primary rasterizer没有该输入，仍把logical font size直接交给Swash并把结果标记为1x。上层8x常量实际只控制fontdue fallback的raster/downsample路径，主UI字体命中Swash时不会获得同一物理像素或local supersampling保证。由此形成两个采样authority：Swash native 1x与fontdue supersampled fallback。

## 架构修复验收

- Text04建立单一glyph raster density policy。Swash主路径的有效栅格尺寸不得低于当前surface physical scale；local supersampling是否高于物理scale应由清晰、有界、可测的策略决定，而不是由fallback身份决定。
- Swash必须继续作为主rasterizer，保留hinting、bearing、color outline/bitmap、grayscale/subpixel coverage和pen-origin phase语义；不得通过强制fontdue fallback获得表面上的高分辨率。
- glyph cache identity必须包含会改变像素输出的有效raster scale bucket与smoothing；DPI或有效scale改变时只重栅格受影响的glyph，稳态不得每帧重建。
- 若采用高于物理scale的local supersampling，alpha/subpixel/color resolve必须覆盖fractional phase、thin-stroke coverage、premultiplied/linear-light语义，并证明不会放大或侵蚀小字号笔画。
- focused lower-layer tests至少覆盖100%、125%、150%、200% scale，断言Swash source bitmap尺寸/metrics/returned raster scale与policy一致，并替换当前固定1x断言。
- UI12重跑retained文字与SVG、analytic rounded rectangle的同帧pixel crop；当前源码Editor WGPU截图必须证明小字号、圆角和矢量边缘在四种scale下无低分辨率放大、无彩边、无bearing/spacing回归。
- UI12性能门继续运行1000 click、1000 pointer move、200 resize；glyph scale切换可以产生有界cache miss，稳态交互不得出现持续栅格或无界RSS增长。

## 禁止临时方案

- 不得在`.zui`里放大字体、加粗字体、整数吸附所有文字或覆盖设备scale来掩盖1x bitmap。
- 不得关闭Swash、强制进入fontdue fallback，或删除hinting/subpixel/color glyph能力。
- 不得把固定8x无条件应用到全部字号、全部DPI而缺少内存、cache与交互性能预算。
- 不得只修改测试中的`raster_scale`字段而不改变Swash实际`.size(...)`与bitmap采样密度。
- 不得用旧WGPU截图、HTML预览或离线放大图代替当前源码产品帧验收。

## 修复结果与回传

### 2026-08-31 current-source预验证实现记录

结构复核以Unreal Slate为主参考：`FSlateFontKey`/`FShapedGlyphEntryKey`把有效
font scale或`ComputeFontPixelSize(...)`得到的物理render size纳入字形身份，字形加载消费同一
pixel size，不对全部字形再套固定8倍超采样。Bevy的`FontAtlasKey.font_size_bits`与Swash
`.size(font_size)`、Slint的physical `run.font_size()`到glyph renderer路径提供了同方向交叉验证；
Fyrox的显式`super_sampling_scale`则证明local supersampling必须是独立、具名、可预算的策略，
不能由fallback身份隐式决定。

本轮已完成最低共享层前向修复：

- `physical_raster_px_size(logical_px, surface_scale_factor)`与Runtime
  `GlyphRasterKey::px_size_bucket`使用同一`round(logical_px * scale_factor).max(1)`语义；
  retained draw中的`glyph.px`已经是物理frame字号，因此生产调用只以scale `1.0`分桶，避免DPI双乘。
- retained cache key移除`logical_px_bits + fallback_raster_scale_bits`双身份，改为唯一
  `raster_px_size`桶，并继续保留font source/cache key、glyph、subpixel phase与smoothing。
  13px在100/125/150/200%分别得到13/16/20/26 ppem；13px@125%与16px@100%
  复用同一bitmap cache entry。
- Runtime Swash主路径消费唯一物理ppem。Swash继续负责color outline/bitmap、alpha/subpixel格式、
  hinting、bearing和pen-origin phase；retained Editor不再拥有Fontdue故障回退、通用downsample helper
  或固定`TEXT_RASTER_SUPERSAMPLE=8.0`生产路径。
- 共享Runtime raster service新增固定低基数profile span/counters，区分request/actual bitmap route、
  bitmap bytes、lock wait/hold、backend time与success/failure。既有native source cache继续负责
  hit/miss、pending去重、worker背压/completion budget、eviction及resident entry/font/bitmap bytes。
  当前不改变cache、worker或单mutex算法；先用这些计数配合1000次交互、RSS和CPU profile确定
  重复同步栅格、锁竞争与驻留增长，再决定single-flight、分片或有界LRU。
- retained Swash现在保留实际`Image.source`：COLR `ColorOutline`的premultiplied RGBA在进入
  straight-alpha linear-light blend前原地unpremultiply，embedded `ColorBitmap`继续保留straight
  pixels；zero-alpha RGB清零。该合同与Runtime Swash owner一致，避免半透明彩色轮廓被二次乘alpha。
- Text04 hard-cut步骤2已完成源码实现：Runtime公开后端无关的完整glyph raster request、不可变
  receipt和类型化失败；每个`FontCollectionService`拥有一个私有`GlyphRasterService`/Swash会话，
  resolved artifact face可携带精确collection、face、instance、generation、source identity、variation
  与共享font bytes请求栅格，不向框架接口泄漏Swash类型或借用字节。共享服务只增加profiling激活时的
  固定低基数request/actual-format/bitmap-byte/lock-wait/backend-time/success/failure观测；在实测前
  不引入LRU、分片、single-flight、异步策略或第二套bitmap cache。
- native `GlyphRasterKey`已先投影为同一`TextGlyphRasterRequest`；同步service与异步worker共用
  `SwashRasterRequest::from_text_glyph_request(...)`的glyph/ppem/3x4 phase/hinting/smoothing/mode/
  synthetic校验和适配。worker batching、pending去重、背压、byte budget、face epoch、source LRU
  和GPU atlas residency保持原算法；SDF/MSDF与不支持的synthetic bold明确fail closed。
- 过去native atlas使用3个水平phase、retained私有路径使用8个phase，消费者会形成不同bitmap
  identity。Runtime request现在拥有唯一3x4 phase计数与screen-position量化函数，服务验证与后续
  consumer不再复制magic count；负坐标和非有限输入也有确定分桶。
- Runtime artifact owner早已能为省略号/合成all-LTR视觉run生成virtual glyph并跨字体代际重建，
  但surface adapter仍有旧的一律拒绝门，导致retained路径拿不到已发布产物。该门已源码删除，并新增
  真实ellipsized artifact回归测试；adapter仍要求精确layout line、font generation、collection与
  handle一致。另新增固定lookup outcome计数，在删除Editor旧成功路径前量化artifact覆盖和失败原因。
- retained Editor现在只接受完整Runtime artifact；布局以Runtime视觉行、glyph origin、line baseline、
  face/instance/generation为唯一事实源，缺失时fail closed，不再进入Fontdue或本地shaping路径。绘制端
  通过精确artifact face请求`TextGlyphRasterReceipt`，直接消费receipt的size、bearing、实际bitmap
  format与共享bitmap；斜体进入Runtime synthetic request，不再由Editor像素行二次倾斜。
- Editor私有system fontdb、Fontdue layout/metrics、Swash `ScaleContext`、无界glyph `HashMap`、
  artifact/font snapshot cache、8-bin phase模型及其源文件/测试/direct Cargo依赖已删除。硬切范围为
  1,566 insertions / 5,490 deletions，相关生产owner均低于800行。

算法规模：artifact face lookup为均摊O(1)，单次布局与绘制对glyph数保持O(n)，且不再执行
Fontdue与Runtime两套布局比对。此前Fontdue fallback固定8倍边长会产生理论64倍bitmap面积；硬切后
bitmap面积只随实际物理ppem平方增长。当前共享service刻意尚未加入bitmap residency cache，因此
重复帧可能重复栅格；必须先用已加入的lock/backend/route/bytes计数和CPU采样量化，再决定single-flight、
分片、有界LRU或异步调度。以上是结构性上界与待测风险，不是实测性能或功耗数据。

已完成非Cargo验收：retained硬切与Runtime契约/服务owner通过scoped Rust 2024
`rustfmt --check`与`git diff --check`。Runtime text静态合同本轮为`114/116`；两个失败都来自未修改、
未归属的并行共享owner：UI texture dependency实现不再匹配测试中的旧返回类型源码字符串，以及
`zircon_runtime/src/graphics/scene/scene_renderer/ui/image.rs`当前829行，触发800行结构门。当前终态
Runtime定向验证副本`d4da4a0a64424fbfa97845713680b0cf`在overlay ownership物化阶段失败，
Cargo未启动；刷新精确claim后，替代副本`1e51cc39937545b2a729d7ff8bcbd37a` / request
`8c9c4e7e59774579a4db9544fb067aee`已在`D:/cargo-targets/verify`接受，但尚无Cargo结果。
尚未获得managed `zircon_editor` Cargo check/focused tests，
也未运行100/125/150/200%当前源码WGPU截图、1000 click/pointer/resize、RSS、CPU/GPU timestamp
或功耗采样。因此本failure继续保持`open / runtime_and_editor_hard_cut_source_implemented /
managed_validation_and_ui12_visual_perf_pending`，不得声明Cargo GREEN、视觉GREEN、性能最优、
功耗接近其他引擎或UI12产品验收完成。
