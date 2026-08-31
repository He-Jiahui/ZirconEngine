$script:MachineManifestScript = Join-Path $PSScriptRoot "..\performance-machine-manifest.ps1"

if (Test-Path -LiteralPath $script:MachineManifestScript) {
    . $script:MachineManifestScript
}

Describe "performance machine manifest contract" {
    It "requires every machine and load category with explicit availability" {
        Get-Command New-ZirconPerformanceMachineManifest -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty

        $observations = [ordered]@{
            cpu = @{ status = "captured"; data = @(@{ model = "fixture cpu" }) }
            gpu = @{ status = "captured"; data = @(@{ adapter = "fixture gpu" }) }
            memory = @{ status = "captured"; data = @(@{ bytes = 16GB }) }
            bios = @{ status = "captured"; data = @(@{ version = "fixture bios" }) }
            os = @{ status = "captured"; data = @(@{ build = "fixture os" }) }
            display_modes = @{ status = "captured"; data = @(@{ mode = "1280x720@60" }) }
            power_policy = @{ status = "captured"; data = @(@{ active_scheme = "fixture power" }) }
            thermal_frequency = @{ status = "unavailable"; reason = "fixture lacks sensors" }
            background_load = @{ status = "captured"; data = @(@{ process_count = 42 }) }
            virtualization = @{ status = "captured"; data = @(@{ hypervisor_present = $false }) }
        }

        $manifest = New-ZirconPerformanceMachineManifest -Observations $observations

        $manifest.schema_version | Should Be 1
        $manifest.manifest_kind | Should Be "zircon_performance_machine_snapshot"
        @($manifest.required_categories) | Should Be @(
            "cpu", "gpu", "memory", "bios", "os", "display_modes", "power_policy",
            "thermal_frequency", "background_load", "virtualization"
        )
        $manifest.thermal_frequency.status | Should Be "unavailable"
        $manifest.all_required_observed | Should Be $false
    }

    It "rejects an observation set that omits a required category" {
        Get-Command New-ZirconPerformanceMachineManifest -ErrorAction SilentlyContinue |
            Should Not BeNullOrEmpty

        $observations = [ordered]@{
            cpu = @{ status = "captured" }
        }
        {
            New-ZirconPerformanceMachineManifest -Observations $observations
        } | Should Throw "categories do not match"
    }

    It "rejects a captured observation without data" {
        $observations = [ordered]@{
            cpu = @{ status = "captured" }
            gpu = @{ status = "unavailable"; reason = "fixture" }
            memory = @{ status = "unavailable"; reason = "fixture" }
            bios = @{ status = "unavailable"; reason = "fixture" }
            os = @{ status = "unavailable"; reason = "fixture" }
            display_modes = @{ status = "unavailable"; reason = "fixture" }
            power_policy = @{ status = "unavailable"; reason = "fixture" }
            thermal_frequency = @{ status = "unavailable"; reason = "fixture" }
            background_load = @{ status = "unavailable"; reason = "fixture" }
            virtualization = @{ status = "unavailable"; reason = "fixture" }
        }

        {
            New-ZirconPerformanceMachineManifest -Observations $observations
        } | Should Throw "captured category 'cpu' requires data"
    }

    It "serializes captured data as an array" {
        $observations = [ordered]@{
            cpu = @{ status = "captured"; data = "fixture cpu" }
            gpu = @{ status = "unavailable"; reason = "fixture" }
            memory = @{ status = "unavailable"; reason = "fixture" }
            bios = @{ status = "unavailable"; reason = "fixture" }
            os = @{ status = "unavailable"; reason = "fixture" }
            display_modes = @{ status = "unavailable"; reason = "fixture" }
            power_policy = @{ status = "unavailable"; reason = "fixture" }
            thermal_frequency = @{ status = "unavailable"; reason = "fixture" }
            background_load = @{ status = "unavailable"; reason = "fixture" }
            virtualization = @{ status = "unavailable"; reason = "fixture" }
        }

        $manifest = New-ZirconPerformanceMachineManifest -Observations $observations
        $decoded = ($manifest | ConvertTo-Json -Depth 8) | ConvertFrom-Json

        $decoded.cpu.data -is [array] | Should Be $true
        $decoded.cpu.data | Should Be @("fixture cpu")
    }
}
