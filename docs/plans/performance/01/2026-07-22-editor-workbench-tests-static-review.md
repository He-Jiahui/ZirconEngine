---
related_code:
  - zircon_editor/src/tests/workbench
  - zircon_editor/src/ui/workbench
  - zircon_editor/src/ui/template_runtime
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/06-ui-extension-framework.md
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
tests:
  - workbench Rust test owner inventory 40/40 statically read
  - Editor06 Python source contracts 2/2 passed
  - current-source Windows Cargo and product traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor workbench tests静态审查（2026-07-22）

`zircon_editor/src/tests/workbench`当前 **40/40** 个Rust文件已逐文件阅读：root 2、chrome snapshot 5、fixture 3、host events 2、layout 12、project 3、reflection 5、registry 3、view model 5。

布局/窗口测试锁定drawer归属、attach/focus/close、split、preset、roundtrip与responsive geometry，但规模均为1–3个view/drawer，未覆盖focus/resize storm下全placement收集、window registry重建、同步持久化与changed=false；继续归PERF-MVP-077/097。view-model/reflection测试每例从完整fixture重建chrome、menu、drawer、floating window与reflection，plugin menu只含2项，不能替代PERF-MVP-099/538/560的1k/10k generation build、clone bytes与p95门。

`reference_surface.rs`连续调用`node_by_control_id`暴露共同热点：组件化workbench一次frame projection会查询9个required control，host `layout_frames`再查询约20个control；原`control_frame/visible_control_frame`每次线性扫描tree/arranged tree。本轮按TDD在`EditorWorkbenchTemplateSurface`构造时一次建立`control_id -> UiNodeId`，同surface generation的frame/visible lookup改为索引+node lookup，未知动态ID保留旧扫描回退。Editor06 Python源码合同由RED转2/2 GREEN，rustfmt与scoped diff check通过。该设计与本地Slint `dev/slint/internal/core/item_tree.rs`的`ItemRc { item_tree, index }`及静态`ItemTreeNode`数组一致：稳定树内以dense/index handle寻址，而不是每次按字符串遍历。

这只是PERF-MVP-128的局部止损：`RetainedUiHostModel/Projection::node_by_control_id`及若干popup/data-sync bridge仍线性扫描，动态/virtual结构也需要由`UiSurface` generation owner统一维护duplicate-aware control index，不能在各consumer保存失效副本。asset creation menu重建仍归PERF-MVP-560。

project测试使用真实目录创建/scan/import/save/load，且已有“active generation不重开manifest”正向合同；大项目I/O、workspace字节与异步durability继续复用PERF-MVP-075/453。reference/reflection及project测试会重建完整runtime/fixture，属于验证墙钟成本；不把它误记为产品热点。current-source Cargo、1k/10k control counter、F4 resize/menu/docking产品trace与RenderDoc仍待，目录不进入`review.md`。
