---
handoff_kind: failure
status: open
failure_scope: local
created_at: 2026-09-13
summary_slug: runtime15-validation-manifest-source-closure
origin_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
fixing_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
origin_child_dir: docs/plans/zircon_runtime/runtime/15
fixing_child_dir: docs/plans/zircon_runtime/runtime/15
plan_link_mode: child_record_only
related_code:
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_naming_boundary.py
  - tools/tests/test_runtime_init_level_naming.py
  - zircon_runtime/src/core/runtime/lifecycle.rs
---

# runtime15-validation-manifest-source-closure: 验证失败回写

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 来源执行切片：Runtime15 managed ticket d24a7d06efd241b38598dfd9e762a2f3 failed after the six exact tests because the immutable validation copy omitted zircon_runtime/src/core/runtime/lifecycle.rs and the audit's broader zircon_runtime/src tree; preserve the naming source repair and correct only dependencyRoots for the next ticket.
- 修复责任计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 交接原因：同一编号计划拥有已集成快照及其前向修复。

## 失败现象与复现证据

- 验证回写：`Runtime15 managed ticket d24a7d06efd241b38598dfd9e762a2f3 failed after the six exact tests because the immutable validation copy omitted zircon_runtime/src/core/runtime/lifecycle.rs and the audit's broader zircon_runtime/src tree; preserve the naming source repair and correct only dependencyRoots for the next ticket.` — Managed ticket d24a7d06efd241b38598dfd9e762a2f3: python -B -m unittest tools.tests.test_runtime_init_level_naming -v; five tests passed and test_service_layer_does_not_use_network_server_vocabulary raised FileNotFoundError for zircon_runtime/src/core/runtime/lifecycle.rs in F:\\cargo-targets\\verify\\89fdd9ab295c4747801eaebde59d5c42\\source.

## 最低共享层根因

The non-Cargo validation ticket declared dependencyRoots that omitted the shared zircon_runtime/src tree, so immutable-copy materialization could not satisfy the test's direct lifecycle/module reads and full naming audit scan.

## 架构修复验收

- A fresh managed Windows-native Python ticket declares zircon_runtime/src as a dependency root and the exact six RuntimeInitLevelNamingTests execute with zero errors.
- The original naming-boundary source repair remains unchanged; no production bytes are changed for this validation-scope correction.

## 禁止临时方案

- 不回滚已集成快照来掩盖普通测试失败；应通过前向修复返回 `fixed-*` 记录。
- 不得添加别名、兼容垫片、静默回退、测试旁路或调用点特例。

## 修复结果与回传

Open state: `待修复`; the coordinator must keep the validation ticket and route this Plan to repair work.

## 2026-09-19 coordinator dependency receipt

- The successor managed ticket `efa32765b6ab4644ba02362c11044741` is queued with the corrected immutable source roots (including `zircon_runtime/src`) and the exact command `python -B -m unittest tools.tests.test_runtime_init_level_naming -v`.
- Coordinator admission currently reports `validation_dependency_failed`; no Cargo/Python execution result exists for this ticket and no dynamic pass is claimed.
- The blocking lower-layer failure chain is: `input-manager-bound-text-owner-budget`, `camera-table-render-extract-stale-map`, `render-graph-rust-2021-let-chain`, `runtime-package-asset-root-projection`, `postprocess-plugin-legacy-pass-order-drift`, and `kira-send-frame-capture-routing`.
- This receipt preserves the queued ticket and source-closure evidence while the failure remains open. Focused validation, independent Critical/Important/Moderate review, canonical `fixed-*`/return records, and coordinator closeout remain pending until the dependency chain is resolved.

## 2026-10-03 独立 Windows 验证进度（仍 open）

### 现行执行方式与验收范围

用户已授权按现行 [验证政策](../../../milestone-validation-policy.md) 使用独立 Windows 验证、精确范围提交和按提交 SHA 去重通知，保持不 push。[协调器退役记录](../../../../cli-and-tooling/coordinator-retirement.md)适用。上面的 d24a7d/efa327 历史票据、命令、源码根因及依赖链原样保留；不重放票据，也不修改原票据的源码清单。

本项的完整验收仍要求三个原 Python 输入与完整 `zircon_runtime/src` 的精确不可变闭包，以及原六项 `RuntimeInitLevelNamingTests` 实际执行、零失败、零错误、零跳过。原命名修复不变。本项复制依赖修复与 Rust 下层链、Runtime15 里程碑、Jenkins 验收分别记录。

### 完整闭包捕获及启动前阻断

- 原生 Windows 捕获完成：9,467 个文件，其中 9,350 个 Rust 文件；全量 `zircon_runtime/src` 与三个 Python 输入的 Main 前后清单、复制字节均匹配。原副本保留在 `D:\cargo-targets\zircon-local\windows\failure-roll-20261003-runtime15-python6-capture-r1`，捕获终态 `terminal.json` SHA256 为 `7604e676f4a1eef57a7ab0f1e44532879f0709e996155ab306b59459591ccad6`。
- Root 完整六项运行器 `run-r3.py`（SHA256 `93f4f5f8f49c2668381e2e725675b17c77152b928a27ec44f25d50506e5cd1d1`）实际 exec `65370` 退出 1。Main 相对捕获输入新增 5 文件、变化 23 文件，输入守卫在首个子进程启动前终止；实际子进程数和测试数均为 0。
- 五个新增文件涉及 Core runtime shutdown、activation closure atomicity、retired service drop、module lifecycle shutdown 及 Dynamic API shutdown consumer。当前作者、接收路由和自然边界尚未建立；保留全部外来 postimage。
- 该次 Main 前后读取相等，但已不同于捕获时 Main。冻结复制清单仍与原复制清单逐值相等。原始运行终态的 `copyInputsUnchanged:false` 来自前置复制清单读取尚未执行，不能解释为冻结副本变化。
- 启动前回收证据：`.codex/tmp/failure-roll-20261003-runtime15-python6-root-runner-r1/prechild-drift-terminal-reception.json`，SHA256 `d8515a3c27e024ea2081af14a057cc10c363ec7f9e5abbb0d56dd1abdf59bcae`。独立复核 `.codex/tmp/failure-roll-20261003-runtime15-prechild-drift-independent-review-r1/compactreview.json`，SHA256 `04deafba315e68080792ab2b17aa4f52b83747c9614641fd7ffbe2283d9f84cd`，Critical/Important/Moderate 为 0/0/0。

### 已通过的下层三项回归

Root 使用精确三个 Python 输入的独立 D 盘复制与私有夹具，实际执行现有用例：

1. `test_editor_owner_classifications_do_not_hide_broader_production_paths`
2. `test_legacy_cfg_test_items_do_not_hide_production_naming_debt`
3. `test_legacy_scan_applies_cfg_test_boundary_per_item`

实际命令入口为 `python -u -B -S E:\Git\ZirconEngine\.codex\tmp\failure-roll-20261003-runtime15-lower3-root-runner-r1\run.py`。Root exec `78430` 实际退出 0，三项用例实际执行，3/3 通过，失败、错误、跳过均为 0。原始日志和子进程证据在 `D:\cargo-targets\zircon-local\windows\failure-roll-20261003-runtime15-lower3-r1`；单个子进程 PID 3988、创建 FILETIME `134354924706283897`、实际退出 0，pending/unknown 均为 0。

Main 三个 Python 输入、复制输入及工具身份在前后检查中相等；导入来自精确复制路径，夹具位于物理 D 盘目录。复制目录仅含三个 Python 文件，不包含 Runtime 源码。输入 SHA256：

- `tools/tests/test_runtime_init_level_naming.py`：`ed14ab0e8d4673205af6135c6e874868702f276f0b83b275aff3e065dcb798bd`
- `runtime_structure_audits/__init__.py`：`8816d686dcea3b4861ec6252b11619bf80dea170b67803b2dda641b0026da440`
- `runtime_structure_audits/runtime_naming_boundary.py`：`413fc2e2f98349cbe75419c24a284451d257bb99550f8908df560f47702fd03a`

下层回收 `.codex/tmp/failure-roll-20261003-runtime15-lower3-actual-terminal-reception-r1/terminal.json`，SHA256 `20ad6b8563a981c25f6bb05bc5da267fbefd005d19c52cf783bd5ca46db043b7`，14 项证据检查通过。独立复核 `.codex/tmp/failure-roll-20261003-runtime15-lower3-terminal-independent-review-r1/compactreview.json`，SHA256 `e417882737e3b34667e17b1aac03345de8d40b0eee054ca3a0c4f7169b8c17e5`，Critical/Important/Moderate 为 0/0/0。

### 尚未完成的验收与下一步

完整六项批次保持挂起，本 failure 保持 `open`。下层 3/3 不构成源闭包修复、当前 Runtime 行为、Rust 下层链或 Runtime15 里程碑验收。取得当前 Core/Dynamic API 实际 writer 的精确 postimage 和自然边界后，重新核对完整输入并冻结新闭包，再执行原六项；保存上述捕获和阻断历史。未修改生产源码、建立 `fixed-*`、提交或发送通知。
