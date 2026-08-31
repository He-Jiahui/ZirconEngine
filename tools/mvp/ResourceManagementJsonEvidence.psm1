Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Import-Module (Join-Path $PSScriptRoot '..\WindowsPathResolver.psm1') -ErrorAction Stop

$script:ResourceManagementEvidenceReadBufferBytes = 81920
$script:ResourceManagementEvidenceHexDigits = [char[]]'0123456789ABCDEF'

function ConvertTo-ResourceManagementEvidenceSha256 {
    param([Parameter(Mandatory)][byte[]]$HashBytes)

    [char[]]$characters = [char[]]::new($HashBytes.Length * 2)
    for ($index = 0; $index -lt $HashBytes.Length; $index++) {
        $value = [int]$HashBytes[$index]
        $characters[$index * 2] = $script:ResourceManagementEvidenceHexDigits[$value -shr 4]
        $characters[($index * 2) + 1] = $script:ResourceManagementEvidenceHexDigits[$value -band 0x0F]
    }
    return [string]::new($characters)
}

function Get-ResourceManagementJsonEvidence {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$Label,
        [Parameter(Mandatory)][ValidateRange(1, [Int32]::MaxValue)][int]$MaximumBytes
    )

    $resolution = Resolve-ZirconWindowsPath -Path $Path
    if (-not [IO.File]::Exists($resolution.OperationalPath)) {
        throw "$Label does not exist: $($resolution.DisplayPath)"
    }
    $stream = [IO.File]::Open(
        $resolution.OperationalPath,
        [IO.FileMode]::Open,
        [IO.FileAccess]::Read,
        [IO.FileShare]::Read)
    try {
        $length = $stream.Length
        if ($length -eq 0) {
            throw "$Label is empty: $($resolution.DisplayPath)"
        }
        if ($length -gt $MaximumBytes) {
            throw "$Label exceeds its byte budget of $MaximumBytes bytes: $($resolution.DisplayPath)"
        }
        [byte[]]$bytes = [byte[]]::new([int]$length)
        $offset = 0
        while ($offset -lt $bytes.Length) {
            $read = $stream.Read(
                $bytes,
                $offset,
                [Math]::Min($script:ResourceManagementEvidenceReadBufferBytes, $bytes.Length - $offset))
            if ($read -eq 0) {
                throw "$Label changed while it was being read: $($resolution.DisplayPath)"
            }
            $offset += $read
        }
        if ($stream.Length -ne $length -or $stream.ReadByte() -ne -1) {
            throw "$Label changed while it was being read: $($resolution.DisplayPath)"
        }
    }
    finally {
        $stream.Dispose()
    }

    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        $sha256 = ConvertTo-ResourceManagementEvidenceSha256 -HashBytes $hasher.ComputeHash($bytes)
    }
    finally {
        $hasher.Dispose()
    }
    try {
        $text = [Text.UTF8Encoding]::new($false, $true).GetString($bytes)
        if ($text.Length -gt 0 -and $text[0] -eq [char]0xFEFF) {
            $text = $text.Substring(1)
        }
        $json = $text | ConvertFrom-Json -ErrorAction Stop
    }
    catch {
        throw "$Label is not strict UTF-8 JSON: $($resolution.DisplayPath): $($_.Exception.Message)"
    }
    return [pscustomobject][ordered]@{
        json = $json
        sha256 = $sha256
        display_path = $resolution.DisplayPath
        bytes = $bytes.Length
    }
}

Export-ModuleMember -Function Get-ResourceManagementJsonEvidence
