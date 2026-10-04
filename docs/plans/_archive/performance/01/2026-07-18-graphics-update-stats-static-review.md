---
related_code:
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats
tests:
  - current update_stats slice 12 of 12 Rust files reviewed, 2165 lines
  - all 27 tests read; 5 coverage/aggregation/storage/index regressions added
  - pass/executor/visibility/UI/VG repeated scans and stable String allocation gates changed from RED to GREEN
  - scoped rustfmt, source contracts and diff check passed
  - current-source Cargo, F2 diagnostics trace and RenderDoc capture pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics update_stats静态审查（2026-07-18）

## 当前源覆盖

`update_stats/**`当前12/12个Rust文件、2,165行已逐文件静态阅读，27条测试均已读。覆盖base graph/visibility/HZB/post/UI/mesh/sprite/light统计，Hybrid GI、Virtual Geometry、particle、quality profile、light grid与shared-product reports，以及feature-off reset路径。

## 直接止损

graph execution coverage原先把planned/executed pass名全部clone进`BTreeSet<String>`再排序求交差。本轮改为`HashSet<&str>`借用现有名称，保留planned/executed/matched/missing/unexpected/duplicate语义，名称clone和排序归零。executor family计数原先为AA/HZB/VG/GI/particle/shadow/sprite执行8次全slice扫描，现由单次`ExecutorPassCounts`聚合；visibility五项计数从5次views扫描改为1次。当前post diagnostics也已把resource/executor多次`any`收敛为各1次遍历。

UI pass order从3次`position`改为一次first-wins扫描，并返回static标签；quality profile、UI order与post output-transfer三类稳定String通过`update_optional_stat_string`复用既有容量，stable value不再分配。VG visible entity、resident/requested/seen/page集合由`BTreeSet`改为`HashSet`；hierarchy traversal原先每个work item线性`find` node，现每次stats构建一次first-wins node-id索引，查找由O(work×nodes)降为均摊O(work+nodes)。新增借用coverage、executor聚合、String storage、page membership及hierarchy index五组回归先RED后GREEN。

## 剩余根因

`update_stats`仍在framework state锁内无条件物化完整`RenderStats`：每帧复制executed passes、executor ids、debug markers、post nodes、advanced provider/alias/Solari大报告，并从renderer/context重新扫描visibility、lights、post graph与VG。VG stats仍在CPU重放完整node/cluster traversal及execution page分类，和PERF-MVP-416的debug snapshot/真实render report存在多owner；feature-off reset也逐字段写大量冷诊断状态。即使没有stats/debug/editor订阅，上述观察者成本仍进入terminal camera提交关键路径。

新增`PERF-MVP-418`：Render10/01让各render pass/prepare owner把确定性计数写入generation-owned `SealedRenderFrameDiagnostics`，renderer outcome只封存一次Arc，不在framework层逆向扫描重建。Runtime07在PERF-MVP-411 Phase C只发布Arc+少量always-on health counters；详细graph/VG/UI/provider report按subscription/capture启用并lazy materialize，query-if-newer短锁clone Arc，历史有界。Editor07以generation subscription读取summary/detail，不按UI tick深clone；Render17单独量化observer overhead。VG execution/traversal继续复用PERF-MVP-416唯一报告。

本地Bevy `Diagnostics::add_measurement`用闭包延迟昂贵值计算，并在diagnostic disabled时不求值；本地Unreal Engine用静态`DECLARE_GPU_STAT`与RDG event scope让pass owner产出计数/时间。采用“producer owner计数、disabled不求值、consumer按generation读取”的原则，不复制它们的ECS或RHI宏体系。

## 验收状态

静态、五组RED→GREEN、rustfmt、source contract与diff门禁完成。Windows Cargo validator仍在启动前`ConvertFrom-Json`失败，27条测试没有current-source结果；RenderDoc CLI不可用且无capture。diagnostics off/on、passes/executors/views/lights 0/16/1k/100k、VG nodes/work 0/1k/1M、UI poll 1/60/240 Hz的scan/clone/alloc/state-lock/CPU p95与GPU timestamp证据未完成，继续留在`pending.md`，不进入`review.md`。
