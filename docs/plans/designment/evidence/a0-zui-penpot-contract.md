# A0 `.zui` / Penpot bridge contract 证据

- Gate: design-ready
- Owner session(s): `root-designment02-penpot-zui-roundtrip-20260831`
- Changed scope: `.zui` v2 parser/serializer、Penpot projection model、baseline/current reconcile、可序列化 bridge asset、CLI
- Manifest: [a0-zui-penpot.yaml](../manifests/a0-zui-penpot.yaml)
- Commands actually run: `pnpm install --offline --frozen-lockfile`；`pnpm --filter zircon-zui-plugin test`；`pnpm exec tsc -b apps/zircon-zui-plugin/tsconfig.json --pretty false`；全量 tracked `.zui` audit
- Result summary: 4 个 Vitest 文件共 39 项通过；根 project-reference TypeScript build 通过；319 个 tracked `.zui` 中 304 个当前 v2 资产完成 parse -> project -> serialize -> parse，覆盖 5524 个语义节点和 120 个投影颜色
- Repaired failures: node-less style/theme_tokens profile、完整 Runtime v2 schema 边界、BigInt/TOML date/non-finite metadata envelope、空文本 sentinel、透明色、原样保留未知字段、重复/循环/基线篡改与不支持样式的 fail-closed guard
- Deferred external checks: A2-C 的 Rust loader/compiler 接受性由独立 evidence 跟踪
- Evidence links: [主设计的 bridge 边界](../01-penpot-inspired-interface-design.md#13-penpot-authoring-bridge-与自举边界)
- Unlocks: A1 Penpot adapter；A2-C fixture/test authoring

兼容结论只覆盖 schema v2。其余 15 个 `zircon_editor/src/tests/fixtures/ui_zui` 下的 v1/旧 kind fixture 按 contract 拒绝，不计为 bridge 回归。
