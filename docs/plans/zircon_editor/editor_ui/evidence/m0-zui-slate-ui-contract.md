# M0 证据

- Gate: design-ready
- Owner session(s): pending
- Changed scope: 建立 Editor UI / `.zui` Penpot-Slate 收敛计划与 M0 manifest；未修改运行时代码、`.zui` 资产或 MVP 状态。
- Manifest: `docs/plans/zircon_editor/editor_ui/manifests/m0-zui-slate-ui-contract.yaml`
- Commands actually run: 只读 Glob/Read/Grep 审计；Explore worktree 尝试因 Windows 长文件名失败，未作为验证证据。
- Result summary: 已确认 `.zui` v2、Taffy、retained host、GPU command stream、shell region、token、binding/dispatch、selection/viewport 与现有 UI 性能 owner；已映射 Unreal Slate 的 widget tree、WidgetPath、attribute 最小 invalidation、popup/click-outside 与统一输入入口。
- Repaired failures: none
- Deferred external checks: Windows managed Cargo、真实 Penpot browser round-trip、产品输入/截图/frame parity、性能 profile。
- Evidence links: `docs/plans/designment/01-penpot-inspired-interface-design.md`; `docs/plans/designment/02-milestone-execution-and-evidence.md`; `docs/plans/optimize/zircon_editor/01/2026-08-29-ui-hotspot-ownership-review.md`; `docs/plans/zircon_editor/editor_ui/index.md`; `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Widgets/SWidget.h`; `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Application/SlateApplication.h`。
- Unlocks: M1 设计 token、组件状态与 `.zui` shell 布局基线的实现准备。

## 审计结论

1. `.zui` v2 是运行时资产事实源，Penpot 只能作为可逆 authoring projection；padding、顶层 board 原点和 auto-layout 派生几何必须保持明确的导出策略。
2. Shell 已具备 toolbar、activity rail、document host、drawer、status bar 和 main band；后续应收敛 token/variant/布局来源，而不是重建 shell。
3. Taffy 负责 Flex/Grid/Block/Wrap，`UiSurfaceFrame`/arranged tree 应继续作为 layout、hit-test、render extract 的共同空间事实。
4. 高频交互必须走 transient interaction patch 和已提交 generation；pointer move 不应触发全 surface 反射、全树重建、磁盘写入或 undo entry。
5. 性能优化必须遵循现有 owner 记录：优先 generation/index/dirty-domain/compiled paint authority，不在叶子 converter 处添加无失效旁路 cache。
6. 当前 `docs/plans/mvp/index.md` 仍为 `in_progress`，F0-F5 blocked；本 M0 不宣称产品可用或 MVP accepted。

## 风险与边界

- 当前工作树存在大量其他 Session 修改，后续每个 milestone 开始前必须重新取 status/fingerprint，只 stage 明确 owner 文件。
- Git push 会影响共享远端；“推送企微”没有在仓库中找到可执行定义，必须在实际 push 前确认 remote、branch 和外部入口。
- Explore 隔离 worktree 因仓库长文件名失败；后续只读审计应在当前工作树进行，避免复制全仓。
