[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
if ($PSVersionTable.PSEdition -eq 'Core') {
    $windowsPowerShell = (Get-Command powershell.exe -ErrorAction Stop).Source
    & $windowsPowerShell -NoProfile -ExecutionPolicy Bypass -File $PSCommandPath
    exit $LASTEXITCODE
}
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$stager = Join-Path $repoRoot 'tools\mvp\Stage-MvpProducts.ps1'
$supervisorModule = Join-Path $repoRoot 'tools\mvp\StagedProcessSupervisor.psm1'
$journalModule = Join-Path $repoRoot 'tools\mvp\MvpProcessLifecycleJournal.psm1'
$outputCaptureModule = Join-Path $repoRoot 'tools\mvp\MvpProcessOutputCapture.psm1'
$environmentPolicyModule = Join-Path $repoRoot 'tools\mvp\MvpProcessEnvironmentPolicy.psm1'
$stageEnvironmentPolicyModule = Join-Path $repoRoot 'tools\mvp\MvpStageProcessEnvironmentPolicy.psm1'
$cancellationModule = Join-Path $repoRoot 'tools\mvp\MvpStagingCancellationRequest.psm1'
$terminalReceiptModule = Join-Path $repoRoot 'tools\mvp\MvpStagingTerminalReceipt.psm1'
$preflightModule = Join-Path $repoRoot 'tools\mvp\MvpStagingPreflight.psm1'
$productInputManifestModule = Join-Path $repoRoot 'tools\mvp\MvpProductInputManifest.psm1'
$productProfileRegistryModule = Join-Path $repoRoot 'tools\mvp\MvpProductProfileRegistry.psm1'
$buildSetModule = Join-Path $repoRoot 'tools\mvp\MvpBuildSet.psm1'
$fixturePathsModule = Join-Path $repoRoot 'tools\mvp\MvpTestFixturePaths.psm1'
$stagingTreeManifestModule = Join-Path $repoRoot 'tools\mvp\MvpAcceptanceStagingTreeManifest.psm1'
$windowsPathResolverModule = Join-Path $repoRoot 'tools\WindowsPathResolver.psm1'
Import-Module $productInputManifestModule -Force -ErrorAction Stop
Import-Module $productProfileRegistryModule -Force -ErrorAction Stop
Import-Module $buildSetModule -Force -ErrorAction Stop
Import-Module $cancellationModule -Force -ErrorAction Stop
Import-Module $fixturePathsModule -Force -ErrorAction Stop
Import-Module $stagingTreeManifestModule -Force -ErrorAction Stop
Import-Module $windowsPathResolverModule -Force -Global -ErrorAction Stop

function Assert-True {
    param($Condition, [string]$Message)

    if ($Condition -isnot [bool]) {
        $caller = @(Get-PSCallStack)[1]
        throw "Assertion condition at $($caller.ScriptName):$($caller.ScriptLineNumber) must be one Boolean, got '$($Condition.GetType().FullName)'."
    }
    if (-not $Condition) { throw $Message }
}

function Invoke-MvpStagingFixtureGit {
    param(
        [Parameter(Mandatory)][string]$GitPath,
        [Parameter(Mandatory)][string]$RepositoryRoot,
        [Parameter(Mandatory)][string[]]$Arguments
    )

    & $GitPath -C $RepositoryRoot @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "Could not create the immutable BuildSet fixture with git $($Arguments -join ' ')."
    }
}

function Remove-MvpStagingFixtureBuildSet {
    param([Parameter(Mandatory)][pscustomobject]$Fixture)

    if ($null -eq $Fixture.PSObject.Properties['BuildSet'] -or $null -eq $Fixture.BuildSet) {
        return
    }
    $git = Get-Command git.exe -ErrorAction Stop
    if (Test-Path -LiteralPath $Fixture.BuildSet.snapshot_root -PathType Container) {
        & $git.Source -C $Fixture.BuildSetSourceRoot worktree remove --force $Fixture.BuildSet.snapshot_root
        if ($LASTEXITCODE -ne 0) {
            throw "Could not release the immutable BuildSet fixture worktree '$($Fixture.BuildSet.snapshot_root)'."
        }
    }
    & $git.Source -C $Fixture.BuildSetSourceRoot worktree prune
    if ($LASTEXITCODE -ne 0) {
        throw "Could not prune the immutable BuildSet fixture source repository '$($Fixture.BuildSetSourceRoot)'."
    }
}

function Set-MvpStagingFixtureControl {
    param(
        [Parameter(Mandatory)][pscustomobject]$Fixture,
        [Parameter(Mandatory)][ValidateSet('Project', 'Template')][string]$Scope,
        [Parameter(Mandatory)][string]$Control
    )

    $controlRoot = if ($Scope -eq 'Project') {
        $Fixture.ProjectRoot
    }
    else {
        Join-Path $Fixture.TemplateRoot 'renderable-empty'
    }
    [IO.File]::WriteAllText(
        (Join-Path $controlRoot 'fixture-control.txt'),
        ($Control + [Environment]::NewLine),
        [Text.UTF8Encoding]::new($false)
    )
}

function Clear-MvpStagingFixtureControl {
    param(
        [Parameter(Mandatory)][pscustomobject]$Fixture,
        [Parameter(Mandatory)][ValidateSet('Project', 'Template')][string]$Scope
    )

    $controlRoot = if ($Scope -eq 'Project') {
        $Fixture.ProjectRoot
    }
    else {
        Join-Path $Fixture.TemplateRoot 'renderable-empty'
    }
    Remove-Item -LiteralPath (Join-Path $controlRoot 'fixture-control.txt') -Force -ErrorAction SilentlyContinue
}

function Assert-ProcessTiming {
    param(
        [Parameter(Mandatory)]$Evidence,
        [Parameter(Mandatory)][string]$Label
    )

    $startedAt = [DateTimeOffset]::MinValue
    $endedAt = [DateTimeOffset]::MinValue
    Assert-True ([DateTimeOffset]::TryParse([string]$Evidence.started_at_utc, [ref]$startedAt)) "$Label is missing a parseable started_at_utc."
    Assert-True ([DateTimeOffset]::TryParse([string]$Evidence.ended_at_utc, [ref]$endedAt)) "$Label is missing a parseable ended_at_utc."
    Assert-True ($endedAt -ge $startedAt) "$Label ended before it started."
    Assert-True ($Evidence.exit_code -eq 0) "$Label did not retain its successful exit code."
}

function Get-ProcessJournalEntries {
    param([Parameter(Mandatory)][string]$StageRoot)

    return @(Get-ProcessJournalLifecycleEntries -StageRoot $StageRoot | Where-Object {
            $_.event_kind -eq 'terminal'
        })
}

function Get-MvpStagingTerminalReceiptFixture {
    param(
        [Parameter(Mandatory)][string]$StagingRoot,
        [Parameter(Mandatory)][string]$RunId
    )

    $path = Join-Path $StagingRoot ".mvp-staging-receipts\$RunId.json"
    Assert-True (Test-Path -LiteralPath $path -PathType Leaf) "MVP staging run '$RunId' has no terminal receipt."
    return Get-Content -LiteralPath $path -Raw -Encoding UTF8 | ConvertFrom-Json
}

function Get-ProcessJournalLifecycleEntries {
    param([Parameter(Mandatory)][string]$StageRoot)

    $logRoot = Join-Path $StageRoot 'logs'
    $journalPaths = @(Get-ChildItem -LiteralPath $logRoot -Filter 'process-execution-journal*.jsonl' -File |
            Sort-Object Name)
    Assert-True ($journalPaths.Count -gt 0) "Stage did not persist a process journal under '$logRoot'."
    return @(
        $journalPaths |
            ForEach-Object { Get-Content -LiteralPath $_.FullName -Encoding UTF8 } |
            Where-Object { -not [string]::IsNullOrWhiteSpace($_) } |
            ForEach-Object { $_ | ConvertFrom-Json } |
            Sort-Object `
                @{ Expression = { [int]$_.journal_segment } }, `
                @{ Expression = { [Int64]$_.journal_offset_bytes } }
    )
}

function Assert-ProcessJournalEntry {
    param(
        [Parameter(Mandatory)]$Entry,
        [Parameter(Mandatory)][string]$Phase,
        [Parameter(Mandatory)][string]$Outcome,
        [Parameter(Mandatory)][AllowNull()][Nullable[int]]$ExitCode
    )

    Assert-True ($Entry.phase -eq $Phase) "Process journal phase differs from '$Phase'."
    Assert-True ($Entry.outcome -eq $Outcome) "Process journal outcome differs from '$Outcome'."
    Assert-True ($Entry.exit_code -eq $ExitCode) "Process journal exit code differs from '$ExitCode'."
    $startedAt = [DateTimeOffset]::MinValue
    $endedAt = [DateTimeOffset]::MinValue
    Assert-True ([DateTimeOffset]::TryParse([string]$Entry.started_at_utc, [ref]$startedAt)) "Process journal '$Phase' has no parseable start time."
    Assert-True ([DateTimeOffset]::TryParse([string]$Entry.ended_at_utc, [ref]$endedAt)) "Process journal '$Phase' has no parseable end time."
    Assert-True ($endedAt -ge $startedAt) "Process journal '$Phase' ended before it started."
}

function Assert-ProcessJournalProgress {
    param(
        [Parameter(Mandatory)][object[]]$Entries,
        [Parameter(Mandatory)][string]$Phase,
        [Parameter(Mandatory)][string[]]$ExpectedNames
    )

    $phaseEntries = @($Entries | Where-Object { $_.phase -eq $Phase })
    $progressEntries = @($phaseEntries | Where-Object { $_.event_kind -eq 'progress' })
    $exitEntries = @($phaseEntries | Where-Object { $_.event_kind -eq 'exit' })
    $terminalEntries = @($phaseEntries | Where-Object { $_.event_kind -eq 'terminal' })
    Assert-True ($progressEntries.Count -eq $ExpectedNames.Count) "Process journal '$Phase' progress count differs from $($ExpectedNames.Count)."
    for ($index = 0; $index -lt $ExpectedNames.Count; $index++) {
        Assert-True ($progressEntries[$index].progress_name -eq $ExpectedNames[$index]) "Process journal '$Phase' progress $index differs from '$($ExpectedNames[$index])'."
    }
    Assert-True ($exitEntries.Count -eq 1) "Process journal '$Phase' must contain exactly one exit after progress."
    Assert-True ($terminalEntries.Count -eq 1) "Process journal '$Phase' must contain exactly one terminal entry after progress."
    Assert-True ([array]::IndexOf($Entries, $progressEntries[-1]) -lt [array]::IndexOf($Entries, $exitEntries[0])) "Process journal '$Phase' recorded its final progress after exit."
    Assert-True ($terminalEntries[0].phase_progress.last_name -eq $ExpectedNames[-1]) "Process journal '$Phase' terminal entry lost its final progress milestone."
}

function New-MvpStagingFixture {
    $root = New-MvpTestFixtureRoot -Prefix 'zircon-mvp-staging'
    $build = Join-Path $root 'build'
    $templates = Join-Path $root 'templates\projects\renderable-empty'
    $engineAssets = Join-Path $root 'engine-assets'
    $project = Join-Path $root 'project'
    $projectAssets = Join-Path $project 'assets\scenes'
    $projectCache = Join-Path $project '.zircon\cache'
    $projectRegistry = Join-Path $project '.zircon\registry'
    New-Item -ItemType Directory -Force -Path $build, $templates, $engineAssets, $projectAssets, $projectCache, $projectRegistry | Out-Null

    $fixtureProduct = Join-Path $build 'fixture_product.exe'
    Add-Type -TypeDefinition @'
using System;
using System.Diagnostics;
using System.Drawing;
using System.Drawing.Imaging;
using System.IO;

public static class FixtureProduct
{
    private static void WriteVisibleCapture(string path, bool afterAuthoring = false)
    {
        var directory = Path.GetDirectoryName(path);
        if (!String.IsNullOrWhiteSpace(directory))
        {
            Directory.CreateDirectory(directory);
        }
        using (var capture = new Bitmap(16, 16))
        {
            for (var y = 0; y < capture.Height; y++)
            {
                for (var x = 0; x < capture.Width; x++)
                {
                    capture.SetPixel(x, y, x < 8 ? Color.Black : (
                        afterAuthoring ? Color.FromArgb(255, 48, 192, 112) : Color.FromArgb(255, 64, 128, 255)
                    ));
                }
            }
            capture.Save(path, ImageFormat.Png);
        }
    }

    private static bool HasFixtureControl(string[] args, string control)
    {
        var projectIndex = Array.IndexOf(args, "--project");
        var candidateRoots = new[]
        {
            projectIndex >= 0 && projectIndex + 1 < args.Length ? args[projectIndex + 1] : null,
            Environment.CurrentDirectory,
            Path.Combine(Environment.CurrentDirectory, "..", "templates", "renderable-empty"),
        };
        foreach (var candidateRoot in candidateRoots)
        {
            if (String.IsNullOrWhiteSpace(candidateRoot))
            {
                continue;
            }
            var controlPath = Path.Combine(Path.GetFullPath(candidateRoot), "fixture-control.txt");
            if (!File.Exists(controlPath))
            {
                continue;
            }
            foreach (var line in File.ReadAllLines(controlPath))
            {
                if (String.Equals(line.Trim(), control, StringComparison.Ordinal))
                {
                    return true;
                }
            }
        }
        return false;
    }

    public static int Main(string[] args)
    {
        if (Array.IndexOf(args, "--fixture-child") >= 0)
        {
            System.Threading.Thread.Sleep(30000);
            return 0;
        }
        var createProject = Array.IndexOf(args, "--create-project");
        if (createProject >= 0)
        {
            if (HasFixtureControl(args, "fail-create-with-child"))
            {
                using (var child = Process.Start(new ProcessStartInfo
                {
                    FileName = Process.GetCurrentProcess().MainModule.FileName,
                    Arguments = "--fixture-child",
                    UseShellExecute = true,
                }))
                {
                }
                System.Threading.Thread.Sleep(100);
                return 24;
            }
            var nameIndex = Array.IndexOf(args, "--project-name");
            var locationIndex = Array.IndexOf(args, "--location");
            if (nameIndex < 0 || locationIndex < 0 || nameIndex + 1 >= args.Length || locationIndex + 1 >= args.Length)
            {
                return 3;
            }
            var projectRoot = Path.Combine(args[locationIndex + 1], args[nameIndex + 1]);
            Directory.CreateDirectory(Path.Combine(projectRoot, "assets", "scenes"));
            File.WriteAllText(Path.Combine(projectRoot, "zircon-project.toml"), "name = \"fixture-created-project\"");
            File.WriteAllText(Path.Combine(projectRoot, "assets", "scenes", "main.scene.toml"), "format_version = 1");
            var creationCapturePath = Environment.GetEnvironmentVariable("ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG");
            var creationCaptureDiagnostic = "";
            var creationProductDiagnostic = "";
            if (!String.IsNullOrWhiteSpace(creationCapturePath) &&
                !HasFixtureControl(args, "skip-editor-capture"))
            {
                if (!HasFixtureControl(args, "skip-editor-capture-file"))
                {
                    WriteVisibleCapture(creationCapturePath);
                }
                creationCaptureDiagnostic = "editor_product_frame_capture_written" + Environment.NewLine;
                creationProductDiagnostic =
                    "editor_product_frame_diagnostics project_path=" + Uri.EscapeDataString(projectRoot) +
                    " selected_node_id=3 selected_node_name=Cube inspector_translation_x=0 inspector_translation_y=0 inspector_translation_z=0 inspector_scale_x=1.00 inspector_scale_y=1.00 inspector_scale_z=1.00" +
                    Environment.NewLine;
            }
            var creationDiagnosticRoot = Environment.GetEnvironmentVariable("ZIRCON_LOG_ROOT");
            if (!String.IsNullOrWhiteSpace(creationDiagnosticRoot))
            {
                Directory.CreateDirectory(creationDiagnosticRoot);
                var projectOpenDiagnostic = "";
                if (!HasFixtureControl(args, "skip-project-open-diagnostic"))
                {
                    projectOpenDiagnostic =
                        "editor_project_open result=completed project_root=" + Uri.EscapeDataString(projectRoot) +
                        " manifest_identity=fixture-created-project%40v1 scene_uri=res%3A%2F%2Fscenes%2Fmain.scene.toml" +
                        " registry_asset_count=4 registry_ready_asset_count=4 registry_failed_asset_count=0" +
                        " registry_diagnostic_count=0 project_generation=1 project_generation_publish_epoch=1" +
                        " catalog_asset_count=4 settings_source=persisted-v1" + Environment.NewLine;
                }
                File.WriteAllText(Path.Combine(creationDiagnosticRoot, "fixture.log"),
                    "editor_first_frame_presented" + Environment.NewLine +
                    "editor_process_teardown_complete" + Environment.NewLine +
                    creationCaptureDiagnostic +
                    creationProductDiagnostic +
                    projectOpenDiagnostic);
            }
            return 0;
        }

        var commandletIndex = Array.IndexOf(args, "--run");
        var automationIndex = Array.IndexOf(args, "--automation");
        if (commandletIndex >= 0 && commandletIndex + 1 < args.Length &&
            args[commandletIndex + 1] == "authoring-automation")
        {
            var automationProjectIndex = Array.IndexOf(args, "--project");
            if (automationProjectIndex < 0 || automationProjectIndex + 1 >= args.Length ||
                Array.IndexOf(args, "--headless") >= 0 ||
                automationIndex + 1 >= args.Length ||
                !File.Exists(args[automationIndex + 1]))
            {
                return 30;
            }
            var request = File.ReadAllText(args[automationIndex + 1]);
            var hasSelection = request.IndexOf("Hierarchy", StringComparison.Ordinal) >= 0 &&
                request.IndexOf("SelectCube", StringComparison.Ordinal) >= 0;
            var hasTransform = request.IndexOf("TransformPositionXCommit", StringComparison.Ordinal) >= 0;
            var hasScale = request.IndexOf("TransformScaleXCommit", StringComparison.Ordinal) >= 0;
            var hasSave = request.IndexOf("SaveProject", StringComparison.Ordinal) >= 0;
            if (!hasSelection || hasTransform != hasScale || hasTransform != hasSave)
            {
                return 31;
            }
            if (hasTransform && HasFixtureControl(args, "fail-automation-with-child"))
            {
                using (var child = Process.Start(new ProcessStartInfo
                {
                    FileName = Process.GetCurrentProcess().MainModule.FileName,
                    Arguments = "--fixture-child",
                    UseShellExecute = true,
                }))
                {
                }
                Console.Error.WriteLine("fixture automation failed after spawning child");
                return 32;
            }
            var reportedProjectRoot = HasFixtureControl(args, "report-wrong-authoring-project-path")
                ? Path.GetFullPath(Path.Combine(args[automationProjectIndex + 1], ".."))
                : args[automationProjectIndex + 1];
            var automationProjectIdentity = "fixture-project";
            var automationManifestPath = Path.Combine(args[automationProjectIndex + 1], "zircon-project.toml");
            if (File.Exists(automationManifestPath) && File.ReadAllText(automationManifestPath).IndexOf("fixture-created-project", StringComparison.Ordinal) >= 0)
            {
                automationProjectIdentity = "fixture-created-project";
            }
            var projectPath = reportedProjectRoot.Replace("\\", "\\\\");
            var authoredMarkerPath = Path.Combine(args[automationProjectIndex + 1], ".fixture-authored");
            if (hasTransform)
            {
                File.WriteAllText(authoredMarkerPath, "authored");
            }
            var translationX = File.Exists(authoredMarkerPath) ? "42" : "0";
            var scaleX = File.Exists(authoredMarkerPath) ? "1.25" : "1.00";
            var records = hasTransform
                ? "{\"binding_path\":\"Hierarchy/SelectCube:onClick\",\"source\":\"Cli\"}," +
                    "{\"binding_path\":\"Inspector/TransformPositionXCommit:onSubmit\",\"source\":\"Cli\",\"operation_id\":\"inspector.field.apply_batch\",\"transaction_id\":1}," +
                    "{\"binding_path\":\"Inspector/TransformScaleXCommit:onSubmit\",\"source\":\"Cli\",\"operation_id\":\"inspector.field.apply_batch\",\"transaction_id\":2}," +
                    "{\"binding_path\":\"WorkbenchMenuBar/Undo:onClick\",\"source\":\"Cli\",\"operation_id\":\"edit.history.undo\"}," +
                    "{\"binding_path\":\"WorkbenchMenuBar/Redo:onClick\",\"source\":\"Cli\",\"operation_id\":\"edit.history.redo\"}," +
                    "{\"binding_path\":\"WorkbenchMenuBar/SaveProject:onClick\",\"source\":\"Cli\",\"operation_id\":\"file.project.save\",\"save_generation\":2}"
                : "{\"binding_path\":\"Hierarchy/SelectCube:onClick\",\"source\":\"Cli\"}";
            Console.WriteLine(
                "{\"command\":\"authoring-automation\",\"status\":\"succeeded\",\"exit_code\":0,\"migration\":null,\"plugins\":null,\"automation\":{\"project_path\":\"" + projectPath +
                "\",\"project_identity\":\"" + automationProjectIdentity +
                "\",\"manifest_identity\":\"" + automationProjectIdentity + "@v1" +
                "\",\"scene_uri\":\"res://scenes/main.scene.toml\"" +
                ",\"selected_model_resource_id\":\"fixture-cube-model-resource\"" +
                ",\"selected_material_resource_id\":\"fixture-default-material-resource\"" +
                ",\"opened_project_inspection_generation\":1,\"records\":[" + records + "]," +
                "\"snapshot\":{\"project_open\":true,\"scene_entry_count\":3,\"selected_node_id\":3,\"selected_node_name\":\"Cube\",\"inspector_translation\":[\"" + translationX + "\",\"0\",\"0\"],\"inspector_scale\":[\"" + scaleX + "\",\"1.00\",\"1.00\"]}}}"
            );
            var automationDiagnosticRoot = Environment.GetEnvironmentVariable("ZIRCON_LOG_ROOT");
            if (!String.IsNullOrWhiteSpace(automationDiagnosticRoot))
            {
                Directory.CreateDirectory(automationDiagnosticRoot);
                File.WriteAllText(Path.Combine(automationDiagnosticRoot, "fixture.log"),
                    "editor_authoring_trace result=completed" + Environment.NewLine);
            }
            return 0;
        }

        var diagnosticRoot = Environment.GetEnvironmentVariable("ZIRCON_LOG_ROOT");
        if (String.IsNullOrWhiteSpace(diagnosticRoot))
        {
            return 2;
        }

        Directory.CreateDirectory(diagnosticRoot);
        if (HasFixtureControl(args, "timeout-with-child"))
        {
            using (var child = Process.Start(new ProcessStartInfo
            {
                FileName = Process.GetCurrentProcess().MainModule.FileName,
                Arguments = "--fixture-child",
                UseShellExecute = true,
            }))
            {
            }
            Console.Error.WriteLine("fixture timeout emitted before termination");
            System.Threading.Thread.Sleep(30000);
        }
        if (HasFixtureControl(args, "diagnostic-file-flood"))
        {
            for (var index = 0; index < 65; index++)
            {
                File.WriteAllText(Path.Combine(diagnosticRoot, "fixture-flood-" + index + ".log"), "fixture diagnostic flood");
            }
        }
        if (HasFixtureControl(args, "diagnostic-depth-overflow"))
        {
            var nestedDiagnosticRoot = diagnosticRoot;
            for (var index = 0; index < 9; index++)
            {
                nestedDiagnosticRoot = Path.Combine(nestedDiagnosticRoot, "nested-" + index);
            }
            Directory.CreateDirectory(nestedDiagnosticRoot);
            File.WriteAllText(Path.Combine(nestedDiagnosticRoot, "fixture-nested.log"), "fixture nested diagnostic");
        }
        var logPath = Path.Combine(diagnosticRoot, "fixture.log");
        File.AppendAllText(logPath,
            "fixture_asset_root=" + Uri.EscapeDataString(
                Environment.GetEnvironmentVariable("ZIRCON_ASSET_ROOT") ?? "") + Environment.NewLine);
        var projectIdentity = "fixture-project";
        var projectIndex = Array.IndexOf(args, "--project");
        if (projectIndex >= 0 && projectIndex + 1 < args.Length)
        {
            var manifestPath = Path.Combine(args[projectIndex + 1], "zircon-project.toml");
            if (File.Exists(manifestPath) && File.ReadAllText(manifestPath).IndexOf("fixture-created-project", StringComparison.Ordinal) >= 0)
            {
                projectIdentity = "fixture-created-project";
            }
        }
        if (Environment.GetEnvironmentVariable("ZIRCON_RUNTIME_EXIT_AFTER_FIRST_FRAME") != null)
        {
            File.AppendAllText(logPath, "runtime_first_frame_presented" + Environment.NewLine);
            File.AppendAllText(logPath, "runtime_process_teardown_complete" + Environment.NewLine);
            if (!HasFixtureControl(args, "skip-runtime-diagnostics"))
            {
                var inputEvidence = Environment.GetEnvironmentVariable("ZIRCON_RUNTIME_MVP_INPUT_PROBE") == "1"
                    ? " input_viewport_resize_count=2 input_pointer_move_count=1 input_mouse_button_press_count=1 input_mouse_button_release_count=1 input_keyboard_press_count=1 input_keyboard_release_count=1"
                    : "";
                var materialFallbackCount = HasFixtureControl(args, "material-fallback") ? "1" : "0";
                File.AppendAllText(logPath,
                    "runtime_product_frame_diagnostics frame_index=1 viewport=16x16 project_identity=" + projectIdentity + " scene_uri=res://scenes/main.scene.toml selected_model_resource_id=fixture-cube-model-resource selected_material_resource_id=fixture-default-material-resource render_backend=fixture-wgpu render_adapter=Fixture WGPU Adapter render_adapter_type=discrete_gpu device_max_bind_groups=5 device_max_texture_dimension_2d=16384 device_max_texture_array_layers=256 device_max_sampled_textures_per_shader_stage=16 device_max_storage_buffers_per_shader_stage=8 device_max_storage_buffer_binding_size=134217728 graph_executed_pass_count=1 mesh_draw_count=1 directional_light_count=1 material_fallback_count=" + materialFallbackCount + " material_validation_error_count=0" + inputEvidence +
                    Environment.NewLine);
            }
            var capturePath = Environment.GetEnvironmentVariable("ZIRCON_RUNTIME_CAPTURE_FRAME_PNG");
            if (!String.IsNullOrWhiteSpace(capturePath) &&
                !HasFixtureControl(args, "skip-runtime-capture"))
            {
                WriteVisibleCapture(capturePath);
                File.AppendAllText(logPath, "runtime_product_frame_capture_written" + Environment.NewLine);
            }
        }
        if (Environment.GetEnvironmentVariable("ZIRCON_EDITOR_EXIT_AFTER_FIRST_FRAME") != null &&
            Environment.GetEnvironmentVariable("ZIRCON_RUNTIME_MVP_INPUT_PROBE") != null)
        {
            File.AppendAllText(logPath, "editor_runtime_input_probe_leaked" + Environment.NewLine);
            return 29;
        }
        if (Environment.GetEnvironmentVariable("ZIRCON_EDITOR_EXIT_AFTER_FIRST_FRAME") != null)
        {
            File.AppendAllText(logPath, "editor_first_frame_presented" + Environment.NewLine);
            File.AppendAllText(logPath, "editor_process_teardown_complete" + Environment.NewLine);
            var editorCapturePath = Environment.GetEnvironmentVariable("ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG");
            if (!String.IsNullOrWhiteSpace(editorCapturePath) &&
                !HasFixtureControl(args, "skip-editor-capture"))
            {
                if (!HasFixtureControl(args, "skip-editor-capture-file"))
                {
                    WriteVisibleCapture(editorCapturePath, true);
                }
                File.AppendAllText(logPath, "editor_product_frame_capture_written" + Environment.NewLine);
                File.AppendAllText(logPath,
                    "editor_product_frame_diagnostics project_path=" + Uri.EscapeDataString(args[projectIndex + 1]) +
                    " selected_node_id=3 selected_node_name=Cube inspector_translation_x=42 inspector_translation_y=0 inspector_translation_z=0 inspector_scale_x=1.25 inspector_scale_y=1.00 inspector_scale_z=1.00" +
                    Environment.NewLine);
            }
        }
        if (HasFixtureControl(args, "leak-staged-child"))
        {
            using (var child = Process.Start(new ProcessStartInfo
            {
                FileName = "ping.exe",
                Arguments = "127.0.0.1 -n 30",
                UseShellExecute = false,
                CreateNoWindow = true,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
            }))
            {
                File.WriteAllText(Path.Combine(diagnosticRoot, "leaked-child.pid"), child.Id.ToString());
            }
        }
        if (HasFixtureControl(args, "leak-external-child") &&
            Environment.GetEnvironmentVariable("ZIRCON_RUNTIME_EXIT_AFTER_FIRST_FRAME") != null)
        {
            using (var child = Process.Start(new ProcessStartInfo
            {
                FileName = "ping.exe",
                Arguments = "127.0.0.1 -n 30",
                UseShellExecute = false,
                CreateNoWindow = true,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
            }))
            {
                File.WriteAllText(Path.Combine(diagnosticRoot, "escaped-child.pid"), child.Id.ToString());
            }
        }
        if (HasFixtureControl(args, "fail-with-child"))
        {
            using (var child = Process.Start(new ProcessStartInfo
            {
                FileName = Process.GetCurrentProcess().MainModule.FileName,
                Arguments = "--fixture-child",
                UseShellExecute = true,
            }))
            {
            }
            return 23;
        }
        if (HasFixtureControl(args, "spam-process-output") &&
            Environment.GetEnvironmentVariable("ZIRCON_RUNTIME_EXIT_AFTER_FIRST_FRAME") != null)
        {
            Console.Out.Write(new string('o', 8192));
            Console.Error.Write(new string('e', 8192));
        }
        return 0;
    }
}
'@ -ReferencedAssemblies 'System.Drawing' -OutputAssembly $fixtureProduct -OutputType ConsoleApplication
    Copy-Item -LiteralPath $fixtureProduct -Destination (Join-Path $build 'zircon_runtime.exe')
    Copy-Item -LiteralPath $fixtureProduct -Destination (Join-Path $build 'zircon_editor.exe')
    [IO.File]::WriteAllText((Join-Path $build 'zircon_runtime.dll'), 'fixture-zircon_runtime.dll', [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText((Join-Path $build 'zircon_runtime_editor.dll'), 'fixture-editor-runtime', [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText((Join-Path $templates 'project.toml'), 'name = "fixture"', [Text.UTF8Encoding]::new($false))
    $editorAsset = Join-Path $engineAssets 'ui\editor\welcome.zui'
    $runtimeAsset = Join-Path $engineAssets 'ui\runtime\fixtures\hud_overlay.zui'
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $editorAsset), (Split-Path -Parent $runtimeAsset) | Out-Null
    [IO.File]::WriteAllText($editorAsset, 'editor-asset', [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText($runtimeAsset, 'runtime-asset', [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText((Join-Path $project 'zircon-project.toml'), 'name = "fixture-project"', [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText((Join-Path $projectAssets 'main.scene.toml'), 'format_version = 1', [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText((Join-Path $projectCache 'stale.zasset'), 'machine cache', [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText((Join-Path $projectRegistry 'asset-registry.json'), '{"stale":true}', [Text.UTF8Encoding]::new($false))

    $fixture = [pscustomobject]@{
        Root = $root
        RuntimeExecutable = Join-Path $build 'zircon_runtime.exe'
        EditorExecutable = Join-Path $build 'zircon_editor.exe'
        RuntimeLibrary = Join-Path $build 'zircon_runtime.dll'
        EditorRuntimeLibrary = Join-Path $build 'zircon_runtime_editor.dll'
        TemplateRoot = Join-Path $root 'templates\projects'
        EngineAssetRoot = $engineAssets
        ProjectRoot = $project
        StagingRoot = Join-Path $root 'staging'
        ProductInputRoot = Join-Path $root 'product-inputs'
        BuildSetSourceRoot = Join-Path $root 'build-set-source'
        SourceFingerprint = $null
    }
    $fixtureSourceRoot = $fixture.BuildSetSourceRoot
    [IO.Directory]::CreateDirectory($fixtureSourceRoot) | Out-Null
    [IO.File]::WriteAllText((Join-Path $fixtureSourceRoot 'fixture-source.txt'), 'immutable staging fixture source', [Text.UTF8Encoding]::new($false))
    $git = Get-Command git.exe -ErrorAction Stop
    Invoke-MvpStagingFixtureGit -GitPath $git.Source -RepositoryRoot $fixtureSourceRoot -Arguments @('init', '--quiet')
    Invoke-MvpStagingFixtureGit -GitPath $git.Source -RepositoryRoot $fixtureSourceRoot -Arguments @('config', 'user.email', 'zircon-fixture@example.invalid')
    Invoke-MvpStagingFixtureGit -GitPath $git.Source -RepositoryRoot $fixtureSourceRoot -Arguments @('config', 'user.name', 'Zircon fixture')
    Invoke-MvpStagingFixtureGit -GitPath $git.Source -RepositoryRoot $fixtureSourceRoot -Arguments @('add', '--all')
    Invoke-MvpStagingFixtureGit -GitPath $git.Source -RepositoryRoot $fixtureSourceRoot -Arguments @('commit', '--quiet', '-m', 'fixture source')
    [IO.Directory]::CreateDirectory($fixture.ProductInputRoot) | Out-Null
    $fixture | Add-Member -NotePropertyName BuildSet -NotePropertyValue (New-MvpProductBuildSet `
            -RepositoryRoot $fixtureSourceRoot `
            -BuildSetRoot (Join-Path $fixture.ProductInputRoot 'build-set'))
    $fixture.SourceFingerprint = $fixture.BuildSet.build_set_id
    $fixture | Add-Member -NotePropertyName ProductInputManifest -NotePropertyValue (New-MvpProductInputManifestFixture -Fixture $fixture)
    return $fixture
}

function New-MvpProductInputManifestFixture {
    param([Parameter(Mandatory)][pscustomobject]$Fixture)

    # Stage imports replace these modules under Windows PowerShell 5.1; keep each
    # independently rebuilt fixture bound to fresh module exports.
    Import-Module $productInputManifestModule -Force -ErrorAction Stop
    Import-Module $productProfileRegistryModule -Force -ErrorAction Stop
    Import-Module $windowsPathResolverModule -Force -ErrorAction Stop

    $pathsByLogicalId = @{
        'runtime-executable' = $Fixture.RuntimeExecutable
        'runtime-library/runtime' = $Fixture.RuntimeLibrary
        'editor-executable' = $Fixture.EditorExecutable
        'runtime-library/editor' = $Fixture.EditorRuntimeLibrary
    }
    $artifacts = foreach ($specification in Get-MvpProductInputSpecifications) {
        $path = $pathsByLogicalId[$specification.logical_id]
        $resolution = Resolve-ZirconWindowsPath -Path $path
        [ordered]@{
            LogicalId = $specification.logical_id
            Package = $specification.package
            Bin = $specification.bin
            Features = $specification.features
            OutputGroup = $specification.output_group
            ArtifactName = $specification.artifact_name
            Path = $resolution.DisplayPath
            Bytes = [IO.FileInfo]::new($resolution.OperationalPath).Length
            Sha256 = Get-MvpProductInputFileSha256 -Path $resolution.OperationalPath
        }
    }
    $manifestPath = Join-Path $Fixture.ProductInputRoot 'mvp-product-inputs.json'
    $manifest = [ordered]@{
        schema_version = 2
        generated_at_utc = [DateTimeOffset]::UtcNow.ToString('o')
        source_fingerprint = $Fixture.SourceFingerprint
        product_profile_registry = (Get-MvpProductProfileRegistrySnapshot).receipt
        build_set = [ordered]@{
            build_set_id = $Fixture.BuildSet.build_set_id
            git_revision = $Fixture.BuildSet.git_revision
            dirty_overlay_sha256 = $Fixture.BuildSet.dirty_overlay_sha256
            manifest_relative_path = 'build-set/build-set.json'
        }
        artifact_output_directory = (Resolve-ZirconWindowsPath -Path (Split-Path -Parent $manifestPath)).DisplayPath
        artifacts = @($artifacts)
    }
    [IO.File]::WriteAllText($manifestPath, ($manifest | ConvertTo-Json -Depth 5), [Text.UTF8Encoding]::new($false))
    return $manifestPath
}

function Invoke-MvpStager {
    param(
        [pscustomobject]$Fixture,
        [string]$RunId = 'fixture-run'
    )

    return & $stager `
        -ProductInputManifest $Fixture.ProductInputManifest `
        -TemplateRoot $Fixture.TemplateRoot `
        -EngineAssetRoot $Fixture.EngineAssetRoot `
        -ProjectRoot $Fixture.ProjectRoot `
        -StagingRoot $Fixture.StagingRoot `
        -RunId $RunId `
        -NoLaunch `
        -AllowUnsafeStagingRoot
}

$stagerSource = Get-Content -LiteralPath $stager -Raw -Encoding UTF8
$supervisorSource = Get-Content -LiteralPath $supervisorModule -Raw -Encoding UTF8
$journalSource = Get-Content -LiteralPath $journalModule -Raw -Encoding UTF8
$outputCaptureSource = Get-Content -LiteralPath $outputCaptureModule -Raw -Encoding UTF8
$environmentPolicySource = Get-Content -LiteralPath $environmentPolicyModule -Raw -Encoding UTF8
$stageEnvironmentPolicySource = Get-Content -LiteralPath $stageEnvironmentPolicyModule -Raw -Encoding UTF8
$terminalReceiptSource = Get-Content -LiteralPath $terminalReceiptModule -Raw -Encoding UTF8
$preflightSource = Get-Content -LiteralPath $preflightModule -Raw -Encoding UTF8
$productInputManifestSource = Get-Content -LiteralPath $productInputManifestModule -Raw -Encoding UTF8
$projectOpenEvidenceSource = Get-Content -LiteralPath (Join-Path $repoRoot 'tools\mvp\MvpProjectOpenEvidence.psm1') -Raw -Encoding UTF8
Assert-True ($stagerSource -notmatch '`\$') 'MVP staging diagnostics must interpolate their input values.'
Assert-True ($stagerSource -match 'WindowsPathResolver\.psm1') 'MVP staging must import the shared Windows final-path resolver.'
Assert-True ($stagerSource -match 'tools\\WindowsPathResolver\.psm1') 'MVP staging must import the shared resolver from tracked tools source.'
Assert-True ($stagerSource -match 'MvpProductInputManifest\.psm1') 'MVP staging must import the source-bound product input manifest boundary.'
Assert-True ($stagerSource -match 'MvpBuildSet\.psm1') 'MVP staging must import the immutable BuildSet receipt boundary.'
Assert-True ($stagerSource -match 'Resolve-MvpProductInputManifest -Path \$ProductInputManifest') 'MVP staging must resolve product binaries from their signed input manifest.'
Assert-True ($stagerSource -match 'Assert-MvpProductBuildSet -ManifestPath \$buildSetManifestPath') 'MVP staging must validate the published BuildSet receipt before consuming product artifacts.'
Assert-True ($stagerSource -match 'product_input_manifest = \$productInputManifestEvidence') 'MVP staging must retain immutable product-input manifest evidence.'
Assert-True ($stagerSource -match "-LogicalId 'product-input-manifest'") 'MVP staging must copy the original product-input manifest into the staged product.'
Assert-True ($stagerSource -match "-TargetRelativePath 'build\\mvp-product-inputs.json'") 'MVP staging must give the staged product-input manifest a canonical relative path.'
Assert-True ($stagerSource -match '-ExpectedBytes \(\[Int64\]\$productInputs\.bytes\)') 'MVP staging must reject a product-input manifest replaced after resolution.'
Assert-True ($stagerSource -match '-ExpectedSha256 \$productInputs\.sha256') 'MVP staging must verify the staged manifest against the digest consumed during resolution.'
Assert-True ($stagerSource -match 'function Copy-MvpStageFile') 'MVP staging must centralize staged-file identity checks.'
Assert-True ($stagerSource -match 'Target .* byte length differs from the expected product input') 'MVP staging must reject a copied product input whose bytes differ from its resolved manifest.'
Assert-True ($stagerSource -match 'Target .* SHA-256 differs from the expected product input') 'MVP staging must reject a copied product input whose hash differs from its resolved manifest.'
Assert-True ($stagerSource -notmatch '\[string\]\$RuntimeExecutable') 'MVP staging must not accept an unbound runtime executable path.'
Assert-True ($stagerSource -notmatch '\[string\]\$EditorExecutable') 'MVP staging must not accept an unbound editor executable path.'
Assert-True ($stagerSource -match 'function Assert-MvpDistinctProfileRuntimeLibraries') 'MVP staging must own the profile-specific runtime-library identity boundary.'
Assert-True ($stagerSource -match 'Get-ZirconWindowsFileIdentity -Path \$RuntimeLibraryPath') 'MVP staging must compare the runtime DLL through the shared resolver identity.'
Assert-True ($stagerSource -match 'Get-ZirconWindowsFileIdentity -Path \$EditorRuntimeLibraryPath') 'MVP staging must compare the editor DLL through the shared resolver identity.'
Assert-True ($stagerSource -match 'Resolve-ZirconWindowsPath -Path \$Path') 'MVP staging must apply approved-root policy to the resolved Windows path.'
Assert-True ($stagerSource -match 'return \$resolvedPath') 'MVP staging must return external file and directory inputs through the shared Windows final-path resolver.'
Assert-True ($stagerSource -match '\$resolvedPath = \(Resolve-ZirconWindowsPath -Path \$Path\)\.OperationalPath') 'MVP staging must resolve each input before checking its filesystem type.'
Assert-True ($stagerSource -match '\[IO\.File\]::Exists\(\$resolvedPath\)') 'MVP staging must validate input files through the resolver operational path.'
Assert-True ($stagerSource -match '\[IO\.Directory\]::Exists\(\$resolvedPath\)') 'MVP staging must validate input directories through the resolver operational path.'
Assert-True ($stagerSource -match 'function Get-MvpOperationalFileList') 'MVP staging must enumerate source trees through an operational-path helper.'
Assert-True ($stagerSource -match '\[IO\.Directory\]::GetFiles\(\$directory\)') 'MVP staging must enumerate source files through Windows PowerShell-compatible System.IO calls.'
Assert-True ($stagerSource -match '\[IO\.Directory\]::GetDirectories\(\$directory\)') 'MVP staging must traverse source directories through Windows PowerShell-compatible System.IO calls.'
Assert-True ($stagerSource -match '\[IO\.FileAttributes\]::ReparsePoint') 'MVP staging must not traverse reparse points after resolving source roots.'
Assert-True ($stagerSource -match 'cannot be staged because it is a reparse point') 'MVP staging must fail closed instead of silently omitting a source reparse point.'
Assert-True ($stagerSource -notmatch 'Get-ChildItem -LiteralPath \$engineAssetRootPath -Recurse -File') 'MVP staging must not enumerate operational source paths through the PowerShell provider.'
Assert-True ($stagerSource -match '-SourcePath \$engineAssetFile\s+`') 'MVP staging must copy engine assets from their operational string paths.'
Assert-True ($stagerSource -match '-SourcePath \$templateFile\s+`') 'MVP staging must copy templates from their operational string paths.'
Assert-True ($stagerSource -match '-SourcePath \$projectFile\s+`') 'MVP staging must copy project files from their operational string paths.'
Assert-True ($stagerSource -notmatch '-SourcePath \$engineAssetFile\.FullName') 'MVP staging must not read FileInfo members from operational engine asset paths.'
Assert-True ($stagerSource -notmatch '-SourcePath \$templateFile\.FullName') 'MVP staging must not read FileInfo members from operational template paths.'
Assert-True ($stagerSource -notmatch '-SourcePath \$projectFile\.FullName') 'MVP staging must not read FileInfo members from operational project paths.'
Assert-True ($stagerSource -match 'diagnostic_logs = @\(\$diagnosticFiles \| ForEach-Object \{ Get-MvpStagedFileEvidence -Path \$_ ') 'MVP staging must pass operational diagnostic strings directly to evidence collection.'
Assert-True ($stagerSource -notmatch 'diagnostic_logs = @\(\$diagnosticFiles \| ForEach-Object \{ Get-MvpStagedFileEvidence -Path \$_\.FullName') 'MVP staging must not treat operational diagnostic strings as FileInfo values.'
Assert-True ($stagerSource -match 'Resolve-MvpArtifactStorageRootPath') 'MVP staging must delegate approved-root authorization to the shared storage policy.'
Assert-True ($stagerSource -match 'return \$storage\.operation_path') 'MVP staging must retain the policy-resolved staging root for filesystem operations.'
Assert-True ($stagerSource -notmatch '\$storageCapabilityEvidence = if \(\$AllowUnsafeStagingRoot\)') 'Unsafe test namespace admission must not bypass physical storage capability evidence.'
Assert-True ($stagerSource -match 'function Test-MvpStagingDirectoryReleased') 'MVP staging must retain a directory-release probe after supervised processes reach terminal state.'
Assert-True ($stagerSource -match 'Move-ZirconWindowsPath -Source \$StageDirectory -Destination \$probe') 'MVP staging must test release by moving the physical staging directory through the shared Windows path boundary.'
Assert-True ($stagerSource -match '\$resolvedRoot = \(Resolve-ZirconWindowsPath -Path \$Root\)\.OperationalPath') 'MVP staging must derive staged-file containment from the resolver operational path.'
Assert-True ($stagerSource -match '\$resolvedPath = \(Resolve-ZirconWindowsPath -Path \$Path\)\.OperationalPath') 'MVP staging must derive staged-file identity from the resolver operational path.'
Assert-True ($stagerSource -match '\$createProjectLocation = Join-ZirconWindowsPath -Path \$stageDirectory -ChildPath ''project''') 'MVP staging must establish one physical creation parent inside the stage.'
Assert-True ($stagerSource -match '\[IO\.Directory\]::CreateDirectory\(\$createProjectLocation\)') 'MVP staging must create the project working directory before launching the editor.'
Assert-True ($stagerSource -match '-WorkingDirectory \$createProjectLocation') 'MVP staging must make the creation parent the editor working directory.'
Assert-True ($stagerSource -match "'--location', '\.', '--template', 'renderable-empty'") 'MVP staging must create a project through the portable --location . contract.'
Assert-True ($stagerSource -notmatch '\$createProjectLocationArgument') 'MVP staging must not pass an absolute creation location to the product CLI.'
Assert-True ($stagerSource -match '\$createdProjectParentResolution = Resolve-ZirconWindowsPath -Path \(Join-ZirconWindowsPath -Path \$stageDirectory -ChildPath ''project''\)') 'MVP staging must resolve the created-project parent through the shared Windows final-path resolver.'
Assert-True ($stagerSource -match '(?s)\$createdProjectExpectedResolution = Resolve-ZirconWindowsPath -Path \(Join-ZirconWindowsPath\s+`\s+-Path \$createdProjectParentResolution\.OperationalPath\s+`\s+-ChildPath \$ProjectName\)') 'MVP staging must derive the created-project identity from the resolved parent, not a caller-specific Windows path form.'
Assert-True ($stagerSource -match '\$createdProjectExpectedRoot = \$createdProjectExpectedResolution\.OperationalPath') 'MVP staging must compare the created-project root using the resolver operational path.'
Assert-True ($projectOpenEvidenceSource -match '\$resolvedStagingRoot = \$stagingResolution\.OperationalPath') 'MVP project-open evidence must derive staging containment from the resolver operational path.'
Assert-True ($projectOpenEvidenceSource -match '\$resolvedProjectRoot = \$effectiveProjectResolution\.OperationalPath') 'MVP project-open evidence must derive project identity from the effective resolver operational path.'
Assert-True ($stagerSource -match 'could not launch from') 'MVP staging launch failures must identify the staged executable path.'
Assert-True ($stagerSource -match 'first_frame_exit_requested') 'MVP staging must record that each product used the first-frame exit path.'
Assert-True ($stagerSource -match 'ZIRCON_LOG_ROOT') 'MVP staging must isolate product diagnostics under the stage directory.'
Assert-True ($stagerSource -match 'ZIRCON_LOG_FILTER') 'MVP staging must override inherited host log filtering for product evidence.'
Assert-True ($stagerSource -match 'ZIRCON_ASSET_ROOT') 'MVP staging must force products to resolve staged engine assets.'
Assert-True ($stagerSource -match "ZIRCON_ASSET_ROOT = 'assets'") 'MVP staging must pass the staged asset root as the product-relative assets request.'
Assert-True ($stagerSource -notmatch 'ZIRCON_ASSET_ROOT\s*=\s*\(Join-ZirconWindowsPath') 'MVP staging must not persist an absolute staged asset root into product configuration.'
Assert-True ($stagerSource -notmatch "'ZIRCON_ASSET_ROOT',\s*\r?\n\s*'ZIRCON_LOG_ROOT'") 'MVP staging must preserve the product-relative asset request instead of resolving it in the driver.'
Assert-True ($stagerSource -match 'runtime_first_frame_presented') 'MVP staging must verify the runtime first-presented-frame diagnostic from its log files.'
Assert-True ($stagerSource -match 'editor_first_frame_presented') 'MVP staging must verify the editor first-presented-frame diagnostic from its log files.'
Assert-True ($stagerSource -match 'runtime_process_teardown_complete') 'MVP staging must verify runtime teardown after the first presented frame.'
Assert-True ($stagerSource -match 'editor_process_teardown_complete') 'MVP staging must verify editor teardown after the first presented frame.'
Assert-True ([regex]::Matches($stagerSource, '-ProgressInactivityTimeoutSeconds \$ExecutionPolicy\.progress_inactivity_timeout_seconds').Count -eq 2) 'MVP staged product and automation helpers must consume their resolved semantic-progress policy.'
Assert-True ([regex]::Matches($stagerSource, '-ProgressInactivityTimeoutSeconds \$createExecutionPolicy\.progress_inactivity_timeout_seconds').Count -eq 1) 'MVP project creation must consume its resolved semantic-progress policy.'
Assert-True ($stagerSource -notmatch '-ProgressInactivityTimeoutSeconds \$ProgressInactivityTimeoutSeconds') 'MVP staging must not pass one global inactivity timeout directly to supervised processes.'
Assert-True ([regex]::Matches($stagerSource, 'Resolve-MvpScenarioExecutionPolicy').Count -ge 5) 'MVP staging must resolve execution policy independently for every registered scenario.'
Assert-True ($stagerSource -match 'scenario_execution_policies = \$scenarioExecutionPolicyReceipts') 'MVP staging must receipt the resolved scenario execution policies.'
Assert-True ($stagerSource -match 'ZIRCON_RUNTIME_CAPTURE_FRAME_PNG') 'MVP staging must request runtime first-frame PNG evidence only for the staged runtime product.'
Assert-True ($stagerSource -match 'ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG') 'MVP staging must request a native editor first-frame PNG only for the selected staged editor run.'
Assert-True ($stagerSource -match 'ZIRCON_RUNTIME_MVP_INPUT_PROBE') 'MVP staging must request the runtime host input probe before first-frame evidence.'
Assert-True ($stagerSource -match 'Get-MvpRuntimeFrameCaptureEvidence') 'MVP staging must inspect the captured runtime PNG rather than only checking its path.'
Assert-True ($stagerSource -match 'Get-MvpEditorWindowCaptureEvidence') 'MVP staging must inspect the captured editor window PNG rather than only checking its path.'
Assert-True ($stagerSource -match 'System\.IO\.FileStream\(\s*\r?\n\s*path,\s*\r?\n\s*System\.IO\.FileMode\.Open') 'MVP staging PNG evidence must open resolver paths through a fully qualified System.IO file stream.'
Assert-True ($stagerSource -match '\$pngEvidenceReferences\s*=\s*@\(') 'MVP staging must compile its PNG evidence helper with an explicit assembly reference collection.'
Assert-True ($stagerSource -match '\[Security\.Cryptography\.SHA256\]\.Assembly\.Location') 'MVP staging must include the SHA-256 assembly when compiling its PNG evidence helper.'
Assert-True ($stagerSource -match '-ReferencedAssemblies \$pngEvidenceReferences -ErrorAction Stop') 'MVP staging must pass the complete PNG evidence assembly reference collection to Add-Type.'
Assert-True ($stagerSource -match 'non_background_pixels') 'MVP staging must record captured runtime PNG pixel evidence.'
Assert-True ($stagerSource -match 'runtime_product_frame_capture_written') 'MVP staging must require the runtime capture completion diagnostic.'
Assert-True ($stagerSource -match 'editor_product_frame_capture_written') 'MVP staging must require the editor capture completion diagnostic.'
Assert-True ($stagerSource -match 'editor-before-edit.png') 'MVP staging must preserve the created-project editor screenshot before authoring.'
Assert-True ($stagerSource -match 'editor-after-reopen.png') 'MVP staging must preserve the reopened editor screenshot after authoring.'
Assert-True ($stagerSource -match 'Get-MvpRuntimeProductDiagnosticsEvidence') 'MVP staging must parse runtime product diagnostics from the staged product logs.'
Assert-True ($stagerSource -match 'graph_executed_pass_count') 'MVP staging must require runtime graph-pass evidence.'
Assert-True ($stagerSource -match 'mesh_draw_count') 'MVP staging must require runtime mesh-draw evidence.'
Assert-True ($stagerSource -match 'directional_light_count') 'MVP staging must require runtime light evidence.'
Assert-True ($stagerSource -match 'material_fallback_count') 'MVP staging must reject runtime material fallback usage.'
Assert-True ($stagerSource -match 'material_validation_error_count') 'MVP staging must reject runtime material validation failures.'
Assert-True ($stagerSource -match 'input_pointer_move_count') 'MVP staging must preserve runtime pointer-input evidence.'
Assert-True ($stagerSource -match 'input_mouse_button_press_count') 'MVP staging must preserve runtime mouse-button input evidence.'
Assert-True ($stagerSource -match 'input_keyboard_press_count') 'MVP staging must preserve runtime keyboard input evidence.'
Assert-True ($stagerSource -match 'teardown_complete') 'MVP staging must record successful product teardown in its structured result.'
Assert-True ($stagerSource -notmatch '\[IO\.Path\]::GetRelativePath') 'MVP staging must remain compatible with Windows PowerShell hosts that lack Path.GetRelativePath.'
Assert-True ($stagerSource -match 'ProcessStartInfo') 'MVP staging must launch products through the host-compatible process API.'
Assert-True ($stagerSource -notmatch 'Get-Command Start-Process') 'MVP staging must not require the PowerShell 7-only Start-Process Environment parameter.'
Assert-True ($stagerSource -match 'StagedProcessSupervisor\.psm1') 'MVP staging must import its dedicated process-supervisor boundary.'
Assert-True ($stagerSource -match 'MvpStageProcessEnvironmentPolicy\.psm1') 'MVP staging must import its scenario environment-policy registry.'
Assert-True ($stagerSource -match 'MvpStagingTerminalReceipt\.psm1') 'MVP staging must import its terminal receipt boundary.'
Assert-True ($supervisorSource -match 'RenderExtractProcessJob\.psm1') 'The staged process supervisor must import the Job Object process-containment boundary.'
Assert-True ($supervisorSource -match 'New-RenderExtractBaselineProcessJob') 'The staged process supervisor must create a Job Object before launching each product process.'
Assert-True ($supervisorSource -match 'Start-RenderExtractBaselineSuspendedProcess') 'The staged process supervisor must assign each product process to its Job Object before it runs.'
Assert-True ($supervisorSource -match 'Test-RenderExtractBaselineProcessJobEmpty') 'The staged process supervisor must reject a Job Object that retains descendants after its root product exits.'
Assert-True ($stagerSource -match 'MaxProcessLogBytes') 'MVP staging must declare a bounded process-log limit.'
Assert-True ($supervisorSource -match 'Start-RenderExtractBaselineBoundedOutputCapture') 'The staged process supervisor must stream product output directly to bounded files.'
Assert-True (($supervisorSource + $outputCaptureSource) -notmatch 'ReadToEndAsync\(\)') 'The staged process path must not retain entire product stdout or stderr in memory.'
Assert-True ($journalSource -match 'dropped_bytes') 'The staged process journal must retain output truncation evidence.'
Assert-True ($supervisorSource -match 'MvpSupervisorMaximumTailOutputBytes') 'The staged process supervisor must impose a fixed shared tail-output byte ceiling.'
Assert-True ($supervisorSource -match 'TailOutputPath') 'The staged process supervisor must create a separate bounded tail artifact for each output stream.'
Assert-True ($journalSource -match 'tail_retained_bytes') 'The staged process journal must retain tail-artifact byte evidence.'
Assert-True ($supervisorSource -match 'MvpSupervisorMaximumJournalBytes') 'The staged process supervisor must impose a fixed process-journal byte ceiling.'
Assert-True ($journalSource -match 'journal_segment') 'The staged process journal must retain a stable rotation segment cursor.'
Assert-True ($journalSource -match 'journal_offset_bytes') 'The staged process journal must retain a byte-offset tail cursor.'
Assert-True ($supervisorSource -match 'function Get-MvpSupervisedJournalTail') 'The staged process supervisor must expose a bounded journal-tail reader.'
Assert-True ($supervisorSource -match 'function Get-MvpSupervisedBoundedTailText') 'The staged process supervisor must own the bounded output-summary reader.'
Assert-True ($supervisorSource -match 'Seek\(-\$bytesToRead') 'The staged process supervisor summary reader must seek only a bounded tail window.'
Assert-True ($stagerSource -match 'Get-MvpSupervisedBoundedTailText -Path \$summaryPath') 'MVP staging failure summaries must prefer the bounded tail artifact over full log reads.'
Assert-True ($supervisorSource -match 'function Get-MvpSupervisedBoundedDiagnosticText') 'The staged process supervisor must own bounded diagnostic aggregation.'
Assert-True ($stagerSource -match 'Get-MvpSupervisedBoundedDiagnosticText -Paths \$diagnosticFiles') 'MVP staging product checks must use bounded diagnostic aggregation.'
Assert-True ($supervisorSource -match 'Write-MvpSupervisedProcessHeartbeat') 'The staged process supervisor must emit explicit liveness heartbeats.'
Assert-True ($supervisorSource -match "EventKind 'heartbeat'") 'The staged process supervisor heartbeat must be an append-only journal event.'
Assert-True ($supervisorSource -match "EventKind 'exit'") 'The staged process supervisor must journal root-process exit before terminal evidence.'
Assert-True ($supervisorSource -match "EventKind 'cleanup'") 'The staged process supervisor must journal process-tree cleanup before terminal evidence.'
Assert-True ($supervisorSource -match 'run_id') 'The staged process supervisor journal must bind every lifecycle event to the staging run id.'
Assert-True ($supervisorSource -match 'executable_sha256') 'The staged process supervisor journal must bind every lifecycle event to its executable hash.'
Assert-True ($supervisorSource -match 'arguments_sha256') 'The staged process supervisor journal must bind every lifecycle event to its argument digest.'
Assert-True ($supervisorSource -match 'environment_sha256') 'The staged process supervisor journal must bind every lifecycle event to its environment digest.'
Assert-True ($supervisorSource -match 'environment_policy_id') 'The staged process supervisor journal must bind every lifecycle event to its scenario environment policy.'
Assert-True ($environmentPolicySource -match 'EnvironmentVariables\.Clear\(\)') 'The process environment policy must clear inherited environment before applying its allowlist.'
Assert-True ($environmentPolicySource -match 'MvpProcessHostEnvironmentNames') 'The process environment policy must declare the maximum host environment allowlist.'
Assert-True ($environmentPolicySource -match 'MvpProcessDeclaredEnvironmentNames') 'The process environment policy must declare the maximum product environment allowlist.'
Assert-True ($stageEnvironmentPolicySource -match 'runtime_first_frame') 'The stage environment registry must own a runtime first-frame policy.'
Assert-True ($stageEnvironmentPolicySource -match 'editor_first_frame') 'The stage environment registry must own an editor first-frame policy.'
Assert-True ($stageEnvironmentPolicySource -match 'editor_project_create') 'The stage environment registry must own an editor project-creation policy.'
Assert-True ($stageEnvironmentPolicySource -match 'editor_authoring') 'The stage environment registry must own an editor authoring policy.'
Assert-True ($stagerSource -match '-EnvironmentPolicy \$environmentPolicy') 'MVP staged products and authoring automation must pass their scenario environment policy.'
Assert-True ($stagerSource -match '-EnvironmentPolicy \$createEnvironmentPolicy') 'MVP project creation must pass its scenario environment policy.'
Assert-True ($terminalReceiptSource -match '\[IO\.FileMode\]::CreateNew') 'MVP staging terminal receipts must use exclusive temporary files.'
Assert-True ($terminalReceiptSource -match '\[IO\.File\]::Move\(\$temporaryPath, \$path\)') 'MVP staging terminal receipts must publish through an atomic same-directory move.'
Assert-True ($terminalReceiptSource -match 'MvpStagingTerminalReceiptMaximumBytes = 16384') 'MVP staging terminal receipts must enforce a fixed byte ceiling.'
Assert-True ($stagerSource -match "-Outcome 'succeeded'") 'MVP staging must publish a successful terminal receipt.'
Assert-True ($stagerSource -match '-Outcome \$terminalOutcome') 'MVP staging must publish failed, timed-out, or cancelled terminal outcomes.'
Assert-True ($supervisorSource -match 'environment_variables') 'The staged process supervisor journal must retain environment provenance records.'
Assert-True ($journalSource -match 'previous_event_sha256') 'The staged process journal must chain events to their predecessor hash.'
Assert-True ($journalSource -match 'event_sha256') 'The staged process journal must hash each event payload.'
Assert-True ($stagerSource -match '\$stagedProductRoot = \(Resolve-ZirconWindowsPath -Path \$StageRoot\)\.OperationalPath') 'MVP staging must resolve the process-tree root through the Windows path resolver before launch.'
Assert-True ($stagerSource -match 'staged_product_root = \$stagedProductRoot') 'MVP staging must retain the resolver operation path for process cleanup and journaling.'
Assert-True ($stagerSource -notmatch 'staged_product_root = \[IO\.Path\]::GetFullPath\(\$StageRoot\)') 'MVP staging must not derive process cleanup identity through lexical GetFullPath.'
Assert-True ($stagerSource -match '\$executableResolution = Resolve-ZirconWindowsPath -Path \$ExecutablePath') 'MVP staging must resolve the executable through the Windows path resolver before launch.'
Assert-True ($stagerSource -match '\$workingDirectoryResolution = Resolve-ZirconWindowsPath -Path \$WorkingDirectory') 'MVP staging must resolve the working directory through the Windows path resolver before launch.'
Assert-True ($stagerSource -match '\$startInfo\.FileName = \$executableResolution\.OperationalPath') 'MVP staging must launch the executable through the resolver operational path.'
Assert-True ($stagerSource -match '\$projectRootResolution = if \(\[string\]::IsNullOrWhiteSpace\(\$ProjectRoot\)\)') 'MVP staging must resolve a provided project root once before product launch.'
Assert-True ($stagerSource -match '\$startInfo\.WorkingDirectory = if \(\$null -eq \$projectRootResolution\)') 'MVP staging must use the project root as the child working directory when a project is selected.'
Assert-True ($stagerSource -match '\$workingDirectoryResolution\.DisplayPath' -and $stagerSource -match '\$projectRootResolution\.DisplayPath') 'MVP staging must cross the child cwd boundary with canonical display paths.'
Assert-True ($stagerSource.Contains("@('--project', '.') + @(`$Arguments)")) 'MVP staging must pass the selected project through the portable --project . contract.'
Assert-True ($stagerSource -notmatch 'Assert-MvpStagingProcessesReleased') 'MVP staging must not repeat process-tree cleanup after the supervised Job has reached its terminal state.'
Assert-True ($stagerSource -notmatch 'Get-CimInstance Win32_Process') 'MVP staging production code must not scan the machine-wide process table after each product attempt.'
Assert-True ($stagerSource -notmatch 'taskkill\.exe') 'MVP staging production code must not bypass the supervised Job with taskkill.'
Assert-True ($stagerSource -match 'Test-MvpStagingDirectoryReleased -StageDirectory \$stageDirectory') 'MVP staging must retain the final directory rename probe for residual file handles.'
Assert-True ($supervisorSource -match 'Wait-RenderExtractBaselineProcessJobEmpty' -and $supervisorSource -match 'Test-RenderExtractBaselineProcessJobEmpty') 'The staged process supervisor must wait for and verify an empty Job before returning.'
Assert-True ($supervisorSource -match 'Stop-RenderExtractBaselineProcessJob -Job \$processJob') 'The staged process supervisor must terminate timed-out descendants through the Job Object.'
Assert-True ($stagerSource -notmatch 'Stop-MvpTimedOutStagedProcessTree') 'MVP staging timeout cleanup must not fall back to executable-path process-tree discovery.'
Assert-True ($supervisorSource -match '\$terminationCleanupErrors\.Add\(\$_\.Exception\.Message\)') 'The staged process supervisor must retain termination cleanup failures until after process stream collection.'
$stderrCaptureIndex = $supervisorSource.IndexOf('Start-RenderExtractBaselineBoundedOutputCapture', [StringComparison]::Ordinal)
$timeoutThrowIndex = $supervisorSource.IndexOf('throw [TimeoutException]::new', [StringComparison]::Ordinal)
Assert-True ($stderrCaptureIndex -ge 0 -and $timeoutThrowIndex -gt $stderrCaptureIndex) 'The staged process supervisor must start streaming process logs before reporting a timeout or timeout-cleanup failure.'
Assert-True ($productInputManifestSource -match 'Resolve-MvpProductInputBuildSet') 'MVP product inputs must resolve immutable BuildSet provenance from the manifest.'
Assert-True ($productInputManifestSource -match '\$sourceFingerprint\.Equals\(\[string\]\$buildSet\.build_set_id') 'MVP source_fingerprint compatibility must bind exactly to BuildSetId.'
Assert-True ($productInputManifestSource -notmatch "@\('diff', '--no-ext-diff'" -and $productInputManifestSource -notmatch "@\('ls-files'" -and $productInputManifestSource -notmatch "@\('hash-object'") 'MVP product-input validation must not rescan the active Git checkout.'
Assert-True ($stagerSource -match 'function Get-FileSha256') 'MVP staging must hash files without a PowerShell module auto-load dependency.'
Assert-True ($stagerSource -notmatch 'Get-FileHash') 'MVP staging must not require the Get-FileHash cmdlet in the Windows PowerShell host.'
Assert-True ($stagerSource -match '\[char\[\]\]::new\(\$hashBytes\.Length \* 2\)') 'MVP staging SHA-256 output must allocate one fixed-size character buffer.'
Assert-True ($stagerSource -notmatch "ForEach-Object \{ \$_.ToString\('X2'\) \}") 'MVP staging SHA-256 output must not dispatch one PowerShell pipeline stage per digest byte.'
Assert-True ($stagerSource -notmatch '(?m)^\s*\[string\]\$Toolchain\s*[,)]') 'MVP staging must not accept caller-provided toolchain provenance.'
Assert-True ($stagerSource -notmatch '(?m)^\s*\[string\]\$Target\s*[,)]') 'MVP staging must not accept caller-provided target provenance.'
Assert-True ($stagerSource -match 'rustc -Vv') 'MVP staging must record toolchain provenance from the active Rust compiler.'
Assert-True ($stagerSource -match 'MvpStagingPreflight\.psm1') 'MVP staging must import its dedicated environment preflight boundary.'
Assert-True ($preflightSource -match 'function Get-MvpStagingRequiredBytes') 'MVP staging must derive its disk budget from the files that will be copied.'
Assert-True ($preflightSource -match '\$file = \[IO\.FileInfo\]::new\(\$path\)') 'MVP staging preflight must create one System.IO metadata view for each resolved source file.'
Assert-True ($preflightSource -match '-not \$file\.Exists' -and $preflightSource -match '\$fileLength = \[Int64\]\$file\.Length') 'MVP staging preflight must reuse the same metadata view for existence and length validation.'
Assert-True ($preflightSource -notmatch 'Get-Item -LiteralPath \$path') 'MVP staging preflight must not send resolver operational input paths through the PowerShell provider.'
Assert-True ($preflightSource -match 'function Assert-MvpStagingDiskCapacity') 'MVP staging must reject a run before copying when its staging drive lacks capacity.'
Assert-True ($preflightSource -match 'function Assert-MvpStagingCapacityValues') 'MVP staging capacity policy must have a directly testable value boundary.'
Assert-True ($preflightSource -match 'function Get-MvpInteractiveDesktopPreflight') 'MVP staging must check the interactive desktop before launching windowed products.'
Assert-True ($preflightSource -match 'function Assert-MvpInteractiveSessionValues') 'MVP staging interactive-session policy must have a directly testable value boundary.'
Assert-True ($preflightSource -match 'function Assert-MvpAttachedDisplayCount') 'MVP staging monitor policy must have a directly testable value boundary.'
Assert-True ($preflightSource -match 'function Assert-MvpStagingEntryBudget') 'MVP staging must verify copied manifest entries against the preflight byte budget.'
$preflightIndex = $stagerSource.IndexOf('$preflight = Get-MvpStagingPreflight', [StringComparison]::Ordinal)
$partialDirectoryCreateIndex = $stagerSource.IndexOf('[IO.Directory]::CreateDirectory($partialDirectory)', [StringComparison]::Ordinal)
Assert-True ($preflightIndex -ge 0 -and $partialDirectoryCreateIndex -gt $preflightIndex) 'MVP staging must complete disk and desktop preflight before creating its partial output directory.'
Assert-True ($stagerSource -notmatch 'Get-ChildItem -LiteralPath \$diagnosticRoot') 'MVP staging must enumerate product diagnostics through the operational-path helper.'
Assert-True ($stagerSource -match 'Move-ZirconWindowsPath -Source \$partialDirectory -Destination \$stageDirectory') 'MVP staging must publish its resolved staging directory without the PowerShell provider.'
Assert-True ($stagerSource -match 'staging_root = \(Resolve-ZirconWindowsPath -Path \$stageDirectory\)\.DisplayPath') 'MVP staging results must expose a display path instead of the operational path.'
Assert-True ($stagerSource -match 'preflight = \$preflight') 'MVP staging manifest must retain the source-bound environment preflight evidence.'
$entryBudgetIndex = $stagerSource.IndexOf('$null = Assert-MvpStagingEntryBudget', [StringComparison]::Ordinal)
$manifestIndex = $stagerSource.IndexOf('$manifest = [ordered]@{', [StringComparison]::Ordinal)
Assert-True ($entryBudgetIndex -ge 0 -and $manifestIndex -gt $entryBudgetIndex) 'MVP staging must verify final entry bytes before writing its manifest.'
$preflightModuleHandle = Import-Module $preflightModule -Force -PassThru -ErrorAction Stop
$entryBudgetMismatchRejected = $false
try {
    & $preflightModuleHandle {
        Assert-MvpStagingEntryBudget `
            -Entries @([ordered]@{ size_bytes = 7 }) `
            -ExpectedInputCopyBytes 8
    } | Out-Null
}
catch {
    $entryBudgetMismatchRejected = $_.Exception.Message -match 'final entry bytes.*preflight input_copy_bytes'
}
Assert-True $entryBudgetMismatchRejected 'MVP staging did not reject final entry bytes detached from preflight input_copy_bytes.'
$insufficientCapacityRejected = $false
try {
    & $preflightModuleHandle {
        Assert-MvpStagingCapacityValues `
            -StagingRootPath 'E:\ZirconBuilds' `
            -RequiredFreeSpaceBytes 1024 `
            -AvailableFreeSpaceBytes 1023
    } | Out-Null
}
catch {
    $insufficientCapacityRejected = $_.Exception.Message -match 'requires at least 1024.*only 1023.*available'
}
Assert-True $insufficientCapacityRejected 'MVP staging did not reject an insufficient capacity observation.'
foreach ($desktopFailure in @(
    @{ user_interactive = $false; session_id = 1; monitor_count = 1; pattern = 'interactive Windows user session' },
    @{ user_interactive = $true; session_id = 0; monitor_count = 1; pattern = 'non-interactive Windows session 0' },
    @{ user_interactive = $true; session_id = 1; monitor_count = 0; pattern = 'at least one attached display' }
)) {
    $desktopFailureRejected = $false
    try {
        & $preflightModuleHandle {
            param($fixture)
            Assert-MvpInteractiveSessionValues `
                -UserInteractive $fixture.user_interactive `
                -SessionId $fixture.session_id
            Assert-MvpAttachedDisplayCount -MonitorCount $fixture.monitor_count
        } $desktopFailure | Out-Null
    }
    catch {
        $desktopFailureRejected = $_.Exception.Message -match $desktopFailure.pattern
    }
    Assert-True $desktopFailureRejected "MVP staging did not reject desktop preflight fixture '$($desktopFailure.pattern)'."
}
Assert-True ($stagerSource -match '\[switch\]\$CreateProject') 'MVP staging must expose an explicit fresh-project creation mode.'
Assert-True ($stagerSource -match 'CreateProject cannot be combined with ProjectRoot') 'MVP staging must reject a pre-existing project when staged creation is requested.'
Assert-True ($stagerSource -match 'CreateProject cannot be combined with NoLaunch') 'MVP staging must reject a project-creation request that would skip the staged editor launch.'
Assert-True ($stagerSource -match 'function Assert-MvpProjectName') 'MVP staging must validate a created project name before composing its target path.'
Assert-True ($stagerSource -match 'GetInvalidFileNameChars') 'MVP staging must reject project names that are not one filesystem directory segment.'
Assert-True ($stagerSource -match 'createdProjectExpectedRoot') 'MVP staging must bind the created project root to the expected child of stage/project.'
Assert-True ($stagerSource -match 'createdProjectExpectedResolution\.OperationalPath') 'MVP staging must compare a created project root by physical identity.'
Assert-True ($stagerSource -match 'if \(\$reportedEditorProjectPath -eq ''\.''\)') 'MVP staging must accept the explicit project-relative editor diagnostic emitted from a project-root child cwd.'
Assert-True ($stagerSource -match 'GetParent\(\$expectedEditorProjectResolution\.DisplayPath\)') 'MVP staging must resolve created-project editor diagnostics from the staged project parent.'
Assert-True ($stagerSource -match 'Resolve-ZirconWindowsPath -Path \$reportedEditorProjectPath') 'MVP staging must compare absolute editor product diagnostics by physical project identity.'
Assert-True ($stagerSource -match 'Resolve-ZirconWindowsPath -Path \$reportedProjectPath') 'MVP staging must compare authoring automation project identities through the shared resolver.'
Assert-True ($stagerSource -match 'if \(\$reportedProjectPath -eq ''\.''\)') 'MVP staging must accept the explicit project-relative authoring report emitted from a project-root child cwd.'
Assert-True ($stagerSource -match 'Expected ''\.'' or an absolute path') 'MVP staging must reject ambiguous relative authoring report paths.'
Assert-True ($stagerSource -match 'Test-MvpFullyQualifiedWindowsPath -Path \$reportedProjectPath') 'MVP staging must reject drive-relative and root-relative authoring report paths before resolving them.'
Assert-True ($stagerSource -match '\$automationRequestArgument = \(Resolve-ZirconWindowsPath -Path \$AutomationRequestPath\)\.DisplayPath') 'MVP staging must pass authoring automation requests through the display-path product CLI boundary.'
Assert-True ($stagerSource -match '''--run'', ''authoring-automation'', ''--automation'', \$automationRequestArgument') 'MVP staging must invoke authoring through the canonical commandlet while passing the resolver display path.'
Assert-True ($stagerSource -match '\$commandlet = \$reports\[0\]') 'MVP staging must validate the outer commandlet envelope before reading authoring evidence.'
Assert-True ($stagerSource -match '\$report = \$commandlet\.automation') 'MVP staging must read authoring evidence only from the typed commandlet payload.'
Assert-True ($stagerSource -match '-ProjectRoot \$ProjectRoot') 'MVP authoring automation must reuse the project-relative staged process launch contract.'
Assert-True ($stagerSource -match '\$productPathEnvironmentVariables = @\(') 'MVP staging must centralize path-bearing product environment variables.'
Assert-True ($stagerSource -match 'Resolve-ZirconWindowsPath -Path \$environmentValue\)\.DisplayPath') 'MVP staging must pass environment paths to products in display form for product-side resolution.'
$releaseSource = Get-Content -LiteralPath (Join-Path $repoRoot 'tools\mvp\MvpStagingRelease.psm1') -Raw -Encoding UTF8
Assert-True ($releaseSource -match 'Resolve-ZirconWindowsPath -Path \$ProjectDirectory\)\.OperationalPath') 'MVP staging release probes must retain the physical path for filesystem operations.'
Assert-True ($stagerSource -match 'Resolve-ZirconWindowsPath -Path \$stagedProjectRoot\)\.DisplayPath') 'MVP staging result output must expose a display path rather than a verbatim operational path.'
Assert-True ($stagerSource -notmatch '\$projectRootArgument = \(Resolve-ZirconWindowsPath -Path \$ProjectRoot\)\.DisplayPath') 'MVP staging must not pass an absolute project path to child product processes.'
Assert-True ($stagerSource -match "'--create-project', '--project-name'") 'MVP staging must create projects through the normal staged editor CLI.'
Assert-True ($stagerSource -match "'--template', 'renderable-empty'") 'MVP staging fresh-project creation must use the renderable-empty template.'
Assert-True ($stagerSource -match 'Staged created project') 'MVP staging must verify that the staged editor created the canonical project root.'
Assert-True ($stagerSource -match 'AuthoringAutomationRequest') 'MVP staging must accept a staged normal editor automation request.'
Assert-True ($stagerSource -match 'Invoke-MvpStagedAuthoringAutomation') 'MVP staging must run authoring through the normal staged editor automation CLI.'
Assert-True ($stagerSource -match 'authoring_automation') 'MVP staging must preserve the structured authoring automation report in startup evidence.'
Assert-True ($stagerSource -match 'ReopenAutomationRequest') 'MVP staging must accept a second source-bound reopen automation request.'
Assert-True ($stagerSource -match 'reopen_automation') 'MVP staging must preserve repeated reopen automation reports in startup evidence.'
Assert-True ($stagerSource -match 'AttemptOffset') 'MVP staging must allocate a non-duplicate runtime attempt number after authoring.'
Assert-True ($stagerSource -match '\$runtimeExecutionPolicy\.attempt_count -ne 2 -or \$reopenExecutionPolicy\.attempt_count -ne 2') 'MVP staging must reject a reopen sequence whose resolved policies cannot satisfy the fixed F5 repeat contract.'
Assert-True ($stagerSource -match 'Get-MvpStagedFileEvidence') 'MVP staging must hash product stdout, stderr, and diagnostic evidence files.'
Assert-True ($stagerSource -notmatch 'source_path = \$SourcePath') 'MVP staging manifest must not retain absolute source input paths in uploaded evidence.'
Assert-True ($stagerSource -match 'project_creation') 'MVP staging must record the staged editor project-creation process as structured evidence.'
Assert-True ($stagerSource -match 'Get-MvpEditorProjectOpenEvidence') 'MVP staging must parse the normal editor project-open diagnostic from the creation process.'
$projectOpenEvidenceSource = Get-Content -LiteralPath (Join-Path $repoRoot 'tools\mvp\MvpProjectOpenEvidence.psm1') -Raw -Encoding UTF8
Assert-True ($projectOpenEvidenceSource -match 'if \(\$reportedProjectRoot -eq ''\.''\)') 'MVP project-open evidence must bind the explicit current-project diagnostic to the expected project.'
Assert-True ($projectOpenEvidenceSource -match 'GetParent\(\$expectedProjectRootResolution\.DisplayPath\)') 'MVP project-open evidence must resolve created-project relative diagnostics from the staged project parent.'
Assert-True ($projectOpenEvidenceSource.Contains('[IO.Path]::IsPathRooted($reportedProjectRoot) -or $reportedProjectRoot.Contains('':'')')) 'MVP project-open evidence must reject root-relative and drive-relative diagnostics before resolving them.'
Assert-True ($stagerSource -match 'Authoring automation diagnostic log') 'MVP staging must retain diagnostic evidence for normal editor automation processes.'
Assert-True ($journalSource -match 'process-execution-journal\.jsonl') 'The staged process lifecycle owner must persist an incremental journal for every started child process.'

$defaultAuthoringAutomationPath = Join-Path $repoRoot 'tools\mvp\mvp-authoring-automation.json'
Assert-True (Test-Path -LiteralPath $defaultAuthoringAutomationPath -PathType Leaf) 'The source-bound F5 authoring automation request is missing.'
$defaultAuthoringAutomation = Get-Content -LiteralPath $defaultAuthoringAutomationPath -Raw -Encoding UTF8 | ConvertFrom-Json
Assert-True ($defaultAuthoringAutomation.bindings.Count -eq 6) 'The F5 authoring automation request must contain selection, translation, scale, undo, redo, and save bindings.'
Assert-True ($defaultAuthoringAutomation.bindings[0].path.view_id -eq 'Hierarchy') 'The F5 authoring automation request must select the renderable template cube through Hierarchy.'
Assert-True ($defaultAuthoringAutomation.bindings[1].path.control_id -eq 'TransformPositionXCommit') 'The F5 authoring automation request must commit the X transform through Inspector.'
Assert-True ($defaultAuthoringAutomation.bindings[2].path.control_id -eq 'TransformScaleXCommit') 'The F5 authoring automation request must commit the X scale through Inspector.'
Assert-True ($defaultAuthoringAutomation.bindings[3].payload.MenuAction.action_id -eq 'workbench.history.undo') 'The F5 authoring automation request must undo through the normal history action.'
Assert-True ($defaultAuthoringAutomation.bindings[4].payload.MenuAction.action_id -eq 'workbench.history.redo') 'The F5 authoring automation request must redo through the normal history action.'
Assert-True ($defaultAuthoringAutomation.bindings[5].payload.MenuAction.action_id -eq 'workbench.project.save') 'The F5 authoring automation request must persist through the normal project save action.'
$defaultReopenAutomationPath = Join-Path $repoRoot 'tools\mvp\mvp-reopen-automation.json'
Assert-True (Test-Path -LiteralPath $defaultReopenAutomationPath -PathType Leaf) 'The source-bound F5 reopen automation request is missing.'
$defaultReopenAutomation = Get-Content -LiteralPath $defaultReopenAutomationPath -Raw -Encoding UTF8 | ConvertFrom-Json
Assert-True ($defaultReopenAutomation.bindings.Count -eq 1) 'The F5 reopen automation request must contain only its normal persisted-state selection binding.'
Assert-True ($defaultReopenAutomation.bindings[0].payload.SelectionCommand.SelectSceneNode.node_id -eq 3) 'The F5 reopen automation request must select the persisted renderable template Cube identity.'

$fixture = New-MvpStagingFixture
try {
    Assert-True ($fixture.SourceFingerprint -eq $fixture.BuildSet.build_set_id) 'BuildSet-backed staging fixtures must derive their compatibility source fingerprint from the immutable BuildSet identity.'
    $stagerSource = Get-Content -LiteralPath $stager -Raw
    Assert-True (([regex]::Matches($stagerSource, '\bGet-MvpSourceFingerprint\b')).Count -eq 0) 'Staging must not rescan the active checkout for legacy source currentness.'
    $inputManifest = Get-Content -LiteralPath $fixture.ProductInputManifest -Raw -Encoding UTF8 | ConvertFrom-Json
    $inputManifest.build_set.build_set_id = '0000000000000000000000000000000000000000000000000000000000000000'
    [IO.File]::WriteAllText($fixture.ProductInputManifest, ($inputManifest | ConvertTo-Json -Depth 5), [Text.UTF8Encoding]::new($false))
    $buildSetMismatchRejected = $false
    $buildSetMismatchError = $null
    try {
        & $stager `
            -ProductInputManifest $fixture.ProductInputManifest `
            -TemplateRoot $fixture.TemplateRoot `
            -EngineAssetRoot $fixture.EngineAssetRoot `
            -StagingRoot $fixture.StagingRoot `
            -RunId 'fixture-build-set-mismatch' `
            -NoLaunch `
            -AllowUnsafeStagingRoot | Out-Null
    }
    catch {
        $buildSetMismatchError = $_.Exception.Message
        $buildSetMismatchRejected = $_.Exception.Message -match 'source_fingerprint must equal its BuildSetId'
    }
    Assert-True $buildSetMismatchRejected "Staging failed to reject the BuildSet identity mismatch. Actual result: $buildSetMismatchError"
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixture.StagingRoot 'fixture-build-set-mismatch'))) 'BuildSet mismatch rejection created a staging directory.'
    $buildSetMismatchReceipt = Get-MvpStagingTerminalReceiptFixture -StagingRoot $fixture.StagingRoot -RunId 'fixture-build-set-mismatch'
    Assert-True ($buildSetMismatchReceipt.outcome -eq 'failed') 'BuildSet mismatch did not publish a failed staging receipt.'
    Assert-True ($buildSetMismatchReceipt.phase -eq 'admission') 'BuildSet mismatch terminal receipt lost its admission phase.'
    Assert-True (-not $buildSetMismatchReceipt.staging_directory_published) 'BuildSet mismatch terminal receipt claimed a published stage directory.'

    $fixture.ProductInputManifest = New-MvpProductInputManifestFixture -Fixture $fixture
    $inputManifest = Get-Content -LiteralPath $fixture.ProductInputManifest -Raw -Encoding UTF8 | ConvertFrom-Json
    $inputManifest.PSObject.Properties.Remove('build_set')
    [IO.File]::WriteAllText($fixture.ProductInputManifest, ($inputManifest | ConvertTo-Json -Depth 5), [Text.UTF8Encoding]::new($false))
    $missingBuildSetRejected = $false
    $missingBuildSetError = $null
    try {
        & $stager `
            -ProductInputManifest $fixture.ProductInputManifest `
            -TemplateRoot $fixture.TemplateRoot `
            -EngineAssetRoot $fixture.EngineAssetRoot `
            -StagingRoot $fixture.StagingRoot `
            -RunId 'fixture-source-mismatch' `
            -NoLaunch `
            -AllowUnsafeStagingRoot | Out-Null
    }
    catch {
        $missingBuildSetError = $_.Exception.Message
        $missingBuildSetRejected = $_.Exception.Message -match 'requires a BuildSet receipt'
    }
    Assert-True $missingBuildSetRejected "Staging accepted product inputs without a BuildSet receipt. Actual result: $missingBuildSetError"
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixture.StagingRoot 'fixture-source-mismatch'))) 'Missing BuildSet rejection created a staging directory.'

    $fixture.ProductInputManifest = New-MvpProductInputManifestFixture -Fixture $fixture
    $inputManifest = Get-Content -LiteralPath $fixture.ProductInputManifest -Raw -Encoding UTF8 | ConvertFrom-Json
    $inputManifest.artifacts[0].Sha256 = '0000000000000000000000000000000000000000000000000000000000000000'
    [IO.File]::WriteAllText($fixture.ProductInputManifest, ($inputManifest | ConvertTo-Json -Depth 5), [Text.UTF8Encoding]::new($false))
    $hashDriftRejected = $false
    try {
        & $stager `
            -ProductInputManifest $fixture.ProductInputManifest `
            -TemplateRoot $fixture.TemplateRoot `
            -EngineAssetRoot $fixture.EngineAssetRoot `
            -StagingRoot $fixture.StagingRoot `
            -RunId 'fixture-hash-drift' `
            -NoLaunch `
            -AllowUnsafeStagingRoot | Out-Null
    }
    catch {
        $hashDriftRejected = $_.Exception.Message -match 'SHA-256 differs from ProductInputManifest'
    }
    Assert-True $hashDriftRejected 'Staging accepted a product artifact whose bytes drifted from its manifest hash.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixture.StagingRoot 'fixture-hash-drift'))) 'Hash-drift rejection created a staging directory.'
    $fixture.ProductInputManifest = New-MvpProductInputManifestFixture -Fixture $fixture

    $cancelledRunId = 'fixture-prelaunch-cancelled'
    $cancelRequest = Write-MvpStagingCancellationRequest `
        -StagingRoot $fixture.StagingRoot `
        -RunId $cancelledRunId `
        -Reason 'operator_requested'
    $prelaunchCancelled = $false
    try {
        & $stager `
            -ProductInputManifest $fixture.ProductInputManifest `
            -TemplateRoot $fixture.TemplateRoot `
            -EngineAssetRoot $fixture.EngineAssetRoot `
            -StagingRoot $fixture.StagingRoot `
            -RunId $cancelledRunId `
            -NoLaunch `
            -AllowUnsafeStagingRoot | Out-Null
    }
    catch [OperationCanceledException] {
        $prelaunchCancelled = $true
    }
    Assert-True $prelaunchCancelled 'A run-bound external request did not cancel staging before publication.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixture.StagingRoot $cancelledRunId))) 'Prelaunch cancellation created a staging directory.'
    Assert-True (Test-Path -LiteralPath $cancelRequest.path -PathType Leaf) 'Prelaunch cancellation removed its immutable request evidence.'
    $prelaunchCancellationReceipt = Get-MvpStagingTerminalReceiptFixture -StagingRoot $fixture.StagingRoot -RunId $cancelledRunId
    Assert-True ($prelaunchCancellationReceipt.outcome -eq 'cancelled') 'Prelaunch cancellation did not publish a cancelled terminal outcome.'
    Assert-True ($prelaunchCancellationReceipt.phase -eq 'admission') 'Prelaunch cancellation terminal receipt lost its admission phase.'
    Assert-True (-not $prelaunchCancellationReceipt.staging_directory_published) 'Prelaunch cancellation claimed a published staging directory.'
    Assert-True ($prelaunchCancellationReceipt.cleanup.outcome -eq 'not_required') 'Prelaunch cancellation incorrectly claimed process cleanup.'
    Assert-True ($prelaunchCancellationReceipt.failure.kind -eq 'operation_cancelled') 'Prelaunch cancellation terminal receipt lost its failure kind.'

    $result = Invoke-MvpStager -Fixture $fixture
    $manifestPath = Join-Path $result.staging_root 'staging-manifest.json'
    $manifest = Get-Content -Raw -Encoding UTF8 $manifestPath | ConvertFrom-Json
    $successfulTerminalReceipt = Get-MvpStagingTerminalReceiptFixture -StagingRoot $fixture.StagingRoot -RunId 'fixture-run'

    Assert-True (Test-Path -LiteralPath $result.tree_manifest -PathType Leaf) 'MVP staging did not publish its complete tree manifest.'
    Assert-True ($successfulTerminalReceipt.outcome -eq 'succeeded') 'NoLaunch staging did not publish a successful terminal outcome.'
    Assert-True ($successfulTerminalReceipt.phase -eq 'complete') 'NoLaunch staging terminal receipt did not reach the complete phase.'
    Assert-True ($successfulTerminalReceipt.staging_manifest_sha256 -eq $result.output_hash.ToLowerInvariant()) 'NoLaunch staging terminal receipt is not bound to its staging manifest.'
    Assert-True ($result.terminal_receipt.sha256 -match '^[0-9a-f]{64}$') 'MVP staging result did not expose the terminal receipt digest.'
    $stagingTreeEntries = @(Read-MvpAcceptanceStagingTreeManifest -StagingRoot $result.staging_root)
    Assert-True ($stagingTreeEntries.relative_path -contains 'staging-manifest.json') 'MVP staging tree manifest omitted its staging manifest.'
    Assert-True ($stagingTreeEntries.relative_path -contains 'runtime/zircon_runtime.exe') 'MVP staging tree manifest omitted the staged runtime executable.'
    Assert-True ($stagingTreeEntries.relative_path -contains 'editor/zircon_editor.exe') 'MVP staging tree manifest omitted the staged editor executable.'

    Assert-True (Test-Path -LiteralPath (Join-Path $result.staging_root 'runtime\zircon_runtime.exe')) 'Runtime executable was not staged.'
    Assert-True (Test-Path -LiteralPath (Join-Path $result.staging_root 'editor\zircon_editor.exe')) 'Editor executable was not staged.'
    Assert-True (Test-Path -LiteralPath (Join-Path $result.staging_root 'runtime\assets\ui\editor\welcome.zui')) 'Runtime engine assets were not staged beside the executable.'
    Assert-True (Test-Path -LiteralPath (Join-Path $result.staging_root 'editor\assets\ui\runtime\fixtures\hud_overlay.zui')) 'Editor engine assets were not staged beside the executable.'
    Assert-True (Test-Path -LiteralPath (Join-Path $result.staging_root 'templates\renderable-empty\project.toml')) 'Project template was not staged.'
    Assert-True ($result.staged_project_root -eq (Join-Path $result.staging_root 'project')) 'Staging result did not report the staged project root.'
    Assert-True (Test-Path -LiteralPath (Join-Path $result.staging_root 'project\zircon-project.toml')) 'Project manifest was not staged.'
    Assert-True (Test-Path -LiteralPath (Join-Path $result.staging_root 'project\assets\scenes\main.scene.toml')) 'Project scene was not staged.'
    Assert-True ($manifest.preflight.required_free_space_bytes -gt 0) 'NoLaunch staging did not record its source-derived disk budget.'
    [Int64]$manifestEntryBytes = 0
    foreach ($entry in @($manifest.entries)) {
        $manifestEntryBytes += [Int64]$entry.size_bytes
    }
    Assert-True ($manifest.preflight.input_copy_bytes -eq $manifestEntryBytes) 'NoLaunch staging disk budget does not equal the staged manifest entry bytes.'
    Assert-True ($manifest.preflight.evidence_reserve_bytes -eq 512MB) 'NoLaunch staging lost the fixed MVP evidence reserve.'
    Assert-True ($manifest.preflight.available_free_space_bytes -ge $manifest.preflight.required_free_space_bytes) 'NoLaunch staging recorded insufficient disk capacity as successful.'
    Assert-True (-not $manifest.preflight.interactive_desktop.required) 'NoLaunch staging must not require an interactive desktop.'
    Assert-True ($null -eq $manifest.preflight.interactive_desktop.monitor_count) 'NoLaunch staging must not probe a display that it will not use.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $result.staging_root 'project\.zircon\cache\stale.zasset'))) 'Machine-local project cache must not be staged.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $result.staging_root 'project\.zircon\registry\asset-registry.json'))) 'Machine-local project registry must not be staged.'
    $inputManifest = Get-Content -LiteralPath $fixture.ProductInputManifest -Raw -Encoding UTF8 | ConvertFrom-Json
    Assert-True ($manifest.source_fingerprint -eq $inputManifest.source_fingerprint) 'Manifest lost the source-bound input fingerprint.'
    Assert-True ($manifest.product_input_manifest.source_fingerprint -eq $manifest.source_fingerprint) 'Manifest lost product-input source provenance.'
    Assert-True ($manifest.product_input_manifest.sha256 -match '^[0-9A-F]{64}$') 'Manifest lost the product-input manifest hash.'
    Assert-True (@($manifest.product_input_manifest.artifacts).Count -eq 4) 'Manifest lost product-input artifact provenance.'
    Assert-True ($manifest.toolchain -match '^rustc\s+') 'Manifest did not record the active Rust toolchain.'
    Assert-True ($manifest.target -match '^[A-Za-z0-9_][A-Za-z0-9_-]*$') 'Manifest did not record a valid Rust target triple.'
    Assert-True ($manifest.entries.Count -eq 12) 'Manifest did not record every staged input.'
    $stagedProductInputManifest = @($manifest.entries | Where-Object { $_.logical_id -eq 'product-input-manifest' })
    Assert-True ($stagedProductInputManifest.Count -eq 1) 'Manifest did not record the original product-input manifest.'
    Assert-True ($stagedProductInputManifest[0].target_relative_path -eq 'build/mvp-product-inputs.json') 'Manifest did not use the canonical staged product-input manifest path.'
    Assert-True ($stagedProductInputManifest[0].sha256 -eq $manifest.product_input_manifest.sha256) 'Manifest summary is not bound to the staged product-input manifest digest.'
    Assert-True ($stagedProductInputManifest[0].size_bytes -eq $manifest.product_input_manifest.size_bytes) 'Manifest summary is not bound to the staged product-input manifest byte count.'
    Assert-True (@($manifest.entries | Where-Object { $_.logical_id -eq 'runtime-library/runtime' }).Count -eq 1) 'Runtime product library entry is missing.'
    Assert-True (@($manifest.entries | Where-Object { $_.logical_id -eq 'runtime-library/editor' }).Count -eq 1) 'Editor product library entry is missing.'
    Assert-True (@($manifest.entries | Where-Object { $_.logical_id -eq 'project/zircon-project.toml' }).Count -eq 1) 'Project manifest entry is missing.'
    Assert-True (@($manifest.entries | Where-Object { $_.logical_id -eq 'project/assets/scenes/main.scene.toml' }).Count -eq 1) 'Project scene entry is missing.'
    Assert-True (@($manifest.entries | Where-Object { $_.logical_id -eq 'engine-asset/runtime/ui/editor/welcome.zui' }).Count -eq 1) 'Runtime engine asset entry is missing.'
    Assert-True (@($manifest.entries | Where-Object { $_.logical_id -eq 'engine-asset/editor/ui/runtime/fixtures/hud_overlay.zui' }).Count -eq 1) 'Editor engine asset entry is missing.'
    Assert-True (@($manifest.entries | Where-Object { $_.logical_id -like 'project/.zircon/cache/*' }).Count -eq 0) 'Manifest must not record machine-local project cache files.'
    Assert-True (@($manifest.entries | Where-Object { $_.logical_id -like 'project/.zircon/registry/*' }).Count -eq 0) 'Manifest must not record machine-local project registry files.'
    $runtimeLibrary = @($manifest.entries | Where-Object { $_.logical_id -eq 'runtime-library/runtime' })[0]
    $editorLibrary = @($manifest.entries | Where-Object { $_.logical_id -eq 'runtime-library/editor' })[0]
    Assert-True ($null -ne $runtimeLibrary -and $null -ne $editorLibrary) 'Manifest must preserve separate runtime and editor library inputs.'
    Assert-True (@($manifest.entries | Where-Object { $_.sha256 -notmatch '^[0-9A-F]{64}$' }).Count -eq 0) 'Manifest entries must use SHA-256 hashes.'
    Assert-True ($result.output_hash -match '^[0-9A-F]{64}$') 'Stage output hash is not a SHA-256 value.'

    $json = (& $stager `
        -ProductInputManifest $fixture.ProductInputManifest `
        -TemplateRoot $fixture.TemplateRoot `
        -EngineAssetRoot $fixture.EngineAssetRoot `
        -ProjectRoot $fixture.ProjectRoot `
        -StagingRoot $fixture.StagingRoot `
        -RunId 'fixture-json' `
        -NoLaunch `
        -AllowUnsafeStagingRoot `
        -Json | ConvertFrom-Json)
    Assert-True ($json.staging_root -match 'fixture-json$') 'JSON output lost the staged run path.'
    Assert-True ($json.output_hash -match '^[0-9A-F]{64}$') 'JSON output lost the staging manifest hash.'
    Assert-True ($json.staged_project_root -match 'fixture-json[\\/]project$') 'JSON output lost the staged project root.'

    $authoringAutomationRequest = Join-Path $fixture.Root 'authoring-automation.json'
    [IO.File]::WriteAllText($authoringAutomationRequest, @'
{
  "schema_version": 1,
  "scenario_kind": "zircon.mvp-editor-automation-scenario",
  "scenario_id": "mvp.editor-authoring.v1",
  "bindings": [
    { "path": { "view_id": "Hierarchy", "control_id": "SelectCube", "event_kind": "Click" }, "payload": { "SelectionCommand": { "SelectSceneNode": { "node_id": 3 } } } },
    { "path": { "view_id": "Inspector", "control_id": "TransformPositionXCommit", "event_kind": "Submit" }, "payload": { "InspectorFieldBatch": { "subject_path": "entity://selected", "changes": [{ "field_id": "transform.translation.x", "value": { "Float": 42.0 } }] } } },
    { "path": { "view_id": "Inspector", "control_id": "TransformScaleXCommit", "event_kind": "Submit" }, "payload": { "InspectorFieldBatch": { "subject_path": "entity://selected", "changes": [{ "field_id": "transform.scale.x", "value": { "Float": 1.25 } }] } } },
    { "path": { "view_id": "WorkbenchMenuBar", "control_id": "SaveProject", "event_kind": "Click" }, "payload": { "MenuAction": { "action_id": "workbench.project.save" } } }
  ]
}
'@, [Text.UTF8Encoding]::new($false))
    $authoringLaunched = (& $stager `
        -ProductInputManifest $fixture.ProductInputManifest `
        -TemplateRoot $fixture.TemplateRoot `
        -EngineAssetRoot $fixture.EngineAssetRoot `
        -ProjectRoot $fixture.ProjectRoot `
        -AuthoringAutomationRequest $authoringAutomationRequest `
        -ReopenAutomationRequest $defaultReopenAutomationPath `
        -StagingRoot $fixture.StagingRoot `
        -RunId 'fixture-authoring-automation' `
        -RepeatCount 2 `
        -TimeoutSeconds 10 `
        -AllowUnsafeStagingRoot)
    Assert-True ($null -ne $authoringLaunched.authoring_automation) 'MVP staging launch fixture did not return the structured authoring automation report.'
    Assert-True ($authoringLaunched.baseline_automation.snapshot.inspector_translation[0] -eq '0') 'MVP staging did not capture the canonical Cube state before authoring.'
    Assert-True ($authoringLaunched.authoring_automation.records.Count -eq 6) 'MVP staging launch fixture lost the normal authoring binding sequence.'
    Assert-True ($authoringLaunched.authoring_automation.records[1].transaction_id -eq 1) 'MVP staging launch fixture did not preserve the inspector transaction.'
    Assert-True ($authoringLaunched.authoring_automation.records[2].transaction_id -eq 2) 'MVP staging launch fixture did not preserve the scale transaction.'
    Assert-True ($authoringLaunched.authoring_automation.records[3].binding_path -eq 'WorkbenchMenuBar/Undo:onClick') 'MVP staging launch fixture lost the authoring Undo binding.'
    Assert-True ($authoringLaunched.authoring_automation.records[4].binding_path -eq 'WorkbenchMenuBar/Redo:onClick') 'MVP staging launch fixture lost the authoring Redo binding.'
    Assert-True ($authoringLaunched.authoring_automation.records[5].save_generation -eq 2) 'MVP staging launch fixture did not preserve the project save generation.'
    Assert-True ($authoringLaunched.authoring_automation.snapshot.selected_node_name -eq 'Cube') 'MVP staging launch fixture did not preserve the retained-host authoring snapshot.'
    Assert-True ($authoringLaunched.authoring_automation.project_identity -eq 'fixture-project') 'MVP staging launch fixture lost the authoring project identity.'
    Assert-True ($authoringLaunched.authoring_automation.scene_uri -eq 'res://scenes/main.scene.toml') 'MVP staging launch fixture lost the authoring scene URI.'
    Assert-True ($authoringLaunched.authoring_automation.selected_model_resource_id -eq 'fixture-cube-model-resource') 'MVP staging launch fixture lost the selected Cube model reference.'
    Assert-ProcessTiming -Evidence $authoringLaunched.baseline_automation -Label 'Baseline automation process'
    Assert-ProcessTiming -Evidence $authoringLaunched.authoring_automation -Label 'Authoring automation process'
    foreach ($reopenProcess in @($authoringLaunched.reopen_automation)) {
        Assert-ProcessTiming -Evidence $reopenProcess -Label 'Reopen automation process'
    }
    foreach ($productProcess in @($authoringLaunched.product_runs)) {
        Assert-ProcessTiming -Evidence $productProcess -Label "$($productProcess.product) product process"
    }
    Assert-True (@($authoringLaunched.reopen_automation).Count -eq 2) 'MVP staging launch fixture did not run independent persisted-state reopen reports twice.'
    Assert-True (@($authoringLaunched.product_runs).Count -eq 5) 'MVP staging launch fixture did not preserve two pre-edit products, two editor reopens, and one after-edit runtime.'
    Assert-True (@($authoringLaunched.product_runs | Where-Object { $_.product -eq 'runtime' -and $_.attempt -eq 3 }).Count -eq 1) 'MVP staging launch fixture did not assign a new runtime attempt after authoring.'
    $reopenedEditorRun = @($authoringLaunched.product_runs | Where-Object { $_.product -eq 'editor' -and $_.attempt -eq 1 })[0]
    Assert-True ($reopenedEditorRun.editor_window_capture.path -eq 'captures/editor-after-reopen.png') 'MVP staging launch fixture did not archive the reopened editor window PNG.'
    Assert-True ($reopenedEditorRun.editor_window_capture.width -eq 16 -and $reopenedEditorRun.editor_window_capture.height -eq 16) 'MVP staging launch fixture did not inspect the reopened editor window PNG dimensions.'
    Assert-True ($reopenedEditorRun.editor_window_capture.non_background_pixels -ge 100) 'MVP staging launch fixture accepted an insufficiently visible reopened editor window PNG.'
    Assert-True ($reopenedEditorRun.editor_product_diagnostics.selected_node_name -eq 'Cube') 'MVP staging launch fixture did not tie the reopened editor capture to Cube.'
    Assert-True ($reopenedEditorRun.editor_product_diagnostics.inspector_translation_x -eq '42') 'MVP staging launch fixture did not tie the reopened editor capture to persisted Inspector X.'
    Assert-True ($reopenedEditorRun.editor_product_diagnostics.inspector_scale_x -eq '1.25') 'MVP staging launch fixture did not tie the reopened editor capture to persisted Inspector scale X.'
    Assert-True ($authoringLaunched.product_runs[0].stdout.sha256 -match '^[0-9A-F]{64}$') 'MVP staging launch fixture did not hash product stdout evidence.'
    Assert-True ($authoringLaunched.product_runs[0].diagnostic_logs[0].sha256 -match '^[0-9A-F]{64}$') 'MVP staging launch fixture did not hash diagnostic log evidence.'
    $authoringStartupSummary = Get-Content -Raw -Encoding UTF8 (Join-Path $authoringLaunched.staging_root 'startup-summary.json') | ConvertFrom-Json
    Assert-True ($authoringStartupSummary.authoring_automation.project_path -eq 'project') 'MVP staging startup evidence did not bind authoring automation to the staging-relative project root.'
    Assert-True (Test-Path -LiteralPath (Join-Path $authoringLaunched.staging_root 'authoring\automation.json')) 'MVP staging did not copy the authoring request into the source-bound staging root.'
    $authoringStdoutReport = Get-Content -Raw -Encoding UTF8 (Join-Path $authoringLaunched.staging_root 'logs\editor-authoring.stdout.log') | ConvertFrom-Json
    Assert-True ($authoringStdoutReport.project_path -eq 'project') 'MVP staging did not redact the machine-specific project root from portable automation stdout evidence.'
    Assert-True ($authoringStartupSummary.authoring_automation.automation_request.sha256 -match '^[0-9A-F]{64}$') 'MVP staging startup evidence did not hash the authoring automation request.'
    Assert-True ($authoringStartupSummary.authoring_automation.stdout.sha256 -match '^[0-9A-F]{64}$') 'MVP staging startup evidence did not hash the authoring automation stdout.'
    Assert-True ($authoringStartupSummary.authoring_automation.diagnostic_logs.Count -gt 0) 'MVP staging startup evidence did not retain authoring diagnostic log evidence.'
    Assert-True ($authoringStartupSummary.reopen_automation.Count -eq 2) 'MVP staging startup evidence did not retain both independent reopen reports.'
    Assert-True ($authoringStartupSummary.reopen_automation[0].automation_request.sha256 -match '^[0-9A-F]{64}$') 'MVP staging startup evidence did not hash the reopen automation request.'
    Assert-True ($authoringStartupSummary.reopen_automation[0].stdout.sha256 -match '^[0-9A-F]{64}$') 'MVP staging startup evidence did not hash the reopen automation stdout.'
    Assert-True ($authoringStartupSummary.reopen_automation[0].diagnostic_logs.Count -gt 0) 'MVP staging startup evidence did not retain reopen diagnostic log evidence.'
    Assert-True (Test-Path -LiteralPath (Join-Path $authoringLaunched.staging_root 'reopen\automation.json')) 'MVP staging did not copy the reopen request into the source-bound staging root.'
    $authoringLifecycleEntries = Get-ProcessJournalLifecycleEntries -StageRoot $authoringLaunched.staging_root
    Assert-ProcessJournalProgress `
        -Entries $authoringLifecycleEntries `
        -Phase 'editor-authoring' `
        -ExpectedNames @('mvp.editor.automation.completed.v1')

    $wrongAutomationProjectRejected = $false
    $wrongAutomationProjectError = $null
    Set-MvpStagingFixtureControl -Fixture $fixture -Scope Project -Control 'report-wrong-authoring-project-path'
    try {
        try {
            $null = & $stager `
                -ProductInputManifest $fixture.ProductInputManifest `
                -TemplateRoot $fixture.TemplateRoot `
                -EngineAssetRoot $fixture.EngineAssetRoot `
                -ProjectRoot $fixture.ProjectRoot `
                -AuthoringAutomationRequest $authoringAutomationRequest `
                -ReopenAutomationRequest $defaultReopenAutomationPath `
                -StagingRoot $fixture.StagingRoot `
                -RunId 'fixture-wrong-authoring-project' `
                -RepeatCount 2 `
                -TimeoutSeconds 10 `
                -AllowUnsafeStagingRoot
        }
        catch {
            $wrongAutomationProjectError = $_.Exception.Message
            $wrongAutomationProjectRejected = $_.Exception.Message -match 'authoring automation report project_path.*differs from staged project'
        }
    }
    finally {
        Clear-MvpStagingFixtureControl -Fixture $fixture -Scope Project
    }
    Assert-True $wrongAutomationProjectRejected "MVP staging failed to reject authoring automation evidence from a different project root. Actual result: $wrongAutomationProjectError"

    $authoringFailureRunId = 'fixture-authoring-nonzero-child'
    $authoringFailureStage = Join-Path $fixture.StagingRoot $authoringFailureRunId
    Set-MvpStagingFixtureControl -Fixture $fixture -Scope Project -Control 'fail-automation-with-child'
    try {
        $authoringFailureCleaned = $false
        $authoringFailureDiagnostics = ''
        try {
            $null = & $stager `
                -ProductInputManifest $fixture.ProductInputManifest `
                -TemplateRoot $fixture.TemplateRoot `
                -EngineAssetRoot $fixture.EngineAssetRoot `
                -ProjectRoot $fixture.ProjectRoot `
                -AuthoringAutomationRequest $authoringAutomationRequest `
                -ReopenAutomationRequest $defaultReopenAutomationPath `
                -StagingRoot $fixture.StagingRoot `
                -RunId $authoringFailureRunId `
                -RepeatCount 2 `
                -TimeoutSeconds 10 `
                -AllowUnsafeStagingRoot
        }
        catch {
            $authoringFailureDiagnostics = $_.Exception.Message
            $authoringFailureCleaned = $_.Exception.Message -match 'exited with code 32|remain after product exit and were terminated'
        }
        Assert-True $authoringFailureCleaned 'A nonzero authoring automation exit with a staged child was not rejected after cleanup.'
        Assert-True ($authoringFailureDiagnostics -match 'fixture automation failed after spawning child') 'A nonzero authoring automation exit did not retain the child stderr diagnostic.'
        $authoringFailureStderr = Join-Path $authoringFailureStage 'logs/editor-authoring.stderr.log'
        Assert-True (Test-Path -LiteralPath $authoringFailureStderr) 'A nonzero authoring automation exit did not preserve its stderr log.'
        Assert-True ([IO.File]::ReadAllText($authoringFailureStderr) -match 'fixture automation failed after spawning child') 'A nonzero authoring automation stderr log was not drained before failure.'
        $authoringFailureJournal = @(Get-ProcessJournalEntries -StageRoot $authoringFailureStage | Where-Object { $_.phase -eq 'editor-authoring' })
        Assert-True ($authoringFailureJournal.Count -eq 1) 'Nonzero authoring automation did not emit exactly one journal entry.'
        Assert-ProcessJournalEntry -Entry $authoringFailureJournal[0] -Phase 'editor-authoring' -Outcome 'crashed' -ExitCode 32
        Start-Sleep -Milliseconds 250
        $authoringFailurePrefix = [IO.Path]::GetFullPath($authoringFailureStage).TrimEnd('\') + [IO.Path]::DirectorySeparatorChar
        $authoringFailurePids = @(
            Get-CimInstance Win32_Process | Where-Object {
                $executablePath = [string]$_.ExecutablePath
                -not [string]::IsNullOrWhiteSpace($executablePath) -and
                    $executablePath.StartsWith($authoringFailurePrefix, [StringComparison]::OrdinalIgnoreCase)
            } | Select-Object -ExpandProperty ProcessId
        )
        Assert-True ($authoringFailurePids.Count -eq 0) 'A nonzero authoring automation exit left a staged child process.'
    }
    finally {
        Clear-MvpStagingFixtureControl -Fixture $fixture -Scope Project
        $authoringFailurePrefix = [IO.Path]::GetFullPath($authoringFailureStage).TrimEnd('\') + [IO.Path]::DirectorySeparatorChar
        $authoringFailurePids = @(
            Get-CimInstance Win32_Process | Where-Object {
                $executablePath = [string]$_.ExecutablePath
                -not [string]::IsNullOrWhiteSpace($executablePath) -and
                    $executablePath.StartsWith($authoringFailurePrefix, [StringComparison]::OrdinalIgnoreCase)
            } | Select-Object -ExpandProperty ProcessId
        )
        foreach ($processId in $authoringFailurePids) {
            Stop-Process -Id $processId -Force -ErrorAction SilentlyContinue
        }
        foreach ($processId in $authoringFailurePids) {
            Wait-Process -Id $processId -Timeout 5 -ErrorAction SilentlyContinue
        }
    }

    Set-MvpStagingFixtureControl -Fixture $fixture -Scope Project -Control 'skip-editor-capture'
    try {
        $missingReopenedEditorCaptureRejected = $false
        try {
            $null = & $stager `
                -ProductInputManifest $fixture.ProductInputManifest `
                -TemplateRoot $fixture.TemplateRoot `
                -EngineAssetRoot $fixture.EngineAssetRoot `
                -ProjectRoot $fixture.ProjectRoot `
                -AuthoringAutomationRequest $authoringAutomationRequest `
                -ReopenAutomationRequest $defaultReopenAutomationPath `
                -StagingRoot $fixture.StagingRoot `
                -RunId 'fixture-missing-reopened-editor-capture' `
                -RepeatCount 2 `
                -TimeoutSeconds 10 `
                -AllowUnsafeStagingRoot
        }
        catch {
            $missingReopenedEditorCaptureRejected = $_.Exception.Message -match 'editor_product_frame_capture_written'
        }
        Assert-True $missingReopenedEditorCaptureRejected 'MVP staging did not reject a missing reopened editor window PNG diagnostic.'
    }
    finally {
        Clear-MvpStagingFixtureControl -Fixture $fixture -Scope Project
    }

    Set-MvpStagingFixtureControl -Fixture $fixture -Scope Project -Control 'skip-editor-capture-file'
    try {
        $missingReopenedEditorCaptureFileRejected = $false
        try {
            $null = & $stager `
                -ProductInputManifest $fixture.ProductInputManifest `
                -TemplateRoot $fixture.TemplateRoot `
                -EngineAssetRoot $fixture.EngineAssetRoot `
                -ProjectRoot $fixture.ProjectRoot `
                -AuthoringAutomationRequest $authoringAutomationRequest `
                -ReopenAutomationRequest $defaultReopenAutomationPath `
                -StagingRoot $fixture.StagingRoot `
                -RunId 'fixture-missing-reopened-editor-capture-file' `
                -RepeatCount 2 `
                -TimeoutSeconds 10 `
                -AllowUnsafeStagingRoot
        }
        catch {
            $missingReopenedEditorCaptureFileRejected = $_.Exception.Message -match 'Editor window capture.*was not written'
        }
        Assert-True $missingReopenedEditorCaptureFileRejected 'MVP staging did not reject a completed reopened editor capture diagnostic without its PNG file.'
    }
    finally {
        Clear-MvpStagingFixtureControl -Fixture $fixture -Scope Project
    }

    $launched = (& $stager `
            -ProductInputManifest $fixture.ProductInputManifest `
            -TemplateRoot $fixture.TemplateRoot `
            -EngineAssetRoot $fixture.EngineAssetRoot `
            -ProjectRoot $fixture.ProjectRoot `
            -StagingRoot $fixture.StagingRoot `
            -RunId 'fixture-launch' `
            -RepeatCount 1 `
            -TimeoutSeconds 10 `
            -AllowUnsafeStagingRoot)
        Assert-True $launched.launched 'MVP staging launch fixture did not run the staged products.'
        $launchedManifest = Get-Content -Raw -Encoding UTF8 $launched.manifest | ConvertFrom-Json
        [Int64]$launchedManifestEntryBytes = 0
        foreach ($entry in @($launchedManifest.entries)) {
            $launchedManifestEntryBytes += [Int64]$entry.size_bytes
        }
        Assert-True ($launchedManifest.preflight.input_copy_bytes -eq $launchedManifestEntryBytes) 'Launched staging disk budget does not equal the staged manifest entry bytes.'
        Assert-True ($launchedManifest.preflight.available_free_space_bytes -ge $launchedManifest.preflight.required_free_space_bytes) 'Launched staging did not prove sufficient disk capacity before copying.'
        Assert-True $launchedManifest.preflight.interactive_desktop.required 'Launched staging did not require an interactive desktop.'
        Assert-True $launchedManifest.preflight.interactive_desktop.user_interactive 'Launched staging did not record an interactive Windows user session.'
        Assert-True ($launchedManifest.preflight.interactive_desktop.session_id -gt 0) 'Launched staging did not record a non-service Windows session.'
        Assert-True ($launchedManifest.preflight.interactive_desktop.monitor_count -gt 0) 'Launched staging did not record an attached display.'
        Assert-True (@($launched.product_runs).Count -eq 2) 'MVP staging launch fixture did not record both products.'
        Assert-True (@($launched.product_runs | Where-Object { $_.first_frame_presented -and $_.teardown_complete }).Count -eq 2) 'MVP staging launch fixture did not verify first-frame teardown for both products.'
        $runtimeRun = @($launched.product_runs | Where-Object { $_.product -eq 'runtime' })[0]
        $editorRun = @($launched.product_runs | Where-Object { $_.product -eq 'editor' })[0]
        foreach ($productRun in @($runtimeRun, $editorRun)) {
            $assetRootDiagnostics = @($productRun.diagnostic_logs | ForEach-Object {
                Get-Content -Raw -LiteralPath (Join-Path $launched.staging_root $_.path)
            })
            Assert-True ((@($assetRootDiagnostics) -join "`n") -match '(?m)^fixture_asset_root=assets\r?$') "MVP staging passed a non-relative asset root to the $($productRun.product) product."
        }
        Assert-True ($runtimeRun.frame_capture.path -eq 'captures/runtime-1.png') 'MVP staging launch fixture did not archive the runtime PNG under the stage capture root.'
        Assert-True ($runtimeRun.frame_capture.sha256 -match '^[0-9A-F]{64}$') 'MVP staging launch fixture did not hash the runtime PNG evidence.'
        Assert-True ($runtimeRun.frame_capture.width -eq 16 -and $runtimeRun.frame_capture.height -eq 16) 'MVP staging launch fixture did not inspect runtime PNG dimensions.'
        Assert-True ($runtimeRun.frame_capture.non_background_pixels -ge 100) 'MVP staging launch fixture accepted an insufficiently visible runtime PNG.'
        Assert-True ($runtimeRun.frame_capture.non_transparent_pixels -eq 256) 'MVP staging launch fixture lost runtime PNG alpha evidence.'
        Assert-True ($runtimeRun.runtime_product_diagnostics.project_identity -eq 'fixture-project') 'MVP staging launch fixture did not preserve runtime project identity evidence.'
        Assert-True ($runtimeRun.runtime_product_diagnostics.scene_uri -eq 'res://scenes/main.scene.toml') 'MVP staging launch fixture did not preserve runtime scene evidence.'
        Assert-True ($runtimeRun.runtime_product_diagnostics.render_adapter -eq 'Fixture WGPU Adapter') 'MVP staging launch fixture did not preserve an adapter identity containing spaces.'
        Assert-True ($runtimeRun.runtime_product_diagnostics.render_adapter_type -eq 'discrete_gpu') 'MVP staging launch fixture did not preserve adapter device-type evidence.'
        Assert-True ($runtimeRun.runtime_product_diagnostics.device_max_bind_groups -eq '5') 'MVP staging launch fixture did not preserve max bind-group evidence.'
        Assert-True ($runtimeRun.runtime_product_diagnostics.device_max_texture_dimension_2d -eq '16384') 'MVP staging launch fixture did not preserve max texture dimension evidence.'
        Assert-True ($runtimeRun.runtime_product_diagnostics.device_max_texture_array_layers -eq '256') 'MVP staging launch fixture did not preserve max texture array-layer evidence.'
        Assert-True ($runtimeRun.runtime_product_diagnostics.device_max_sampled_textures_per_shader_stage -eq '16') 'MVP staging launch fixture did not preserve sampled-texture limit evidence.'
        Assert-True ($runtimeRun.runtime_product_diagnostics.device_max_storage_buffers_per_shader_stage -eq '8') 'MVP staging launch fixture did not preserve storage-buffer count evidence.'
        Assert-True ($runtimeRun.runtime_product_diagnostics.device_max_storage_buffer_binding_size -eq '134217728') 'MVP staging launch fixture did not preserve storage-buffer byte-limit evidence.'
        Assert-True ($runtimeRun.runtime_product_diagnostics.graph_executed_pass_count -eq '1') 'MVP staging launch fixture did not preserve graph-pass evidence.'
        Assert-True ($runtimeRun.runtime_product_diagnostics.mesh_draw_count -eq '1') 'MVP staging launch fixture did not preserve mesh-draw evidence.'
        Assert-True ($runtimeRun.runtime_product_diagnostics.directional_light_count -eq '1') 'MVP staging launch fixture did not preserve directional-light evidence.'
        Assert-True ($runtimeRun.runtime_product_diagnostics.material_fallback_count -eq '0') 'MVP staging launch fixture did not preserve material-fallback evidence.'
        Assert-True ($runtimeRun.runtime_product_diagnostics.material_validation_error_count -eq '0') 'MVP staging launch fixture did not preserve material-validation evidence.'
        Assert-True ($runtimeRun.runtime_product_diagnostics.input_viewport_resize_count -eq '2') 'MVP staging launch fixture did not preserve viewport-resize input evidence.'
        Assert-True ($runtimeRun.runtime_product_diagnostics.input_pointer_move_count -eq '1') 'MVP staging launch fixture did not preserve pointer-input evidence.'
        Assert-True ($runtimeRun.runtime_product_diagnostics.input_mouse_button_press_count -eq '1') 'MVP staging launch fixture did not preserve mouse-button press evidence.'
        Assert-True ($runtimeRun.runtime_product_diagnostics.input_mouse_button_release_count -eq '1') 'MVP staging launch fixture did not preserve mouse-button release evidence.'
        Assert-True ($runtimeRun.runtime_product_diagnostics.input_keyboard_press_count -eq '1') 'MVP staging launch fixture did not preserve keyboard press evidence.'
        Assert-True ($runtimeRun.runtime_product_diagnostics.input_keyboard_release_count -eq '1') 'MVP staging launch fixture did not preserve keyboard release evidence.'
        Assert-True ($null -eq $editorRun.frame_capture) 'MVP staging must not request a runtime PNG from the editor product.'
        Assert-True ($null -eq $editorRun.editor_window_capture) 'MVP staging must not request editor window PNG evidence outside the F5 reopen workflow.'
        Assert-True ($null -eq $editorRun.runtime_product_diagnostics) 'MVP staging must not require runtime diagnostics from the editor product.'
        $lifecycleEntries = Get-ProcessJournalLifecycleEntries -StageRoot $launched.staging_root
        foreach ($phase in @('runtime-1', 'editor-1')) {
            $phaseEntries = @($lifecycleEntries | Where-Object { $_.phase -eq $phase })
            $startedEntries = @($phaseEntries | Where-Object { $_.event_kind -eq 'started' })
            $exitEntries = @($phaseEntries | Where-Object { $_.event_kind -eq 'exit' })
            $cleanupEntries = @($phaseEntries | Where-Object { $_.event_kind -eq 'cleanup' })
            $terminalEntries = @($phaseEntries | Where-Object { $_.event_kind -eq 'terminal' })
            Assert-True ($startedEntries.Count -eq 1) "MVP staging did not persist exactly one started lifecycle record for '$phase'."
            Assert-True ($exitEntries.Count -eq 1) "MVP staging did not persist exactly one exit lifecycle record for '$phase'."
            Assert-True ($cleanupEntries.Count -eq 1) "MVP staging did not persist exactly one cleanup lifecycle record for '$phase'."
            Assert-True ($terminalEntries.Count -eq 1) "MVP staging did not persist exactly one terminal lifecycle record for '$phase'."
            Assert-True ($startedEntries[0].run_id -eq $launched.run_id) "MVP staging lifecycle start record for '$phase' does not bind the staging run id."
            Assert-True ($exitEntries[0].run_id -eq $launched.run_id) "MVP staging lifecycle exit record for '$phase' does not bind the staging run id."
            Assert-True ($cleanupEntries[0].run_id -eq $launched.run_id) "MVP staging lifecycle cleanup record for '$phase' does not bind the staging run id."
            Assert-True ($terminalEntries[0].run_id -eq $launched.run_id) "MVP staging lifecycle terminal record for '$phase' does not bind the staging run id."
            Assert-True ([int]$startedEntries[0].process_id -gt 0) "MVP staging lifecycle start record for '$phase' has no process id."
            Assert-True (-not [string]::IsNullOrWhiteSpace([string]$startedEntries[0].process_started_at_utc)) "MVP staging lifecycle start record for '$phase' has no process creation time."
            Assert-True ($exitEntries[0].process_id -eq $startedEntries[0].process_id) "MVP staging lifecycle exit record for '$phase' does not bind the started process id."
            Assert-True ($exitEntries[0].process_started_at_utc -eq $startedEntries[0].process_started_at_utc) "MVP staging lifecycle exit record for '$phase' does not bind the started process creation time."
            Assert-True ($exitEntries[0].root_process_exited -eq $true) "MVP staging lifecycle exit record for '$phase' does not record root process completion."
            Assert-True ($cleanupEntries[0].process_id -eq $startedEntries[0].process_id) "MVP staging lifecycle cleanup record for '$phase' does not bind the started process id."
            Assert-True ($cleanupEntries[0].process_started_at_utc -eq $startedEntries[0].process_started_at_utc) "MVP staging lifecycle cleanup record for '$phase' does not bind the started process creation time."
            Assert-True ($cleanupEntries[0].job_empty -eq $true) "MVP staging lifecycle cleanup record for '$phase' did not confirm an empty process job."
            Assert-True ($terminalEntries[0].process_id -eq $startedEntries[0].process_id) "MVP staging lifecycle terminal record for '$phase' does not bind the started process id."
            Assert-True ($terminalEntries[0].process_started_at_utc -eq $startedEntries[0].process_started_at_utc) "MVP staging lifecycle terminal record for '$phase' does not bind the started process creation time."
            Assert-True ([array]::IndexOf($lifecycleEntries, $startedEntries[0]) -lt [array]::IndexOf($lifecycleEntries, $exitEntries[0])) "MVP staging exit record for '$phase' was written before its start record."
            Assert-True ([array]::IndexOf($lifecycleEntries, $exitEntries[0]) -lt [array]::IndexOf($lifecycleEntries, $cleanupEntries[0])) "MVP staging cleanup record for '$phase' was written before its exit record."
            Assert-True ([array]::IndexOf($lifecycleEntries, $cleanupEntries[0]) -lt [array]::IndexOf($lifecycleEntries, $terminalEntries[0])) "MVP staging terminal record for '$phase' was written before its cleanup record."
            Assert-ProcessJournalProgress `
                -Entries $lifecycleEntries `
                -Phase $phase `
                -ExpectedNames $(if ($phase.StartsWith('runtime-', [StringComparison]::Ordinal)) {
                    @(
                        'mvp.runtime.startup-ready.v1',
                        'mvp.runtime.first-frame-presented.v1',
                        'mvp.runtime.teardown-complete.v1'
                    )
                }
                else {
                    @(
                        'mvp.editor.startup-ready.v1',
                        'mvp.editor.first-frame-presented.v1',
                        'mvp.editor.teardown-complete.v1'
                    )
                })
        }

    $createWithoutLaunchRejected = $false
    try {
        $null = & $stager `
            -ProductInputManifest $fixture.ProductInputManifest `
            -TemplateRoot $fixture.TemplateRoot `
            -EngineAssetRoot $fixture.EngineAssetRoot `
            -StagingRoot $fixture.StagingRoot `
            -RunId 'fixture-created-without-launch' `
            -CreateProject `
            -NoLaunch `
            -AllowUnsafeStagingRoot
    }
    catch {
        $createWithoutLaunchRejected = $_.Exception.Message -match 'CreateProject cannot be combined with NoLaunch'
    }
    Assert-True $createWithoutLaunchRejected 'Created projects must not silently skip the staged editor launch.'

    $created = (& $stager `
        -ProductInputManifest $fixture.ProductInputManifest `
        -TemplateRoot $fixture.TemplateRoot `
        -EngineAssetRoot $fixture.EngineAssetRoot `
        -StagingRoot $fixture.StagingRoot `
        -RunId 'fixture-created-project' `
        -CreateProject `
        -ProjectName 'ZirconMvpFixture' `
        -RepeatCount 1 `
        -TimeoutSeconds 10 `
        -AllowUnsafeStagingRoot)
    Assert-True ($created.staged_project_root -match 'fixture-created-project[\\/]project[\\/]ZirconMvpFixture$') 'Created project root was not returned from staging.'
    Assert-True (Test-Path -LiteralPath (Join-Path $created.staged_project_root 'zircon-project.toml')) 'Staged editor did not create the project manifest.'
    Assert-True (@($created.product_runs).Count -eq 2) 'Created project did not flow into the staged runtime and editor runs.'
    $createdRuntimeRun = @($created.product_runs | Where-Object { $_.product -eq 'runtime' })[0]
    Assert-True ($createdRuntimeRun.runtime_product_diagnostics.project_identity -eq 'fixture-created-project') 'Created project did not become the staged runtime product identity.'
    $createdStartupSummary = Get-Content -Raw -Encoding UTF8 (Join-Path $created.staging_root 'startup-summary.json') | ConvertFrom-Json
    Assert-True ($createdStartupSummary.staged_project_root -match '^project[\\/]ZirconMvpFixture$') 'Created project startup evidence did not record the canonical staged project root.'
    Assert-True ($createdStartupSummary.project_creation.exit_code -eq 0) 'Created project startup evidence did not preserve the staged editor creation exit code.'
    Assert-ProcessTiming -Evidence $createdStartupSummary.project_creation -Label 'Project creation process'
    Assert-True ($createdStartupSummary.project_creation.editor_window_capture.path -eq 'captures/editor-before-edit.png') 'Created project startup evidence did not archive the editor window PNG before authoring.'
    Assert-True ($createdStartupSummary.project_creation.editor_window_capture.non_background_pixels -ge 100) 'Created project startup evidence accepted an insufficiently visible editor window PNG before authoring.'
    Assert-True ($createdStartupSummary.project_creation.editor_product_diagnostics.selected_node_name -eq 'Cube') 'Created project startup evidence did not tie the editor window PNG to Cube.'
    Assert-True ($createdStartupSummary.project_creation.editor_product_diagnostics.inspector_translation_x -eq '0') 'Created project startup evidence did not tie the editor window PNG to the initial Inspector X.'
    Assert-True ($createdStartupSummary.project_creation.editor_product_diagnostics.inspector_scale_x -eq '1.00') 'Created project startup evidence did not tie the editor window PNG to the initial Inspector scale X.'
    Assert-True ($createdStartupSummary.project_creation.stdout.sha256 -match '^[0-9A-F]{64}$') 'Created project startup evidence did not hash the staged editor creation stdout.'
    Assert-True ($createdStartupSummary.project_creation.diagnostic_logs.Count -gt 0) 'Created project startup evidence did not retain the staged editor creation diagnostics.'
    Assert-True ($createdStartupSummary.project_creation.project_open.project_root -eq 'project/ZirconMvpFixture') 'Created project startup evidence did not preserve the canonical project-open root.'
    Assert-True ($createdStartupSummary.project_creation.project_open.manifest_identity -eq 'fixture-created-project@v1') 'Created project startup evidence did not preserve the manifest identity from the normal editor project-open diagnostic.'
    Assert-True ($createdStartupSummary.project_creation.project_open.scene_uri -eq 'res://scenes/main.scene.toml') 'Created project startup evidence did not preserve the default scene URI from the normal editor project-open diagnostic.'
    Assert-True ($createdStartupSummary.project_creation.project_open.registry_ready_asset_count -eq 4) 'Created project startup evidence did not preserve ready starter-asset count.'
    Assert-True ($createdStartupSummary.project_creation.project_open.settings_source -eq 'persisted-v1') 'Created project startup evidence did not preserve persisted project-settings provenance.'
    Assert-True (Test-Path -LiteralPath (Join-Path $created.staging_root 'logs/editor-create.stdout.log')) 'Created project staging did not retain the editor creation stdout log.'
    Assert-True (Test-Path -LiteralPath (Join-Path $created.staging_root 'logs/editor-create.stderr.log')) 'Created project staging did not retain the editor creation stderr log.'
    $createdLifecycleEntries = Get-ProcessJournalLifecycleEntries -StageRoot $created.staging_root
    Assert-ProcessJournalProgress `
        -Entries $createdLifecycleEntries `
        -Phase 'editor-create' `
        -ExpectedNames @(
            'mvp.editor.project-opened.v1',
            'mvp.editor.startup-ready.v1',
            'mvp.editor.first-frame-presented.v1',
            'mvp.editor.teardown-complete.v1'
        )

    $unicodeProjectName = (-join ([char[]]@(0x9879, 0x76EE))) + ' ' + (-join ([char[]]@(0x8DEF, 0x5F84)))
    $unicodeProjectRunId = 'fixture-created-project-unicode'
    $unicodeCreated = (& $stager `
        -ProductInputManifest $fixture.ProductInputManifest `
        -TemplateRoot $fixture.TemplateRoot `
        -EngineAssetRoot $fixture.EngineAssetRoot `
        -StagingRoot $fixture.StagingRoot `
        -RunId $unicodeProjectRunId `
        -CreateProject `
        -ProjectName $unicodeProjectName `
        -RepeatCount 1 `
        -TimeoutSeconds 10 `
        -AllowUnsafeStagingRoot)
    $expectedUnicodeProjectRoot = Join-Path `
        (Join-Path $fixture.StagingRoot $unicodeProjectRunId) `
        (Join-Path 'project' $unicodeProjectName)
    Assert-True `
        ([IO.Path]::GetFullPath($unicodeCreated.staged_project_root) -eq [IO.Path]::GetFullPath($expectedUnicodeProjectRoot)) `
        'Created project staging did not preserve the Unicode project root through the Windows PowerShell process launch.'
    $unicodeStartupSummary = Get-Content -Raw -Encoding UTF8 (Join-Path $unicodeCreated.staging_root 'startup-summary.json') | ConvertFrom-Json
    Assert-True ($unicodeStartupSummary.staged_project_root -eq "project/$unicodeProjectName") 'Created project staging did not write the Unicode project root as UTF-8 startup evidence.'
    Assert-True ($unicodeStartupSummary.project_creation.project_open.project_root -eq "project/$unicodeProjectName") 'Created project staging did not retain the Unicode project-open root from the normal editor diagnostic.'

    $missingProjectOpenDiagnosticRunId = 'fixture-created-project-missing-open-diagnostic'
    Set-MvpStagingFixtureControl -Fixture $fixture -Scope Template -Control 'skip-project-open-diagnostic'
    try {
        $missingProjectOpenDiagnosticRejected = $false
        try {
            & $stager `
                -ProductInputManifest $fixture.ProductInputManifest `
                -TemplateRoot $fixture.TemplateRoot `
                -EngineAssetRoot $fixture.EngineAssetRoot `
                -StagingRoot $fixture.StagingRoot `
                -RunId $missingProjectOpenDiagnosticRunId `
                -CreateProject `
                -ProjectName 'ZirconMvpFixtureMissingProjectOpenDiagnostic' `
                -RepeatCount 1 `
                -TimeoutSeconds 10 `
                -AllowUnsafeStagingRoot | Out-Null
        }
        catch {
            $missingProjectOpenDiagnosticRejected = $_.Exception.Message -match 'editor_project_open'
        }
        Assert-True $missingProjectOpenDiagnosticRejected 'A staged project creation without the normal editor project-open diagnostic was not rejected.'
    }
    finally {
        Clear-MvpStagingFixtureControl -Fixture $fixture -Scope Template
    }

    $createFailureRunId = 'fixture-created-project-failure'
    $createFailureStage = Join-Path $fixture.StagingRoot $createFailureRunId
    Set-MvpStagingFixtureControl -Fixture $fixture -Scope Template -Control 'fail-create-with-child'
    try {
        $createFailureCleaned = $false
        $createFailureDiagnostics = ''
        try {
            & $stager `
                -ProductInputManifest $fixture.ProductInputManifest `
                -TemplateRoot $fixture.TemplateRoot `
                -EngineAssetRoot $fixture.EngineAssetRoot `
                -StagingRoot $fixture.StagingRoot `
                -RunId $createFailureRunId `
                -CreateProject `
                -ProjectName 'ZirconMvpFixtureFailure' `
                -RepeatCount 1 `
                -TimeoutSeconds 10 `
                -AllowUnsafeStagingRoot | Out-Null
        }
        catch {
            $createFailureDiagnostics = $_.Exception.Message
            $createFailureCleaned = $_.Exception.Message -match 'failed with exit code 24|remain after product exit and were terminated'
        }
        Assert-True $createFailureCleaned "A nonzero staged project-creation exit with a child process must be rejected only after the staged child is cleaned up. Actual result: $createFailureDiagnostics"
        Start-Sleep -Milliseconds 250
        $createFailurePids = @(
            Get-CimInstance Win32_Process | Where-Object {
                $executablePath = [string]$_.ExecutablePath
                -not [string]::IsNullOrWhiteSpace($executablePath) -and
                    $executablePath.StartsWith(
                        [IO.Path]::GetFullPath($createFailureStage).TrimEnd('\\') + [IO.Path]::DirectorySeparatorChar,
                        [StringComparison]::OrdinalIgnoreCase
                    )
            } | Select-Object -ExpandProperty ProcessId
        )
        Assert-True ($createFailurePids.Count -eq 0) 'A nonzero staged project-creation exit must not leave a staged child process.'
        $createFailureJournal = @(Get-ProcessJournalEntries -StageRoot $createFailureStage | Where-Object { $_.phase -eq 'editor-create' })
        Assert-True ($createFailureJournal.Count -eq 1) 'Nonzero project creation did not emit exactly one journal entry.'
        Assert-ProcessJournalEntry -Entry $createFailureJournal[0] -Phase 'editor-create' -Outcome 'crashed' -ExitCode 24
    }
    finally {
        Clear-MvpStagingFixtureControl -Fixture $fixture -Scope Template
        $createFailurePrefix = [IO.Path]::GetFullPath($createFailureStage).TrimEnd('\\') + [IO.Path]::DirectorySeparatorChar
        $createFailurePids = @(
            Get-CimInstance Win32_Process | Where-Object {
                $executablePath = [string]$_.ExecutablePath
                -not [string]::IsNullOrWhiteSpace($executablePath) -and
                    $executablePath.StartsWith($createFailurePrefix, [StringComparison]::OrdinalIgnoreCase)
            } | Select-Object -ExpandProperty ProcessId
        )
        foreach ($processId in $createFailurePids) {
            Stop-Process -Id $processId -Force -ErrorAction SilentlyContinue
        }
        foreach ($processId in $createFailurePids) {
            Wait-Process -Id $processId -Timeout 5 -ErrorAction SilentlyContinue
        }
    }

    $missingCaptureRunId = 'fixture-missing-runtime-capture'
    Set-MvpStagingFixtureControl -Fixture $fixture -Scope Project -Control 'skip-runtime-capture'
    try {
        $missingCaptureDetected = $false
        try {
            & $stager `
                -ProductInputManifest $fixture.ProductInputManifest `
                -TemplateRoot $fixture.TemplateRoot `
                -EngineAssetRoot $fixture.EngineAssetRoot `
                -ProjectRoot $fixture.ProjectRoot `
                -StagingRoot $fixture.StagingRoot `
                -RunId $missingCaptureRunId `
                -RepeatCount 1 `
                -TimeoutSeconds 10 `
                -AllowUnsafeStagingRoot | Out-Null
        }
        catch {
            $missingCaptureDetected = $_.Exception.Message -match 'runtime_product_frame_capture_written|Runtime frame capture'
        }
        Assert-True $missingCaptureDetected 'A runtime product that omits requested PNG evidence was not rejected.'
        $missingCaptureReceipt = Get-MvpStagingTerminalReceiptFixture -StagingRoot $fixture.StagingRoot -RunId $missingCaptureRunId
        Assert-True ($missingCaptureReceipt.outcome -eq 'failed') 'Missing runtime capture did not publish a failed terminal outcome.'
        Assert-True ($missingCaptureReceipt.phase -eq 'product_startup') 'Missing runtime capture terminal receipt lost its product-startup phase.'
        Assert-True $missingCaptureReceipt.staging_directory_published 'Missing runtime capture terminal receipt did not retain its published stage identity.'
        Assert-True ($missingCaptureReceipt.staging_manifest_sha256 -match '^[0-9a-f]{64}$') 'Missing runtime capture terminal receipt lost its staging manifest digest.'
        Assert-True ($missingCaptureReceipt.failure.message_sha256 -match '^[0-9a-f]{64}$') 'Missing runtime capture terminal receipt lost its failure digest.'
        Assert-True ($null -eq $missingCaptureReceipt.failure.PSObject.Properties['message']) 'Missing runtime capture terminal receipt retained raw failure text.'
    }
    finally {
        Clear-MvpStagingFixtureControl -Fixture $fixture -Scope Project
    }

    $missingDiagnosticsRunId = 'fixture-missing-runtime-diagnostics'
    Set-MvpStagingFixtureControl -Fixture $fixture -Scope Project -Control 'skip-runtime-diagnostics'
    try {
        $missingDiagnosticsDetected = $false
        try {
            & $stager `
                -ProductInputManifest $fixture.ProductInputManifest `
                -TemplateRoot $fixture.TemplateRoot `
                -EngineAssetRoot $fixture.EngineAssetRoot `
                -ProjectRoot $fixture.ProjectRoot `
                -StagingRoot $fixture.StagingRoot `
                -RunId $missingDiagnosticsRunId `
                -RepeatCount 1 `
                -TimeoutSeconds 10 `
                -AllowUnsafeStagingRoot | Out-Null
        }
        catch {
            $missingDiagnosticsDetected = $_.Exception.Message -match 'runtime_product_frame_diagnostics'
        }
        Assert-True $missingDiagnosticsDetected 'A runtime product that omits structured diagnostics was not rejected.'
    }
    finally {
        Clear-MvpStagingFixtureControl -Fixture $fixture -Scope Project
    }

    $diagnosticFileBudgetRunId = 'fixture-diagnostic-file-budget'
    Set-MvpStagingFixtureControl -Fixture $fixture -Scope Project -Control 'diagnostic-file-flood'
    try {
        $diagnosticFileBudgetDetected = $false
        try {
            & $stager `
                -ProductInputManifest $fixture.ProductInputManifest `
                -TemplateRoot $fixture.TemplateRoot `
                -EngineAssetRoot $fixture.EngineAssetRoot `
                -ProjectRoot $fixture.ProjectRoot `
                -StagingRoot $fixture.StagingRoot `
                -RunId $diagnosticFileBudgetRunId `
                -RepeatCount 1 `
                -TimeoutSeconds 10 `
                -AllowUnsafeStagingRoot | Out-Null
        }
        catch {
            $diagnosticFileBudgetDetected = $true
        }
        Assert-True $diagnosticFileBudgetDetected 'A product diagnostic directory that exceeds its file-count budget was not rejected.'
        $diagnosticFileBudgetTerminal = @(Get-ProcessJournalEntries -StageRoot (Join-Path $fixture.StagingRoot $diagnosticFileBudgetRunId) | Where-Object { $_.phase -eq 'runtime-1' })
        Assert-True ($diagnosticFileBudgetTerminal.Count -eq 1) 'Diagnostic file-count rejection did not publish one runtime terminal journal entry.'
        Assert-True ($diagnosticFileBudgetTerminal[0].outcome -eq 'supervisor_failed') 'Diagnostic file-count rejection lost its supervisor-failed terminal outcome.'
        Assert-True ($diagnosticFileBudgetTerminal[0].supervisor_failure.kind -eq 'progress_probe_failed') 'Diagnostic file-count rejection lost its progress-probe failure kind.'
    }
    finally {
        Clear-MvpStagingFixtureControl -Fixture $fixture -Scope Project
    }

    $diagnosticDepthBudgetRunId = 'fixture-diagnostic-depth-budget'
    Set-MvpStagingFixtureControl -Fixture $fixture -Scope Project -Control 'diagnostic-depth-overflow'
    try {
        $diagnosticDepthBudgetDetected = $false
        try {
            & $stager `
                -ProductInputManifest $fixture.ProductInputManifest `
                -TemplateRoot $fixture.TemplateRoot `
                -EngineAssetRoot $fixture.EngineAssetRoot `
                -ProjectRoot $fixture.ProjectRoot `
                -StagingRoot $fixture.StagingRoot `
                -RunId $diagnosticDepthBudgetRunId `
                -RepeatCount 1 `
                -TimeoutSeconds 10 `
                -AllowUnsafeStagingRoot | Out-Null
        }
        catch {
            $diagnosticDepthBudgetDetected = $true
        }
        Assert-True $diagnosticDepthBudgetDetected 'A product diagnostic directory that exceeds its depth budget was not rejected.'
        $diagnosticDepthBudgetTerminal = @(Get-ProcessJournalEntries -StageRoot (Join-Path $fixture.StagingRoot $diagnosticDepthBudgetRunId) | Where-Object { $_.phase -eq 'runtime-1' })
        Assert-True ($diagnosticDepthBudgetTerminal.Count -eq 1) 'Diagnostic depth rejection did not publish one runtime terminal journal entry.'
        Assert-True ($diagnosticDepthBudgetTerminal[0].outcome -eq 'supervisor_failed') 'Diagnostic depth rejection lost its supervisor-failed terminal outcome.'
        Assert-True ($diagnosticDepthBudgetTerminal[0].supervisor_failure.kind -eq 'progress_probe_failed') 'Diagnostic depth rejection lost its progress-probe failure kind.'
    }
    finally {
        Clear-MvpStagingFixtureControl -Fixture $fixture -Scope Project
    }

    $materialFallbackRunId = 'fixture-runtime-material-fallback'
    Set-MvpStagingFixtureControl -Fixture $fixture -Scope Project -Control 'material-fallback'
    try {
        $materialFallbackDetected = $false
        try {
            & $stager `
                -ProductInputManifest $fixture.ProductInputManifest `
                -TemplateRoot $fixture.TemplateRoot `
                -EngineAssetRoot $fixture.EngineAssetRoot `
                -ProjectRoot $fixture.ProjectRoot `
                -StagingRoot $fixture.StagingRoot `
                -RunId $materialFallbackRunId `
                -RepeatCount 1 `
                -TimeoutSeconds 10 `
                -AllowUnsafeStagingRoot | Out-Null
        }
        catch {
            $materialFallbackDetected = $_.Exception.Message -match 'material_fallback_count'
        }
        Assert-True $materialFallbackDetected 'A runtime product that used a fallback material was not rejected.'
    }
    finally {
        Clear-MvpStagingFixtureControl -Fixture $fixture -Scope Project
    }

    $timeoutRunId = 'fixture-timeout-process-tree'
    $timeoutStage = Join-Path $fixture.StagingRoot $timeoutRunId
    Set-MvpStagingFixtureControl -Fixture $fixture -Scope Project -Control 'timeout-with-child'
    try {
        $timedOut = $false
        try {
            & $stager `
                -ProductInputManifest $fixture.ProductInputManifest `
                -TemplateRoot $fixture.TemplateRoot `
                -EngineAssetRoot $fixture.EngineAssetRoot `
                -ProjectRoot $fixture.ProjectRoot `
                -StagingRoot $fixture.StagingRoot `
                -RunId $timeoutRunId `
                -RepeatCount 1 `
                -TimeoutSeconds 5 `
                -AllowUnsafeStagingRoot | Out-Null
        }
        catch {
            $timedOut = $_.Exception.Message -match 'did not exit within'
        }
        Assert-True $timedOut 'A timed-out staged product was not reported as a failure.'
        $timeoutStderr = Join-Path $timeoutStage 'logs/runtime-1.stderr.log'
        Assert-True (Test-Path -LiteralPath $timeoutStderr) 'A timed-out staged product did not preserve its stderr log.'
        Assert-True ([IO.File]::ReadAllText($timeoutStderr) -match 'fixture timeout emitted before termination') 'A timed-out staged product stderr stream was not drained before failure.'
        $timeoutJournal = @(Get-ProcessJournalEntries -StageRoot $timeoutStage | Where-Object { $_.phase -eq 'runtime-1' })
        Assert-True ($timeoutJournal.Count -eq 1) 'Timed-out runtime did not emit exactly one journal entry.'
        Assert-ProcessJournalEntry -Entry $timeoutJournal[0] -Phase 'runtime-1' -Outcome 'timed_out' -ExitCode $null
        $timeoutReceipt = Get-MvpStagingTerminalReceiptFixture -StagingRoot $fixture.StagingRoot -RunId $timeoutRunId
        Assert-True ($timeoutReceipt.outcome -eq 'timed_out') 'Timed-out staging run did not publish a timed-out terminal outcome.'
        Assert-True $timeoutReceipt.staging_directory_published 'Timed-out staging run lost its published stage identity.'
        Assert-True ($timeoutReceipt.cleanup.outcome -eq 'succeeded') 'Timed-out staging run did not record successful process cleanup.'
        Start-Sleep -Milliseconds 250
        $timeoutPids = @(
            Get-CimInstance Win32_Process | Where-Object {
                $executablePath = [string]$_.ExecutablePath
                -not [string]::IsNullOrWhiteSpace($executablePath) -and
                    $executablePath.StartsWith(
                        [IO.Path]::GetFullPath($timeoutStage).TrimEnd('\\') + [IO.Path]::DirectorySeparatorChar,
                        [StringComparison]::OrdinalIgnoreCase
                    )
            } | Select-Object -ExpandProperty ProcessId
        )
        Assert-True ($timeoutPids.Count -eq 0) 'A timeout must terminate every staged product child process.'
    }
    finally {
        Clear-MvpStagingFixtureControl -Fixture $fixture -Scope Project
        $timeoutPrefix = [IO.Path]::GetFullPath($timeoutStage).TrimEnd('\\') + [IO.Path]::DirectorySeparatorChar
        $timeoutPids = @(
            Get-CimInstance Win32_Process | Where-Object {
                $executablePath = [string]$_.ExecutablePath
                -not [string]::IsNullOrWhiteSpace($executablePath) -and
                    $executablePath.StartsWith($timeoutPrefix, [StringComparison]::OrdinalIgnoreCase)
            } | Select-Object -ExpandProperty ProcessId
        )
        foreach ($processId in $timeoutPids) {
            Stop-Process -Id $processId -Force -ErrorAction SilentlyContinue
        }
        foreach ($processId in $timeoutPids) {
            Wait-Process -Id $processId -Timeout 5 -ErrorAction SilentlyContinue
        }
    }

    $nonzeroRunId = 'fixture-nonzero-process-tree'
    $nonzeroStage = Join-Path $fixture.StagingRoot $nonzeroRunId
    Set-MvpStagingFixtureControl -Fixture $fixture -Scope Project -Control 'fail-with-child'
    try {
        $nonzeroExitDetected = $false
        $nonzeroExitError = '<no exception>'
        try {
            & $stager `
                -ProductInputManifest $fixture.ProductInputManifest `
                -TemplateRoot $fixture.TemplateRoot `
                -EngineAssetRoot $fixture.EngineAssetRoot `
                -ProjectRoot $fixture.ProjectRoot `
                -StagingRoot $fixture.StagingRoot `
                -RunId $nonzeroRunId `
                -RepeatCount 1 `
                -TimeoutSeconds 10 `
                -AllowUnsafeStagingRoot | Out-Null
        }
        catch {
            $nonzeroExitError = $_.Exception.Message
            $nonzeroExitDetected = $nonzeroExitError -match 'exited with code 23'
        }
        Assert-True `
            $nonzeroExitDetected `
            "A nonzero staged product exit was not reported as a failure. Actual: '$nonzeroExitError'."
        $nonzeroJournal = @(Get-ProcessJournalEntries -StageRoot $nonzeroStage | Where-Object { $_.phase -eq 'runtime-1' })
        Assert-True ($nonzeroJournal.Count -eq 1) 'Nonzero runtime did not emit exactly one journal entry.'
        Assert-ProcessJournalEntry -Entry $nonzeroJournal[0] -Phase 'runtime-1' -Outcome 'crashed' -ExitCode 23
        Start-Sleep -Milliseconds 250
        $nonzeroPids = @(
            Get-CimInstance Win32_Process | Where-Object {
                $executablePath = [string]$_.ExecutablePath
                -not [string]::IsNullOrWhiteSpace($executablePath) -and
                    $executablePath.StartsWith(
                        [IO.Path]::GetFullPath($nonzeroStage).TrimEnd('\\') + [IO.Path]::DirectorySeparatorChar,
                        [StringComparison]::OrdinalIgnoreCase
                    )
            } | Select-Object -ExpandProperty ProcessId
        )
        Assert-True ($nonzeroPids.Count -eq 0) 'A nonzero staged product exit must not leave a staged child process.'
    }
    finally {
        Clear-MvpStagingFixtureControl -Fixture $fixture -Scope Project
        $nonzeroPrefix = [IO.Path]::GetFullPath($nonzeroStage).TrimEnd('\\') + [IO.Path]::DirectorySeparatorChar
        $nonzeroPids = @(
            Get-CimInstance Win32_Process | Where-Object {
                $executablePath = [string]$_.ExecutablePath
                -not [string]::IsNullOrWhiteSpace($executablePath) -and
                    $executablePath.StartsWith($nonzeroPrefix, [StringComparison]::OrdinalIgnoreCase)
            } | Select-Object -ExpandProperty ProcessId
        )
        foreach ($processId in $nonzeroPids) {
            Stop-Process -Id $processId -Force -ErrorAction SilentlyContinue
        }
        foreach ($processId in $nonzeroPids) {
            Wait-Process -Id $processId -Timeout 5 -ErrorAction SilentlyContinue
        }
    }

    $leakedRunId = 'fixture-leaked-process'
    $leakedStage = Join-Path $fixture.StagingRoot $leakedRunId
    Set-MvpStagingFixtureControl -Fixture $fixture -Scope Project -Control 'leak-staged-child'
    try {
        $leakedProcessDetected = $false
        $leakedProcessError = '<no exception>'
        try {
            & $stager `
                -ProductInputManifest $fixture.ProductInputManifest `
                -TemplateRoot $fixture.TemplateRoot `
                -EngineAssetRoot $fixture.EngineAssetRoot `
                -ProjectRoot $fixture.ProjectRoot `
                -StagingRoot $fixture.StagingRoot `
                -RunId $leakedRunId `
                -RepeatCount 1 `
                -TimeoutSeconds 10 `
                -AllowUnsafeStagingRoot | Out-Null
        }
        catch {
            $leakedProcessError = $_.Exception.Message
            $leakedProcessDetected = $leakedProcessError -match 'Staged process job retained a descendant after its root product exited|did not exit within'
        }
        Assert-True $leakedProcessDetected "A staged child process that outlives product exit was not reported. Actual: '$leakedProcessError'."
    }
    finally {
        Clear-MvpStagingFixtureControl -Fixture $fixture -Scope Project
        $leakedPidPath = Join-Path $leakedStage 'logs/runtime-1.diagnostics/leaked-child.pid'
        if (Test-Path -LiteralPath $leakedPidPath) {
            $leakedPid = [int](Get-Content -LiteralPath $leakedPidPath -Raw)
            Stop-Process -Id $leakedPid -Force -ErrorAction SilentlyContinue
            Wait-Process -Id $leakedPid -Timeout 5 -ErrorAction SilentlyContinue
        }
        $leakedPrefix = [IO.Path]::GetFullPath($leakedStage).TrimEnd('\\') + [IO.Path]::DirectorySeparatorChar
        $leakedPids = @(
            Get-CimInstance Win32_Process | Where-Object {
                $executablePath = [string]$_.ExecutablePath
                -not [string]::IsNullOrWhiteSpace($executablePath) -and
                    $executablePath.StartsWith($leakedPrefix, [StringComparison]::OrdinalIgnoreCase)
            } | Select-Object -ExpandProperty ProcessId
        )
        foreach ($processId in $leakedPids) {
            Stop-Process -Id $processId -Force -ErrorAction SilentlyContinue
        }
        foreach ($processId in $leakedPids) {
            Wait-Process -Id $processId -Timeout 5 -ErrorAction SilentlyContinue
        }
    }

    $externalLeakRunId = 'fixture-external-process-job-containment'
    $externalLeakStage = Join-Path $fixture.StagingRoot $externalLeakRunId
    Set-MvpStagingFixtureControl -Fixture $fixture -Scope Project -Control 'leak-external-child'
    $externalChildPid = $null
    try {
        $externalLeakDetected = $false
        $externalLeakError = '<no exception>'
        try {
            & $stager `
                -ProductInputManifest $fixture.ProductInputManifest `
                -TemplateRoot $fixture.TemplateRoot `
                -EngineAssetRoot $fixture.EngineAssetRoot `
                -ProjectRoot $fixture.ProjectRoot `
                -StagingRoot $fixture.StagingRoot `
                -RunId $externalLeakRunId `
                -RepeatCount 1 `
                -TimeoutSeconds 10 `
                -AllowUnsafeStagingRoot | Out-Null
        }
        catch {
            $externalLeakError = $_.Exception.Message
            $externalLeakDetected = $externalLeakError -match 'process job.*descendant|descendant.*process job|did not exit within'
        }
        $externalPidPath = Join-Path $externalLeakStage 'logs/runtime-1.diagnostics/escaped-child.pid'
        if (Test-Path -LiteralPath $externalPidPath) {
            $externalChildPid = [int](Get-Content -LiteralPath $externalPidPath -Raw)
        }
        Assert-True $externalLeakDetected ("A child process outside the staging executable directory must be contained and rejected by the staged product Job Object. Actual error: $externalLeakError")
        if ($null -ne $externalChildPid) {
            Assert-True (-not (Get-Process -Id $externalChildPid -ErrorAction SilentlyContinue)) 'A Job Object-contained external child process survived its root product exit.'
        }
    }
    finally {
        Clear-MvpStagingFixtureControl -Fixture $fixture -Scope Project
        if ($null -ne $externalChildPid) {
            Stop-Process -Id $externalChildPid -Force -ErrorAction SilentlyContinue
            Wait-Process -Id $externalChildPid -Timeout 5 -ErrorAction SilentlyContinue
        }
    }

    $boundedLogRunId = 'fixture-bounded-process-output'
    $boundedLogStage = Join-Path $fixture.StagingRoot $boundedLogRunId
    Set-MvpStagingFixtureControl -Fixture $fixture -Scope Project -Control 'spam-process-output'
    try {
        & $stager `
            -ProductInputManifest $fixture.ProductInputManifest `
            -TemplateRoot $fixture.TemplateRoot `
            -EngineAssetRoot $fixture.EngineAssetRoot `
            -ProjectRoot $fixture.ProjectRoot `
            -StagingRoot $fixture.StagingRoot `
            -RunId $boundedLogRunId `
            -RepeatCount 1 `
            -TimeoutSeconds 10 `
            -MaxProcessLogBytes 1024 `
            -AllowUnsafeStagingRoot | Out-Null
        $boundedStdout = Join-Path $boundedLogStage 'logs/runtime-1.stdout.log'
        $boundedStderr = Join-Path $boundedLogStage 'logs/runtime-1.stderr.log'
        $boundedStdoutTail = Join-Path $boundedLogStage 'logs/runtime-1.stdout.tail.log'
        $boundedStderrTail = Join-Path $boundedLogStage 'logs/runtime-1.stderr.tail.log'
        Assert-True ([IO.FileInfo]::new($boundedStdout).Length -le 1024) 'Staged stdout exceeded its configured byte limit.'
        Assert-True ([IO.FileInfo]::new($boundedStderr).Length -le 1024) 'Staged stderr exceeded its configured byte limit.'
        Assert-True ([IO.FileInfo]::new($boundedStdoutTail).Length -le 1024) 'Staged stdout tail exceeded its configured byte limit.'
        Assert-True ([IO.FileInfo]::new($boundedStderrTail).Length -le 1024) 'Staged stderr tail exceeded its configured byte limit.'
        $boundedJournal = @(Get-ProcessJournalEntries -StageRoot $boundedLogStage | Where-Object { $_.phase -eq 'runtime-1' })
        Assert-True ($boundedJournal.Count -eq 1) 'Bounded-output runtime did not emit exactly one journal entry.'
        Assert-True ([Int64]$boundedJournal[0].stdout.dropped_bytes -gt 0) 'Bounded stdout did not record dropped-byte evidence.'
        Assert-True ([Int64]$boundedJournal[0].stderr.dropped_bytes -gt 0) 'Bounded stderr did not record dropped-byte evidence.'
        Assert-True ([string]$boundedJournal[0].stdout.tail_file_name -eq 'runtime-1.stdout.tail.log') 'Bounded stdout journal did not identify its tail artifact.'
        Assert-True ([string]$boundedJournal[0].stderr.tail_file_name -eq 'runtime-1.stderr.tail.log') 'Bounded stderr journal did not identify its tail artifact.'
        Assert-True ([Int64]$boundedJournal[0].stdout.tail_retained_bytes -le 1024) 'Bounded stdout tail did not record its byte ceiling.'
        Assert-True ([Int64]$boundedJournal[0].stderr.tail_retained_bytes -le 1024) 'Bounded stderr tail did not record its byte ceiling.'
    }
    finally {
        Clear-MvpStagingFixtureControl -Fixture $fixture -Scope Project
    }

    $originalEditorRuntimeLibrary = $fixture.EditorRuntimeLibrary
    $fixture.EditorRuntimeLibrary = $fixture.RuntimeLibrary
    $fixture.ProductInputManifest = New-MvpProductInputManifestFixture -Fixture $fixture
    $sameLibraryRejected = $false
    try {
        Invoke-MvpStager -Fixture $fixture -RunId 'fixture-shared-profile-library' | Out-Null
    }
    catch {
        $sameLibraryRejected = $_.Exception.Message -match 'distinct physical profile artifacts'
    }
    finally {
        $fixture.EditorRuntimeLibrary = $originalEditorRuntimeLibrary
        $fixture.ProductInputManifest = New-MvpProductInputManifestFixture -Fixture $fixture
    }
    Assert-True $sameLibraryRejected 'Staging accepted one physical runtime DLL for both product profiles.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixture.StagingRoot 'fixture-shared-profile-library'))) 'Shared profile-library rejection left a partial product directory.'

    $runtimeHardLink = Join-Path $fixture.Root 'runtime-library-hard-link.dll'
    New-Item -ItemType HardLink -Path $runtimeHardLink -Target $fixture.RuntimeLibrary | Out-Null
    $fixture.EditorRuntimeLibrary = $runtimeHardLink
    $fixture.ProductInputManifest = New-MvpProductInputManifestFixture -Fixture $fixture
    $hardLinkedLibraryRejected = $false
    try {
        Invoke-MvpStager -Fixture $fixture -RunId 'fixture-hard-linked-profile-library' | Out-Null
    }
    catch {
        $hardLinkedLibraryRejected = $_.Exception.Message -match 'distinct physical profile artifacts'
    }
    finally {
        $fixture.EditorRuntimeLibrary = $originalEditorRuntimeLibrary
        Remove-Item -LiteralPath $runtimeHardLink -Force -ErrorAction SilentlyContinue
        $fixture.ProductInputManifest = New-MvpProductInputManifestFixture -Fixture $fixture
    }
    Assert-True $hardLinkedLibraryRejected 'Staging accepted a hard-linked runtime DLL for both product profiles.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixture.StagingRoot 'fixture-hard-linked-profile-library'))) 'Hard-linked profile-library rejection left a partial product directory.'

    $equalContentEditorLibrary = Join-Path $fixture.Root 'editor-runtime-equal-content.dll'
    Copy-Item -LiteralPath $fixture.RuntimeLibrary -Destination $equalContentEditorLibrary
    $fixture.EditorRuntimeLibrary = $equalContentEditorLibrary
    $fixture.ProductInputManifest = New-MvpProductInputManifestFixture -Fixture $fixture
    $equalContentLibraryAccepted = $false
    try {
        Invoke-MvpStager -Fixture $fixture -RunId 'fixture-equal-content-profile-library' | Out-Null
        $equalContentLibraryAccepted = $true
    }
    finally {
        $fixture.EditorRuntimeLibrary = $originalEditorRuntimeLibrary
        Remove-Item -LiteralPath $equalContentEditorLibrary -Force -ErrorAction SilentlyContinue
        $fixture.ProductInputManifest = New-MvpProductInputManifestFixture -Fixture $fixture
    }
    Assert-True $equalContentLibraryAccepted 'Staging rejected distinct profile DLL files solely because their content matched.'

    Remove-Item -LiteralPath $fixture.RuntimeLibrary -Force
    $failed = $false
    try {
        Invoke-MvpStager -Fixture $fixture -RunId 'fixture-failure' | Out-Null
    }
    catch {
        $failed = $_.Exception.Message -match 'runtime-library/runtime.*does not exist'
    }
    Assert-True $failed 'A missing runtime library did not produce an actionable failure.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixture.StagingRoot 'fixture-failure'))) 'Failed staging left a partial product directory.'

    Write-Host 'MVP staging contract passed'
}
finally {
    Remove-MvpStagingFixtureBuildSet -Fixture $fixture
    if (Test-Path -LiteralPath $fixture.Root) {
        Remove-MvpTestFixtureRoot -Path $fixture.Root
    }
}
