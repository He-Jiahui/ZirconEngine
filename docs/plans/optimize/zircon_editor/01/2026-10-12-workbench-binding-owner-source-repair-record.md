---
doc_type: source-adoption-record
title: Editor Workbench binding owner source repair
date: 2026-10-12
task_id: runtime-editor-optimize
source_status: adopted_source_only
native_status: pending
performance_status: pending
---

# Editor Workbench binding-owner Source 修复记录

这项 Source 修复验证当前 Workbench control 是否拥有拟执行的 binding route，再允许更新该 binding state。此前实际 PaneSurface control-click callback 将 `control_id` 与 `action_id` 分开传入；全局 current-binding lookup 可找到另一行的 action，而 dispatch 未验证其属于当前 source control。新检查挡住这一错配，保留现有 admission 与错误反馈路径。

## 实际采用的 postimages

Combined adoption receipt 中的三项 actual raw SHA、LF SHA、bytes 与冻结 packet 逐项相符；新文件是 `binding_owner.rs`。receipt 记录 physical postimages matched、index unchanged。

| Canonical path | Raw SHA-256 | LF SHA-256 | Bytes |
| --- | --- | --- | ---: |
| `zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/componentized_window.rs` | `eaea6ed418920ae13fd364423ea549a9c7f73aa932135af670689a8931faa5cc` | `ea83b1b9892220972eca4230b3449a727a3e9abbe61f5ab1e85b797bd4dd62e1` | 34,642 |
| `zircon_editor/src/ui/retained_host/app/tests/module_tools_menus.rs` | `0ae13346184d79949ecd30e7a50513672ee504f4524ef3dca78700827be58e04` | `363bf6f2568710aba57a738b206e0e191586136aa38025d3fdbb077f11b4c7f3` | 34,242 |
| `zircon_editor/src/ui/retained_host/app/tests/module_tools_menus/binding_owner.rs` (new) | `a23f3f4d0ffcb0d72066ccc5b0c44da64405924bd31f7fd255e8b9ac0070f4f2` | `a23f3f4d0ffcb0d72066ccc5b0c44da64405924bd31f7fd255e8b9ac0070f4f2` | 10,007 |

One ordinary, non-ignored case was added: `ui::retained_host::app::tests::module_tools_menus::binding_owner::workbench_public_preview_binding_requires_current_control_owner`. It has not been run.

## Contracts left unchanged

The existing PaneSurface admissions and module-tools menu selection path remain in place. The authored 139-event contract and original six module-tools cases are preserved. No hard cut, action whitelist, compile backend, generated input lease, or legacy retirement gate was changed. Compile no-op remains a no-op; it is not reported as a successful action. All three legacy retirement gates remain open.

- [x] Source adopted for these three canonical files only.
- [ ] Native case and Editor validation family.
- [ ] The three legacy retirement gates and Compile completion.
- [ ] Editor product, Jenkins and whole-milestone acceptance.

## Frozen packet and combined receipt

- [Frozen candidate patch](../../../../../.codex/outbox/editor-workbench-binding-owner-repair-v1/candidate-v1.patch) — SHA-256 `1161e04c96bd379f24f0a925b9a82cc1d7e93aa7b9d9ea0f3a09a533a5782118`, 12,153 bytes.
- [Source binding](../../../../../.codex/outbox/editor-workbench-binding-owner-repair-v1/source-binding-v1.json) — SHA-256 `217b776d74925191bdf091f5afbade8e2056c8097888ec33a98001ab019e770e`, 7,896 bytes.
- [Source contract proof](../../../../../.codex/outbox/editor-workbench-binding-owner-repair-v1/source-contract-proof-v1.json) — SHA-256 `6b6c9c4e4132d7d94425f3bc4bc60b234da8bae0f43c2e2e4e83651f831b167f`, 12,026 bytes.
- [Native test selectors](../../../../../.codex/outbox/editor-workbench-binding-owner-repair-v1/native-test-entrypoints-v1.json) — SHA-256 `fd442ad55ad80c627843118975fb411b786f54ce710b97ac3452d561887f984a`, 2,507 bytes.
- [Independent peer review](../../../../../.codex/outbox/editor-workbench-binding-owner-repair-v1/independent-review-v1.json) — SHA-256 `7108f3a76f149add6e82bddbc66f35a3782b7f3cb2cbdee5afa256b54d0299f2`, 2,427 bytes; approved for exact canonical Source adoption only.
- [Combined actual adoption receipt](../../../../../.codex/outbox/runtime-editor-public-contract-repairs-actual-adoption-20261012-v1/actual-canonical-apply-v1.json) — SHA-256 `e7c5bccb8e948bd5bf448266fb2e49c826245806b6b9bc9c1d5b9dcea2547e80`, 30,354 bytes.
