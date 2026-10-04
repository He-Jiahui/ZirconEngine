---
handoff_kind: failure
status: open
created_at: 2026-09-08
summary_slug: template-receipt-roundtrip-type-inference
origin_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
fixing_plan: docs/plans/optimize/zircon_app/07-renderable-empty-project-template-create-import-render-export-evidence-product-integration-review.md
origin_child_dir: docs/plans/optimize/zircon_runtime_interface/03
fixing_child_dir: docs/plans/optimize/zircon_app/07
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/project/template_pack/content_digest.rs
  - zircon_runtime_interface/src/project/tests/template_pack.rs
tests:
  - cargo test -p zircon_runtime_interface --locked --no-default-features --lib template_pack
  - cargo test -p zircon_runtime_interface --locked --no-default-features --lib serialization::tests::
  - cargo test -p zircon_runtime_interface --locked --no-default-features --lib
---

# App07: template receipt round-trip tests require canonical decode types

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md`
- 来源执行切片：共享接口库测试编译解阻及 UI 契约回归。
- 修复责任计划：`docs/plans/optimize/zircon_app/07-renderable-empty-project-template-create-import-render-export-evidence-product-integration-review.md`
- 交接原因：模板 descriptor 与 content digest 的往返测试由 App07 拥有；编译错误会阻断
  同一库内的 UI、Editor08 command contract 及 Editor11 serialization 测试。

## 失败现象与复现证据

Windows 受管作业 `c34098ad9f3740918f1415b061977924` 在 locked、static、
no-default-features 配置执行 `serialization::tests::`，产生 3 个编译错误、0 个实际测试。
其中本计划拥有 2 个 E0283：`content_digest.rs` 的 digest round-trip 与
`tests/template_pack.rs` 的 descriptor round-trip 无法推断 `serde_json::from_str` 的目标类型。
第三个 Editor08 命令契约导入错误由独立源码快照 `3134` 处理。

原始受管输入：`E:/cargo-targets/zircon-engine/cache/build-benchmarks/interface-library-consumers-3128-20260908`。
manifest SHA-256：`7b82a7b3c9f8b9d93ce1e184e44c984a5e2e8cae1b1cd72fa7331d555d69706b`。
原始证据保留在该输入的 `results/interface-library-3128.json` 和 `.log`。

## 最低共享层根因

`serde_json::from_str` 的泛型参数需要 `Deserialize` 目标；后续 `assert_eq!` 的
`PartialEq` 约束不能唯一决定该类型。测试应明确解码为实际 wire contract 的
`ProjectTemplateContentDigest` 和 `ProjectTemplateDescriptor`。

## 架构修复验收

- digest 与 descriptor 的原有往返测试实际执行并通过；保持 lowercase hex、完整字段、
  canonical registry identity、伪造 descriptor 拒绝及模板内容身份断言。
- 原始 serialization 测试过滤器实际执行且通过。
- 全部接口库测试通过，包括直接 UI 与 editor contribution 消费者；ignored 性能测试
  不替代其他 failure 原文要求的 release 性能验收。
- 接管的既有模板源代码必须具备精确归属、依赖快照和独立审查，才可进入 closeout。

## 禁止临时方案

- 不解码为 `serde_json::Value` 以绕过规范 `Deserialize`，不删除或 ignore 测试。
- 不放宽 digest、descriptor、capability、entry 或 version 验证。
- 不将旧归档 attribution 的不同哈希当作当前源码证据，不将 0 个测试的编译回执算作通过。

## 修复结果与回传

Open state: `source-repaired_managed-validation-pending`。

已完整读取两个现行文件及 descriptor 生产反序列化调用链。两文件原 owner
`root-app07-create-admission-r1-20260901` 已 archived；当前哈希与旧 attribution 不同。
本轮通过审核可接管的精确路径预览 `8db1a8cea35c4d54ae98ac9f2058baee`，
使用 transfer `f1c003ea1a5e494b9fb8c0f3a9e7848b` 接管当前字节，并保留前置快照 `3136`。
该接管保存现存模板实现与断言，不声称这些既有修改由本轮编写。

本轮只增加两个明确目标类型并按仓库格式规范格式化，源码快照 `3137`：

- `content_digest.rs`：`9bcd748dbfd014dd6f895444ab5eb00c8f8a1a300b48d356877ef2d972539119`。
- `tests/template_pack.rs`：`ba0b5c34326ad9c927929c5c5237284aac20ac93eee4b4e47a09b206c77a3bcf`。

新受管输入 `interface-library-consumers-3137-20260908` 已从上述原始输入派生，
仅叠加各自 owner 的 `3134`、`3137` 源码快照；manifest SHA-256 为
`484b58e5512bb5619941864b4906bbfaaeef0cff78f89267586fe6e4b00d2b63`，完整验证了 10,956 个输入文件。
动态验收、独立审查、正式 fixing Session 验证票据及 failure return/closeout 尚未完成。
本条保持 open；App07 项目创建、渲染、导出产品资格也不因修复两处测试编译错误而完成。

## 2026-09-08 实际动态结果

受管 Windows 作业 `f95f64a6a06345d3940884140d9e3e50` 在上述 `3137` 派生输入上
执行完整 `cargo test -p zircon_runtime_interface --no-default-features --locked --lib`。
编译成功；结果为 739 passed、23 failed、101 ignored，0 filtered out。
两个 App07 目标 round-trip 测试均实际通过；`project::*template_pack::*` 共 21 passed、
0 failed、0 ignored，规范 descriptor 伪造拒绝、内容摘要和原有模板树回归均保留。
原始来源 `serialization::tests::*` 实际 72 passed、0 failed、1 ignored；唯一忽略项是
原有 512 MiB streaming 测试。Editor08 contribution 测试 7 passed。
原始输出、测试名和受管 receipt 保留在新输入的 `results/interface-library-3137.{json,log}`。
全库 23 项其他失败仍然存在，包含项目身份、旧 fixture 路径、ABI inventory、boundary
和 UI geometry/state；上述通过子集不代表全库验收通过。独立审查与正式 closeout 待完成。

2026-09-08 独立审查返回 Critical 0 / Important 0 / Moderate 0，报告
`.codex/tmp/interface-app-editor-3137-review-20260908-result.txt`。审查覆盖接管的完整 digest
实现、模板既有新增断言、3136 到 3137 的类型标注，以及 production descriptor serde。
所选当前/attribution/ObjectStore 哈希与受管输入一致，审查前后没有漂移或 reviewer ownership
冲突。源码审查已完成；全库和产品 gate、正式 fixing-Session 绑定、return/closeout 仍未完成。

## 2026-09-11 current-source continuation

The archived App07 owner was replaced by the stable fixing Session
`failure-roll-01a084c8-app07-template-roundtrip-r2`. Its exact current-source
ownership was transferred from the stale attribution
`astra-optimize-20260909-root` (transfer fingerprint
`cb07c7d4751fec833cf7c851fe34572a832561e1739b972bf9d7d57f7ec56d1a`), including
the package manifest and the complete template-pack production closure. The
formatted current source was frozen in snapshot `3463`; the owned source
manifest includes the 11 package/production/test paths and records their
current hashes.

A direct structured Cargo request
`app07-template-roundtrip-current-20260911-r1` was submitted for
`cargo +1.94.1 test -p zircon_runtime_interface --no-default-features --locked
--lib project::tests::template_pack:: -- --nocapture --test-threads=1`.
Coordinator immutable admission rejected it before materialization because the
external `E:\Git\zr_vm` worktree is dirty. No test result or pass is claimed;
the fixing Session is `waiting_validation` and must not resubmit this request
until that external state changes.

## 2026-09-19 rolling successor source reconciliation

The archived r2 fixing Session was replaced by successor
`failure-roll-01a084c8-app07-template-roundtrip-r3`. The complete current
template-pack closure (package manifest, project module, production descriptor /
digest / receipt / render files, and template-pack tests) plus this failure
record transferred under fingerprint
`2aec5e684973fa69e7b8b75d5cec6b23c52b6753f74f1e6d90fa6afcbea2d24c`.
The transfer preserves existing foreign changes and does not claim authorship
of the earlier type annotations. Current source bytes match the transfer
preview; dynamic Cargo remains blocked by the recorded external worktree
condition. A static current-source contract ticket will be submitted only for
the owned overlay, with `zircon_runtime_interface/src/project` used as the
tracked readonly dependency root so the three added template-pack files are
not incorrectly requested from `git archive`.

The static contract ticket `469d3b11462c468792bcbfa8726325c2` (request
`failure-roll-01a084c8-app07-template-roundtrip-20260919-r1`) was admitted with
source-manifest hash
`03968081a4f5802a8762ef6559bc448bcd407df172370891c2b4aade7093c995`.
It checks the explicit digest and descriptor decode types, canonical template
module wiring, and the existing round-trip assertions; its terminal marker is
`APP07_TEMPLATE_RECEIPT_ROUNDTRIP_CURRENT_SOURCE_CONTRACT_PASS`. The ticket is
queued and static-only; no dynamic Cargo, review, fixed return, or closeout is
claimed yet.

The ticket executed as managed job `f217433e257248ec953f2d5f776142c7` / run
`469d3b11462c468792bcbfa8726325c2`, exited `0`, and emitted
`APP07_TEMPLATE_RECEIPT_ROUNDTRIP_CURRENT_SOURCE_CONTRACT_PASS`; cleanup
completed. This proves the current source contract only. The focused and full
Cargo gates, fresh review binding, canonical return and closeout remain
pending behind the external worktree condition.

## 2026-09-20 independent current-source review r3

Reviewer session: `review-app07-template-roundtrip-r3`, child of
`failure-roll-01a084c8-app07-template-roundtrip-r3`. The review covered the
successor's complete template-pack closure; no source was edited or re-owned.

Result: **Critical=0 / Important=0 / Moderate=0**.

- `ProjectTemplateContentDigest` serializes as lowercase, fixed-width SHA-256
  hex and rejects uppercase, malformed, or incorrectly sized input. Its canonical
  path/byte-length ordering keeps declaration order from changing identity while
  path or payload changes do change the digest.
- `ProjectTemplateDescriptor` and its target-requirement wire types use explicit
  serde decode targets, deny unknown fields, and compare version, digest,
  engine-range, target/provider requirements, and sorted entry descriptors against
  the canonical registry descriptor. The round-trip and forged-field tests exercise
  those checks directly.
- Rendering rewrites only project name/GUID/engine requirement, validates required
  provider selections for every target, rejects duplicate/file-prefix entry paths,
  and preserves the descriptor identity. Receipt/render modules and the project
  re-export surface remain wired to the same canonical types.
- The current-source contract marker and scoped `git diff --check` passed. Local
  rustfmt reports pre-existing import/order and assertion-wrap differences in the
  broad snapshot; no formatter-only rewrite was made during this review.

Current reviewed hashes:

```text
zircon_runtime_interface/Cargo.toml
  b5e30acb07e3e73019945a9bd855b59b4d7a4ca70d9c952971a1063af5669fa4
zircon_runtime_interface/src/project/mod.rs
  e41f17e4eb326152a5f8df3e8313fa04a608d13fa04ba4a4b06b7a70926b7a4a
zircon_runtime_interface/src/project/template_pack/content_digest.rs
  a3efb1feaafa20739eaaaaf414ceafa7981998c78c742b101d5170708881eb1c
zircon_runtime_interface/src/project/template_pack/descriptor.rs
  b3256c2ff15256d94f8e01b06cd08011bdcb624512d89c1f5da6b7bf8080815e
zircon_runtime_interface/src/project/template_pack/error.rs
  1ce4f99571617b1e132a0514477b89a47d3552b72530948f0ae1c57314675e54
zircon_runtime_interface/src/project/template_pack/mod.rs
  1af8be24b3eb3cfe18d73b472e960e99017e83185b0c2090cc4f0ca4957c9b5c
zircon_runtime_interface/src/project/template_pack/project_template_id.rs
  3acce0cead7c3d0cdc8326646544a2f8f196d70983e1e243a817a828237fb95a
zircon_runtime_interface/src/project/template_pack/receipt.rs
  2b5bfa5e6be903fd33a15c8a5449e789494af5ce944a1eff4917fb939ec18df3
zircon_runtime_interface/src/project/template_pack/render.rs
  a79d7d5f073177ae8c6af1493a4b9f47e2c94f510be42344ddaf5a19d8fae062
zircon_runtime_interface/src/project/template_pack/rendered_template.rs
  42f30ecf5d84f096a089f867090ea2035f393e6171002011009dcbc9ff468791
zircon_runtime_interface/src/project/tests/template_pack.rs
  812042f9a1b2fe3542062e1bc423bc0023e084ff85929e08174f9c9917a0607e
```

The focused/full RuntimeInterface Cargo tests, App07 create/import/render/export
product evidence, UI accessibility upward gate, canonical `fixed-*` return, and
closeout remain pending. Static source evidence is not promoted to those gates.

## 2026-09-25 rolling successor r4 current-source reconciliation

The stable fixing Session `failure-roll-01a084c8-app07-template-roundtrip-r4`
was registered after the archived r3 retention window and claimed this failure
record under request `9407c09fd4244ab1878352c775a2bd70`. The current source
bytes for all eleven paths in the r3 manifest still match the reviewed hashes:

```text
zircon_runtime_interface/Cargo.toml
  b5e30acb07e3e73019945a9bd855b59b4d7a4ca70d9c952971a1063af5669fa4
zircon_runtime_interface/src/project/mod.rs
  e41f17e4eb326152a5f8df3e8313fa04a608d13fa04ba4a4b06b7a70926b7a4a
zircon_runtime_interface/src/project/template_pack/content_digest.rs
  a3efb1feaafa20739eaaaaf414ceafa7981998c78c742b101d5170708881eb1c
zircon_runtime_interface/src/project/template_pack/descriptor.rs
  b3256c2ff15256d94f8e01b06cd08011bdcb624512d89c1f5da6b7bf8080815e
zircon_runtime_interface/src/project/template_pack/error.rs
  1ce4f99571617b1e132a0514477b89a47d3552b72530948f0ae1c57314675e54
zircon_runtime_interface/src/project/template_pack/mod.rs
  1af8be24b3eb3cfe18d73b472e960e99017e83185b0c2090cc4f0ca4957c9b5c
zircon_runtime_interface/src/project/template_pack/project_template_id.rs
  3acce0cead7c3d0cdc8326646544a2f8f196d70983e1e243a817a828237fb95a
zircon_runtime_interface/src/project/template_pack/receipt.rs
  2b5bfa5e6be903fd33a15c8a5449e789494af5ce944a1eff4917fb939ec18df3
zircon_runtime_interface/src/project/template_pack/render.rs
  a79d7d5f073177ae8c6af1493a4b9f47e2c94f510be42344ddaf5a19d8fae062
zircon_runtime_interface/src/project/template_pack/rendered_template.rs
  42f30ecf5d84f096a089f867090ea2035f393e6171002011009dcbc9ff468791
zircon_runtime_interface/src/project/tests/template_pack.rs
  812042f9a1b2fe3542062e1bc423bc0023e084ff85929e08174f9c9917a0607e
```

The checkout still contains foreign dirty overlays across the
`zircon_runtime_interface` crate, including several of these paths; r4 did not
edit or absorb those bytes. The prior managed static contract ticket
`469d3b11462c468792bcbfa8726325c2` / job
`f217433e257248ec953f2d5f776142c7` and the r3 independent review
(`Critical=0 / Important=0 / Moderate=0`) therefore remain reusable only as
same-hash source evidence. The external `E:\Git\zr_vm` dirty-worktree
admission blocker still prevents a new immutable Cargo ticket. Focused/full
Cargo, product and upward gates, a fresh independent review binding, canonical
fixed return, and closeout remain pending; this lifecycle stays `open`.

### r4 independent review receipt

Reviewer Session `review-app07-template-roundtrip-r4` re-read the current
manifest and source on 2026-09-25. All eleven hashes and ownership states match
the manifest; the eight tracked modifications and three untracked template-pack
files remain foreign/archived-owner bytes and were not edited. Explicit
`ProjectTemplateContentDigest`/`ProjectTemplateDescriptor` decode targets,
round-trip assertions, and forged-descriptor checks are present. The declared
`template_pack`, `serialization::tests::`, and full-library filters are valid.
The static marker is scoped to source contract only, and the historical
739/23/101 Cargo result is not promoted to full acceptance. Independent review
result: **Critical=0 / Important=0 / Moderate=0**. External `E:\Git\zr_vm`
admission, focused/full Cargo, product/upward gates, fixed return, and closeout
remain pending.
