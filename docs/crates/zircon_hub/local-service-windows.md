# Zircon Hub local service on Windows

This procedure runs Keycloak and `zircon_hub_service` only on IPv4 loopback. It is a local
development deployment, not an internet-facing production configuration. The checked-in realm
contains no users, passwords, client secrets, bearer tokens, or private keys.

The deployment artifacts are under `zircon_hub/deploy/local`. They currently pin Keycloak
26.7.3 because that is the version in the official ZIP getting-started guide reviewed on
2026-09-07. Check the release download and its published digest before installing a newer ZIP.

Official references:

- [Keycloak ZIP getting started](https://www.keycloak.org/getting-started/getting-started-zip)
- [Supported configurations](https://www.keycloak.org/server/supported-configurations)
- [Importing and exporting realms](https://www.keycloak.org/server/importExport)
- [Server configuration](https://www.keycloak.org/server/configuration)
- [OIDC client and PKCE administration](https://www.keycloak.org/docs/latest/server_admin/)
- [Health checks](https://www.keycloak.org/observability/health)

## Prerequisites

Use the official Keycloak 26.7.3 ZIP and a supported 64-bit OpenJDK. Keycloak currently supports
OpenJDK 17, 21, and 25; its current ZIP guide recommends OpenJDK 25. The workstation's global
`JAVA_HOME` pointed to Java 8 (`1.8.0_291`) during the 2026-09-07 inspection. A separate, verified
Temurin 25.0.4.1+1 archive was used for the local deployment check without changing that global
setting. Install or unpack a supported JDK separately and pass its absolute directory as `-JavaHome`.

Build `zircon_hub_service` with the independent Windows wrapper. The wrapper keeps Cargo's
target, compiler caches, build directories, and temporary compiler outputs physically under a
drive-root `D:\cargo-targets`, `E:\cargo-targets`, or `F:\cargo-targets` leaf and returns the
actual Cargo exit code. Local command evidence leaves Jenkins, milestone and product acceptance
separate.

```powershell
$TargetDir = 'D:\cargo-targets\zircon-local\hub-local-service-operator\target'
.\tools\local-cargo.ps1 -TargetDir $TargetDir build -p zircon_hub `
    --no-default-features --features local-service --bin zircon_hub_service --locked
```

The command builds the standalone service for the current source under `$TargetDir`. Its
`local-service` feature includes the shared runtime-interface DTO dependency used by cloud
package-lock validation. The private run6 used the earlier binary
`D:\cargo-targets\zircon-local\hub-local-service-validation-20261002-r1\target\debug\zircon_hub_service.exe`
(SHA-256 `45dcaaa2346d830d1f764f394f31ae325a818d46a8189503d747dbb130364643`).
That binary predates the current S6 typed package-lock source and does not validate it.

Choose absolute local paths for the unpacked Keycloak ZIP, the supported JDK, Hub state, an
introspection secret file, and a cloud encryption key file outside the repository and outside the
Hub state root. The introspection secret file is the
confidential client's initial secret. It must contain 32-4096 unpadded base64url characters
(`A-Z`, `a-z`, `0-9`, `_`, or `-`).
Generate and protect it with your approved secret-management process. Do not place it under the
Hub state directory if that directory will be backed up or shared.

The cloud key file must contain exactly 32 cryptographically random raw bytes. Generate it through
your approved secret-management process; do not type text into that file. Keep this key with the
backup set in the approved secret backup system because encrypted CAS blobs cannot be restored
without the matching key.

The examples below use placeholders. Set them to real absolute paths in the operator terminal.
Keycloak may generate Quarkus augmentation when built or started, so its writable runtime and
compiler temporary/cache paths must also stay physically under an approved `D/E/F:\cargo-targets`
drive root. The prebuilt JDK, SQLite/CAS state and secret files remain separate inputs:

```powershell
$Deploy = 'E:\Git\ZirconEngine\zircon_hub\deploy\local\Manage-LocalDeployment.ps1'
$KeycloakHome = 'D:\cargo-targets\zircon-local\hub-local-deployment\keycloak-26.7.3'
$JavaHome = 'D:\Tools\jdk-25'
$StateRoot = 'D:\ZirconData\hub-local'
$SecretFile = 'D:\ZirconSecrets\hub-introspection.txt'
$CloudKeyFile = 'D:\ZirconSecrets\hub-cloud.key'
$ServiceExe = Join-Path $TargetDir 'debug\zircon_hub_service.exe'
# Future desktop build example only; this binary was not built or accepted in run6.
$HubExe = 'D:/cargo-targets/zircon-local/hub-desktop-validation-r1/target/debug/zircon_hub.exe'
```

## Validate and configure

Validate the checked-in realm and deployment manifest without starting any process:

```powershell
& $Deploy ValidateArtifacts
```

Render the exact Rust service TOML and desktop account JSON. The generated service configuration
contains only the absolute path to the secret file, never its contents.

```powershell
& $Deploy Configure `
    -StateRoot $StateRoot `
    -IntrospectionSecretFile $SecretFile `
    -CloudKeyFile $CloudKeyFile
```

The resulting contracts include:

- issuer: `http://127.0.0.1:8080/realms/zircon-local`
- service audience and confidential introspection client: `zircon-hub-service`
- public desktop client: `zircon-hub-desktop`
- exact callback: `http://127.0.0.1:8480/callback`
- Hub service: `http://127.0.0.1:8787`
- encrypted cloud CAS: `$StateRoot\data\cloud`, with its 32-byte key outside `$StateRoot`

The desktop client has client authentication disabled, Authorization Code flow enabled, implicit
and password grants disabled, and mandatory PKCE method `S256`. Its default `basic` client scope
keeps the `sub` claim in access tokens on Keycloak 26.7.3, while the audience mapper adds
`zircon-hub-service` to access tokens. The separate confidential client has all user-facing grants
disabled and exists only so the service can call Keycloak's introspection endpoint with HTTP Basic
client authentication.

First-login ID tokens must contain the authorization request's nonce. On refresh, the broker
accepts an omitted nonce and verifies it against the saved original when present, as specified by
[OIDC Core section 12.2](https://openid.net/specs/openid-connect-core-1_0.html#RefreshTokenResponse).
Signature, issuer, audience, expiry, original subject and any access-token hash remain checked.

## Bootstrap Keycloak

Keycloak has no default administrator. With Keycloak stopped, use its official interactive
bootstrap command so no password appears in repository files, shell history, or process arguments:

```powershell
$env:JAVA_HOME = $JavaHome
$env:Path = (Join-Path $JavaHome 'bin') + [IO.Path]::PathSeparator + $env:Path
& (Join-Path $KeycloakHome 'bin\kc.bat') bootstrap-admin user
```

Follow the prompt and treat this as a temporary master-realm administrator. Import the Zircon
realm while Keycloak remains stopped. Realm import substitutes the confidential client secret
from the user-provided secret file through a process-local environment variable; the script clears
its copy after the import command exits.

```powershell
& $Deploy ImportRealm `
    -KeycloakHome $KeycloakHome `
    -JavaHome $JavaHome `
    -IntrospectionSecretFile $SecretFile
```

The import uses `--override false`. A subsequent import will not overwrite an existing realm or
its users. To change the confidential client secret after first import, rotate it in Keycloak and
atomically replace the secret file while both Hub processes are stopped. Re-importing with
override enabled is intentionally not automated because it can destroy local realm state.

Start Keycloak in a dedicated terminal. The script remains attached to the foreground process so
`Ctrl+C` reaches Keycloak. It fixes the public and management listeners to `127.0.0.1`, fixes the
issuer hostname, enables database-aware health checks, and disables asynchronous bootstrap.

```powershell
& $Deploy StartKeycloak -KeycloakHome $KeycloakHome -JavaHome $JavaHome
```

Open `http://127.0.0.1:8080/admin/`, sign in to the master realm with the temporary administrator,
and create actual test users in the `zircon-local` realm. Do not add user credentials to the realm
JSON. Remove the temporary bootstrap administrator after establishing the intended administrative
access.

## Start and check the Hub service

In a second terminal, start the service in the foreground. It refuses to bind until OIDC discovery,
JWKS loading, the introspection secret, and the SQLite migration have succeeded.

```powershell
& $Deploy StartService `
    -StateRoot $StateRoot `
    -HubServiceExecutable $ServiceExe
```

In a third terminal, check both readiness contracts:

```powershell
& $Deploy Readiness
```

The check requires Keycloak `/health/ready` to report `UP` and Hub `/health` to report
`{"status":"alive","protocolVersion":1}`. The Hub endpoint is reachable only after identity and
database initialization because the listener binds last.

Start the desktop executable with the generated account configuration. The desktop reads this
trusted path once at startup; browser previews have no native account broker.

```powershell
$env:ZIRCON_HUB_ACCOUNT_CONFIG = Join-Path $StateRoot 'config\account.json'
& $HubExe
```

Open the Team page to sign in through the system browser. The account panel supports paged
organization, member, project, and invitation lists, organization/project creation, and invitation
acceptance. Before a mutation is sent, the native broker persists its operation ID and original
request in `%LOCALAPPDATA%\ZirconHub\Account\operations.dat`. The Windows user account protects
this file with DPAPI; it contains operation payloads, not credentials. A trusted account config
can override `operation_journal_path` with an absolute local path. Do not delete or replace this
journal to clear an uncertain result.

After restarting and signing into the same issuer, subject, client and service, the panel restores
pending operations. Check result queries the server receipt, Retry retains the original operation
ID and payload, and Dismiss acknowledges only a committed or failed operation. Unknown outcomes
block new mutations. A journal read failure also blocks new mutations and exposes a separate
recovery error. Windows journal durability, process races and account isolation still require the
managed native test batch and real desktop acceptance.

Journal format v2 retains a separate revision for each qualified identity after acknowledgement.
Existing v1 journals migrate explicitly while keeping their pending operations. The 16 MiB journal
budget applies to both payloads and identity history; no fixed lifetime count of identities blocks
an otherwise valid journal.

Stop the Hub service and Keycloak with `Ctrl+C` in their foreground terminals. Hub requests have a
15-second bound. After the stop signal, the service stops database admission and shares a
20-second deadline across startup owners, axum and tracked database jobs, including jobs whose caller
was cancelled. Ctrl-C is registered before configuration is read. The binary then bounds the final Tokio runtime wait to two seconds. A timed-out drain
reports an unknown outcome and active/finished/failed database counts; it does not report a clean
stop. Startup cancellation and real blocked-I/O process termination remain pending acceptance.
Confirm that both processes exited and their three loopback ports closed during that acceptance.

## Backup and restore

Stop both processes before backup. The command refuses to continue while ports 8080, 9000, or 8787
are open. It copies the closed Keycloak data directory, the closed Hub SQLite state, generated
configuration, and non-secret metadata into a new or empty destination.

```powershell
$Backup = 'F:\ZirconBackups\hub-local-2026-09-07'
& $Deploy Backup `
    -KeycloakHome $KeycloakHome `
    -JavaHome $JavaHome `
    -StateRoot $StateRoot `
    -BackupPath $Backup
```

The backup contains identity data and must be protected accordingly. The introspection secret and
cloud encryption key files are deliberately excluded; retain both in the approved secret backup
system. Keep the hidden CAS `.store-id` with the SQLite backup. Schema v5 binds its UUID and cloud
key fingerprint to the database; a different root identity or key is rejected even for an empty
CAS. The backup script includes hidden files. A matching backup can move to new absolute paths.
The per-user desktop operation journal is outside `$StateRoot` and is not part of this server
backup; DPAPI recovery requires the same Windows user context. Keycloak realm export alone
is not a complete backup: official documentation states that it omits persisted sessions, revoked
tokens, events, and workflow state. The stopped data-directory copy is the authoritative local
restore input for this deployment.

Restore into an unpacked Keycloak 26.7.3 directory whose `data` directory is absent or empty, and
an absent or empty Hub state directory. The restore refuses to delete or overwrite either target.
Supply the restored secret file; the script rewrites Hub configuration with the new absolute paths.

```powershell
& $Deploy Restore `
    -KeycloakHome $KeycloakHome `
    -JavaHome $JavaHome `
    -StateRoot $StateRoot `
    -IntrospectionSecretFile $SecretFile `
    -CloudKeyFile $CloudKeyFile `
    -BackupPath $Backup
```

Start Keycloak, start the Hub service, run `Readiness`, then perform a fresh desktop login. Do not
assume browser or refresh-token sessions survive a restore unless that behavior was explicitly
validated for the restored snapshot.

## 2026-10-03 private run6 service evidence

The private run6 receipt exercised the normal local HTTP service with a fresh Keycloak 26.7.3
runtime and a new SQLite/CAS state root. The real authorization-code PKCE S256 flow produced a
signed RS256 access token; the issuer, service audience, signature, JWKS and subject were verified.
The 23 recorded API categories were: health; organizations-empty; organization-create and replay;
project-create; members; projects; cloud-blob-upload; signed catalog publish; catalog artifact
upload; licenses-before; license acceptance; catalog manifest; catalog artifact download;
cloud commit; post-commit cloud blob download; commit replay; typed conflict; cloud head;
post-logout old-token rejection; post-restart cloud head; post-restart cloud blob; and
post-restart authenticated list. Refresh succeeded, logout returned 204, and the old access token
was rejected with 401. The service restarted from the same SQLite/CAS paths and read the committed
head and blob back successfully. SQLite byte identity was not equal because audit/receipt rows are
written during operation; semantic readback passed.

Both owned service and Keycloak processes stopped through the normal hidden dedicated-console
Ctrl+C path with `targetExited=true`; ports 8080, 9000, 8787 and 8480 were closed afterward.
The receipt is [terminal-result.json](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-native-identity-execution-r2/terminal-result.json),
SHA-256 `b50ec983fa9996cec18cdcf0dc7df11bd51262e2cee633f03ff8e8697eebd5ac`.

Independent receipt review verified the 16 earlier source inputs; run6 did not execute the
negative cases. The separate live r10 run on October 3, 2026, 03:22:11–03:24:01 UTC executed
13 negative assertions: invalid signature, unknown key ID, malformed bearer, wrong audience,
wrong issuer and expired token returned 401; cross-tenant read/mutation, member self-revocation
and revoked-member read/mutation returned 403; pre-commit blob GET returned 403; stale commit
returned 409. Member and head semantic hashes stayed unchanged after denied operations.
Post-commit payload read, same-result replay and head revision 1 also passed. PKCE wrong-verifier
and code-replay assertions reuse the earlier live r6 observation; r10 did not repeat them.
Both owned processes stopped normally and all four ports closed. The immutable
[negative terminal receipt](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-native-identity-negative-verification-r10/terminal-result.json)
has SHA-256 `72ee2d32697f009d483ca70fda52e42c905d31a75167491fd832c229b21b5a3e`;
its observation is `35040df77cf11326ec28a2517ed5980614f6ee43b0c40f44bdb5d730adf13dcf`.
The replay observation's `status` field was overwritten by the JSON body's `committed` value;
the driver's explicit HTTP-200 assertion and same-result comparison are the replay evidence.

This is service/API evidence for the earlier binary generation only; it excludes the current
S6 typed package-lock source. Desktop UI, the native account broker and Windows Credential
Manager, signed package installation/rollback, native S6 two-client sync, power-loss, backup/
restore, performance and Jenkins acceptance remain open. It does not claim a three-process UI
run or complete S1. The earlier r1 warm-build output was overwritten accidentally; the exact r5
build bytes and result are retained under the r2 warm-build reconciliation directory, while the
old r1 stdout hash is explicitly unavailable.

## Remaining product gaps

The deployment artifacts describe the current S0/S1 server and broker contracts. The
desktop application now loads the generated `account.json` through `ZIRCON_HUB_ACCOUNT_CONFIG`,
exposes the account and team operations listed above, and fences every response by the Hub backend
epoch and account generation. Browser previews report account service unavailable. Member role
changes, ownership transfer and issuing/revoking invitations now have browser-validated UI
workflows; their native service acceptance is pending. Marketplace license review and package
installation now have browser-validated UI with inventory refresh and unknown-outcome recovery:
the Chrome service-catalog fixture passed 18/18, including English/Chinese and 360/768/1280/1920
layouts. These mocked browser flows do not prove signed package download, installation or rollback
against the native service. Desktop cloud synchronization remains a pending UI workflow.

`zircon_hub_service` tracks database jobs and bounds HTTP draining, but has no durable shutdown
receipt. Process termination with in-flight blocking I/O and restart recovery remain unaccepted.
Windows database admission rejects existing hardlinks before SQLite migration. Configuration,
account journal and database path opens reject reparse points throughout the path. The checked
handles remain owned through each read, transaction or database job. Native regression execution
and process-level race acceptance remain pending; other platforms do not yet have the same
database admission contract.

The database directory and existing database, WAL, SHM and rollback-journal files must grant
access only to the service's Windows user, SYSTEM or Administrators. Their owners must also be
one of those principals. The directory inheritance chain must remain private through a protected
DACL. Configure, Backup and Restore create private destinations; an existing directory with
broader access is rejected without rewriting its ACL. Use a new private state directory or
restore the stopped deployment into one. The service verifies access through opened handles
before migration and does not modify directory permissions. SQLite keeps control of deleting
and recreating its sidecars during recovery and checkpointing. Processes running as the same
Windows user, SYSTEM or an administrator are inside this trusted operating-system boundary.

Full S1 acceptance still requires a real three-process run that proves login, refresh, logout and
subsequent introspection rejection, expired and wrong-audience token rejection, JWKS rotation, two
concurrent callback attempts, Windows Credential Manager persistence, and absence of tokens from
Hub DTOs and logs.

## Executed identity deployment check

On 2026-09-07, the checked-in deployment script bootstrapped and imported a fresh private Keycloak
26.7.3 instance using Temurin 25.0.4.1+1. A real Chromium login and HTTP callback exercised PKCE,
RSA signature/identity/audience verification, confidential introspection, refresh and logout.
All 15 checks passed, including denial of password grants, missing PKCE, an unregistered callback,
an incorrect introspection secret and an invalid bearer. Logout made introspection inactive and
invalidated refresh. The synthetic realm user was removed, Keycloak stopped normally, and ports
8080, 9000 and 8480 were confirmed closed. A separate script check accepted the current realm and
rejected a copy with the required `basic` scope removed.

This check ran the identity provider and a protocol probe. It did not execute the Rust credential
broker, Hub service or Tauri desktop, and does not satisfy the three-process acceptance above.
The exact archive digests, source hashes and check results are linked from the
[owning plan](../plans/astra/features/hub/02-local-service-authority.md).
