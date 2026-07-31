---
related_code:
  - zircon_runtime/src/graphics/runtime/offline_bake
tests:
  - current offline-bake slice 5 of 5 Rust files reviewed, 116 lines
  - all 1 probe-count boundary test read; zero-work/capacity regression added
  - zero-budget/empty-scene and Vec growth gates changed from RED to GREEN
  - scoped rustfmt, source contracts and diff check passed
  - current-source Cargo and editor-manual bake scale trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics offline bake静态审查（2026-07-18）

## 当前源覆盖

`graphics/runtime/offline_bake/**`当前5/5个Rust文件、116行已逐文件静态阅读，1条新增边界测试已读。覆盖settings/default、directional-light intensity汇总、mesh→manual reflection probe投影、shape/intensity validation、output DTO与模块出口。

## 直接止损

原实现即使probe预算为0或scene无mesh也先遍历全部directional lights，并从空`Vec`逐个push已知上限的probe。现于light扫描前对zero budget/empty mesh早退；`eligible_reflection_probe_count`统一处理无光照、NaN、mesh count与max budget clamp；有效路径用`Vec::with_capacity(probe_count)`，避免增长重分配。probe顺序、radius、intensity、invalid shape跳过及`EditorManual` timing语义保持。边界测试先RED后GREEN。

## 剩余根因

该函数是同步CPU editor-manual helper，默认最多4 probes，不在每帧MVP热路径。若未来把预算扩到1k/100k或加入真实cubemap/GPU bake，Editor14必须以可取消job/timeslice执行，Render11负责dirty probe generation与增量artifact，不能在UI/render state锁内整批运行。当前不为低预算纯投影另建P0任务。

本地Unreal reflection environment capture对runtime capture提供face count/time slicing与render-command owner；采用“大规模bake可预算、可取消、按generation提交”的原则，不复制其capture/RHI实现。

## 验收状态

静态、boundary RED→GREEN、rustfmt、source contract与diff门禁完成。Windows Cargo validator仍在启动前`ConvertFrom-Json`失败，1条测试没有current-source结果。meshes/lights/probe budget 0/1/4/1k/100k的visits、alloc/growth、job wall/cancel与editor responsiveness未完成，继续留在`pending.md`，不进入`review.md`。
