# A1 Penpot import/edit/export adapter 证据

- Gate: design-ready
- Status: browser-validated；本地自托管后端验证 deferred
- Owner session(s): `root-designment02-penpot-zui-roundtrip-20260831`
- Changed scope: Angular plugin UI、Penpot plugin runtime、semantic board metadata、detached lane、import/export download、plugin manifest
- Manifest: [a1-penpot-roundtrip.yaml](../manifests/a1-penpot-roundtrip.yaml)
- Commands actually run: offline frozen-lock install；ESLint；Prettier check；root `tsc -b`；Vitest；官方 `plugins-runtime` SES host contract；plugin/Angular production build（Node 24.19）；HTTP manifest/script probe；官方 Penpot 当前 SaaS 前端 + mock backend 的 Playwright canvas contract；Puppeteer visual/overflow probe at 420x520 and 320x520
- Result summary: 4 个 Vitest 文件共 40 项通过；官方 `createSandbox`/`createApi` host contract 执行真实打包插件入口，导入 fixture 后得到 8 个 semantic board / 18 个 shape，从所选文本后代回溯资产，编辑标题和圆角并导出 2 项 projected edits，同时保留 event、`zircon_extension` 和 `runtime_only`。随后浏览器契约加载当前 `https://design.penpot.app` 的真实工作区前端（所有 RPC/WebSocket 均由仓库 fixture/mock 接管，不写远端数据），通过真实层树展开、文本编辑、Inspector 圆角输入和插件下载完成同一 round-trip；输出 `semanticBoards=8`、`totalShapes=18`、`projectedEdits=2`、`preservedRuntimeFields=true`。截图显示实际 canvas、layer tree、`Edited in Penpot canvas`、圆角 `12` 和导出状态。TypeScript project references、ESLint、Prettier 均通过；`assets/plugin.js` 为 59,870 bytes；Angular initial production bundle 185.35 kB；`manifest.json`、UI 和 `assets/plugin.js` HTTP 200，脚本响应为 `text/javascript`；两个视口的 document/body `scrollWidth == clientWidth` 且 `scrollHeight == clientHeight`，无越界元素、控制台错误、页面异常、请求失败或 4xx/5xx 响应
- Repaired failures: Node 24 运行时选择；warning/单复数/窄宽页脚；component/baseline metadata 一致性；不支持 paint/text/layout 的 fail-closed guard；Penpot `StrokeProxy` 返回 `null` cap 的兼容边界；auto-layout 派生几何抑制；free <-> auto 子项位置/尺寸实体化；child mount `slot.layout` 同步；跨父级移动使用 source/current 两套父级语义；官方 SES sandbox 的状态化 Shape host contract；陈旧 deployable bundle 重建；当前层树初始展开状态和真实 contenteditable 文本编辑手势
- Browser contract: [penpot-browser-canvas-contract.ts](../../../../dev/penpot/plugins/apps/zircon-zui-plugin/tools/penpot-browser-canvas-contract.ts)、[JSON result](./a1-penpot-browser-contract.json)、[canvas screenshot](./screenshots/a1-penpot-canvas-roundtrip.png)
- Historical failure capture: [initial contenteditable attempt](./screenshots/a1-penpot-canvas-roundtrip-text-edit-failure.png) is retained only as a repair trace, not acceptance evidence.
- Boundary: 该浏览器证据验证的是当前官方 Penpot 前端、插件 API 和真实画布交互；后端、持久化和登录由官方 Playwright fixture/mock 隔离，不能替代本机 Docker/Nix/Podman 自托管服务的部署/认证/持久化验收。自托管检查保留为后续环境 gate，不阻断 adapter 的 A1 browser contract。
- Evidence links: [SES host-contract harness](../../../../dev/penpot/plugins/apps/zircon-zui-plugin/tools/penpot-runtime-host-contract.ts)、[420px 截图](./screenshots/a1-zui-plugin-420.png)、[320px 截图](./screenshots/a1-zui-plugin-320.png)
- Unlocks: A2-C loader/compiler contract 可以继续；A2-P 仍必须等待 Runtime container padding、Editor 产品接线和 MVP owner gates

本地插件清单开发地址为 `http://127.0.0.1:4213/manifest.json`。服务只用于本次工作树联调，不作为持久发布地址。
