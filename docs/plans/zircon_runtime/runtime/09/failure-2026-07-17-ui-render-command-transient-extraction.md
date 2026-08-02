---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: ui-render-command-transient-extraction
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/09
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/list.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/ui/surface/render/sliders.rs
  - zircon_runtime_interface/src/ui/surface/render
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_command_conversion
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_sliders
---

# Runtime09 failure handoff: UI render command transient extraction

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：F4 template command pipeline 63文件逐文件静态审查
- 修复责任计划：`docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`
- 共同责任：`docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`
- 交接原因：Runtime09拥有`UiRenderCommand -> UiPaintElement`共享转换与render-extract generation；Editor consumer不得各自修补同一cache-generation和typed element权威。

## 失败现象与复现证据

PERF-MVP-178静态审查确认`UiRenderCommand::to_paint_elements`每次分配element Vec。一个带background、image、text与border的command最多产出4个element，而每次`base_paint_element`都重新调用`cache_generation()`；当前实现把包含style、text layout、glyph/cluster与resource payload的完整command执行`serde_json::to_vec`，因此同一command最多生成4份临时JSON bytes并重复计算相同hash。Runtime scene renderer、editor retained host、debug/parity/list路径均消费这一入口。

PERF-MVP-196补充确认runtime UI slider与editor retained slider都把外部tick/steps/layout值直接cast为`usize`后逐项push quad，没有硬预算或pixel密度上限。Runtime09须在`zircon_runtime_interface::ui::surface::render`发布唯一`MAX_UI_SLIDER_TICK_COUNT`合同，两条consumer在解析与最终loop双重clamp；共享预算不能复制为两个私有常量。

## 最低共享层根因

Cache generation属于完整render command，却在per-element constructor内部计算；转换API又只返回owned Vec，未暴露可由runtime/editor共同复用的generation-owned typed element range。因此consumer只能每次重新展开、clone并排序。

## 架构修复验收

Runtime09先把generation计算移到一次command转换前，并让确定性serializer直接写入hash state，保持现有JSON字节hash兼容且不分配临时byte Vec。随后render extract发布typed paint element/role和generation-owned element range，供runtime renderer与EditorUI08共同消费；不得让两个consumer各自建立不同的command-kind或cache-generation权威。

- representative command的新generation等于旧`stable_hash64(serde_json::to_vec(command))`。
- 一个command无论产出1或4个elements，generation计算次数均为1，JSON heap bytes为0。
- 1/100/10,000 commands记录element Vec build、generation calls、serialized bytes、payload clone与CPU scope；stable extract generation转换build为0。
- 保持paint order、z-index、cache invalidation、background/image/text/border拆分、text decorations、resource key、debug/parity和runtime/editor pixels一致。
- 任意有限/非有限/极大slider tick输入在runtime/editor均产生不超过共享预算且不超过track可分辨columns的commands；0/1不画，2/5/常用值语义与pixels等价。

## 禁止临时方案

- 不得只在Editor host复制一套cache generation或typed element cache。
- 不得移除cache generation、改用地址hash或改变序列化兼容语义来换取速度。
- 不得以无界command/element map隐藏稳定帧重建。

## 修复结果与回传

完成后在本目录写入`fixed-*`或return记录，附current-source Windows Cargo、focused contract tests、规模counter与Runtime/Editor consumer parity证据；在此之前本交接保持open。

### 2026-07-22 generation streaming 子叶

- TDD 静态红态确认 `UiRenderCommand::cache_generation` 每次使用 `serde_json::to_vec(self)`，且 `base_paint_element` 为每个 element 重算 generation；转换入口没有预计算值。
- `StableHashWriter` 现在实现 `std::io::Write`，让 `serde_json::to_writer` 直接把原 JSON bytes 写入 FNV-1a state；单 element 与多 element 转换各只计算一次 generation，再把同一值传给 `base_paint_element`。公开 API、paint order、payload 与 cache 字段语义不变。
- parity contract 用旧 `serde_json::to_vec(command)` 独立计算代表性 background/image/text/border command 的期望值，并锁定 1/4 elements generation 一致；`PartialThenFail` fixture 在写入部分 JSON 后主动报错，确认回退仍是旧空-byte FNV，而不是部分 hash。
- source guard 禁止 production `serde_json::to_vec(` 回流，要求两个转换入口恰好各预计算一次，且 element helper 不再调用 `self.cache_generation()`；测试新增到独立 `render_contracts/cache_generation.rs`，没有继续扩大 1799 行根测试 owner。
- source-bound snapshot `940` 冻结 exact3：`command.rs=202b15ad042f2d4ca914a6fc74ba9b602e8b5967479ddc396b46008ca68846f7`、`render_contracts.rs=ee4cc40e7a87eca3c4291968488e0b6dd993d9c5b6eca44460e223355066e5af`、`cache_generation.rs=3aef6535b1075f6e87dd9b8c4c55ccc435ee54d575a6054cbf61c6f0a7ade22d`；`rustfmt +1.94.1 --check`、scoped static gate 与 `git diff --check` 通过，独立 review 为 `Critical 0 / Important 0 / Minor 0`。
- focused Windows reservation `dd66c94d2e5949adba4443170be71b30` 已绑定 snapshot940 exact3；snapshot942 记录时它位于 CPU FIFO position 4，前方为 Text01 running、Plugins01 pending、Shader06 pending。它尚未创建 job，因此不声明 Cargo pass、failure fixed 或 commit；动态 FIFO position 会随前序任务终态变化，不作为固定验收锚。

Open state: `generation streaming 子叶已静态实现并排队验证；typed generation-owned element range、element Vec/payload clone 收束、共享 slider tick budget、1/100/10,000 counters 与 Runtime/Editor consumer parity 仍待完成`。

### 2026-07-22 shared slider tick budget 子叶

- TDD 静态红态确认 Runtime 与 Editor 都把外部 tick/steps 声明直接 `round() as usize`，最终生成循环都直接使用原始 `0..tick_count`；共享上限和 track 可分辨列预算均不存在。
- `zircon_runtime_interface::ui::surface` 现在唯一发布 `MAX_UI_SLIDER_TICK_COUNT = 256`、`bounded_ui_slider_tick_count(...)` 与 `ui_slider_tick_count_for_track(...)`。普通 slider 的 0/1/NaN/负值不生成 ticks，正 Infinity/极大值限到 256；Editor `StepsSlider` 无有效声明时的默认 5 保持不变。
- Runtime 与 Editor 的声明解析 owner 先使用共享 256 上限，两个最终 loop owner 再按 `min(256, floor(track_width))` 收敛；宽 Runtime track 锁定 256、64px track 锁定 64，Editor 512px/24px track 分别锁定 256/24，2/5 常用值路径不变。
- source-bound snapshot `943` 冻结 exact8，manifest fingerprint `42309d5e7c8ca8c1c3692d2f8cb6423e97640388ed86741c014eea84e484eac9`；`rustfmt +1.94.1 --check`、scoped static gate 与 `git diff --check` 通过，`limits.rs` Windows standalone Rust test 1/1 通过，独立 review 为 `Critical 0 / Important 0 / Minor 0`。该 standalone test 只证明纯预算 helper，不代替 interface package、Runtime 或 Editor Cargo gates。
- interface directional reservation `d9e6301dc46a438cb414fa10fed95c9f` 已绑定 snapshot943 exact8；snapshot944 记录时它位于 CPU FIFO position 4，generation reservation `dd66c94d2e5949adba4443170be71b30` 位于 position 3，前方 Plugins01 running、Shader06 pending。两条 Runtime09 reservation 当时都为 pending/no-job；后续验收读取协调器实时状态，不把这些历史 position 当作当前事实。

Open state: `generation streaming 与 shared slider tick budget 两个子叶已静态实现并排队；typed generation-owned element range、element Vec/payload clone 收束、interface→Runtime→Editor managed gates、1/100/10,000 counters 与 Runtime/Editor pixel parity 仍待完成`。

### 2026-07-22 generation managed gate 终态与下层交接

- generation reservation `dd66c94d2e5949adba4443170be71b30` 已绑定 job `5f20293b706949e492df72852c51c725` / run `fb0328e4f22c45d98361d20e9acc4f11`，执行 exact command `cargo +1.94.1 test -p zircon_runtime_interface ui_render_command_cache_generation --locked --jobs 1 -- --nocapture --test-threads=1`。终态为 `released`、exit 101、live process 为空；目标测试执行数为 0，因此不构成 generation red/green。
- rustc 在进入 Runtime09 目标前发现 6 个外部夹具编译错误：Layout18 `input_response_contracts.rs` 的 3 个 `UiNodePath: From<&str>` 漂移，以及 Layout19 `focus_tests.rs` 的 `UiNodePath: From<String>`、`UiTreeId: From<&str>` 和 `serde_json::from_value` 结果类型缺失。最低 owner 后续分别通过 [Layout18 fixed return](fixed-2026-07-22-input-response-fixture-node-path-hardcut-drift.md) 与 [Layout19 fixed return](fixed-2026-07-22-focus-fixture-typed-id-hardcut-drift.md) 回传。
- 下层修复已按硬切边界改用 `UiNodePath::new`、`UiTreeId::new` 和显式 `UiNavigationBoundary` 类型；最终 snapshot `954` / `955` exact2 哈希稳定，独立 review 为 `Critical 0 / Important 0 / Minor 0`。Layout18 focused job `1f7474ecc92e4617a36afaafc971541f` 为 `7 passed / 0 failed / 287 filtered`，Layout19 focused job 为 3/3，原 Runtime09 reproduction 为 2/2；两份下层 handoff 已 fixed return，但本 Runtime09 performance handoff 仍按后续未完成项保持 open。
- slider reservation `d9e6301dc46a438cb414fa10fed95c9f` 已在共享 interface 编译失败后未消费释放，避免继续占用 Plugins01 FIFO；其 snapshot943/standalone 证据不变，但 package/Runtime/Editor gates 仍未执行。
- Layout18 focused reservation `851f554923f34eca832d55d81c962319` 已完成上述 7/7 gate；Layout19 reservation `3203e0c12ea347688ed1091e7fa77ecb` 已绑定 snapshot955 source manifest，等待当前 FIFO 前序自然终态。

Open state: `generation 与 slider 两个实现子叶保持静态完成；先完成 Layout18/19 下层 focused gate 和 failure return，再重跑 Runtime09 original reproduction；不声明 Cargo pass、failure fixed 或 commit`。

### 2026-07-22 lower fixture recovery 与 generation 原复现终态

- Layout19 focused reservation `3203e0c12ea347688ed1091e7fa77ecb` 已绑定 job `fa5a2a52c7024673b8d3f213a6ab0597` / run `6e3f5675a6e648778f974960fe8df4e4`，执行 exact `cargo +1.94.1 test -p zircon_runtime_interface --lib ui::focus::focus_tests:: --locked --jobs 1 -- --nocapture --test-threads=1`。终态 `released`、exit 0、无 live PIDs；原始 stdout 为 `running 3 tests`、`3 passed / 0 failed / 291 filtered`。结合 Layout18 已有的 7/7 focused 证据，两份下层 fixture failure 均已越过各自最低 owner gate。
- Runtime09 原 reproduction reservation `f85113124179480bbfb71a4eaaff8e11` 绑定 source manifest exact5 fingerprint `6d74c769811479db7129ac9fbc6812931572cbbae5866437213bb3aa0b6c3690`，并由 job `d1f58edc59e84a08bd38b30ace51149f` / run `07349ea9c4aa4699a5871a580cbe4bae` 执行原命令 `cargo +1.94.1 test -p zircon_runtime_interface ui_render_command_cache_generation --locked --jobs 1 -- --nocapture --test-threads=1`。终态 `released`、exit 0；原始 lib-test stdout 实际运行 2 个 generation tests，二者均 `ok`，汇总 `2 passed / 0 failed / 292 filtered`。随后 integration binary 为 `0 tests / 3 filtered`，不用于替代已实际执行的 lib-test 证据。
- 该终态证明 generation streaming 子叶和两个下层 fixture recovery 的 focused/upward 链路已绿；Layout18/19 可进入 canonical failure return。Runtime09 顶层 performance failure 仍保持 open：typed generation-owned element range、element Vec/payload clone 收束、slider package→Runtime→Editor gates、1/100/10,000 counters 与 Runtime/Editor pixel parity 尚未完成，不能把本次 2-test pass 扩张为整个 Runtime09 failure fixed。

Open state: `generation streaming managed gate green; Layout18/19 lower failures ready for fixed return; Runtime09 full performance handoff remains open for typed range, slider upward gates, counters, and pixel parity`.

### 2026-07-23 slider managed retry

- snapshot943 exact8 已按当前工作树逐项复核，8 个 SHA-256 均与 manifest 一致，fingerprint 仍为 `42309d5e7c8ca8c1c3692d2f8cb6423e97640388ed86741c014eea84e484eac9`；没有吸收其他 Session 的改动。
- interface focused command `cargo +1.94.1 test -p zircon_runtime_interface --lib shared_slider_tick_budget_bounds_declarations_and_track_columns --locked --jobs 1 -- --nocapture --test-threads=1` 已由 reservation `03e513faed14416382809931471196ed` 绑定上述 exact8 source manifest。创建时位于 CPU FIFO position 5，前方为 Frameworks03 running、Plugins01、Performance01 与 Text01；position 是动态状态，不作为验收锚。
- reservation `03e513faed14416382809931471196ed` 后续绑定 job `89d7d44cc03e49809d9238d3e1ebaf7d` / run `1c832ef0cabf45c195b47f2eb2ab30f4`。终态为 `released`、exit 0、无 live PIDs；原始 stdout 明确为 `running 1 test`，目标 `shared_slider_tick_budget_bounds_declarations_and_track_columns ... ok`，汇总 `1 passed / 0 failed / 293 filtered`，构建耗时 `8m13s`。这只证明共享 interface 预算 helper 的 package gate，不替代 Runtime 与 Editor consumer gate。
- Runtime focused command `cargo +1.94.1 test -p zircon_runtime --lib runtime_slider_tick_commands_are_capped_by_shared_budget_and_track_columns --locked --jobs 1 -- --nocapture --test-threads=1` 已由新 reservation `d773dbcc5a554d85afca017aa26e96a3` 绑定同一 snapshot943 exact8 manifest，compatibility key 为 `267f55fac285e3e2c6260e2934bb04896515027b800b7d05fd2aa1de9a1ea72f`。记录时它仍为 pending/no-job，前方存在 Frameworks03 active job 与 Text01、Performance01 pending reservations；后续只按 terminal job/run 与原始 stdout 记账。

Open state: `slider interface package gate green；Runtime exact8 source-bound retry pending/no job；Editor、pixel-parity、fixed 与 commit 均不声明完成`.
