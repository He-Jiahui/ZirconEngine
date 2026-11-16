# Codex file permissions

## Current selection: Full Access

On 2026-10-18 the user explicitly requested Full Access instead of repairing
the Codex Windows sandbox. The project configuration now selects:

```toml
approval_policy = "never"
default_permissions = ":danger-full-access"
```

The user-level configuration already selects the same built-in profile. Full
Access removes Codex command sandbox restrictions; it uses the current Windows
user's operating-system permissions and does not make that user an administrator.
It does not require elevated sandbox setup, refreshing sandbox ACLs, or changing
file owners. The retained `[windows] sandbox = "elevated"` setting is only relevant
if a restricted profile is explicitly selected later.

New sessions use the resolved default unless another explicit selection or
managed requirement overrides it. Existing chats may retain `zircon-strict` or
another older selection: choose **Full Access** in that chat's permissions menu,
then verify a real command. Do not edit the chat database or claim that changing
the project default changed every active chat. This checkout's current chat
already has unrestricted command execution; that observation alone does not
prove another chat recovered.

Agents directly manage authorized native Git/worktrees, source preparation,
generation, local validation, artifact reuse and bounded cleanup under
[the management standard](../tools/agent-artifact-management.md). Compiler
products, caches and task evidence retain the physical cargo-targets placement,
35 GiB reserve and native locks. Preserve foreign changes, unknown source/artifacts,
Git metadata and historical receipts. Full Access does not expand task scope,
commit/remote-action authority or deletion permission.

## Optional restricted profile

`E:\Git\ZirconEngine\.codex\config.toml` defines the `zircon-strict` permission
profile and its maintained write allowlist. It is retained as an optional
restricted selection, not the project default. It extends `:read-only` and
grants writes only to the explicitly listed source, documentation, fixture,
skill, session, state and output paths. Under this profile the checkout root is read-only; its
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

## Activate the optional restricted profile

These steps apply only if the user explicitly requests `zircon-strict` again.

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

ZirconEngine currently selects Full Access through its trusted project
configuration. An existing chat's explicit selection can still override that
project default.

## Optional profile coverage

The profile controls sandboxed local commands and their child processes. MCP
servers and app tools use separate execution boundaries, so this checkout
disables its configured RenderDoc and CUA MCP servers and default app access.
Command networking is enabled. Read the filesystem table for actual grants.
The optional profile lists `.git`, `.github` and historical workflow data as
read-only. Perform authorized Git or lifecycle actions only in a task checkout
where the current permissions permit them. Use `.codex/outbox` for scoped
handoff metadata, not source backups. Historical credentials and state remain protected.

New directory grants need existing physical directories before the Windows
sandbox can grant access beneath a read-only parent. Permission maintenance may
initialize these exact, user-authorized empty directories, then rerun the probes;
do not grant writes to the whole `.codex/state` parent to work around creation.

The network host rules explicitly allow `127.0.0.1` and `localhost` when a
network proxy is active. They do not enable the proxy or restrict direct networking
when it is off. This is not a localhost-only network policy, and the command
sandbox does not confine independently running host services.

Under the optional profile, unlisted control configuration, AGENTS.md, reference
trees, retired coordinator data, user directories and system temp have no write
grant. The host's own state, logs and sandbox provisioning remain outside the
command profile. A saved profile does not activate it in an existing chat; real
operation probes are still required when claiming restricted enforcement.

These are project defaults, not an administrator-enforced device policy.
Explicitly selecting another profile or supplying a command-line override
changes that invocation's permissions. The user explicitly authorized the current
Full Access selection; the optional profile's allowlist does not confine it.

## Optional restricted-profile runtime verification

Use the installed CLI's `codex sandbox --permission-profile zircon-strict
--include-managed-config -C E:\Git\ZirconEngine -- <command>` interface for
actual file-operation probes for the selected profile. Record the configuration
hash, installed CLI identity, exact command and terminal result; register owned
canaries in the task artifact manifest before creating them and remove them after
the probe. Use a general-purpose Python interpreter if a probe requires Python.
An interpreter failure, timeout, missing directory or failed positive probe is
a failed verification. Negative probes must receive actual access denials.

An earlier allowlist passed 15 checks with Codex 0.160.0 and native Windows
`elevated` sandboxing; its historical receipt records no stderr, no forbidden output
and an unchanged configuration:
[runtime receipt](E:/cargo-targets/zircon-local/codex-permissions/probe-6151a82962894e9d9dea183eb2e5b764/receipt.json).
That result does not validate the current, expanded allowlist. The historical Jenkins
handoff permission change records its own probes in
[the task receipt](E:/cargo-targets/zircon-local/reports/def-disk-cleanup-20261016/jenkins-permission-open/validation.json).
Each receipt verifies only its tested sandbox invocation; the desktop chat still
requires the profile selection described above.

Re-run after changing the allowlist, Codex version, native sandbox mode or
relevant Windows permissions. This is permission acceptance evidence, not Rust
build, migration or whole-workspace acceptance.

## Official references

- [Permission profiles](https://learn.chatgpt.com/docs/permissions)
- [Windows sandbox](https://learn.chatgpt.com/docs/windows/windows-sandbox)
- [Configuration precedence](https://learn.chatgpt.com/docs/config-file/config-basic)
