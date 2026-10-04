---
related_code:
  - zircon_runtime/src/ui/tests/asset_mui_web_mui_x_style.rs
  - zircon_runtime/src/ui/tests/asset_mui_web_mui_x_style
  - zircon_runtime/src/ui/tests/asset_mui_web_navigation_style.rs
  - zircon_runtime/src/ui/tests/asset_mui_web_style.rs
  - zircon_runtime/src/ui/tests/asset_mui_web_style
  - zircon_runtime/src/ui/tests/asset_mui_web_surface_child_style.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
tests:
  - eighteen tests reviewed across fourteen tracked Rust files
  - two existing template hot-path source guards identified
  - current-source Cargo, MUI scale counters and F4 product trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI MUI Web扩展样式测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读MUI X root与5个子模块、navigation、MUI Web root与5个子模块、surface-child共14/14个tracked Rust文件，合计3,786行/18测试。覆盖DataGrid/tree/pickers/charts/agent-chat、tabs/stepper/transfer-list、sx/state/slots、feedback/data-display/surfaces，以及accordion/drawer/speed-dial等utility class和style匹配语义。

## PERF-MVP-275/276/307：大常量fixture仍是小规模单次compile

除MUI X按5个测试重复执行外，各语义场景都只解析并编译小树一次。MUI X的5个子测试各自重复解析同一style/layout并完整compile，属于测试吞吐冗余；为保持产品代码优先，本轮不以共享fixture缓存挤占产品修复。MUI Web root已有2项源码守卫锁定borrowed matched rules和borrowed slot owner attributes，但源码形状守卫不能替代selector parse/probe/allocation counter。

DataGrid fixture虽然设置`rowCount=100`、overscan、viewport/requested range和scroll offset，实际rows仍只有1至2项，且测试只断言生成class；没有运行虚拟行调度、滚动更新、dirty propagation或paint。因此它不能证明100/10k/100k rows下的可视区复杂度和稳定帧预算。相关产品优化继续回链PERF-MVP-275/276/307及EditorUI04/05。

## PERF-MVP-315：动态组件状态没有频率证据

loading/streaming/open/expanded/selected/focused等状态均在静态TOML中一次性给定。没有连续chat token、chart data、picker popup、drawer swipe、accordion/speed-dial transition或tabs scroll输入，也没有记录每次状态变化的class重建、selector全扫、layout/paint dirty域和主线程积压。动态失效与调度继续回链PERF-MVP-315。

## 验收要求

加入100/10k/100k DataGrid rows与连续scroll、100/10k tree/chat/chart items、连续picker/drawer/accordion transition场景；记录generated class/property count、selector parses/probes、tree/visible-row visits、allocation bytes、dirty domains、main-thread queue depth及compile/update/layout/paint p50/p95。MUI X公共fixture只允许每测试进程解析/compile一次或由基准明确计费；当前源码Cargo与F4产品trace完成前，14文件留在`pending.md`。
