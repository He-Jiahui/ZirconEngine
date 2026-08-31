---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-dispatch-route-clone-and-timer-scan
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs
  - zircon_runtime/src/ui/dispatch/navigation/dispatcher.rs
  - zircon_runtime/src/ui/dispatch/input_manager/timers.rs
  - zircon_runtime/src/ui/dispatch/input_manager/manager.rs
  - zircon_runtime_interface/src/ui/surface/pointer/route.rs
  - zircon_runtime_interface/src/ui/surface/navigation/route.rs
---

# Runtime UI owned route clone与timer全扫

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/ui/dispatch` 13/13与`platform_input` 3/3
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`
- 联动责任：Runtime12已拥有跨系统input retention/coalescing；EditorUI01仍需验证UI batch不绕过频率语义。
- 交接原因：统一input manager、route context与timer owner属于EditorUI01 M2/M4。

## 失败现象与复现证据

PERF-MVP-254：pointer/navigation route包含多组node Vec，dispatch会在result、candidate与逐node context间深copy。pointer同node多handler原先还逐handlerclone，本轮已TDD降到每node/phase一次。PERF-MVP-255：每tick独立全扫四个timer BTreeMap并clone due String，再逐项dispatch。

## 最低共享层根因

route/result/context都要求owned DTO，没有一份event-lifetime共享route；timer按功能拆成四份以target排序的表，没有统一deadline authority、generation取消和frame budget。

## 架构修复验收

- handler context借用或Arc共享单一route，result move/共享同一payload；候选遍历不得clone stacked/bubbled/root slices。
- 1/10/100 depth×1/4 handlers记录route clone count/bytes、Vec alloc、visited/candidate copy与CPU p95；handler数不增加clone bytes。
- timer使用统一deadline queue/wheel，`target+kind+generation`支持replace/cancel；tick无due为O(1)，due近O(K log T)。
- due dispatch有count/time budget、age/fairness与deferred计数；同deadline次序稳定，stale entry不触发。
- capture/preview/direct/bubble/passthrough、tooltip/submenu/typeahead/toast/double-click、saturation与current-source Cargo/产品trace通过。

## 禁止临时方案

- 不得只reserve route Vec或把BTreeMap改HashMap而保留深copy/全扫。
- 不得每timer kind各建私有heap；deadline与预算必须统一。
- 不得通过丢弃pointer edge或延迟所有timer破坏输入/弹窗语义。

## 修复结果与回传

2026-08-31 route子项源码候选：pointer/navigation handler context改为借用event-lifetime route，dispatcher只累计effect state，遍历完成后将同一route移动进公开result；surface后处理统一读取`result.route`。候选遍历继续借用stacked/bubbled/root slice，没有引入`Arc`或第二份route authority。focused源码合同与确定性压力模型8/8通过；`rustfmt --check`和scoped `git diff --check`通过。v2模型同时绑定HEAD基线dispatcher与当前worktree，覆盖1/10/100 depth x 每node/phase 1/4 callbacks，并把handler-bearing node/phase数独立建模；100万事件、depth=100、一个handler-bearing node/phase时，HEAD pointer总节点payload复制下界为49.28亿字节、route clone 200万次、candidate Vec copy 100万次、非空Vec分配下界1100万次，navigation对应为24亿字节、200万次、100万次、300万次；候选route/candidate复制均为0。visited insert模型仍为1亿次，明确没有被本轮消除。artifact：`E:\zircon-profiles\runtime-ui-dispatch-route-sharing-pressure-20260831-r1.json`，SHA-256 `0F307A4032843AE6EF316EC9EE6D2514ECB38575DA4C88AD0937EFA359389C75`，source manifest `188B0D49B891C621DA3A1D696F00242F9AA3668607D210071B8BD48944371AC2`。该模型不是implementation counter、CPU、allocator、RSS或产品时延证据，不能替代后续受管动态验收。

同日 construction-path 子项继续收敛：`UiHitPath` 只保存root-to-leaf规范序列，`UiPointerRoute`普通命中直接复用，capture/redirect才拥有独立routing path；bubble/tunnel按方向迭代，serde继续输出旧数组形状。结构合同与ownership模型合计10/10通过；100万事件、depth=100时普通命中node identity写入下界3亿->1亿、路径Vec分配300万->100万，capture写入3亿->2亿。artifact：`E:\zircon-profiles\runtime-ui-pointer-path-authority-pressure-20260831-r1.json`，SHA-256 `C485C250A5B5F4826FF12A9B417F86811FEBBA03AE43EEF21F5D9166ACAE47A9`，source manifest `93B2F73D10367F386D16C3893DDBD1436AB16A8DF122D6B9F5F75B3C33799489`。这仍是确定性下界模型，dynamic allocation counter、CPU p95和产品trace未执行。

2026-08-31 support-first compile repair：Frameworks01 managed job `c373ffe7a3164d06bd9eaabb1f75086b` 在进入 `zr_resource` 前编译 `zircon_runtime_interface` 失败；报告 blob `a75a6782f5b3baaca2246b19092a132ae3240039647121c929765e2faa26c18d` 已把 `UiPointerRoute` 改为手写 serde，却仍把 derive 宏专用的 `#[serde(default)]` 留在外层字段。当前完整 route-authority blob `9685b7a29c88c056e16fce8fba1b4e0c7ef22fa73557921aa14fac16fdf11716` 保持手写 wire 格式并把全部六个 backward-default 属性放到内部 `WirePointerRoute`；外层 `UiPointerRoute` 不再携字段 helper attribute。`test_runtime_ui_dispatch_route_sharing_performance_contract.py` 新增投影边界守卫，focused 静态批次 7/7 通过。Session `root-editorui01-pointer-route-serde-failure-r1-20260831` 已接管完整当前 blob；managed lower/interface + original `zr_resource` upward batch 使用 request `81a884f4e1e5442bbdc477fc050efb3f` 异步提交，终端 Cargo 结果尚未声明。

Open state: `route源码候选已完成；仍等待current-source Cargo与产品input trace验证capture/preview/direct/bubble/passthrough语义，并等待unified deadline timer、真实clone/alloc counter与CPU p95`。
