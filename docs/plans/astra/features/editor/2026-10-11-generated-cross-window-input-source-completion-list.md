# Editor Generated 跨窗口旧输入退休 Source 完成列表

日期：2026-10-11。Source repair 已采用；Native、性能和产品验收仍未完成。

- [x] Source contract proof 和 independent peer review 接受该修复；review 没有 finding，且未执行 compiler 或 Native。
- [x] 修复保留既有 Generated registry holder 为所有权 authority；增加只读 visibility getter，只在仍呈现但 holder 已隐藏时请求一次普通完整 publication。
- [x] 旧窗口的 focus、IME、capture 与旧 Source/token 仍由现有 deactivate/input-retirement 路径清理。hide 本体不推进 holder epoch；外层 deactivate 在成功 hide 后推进，后续 project 生成新的 Source/token publication epoch。
- [x] 新增一个非 ignored ordinary case，覆盖 public Attach、旧 child 输入退休、destination holder 即时恢复与恢复后的真实输入合同。
- [x] ordinal 15 原有 191 普通 case 下限加一个唯一 ordinary case 后为 192；这个数字是 Rust test case 数，不是性能 samples。原命令参数、features、selector 与 required names 保持不变。
- [x] 保留原 15 条命令和 14 个原 producer，并记录新增两个 Runtime full-matrix producer 后总计 16 个 producer；新 Editor ordinary case 不计为性能样本。
- [x] 实际 canonical apply exit 0，四个物理 postimages 精确匹配；旧 test file 830 行、31,769-byte 前缀 byte-exact 保留。
- [x] Shared index unchanged、indexWrites=0；两个既有 foreign source inputs 完整保留；sourceCopies=0。
- [x] actualReleasedBytes=0；本次不声称释放空间。
- [ ] Native compile/test 尚未运行或验收。
- [ ] 原 Runtime/Editor performance producers 与 full-matrix 预算尚未运行或验收。
- [ ] Editor products、Jenkins 与 milestone acceptance 尚未完成。
- [ ] Batch06 正准备一次完整异步启动；截至本列表记录时尚未 Native。此项不链接尚未生成的异步回执。

详情见[跨窗口输入 Source 修复记录](../../../optimize/zircon_editor/01/2026-10-11-generated-cross-window-input-repair-record.md)。

实际 evidence： [canonical adoption receipt](../../../../../.codex/outbox/editor-generated-cross-window-retirement-repair-v1/actual-canonical-apply-v1.json)、[Source proof](../../../../../.codex/outbox/editor-generated-cross-window-retirement-repair-v1/source-contract-proof-v1.json)、[peer review](../../../../../.codex/outbox/editor-generated-cross-window-retirement-repair-v1/independent-review-v1.json)、[Native selector](../../../../../.codex/outbox/editor-generated-cross-window-retirement-repair-v1/native-test-entrypoints-v1.json)。
