# A0 `.zui` / Penpot bridge contract 证据

- Gate: design-ready
- Owner session(s): `root-designment02-penpot-zui-roundtrip-20260831`
- Changed scope: `.zui` v2 parser/serializer、Penpot projection model、baseline/current reconcile、可序列化 bridge asset、CLI
- Manifest: [a0-zui-penpot.yaml](../manifests/a0-zui-penpot.yaml)
- Commands actually run: `pnpm --filter zircon-zui-plugin test`；四个独立 `tsc --noEmit` config；全量 tracked `.zui` audit
- Result summary: 11 个 Vitest 用例通过；plugin/CLI/app/spec TypeScript 检查通过；303 个当前可读 v2 资产完成 parse -> project -> serialize -> parse，覆盖 5516 个语义节点
- Repaired failures: 合法的 node-less style/theme_tokens profile 不再误报缺少 `[nodes]`，序列化不会插入空表
- Deferred external checks: A2 的 Rust loader/compiler 接受性由独立 evidence 跟踪
- Evidence links: [主设计的 bridge 边界](../01-penpot-inspired-interface-design.md#13-penpot-authoring-bridge-与自举边界)
- Unlocks: A1 Penpot adapter；A2 fixture/test authoring

兼容结论只覆盖 schema v2。15 个 `zircon_editor/src/tests/fixtures/ui_zui` 下的 v1/旧 kind fixture 按 contract 拒绝；`zircon_editor/assets/ui/editor/animation_editor.zui` 在共享工作树中缺失，因此未计入通过或失败。
