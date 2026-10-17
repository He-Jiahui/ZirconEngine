# Codex file permissions

## Canonical configuration

`E:\Git\ZirconEngine\.codex\config.toml` defines the `zircon-strict` permission
profile and the sole maintained write allowlist. It extends `:read-only` and
grants writes only to the explicitly listed source, documentation, fixture,
skill, session, state and output paths. The checkout root is read-only; its
editable configuration files are granted individually.

Directory grants include descendants. A new file inside a granted directory
is permitted; a new sibling directory or unlisted root file is not. Reading an
external input never grants permission to move, overwrite or delete it.

Compiler products, compiler caches, scratch, exports and generated evidence use
the registered `zircon-local` namespaces under the physical drive-root
`D:\cargo-targets`, `E:\cargo-targets` or `F:\cargo-targets`. Source code in
`zircon_runtime\build` remains source, not a compiler output directory.
The existing Cargo wrapper retains physical-path checks and native locking.
The policy does not authorize clearing retired coordinator data or restoring
its services, registration, leases or validation queue.

Shell environment values route temporary, Cargo, Python, pip, npm and XDG
outputs into the E: namespace. Other tools must use their explicit output
options. A write denial is a reason to choose a permitted destination; it does
not authorize adding a grant or running the command without confinement.

## Activation

1. Open ZirconEngine as a trusted project. Codex loads project configuration
   only for trusted projects.
2. Select **zircon-strict** in the desktop permissions menu. Existing chats
   that already selected Full Access can retain that selection; the saved
   default does not retroactively replace an active turn's execution policy.
3. Use native Windows `elevated` sandboxing. If setup reports a problem, run
   the official sandbox setup and complete any Windows administrator prompt.
   Do not erase credentials, rotate sandbox accounts manually or weaken the
   profile as a workaround.
4. Verify real operations with the command below. Configuration parsing, profile
   discovery and `windowsSandbox/readiness = ready` are separate from passing
   runtime evidence.

ZirconEngine selects `zircon-strict` through its own trusted project
configuration. The user-level permission default applies to other projects;
an existing chat's explicit selection can still override this project default.

## Tool coverage

The profile controls sandboxed local commands and their child processes. MCP
servers and app tools use separate execution boundaries, so this checkout
disables its configured RenderDoc and CUA MCP servers and default app access.
Command networking is enabled. On 2026-10-16 the user explicitly authorized
opening this profile for task-scoped handoff to the verified independent Jenkins
runtime. The `.jenkins` and `.git` grants support its runtime and Git/worktree
metadata; `.codex/state/jenkins-coordinator`, `jenkins-pilot` and `jenkins-tray`
also have explicit write grants. The existing `.codex/outbox` grant covers scoped
request inputs. These grants permit the filesystem operations; the task still
determines which commands, cleanup candidates and version actions are authorized.

New directory grants need existing physical directories before the Windows
sandbox can grant access beneath a read-only parent. Permission maintenance may
initialize these exact, user-authorized empty directories, then rerun the probes;
do not grant writes to the whole `.codex/state` parent to work around creation.

The network host rules explicitly allow `127.0.0.1` and `localhost`, including the
current Jenkins endpoint at `http://127.0.0.1:18080`. These rules apply when a
network proxy is active. They do not enable the proxy; networking is direct when
no proxy is active, so this is not a localhost-only network policy. The running
Jenkins service has its own execution boundary and remains subject to its request
authorization, selected-driver/runtime checks, physical output checks and native
ownership locks. The command sandbox does not confine that preexisting service.

The configuration itself, hooks outside `.git`, AGENTS.md, unlisted reference
repositories/worktrees, retired coordinator data, user directories and system
temp have no write grant. Directory grants include all descendants, including
Git hooks inside `.git` and worktrees inside `.jenkins`; granting writes does not
authorize changing hooks or creating worktrees outside the user's task. A Codex
host's own state, logs and sandbox provisioning are outside the command profile;
this profile does not claim to confine every background write by Windows or the
desktop host.

These are project defaults, not an administrator-enforced device policy.
Explicitly selecting another profile or supplying a command-line override
changes that invocation's permissions. The working agreement prohibits using
that route to bypass this project's allowlist.

## Runtime verification

Use the installed CLI's `codex sandbox --permission-profile zircon-strict
--include-managed-config -C E:\Git\ZirconEngine -- <command>` interface for
actual file-operation and Jenkins connectivity probes. Record the configuration
hash, installed CLI identity, exact command and terminal result; register owned
canaries in the task artifact manifest before creating them and remove them after
the probe. Do not use the Jenkins service interpreter for ad hoc probe scripts.
An interpreter failure, timeout, missing directory or failed positive probe is
a failed verification. Negative probes must receive actual access denials.

An earlier allowlist passed 15 checks with Codex 0.160.0 and native Windows
`elevated` sandboxing; its historical receipt records no stderr, no forbidden output
and an unchanged configuration:
[runtime receipt](E:/cargo-targets/zircon-local/codex-permissions/probe-6151a82962894e9d9dea183eb2e5b764/receipt.json).
That result does not validate the current, expanded allowlist. The current
Jenkins handoff permission change records its own probes in
[the task receipt](E:/cargo-targets/zircon-local/reports/def-disk-cleanup-20261016/jenkins-permission-open/validation.json).
Each receipt verifies only its tested sandbox invocation; the desktop chat still
requires the profile selection described above.

Re-run after changing the allowlist, Codex version, native sandbox mode or
relevant Windows permissions. This is permission acceptance evidence, not Rust
build, Jenkins migration or whole-workspace acceptance.

## Official references

- [Permission profiles](https://learn.chatgpt.com/docs/permissions)
- [Windows sandbox](https://learn.chatgpt.com/docs/windows/windows-sandbox)
- [Configuration precedence](https://learn.chatgpt.com/docs/config-file/config-basic)
