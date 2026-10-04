---
related_code:
  - zircon_runtime/src/graphics/text_transport
tests:
  - graphics text transport current source 1 of 1 Rust files and 286 lines reviewed
  - all 5 tests read
  - transport call sites traced through UI measure layout wrapping ellipsis rich text and hit testing
  - no isolated code change; ownership fix remains PERF-MVP-157
  - current-source Cargo, style conversion counters and F4 text trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics text transport静态审查（2026-07-18）

## 当前源覆盖

`graphics/text_transport/**`当前1/1个Rust文件、286行已逐文件静态阅读，5条测试已读。除`&UiResolvedStyle -> TextStyle`外，range/direction/writing/alignment/wrap/rich-format/render-mode/frame/size均为Copy字段或有限enum match，无分配、锁、I/O或调度热点。

## 发现与责任计划

style transport每次clone `font`、`font_family`、`language`三项`Option<String>`。调用链不是低频边界：`ui/text/adapter.rs::text_style`被measure、line metrics、wrapping、ellipsis、paragraph/rich inline、hit-test与measure-cache key多次调用；同一layout操作可重复复制相同字体/语言identity，且部分候选测量循环会放大次数。

该问题已由PERF-MVP-157覆盖：EditorUI03发布typography/style generation，字体资产/family/language使用shared/interned descriptor，layout/shaping/raster一次抓取并借用；Text09记录转换/clone/cache计数。直接把本文件改成借用DTO会穿透`TextStyle` serde、rich-run mutation、measure-cache key和worker lifetime，形成第二套owner，因此本轮不做局部双轨类型。

本地Bevy `dev/bevy/crates/bevy_text/src/text.rs::TextFont`使用`FontSource`，handle路径共享资产、family路径使用`SmolStr`；UE `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/SlateFontInfo.h`以`TSharedPtr<const FCompositeFont>`和`FName`表达字体权威与typeface identity。采用“generation-owned字体描述符、短identity共享、一次layout构造”的原则，不复制其ECS/Slate API。

Text09计划文件当前被另一会话租约占用，本轮未越权写入；PERF-MVP-157主计划与EditorUI03已明确保留该责任。

## 验收状态

1/1静态阅读和5条测试阅读完成。无独立代码改动；Windows Cargo validator仍在启动前`ConvertFrom-Json`失败。仍需1/1k/10k text nodes、plain/rich、wrapping/ellipsis/hit-test下记录style conversions、font/family/language clone bytes、layout/shape/cache calls、CPU p95/RSS，并补F4 editor text/IME与像素对拍。完成前保留`pending.md`，不进入`review.md`。
