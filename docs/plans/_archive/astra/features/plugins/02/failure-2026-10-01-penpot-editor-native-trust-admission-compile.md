---
handoff_kind: failure
status: open
created_at: 2026-10-01
summary_slug: penpot-editor-native-trust-admission-compile
origin_plan: docs/plans/designment/02-milestone-execution-and-evidence.md
fixing_plan: docs/plans/astra/features/plugins/02-native-trust-admission.md
origin_child_dir: docs/plans/designment/02
fixing_child_dir: docs/plans/astra/features/plugins/02
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/package_service/admission/policy_index.rs
  - zircon_runtime/src/plugin/package_service/admission/project_resolution.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/loading.rs
  - zircon_runtime/src/plugin/package_service/mod.rs
tests:
  - tools/build-editor.ps1 -OutputDirectory D:/cargo-targets/penpot-editor-20260930-01a0f04d-6fc0351c
---

# Penpot 主工作台引擎验证：penpot-editor-native-trust-admission-compile

## 来源执行者

- 来源计划：`docs/plans/designment/02-milestone-execution-and-evidence.md`
- 来源执行切片：当前 App Editor / Runtime DLL 产品构建与真实双端工作台截图。
- 修复责任计划：`docs/plans/astra/features/plugins/02-native-trust-admission.md`
- 协调 session：`astra-plugin-a3-trust-20260930-74fc2c19`。
- 交接原因：错误在最低共享运行时 owner，来源 UI 计划不覆盖其候选修改。

## 失败现象与复现证据

- 托管 Windows 产品入口 `tools/build-editor.ps1` 通过 `validate-matrix.ps1` 启动 job `ff63696a9e12459cb4c2fe0171bb6c0c`；源码 input manifest `b60bab36ef66a0e1b80228fe5ae8194c9828f8ed32fe657f13455f5ca2a939fe`。job 已 release，产品未发布，Cargo exit 101 / wrapper exit 1。
- 完整输出：`D:\cargo-targets\mvp-test-fixtures-45460\penpot-workbench-6fc0351c5c1e43178d5e205f123e139f\tmp\editor-build-r2\outer-stderr.log`；SHA-256 `36889d0508cf2d19c79e55e59623c1dd2f712c60644854359e7453e9145ab6f0`。该完整外层日志属于此 job；pool 内 earlier-check diagnostics 不属于该失败构建。
- 18 条总错误中，本 fixing scope 占 14 条。2026-10-01 只读核对确认 affected primary source 的当前 checkout、sealed source 和 manifest hashes 一致，失败仍适用；related tests 没有被本轮执行。
- UI timer scope 在交接时仍由原 session live lease 保护；plugin 和 graph 原 sessions 分别 registered / resolving_failure，但缺 live lease，修复者应先重新取得精确 scope 并保留来源 attribution。来源 session 未接管这些源文件。

- `zircon_runtime/src/plugin/package_service/admission/policy_index.rs`: `9c1939101ce20e392f071d5615db5f03a3a71609b86759e0720225ff6ac8c6a6`
- `zircon_runtime/src/plugin/package_service/admission/project_resolution.rs`: `d559d8798516afe1d611494e703269b6d6fe3eb8a1250cd6b6f5395385af55ed`
- `zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/loading.rs`: `e959aa1bdd1ea11798979caeea5aa089e2b4ab4010918a3c4557cacc8c93a293`
- `zircon_runtime/src/plugin/package_service/mod.rs`: `271e1c26db9d008600cc5282716717826a501893ecc2cced9b40cf4bd5a081f9`

## 最低共享层根因

Policy index 序列化合同、host policy 的 Debug 合同、digest helper 可见域和 policy accessor 未在同一 owner 边界内对齐；roots 被推断成无所有权的 Path；staged plugin 计数保留 &str 借用却随后消费其 owner vector。

## 架构修复验收

- 输入专用 policy-index document 只需 Deserialize；持久化 installed-selection document 需 Serialize。沿用 ancestor is_digest 与 loaded.policy()。roots 显式 Vec<PathBuf>。Debug 必须保留 authority 数据的暴露边界。staged counts 使用拥有所有权的 plugin ID key，保留重复 ID / 注册失败前的拒绝语义。
- 保留并执行 digest rejection、context roots、installed selection persistence/upsert 及 native staged duplicate-ID/registration failure 的已有或新增聚焦回归。
- 修复 owner 验证最低层后，来源计划重跑同一托管产品构建；通过仍须取得正常 Editor 与 native capture 的实际 PNG / source、token、font、media、locale receipts。编译过关不等于画面一致。

## 禁止临时方案

- 不添加兼容 shim、复制 authority、候选自证、unsafe 借用绕过或单调用点成功旁路。
- 不删除或弱化测试、trust / allocation / timer 验证及产品验收门。
- 不把 pending receipt、静态 mock 或旧产品截图标为 passed / accepted。

## 修复结果与回传

待修复，状态保持 `open`。来源 Penpot 导入诊断继续推进；实际引擎产品门未完成。

## 原会话修复提案与路由

- 2026-10-01 用户选择由 PLUGIN-A3 原会话应用提案并回传验证。来源会话不接管三份 trust 源文件；该故障保持 `open`。
- 已确认原子会话 `01a0f039-f39a-7021-b6fd-b539a5bad1bf`（`/root/plugins_vm` / Archimedes）及父会话 `01a0f033-ce80-77e2-86f0-eed5bfbb5926`。原 rollout 有 `astra-plugin-a3-trust-20260930-74fc2c19` 的实际 register 命令；父会话也有该 scope 的历史租约/归属操作。
- 原始三文件最小提案独立审查未发现 P1/P2，三个 candidate 的 rustfmt 语法检查 exit 0，尚无 Cargo 通过证据。提案在 `D:/cargo-targets/mvp-test-fixtures-23160/penpot-workbench-f442f16d73bc4e7dae78954f60db1393/foreign-trust-compile-repair-draft-r2/candidate.patch`；相邻 `candidate/` 和 `receipt.json` 保存原文与预映像指纹。receipt 中旧 fixture 地址属于历史记录，当前提案地址以本节为准。
- 本次重核三预映像：policy_index `9c1939101ce20e392f071d5615db5f03a3a71609b86759e0720225ff6ac8c6a6`；project_resolution `d559d8798516afe1d611494e703269b6d6fe3eb8a1250cd6b6f5395385af55ed`；loading `798660cd63eaf80681bcbf17afdc20aa0d71c37e86cdb00da494c44573d61e48`。loading 与原失败快照不同，修复者应在正常租约内保留现有修改并按当前预映像审查提案。
- 定向 `codex exec resume` 子会话终态 exit 1，要求先加载 parent；定向恢复 parent 终态 exit 1，报 `already has an active writer`。未强行关闭现有 writer 或修改会话存储。命令/终态日志分别保存在当前 fixture 的 `tmp/plugin-a3-original-session-handoff-r1` 与 `tmp/plugin-a3-original-parent-handoff-r1`。这两次失败没有送达执行者，不能当作已应用或已验证。
- 请原 PLUGIN-A3 在下一修复窗口领取自己的三文件，应用/合并提案，按本记录的最低层及产品门验收回传实际终态和最终 hashes；不得用 pending receipt 或语法检查替代编译结果。

## 原会话遗失后的正式接管

- 用户随后确认 PLUGIN-A3 原会话已丢失，原会话回传路线不再可执行。只读核对原 owner 无 live leases、排队 patches、Cargo jobs、lane reservations 或 validation copies；通过正常 `session.set_status` 记录 `stale` 和用户原因，没有改数据库、终止 writer 或清除锁。
- 正常 `ownership.transfer.preview/apply` 只转移本记录三个精确 Rust 文件，确认指纹 `5536330665ba6d2e541fdc7449db1c9312e4c994d370482edf86457096a1a246`。原完整字节已保存，只应用先前已审查三文件提案；归属与 write scope 由协调器原子更新到来源 Penpot session。
- 已应用 receipt：`D:/cargo-targets/mvp-test-fixtures-23160/penpot-workbench-f442f16d73bc4e7dae78954f60db1393/tmp/lost-plugin-owner-transfer-apply-r1/receipt.json`。最终 policy_index SHA-256 `9cd1cafcbc1a553719f743d1a57277de4bd7ec061f92988f3dc29cc5cad73ecf`；project_resolution `4e950121e6ffaf8638e7480f51659ee8bba1ffdc38282e01997d2f53196036b4`；loading `4e6877d0f12da8440a3835bdcf080d12f4f010a0f9c5285e0b00016dc2683352`。
- 托管产品重试 `tools/build-editor.ps1 -OutputDirectory D:/cargo-targets/penpot-editor-20261001-lost-owner-01a0f04d-r2` 终态 wrapper exit 1。请求 `fdef64fca5d3446b849ff701376e5b7c` 在 `cargo.acquire` 失败，`waiting_disk_space` 的具体原因是 `storage_snapshot_unavailable`；没有分配 Cargo job 或执行 Cargo。D/F 实际可用空间充足，库存快照 `scanComplete:false`，因此不能把该结果描述成编译仍失败或磁盘已满。
- 该提案的 Cargo、聚焦回归、产品 bundle 和真实画面验证仍待正常库存/准入恢复后执行。本 handoff 保持 `open`，没有 `fixed`/`accepted` 回传；协调器实际验证门完整保留。

## 退役后的实际本地验证（2026-10-02）

- 本文上方的协调器、租约和库存准入操作属于历史证据。协调器已退役；后续使用仓库当前独立验证入口，保留历史数据库、票据、原生 Cargo 锁及原文件归属。见 [退役与恢复](../../../../../cli-and-tooling/coordinator-retirement.md)。
- 独立 Editor 检查实际进入 Runtime 编译，发现 `project_resolution.rs` 的重复选择计数仍借用 `selected` 内的字符串，随后消费该列表触发 E0505。已在接管的原 scope 内把 `entry(selection.id.as_str())` 改为 `entry(selection.id.clone())`；查询与拒绝语义不变。最小候选与最终 rustfmt 排版经独立审查无 P1/P2。
- 当前 `project_resolution.rs` SHA-256 为 `2632949f4e2d87729bc546bfeaaeb8dde017367e0a3147a7df74c7d1879758e0`；另外两份接管源文件 SHA-256 保持上一节值。字节与差异保存在 `D:/cargo-targets/zircon-local/penpot-workbench/01a0f04d-20261002/tmp/plugin-selection-borrow-repair-r1` 和相邻 `plugin-selection-borrow-repair-format-r2`。
- r3 实际命令为 `python -u -X utf8 -B -m tools.local_cargo --target-dir D:/cargo-targets/zircon-local/penpot-workbench/01a0f04d-20261002/targets/editor-native-check-r1 -- check -p zircon_editor --lib --no-default-features --locked --jobs 4`，Windows/MSVC。Runtime 借用错误消失，编译进入 Editor；最终 Cargo exit 101，Editor 报 36 条编译错误。该批已封存的源码在命令期间未变化；不能称整个 Editor 编译通过。
- 实际终态：`D:/cargo-targets/zircon-local/penpot-workbench/01a0f04d-20261002/tmp/workbench-managed-scene-local-rust-checks-r3/receipt.json`；完整 stderr 位于同目录 `check-stderr.log`，SHA-256 `839a93f625406e022a25da5f44f9fc70c2a1b4164001b8ee31155e5fad36b817`。该回执为本地命令结果，未授予 Jenkins、里程碑或画面验收。
- 本 handoff 继续 `open`。剩余门是相关 plugin 聚焦回归、新鲜正常 Editor/Runtime 产品 bundle，以及正常窗口和 native capture 的实际双端画面验证；没有 `fixed` 或 `accepted` 回传。
