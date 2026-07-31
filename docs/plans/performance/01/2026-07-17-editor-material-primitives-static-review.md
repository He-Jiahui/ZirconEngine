---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/render/13-texture-pipeline.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
  - dev/bevy/crates/bevy_ui_render/src/lib.rs
tests:
  - material primitive geometry/style/palette/text-measurement tests
  - current-source Windows Cargo pending
  - 1/100/10000 primitive build/lock/allocation/command trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor material primitives逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`material_primitives.rs`与`material_primitives/` directory tree 共 **150/150** 个Rust文件、**6,496** 行已逐文件阅读。覆盖alert、avatar、badge、chip、divider、paper、skeleton、text field、timeline的dispatch、identity、geometry、style、text/image和tests。当前源Cargo、产品计数、规模trace与像素验收未完成，因此仍留在`pending.md`。

## P0：稳定节点重复重建primitive spec

每个进入generic material路径的node先依次探测alert、chip、avatar、badge、skeleton、paper、timeline、divider八个handler。命中后又在几何、style和command helper中反复按分隔符扫描同一`component_variant`：chip的small/outlined/avatar/icon/deletable/color在frame、padding、surface、text和glyph路径多次重判；alert的icon/action/close/color/filled/outlined同样重复。`alert_color_token`对五个tone逐个构造`format!("color{}", pascal_case(token))`，一次token解析就产生多次临时String，且background/border/text/icon/cutout再次各自解析。

主题读取继续按属性扇出：paper background被求两次且与border/shadow分开取锁；skeleton fill/wave/border各取一次；chip background/border/foreground/avatar/delete各取一次；alert各surface/content helper独立取palette。文本路径也重复工作：avatar先分配label判断非空，再为text命令分配一次；divider/overlay badge把已有String再次`to_string`；badge overlay为外框和文字frame测量同一文本两次。

PERF-MVP-184要求changed node只解析一次typed material primitive spec、借用一次theme snapshot、解析一次label/text layout并写入PERF-MVP-178的compiled segment。局部优化可先沿入口传递classification/palette/borrowed label/measured width；最终stable generation的八类probe、variant scan、theme read、text copy/measure和command build均为0。不得给每种primitive建立独立、无generation约束的cache。

## P0：avatar逐帧RGBA圆角mask

Avatar image先经`template_image_pixels`取得owned `HostPaintImagePixels`；随后`apply_rounded_alpha_mask`对目标width*height的每个像素重新计算圆角包含关系并修改alpha，再用`format!`把尺寸、radius和旧key写成新resource key。由于visual pixel cache hit本身深clone RGBA，同一个未变化avatar会每次paint再次复制整图、扫描整图并生成key。label fallback也被重复分配两次。

PERF-MVP-185要求Render13按`(resource,generation,width,height,radius)`拥有有界mask/raster/texture variant，或在command/shader clip可精确表达时完全不生成masked RGBA。EditorUI08 compiled segment只持handle、clip/radius和generation；同一variant每generation最多mask/raster/upload一次，stable paint为0。

## P1：复合glyph与effect命令放大

Alert close和chip delete各用十个圆点quad画一个X；alert icon用三个quad，avatar fallback与chip leading icon各用两个，paper shadow固定三个layer quad。它们都是有界工作，但会扩大host command、clip clone、sort、Softbuffer primitive与GPU draw-list；command compilation只能消除重建，不能消除最终draw放大。

PERF-MVP-186先在MVP产品trace记录各primitive命中与host/RHI command数。常用复合glyph应改为真实resource、一个cached mask/atlas handle或renderer可批处理的typed compound primitive；paper shadow以typed shadow/effect保留三层语义并由renderer一次提交/批处理。不得为减少command而丢失clip、opacity或层次。

## 动态验收

在1/100/10,000个stable与changed alert/avatar/badge/chip/divider/paper/skeleton/text-field/timeline节点上记录handler probes、variant-token scans、palette/metrics locks、String allocations/bytes、text measurements、RGBA copied/masked pixels、host/RHI commands及CPU p95。Changed node要求classification/theme/label/measurement各<=1；stable generation均为0。Avatar同variant mask/raster/upload<=1/generation且cache entries/bytes有界。复合glyph产品命中可见，目标每glyph command=1；shadow保持三层像素语义并证明batch/draw下降。全部state、tone、geometry、text、clip/z/opacity与GPU/Softbuffer pixels parity必须通过。
