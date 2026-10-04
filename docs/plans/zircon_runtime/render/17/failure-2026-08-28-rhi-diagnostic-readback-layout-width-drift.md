---
handoff_kind: failure
status: open
created_at: 2026-08-28
summary_slug: rhi-diagnostic-readback-layout-width-drift
origin_plan: docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
fixing_plan: docs/plans/zircon_runtime/render/17-performance-and-profiling.md
origin_child_dir: docs/plans/zircon_editor/editor/11
fixing_child_dir: docs/plans/zircon_runtime/render/17
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/device/diagnostics.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/diagnostics/readback/layout.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/diagnostics/readback/tests.rs
tests:
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zr_rhi_wgpu -CheckOnly
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zr_rhi_wgpu -SkipBuild -LibTests -TestFilter texture_layout_ -VerboseOutput
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zr_rhi_wgpu -SkipBuild -LibTests -TestFilter diagnostic_boundary_ -VerboseOutput
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter text_cache_indexes_keep_hot_lookup_and_eviction_work_constant -VerboseOutput
---

# Render17: RHI diagnostic readback layout width drift

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/11-serialization-and-versioning.md`
- 来源执行切片：Editor11 binary direct-decode 的 Text09 原始上行复验
- 修复责任计划：`docs/plans/zircon_runtime/render/17-performance-and-profiling.md`
- 交接原因：texture/buffer diagnostic readback 的 row-byte 计算、对齐和 WGPU copy layout 由 Render17 的 RHI diagnostic owner 负责；Editor11 不能修改未归属的 RHI current source 来放行 serialization 上行测试。

## 失败现象与复现证据

Managed job `e91cafa704a644878089247129b8c8dd` 执行
`text_cache_indexes_keep_hot_lookup_and_eviction_work_constant` 上行复验，于
2026-08-28 01:06:53 CST 正常 `released`，exit `1`，`live_process_pids=[]`。
`zircon_runtime_interface` 已编译且没有 direct-decode 诊断，但目标测试执行数为 0；
`zr_rhi_wgpu` 先报三处 E0308：

- `production/device/diagnostics.rs:665`
- `production/device/diagnostics.rs:722`
- `production/device/diagnostics.rs:771`

三处调用把 `u32 copy_row_bytes` 传给
`DiagnosticTextureReadbackLayout::new(unpadded_bytes_per_row: u64, height: u32)`。
观测边界上，调用文件与 layout 文件均为 untracked current source，ownership matrix
显示 `attribution_missing`，无 lease、无 active Session scope；来源 owner未修改这两个文件。

## 最低共享层根因

readback layout authority 已把未对齐 row byte 宽度提升为 `u64`，但 device diagnostic
调用者仍保留 `u32` 中间表示。接口迁移没有在同一 owner transaction 中完成，导致所有经过
`zr_rhi_wgpu` 的上行 crate 在测试执行前停止。编译器建议的 `.into()` 只处理类型，不证明
texture extent、bytes-per-pixel 乘法和 WGPU 对齐后的范围/截断语义正确。

## 架构修复验收

- row-byte 计算从源头使用 checked `u64`，或在已证明不溢出的边界做显式 checked conversion；不得先在 `u32` 中溢出再拓宽。
- texture、depth 和 buffer diagnostic readback 三条路径统一消费同一 layout authority；WGPU API 所需的窄类型只在最终边界校验后转换。
- 回归覆盖最大合法 extent、乘法溢出、对齐溢出和零/非法布局，typed error 不得退化为 panic、截断或静默空读回。
- managed `zr_rhi_wgpu` lib check GREEN 后，重跑本记录中的 Text09 上行命令并实际执行目标测试。

## 禁止临时方案

- 禁止只在三处调用后追加未经范围证明的 `as u32`/`as u64`、`unwrap` 或饱和截断。
- 禁止复制 layout 计算、绕过 diagnostic readback、关闭 WGPU diagnostic feature，或降低上行测试范围来隐藏编译错误。
- 禁止把两份未跟踪 current-source 文件吸收到 Editor11/Coordinator01 commit；必须先由 RHI owner 完成正式 attribution、lease 和 source-bound validation。

## 修复结果与回传

Open state: `已归属的源码修复待动态验证 / depth-buffer 契约待审`; no RHI or Text09 pass is claimed.

### 2026-09-24 源码归属与验收复核

- 原始验证命令分别为 `cargo +1.94.1 check -p zr_rhi_wgpu --lib --locked --jobs 1 --color never` 和 `cargo +1.94.1 test -p zircon_runtime --lib text_cache_indexes_keep_hot_lookup_and_eviction_work_constant --locked --jobs 1 --color never -- --test-threads=1`；上方 `tests` 改列 Windows 受管同类门禁与最低层 `texture_layout_` 三项回归。上行过滤词还会命中 `*_for_exact_updates`，必须从真实测试日志确认原始目标本身执行且通过，不能用另一项的通过替代。
- 2026-08-28 记录中的“untracked / attribution_missing”是当时的观测，不再代表当前工作树：现行 `diagnostics.rs`、`layout.rs`、`tests.rs` 均是已跟踪文件，四条路径（含本 failure 文档）由已归档的 Render17 owner 以匹配 SHA-256 正式归属，本 session 已由协调器无冲突转接；这些既有源码修复不是本次新增的实现。
- 静态生产调用链：`texture_copy_layout` 以 checked `u64` 计算普通纹理行字节；原生 RGBA8/RGBA16F readback 分别从 `u64::from(width)` 做 checked `* 4` / `* 8`；`DiagnosticTextureReadbackLayout::new` 校验非零、行对齐与 staging 长度后，才在 WGPU copy 边界以 `u32::try_from` 收窄。布局回归文件已有最大可表示行、零尺寸、窄化、对齐和长度溢出用例；尚无匹配当前快照的受管执行结果。
- 原文的“texture、depth 和 buffer 三条路径统一消费同一 layout authority”存在需要审查的契约歧义：当前深度纹理的直接 readback 返回 `RhiError::InvalidCopy`（该路径不编码深度转换），buffer readback 使用独立的 COPY_SRC/对齐/范围校验，并不使用纹理 row-padding layout。保留原始验收文字，不把这两个分支误报为纹理布局测试已覆盖；是否需要新增深度转换是独立契约判断，未在这次编译修复中擅自扩张能力。
- 外部 `E:\Git\zr_vm` 当前仍有其他会话未提交的 tracked diff，无法把它纳入本项不可变受管 Cargo 快照；未清理外部工作树、未重复提交会被同一 admission 前置条件拒绝的 Cargo 请求。保持 open，待前置条件解除后运行下层编译、实际布局回归及 Text09 原始上行测试；无通过票据前不得 `failure return` / `closeout`。

### 2026-09-24 独立审查与 typed-error 回归补齐

- 独立审查针对快照 `3750` 给出 Critical 0 / Important 1 / Moderate 1；这不是零问题审查或通过证据。Important 指向原始 depth/buffer 共用纹理行布局的验收文字与当前两种不同语义不一致；原文保持不变，必须先由 Render17 与原验收 owner 裁决三路径可执行合同，不能在本次编译修复中单方降低要求。
- 对 Moderate 的测试覆盖缺口，现于 RHI device diagnostic owner 添加三个 `diagnostic_boundary_` 回归：普通纹理最大 `u32` 宽度导致的 row-narrowing 转为 `RhiError::InvalidCopy`，无深度转换的直接读回明确返回 `InvalidCopy`，以及 buffer 零长度/越界分别返回 typed error。这些测试锁定当前接口行为，不表示上述 Important 已解决或深度转换功能已获批准；当前仅通过格式/源码检查，尚未执行 Rust 测试。
- 只读 Cargo 输入扫描证实，`zr_rhi_wgpu` 下层 check 和 `zircon_runtime` 原始上行测试均发现同一外部 `zr_vm` Git 源；其工作树未清洁前不可为本快照封存动态票据。等待外部归属方处理后，受管运行所有列出的下层与原始命令，并重新请求 C/I/M 均为零的独立审查；在此之前本 failure 保持 open。
- 针对新增 device 测试后的精确快照 `3751` 再次独立审查，结果 Critical 0 / Important 1 / Moderate 0。三个 `diagnostic_boundary_` 测试的静态类型、目标过滤和 typed-error 映射复核通过；未运行 Cargo，不能作为动态测试通过证据。唯一 Important 仍为原始“texture、depth、buffer 共用 layout authority”验收合同与当前直接 depth 拒绝、buffer 独立范围校验的冲突；保持 open，待验收 owner 裁决后再评审。
