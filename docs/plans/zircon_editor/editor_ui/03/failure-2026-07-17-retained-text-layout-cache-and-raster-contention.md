---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: retained-text-layout-cache-and-raster-contention
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/03
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_text
  - zircon_editor/src/ui/retained_host/host_contract/paint_text.rs
  - zircon_runtime/src/ui/text
  - zircon_runtime/src/graphics/text
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input
tests:
  - retained paint text layout/raster/pixel tests
  - runtime text cache and font registry tests
  - editor search/rename/console text product trace
---

# EditorUI03：retained text重复排版、无界cache与栅格锁争用

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：F4 retained host paint geometry 4文件与paint text/test 30文件逐文件静态审查
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md`
- 共同责任：`docs/plans/zircon_runtime/text/01`、`docs/plans/zircon_runtime/text/04`、`docs/plans/zircon_runtime/text/09`
- 交接原因：EditorUI03拥有编辑器文本消费与验收，runtime text计划拥有字体、atlas与两级layout cache权威；retained host不得继续维护第三套平行排版/缓存系统。

## 失败现象与复现证据

Recording-only GPU路径先计算完整glyph layout，随后只记录display text并丢弃glyph。Runtime single-line路径又可依次执行runtime layout、shape line与fontdue layout；cluster advance/origin中存在多处glyph×grapheme扫描和per-glyph临时Vec。

字体偏好在文本和glyph路径反复获取全局RwLock并深clonefamily String。新字体request miss会重建并扫描系统字体database，resolved font以`Box::leak`永久保存；glyph cache是无界全局Mutex，Swash ScaleContext再用全局Mutex串行。Fontdue fallback缓存8倍bitmap，却在每次draw对每logical pixel重复浮点采样和降采样循环。静态证据与验收项见PERF-MVP-156至PERF-MVP-160。

Host window文本输入审查补充PERF-MVP-167：每个insert/backspace原先把完整current value复制为String，修改后再深clone给state，随后把另一份转SharedString发callback，focus也重复clone。本轮已复用单个SharedString handle并只move一次focus，消除同键重复副本；当前源Cargo待验收。最终仍必须消费runtime owned edit buffer/range delta，避免长度N逐键输入总复制O(N²)。

## 最低共享层根因

Retained host没有消费runtime text/09的ShapedRun/LayoutCache和runtime font/atlas生命周期，因而自行拼接layout、fontdb、fontdue、Swash和两个无界global cache。资源generation、cache owner、容量、失效和指标都未贯通，导致正确性测试很多但无法约束稳定帧的重复工作与内存上界。

## 参考边界

- Slint Parley的`TextLayoutCache`按item缓存shaped paragraph，并在scale factor变化和component销毁时清理。
- Slint software vector font以alpha-map字节数加权建立1 MiB thread-local CLRU。
- Slint Skia font cache使用容量64的CLRU；Godot TextServer提供font-size cache clear/remove生命周期。

## 架构修复验收

- Retained host与runtime paint消费同一个resolved layout authority；同content/style/width/generation每帧shape/layout不超过1，cluster merge总访问近线性。
- Typography以immutable snapshot + generation发布；稳定frame/glyph偏好锁和family String clone为0，变更精确失效。
- 进程系统字体扫描每generation不超过1；删除`Box::leak`，resolved-face与glyph cache有entry/byte硬上限、eviction和owner。
- Glyph miss single-flight；稳定命中不被全局ScaleContext串行；hit/miss/evict/bytes/wait/raster count进入性能证据。
- 超采样降采样只在raster miss执行；第二次draw sample-loop=0。Latin/CJK/RTL/emoji、fallback、baseline、ellipsis、cluster hit与GPU/Softbuffer pixels等价。
- 搜索框、树重命名和Console长行以1/1k/10k glyph场景记录主线程p50/p95、访问数、allocation与RSS上界。
- 10k追加输入总copied bytes近线性；单键不重复深copy完整value/focus，range delta、IME/preedit/commit/backspace/grapheme与callback order等价。

## 禁止临时方案

- 不得仅扩大HashMap、继续`Box::leak`或按frame清空cache隐藏无界生命周期。
- 不得让retained host、runtime layout和GPU recording各自保留一份独立shaping权威。
- 不得通过禁用fallback、复杂文本、平滑或超采样来换取指标。
- 不得把全局Mutex替换成另一把全局RwLock并宣称解决并行争用。

## 修复结果与回传

Open state: `待 EditorUI03 + runtime text01/04/09完成单一layout authority、generation/cache生命周期和产品动态验收`。
