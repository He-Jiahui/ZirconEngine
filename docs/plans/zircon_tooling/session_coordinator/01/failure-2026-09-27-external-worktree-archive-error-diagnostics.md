---
handoff_kind: failure
status: open
created_at: 2026-09-27
summary_slug: external-worktree-archive-error-diagnostics
origin_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_editor/editor/09
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
tests:
  - managed capture diagnostic regression for file open/read/short-read and TarError failures
  - unchanged external tree acceptance and observed size/hash/Git drift rejection
  - Editor09 snapshot 4178 guarded validation submit and exact lower/SDK test execution
---

# 外部工作树归档异常丢失原始诊断

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 来源 Session：`failure-roll-01a084c8-editor09-asset-pane-r3`
- 来源执行切片：全仓库滚动 failure 修复中的 registry 下层与直接消费者验收。
- 原始 failure：[asset-type-registry-clone-on-augment](../../../zircon_editor/editor/09/failure-2026-07-17-asset-type-registry-clone-on-augment.md)。
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 实际源码 owner：`astra-tooling-dirty-external-snapshot-20260926-01a0df17`，当前 `active`；保留其身份、源码与未完成票据归属。
- 交接原因：错误来自受管归档边界，原始异常被转换为无法区分原因的准入失败。此记录只交接已证明的诊断缺口；外部 IO 失败的最终原因尚未确定。

计划链接：[Editor09](../../../zircon_editor/editor/09-editor-asset-management.md)、[Coordinator01](../01-workflow-control-center-and-tray.md)。

## 失败现象与复现证据

2026-09-27 07:25:52–07:29:17 UTC，正常 `validation.submit` 请求 `failure-roll-01a0df1a-editor09-registry-4178-20260927-r3` 的 command request `fb23cdfc5d1a4856abf8a67b7dca6d7c` 终态 `failed`：

```text
code: validation_ticket_external_worktree_changed
phase: admission
message: External worktree changed while archiving
repoRoot: E:\Git\zr_vm
repairCondition: Seal the complete external source archive with its fixed hash and resubmit.
```

未生成 validation ticket、validation copy 或可复用的归档 snapshot。实际封包使用内存 `BytesIO`，失败发生在持久化之前。最近两份真实 daemon stderr 和 command journal 均没有保存其原异常链，不能从现有证据恢复失败文件或 IO/tar 原因。

输入固定为 Session 基线 `6b4bc86089cb4464f850136c8079e90cb6513ebc` / epoch 611、源码 snapshot 4178 和下列四条 source overlay：

| 源码 | SHA-256 |
| --- | --- |
| `zircon_editor/src/core/asset/type_registry/registry.rs` | `1dfc460afc3823e0a3249782ab1bd8bfbaf334742b2ebcaec84b06ec1cfc4a69` |
| `zircon_editor/src/core/asset/type_registry/registry/batch.rs` | `b8ac1e585a52ab9ccea9103cfd0a8bfb21dd2fabd1e72fa256b0c706ad380a14` |
| `zircon_editor/src/core/asset/type_registry/registry/optimization_batch_ii_editor619_tests.rs` | `7494404f22af55bd499cd9672c1d0010b2357af5030e0e36963d7945a6bcc61f` |
| `zircon_editor/src/core/asset/type_registry/registry/optimization_batch_ij_editor620_tests.rs` | `f2eb09203fa4eaf81a67fff490f92b19e826eb451a84baa5a4d7de41aaa7542c` |

受管命令及工具链保持原请求：

```text
cargo +1.94.1 test -p zircon_editor --lib --locked --release -- --include-ignored --nocapture --test-threads=1 optimization_batch_ii_editor619 optimization_batch_ij_editor620 tests::editor_asset_type_registry:: tests::editor_plugin_sdk::
rust/cargo: 1.94.1; host: x86_64-pc-windows-msvc
linkMode: static; storageMode: reuse; linker: auto
```

external capture 使用 discovery 返回的精确 Windows 路径 `E:\Git\zr_vm` 和当时完整 commit `20ec27750b8764fe1faf0d2056b090bf1f17a154`；仅提交合法 `repoRoot` / `commit` 字段。没有覆盖 compute 参数或提供 caller archive/provenance。

## 最低共享层根因

`validation_external_worktree.py::_capture_files` 的 `except (OSError, tarfile.TarError)` 将全部归档 IO/tar 异常转换为 `validation_ticket_external_worktree_changed`，只保留 `repoRoot`，丢失文件、阶段、异常类型、message、errno/winerror 与读取字节数。

因此此 generic 错误不能证明发生并发修改。已证明的大小/hash 差异另有带 `path` 的分支，Git/status/index 前后 inventory 变化也有独立检查；这些真实漂移拒绝必须保留。

`validation_preflight.py` 的 repair condition 还建议先封存固定哈希再重提，而新 submit 禁止 caller 提供 archiveHash 或 provenance，当前没有 standalone seal/reuse API。该提示无法通过普通受管入口执行。

## 架构修复验收

- 正常受管封包失败记录当前相对路径与阶段，并保留 bounded 原异常类型/message、errno/winerror、预期与已读取字节；打开文件、读取、tar 短读或其他 TarError 可区分。
- 区分归档 IO 失败与已证明的 size/hash/Git 漂移。原冻结、文件类型、前后 inventory、manifest/config/lock 和预算检查不减弱。
- 下层回归实际执行；覆盖 unchanged acceptance、原 IO/tar 原因保留与真实漂移拒绝。提示指向现有受管入口，不要求调用方提供禁止的归档字段。
- 用新增诊断查明原封包失败的实际原因并修复其 owner；在输入重新核对后通过普通 `validation.submit` 封存完整外部树，取得真实 accepted ticket。保留本次终态拒绝，不重放已失败的 command request 冒充新验证。
- 恢复 Editor09 原命令，确认六个 619/620 目标测试及 registry/SDK 直接消费者实际执行并取得匹配源码、命令和配置的终态。原 Editor09 failure 的 F0/F4、完整审查与 closeout 仍按其自身条件验收。

## 禁止临时方案

- 不手工提供 archive/provenance、不静默忽略易变文件、不删除工作树或绕过 live lease、冻结与准入门禁。
- 不把任意 IO/tar 异常视为已证明的并发源码变化，不吞原错误或自动无限重试。
- 不把本记录、静态检查或 queued receipt 当作动态验收通过。

## 修复结果与回传

Open state: 归档诊断合同已修复，六个精确输入的受管 Python 票据 `ac4c998c345f49af8d9b94058f73e8ca` 通过 17/17。后续正常封包识别出外部工作树真实文件漂移；在写入静默窗口，完整外部树已由正常受管入口封存，Editor09 票据 `f9cb2eef9d874c60acc4462a5ece1bfd` 仍为 `queued`。原始 generic 错误丢失的底层异常无法从旧日志恢复，不能将后续漂移反推为原始原因。该票据的诊断尝试在 pinned Cargo metadata 阶段因失效代理失败；直接路由源码候选尚未加载到当前 daemon，也没有受管 cache-miss 证明。修复仍待真实受管 Cargo 与 Editor09 原测试终态后回传；Editor09、Frameworks01 与 EditorUI04 的受影响动态验收保持 open。详见[Coordinator01 证据记录](2026-09-27-external-worktree-archive-diagnostic-evidence.md)。
