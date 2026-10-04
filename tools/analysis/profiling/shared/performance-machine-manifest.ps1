Set-StrictMode -Version Latest

$script:ZirconPerformanceMachineManifestCategories = @(
    "cpu",
    "gpu",
    "memory",
    "bios",
    "os",
    "display_modes",
    "power_policy",
    "thermal_frequency",
    "background_load",
    "virtualization"
)

function Get-ZirconPerformanceMachineManifestProperty {
    param(
        [Parameter(Mandatory = $true)]$Value,
        [Parameter(Mandatory = $true)][string]$Name
    )

    if ($Value -is [System.Collections.IDictionary]) {
        if ($Value.Contains($Name)) {
            return $Value[$Name]
        }
        return $null
    }
    $property = $Value.PSObject.Properties[$Name]
    if ($null -eq $property) {
        return $null
    }
    return $property.Value
}

function New-ZirconPerformanceMachineUnavailableObservation {
    param([Parameter(Mandatory = $true)][string]$Reason)

    return [ordered]@{
        status = "unavailable"
        reason = $Reason.Substring(0, [Math]::Min($Reason.Length, 512))
    }
}

function Invoke-ZirconPerformanceMachineProbe {
    param([Parameter(Mandatory = $true)][scriptblock]$Probe)

    try {
        $data = & $Probe
        if ($null -eq $data -or @($data).Count -eq 0) {
            return New-ZirconPerformanceMachineUnavailableObservation -Reason "probe returned no data"
        }
        return [ordered]@{
            status = "captured"
            data = @($data)
        }
    }
    catch {
        return New-ZirconPerformanceMachineUnavailableObservation -Reason $_.Exception.Message
    }
}

function Get-ZirconPerformanceMachineObservationSet {
    $cpu = Invoke-ZirconPerformanceMachineProbe -Probe {
        Get-CimInstance -ClassName Win32_Processor -ErrorAction Stop | ForEach-Object {
            [ordered]@{
                name = [string]$_.Name
                manufacturer = [string]$_.Manufacturer
                core_count = [int]$_.NumberOfCores
                logical_processor_count = [int]$_.NumberOfLogicalProcessors
                current_clock_mhz = [int]$_.CurrentClockSpeed
                max_clock_mhz = [int]$_.MaxClockSpeed
            }
        }
    }
    $gpu = Invoke-ZirconPerformanceMachineProbe -Probe {
        Get-CimInstance -ClassName Win32_VideoController -ErrorAction Stop | ForEach-Object {
            [ordered]@{
                name = [string]$_.Name
                pnp_device_id = [string]$_.PNPDeviceID
                driver_version = [string]$_.DriverVersion
                adapter_ram_bytes = [int64]$_.AdapterRAM
            }
        }
    }
    $memory = Invoke-ZirconPerformanceMachineProbe -Probe {
        Get-CimInstance -ClassName Win32_ComputerSystem -ErrorAction Stop | ForEach-Object {
            [ordered]@{
                total_physical_memory_bytes = [int64]$_.TotalPhysicalMemory
                installed_memory_modules = @(
                    Get-CimInstance -ClassName Win32_PhysicalMemory -ErrorAction Stop |
                        ForEach-Object { [int64]$_.Capacity }
                )
            }
        }
    }
    $bios = Invoke-ZirconPerformanceMachineProbe -Probe {
        Get-CimInstance -ClassName Win32_BIOS -ErrorAction Stop | ForEach-Object {
            [ordered]@{
                manufacturer = [string]$_.Manufacturer
                smbios_bios_version = [string]$_.SMBIOSBIOSVersion
                release_date = [string]$_.ReleaseDate
            }
        }
    }
    $os = Invoke-ZirconPerformanceMachineProbe -Probe {
        Get-CimInstance -ClassName Win32_OperatingSystem -ErrorAction Stop | ForEach-Object {
            [ordered]@{
                caption = [string]$_.Caption
                version = [string]$_.Version
                build_number = [string]$_.BuildNumber
            }
        }
    }
    $displayModes = Invoke-ZirconPerformanceMachineProbe -Probe {
        Get-CimInstance -ClassName Win32_VideoController -ErrorAction Stop | ForEach-Object {
            [ordered]@{
                name = [string]$_.Name
                width = [int]$_.CurrentHorizontalResolution
                height = [int]$_.CurrentVerticalResolution
                refresh_rate_hz = [int]$_.CurrentRefreshRate
            }
        }
    }
    $powerPolicy = Invoke-ZirconPerformanceMachineProbe -Probe {
        $output = @(& powercfg.exe /getactivescheme 2>&1)
        if ($LASTEXITCODE -ne 0) {
            throw "powercfg.exe /getactivescheme failed: $($output -join ' ')"
        }
        [ordered]@{ active_scheme = ($output -join [Environment]::NewLine).Trim() }
    }
    $thermalFrequency = Invoke-ZirconPerformanceMachineProbe -Probe {
        $temperatures = @(
            Get-CimInstance -Namespace root/wmi -ClassName MSAcpi_ThermalZoneTemperature -ErrorAction Stop |
                ForEach-Object {
                    [ordered]@{
                        celsius = [Math]::Round((([double]$_.CurrentTemperature) / 10.0) - 273.15, 2)
                        instance_name = [string]$_.InstanceName
                    }
                }
        )
        if ($temperatures.Count -eq 0) {
            throw "no readable thermal zones"
        }
        [ordered]@{
            temperatures = $temperatures
            cpu_current_clock_mhz = @(
                Get-CimInstance -ClassName Win32_Processor -ErrorAction Stop |
                    ForEach-Object { [int]$_.CurrentClockSpeed }
            )
        }
    }
    $backgroundLoad = Invoke-ZirconPerformanceMachineProbe -Probe {
        [ordered]@{
            process_count = @((Get-Process -ErrorAction Stop)).Count
            captured_utc = (Get-Date).ToUniversalTime().ToString("o")
        }
    }
    $virtualization = Invoke-ZirconPerformanceMachineProbe -Probe {
        Get-CimInstance -ClassName Win32_ComputerSystem -ErrorAction Stop | ForEach-Object {
            [ordered]@{
                hypervisor_present = [bool]$_.HypervisorPresent
                manufacturer = [string]$_.Manufacturer
                model = [string]$_.Model
            }
        }
    }

    return [ordered]@{
        cpu = $cpu
        gpu = $gpu
        memory = $memory
        bios = $bios
        os = $os
        display_modes = $displayModes
        power_policy = $powerPolicy
        thermal_frequency = $thermalFrequency
        background_load = $backgroundLoad
        virtualization = $virtualization
    }
}

function New-ZirconPerformanceMachineManifest {
    param([System.Collections.IDictionary]$Observations)

    if ($null -eq $Observations) {
        $Observations = Get-ZirconPerformanceMachineObservationSet
    }
    $actualCategories = @($Observations.Keys | Sort-Object)
    $expectedCategories = @($script:ZirconPerformanceMachineManifestCategories | Sort-Object)
    if (($actualCategories -join "|") -ne ($expectedCategories -join "|")) {
        throw "Performance machine manifest categories do not match the required contract."
    }

    $normalized = [ordered]@{}
    $allRequiredObserved = $true
    foreach ($category in $script:ZirconPerformanceMachineManifestCategories) {
        $observation = $Observations[$category]
        if ($null -eq $observation) {
            throw "Performance machine manifest is missing category '$category'."
        }
        $status = [string](Get-ZirconPerformanceMachineManifestProperty -Value $observation -Name "status")
        if ($status -notin @("captured", "unavailable")) {
            throw "Performance machine manifest category '$category' has an invalid status."
        }
        if ($status -eq "unavailable" -and [string]::IsNullOrWhiteSpace([string](Get-ZirconPerformanceMachineManifestProperty -Value $observation -Name "reason"))) {
            throw "Performance machine manifest unavailable category '$category' requires a reason."
        }
        if ($status -eq "captured") {
            $data = Get-ZirconPerformanceMachineManifestProperty -Value $observation -Name "data"
            if ($null -eq $data -or @($data).Count -eq 0) {
                throw "Performance machine manifest captured category '$category' requires data."
            }
            $normalized[$category] = [pscustomobject][ordered]@{
                status = "captured"
                data = @($data)
            }
        }
        else {
            $allRequiredObserved = $false
            $normalized[$category] = [pscustomobject][ordered]@{
                status = "unavailable"
                reason = [string](Get-ZirconPerformanceMachineManifestProperty -Value $observation -Name "reason")
            }
        }
    }

    return [pscustomobject]([ordered]@{
        schema_version = 1
        manifest_kind = "zircon_performance_machine_snapshot"
        captured_utc = (Get-Date).ToUniversalTime().ToString("o")
        required_categories = @($script:ZirconPerformanceMachineManifestCategories)
        all_required_observed = $allRequiredObserved
        cpu = $normalized.cpu
        gpu = $normalized.gpu
        memory = $normalized.memory
        bios = $normalized.bios
        os = $normalized.os
        display_modes = $normalized.display_modes
        power_policy = $normalized.power_policy
        thermal_frequency = $normalized.thermal_frequency
        background_load = $normalized.background_load
        virtualization = $normalized.virtualization
    })
}
