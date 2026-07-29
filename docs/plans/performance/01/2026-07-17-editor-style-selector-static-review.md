---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme/palette_projection.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
tests:
  - style selector state/palette projection tests
  - current-source Windows Cargo pending
  - 1/100/10000 node theme-lock and role-classification trace pending
doc_type: implementation-evidence
status: partial_static_complete_dynamic_pending
---

# Editor style selector逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`style_selector/`在2026-07-17记录的静态审查基线为 **157/157** 个Rust文件、**7,825** 行；当前目录实测仍为 **157** 个Rust文件、**8,629** 个物理行。两者之间的内容差异尚未完成逐文件静态刷新，当前源Cargo、规模计数与像素验收也未完成，因此本模块仍留在`pending.md`。

## P0：单节点主题读锁扇出

`current_host_palette()`每次获取全局`RwLock<HostMaterialPalette>`读锁。部分选择器已采用正确边界：`slider`、`selection_control`、`status_control`与`chrome`在入口只取得一次palette，再以值或借用传给所有leaf helper；但其他选择器把palette lookup藏在每个属性helper内：

- dropdown一次选择分别为surface、border、text、chevron取锁4次；text field为surface、border、text、stepper、divider取锁5次。
- popup row按background/text/shortcut/adornment取锁，outline还可再次取shared row palette；list/tree row的border width会重新调用border，marked path又重复取得selected-row palette。tree row四个相同文字颜色helper也各自取锁。
- segmented control的background、border、selected surface/border/underline及三类text各自取得palette；normal/focused background还会再次进入normal helper，单次可达约9次palette读锁，并另取最多2次metrics锁。
- table row在selector内重复取得约5至7次palette；随后`WorkbenchTableRowStyle::text_for_cell()`又为每个可见cell取一次锁，所以主题同步次数还会乘以列数。
- button state helper彼此递归组合：focused调用hover，hover再调用normal，每层各取palette；add-component、prominent command与tab-like override继续各自取palette/metrics。一次button选择依state/role可产生约3至8次palette读锁。icon button也为background/border/glyph及可选command override分别取palette，并多次读取metrics。

这不是锁竞争出现后才成立的问题：即使无writer，原子/锁边界、函数调用和整份palette复制也按可见节点、按帧放大；主题切换时还增加与writer竞争机会。PERF-MVP-182要求先把每个selector局部收敛为一次`HostThemeSnapshot`，最终由EditorUI08在frame/changed-node compile边界取得一次generation-owned immutable snapshot。Slint的`ItemCache::evaluate_if_dirty`提供生命周期参考：依赖变化时重算，稳定draw消费缓存结果。

## P1：paint-time字符串角色识别

`icon_button/selection/danger.rs`每次paint把`control_id + icon_name + validation_level`经`format!`拼接，再`to_ascii_lowercase()`分配第二个字符串，随后做四次substring查找。generic icon glyph分类也存在相同的拼接/lowercase链。Workbench button先判断command role，进入override后再次判断同一role；tab-like gate命中后又依次重复module/asset/toolbar/utility分类。它们多数是稳定schema角色，却在每次节点样式选择时重新从字符串推断。

PERF-MVP-183的局部门要求不分配地分别执行ASCII case-insensitive匹配，并在一次selector调用中只分类一次；最终由template/presentation generation发布typed visual role，compiled segment直接消费。不得以无界全局字符串intern表或host-local第二套role cache取代schema owner。

## 动态验收

在1/100/10,000个normal/hover/pressed/focused/loading/selected nodes与4/8列table上记录palette/metrics lock acquisitions、role probes、allocated strings/bytes、selector builds和paint commands。局部门要求每个changed node的theme snapshot acquisition<=1、table cell额外acquisition=0、role classification<=1且steady role string allocation=0；最终stable generation这些计数均为0，frame theme全局读取近常数。保持所有既有state priority、declared color、command/tab/danger identity、theme switch原子性以及GPU/Softbuffer pixels一致。
