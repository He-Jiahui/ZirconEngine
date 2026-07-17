---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_recording.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_recording/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/**/*.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08
reference_sources:
  - dev/slint/internal/renderers/software/draw_functions.rs
  - dev/slint/internal/renderers/software/scene.rs
tests:
  - recording_only_square_borders_emit_one_border_command
  - recording_only_wide_square_borders_emit_one_border_command
  - existing paint frame, clip, alpha, image and pixel tests
  - current-source Windows Cargo and Softbuffer product trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor paint frame/recording/primitives逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`paint_frame.rs` + `paint_frame/**`共 **15** 个Rust文件、**529** 行；`paint_recording.rs` + `paint_recording/**`共 **3** 个Rust文件、**77** 行；`paint_primitives.rs` + `paint_primitives/**`共 **26** 个Rust文件、**1,309** 行。三组已逐文件阅读 **15/15、3/3、26/26**，但当前源Cargo与Softbuffer产品trace未完成，因此仍留在`pending.md`。

## 已有正确边界

Recording-only frame不分配整帧像素，shape/image/text先做有效尺寸和clip相交再记录；CPU frame的rect/line使用连续row span，opaque identity image尝试row copy；透明像素和scaled image保留alpha blend。Damage被裁到frame bounds，现有测试覆盖active/explicit clip不相交、alpha、identity image、record-only和atlas recording。

## 热点与直接修复

- PERF-MVP-154：普通方形border此前在recording-only路径展开为4个quad，宽度W的方形border展开为4W个quad；这些命令随后又分别统计、转换和提交。Command/replay本来支持`Border { width, corner_radius }`，本轮让record-only方形border直接发一个Border，并以1px/3px回归测试冻结命令数和参数。透明、clip外和软件直接像素路径不变。
- PERF-MVP-155：Softbuffer rounded fill对目标矩形每个像素重新验证frame、clamp radius、计算中心和距离；rounded border每像素对outer/inner各做一次。Slint software renderer预存`RoundedRectangle`参数并按line用整数shift/sqrt生成span/coverage，而不是每pixel重建几何。Zircon应预计算clamped outer/inner geometry并按row生成连续span，alpha blend仅处理边缘coverage。
- PERF-MVP-151在本组得到最低层证据：`record_host_frame_commands`设置damage clip后仍无条件调用完整`draw_workbench_presentation_commands`。Primitive clip能避免像素/command写入，但不能避免上层node/pane访问、字符串构造与资源解析。
- PERF-MVP-150/153在本组得到资源证据：atlas duplicate sibling已删除，但`HostPaintAtlasImage`仍内嵌owned RGBA；scaled image每目标pixel执行浮点坐标换算，identity opaque path每generation缺alpha metadata。

## 动态验收

1/1k/10k普通/宽/圆角border记录command count、draw item count、CPU prep和Softbuffer p95；普通/宽方边框每实例command=1。1080p圆角surface记录geometry evaluations、filled spans、edge pixels和allocation。Full/patch、透明alpha、clip、圆角、同z顺序、GPU/Softbuffer pixels必须与当前结果一致。
