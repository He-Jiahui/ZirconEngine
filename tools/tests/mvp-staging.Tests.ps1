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

function Assert-True {
    param([bool]$Condition, [string]$Message)
    if (-not $Condition) { throw $Message }
}

function New-MvpStagingFixture {
    $root = Join-Path ([IO.Path]::GetTempPath()) ('zircon mvp staging-' + [guid]::NewGuid().ToString('N'))
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
            if (Environment.GetEnvironmentVariable("ZIRCON_MVP_FIXTURE_FAIL_CREATE_WITH_CHILD") == "1")
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
                Environment.GetEnvironmentVariable("ZIRCON_MVP_FIXTURE_SKIP_EDITOR_CAPTURE") != "1")
            {
                if (Environment.GetEnvironmentVariable("ZIRCON_MVP_FIXTURE_SKIP_EDITOR_CAPTURE_FILE") != "1")
                {
                    WriteVisibleCapture(creationCapturePath);
                }
                creationCaptureDiagnostic = "editor_product_frame_capture_written" + Environment.NewLine;
                creationProductDiagnostic =
                    "editor_product_frame_diagnostics project_path=" + Uri.EscapeDataString(projectRoot) +
                    " selected_node_id=3 selected_node_name=Cube inspector_translation_x=0 inspector_translation_y=0 inspector_translation_z=0" +
                    Environment.NewLine;
            }
            var creationDiagnosticRoot = Environment.GetEnvironmentVariable("ZIRCON_LOG_ROOT");
            if (!String.IsNullOrWhiteSpace(creationDiagnosticRoot))
            {
                Directory.CreateDirectory(creationDiagnosticRoot);
                var projectOpenDiagnostic = "";
                if (Environment.GetEnvironmentVariable("ZIRCON_MVP_FIXTURE_SKIP_PROJECT_OPEN_DIAGNOSTIC") != "1")
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

        var automationIndex = Array.IndexOf(args, "--automation");
        if (automationIndex >= 0)
        {
            var automationProjectIndex = Array.IndexOf(args, "--project");
            if (automationProjectIndex < 0 || automationProjectIndex + 1 >= args.Length ||
                Array.IndexOf(args, "--headless") < 0 ||
                automationIndex + 1 >= args.Length ||
                !File.Exists(args[automationIndex + 1]))
            {
                return 30;
            }
            var request = File.ReadAllText(args[automationIndex + 1]);
            var hasSelection = request.IndexOf("Hierarchy", StringComparison.Ordinal) >= 0 &&
                request.IndexOf("SelectCube", StringComparison.Ordinal) >= 0;
            var hasTransform = request.IndexOf("TransformPositionXCommit", StringComparison.Ordinal) >= 0;
            var hasSave = request.IndexOf("SaveProject", StringComparison.Ordinal) >= 0;
            if (!hasSelection || hasTransform != hasSave)
            {
                return 31;
            }
            if (hasTransform && Environment.GetEnvironmentVariable("ZIRCON_MVP_FIXTURE_FAIL_AUTOMATION_WITH_CHILD") == "1")
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
            var reportedProjectRoot = Environment.GetEnvironmentVariable("ZIRCON_MVP_FIXTURE_WRONG_AUTOMATION_PROJECT") == "1"
                ? Path.Combine(Path.GetTempPath(), "wrong-automation-project")
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
            var records = hasTransform
                ? "{\"binding_path\":\"Hierarchy/SelectCube:onClick\",\"source\":\"Cli\"}," +
                    "{\"binding_path\":\"Inspector/TransformPositionXCommit:onSubmit\",\"source\":\"Cli\",\"operation_id\":\"inspector.field.apply_batch\",\"transaction_id\":1}," +
                    "{\"binding_path\":\"WorkbenchMenuBar/SaveProject:onClick\",\"source\":\"Cli\",\"operation_id\":\"file.project.save\",\"save_generation\":2}"
                : "{\"binding_path\":\"Hierarchy/SelectCube:onClick\",\"source\":\"Cli\"}";
            Console.WriteLine(
                "{\"project_path\":\"" + projectPath +
                "\",\"project_identity\":\"" + automationProjectIdentity +
                "\",\"manifest_identity\":\"" + automationProjectIdentity + "@v1" +
                "\",\"scene_uri\":\"res://scenes/main.scene.toml\"" +
                ",\"selected_model_resource_id\":\"fixture-cube-model-resource\"" +
                ",\"selected_material_resource_id\":\"fixture-default-material-resource\"" +
                ",\"opened_project_inspection_generation\":1,\"records\":[" + records + "]," +
                "\"snapshot\":{\"project_open\":true,\"scene_entry_count\":3,\"selected_node_id\":3,\"selected_node_name\":\"Cube\",\"inspector_translation\":[\"" + translationX + "\",\"0\",\"0\"]}}"
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
        var logPath = Path.Combine(diagnosticRoot, "fixture.log");
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
            if (Environment.GetEnvironmentVariable("ZIRCON_MVP_FIXTURE_SKIP_RUNTIME_DIAGNOSTICS") != "1")
            {
                var inputEvidence = Environment.GetEnvironmentVariable("ZIRCON_RUNTIME_MVP_INPUT_PROBE") == "1"
                    ? " input_pointer_move_count=1 input_mouse_button_press_count=1 input_mouse_button_release_count=1 input_keyboard_press_count=1 input_keyboard_release_count=1"
                    : "";
                var materialFallbackCount = Environment.GetEnvironmentVariable("ZIRCON_MVP_FIXTURE_MATERIAL_FALLBACK") == "1" ? "1" : "0";
                File.AppendAllText(logPath,
                    "runtime_product_frame_diagnostics frame_index=1 viewport=16x16 project_identity=" + projectIdentity + " scene_uri=res://scenes/main.scene.toml selected_model_resource_id=fixture-cube-model-resource selected_material_resource_id=fixture-default-material-resource render_backend=fixture-wgpu render_adapter=Fixture WGPU Adapter render_adapter_type=discrete_gpu device_max_bind_groups=5 device_max_texture_dimension_2d=16384 device_max_texture_array_layers=256 device_max_sampled_textures_per_shader_stage=16 device_max_storage_buffers_per_shader_stage=8 device_max_storage_buffer_binding_size=134217728 graph_executed_pass_count=1 mesh_draw_count=1 directional_light_count=1 material_fallback_count=" + materialFallbackCount + " material_validation_error_count=0" + inputEvidence +
                    Environment.NewLine);
            }
            var capturePath = Environment.GetEnvironmentVariable("ZIRCON_RUNTIME_CAPTURE_FRAME_PNG");
            if (!String.IsNullOrWhiteSpace(capturePath) &&
                Environment.GetEnvironmentVariable("ZIRCON_MVP_FIXTURE_SKIP_RUNTIME_CAPTURE") != "1")
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
                Environment.GetEnvironmentVariable("ZIRCON_MVP_FIXTURE_SKIP_EDITOR_CAPTURE") != "1")
            {
                if (Environment.GetEnvironmentVariable("ZIRCON_MVP_FIXTURE_SKIP_EDITOR_CAPTURE_FILE") != "1")
                {
                    WriteVisibleCapture(editorCapturePath, true);
                }
                File.AppendAllText(logPath, "editor_product_frame_capture_written" + Environment.NewLine);
                File.AppendAllText(logPath,
                    "editor_product_frame_diagnostics project_path=" + Uri.EscapeDataString(args[projectIndex + 1]) +
                    " selected_node_id=3 selected_node_name=Cube inspector_translation_x=42 inspector_translation_y=0 inspector_translation_z=0" +
                    Environment.NewLine);
            }
        }
        if (Environment.GetEnvironmentVariable("ZIRCON_MVP_FIXTURE_LEAK_STAGED_CHILD") == "1")
        {
            using (var child = Process.Start(new ProcessStartInfo
            {
                FileName = Process.GetCurrentProcess().MainModule.FileName,
                Arguments = "--fixture-child",
                UseShellExecute = true,
            }))
            {
            }
        }
        if (Environment.GetEnvironmentVariable("ZIRCON_MVP_FIXTURE_FAIL_WITH_CHILD") == "1")
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
        if (Environment.GetEnvironmentVariable("ZIRCON_MVP_FIXTURE_TIMEOUT_WITH_CHILD") == "1")
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

    return [pscustomobject]@{
        Root = $root
        RuntimeExecutable = Join-Path $build 'zircon_runtime.exe'
        EditorExecutable = Join-Path $build 'zircon_editor.exe'
        RuntimeLibrary = Join-Path $build 'zircon_runtime.dll'
        EditorRuntimeLibrary = Join-Path $build 'zircon_runtime_editor.dll'
        TemplateRoot = Join-Path $root 'templates\projects'
        EngineAssetRoot = $engineAssets
        ProjectRoot = $project
        StagingRoot = Join-Path $root 'staging'
    }
}

function Invoke-MvpStager {
    param(
        [pscustomobject]$Fixture,
        [string]$RunId = 'fixture-run'
    )

    return & $stager `
        -RuntimeExecutable $Fixture.RuntimeExecutable `
        -EditorExecutable $Fixture.EditorExecutable `
        -RuntimeLibrary $Fixture.RuntimeLibrary `
        -EditorRuntimeLibrary $Fixture.EditorRuntimeLibrary `
        -TemplateRoot $Fixture.TemplateRoot `
        -EngineAssetRoot $Fixture.EngineAssetRoot `
        -ProjectRoot $Fixture.ProjectRoot `
        -StagingRoot $Fixture.StagingRoot `
        -SourceFingerprint 'fixture-source-fingerprint' `
        -RunId $RunId `
        -NoLaunch `
        -AllowUnsafeStagingRoot
}

$stagerSource = Get-Content -LiteralPath $stager -Raw -Encoding UTF8
Assert-True ($stagerSource -notmatch '`\$') 'MVP staging diagnostics must interpolate their input values.'
Assert-True ($stagerSource -match 'could not launch from') 'MVP staging launch failures must identify the staged executable path.'
Assert-True ($stagerSource -match 'first_frame_exit_requested') 'MVP staging must record that each product used the first-frame exit path.'
Assert-True ($stagerSource -match 'ZIRCON_LOG_ROOT') 'MVP staging must isolate product diagnostics under the stage directory.'
Assert-True ($stagerSource -match 'ZIRCON_LOG_FILTER') 'MVP staging must override inherited host log filtering for product evidence.'
Assert-True ($stagerSource -match 'ZIRCON_ASSET_ROOT') 'MVP staging must force products to resolve staged engine assets.'
Assert-True ($stagerSource -match 'runtime_first_frame_presented') 'MVP staging must verify the runtime first-presented-frame diagnostic from its log files.'
Assert-True ($stagerSource -match 'editor_first_frame_presented') 'MVP staging must verify the editor first-presented-frame diagnostic from its log files.'
Assert-True ($stagerSource -match 'runtime_process_teardown_complete') 'MVP staging must verify runtime teardown after the first presented frame.'
Assert-True ($stagerSource -match 'editor_process_teardown_complete') 'MVP staging must verify editor teardown after the first presented frame.'
Assert-True ($stagerSource -match 'ZIRCON_RUNTIME_CAPTURE_FRAME_PNG') 'MVP staging must request runtime first-frame PNG evidence only for the staged runtime product.'
Assert-True ($stagerSource -match 'ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG') 'MVP staging must request a native editor first-frame PNG only for the selected staged editor run.'
Assert-True ($stagerSource -match 'ZIRCON_RUNTIME_MVP_INPUT_PROBE') 'MVP staging must request the runtime host input probe before first-frame evidence.'
Assert-True ($stagerSource -match 'Get-MvpRuntimeFrameCaptureEvidence') 'MVP staging must inspect the captured runtime PNG rather than only checking its path.'
Assert-True ($stagerSource -match 'Get-MvpEditorWindowCaptureEvidence') 'MVP staging must inspect the captured editor window PNG rather than only checking its path.'
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
Assert-True ($stagerSource -match 'Assert-MvpStagingProcessesReleased') 'MVP staging must reject a staged executable that survives its product exit.'
Assert-True ($stagerSource -match 'Assert-MvpStagingProcessesReleased -StageDirectory \$stageDirectory\s*\r?\n\s*if \(\$createExitCode -ne 0\)') 'MVP staging must check that project creation released all staged processes before rejecting a nonzero editor exit.'
Assert-True ($stagerSource -match 'Get-CimInstance Win32_Process') 'MVP staging must inspect live Windows processes for staged executable paths.'
Assert-True ($stagerSource -match 'ExecutablePath') 'MVP staging must scope lingering-process checks to the staging root.'
Assert-True ($stagerSource -match 'function Stop-MvpTimedOutStagedProcessTree') 'MVP staging must sweep staged processes after a timeout race.'
Assert-True ($stagerSource -match '\$ProcessState\.staged_product_root') 'MVP staging timeout cleanup must receive the full staging run root.'
Assert-True ($stagerSource -match '\$timeoutCleanupErrors\.Add\(\$_\.Exception\.Message\)') 'MVP staging must retain timeout cleanup failures until after process stream collection.'
$stderrWriteIndex = $stagerSource.IndexOf('[IO.File]::WriteAllText($StderrPath', [StringComparison]::Ordinal)
$timeoutThrowIndex = $stagerSource.IndexOf('throw [TimeoutException]::new', [StringComparison]::Ordinal)
Assert-True ($stderrWriteIndex -ge 0 -and $timeoutThrowIndex -gt $stderrWriteIndex) 'MVP staging must persist process logs before reporting a timeout or timeout-cleanup failure.'
Assert-True ($stagerSource -match 'diff --no-ext-diff --binary HEAD') 'MVP source fingerprints must include the current tracked working-tree content.'
Assert-True ($stagerSource -match 'ls-files --others --exclude-standard') 'MVP source fingerprints must enumerate untracked source inputs.'
Assert-True ($stagerSource -match 'untracked source input') 'MVP source fingerprints must hash each untracked source input.'
Assert-True ($stagerSource -match 'function Get-FileSha256') 'MVP staging must hash files without a PowerShell module auto-load dependency.'
Assert-True ($stagerSource -notmatch 'Get-FileHash') 'MVP staging must not require the Get-FileHash cmdlet in the Windows PowerShell host.'
Assert-True ($stagerSource -notmatch '(?m)^\s*\[string\]\$Toolchain\s*[,)]') 'MVP staging must not accept caller-provided toolchain provenance.'
Assert-True ($stagerSource -notmatch '(?m)^\s*\[string\]\$Target\s*[,)]') 'MVP staging must not accept caller-provided target provenance.'
Assert-True ($stagerSource -match 'rustc -Vv') 'MVP staging must record toolchain provenance from the active Rust compiler.'
Assert-True ($stagerSource -match '\[switch\]\$CreateProject') 'MVP staging must expose an explicit fresh-project creation mode.'
Assert-True ($stagerSource -match 'CreateProject cannot be combined with ProjectRoot') 'MVP staging must reject a pre-existing project when staged creation is requested.'
Assert-True ($stagerSource -match 'CreateProject cannot be combined with NoLaunch') 'MVP staging must reject a project-creation request that would skip the staged editor launch.'
Assert-True ($stagerSource -match "'--create-project', '--project-name'") 'MVP staging must create projects through the normal staged editor CLI.'
Assert-True ($stagerSource -match "'--template', 'renderable-empty'") 'MVP staging fresh-project creation must use the renderable-empty template.'
Assert-True ($stagerSource -match 'Staged created project') 'MVP staging must verify that the staged editor created the canonical project root.'
Assert-True ($stagerSource -match 'AuthoringAutomationRequest') 'MVP staging must accept a staged normal editor automation request.'
Assert-True ($stagerSource -match 'Invoke-MvpStagedAuthoringAutomation') 'MVP staging must run authoring through the normal staged editor automation CLI.'
Assert-True ($stagerSource -match 'authoring_automation') 'MVP staging must preserve the structured authoring automation report in startup evidence.'
Assert-True ($stagerSource -match 'ReopenAutomationRequest') 'MVP staging must accept a second source-bound reopen automation request.'
Assert-True ($stagerSource -match 'reopen_automation') 'MVP staging must preserve repeated reopen automation reports in startup evidence.'
Assert-True ($stagerSource -match 'AttemptOffset') 'MVP staging must allocate a non-duplicate runtime attempt number after authoring.'
Assert-True ($stagerSource -match 'RepeatCount and ReopenRepeatCount to both equal 2') 'MVP staging must reject a reopen sequence that cannot satisfy the fixed F5 repeat contract.'
Assert-True ($stagerSource -match 'Get-MvpStagedFileEvidence') 'MVP staging must hash product stdout, stderr, and diagnostic evidence files.'
Assert-True ($stagerSource -notmatch 'source_path = \$SourcePath') 'MVP staging manifest must not retain absolute source input paths in uploaded evidence.'
Assert-True ($stagerSource -match 'project_creation') 'MVP staging must record the staged editor project-creation process as structured evidence.'
Assert-True ($stagerSource -match 'Get-MvpEditorProjectOpenEvidence') 'MVP staging must parse the normal editor project-open diagnostic from the creation process.'
Assert-True ($stagerSource -match 'Authoring automation diagnostic log') 'MVP staging must retain diagnostic evidence for normal editor automation processes.'

$defaultAuthoringAutomationPath = Join-Path $repoRoot 'tools\mvp\mvp-authoring-automation.json'
Assert-True (Test-Path -LiteralPath $defaultAuthoringAutomationPath -PathType Leaf) 'The source-bound F5 authoring automation request is missing.'
$defaultAuthoringAutomation = Get-Content -LiteralPath $defaultAuthoringAutomationPath -Raw -Encoding UTF8 | ConvertFrom-Json
Assert-True ($defaultAuthoringAutomation.bindings.Count -eq 3) 'The F5 authoring automation request must contain selection, transform, and save bindings.'
Assert-True ($defaultAuthoringAutomation.bindings[0].path.view_id -eq 'Hierarchy') 'The F5 authoring automation request must select the renderable template cube through Hierarchy.'
Assert-True ($defaultAuthoringAutomation.bindings[1].path.control_id -eq 'TransformPositionXCommit') 'The F5 authoring automation request must commit the X transform through Inspector.'
Assert-True ($defaultAuthoringAutomation.bindings[2].payload.MenuAction.action_id -eq 'workbench.project.save') 'The F5 authoring automation request must persist through the normal project save action.'
$defaultReopenAutomationPath = Join-Path $repoRoot 'tools\mvp\mvp-reopen-automation.json'
Assert-True (Test-Path -LiteralPath $defaultReopenAutomationPath -PathType Leaf) 'The source-bound F5 reopen automation request is missing.'
$defaultReopenAutomation = Get-Content -LiteralPath $defaultReopenAutomationPath -Raw -Encoding UTF8 | ConvertFrom-Json
Assert-True ($defaultReopenAutomation.bindings.Count -eq 1) 'The F5 reopen automation request must contain only its normal persisted-state selection binding.'
Assert-True ($defaultReopenAutomation.bindings[0].payload.SelectionCommand.SelectSceneNode.node_id -eq 3) 'The F5 reopen automation request must select the persisted renderable template Cube identity.'

$fixture = New-MvpStagingFixture
try {
    $result = Invoke-MvpStager -Fixture $fixture
    $manifestPath = Join-Path $result.staging_root 'staging-manifest.json'
    $manifest = Get-Content -Raw -Encoding UTF8 $manifestPath | ConvertFrom-Json

    Assert-True (Test-Path -LiteralPath (Join-Path $result.staging_root 'runtime\zircon_runtime.exe')) 'Runtime executable was not staged.'
    Assert-True (Test-Path -LiteralPath (Join-Path $result.staging_root 'editor\zircon_editor.exe')) 'Editor executable was not staged.'
    Assert-True (Test-Path -LiteralPath (Join-Path $result.staging_root 'runtime\assets\ui\editor\welcome.zui')) 'Runtime engine assets were not staged beside the executable.'
    Assert-True (Test-Path -LiteralPath (Join-Path $result.staging_root 'editor\assets\ui\runtime\fixtures\hud_overlay.zui')) 'Editor engine assets were not staged beside the executable.'
    Assert-True (Test-Path -LiteralPath (Join-Path $result.staging_root 'templates\renderable-empty\project.toml')) 'Project template was not staged.'
    Assert-True ($result.staged_project_root -eq (Join-Path $result.staging_root 'project')) 'Staging result did not report the staged project root.'
    Assert-True (Test-Path -LiteralPath (Join-Path $result.staging_root 'project\zircon-project.toml')) 'Project manifest was not staged.'
    Assert-True (Test-Path -LiteralPath (Join-Path $result.staging_root 'project\assets\scenes\main.scene.toml')) 'Project scene was not staged.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $result.staging_root 'project\.zircon\cache\stale.zasset'))) 'Machine-local project cache must not be staged.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $result.staging_root 'project\.zircon\registry\asset-registry.json'))) 'Machine-local project registry must not be staged.'
    Assert-True ($manifest.source_fingerprint -eq 'fixture-source-fingerprint') 'Manifest lost the source fingerprint.'
    Assert-True ($manifest.toolchain -match '^rustc\s+') 'Manifest did not record the active Rust toolchain.'
    Assert-True ($manifest.target -match '^[A-Za-z0-9_][A-Za-z0-9_-]*$') 'Manifest did not record a valid Rust target triple.'
    Assert-True ($manifest.entries.Count -eq 11) 'Manifest did not record every staged input.'
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
    Assert-True ($runtimeLibrary.sha256 -ne $editorLibrary.sha256) 'Manifest did not preserve distinct runtime and editor library inputs.'
    Assert-True (@($manifest.entries | Where-Object { $_.sha256 -notmatch '^[0-9A-F]{64}$' }).Count -eq 0) 'Manifest entries must use SHA-256 hashes.'
    Assert-True ($result.output_hash -match '^[0-9A-F]{64}$') 'Stage output hash is not a SHA-256 value.'

    $json = (& $stager `
        -RuntimeExecutable $fixture.RuntimeExecutable `
        -EditorExecutable $fixture.EditorExecutable `
        -RuntimeLibrary $fixture.RuntimeLibrary `
        -EditorRuntimeLibrary $fixture.EditorRuntimeLibrary `
        -TemplateRoot $fixture.TemplateRoot `
        -EngineAssetRoot $fixture.EngineAssetRoot `
        -ProjectRoot $fixture.ProjectRoot `
        -StagingRoot $fixture.StagingRoot `
        -SourceFingerprint 'fixture-source-fingerprint' `
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
  "bindings": [
    { "path": { "view_id": "Hierarchy", "control_id": "SelectCube", "event_kind": "Click" }, "payload": { "SelectionCommand": { "SelectSceneNode": { "node_id": 3 } } } },
    { "path": { "view_id": "Inspector", "control_id": "TransformPositionXCommit", "event_kind": "Submit" }, "payload": { "InspectorFieldBatch": { "subject_path": "entity://selected", "changes": [{ "field_id": "transform.translation.x", "value": { "Float": 42.0 } }] } } },
    { "path": { "view_id": "WorkbenchMenuBar", "control_id": "SaveProject", "event_kind": "Click" }, "payload": { "MenuAction": { "action_id": "workbench.project.save" } } }
  ]
}
'@, [Text.UTF8Encoding]::new($false))
    $authoringLaunched = (& $stager `
        -RuntimeExecutable $fixture.RuntimeExecutable `
        -EditorExecutable $fixture.EditorExecutable `
        -RuntimeLibrary $fixture.RuntimeLibrary `
        -EditorRuntimeLibrary $fixture.EditorRuntimeLibrary `
        -TemplateRoot $fixture.TemplateRoot `
        -EngineAssetRoot $fixture.EngineAssetRoot `
        -ProjectRoot $fixture.ProjectRoot `
        -AuthoringAutomationRequest $authoringAutomationRequest `
        -ReopenAutomationRequest $defaultReopenAutomationPath `
        -StagingRoot $fixture.StagingRoot `
        -SourceFingerprint 'fixture-source-fingerprint' `
        -RunId 'fixture-authoring-automation' `
        -RepeatCount 2 `
        -TimeoutSeconds 10 `
        -AllowUnsafeStagingRoot)
    Assert-True ($null -ne $authoringLaunched.authoring_automation) 'MVP staging launch fixture did not return the structured authoring automation report.'
    Assert-True ($authoringLaunched.baseline_automation.snapshot.inspector_translation[0] -eq '0') 'MVP staging did not capture the canonical Cube state before authoring.'
    Assert-True ($authoringLaunched.authoring_automation.records.Count -eq 3) 'MVP staging launch fixture lost the normal authoring binding sequence.'
    Assert-True ($authoringLaunched.authoring_automation.records[1].transaction_id -eq 1) 'MVP staging launch fixture did not preserve the inspector transaction.'
    Assert-True ($authoringLaunched.authoring_automation.records[2].save_generation -eq 2) 'MVP staging launch fixture did not preserve the project save generation.'
    Assert-True ($authoringLaunched.authoring_automation.snapshot.selected_node_name -eq 'Cube') 'MVP staging launch fixture did not preserve the retained-host authoring snapshot.'
    Assert-True ($authoringLaunched.authoring_automation.project_identity -eq 'fixture-project') 'MVP staging launch fixture lost the authoring project identity.'
    Assert-True ($authoringLaunched.authoring_automation.scene_uri -eq 'res://scenes/main.scene.toml') 'MVP staging launch fixture lost the authoring scene URI.'
    Assert-True ($authoringLaunched.authoring_automation.selected_model_resource_id -eq 'fixture-cube-model-resource') 'MVP staging launch fixture lost the selected Cube model reference.'
    Assert-True (@($authoringLaunched.reopen_automation).Count -eq 2) 'MVP staging launch fixture did not run independent persisted-state reopen reports twice.'
    Assert-True (@($authoringLaunched.product_runs).Count -eq 5) 'MVP staging launch fixture did not preserve two pre-edit products, two editor reopens, and one after-edit runtime.'
    Assert-True (@($authoringLaunched.product_runs | Where-Object { $_.product -eq 'runtime' -and $_.attempt -eq 3 }).Count -eq 1) 'MVP staging launch fixture did not assign a new runtime attempt after authoring.'
    $reopenedEditorRun = @($authoringLaunched.product_runs | Where-Object { $_.product -eq 'editor' -and $_.attempt -eq 1 })[0]
    Assert-True ($reopenedEditorRun.editor_window_capture.path -eq 'captures/editor-after-reopen.png') 'MVP staging launch fixture did not archive the reopened editor window PNG.'
    Assert-True ($reopenedEditorRun.editor_window_capture.width -eq 16 -and $reopenedEditorRun.editor_window_capture.height -eq 16) 'MVP staging launch fixture did not inspect the reopened editor window PNG dimensions.'
    Assert-True ($reopenedEditorRun.editor_window_capture.non_background_pixels -ge 100) 'MVP staging launch fixture accepted an insufficiently visible reopened editor window PNG.'
    Assert-True ($reopenedEditorRun.editor_product_diagnostics.selected_node_name -eq 'Cube') 'MVP staging launch fixture did not tie the reopened editor capture to Cube.'
    Assert-True ($reopenedEditorRun.editor_product_diagnostics.inspector_translation_x -eq '42') 'MVP staging launch fixture did not tie the reopened editor capture to persisted Inspector X.'
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

    $previousWrongAutomationProject = $env:ZIRCON_MVP_FIXTURE_WRONG_AUTOMATION_PROJECT
    $env:ZIRCON_MVP_FIXTURE_WRONG_AUTOMATION_PROJECT = '1'
    try {
        $wrongAutomationProjectRejected = $false
        try {
            $null = & $stager `
                -RuntimeExecutable $fixture.RuntimeExecutable `
                -EditorExecutable $fixture.EditorExecutable `
                -RuntimeLibrary $fixture.RuntimeLibrary `
                -EditorRuntimeLibrary $fixture.EditorRuntimeLibrary `
                -TemplateRoot $fixture.TemplateRoot `
                -EngineAssetRoot $fixture.EngineAssetRoot `
                -ProjectRoot $fixture.ProjectRoot `
                -AuthoringAutomationRequest $authoringAutomationRequest `
                -ReopenAutomationRequest $defaultReopenAutomationPath `
                -StagingRoot $fixture.StagingRoot `
                -SourceFingerprint 'fixture-source-fingerprint' `
                -RunId 'fixture-wrong-authoring-project' `
                -RepeatCount 2 `
                -TimeoutSeconds 10 `
                -AllowUnsafeStagingRoot
        }
        catch {
            $wrongAutomationProjectRejected = $_.Exception.Message -match 'authoring automation report project_path.*differs from staged project'
        }
        Assert-True $wrongAutomationProjectRejected 'MVP staging accepted authoring automation evidence from a different project root.'
    }
    finally {
        if ($null -eq $previousWrongAutomationProject) {
            Remove-Item Env:\ZIRCON_MVP_FIXTURE_WRONG_AUTOMATION_PROJECT -ErrorAction SilentlyContinue
        }
        else {
            $env:ZIRCON_MVP_FIXTURE_WRONG_AUTOMATION_PROJECT = $previousWrongAutomationProject
        }
    }

    $authoringFailureRunId = 'fixture-authoring-nonzero-child'
    $authoringFailureStage = Join-Path $fixture.StagingRoot $authoringFailureRunId
    $previousAuthoringFailureWithChild = $env:ZIRCON_MVP_FIXTURE_FAIL_AUTOMATION_WITH_CHILD
    $env:ZIRCON_MVP_FIXTURE_FAIL_AUTOMATION_WITH_CHILD = '1'
    try {
        $authoringFailureCleaned = $false
        try {
            $null = & $stager `
                -RuntimeExecutable $fixture.RuntimeExecutable `
                -EditorExecutable $fixture.EditorExecutable `
                -RuntimeLibrary $fixture.RuntimeLibrary `
                -EditorRuntimeLibrary $fixture.EditorRuntimeLibrary `
                -TemplateRoot $fixture.TemplateRoot `
                -EngineAssetRoot $fixture.EngineAssetRoot `
                -ProjectRoot $fixture.ProjectRoot `
                -AuthoringAutomationRequest $authoringAutomationRequest `
                -ReopenAutomationRequest $defaultReopenAutomationPath `
                -StagingRoot $fixture.StagingRoot `
                -SourceFingerprint 'fixture-source-fingerprint' `
                -RunId $authoringFailureRunId `
                -RepeatCount 2 `
                -TimeoutSeconds 10 `
                -AllowUnsafeStagingRoot
        }
        catch {
            $authoringFailureCleaned = $_.Exception.Message -match 'exited with code 32|remain after product exit and were terminated'
        }
        Assert-True $authoringFailureCleaned 'A nonzero authoring automation exit with a staged child was not rejected after cleanup.'
        $authoringFailureStderr = Join-Path $authoringFailureStage 'logs/editor-authoring.stderr.log'
        Assert-True (Test-Path -LiteralPath $authoringFailureStderr) 'A nonzero authoring automation exit did not preserve its stderr log.'
        Assert-True ((Get-Content -Raw -LiteralPath $authoringFailureStderr) -match 'fixture automation failed after spawning child') 'A nonzero authoring automation stderr log was not drained before failure.'
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
        if ($null -eq $previousAuthoringFailureWithChild) {
            Remove-Item Env:\ZIRCON_MVP_FIXTURE_FAIL_AUTOMATION_WITH_CHILD -ErrorAction SilentlyContinue
        }
        else {
            $env:ZIRCON_MVP_FIXTURE_FAIL_AUTOMATION_WITH_CHILD = $previousAuthoringFailureWithChild
        }
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

    $previousEditorCaptureSkip = $env:ZIRCON_MVP_FIXTURE_SKIP_EDITOR_CAPTURE
    $env:ZIRCON_MVP_FIXTURE_SKIP_EDITOR_CAPTURE = '1'
    try {
        $missingReopenedEditorCaptureRejected = $false
        try {
            $null = & $stager `
                -RuntimeExecutable $fixture.RuntimeExecutable `
                -EditorExecutable $fixture.EditorExecutable `
                -RuntimeLibrary $fixture.RuntimeLibrary `
                -EditorRuntimeLibrary $fixture.EditorRuntimeLibrary `
                -TemplateRoot $fixture.TemplateRoot `
                -EngineAssetRoot $fixture.EngineAssetRoot `
                -ProjectRoot $fixture.ProjectRoot `
                -AuthoringAutomationRequest $authoringAutomationRequest `
                -ReopenAutomationRequest $defaultReopenAutomationPath `
                -StagingRoot $fixture.StagingRoot `
                -SourceFingerprint 'fixture-source-fingerprint' `
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
        if ($null -eq $previousEditorCaptureSkip) {
            Remove-Item Env:\ZIRCON_MVP_FIXTURE_SKIP_EDITOR_CAPTURE -ErrorAction SilentlyContinue
        }
        else {
            $env:ZIRCON_MVP_FIXTURE_SKIP_EDITOR_CAPTURE = $previousEditorCaptureSkip
        }
    }

    $previousEditorCaptureFileSkip = $env:ZIRCON_MVP_FIXTURE_SKIP_EDITOR_CAPTURE_FILE
    $env:ZIRCON_MVP_FIXTURE_SKIP_EDITOR_CAPTURE_FILE = '1'
    try {
        $missingReopenedEditorCaptureFileRejected = $false
        try {
            $null = & $stager `
                -RuntimeExecutable $fixture.RuntimeExecutable `
                -EditorExecutable $fixture.EditorExecutable `
                -RuntimeLibrary $fixture.RuntimeLibrary `
                -EditorRuntimeLibrary $fixture.EditorRuntimeLibrary `
                -TemplateRoot $fixture.TemplateRoot `
                -EngineAssetRoot $fixture.EngineAssetRoot `
                -ProjectRoot $fixture.ProjectRoot `
                -AuthoringAutomationRequest $authoringAutomationRequest `
                -ReopenAutomationRequest $defaultReopenAutomationPath `
                -StagingRoot $fixture.StagingRoot `
                -SourceFingerprint 'fixture-source-fingerprint' `
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
        if ($null -eq $previousEditorCaptureFileSkip) {
            Remove-Item Env:\ZIRCON_MVP_FIXTURE_SKIP_EDITOR_CAPTURE_FILE -ErrorAction SilentlyContinue
        }
        else {
            $env:ZIRCON_MVP_FIXTURE_SKIP_EDITOR_CAPTURE_FILE = $previousEditorCaptureFileSkip
        }
    }

    $previousRuntimeInputProbe = $env:ZIRCON_RUNTIME_MVP_INPUT_PROBE
    $env:ZIRCON_RUNTIME_MVP_INPUT_PROBE = '1'
    try {
        $launched = (& $stager `
            -RuntimeExecutable $fixture.RuntimeExecutable `
            -EditorExecutable $fixture.EditorExecutable `
            -RuntimeLibrary $fixture.RuntimeLibrary `
            -EditorRuntimeLibrary $fixture.EditorRuntimeLibrary `
            -TemplateRoot $fixture.TemplateRoot `
            -EngineAssetRoot $fixture.EngineAssetRoot `
            -ProjectRoot $fixture.ProjectRoot `
            -StagingRoot $fixture.StagingRoot `
            -SourceFingerprint 'fixture-source-fingerprint' `
            -RunId 'fixture-launch' `
            -RepeatCount 1 `
            -TimeoutSeconds 10 `
            -AllowUnsafeStagingRoot)
        Assert-True $launched.launched 'MVP staging launch fixture did not run the staged products.'
        Assert-True (@($launched.product_runs).Count -eq 2) 'MVP staging launch fixture did not record both products.'
        Assert-True (@($launched.product_runs | Where-Object { $_.first_frame_presented -and $_.teardown_complete }).Count -eq 2) 'MVP staging launch fixture did not verify first-frame teardown for both products.'
        $runtimeRun = @($launched.product_runs | Where-Object { $_.product -eq 'runtime' })[0]
        $editorRun = @($launched.product_runs | Where-Object { $_.product -eq 'editor' })[0]
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
        Assert-True ($runtimeRun.runtime_product_diagnostics.input_pointer_move_count -eq '1') 'MVP staging launch fixture did not preserve pointer-input evidence.'
        Assert-True ($runtimeRun.runtime_product_diagnostics.input_mouse_button_press_count -eq '1') 'MVP staging launch fixture did not preserve mouse-button press evidence.'
        Assert-True ($runtimeRun.runtime_product_diagnostics.input_mouse_button_release_count -eq '1') 'MVP staging launch fixture did not preserve mouse-button release evidence.'
        Assert-True ($runtimeRun.runtime_product_diagnostics.input_keyboard_press_count -eq '1') 'MVP staging launch fixture did not preserve keyboard press evidence.'
        Assert-True ($runtimeRun.runtime_product_diagnostics.input_keyboard_release_count -eq '1') 'MVP staging launch fixture did not preserve keyboard release evidence.'
        Assert-True ($null -eq $editorRun.frame_capture) 'MVP staging must not request a runtime PNG from the editor product.'
        Assert-True ($null -eq $editorRun.editor_window_capture) 'MVP staging must not request editor window PNG evidence outside the F5 reopen workflow.'
        Assert-True ($null -eq $editorRun.runtime_product_diagnostics) 'MVP staging must not require runtime diagnostics from the editor product.'
    }
    finally {
        if ($null -eq $previousRuntimeInputProbe) {
            Remove-Item Env:\ZIRCON_RUNTIME_MVP_INPUT_PROBE -ErrorAction SilentlyContinue
        }
        else {
            $env:ZIRCON_RUNTIME_MVP_INPUT_PROBE = $previousRuntimeInputProbe
        }
    }

    $createWithoutLaunchRejected = $false
    try {
        $null = & $stager `
            -RuntimeExecutable $fixture.RuntimeExecutable `
            -EditorExecutable $fixture.EditorExecutable `
            -RuntimeLibrary $fixture.RuntimeLibrary `
            -EditorRuntimeLibrary $fixture.EditorRuntimeLibrary `
            -TemplateRoot $fixture.TemplateRoot `
            -EngineAssetRoot $fixture.EngineAssetRoot `
            -StagingRoot $fixture.StagingRoot `
            -SourceFingerprint 'fixture-source-fingerprint' `
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
        -RuntimeExecutable $fixture.RuntimeExecutable `
        -EditorExecutable $fixture.EditorExecutable `
        -RuntimeLibrary $fixture.RuntimeLibrary `
        -EditorRuntimeLibrary $fixture.EditorRuntimeLibrary `
        -TemplateRoot $fixture.TemplateRoot `
        -EngineAssetRoot $fixture.EngineAssetRoot `
        -StagingRoot $fixture.StagingRoot `
        -SourceFingerprint 'fixture-source-fingerprint' `
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
    Assert-True ($createdStartupSummary.project_creation.editor_window_capture.path -eq 'captures/editor-before-edit.png') 'Created project startup evidence did not archive the editor window PNG before authoring.'
    Assert-True ($createdStartupSummary.project_creation.editor_window_capture.non_background_pixels -ge 100) 'Created project startup evidence accepted an insufficiently visible editor window PNG before authoring.'
    Assert-True ($createdStartupSummary.project_creation.editor_product_diagnostics.selected_node_name -eq 'Cube') 'Created project startup evidence did not tie the editor window PNG to Cube.'
    Assert-True ($createdStartupSummary.project_creation.editor_product_diagnostics.inspector_translation_x -eq '0') 'Created project startup evidence did not tie the editor window PNG to the initial Inspector X.'
    Assert-True ($createdStartupSummary.project_creation.stdout.sha256 -match '^[0-9A-F]{64}$') 'Created project startup evidence did not hash the staged editor creation stdout.'
    Assert-True ($createdStartupSummary.project_creation.diagnostic_logs.Count -gt 0) 'Created project startup evidence did not retain the staged editor creation diagnostics.'
    Assert-True ($createdStartupSummary.project_creation.project_open.project_root -eq 'project/ZirconMvpFixture') 'Created project startup evidence did not preserve the canonical project-open root.'
    Assert-True ($createdStartupSummary.project_creation.project_open.manifest_identity -eq 'fixture-created-project@v1') 'Created project startup evidence did not preserve the manifest identity from the normal editor project-open diagnostic.'
    Assert-True ($createdStartupSummary.project_creation.project_open.scene_uri -eq 'res://scenes/main.scene.toml') 'Created project startup evidence did not preserve the default scene URI from the normal editor project-open diagnostic.'
    Assert-True ($createdStartupSummary.project_creation.project_open.registry_ready_asset_count -eq 4) 'Created project startup evidence did not preserve ready starter-asset count.'
    Assert-True ($createdStartupSummary.project_creation.project_open.settings_source -eq 'persisted-v1') 'Created project startup evidence did not preserve persisted project-settings provenance.'
    Assert-True (Test-Path -LiteralPath (Join-Path $created.staging_root 'logs/editor-create.stdout.log')) 'Created project staging did not retain the editor creation stdout log.'
    Assert-True (Test-Path -LiteralPath (Join-Path $created.staging_root 'logs/editor-create.stderr.log')) 'Created project staging did not retain the editor creation stderr log.'

    $unicodeProjectName = (-join ([char[]]@(0x9879, 0x76EE))) + ' ' + (-join ([char[]]@(0x8DEF, 0x5F84)))
    $unicodeProjectRunId = 'fixture-created-project-unicode'
    $unicodeCreated = (& $stager `
        -RuntimeExecutable $fixture.RuntimeExecutable `
        -EditorExecutable $fixture.EditorExecutable `
        -RuntimeLibrary $fixture.RuntimeLibrary `
        -EditorRuntimeLibrary $fixture.EditorRuntimeLibrary `
        -TemplateRoot $fixture.TemplateRoot `
        -EngineAssetRoot $fixture.EngineAssetRoot `
        -StagingRoot $fixture.StagingRoot `
        -SourceFingerprint 'fixture-source-fingerprint' `
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
    $previousSkipProjectOpenDiagnostic = $env:ZIRCON_MVP_FIXTURE_SKIP_PROJECT_OPEN_DIAGNOSTIC
    $env:ZIRCON_MVP_FIXTURE_SKIP_PROJECT_OPEN_DIAGNOSTIC = '1'
    try {
        $missingProjectOpenDiagnosticRejected = $false
        try {
            & $stager `
                -RuntimeExecutable $fixture.RuntimeExecutable `
                -EditorExecutable $fixture.EditorExecutable `
                -RuntimeLibrary $fixture.RuntimeLibrary `
                -EditorRuntimeLibrary $fixture.EditorRuntimeLibrary `
                -TemplateRoot $fixture.TemplateRoot `
                -EngineAssetRoot $fixture.EngineAssetRoot `
                -StagingRoot $fixture.StagingRoot `
                -SourceFingerprint 'fixture-source-fingerprint' `
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
        if ($null -eq $previousSkipProjectOpenDiagnostic) {
            Remove-Item Env:\ZIRCON_MVP_FIXTURE_SKIP_PROJECT_OPEN_DIAGNOSTIC -ErrorAction SilentlyContinue
        }
        else {
            $env:ZIRCON_MVP_FIXTURE_SKIP_PROJECT_OPEN_DIAGNOSTIC = $previousSkipProjectOpenDiagnostic
        }
    }

    $createFailureRunId = 'fixture-created-project-failure'
    $createFailureStage = Join-Path $fixture.StagingRoot $createFailureRunId
    $previousCreateFailureWithChild = $env:ZIRCON_MVP_FIXTURE_FAIL_CREATE_WITH_CHILD
    $env:ZIRCON_MVP_FIXTURE_FAIL_CREATE_WITH_CHILD = '1'
    try {
        $createFailureCleaned = $false
        try {
            & $stager `
                -RuntimeExecutable $fixture.RuntimeExecutable `
                -EditorExecutable $fixture.EditorExecutable `
                -RuntimeLibrary $fixture.RuntimeLibrary `
                -EditorRuntimeLibrary $fixture.EditorRuntimeLibrary `
                -TemplateRoot $fixture.TemplateRoot `
                -EngineAssetRoot $fixture.EngineAssetRoot `
                -StagingRoot $fixture.StagingRoot `
                -SourceFingerprint 'fixture-source-fingerprint' `
                -RunId $createFailureRunId `
                -CreateProject `
                -ProjectName 'ZirconMvpFixtureFailure' `
                -RepeatCount 1 `
                -TimeoutSeconds 10 `
                -AllowUnsafeStagingRoot | Out-Null
        }
        catch {
            $createFailureCleaned = $_.Exception.Message -match 'remain after product exit and were terminated'
        }
        Assert-True $createFailureCleaned 'A nonzero staged project-creation exit with a child process must be rejected only after the staged child is cleaned up.'
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
    }
    finally {
        if ($null -eq $previousCreateFailureWithChild) {
            Remove-Item Env:\ZIRCON_MVP_FIXTURE_FAIL_CREATE_WITH_CHILD -ErrorAction SilentlyContinue
        }
        else {
            $env:ZIRCON_MVP_FIXTURE_FAIL_CREATE_WITH_CHILD = $previousCreateFailureWithChild
        }
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
    $previousSkipCapture = $env:ZIRCON_MVP_FIXTURE_SKIP_RUNTIME_CAPTURE
    $env:ZIRCON_MVP_FIXTURE_SKIP_RUNTIME_CAPTURE = '1'
    try {
        $missingCaptureDetected = $false
        try {
            & $stager `
                -RuntimeExecutable $fixture.RuntimeExecutable `
                -EditorExecutable $fixture.EditorExecutable `
                -RuntimeLibrary $fixture.RuntimeLibrary `
                -EditorRuntimeLibrary $fixture.EditorRuntimeLibrary `
                -TemplateRoot $fixture.TemplateRoot `
                -EngineAssetRoot $fixture.EngineAssetRoot `
                -ProjectRoot $fixture.ProjectRoot `
                -StagingRoot $fixture.StagingRoot `
                -SourceFingerprint 'fixture-source-fingerprint' `
                -RunId $missingCaptureRunId `
                -RepeatCount 1 `
                -TimeoutSeconds 10 `
                -AllowUnsafeStagingRoot | Out-Null
        }
        catch {
            $missingCaptureDetected = $_.Exception.Message -match 'runtime_product_frame_capture_written|Runtime frame capture'
        }
        Assert-True $missingCaptureDetected 'A runtime product that omits requested PNG evidence was not rejected.'
    }
    finally {
        if ($null -eq $previousSkipCapture) {
            Remove-Item Env:\ZIRCON_MVP_FIXTURE_SKIP_RUNTIME_CAPTURE -ErrorAction SilentlyContinue
        }
        else {
            $env:ZIRCON_MVP_FIXTURE_SKIP_RUNTIME_CAPTURE = $previousSkipCapture
        }
    }

    $missingDiagnosticsRunId = 'fixture-missing-runtime-diagnostics'
    $previousSkipDiagnostics = $env:ZIRCON_MVP_FIXTURE_SKIP_RUNTIME_DIAGNOSTICS
    $env:ZIRCON_MVP_FIXTURE_SKIP_RUNTIME_DIAGNOSTICS = '1'
    try {
        $missingDiagnosticsDetected = $false
        try {
            & $stager `
                -RuntimeExecutable $fixture.RuntimeExecutable `
                -EditorExecutable $fixture.EditorExecutable `
                -RuntimeLibrary $fixture.RuntimeLibrary `
                -EditorRuntimeLibrary $fixture.EditorRuntimeLibrary `
                -TemplateRoot $fixture.TemplateRoot `
                -EngineAssetRoot $fixture.EngineAssetRoot `
                -ProjectRoot $fixture.ProjectRoot `
                -StagingRoot $fixture.StagingRoot `
                -SourceFingerprint 'fixture-source-fingerprint' `
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
        if ($null -eq $previousSkipDiagnostics) {
            Remove-Item Env:\ZIRCON_MVP_FIXTURE_SKIP_RUNTIME_DIAGNOSTICS -ErrorAction SilentlyContinue
        }
        else {
            $env:ZIRCON_MVP_FIXTURE_SKIP_RUNTIME_DIAGNOSTICS = $previousSkipDiagnostics
        }
    }

    $materialFallbackRunId = 'fixture-runtime-material-fallback'
    $previousMaterialFallback = $env:ZIRCON_MVP_FIXTURE_MATERIAL_FALLBACK
    $env:ZIRCON_MVP_FIXTURE_MATERIAL_FALLBACK = '1'
    try {
        $materialFallbackDetected = $false
        try {
            & $stager `
                -RuntimeExecutable $fixture.RuntimeExecutable `
                -EditorExecutable $fixture.EditorExecutable `
                -RuntimeLibrary $fixture.RuntimeLibrary `
                -EditorRuntimeLibrary $fixture.EditorRuntimeLibrary `
                -TemplateRoot $fixture.TemplateRoot `
                -EngineAssetRoot $fixture.EngineAssetRoot `
                -ProjectRoot $fixture.ProjectRoot `
                -StagingRoot $fixture.StagingRoot `
                -SourceFingerprint 'fixture-source-fingerprint' `
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
        if ($null -eq $previousMaterialFallback) {
            Remove-Item Env:\ZIRCON_MVP_FIXTURE_MATERIAL_FALLBACK -ErrorAction SilentlyContinue
        }
        else {
            $env:ZIRCON_MVP_FIXTURE_MATERIAL_FALLBACK = $previousMaterialFallback
        }
    }

    $timeoutRunId = 'fixture-timeout-process-tree'
    $timeoutStage = Join-Path $fixture.StagingRoot $timeoutRunId
    $previousTimeoutChild = $env:ZIRCON_MVP_FIXTURE_TIMEOUT_WITH_CHILD
    $env:ZIRCON_MVP_FIXTURE_TIMEOUT_WITH_CHILD = '1'
    try {
        $timedOut = $false
        try {
            & $stager `
                -RuntimeExecutable $fixture.RuntimeExecutable `
                -EditorExecutable $fixture.EditorExecutable `
                -RuntimeLibrary $fixture.RuntimeLibrary `
                -EditorRuntimeLibrary $fixture.EditorRuntimeLibrary `
                -TemplateRoot $fixture.TemplateRoot `
                -EngineAssetRoot $fixture.EngineAssetRoot `
                -ProjectRoot $fixture.ProjectRoot `
                -StagingRoot $fixture.StagingRoot `
                -SourceFingerprint 'fixture-source-fingerprint' `
                -RunId $timeoutRunId `
                -RepeatCount 1 `
                -TimeoutSeconds 1 `
                -AllowUnsafeStagingRoot | Out-Null
        }
        catch {
            $timedOut = $_.Exception.Message -match 'did not exit within'
        }
        Assert-True $timedOut 'A timed-out staged product was not reported as a failure.'
        $timeoutStderr = Join-Path $timeoutStage 'logs/runtime-1.stderr.log'
        Assert-True (Test-Path -LiteralPath $timeoutStderr) 'A timed-out staged product did not preserve its stderr log.'
        Assert-True ((Get-Content -Raw -LiteralPath $timeoutStderr) -match 'fixture timeout emitted before termination') 'A timed-out staged product stderr stream was not drained before failure.'
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
        if ($null -eq $previousTimeoutChild) {
            Remove-Item Env:\ZIRCON_MVP_FIXTURE_TIMEOUT_WITH_CHILD -ErrorAction SilentlyContinue
        }
        else {
            $env:ZIRCON_MVP_FIXTURE_TIMEOUT_WITH_CHILD = $previousTimeoutChild
        }
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
    $previousNonzeroChild = $env:ZIRCON_MVP_FIXTURE_FAIL_WITH_CHILD
    $env:ZIRCON_MVP_FIXTURE_FAIL_WITH_CHILD = '1'
    try {
        $nonzeroExitDetected = $false
        try {
            & $stager `
                -RuntimeExecutable $fixture.RuntimeExecutable `
                -EditorExecutable $fixture.EditorExecutable `
                -RuntimeLibrary $fixture.RuntimeLibrary `
                -EditorRuntimeLibrary $fixture.EditorRuntimeLibrary `
                -TemplateRoot $fixture.TemplateRoot `
                -EngineAssetRoot $fixture.EngineAssetRoot `
                -ProjectRoot $fixture.ProjectRoot `
                -StagingRoot $fixture.StagingRoot `
                -SourceFingerprint 'fixture-source-fingerprint' `
                -RunId $nonzeroRunId `
                -RepeatCount 1 `
                -TimeoutSeconds 10 `
                -AllowUnsafeStagingRoot | Out-Null
        }
        catch {
            $nonzeroExitDetected = $_.Exception.Message -match 'exited with code 23'
        }
        Assert-True $nonzeroExitDetected 'A nonzero staged product exit was not reported as a failure.'
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
        if ($null -eq $previousNonzeroChild) {
            Remove-Item Env:\ZIRCON_MVP_FIXTURE_FAIL_WITH_CHILD -ErrorAction SilentlyContinue
        }
        else {
            $env:ZIRCON_MVP_FIXTURE_FAIL_WITH_CHILD = $previousNonzeroChild
        }
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
    $previousLeakChild = $env:ZIRCON_MVP_FIXTURE_LEAK_STAGED_CHILD
    $env:ZIRCON_MVP_FIXTURE_LEAK_STAGED_CHILD = '1'
    try {
        $leakedProcessDetected = $false
        try {
            & $stager `
                -RuntimeExecutable $fixture.RuntimeExecutable `
                -EditorExecutable $fixture.EditorExecutable `
                -RuntimeLibrary $fixture.RuntimeLibrary `
                -EditorRuntimeLibrary $fixture.EditorRuntimeLibrary `
                -TemplateRoot $fixture.TemplateRoot `
                -EngineAssetRoot $fixture.EngineAssetRoot `
                -ProjectRoot $fixture.ProjectRoot `
                -StagingRoot $fixture.StagingRoot `
                -SourceFingerprint 'fixture-source-fingerprint' `
                -RunId $leakedRunId `
                -RepeatCount 1 `
                -TimeoutSeconds 10 `
                -AllowUnsafeStagingRoot | Out-Null
        }
        catch {
            $leakedProcessDetected = $_.Exception.Message -match 'Staged executable process\(es\) remain after product exit'
        }
        Assert-True $leakedProcessDetected 'A staged child process that outlives product exit was not reported.'
    }
    finally {
        if ($null -eq $previousLeakChild) {
            Remove-Item Env:\ZIRCON_MVP_FIXTURE_LEAK_STAGED_CHILD -ErrorAction SilentlyContinue
        }
        else {
            $env:ZIRCON_MVP_FIXTURE_LEAK_STAGED_CHILD = $previousLeakChild
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

    Remove-Item -LiteralPath $fixture.RuntimeLibrary -Force
    $failed = $false
    try {
        Invoke-MvpStager -Fixture $fixture -RunId 'fixture-failure' | Out-Null
    }
    catch {
        $failed = $_.Exception.Message -match 'RuntimeLibrary'
    }
    Assert-True $failed 'A missing runtime library did not produce an actionable failure.'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $fixture.StagingRoot 'fixture-failure'))) 'Failed staging left a partial product directory.'

    Write-Host 'MVP staging contract passed'
}
finally {
    if (Test-Path -LiteralPath $fixture.Root) {
        Remove-Item -LiteralPath $fixture.Root -Recurse -Force
    }
}
