# Stateless interpreter discovery for Agent-owned development entry points.
# Loading this helper never launches Python, Cargo, Jenkins, or an installer.
function Resolve-AgentPython {
    [CmdletBinding()]
    param([AllowEmptyString()][string]$PythonInterpreter = '')

    $explicitSelection = $PSBoundParameters.ContainsKey('PythonInterpreter')
    if ($explicitSelection -and [string]::IsNullOrWhiteSpace($PythonInterpreter)) {
        throw '-PythonInterpreter requires a nonempty executable path or command.'
    }
    $requested = if ($explicitSelection) { $PythonInterpreter } elseif ($env:ZIRCON_PYTHON) { $env:ZIRCON_PYTHON } else { $null }
    $candidates = if ($null -ne $requested) { @($requested) } else { @('python.exe', 'python3.exe', 'python', 'python3') }
    foreach ($candidate in $candidates) {
        $path = $null
        if (Test-Path -LiteralPath $candidate -PathType Leaf) {
            $path = (Resolve-Path -LiteralPath $candidate).ProviderPath
        } else {
            $commands = @(Get-Command -Name $candidate -CommandType Application -All -ErrorAction SilentlyContinue)
            foreach ($command in $commands) {
                if ($command.Path -notmatch '(?i)[\\/]WindowsApps[\\/]|[\\/]\.jenkins[\\/]runtime[\\/]') {
                    $path = $command.Path
                    break
                }
            }
        }
        if ($path -and $path -notmatch '(?i)[\\/]WindowsApps[\\/]|[\\/]\.jenkins[\\/]runtime[\\/]') {
            if ([IO.Path]::GetExtension($path) -notin @('.exe', '')) {
                throw 'Select a Python executable, not a script or shell launcher.'
            }
            return [IO.Path]::GetFullPath($path)
        }
        if ($null -ne $requested) {
            throw 'Selected Python is missing or is a WindowsApps/Jenkins runtime alias. Select a general Python 3.11+ executable.'
        }
    }
    throw 'Python 3.11+ was not found. Pass -PythonInterpreter or set ZIRCON_PYTHON; WindowsApps and Jenkins runtime interpreters are not selected.'
}
