---
related_code:
  - tools/zircon-session.ps1
  - tools/jenkins/install-coordinator-worker.ps1
implementation_files:
  - tools/zircon-session.ps1
  - tools/jenkins/install-coordinator-worker.ps1
plan_sources:
  - user: 2026-09-29 improve Coordinator storage policy, Failure claim handling, validation scheduling and single-port remote workers
tests:
  - tools/session_coordinator/tests/test_remote_transport.py
  - tools/session_coordinator/tests/test_artifact_relay.py
  - tools/session_coordinator/tests/test_coordinator_remote_e2e.py
  - tools/session_coordinator/tests/test_upgrade_composition_lifecycle.py
  - tools/session_coordinator/tests/test_upgrade_server_routes.py
  - tools/session_coordinator/tests/test_worker_credentials.py
  - tools/session_coordinator/tests/test_worker_protocol.py
  - tools/session_coordinator/tests/test_provider_registry.py
  - tools/session_coordinator/tests/test_optimization_cli.py
  - tools/session_coordinator/tests/test_optimization_actions.py
  - tools/session_coordinator/tests/test_tls_operator_client.py
  - tools/session_coordinator/tests/test_worker_client.py
  - tools/session_coordinator/tests/test_worker_execution.py
  - tools/session_coordinator/tests/test_worker_cache.py
  - tools/session_coordinator/tests/test_remote_paths.py
  - tools/session_coordinator/tests/test_remote_objects.py
  - tools/session_coordinator/tests/test_durable_tasks.py
doc_type: workflow-detail
---

# Coordinator Worker Protocol

> Retired on 2026-10-02. This protocol and its installation/recovery recipes are historical migration source. The local coordinator and its workers reject execution; follow [retirement](coordinator-retirement.md). Do not restart them to obtain validation or acceptance evidence.

## Current operator surface

This checkout has validated schema-80 source; the primary runtime remains schema 73 on loopback. The original Coordinator01 route is still materializing with `rootReleased:false`. Its terminal evidence and explicit root release are required before stopping or upgrading that instance to activate LAN/VPN access.

`RemoteTransport` is the coordinator's one `aiohttp` listener for the operator UI and control plane, SSE, worker WebSocket, and bounded object transfer. It defaults to `http://127.0.0.1:6518`. The only supported listen hosts are `127.0.0.1` and `0.0.0.0`; use `0.0.0.0` for LAN or VPN access. HTTPS and WSS stay on port 6518, and workers initiate outbound connections, so a worker node does not need an inbound listener. A wildcard bind or explicit remote mode requires `--remote-enabled`, a server certificate, private key, trust CA, allowed-host entries retaining loopback authority, and at least one trusted HTTPS browser origin. Client certificates are optional by default; workers still authenticate with their own bearer credential.

### Start and access the one-port coordinator

For first local access, the existing wrapper starts the coordinator singleton on loopback, HTTP, port 6518:

```powershell
.\tools\zircon-session.ps1 start
.\tools\zircon-session.ps1 status
```

Open `http://127.0.0.1:6518` on that machine. To expose the same listener to an operator over LAN or VPN, stop the default instance first, then launch the configured foreground server directly. Bind `0.0.0.0`; use the actual LAN/VPN authority for the browser and worker endpoint. The certificate SAN must include `127.0.0.1` and that actual LAN/VPN IP address or hostname:

```powershell
.\tools\zircon-session.ps1 stop
python -m tools.session_coordinator --repo-root E:\Git\ZirconEngine --port 6518 serve `
  --host 0.0.0.0 --remote-enabled `
  --tls-certificate <server-cert.pem> --tls-private-key <server-key.pem> `
  --trusted-ca <trusted-ca.pem> --trusted-origin https://<LAN-authority>:6518 `
  --allowed-host 127.0.0.1:6518 --allowed-host <LAN-authority>:6518
```

The server, browser UI/control API, SSE, worker WSS, and object transfers share port 6518. The worker connects outbound to `https://<coordinator-host>:6518`; it does not accept inbound connections. The following help commands are parser-only checks and do not start either listener:

```powershell
python -m tools.session_coordinator serve --help
python -m tools.session_coordinator.remote_worker serve --help
```

The transport source registers these operator routes:

| Method and path | Purpose |
|---|---|
| `POST /remote/v1/pairing-codes` | Issue a one-use code with an optional label allow-list and TTL. The default TTL is 600 seconds. The store accepts 1–3600 seconds; the `worker.pairing_code` control action caps its TTL at 600 seconds. |
| `POST /remote/v1/workers/pair` | Exchange a code and worker metadata for a node identity and its credential. The raw credential is returned once; storage retains its SHA-256 hash. |
| `GET /remote/v1/workers` | List worker projections. |
| `POST /remote/v1/workers/{nodeId}/drain` | Stop assigning new work to a worker or resume assignment. |
| `DELETE /remote/v1/workers/{nodeId}` | Revoke that node's credentials. |
| `GET /remote/v1/workers/connect` | Authenticate a worker bearer credential and upgrade to the worker WebSocket. |

The coordinator CLI exposes `worker list`, `worker pairing-code`, `worker drain`, and `worker revoke`. Pairing-code accepts repeated `--allowed-label` and optional `--ttl-seconds` (the controlled action caps it at 600 seconds); drain accepts `--draining` or `--undrain`. For example:

```powershell
.\tools\zircon-session.ps1 worker pairing-code --allowed-label windows --ttl-seconds 600
.\tools\zircon-session.ps1 worker drain <node-id> --draining
.\tools\zircon-session.ps1 worker revoke <node-id>
```

The coordinator `serve` command accepts `--host`, `--remote-enabled`, `--tls-certificate`, `--tls-private-key`, `--trusted-ca`, repeated `--trusted-origin`, and repeated `--allowed-host`. `--host` accepts only `127.0.0.1` or `0.0.0.0`. Remote mode requires TLS material, explicitly trusted HTTPS origins, and an allowed-host list retaining both `127.0.0.1:6518` and the exact LAN/VPN authority; binding `0.0.0.0` also requires `--remote-enabled`. The server certificate SAN must cover `127.0.0.1` and the LAN/VPN authority used by clients. These flags are opt-in remote composition controls and do not change local Cargo path policy.

The direct server invocation for a LAN/VPN listener is:

```powershell
python -m tools.session_coordinator --repo-root E:\Git\ZirconEngine --port 6518 serve --host 0.0.0.0 --remote-enabled --tls-certificate <server-cert.pem> --tls-private-key <server-key.pem> --trusted-ca <trusted-ca.pem> --trusted-origin https://<LAN-authority>:6518 --allowed-host 127.0.0.1:6518 --allowed-host <LAN-authority>:6518
```

This invocation starts the configured listener. Source validation includes the same-source loopback TLS/WSS receipt below. The primary schema-73 instance has not loaded this schema-80 upgrade; its original Coordinator01 route must become terminal and receive root release before activation. After startup, verify `/transport/v1/status` reports `workerProtocolAvailable` and `objectTransferAvailable` as true before treating remote operations as active.

An installable worker entry point is available as `python -m tools.session_coordinator.remote_worker`; its `serve` command accepts `--endpoint`, `--node-id`, `--token`, and `--storage-root`, plus optional `--ca-file`, `--client-certificate`, `--client-key`, repeated `--label`, and `--once`. `status --health` checks the configured coordinator health endpoint. For example, from the repository root on the worker:

```powershell
python -m tools.session_coordinator.remote_worker --endpoint https://<coordinator-host>:6518 --node-id <node-id> --token <paired-worker-credential> --storage-root <dedicated-absolute-root> --ca-file <trusted-ca.pem> serve
```

The worker `--ca-file` configures the worker's HTTPS/WSS client. sccache 0.15.0 uses rustls native roots and has no per-process CA setting, so shared WebDAV caching is enabled only when every certificate in that configured bundle is already in the worker's OS trust roots. Otherwise the worker reports `cache_private_ca_not_os_trusted`, keeps its local cache, and continues compilation without the shared cache. Install the CA in the OS trust store to enable shared WebDAV caching; TLS verification is never disabled.

The operator uses the authenticated `worker.pairing_code`, `worker.drain`, and `worker.revoke` control actions or the HTTP routes above. Pairing-code default lifetime is 600 seconds; the worker credential defaults to one year, is stored by hash, and its raw value is returned once.

The CLI issues a pairing code, but there is no `zircon-session worker pair` or worker CLI `pair` subcommand to exchange it. `POST /remote/v1/workers/pair` is the code-exchange route; the worker `serve` entry point requires the resulting credential. The services are composed into the coordinator source at startup; confirm the running instance's transport status before treating remote operations as active.

The pairing exchange accepts a one-use code and worker metadata at `POST /remote/v1/workers/pair`. It returns the node identity and raw credential once. Keep that credential in a protected variable and pass it to the installer on the worker machine; the installer protects the stored credential for the current Windows user. A worker node may use any dedicated absolute local storage root that is separate from its install directory. That node-local root is not admitted as a local Coordinator Cargo target.

### Current verified worker package

The archive is ready for copying to a worker after the one-port coordinator runtime is activated:

- ZIP: `D:\cargo-targets\mvp-test-fixtures-5728\coordinator-low-disk-runtime-d53eeda9c89c46d1aab765595982e558\coordinator-worker-dist\coordinator-worker.zip`
- SHA-256: `aa878ecdacb8da27bbaff58b7ff6e2fbdc74f393278f5f33da74c41a6174247a`; 131,291 bytes, 28 entries.
- Manifest: adjacent `coordinator-worker.zip.manifest.json`, SHA-256 `7c023e2fb83dcdabc7f6a33d6446d209e8bc4e49a1d323f455b919fafea45ee5`.
- Package checks: 25 ordinary module imports, both CLI help entry points, CRC, all source-to-entry hashes, three installer markers, and no bytecode. `__main__` is exercised through the CLI. The unchanged installer bytecode-filter regression retains its earlier 1/1 evidence.
- Package receipt: `D:\cargo-targets\mvp-test-fixtures-5728\coordinator-low-disk-runtime-d53eeda9c89c46d1aab765595982e558\worker-package-final-receipt.json`, SHA-256 `b26f7c6bd287b54221416a6c7b2404cc96ae95f063e7b85444e0e403e225efe5`.

The ZIP is unchanged from the package receipt. Its current TLS binding receipt is `D:\cargo-targets\mvp-test-fixtures-5728\coordinator-low-disk-runtime-d53eeda9c89c46d1aab765595982e558\worker-package-command-admission-final-tls-binding-receipt.json`, SHA-256 `314e606768325671e042d1934f8192bf91b8c985c77457d00c374bd4e4c96b52`; this supersedes the earlier manifest bindings and preserves their exact bytes. The preceding manifest `cae8518ca41c15fbaccddb2e22d3e44b1f3effff6308202e34194f3abcf3ee5c` and binding `123867f52cc4be7393adbe64c7a6133f815aee61295a48c6bb75690af9eb599b` remain historical evidence.

NativeStorage source `1ad71f057760043c43aa99edc90a3275e2976e772ab45e617b00c5369fa3de54` returns the original exclusive creation handle's directory identity and preserves the Modify ACL behavior. Earlier package `8630d1c797ea6827e5836890978c314fcba88ae5ef507c375067f78169785c68` and its manifest are preserved in the fixture's `tmp\worker-package-history-e1e1fdc25aba4e3ca261f6ca20e54492`.

The current same-source loopback TLS/WSS E2E passed 1/1 in 36.154 test seconds (41.858 seconds for the receipt envelope), with stable production and test hashes during execution: `D:\cargo-targets\mvp-test-fixtures-5728\coordinator-low-disk-runtime-d53eeda9c89c46d1aab765595982e558\remote-worker-tls-command-admission-final-receipt.json`, SHA-256 `62e1b0f41af9fc26754425aa87eaa189856c159455cf9b81a8cbd3c2caaa39a9`. It covers pairing, sealed input transfer/provider PASS, output/SSE/artifacts, natural terminal proof and reuse indexing, disconnect/rejoin, late-result history, exact stop acknowledgement, and generation 2. The test holds a real executor stop proof at the transport boundary, confirms the old result alone retains its lease, and then forwards that unchanged proof. The preceding passing receipt (`393f04b7c0ddd9ea8f119fb09d3ba2e56149111862af8a71533ccde9849b9385`) and earlier failed delivery receipt (`8d235517cb13292aecee1e4443e92ff83bd3ec8506811769054ffa0895463f06`) remain historical evidence. Package source hashes match the exercised production sources. This test ran the worker from the checkout; ZIP import checks are separate evidence, and no worker installation or physical two-machine acceptance has occurred.

Verify the current ZIP hash before copying. The two actual Windows workers remain deferred until the user connects them and requests acceptance. Tiny and heavy/light Cargo acceptance remain pending: a Main pending reservation is not a cross-database execution lock; use the activated Main authority or a dedicated worker. Preserve the original route release checkpoint.

```powershell
# On the coordinator, verify the current fixture archive before copying it.
$workerZip = 'D:\cargo-targets\mvp-test-fixtures-5728\coordinator-low-disk-runtime-d53eeda9c89c46d1aab765595982e558\coordinator-worker-dist\coordinator-worker.zip'
$expectedWorkerZipHash = 'aa878ecdacb8da27bbaff58b7ff6e2fbdc74f393278f5f33da74c41a6174247a'
$actualWorkerZipHash = (Get-FileHash -LiteralPath $workerZip -Algorithm SHA256).Hash
if ($actualWorkerZipHash -ne $expectedWorkerZipHash) { throw 'Worker package hash does not match its receipt.' }

# Issue the one-use code only after the current coordinator runtime is active.
.\tools\zircon-session.ps1 worker pairing-code --allowed-label windows --ttl-seconds 600

# On the Windows worker, after copying the accepted ZIP there:
$workerZip = 'C:\ZirconWorker\staging\coordinator-worker.zip'
$expectedWorkerZipHash = 'aa878ecdacb8da27bbaff58b7ff6e2fbdc74f393278f5f33da74c41a6174247a'
$actualWorkerZipHash = (Get-FileHash -LiteralPath $workerZip -Algorithm SHA256).Hash
if ($actualWorkerZipHash -ne $expectedWorkerZipHash) { throw 'Worker package hash does not match its receipt.' }
$packageStage = 'C:\ZirconWorker\staging\package'
New-Item -ItemType Directory -Force -Path $packageStage | Out-Null
Expand-Archive -LiteralPath $workerZip -DestinationPath $packageStage -Force
$installer = Join-Path $packageStage 'tools\jenkins\install-coordinator-worker.ps1'
$pairing = Invoke-RestMethod -Method Post `
  -Uri 'https://<coordinator-host>:6518/remote/v1/workers/pair' `
  -ContentType 'application/json' `
  -Body (@{
    code = '<one-use-pairing-code>'
    credentials = @{
      nodeId = 'windows-worker-01'
      displayName = 'Windows Worker 01'
      platform = 'windows'
      architecture = 'amd64'
      labels = @('windows')
    }
  } | ConvertTo-Json -Depth 5)
$nodeId = $pairing.data.worker.nodeId
$workerCredential = $pairing.data.credential
& $installer -Action Install -RepoRoot $packageStage `
  -Archive $workerZip -InstallRoot 'C:\ZirconWorker\software' `
  -StorageRoot 'D:\ZirconWorker\data' `
  -Endpoint 'https://<coordinator-host>:6518' `
  -NodeId $nodeId -Token $workerCredential -CaFile 'C:\ZirconWorker\trust\coordinator-ca.pem' `
  -PrepareMsvc -VsDevCmdPath '<worker-VsDevCmd.bat>'

# Install registers a logon task; launch once now to connect immediately.
& 'C:\ZirconWorker\software\run-coordinator-worker.ps1'
```

Use a TLS-validating PowerShell client for the pairing exchange; ensure the coordinator certificate chains to a CA trusted by that client, and do not bypass certificate checks. `Install`/`Update` require explicit absolute local, non-UNC `-InstallRoot` and `-StorageRoot`, and the roots must be separate directory trees. The worker stores temp and pip cache data below its dedicated `-StorageRoot`. Use `-PrepareMsvc -VsDevCmdPath` when this worker will execute MSVC jobs. The installer has not been run on a real worker. The two real Windows workers are intentionally deferred until the user connects them and requests acceptance; this does not block the one-port coordinator access path.

The concrete `WorkerProtocolService` surface is `create_pairing_code`, `pair_worker`, `authenticate_worker`, `list_workers`, `set_worker_drain`, `revoke_worker`, `heartbeat`, `claim_task`, `submit_result`, `disconnect_worker`, and `confirm_stopped`. `ProviderRegistry` registrations carry `kind`, `name`, `version`, a callable handler, labels, local/enabled state, and an allow-list; task claims require an enabled compatible provider and exact version when one is requested.

## Worker messages and attempt evidence

The worker WebSocket requires a matching `hello` before it accepts control traffic. The control channel handles `heartbeat`, `claim`, `submit_result`/`result`, and `stopped`. Worker log frames are lease-bound and encode raw bytes as `byteCount` plus `dataBase64`, with a 16 KiB raw-frame cap. Object transfer and cache relay use separate logical channels with bounded queues. The HTTP object routes provide manifest lookup and bounded, aligned chunk upload/download with digest checks and explicit finalization. These routes are served by the same listener.

The remote worker sends heartbeats every 10 seconds by default. Remote task/node leases are a separate mechanism from Failure claims: their default TTL is 60 seconds and `WorkerProtocolService` clamps its `lease_ttl_seconds` setting to 5–300 seconds. The current `serve` CLI does not expose that setting. Failure claims retain their own 30-second renewal cadence and 300-second expiry.

Wire task and result envelopes bind protocol version, `taskId`, `attemptId`, generation, and immutable `inputHash`. The final same-source TLS/WSS E2E verifies natural terminal evidence and reuse indexing. A late result missing final cursors or process-tree proof stays historical and retains its lease resources; the real stop acknowledgement releases only the exact old lease, allowing generation 2. Durable Task/Attempt records append input, result, artifact, output, and receipt references. An attempt result alone is not a validation-ticket PASS or a Cargo receipt.

The protocol validates bounded JSON and rejects absolute filesystem paths from worker payloads. Provider registrations are versioned and checked against enabled state and label allow-lists. `WorkerExecutor` provides one heavy and two light execution slots. Its input materializer caps stable source slots at two and fences reuse by slot generation and input hash.

## Integration and acceptance boundary

`CoordinatorUpgradeServices.install()` creates the durable task service, provider registry, and `WorkerProtocolService`, then publishes the worker protocol on the coordinator application. `RemoteTransport` receives that protocol and the sealed-source transfer service from the application/server lifecycle before it starts its single listener. Current source migration schema is version 80; the recorded live coordinator checkpoint remains schema 73, so schema-80 composition is not established as loaded by that instance. `/transport/v1/status` reports `workerProtocolAvailable` and `objectTransferAvailable` for the running instance.

Source validation and runtime activation are separate evidence. A validation receipt proves the declared immutable source inputs against its recorded command. It does not prove that the running coordinator instance loaded those files. Record the loaded instance and its activation result separately. Preserve the existing route 13 checkpoint and its root-owned release decision; neither source validation nor a health response silently advances it.

Earlier affected transport/output/reclaimer checks (28), worker checks (24), browser functional checks (35), controlled Failure claim checks (19), and repaired selection/scroll checks (7) remain scoped evidence for unchanged relevant code. The current loopback TLS/WSS and ZIP checks are listed above. Current source is schema 80; the primary runtime remains schema 73 and has not loaded this upgrade. Actual Tiny/heavy-light Cargo, LAN runtime activation, and physical Windows worker acceptance remain pending. The user will connect the Windows nodes before requesting their acceptance. Source validation does not advance the original root-owned route release checkpoint.

## Owning source

See [the local Coordinator operator guide](local-session-coordinator.md) for the broader service lifecycle and [the storage and task policy](coordinator-storage-and-task-policy.md) for local versus worker storage, scheduler limits, and reclamation.
