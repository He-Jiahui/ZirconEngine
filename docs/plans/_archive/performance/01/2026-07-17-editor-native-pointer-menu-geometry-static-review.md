---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/menu_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/menu_geometry
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/menus.rs
tests:
  - retained root/nested menu pointer and geometry tests
  - current-source Windows Cargo pending
  - 1000-event stable popup geometry build/clone trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor native pointer menu geometry逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`native_pointer/menu_geometry.rs` + `menu_geometry/**`共 **27/27** 个Rust文件、**627** 行已逐文件阅读，并核对move/scroll/press调用顺序。当前源Cargo、稳定popup build counter与规模trace未完成，因此仍留在`pending.md`。

## 已有正确边界

Root popup按shell bounds上下/左右clamp，submenu按parent row anchor选择左右并限制高度；open path只沿已选branch，blocking frame防止popup下方pointer落穿。Damage覆盖top bar和全部open popup bottom；root menu viewport保留Window menu scroll/clamp语义。算法沿submenu depth而不是遍历完整menu tree，深度本身不是当前问题。

## 热点与计划

PERF-MVP-175：同一稳定menu generation上，move/scroll/press会重复从DTO构造root source、root frame和nested popup stack。Popup move先用`menu_popup_handles_point`沿open path算containment；状态变化后`menu_damage_frame`再次沿path求bottom。Press还要分别为before/after state构造damage。Top bar hit线性扫menu frames；root source、damage root和每个nested level均用`row_data` clone menu/frame/selected branch，`SharedString=String`时branch strings为深copy。

局部先改`ModelRc::iter/get`和borrowed branch，消除candidate clone。EditorUI01/08最终应由menu state+layout generation一次提交root/nested popup frames、row ranges、blocking frame与damage bounds；稳定event只查该stack，submenu path delta只更新changed suffix。不能在native geometry建立与menu pointer bridge不同步的第二套open state。Slint的dirty-tracked menu shadow tree同样把menu结构投影绑定到property generation而不是每event重建。

## 动态验收

在root menu、10层submenu和1,000-row Window menu上各运行1,000次stable move/scroll/press，记录geometry build、path levels、candidate DTO clone、string copied bytes、allocation和damage computation。稳定move的stack build与DTO clone为0；一次path change只重算changed suffix；press before/after各读取对应generation bounds。保持scroll/clamp、separator/disabled、submenu左右/上下placement、blocking、z、damage和像素等价。
