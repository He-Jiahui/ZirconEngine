# Editor01 · Tools 菜单与实际模块来源接入

日期：2026-10-09。实际源码已接入，原生功能、产品呈现与性能仍待验证。

## 实际修复

六个既有核心页面的 Tools 菜单通过实际公开输入打开对应菜单，43 个扩展页面按 canonical URI 只加载当前选中来源，挂在实际 UnrealSceneViewViewport 父节点。此前独立接入的 Particle Library 保留，合计44个扩展描述项。

画面、命中、键盘、clickaway 与回调使用同一来源、完整 enabled/visibility 祖先链和 Window 有界裁剪。发布器先核实际身份、拓扑和布局，再更新来源；原稀疏工作集合保留，命令引用从当前 frame 解析。第二个未刷新的叶子保留自己的旧 Arc/ref，随后经真实 typed input 得到当前引用。

六个真实菜单用例保留；43遍历行计一个 native case。既有 Shader 负例改为真实公开 stage 后修改 live focus，再以 stale proof 拒绝 commit。最后有效树、来源、paint/hit、导航、焦点和私有存储仍受原失败原子性断言约束。

## 接入证据

- 38实际文件：32 Rust、4 ZUI、2 locale TOML。Cargo、tooling和依赖保持。
- patch SHA-256：9ccb3731b768b0ef557b6f5736c38fd54dc2a960b3780d74dcf0d8d9042ef018。
- 最终binding：98632cbaca5184ef38ad1be52fb1326bf30ee3ebef25d550c0f0a7645f5f0a49。
- 全图独审：a01d3a6e2a2e5967d2184e98b5344a9483468704ffab236aa825236a1c0ceb52，零阻断P1/P2。
- [实际接入回执](../../../../../.codex/outbox/editor-module-tools-workspaces-v4/dependency-rebind-v4/actual-canonical-apply.json)：ea70fe276db01ef374500543bac4c75197687566d03bb8d680798690e15bbb56。
- 526个raw/LF守卫两次鲜验；488未改依赖原样；实际32 Rust 2021 fmt、六份TOML解析和scoped diff通过。
- 九处共享foreign改动经独立全字节逆构核为兼容并保留。此前UiAsset48回执属于历史接入证据；其中两个文件后续有foreign修复，当前全48后像及47文件格式证据没有重新宣称不变。
- 原历史rustCount=29已不可变订正为32。Root首次preflight因缺失新文件零字节表示拒绝，未写源码；按实际缺失状态、空hash和零长度核验后一次接入，旧记录保留。
- 共享index、split index、控制文件和已有零字节index.lock的内容及时间元数据保持。

## 同批验证

commands-draft-v8.json SHA-256：c0941fa9e5dff8a4cd39a094d72c26e98eca03971e373146d6da9e2139e0f866。仍为15项，前14项命令与v7原样；Editor #15追加一个namespace和六个精确native名字，保守最低140→146。两个required列表中的旧Shader负例名称同步更新，该例数量不变。Runtime02栈计数只加实际来源记录，零新命令和用例。

14原Release producer、profiles/features/thresholds、Runtime44 54行矩阵及20条critical admission原样。205 Component/258资产、53静态根、139事件断言和offenders-empty强门保留。当前源接入没有实际执行通过原96未达组件门。

协调器入口、不可变baseline增量补丁绑定、实际native、真实产品帧及完整性能门保持开放。没有Jenkins提交或跨聊天发送。详见[计划完成列表](../../../astra/features/editor/2026-10-09-editor01-module-tools-source-completion-list.md)。
