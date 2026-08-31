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
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/raster.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/raster/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/raster/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs/metrics.rs
tests:
  - powershell -NoProfile -Command "$r=Get-Content -Raw -LiteralPath 'zircon_editor/src/ui/retained_host/host_contract/paint_text/raster.rs'; $d=Get-Content -Raw -LiteralPath 'zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs.rs'; if ($r -notmatch 'physical_raster_px_size' -or $r -notmatch '\.size\(physical_px\)' -or $r -match 'fallback_raster_scale_bits' -or $d -match 'TEXT_RASTER_SUPERSAMPLE') { exit 1 }"
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
- Swash主路径与Fontdue故障回退都消费同一物理ppem。Swash继续负责color outline/bitmap、
  alpha/subpixel格式、hinting、bearing和pen-origin phase；draw阶段只消费显式
  `sample_scale=1.0`。通用downsample helper仍保留给未来有独立预算的local supersampling，
  但固定`TEXT_RASTER_SUPERSAMPLE=8.0`已从生产路径删除。
- retained raster owner新增固定低基数profile span/counters，区分cache hit/miss、miss生成bitmap
  bytes、Swash/Fontdue route、并发miss重复发布，以及cache当前/峰值entry和bitmap bytes。驻留记账
  仅在profiling/test构建的miss发布时做常数次更新，不遍历cache，普通构建仍只执行原HashMap
  插入。当前不改无界cache或单mutex算法；先用这些计数
  配合1000次交互、RSS和CPU profile确定miss、锁竞争与驻留增长，再决定single-flight、分片或
  有界LRU。
- retained Swash现在保留实际`Image.source`：COLR `ColorOutline`的premultiplied RGBA在进入
  straight-alpha linear-light blend前原地unpremultiply，embedded `ColorBitmap`继续保留straight
  pixels；zero-alpha RGB清零。该合同与Runtime Swash owner一致，避免半透明彩色轮廓被二次乘alpha。

算法规模：缓存查找仍为均摊O(1)，每个有效`(face, glyph, physical ppem, phase, smoothing)`
仅栅格一次；等价物理ppem不再因逻辑字号/scale组合不同而重复驻留。此前Fontdue fallback
的固定8倍边长会产生理论64倍bitmap面积，而Swash又完全绕过该尺度；修复后bitmap面积只随
实际物理ppem平方增长。以上是结构性上界，不是实测性能或功耗数据。

已完成非Cargo验收：6个Rust owner通过scoped `rustfmt --check`与`git diff --check`，
Runtime text静态合同`116/116`通过。尚未获得managed `zircon_editor` Cargo check/focused tests，
也未运行100/125/150/200%当前源码WGPU截图、1000 click/pointer/resize、RSS、CPU/GPU timestamp
或功耗采样。因此本failure继续保持`open / implementation_complete /
managed_validation_and_ui12_visual_perf_pending`，不得声明Cargo GREEN、视觉GREEN、性能最优、
功耗接近其他引擎或UI12产品验收完成。
