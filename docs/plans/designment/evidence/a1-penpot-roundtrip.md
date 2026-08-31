# A1 Penpot import/edit/export adapter 证据

- Gate: design-ready
- Owner session(s): `root-designment02-penpot-zui-roundtrip-20260831`
- Changed scope: Angular plugin UI、Penpot plugin runtime、semantic board metadata、detached lane、import/export download、plugin manifest
- Manifest: [a1-penpot-roundtrip.yaml](../manifests/a1-penpot-roundtrip.yaml)
- Commands actually run: ESLint；Prettier check；app/CLI/plugin/spec TypeScript checks；Vitest；plugin/Angular production build；HTTP manifest/script probe；Chrome iframe visual/message-state probe at 420x520 and 320x520
- Result summary: 3 个测试文件共 12 项通过；plugin bundle 38.2 kB；Angular initial production bundle 185.32 kB；`manifest.json` 和 `assets/plugin.js` HTTP 200，脚本响应为 `text/javascript`、39,108 bytes；两个视口的 document `scrollWidth == clientWidth` 且 `scrollHeight == clientHeight`，无非预期元素溢出、控制台错误、页面异常或 4xx/5xx 响应
- Repaired failures: Node 24 显式运行避免本机 Node 22.13.1 与 Angular 22 不兼容；warning 色、单复数、窄宽页脚、favicon 请求、component metadata 一致性、node-less theme profile、超千行 projection module 均已修正
- Deferred external checks: 本机没有 Docker/Penpot dev instance，尚未在真实 Penpot canvas 中执行 import -> edit -> export；因此 A1 不标 accepted
- Evidence links: [420px 截图](./screenshots/a1-zui-plugin-420.png)、[320px 截图](./screenshots/a1-zui-plugin-320.png)
- Unlocks: A2 loader/compiler contract 可以继续；真实 Penpot 操作证据仍是 A1 accepted 的硬门槛

本地插件清单开发地址为 `http://127.0.0.1:4213/manifest.json`。服务只用于本次工作树联调，不作为持久发布地址。
