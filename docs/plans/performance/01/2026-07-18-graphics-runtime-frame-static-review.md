---
related_code:
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
tests:
  - current runtime-frame builder 1 of 1 Rust file reviewed, 521 lines
  - both tests read; visbuffer mark borrow source regression added
  - visbuffer full-Vec clone gate changed from RED to GREEN
  - scoped rustfmt and diff check passed
  - current-source Cargo, F2 overlay trace and RenderDoc capture pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics runtime frame静态审查（2026-07-18）

## 当前源覆盖

`build_runtime_frame.rs`当前1/1个Rust文件、521行已逐文件静态阅读，覆盖frame装配、Virtual Geometry snapshot→BVH/visbuffer overlay投影、bounds/cross线段生成与2条测试。

## 直接止损

visbuffer overlay只读snapshot marks，却先clone完整`Vec<RenderVirtualGeometryVisBufferMark>`再迭代。本轮helper改为返回borrowed slice；debug关闭返回空slice，开启直接借用snapshot storage，颜色、cluster lookup、线段顺序和overlay输出不变。源码门禁先RED后GREEN，新增borrow回归，rustfmt与diff检查通过。

## 剩余根因

frame builder仍无条件调用VG snapshot builder；BVH启用时每instance重建node-id `BTreeMap`与全部box/connector lines，visbuffer启用时重建cluster-id `BTreeMap`和每mark约16条line，随后clone source overlays追加。该工作只应在PERF-MVP-416明确debug subscription下执行，并消费generation-owned retained overlay/report；正常帧全部为0。

frame还clone frame visibility与previous motion camera，source extract只做Arc clone。前两者的最终owner归PERF-MVP-410/413/414 camera-slot submission artifact，不在frame builder建立第二cache。

## 验收状态

静态、源码RED→GREEN、rustfmt与diff门禁完成。Cargo validator仍在启动前JSON解析失败，2条测试没有current-source结果；RenderDoc CLI不可用且无capture。BVH/visbuffer 1/1k/100k marks/nodes的CPU、alloc、line bytes、draw/pass和pixel parity未验收，继续留在`pending.md`，不进入`review.md`。
