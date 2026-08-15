# Zircon Editor CLI

`zircon_editor` has two product launch modes: GUI startup and a typed headless commandlet.
Arguments that configure process diagnostics are removed before either mode is selected.

## Routing And Diagnostics

`zircon_editor` first parses `--log-level` and `--log-filter`, initializes its process log, and
then selects exactly one typed route: help, a commandlet result, or a GUI startup request. `--help`
and `-h` return successfully before commandlet construction, project preparation, runtime loading,
or window creation.

## GUI Startup

Use one of the following GUI intents:

- `--project <path>` opens an existing project.
- `--scene <res://path.scene.toml>` opens a scene from that requested project after its host is ready.
- `--builtin-view <descriptor-id>` opens a built-in editor view.
- `--layout <preset-id>` loads an existing layout preset after the retained host opens.
- `--create-project --project-name <name> --location <directory> --template renderable-empty` creates the minimal renderable project.

With no startup intent, the editor opens the welcome workspace. An unrecognized GUI argument is a
startup error; it is not silently ignored.

## Headless Commandlets

`--run <commandlet>` is the only product headless entry. It completes before the GUI host or
workbench is created and writes one JSON report to stdout.

Current commandlets:

- `--run plugin-list`
- `--run migrate-assets --project <project-root> --dry-run`
- `--run migrate-assets --project <project-root> --apply`
- `--run authoring-automation --project <project-root> --automation <request.json>`

`migrate-assets` requires exactly one of `--dry-run` or `--apply`. A commandlet report uses exit
code `0` for success, `1` for task failure, `2` for invalid arguments, and `3` for a missing
required capability. `authoring-automation` resolves the project and binding request through the
editor process host, then emits its retained-host evidence in the report's `automation` field.

The retired `--operation`, `--args`, `--operation-group`, `--list-operations`,
`--operation-history`, and bare `--headless` arguments are rejected. `--automation` is only a
typed argument of `--run authoring-automation`; none of these are aliases for `--run`.

## Diagnostics

Both modes accept `--log-level <level>` or `--log-level=<level>`, and `--log-filter <filter>` or
`--log-filter=<filter>`. The same diagnostic argument parser is shared with `zircon_runtime`, so
the syntax and environment precedence remain identical across product executables. Startup values
take precedence over `ZIRCON_LOG_LEVEL`; scoped filters follow the documented `ZIRCON_LOG_FILTER`,
`ZIRCON_LOG`, and `RUST_LOG` precedence.

## Runtime Preview

`zircon_runtime` is a separate executable route. Its Play startup contract accepts
`--project <path>`, `--play-scene <project-relative-path>`, and `--play-report-pipe <name>`.
`--play-scene` loads the versioned Play snapshot before the runtime event loop, while
`--play-report-pipe` selects the logical startup report outlet. The editor process-backend emits
these arguments for Play; they are not interchangeable with editor GUI or `--run` arguments.
