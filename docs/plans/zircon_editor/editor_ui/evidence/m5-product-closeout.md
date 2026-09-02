# M5 产品收口证据

- Gate: blocked
- Owner session(s): current session
- Changed scope: 汇总本轮 M0-M4 实现、验证当前 Penpot A2-C/A2-P 与 MVP gate，并运行可执行的 token/preview/visual oracle 静态契约。
- Manifest: `docs/plans/zircon_editor/editor_ui/manifests/m5-product-closeout.yaml`
- Commands actually run: `python -m unittest tools.tests.test_editor_zui_design_token_palette_parity_contract tools.tests.test_editor_workbench_preview_token_parity tools.tests.test_zircon_editor_ui_visual_oracle`; `git remote -v`; `git branch -vv`。
- Result summary: 35 tests passed；`origin` 为 `https://github.com/He-Jiahui/ZirconEngine.git`，当前 `main` 跟踪 `origin/main`；本地包含本任务提交且分支总体 ahead 7。
- Repaired failures: none
- Deferred external checks: A2-C managed Cargo、A2-P structured Runtime frame、真实 Editor screenshot、GPU/Softbuffer parity、resize/input performance、MVP F0-F4 owner acceptance。
- Evidence links: `docs/plans/designment/evidence/a2-engine-bootstrap-parity.md`; `docs/plans/designment/manifests/a2-engine-bootstrap.yaml`; `docs/plans/mvp/index.md`。
- Unlocks: none until external owner gates close

## 阻塞结论

M5 不能标记 accepted：

1. A2-C 仍为 `in_progress`。既有 managed job 在 test body 运行前被 Runtime shared-source 246 个编译错误阻断。
2. A2-P 明确要求 MVP F0-F4、container padding contract、Penpot geometry、Runtime structured frame、Editor screenshot 和 tolerance report；当前这些证据未齐。
3. `docs/plans/mvp/index.md` 中 F0-F4 仍是依赖链 blocked 状态，本任务不得越权修改。
4. 当前工作树有大量其他 Session 的 Runtime/Editor UI 改动，启动重型产品构建会混入不稳定输入，不能产生可信 closeout。

## 本轮已闭合范围

- M0：Penpot/Slate/.zui owner 和执行契约。
- M1：侧栏 shrink 与 viewport 优先级自动布局。
- M2：Inspector 搜索同值输入最小 invalidation。
- M3：Transform transient edit 同值输入最小 invalidation。
- M4：Button/Tab/Rail selected semantic visual state。

静态契约累计通过 26 + 20 + 16 + 76 + 35 项测试批次；这些证据不替代 Cargo、产品截图或 GPU parity。
