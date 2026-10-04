# Net current-source静态性能审查

## 状态

- 结论：`static_complete / dynamic_pending`。
- 审查日期：2026-07-30。
- 记账：只进入`pending.md`，不进入`review.md`。
- 代码处置：本轮未修改Rust源码；相关文件包含其他会话的dirty/untracked实现，本轮按current hash只读复核，不吸收。

## 精确范围

| module folder | files | lines | tests | current-source fingerprint |
|---|---:|---:|---:|---|
| `zircon_runtime/src/core/framework/net` | 18/18 | 1,867 | 6 | `ff54b92950fde366da539e97de48d925865418b99bb94e9b5003717de353983f` |
| `zircon_plugins/net/runtime` | 49/49 | 4,464 | 34 | `9ccddd6553dd2c25cae054ddc70257c65b466c627f98106513f072ddd5f2ac56` |
| `zircon_plugins/net/features/http/runtime` | 15/15 | 997 | 11 | `1578233023517e1790a1e3be80c5d46706634ce302f96dabef8e61f36115cb57` |
| `zircon_plugins/net/features/websocket/runtime` | 19/19 | 1,081 | 8 | `3a5d0ed98b5c92130f55915cae39a372a8f4fe9b0296258d327986df76643833` |
| `zircon_plugins/net/features/replication` | 24/24 | 1,148 | 9 | `bbbda4012024b6a2eeebd82c1f4dd4cf3dbba92c70a7917911eff7a16082c538` |
| `zircon_plugins/net/features/rpc` | 20/20 | 1,915 | 27 | `842c2c933e8c0c5848d6218d2d0822db1acafbaf5075e7f96c3ab7d9eb1ab923` |
| `zircon_plugins/net/features/reliable_udp` | 22/22 | 1,251 | 12 | `16ce93a9c53aca0c083821408e3f3486ff4843a3c15f1ebf6e413aa0c72683bd` |
| `zircon_plugins/net/features/content_download` | 20/20 | 1,307 | 15 | `71a141c766f6cf564e260471224bac7b5592818788f001b5855808348825fc8b` |
| `zircon_plugins/net/editor` | 6/6 | 397 | 1 | `b8c50ea2702affc0a8cd9039ec09a61d82e672cf0af7e861cfb679aa77b27aea` |
| `zircon_plugins/net/dist` | 1/1 | 86 | 2 | `9760532d3c611537d1f2b75d1cfc42e865a86b6d9e738999708f0aecca68f4b8` |
| total | 194/194 | 14,513 | 125 | framework contract加完整net crate；net crate整体指纹`ab49dbb3cbe7f8fc7873c8123aa4ed6a24decd240ff79cb2e544340d97d0ab17` |

## 已确认瓶颈

1. `runtime_state.rs`和`worker/transport_runtime.rs`各创建一个默认multi-thread Tokio runtime，后者又运行在专用`zircon-net-worker`线程。单manager因此可能占用约`2 * logical cores + 1`个执行线程，并与引擎task pools叠加。
2. TCP/UDP命令通过`try_send`后同步`recv_timeout(2s)`；HTTP request、WebSocket connect/accept/listen使用调用线程`Runtime::block_on`。连接、listener和socket map的mutex还跨这些等待持有，超时不会取消已入队命令。
3. worker ingress容量为1024，但`try_send`失败被静默忽略；main events、HTTP/WebSocket backend events和每连接WebSocket inbound均为无界`VecDeque`。diagnostics每帧先以`usize::MAX`搬运worker事件，再只处理256项，使产品预算失效。
4. `NetConfig.tcp_poll_budget_bytes`、`udp_poll_budget_packets`和WebSocket message budget没有进入实际poll/drain路径。UDP no-data poll仍分配65,535字节buffer，TCP按`max_bytes`分配；send和message DTO边界继续复制payload。
5. HTTP plain request每次新建Hyper client，HTTPS每次新建Reqwest/TLS client，连接池随请求丢弃；response body完整缓冲且无byte limit，retry立即重试并复制request body。server按连接无限制spawn task，路由线性扫描并在dispatch前复制static response和dynamic request。
6. WebSocket出站仅有64帧上限，入站没有frame/byte/age上限；reader把message转换为owned DTO后再clone进队列，并为每帧写全局事件mutex。现有测试验证顺序和“send文件不含block_on”，没有验证connect/accept、同步reply等待、queue bytes、drop或主线程p95。
7. Replication每session每tick在state mutex内先深clone并排序全部snapshot，再做interest/due/budget；publish按新field×旧field线性查找。filtered/not-due/deferred payload照样复制，session×object replication-time String key持续增长。
8. RPC handler在调用线程同步执行，timeout只能在返回后丢弃结果；schema validator在全局mutex内调用。RPC queue只限制entry，channel queue无界，pending request复制payload，每次drain重排全队列，closed session相关quota/netspeed/pending state无统一回收。
9. Reliable UDP的outbound、fragment assembly与ordered gap buffer均无byte/age上限；远端fragment count可先分配65,535个slot。ACK全扫outbound，resend对每个due sequence再次全扫并clone packets，最坏O(sequences×packets)，超预算packet也已复制。
10. Content download同步等待HTTP最多30秒；resume同时持有state prefix、prefix clone、response body和combined Vec，成功后partial仍驻留。chunk/progress/bitmap多处线性扫描，批量resume逐chunk重复扫描可达O(chunks²)，terminal/cancel没有retained-state清理合同。

## 参考实现核对

- Godot `dev/godot/modules/websocket/packet_buffer.h`同时限制packet count与payload ring bytes，满载明确返回错误；`websocket_peer.h`默认`max_queued_packets = 4096`，`wsl_peer.cpp`同时限制queued message count和outbound bytes。
- Godot `dev/godot/scene/main/http_request.cpp`可选择worker thread或非阻塞poll，并在Content-Length已知时和解压/流式累计后两次执行body byte limit，覆盖压缩炸弹路径。
- Bevy `dev/bevy/crates/bevy_remote/src/http.rs`把server放入共享`IoTaskPool`并用bounded response channel；其per-connection spawn和完整request collect不作为Zircon的无限并发/无限body依据，只用于证明executor应复用统一I/O owner。

## 计划回链与验收

- `PERF-MVP-575`：唯一network execution authority、主线程零同步网络等待、短registry锁、deadline/cancel/shutdown终态。责任计划为`docs/plans/zircon_plugins/07-net.md` M1。
- `PERF-MVP-576`：entry+bytes+age队列预算、HTTP/WS body/frame/concurrency限额、HTTP/TLS client generation复用、no-data零分配、indexed route与共享payload owner。责任计划为Plugins07 M1/M2。
- `PERF-MVP-577`：replication metadata-first selection、dense dirty fields、stable priority与有界session state。责任计划为Plugins07 M4。
- `PERF-MVP-578`：异步/affinity RPC handler、锁外validator、queue byte budget、shared payload和session cleanup。责任计划为Plugins07 M3。
- `PERF-MVP-579`：Reliable UDP fragment/window/bytes/age上限、indexed ACK/resend与wrap-safe sequence generation。责任计划为Plugins07 M5。
- `PERF-MVP-580`：异步流式content download、单一chunk owner、slot-indexed progress及terminal cleanup。责任计划为Plugins07 M6。
- 动态矩阵：1/100/10k connections，1/1k/100k packets/events，0/1KiB/1MiB/256MiB payload，1/2/8/64 logical cores，0/1/60s consumer stall。必须记录线程/task、caller blocked、lock wait/hold、queue entries/bytes/age/drop、client/TLS builds、payload clone bytes、RSS和shutdown wall。

## 已执行的静态门

- 194/194个Rust文件逐文件阅读并追踪framework contract到完整net crate consumer；`zircon_plugins/net`物理current source 176/176，无静态剩余。
- 全net crate及framework net的`rustfmt --check --edition 2021`通过。
- `git diff --check`通过；仅出现当前dirty文件的LF到CRLF提示，没有whitespace error。
- 未运行Cargo或产品trace：当前协调器存在其他有效Cargo job，本轮没有创建、替换或旁路managed reservation。完成current-source managed Cargo、loopback/规模counter及F0/F4产品trace前，本范围保持pending。
