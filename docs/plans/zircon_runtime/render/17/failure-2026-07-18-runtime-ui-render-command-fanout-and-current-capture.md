---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-render-command-fanout-and-current-capture
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/render/17-performance-and-profiling.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/render/17
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/surface/render
  - zircon_runtime/src/rhi_wgpu/ui_surface/batching.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/geometry.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/render_pass.rs
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Rendering/ElementBatcher.cpp
tests:
  - repeated guide tick row batch-count scale test
  - owner-specialized overdraw pixel test
  - current-source F4 RenderDoc capture
---

# Runtime UI render command fanout与当前源码GPU capture缺口

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：surface render 32/32
- 修复责任计划：`docs/plans/zircon_runtime/render/17-performance-and-profiling.md`
- 联动责任：EditorUI06限制逻辑primitive，EditorUI08发布command ranges。
- 交接原因：GPU batch/vertex/upload/overdraw指标与RenderDoc验收属于Render17。

## 失败现象与复现证据

PERF-MVP-288/291：多command node缺stable identity，tree guides/slider ticks/all rows可制造大量原子command，每条复制style/String；owner与specialized surface可能重复。当前只有RenderDoc v1.44工具历史探测，尚无当前源码F4 capture。

## 最低共享层根因

CPU逻辑command没有compact brush/style handle、instance range与可观测budget，且当前产品graphics backend还未产出可比较capture。

## 架构修复验收

- repeated guides/ticks/rows按compatible brush/clip/z/state合并或实例化，command/vertex/draw增长近visible primitives。
- 记录CPU command bytes、batch merge、vertices/indices、uploads、draw/pass与overdraw。
- 同配置冷帧+稳定帧当前源码RenderDoc capture，标明backend/adapter/resolution/build hash；像素一致。
- owner/specialized重复surface有明确证据后只保留一份authority。

## 禁止临时方案

- 不得用2026-07-17旧capture或advanced scene代替当前F4产品路径。
- 不得通过降低UI内容、隐藏rows或丢失clip/z语义伪造draw下降。

## 修复结果与回传

Open state（2026-08-01 current source）：非验收实现已完成，accepted closeout仍待真实当前源码证据。

- `UiSurfaceDrawList::{with_compact_styles,with_generation_and_compact_styles}`在runtime-owned RHI边界将重复quad/border/text状态收敛为generation-owned `UiSurfaceStyleHandle`/style table；动态text仍独立持有，font family按可见handle只计一次。editor owned chrome stream（含F4 generation路径）已切换到compact构造器，legacy构造器继续兼容。
- WGPU普通quad与非圆角border叶片不再冷帧分配6顶点Vec，也不再驻留/上传6x24-byte vertex payload；`SolidGeometry::{Instance,Vertices}`互斥保存32-byte `SolidInstance`或复杂tessellation，`SolidDraw.instance_start..instance_end`保存连续range，shader以`vertex_index`生成6个角点。圆角与裁剪三角形继续走原vertex路径，未降低UI内容或丢失clip/z语义。
- repeated guides/ticks/rows scale guard覆盖1/100/1000/10000 primitive：无依赖行列保持1 draw、N instances、0 retained solid vertices、6N实际vertex invocations；WGPU retained pixel tests继续覆盖实例shader的真实离屏像素路径。
- RHI/editor F4 profile已新增submitted/compiled `solid_instance_count`与visible compact style count，同时保留command payload bytes、batch merge、vertices、upload bytes、draw/pass/overlap等既有口径。
- 第二轮实现审查已前向修复read/modify borrow表达式与`SolidItem`空Vec+Option双重元数据；`rhi/ui_surface.rs`按结构规范拆出`rhi/ui_surface/compact_styles.rs`与`rhi/ui_surface/tests.rs`，当前父文件583行、style owner 295行、test owner 375行。
- 当前完整模块树rustfmt与`git diff --check`通过；拆分测试owner的`#[path = "tests/scale_and_cache.rs"]`已由rustfmt模块解析前向校正，WGPU surface setup生产路径也已移除内部不变量`expect`。最新受管编译入口request `4dca61081c8a4e2b88cc857eb66dd89e`仍只确认`session.register` post-response accepted timeout，validator未启动Cargo或产生validation receipt；按协调规则不轮询该请求。
- remaining acceptance：当前源码Cargo编译/聚焦测试、F4 cold/stable GPU counter对拍、真实PNG与RDC（含backend/adapter/resolution/build hash）仍待生成后回传；在此之前保持`status: open`且不宣称accepted/fixed。
