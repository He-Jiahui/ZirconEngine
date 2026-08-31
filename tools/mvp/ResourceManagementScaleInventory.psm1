Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$script:ResourceManagementInventoryUpperHexDigits = [char[]]'0123456789ABCDEF'

function ConvertTo-ResourceManagementInventorySha256 {
    param([Parameter(Mandatory)][byte[]]$HashBytes)

    [char[]]$characters = [char[]]::new($HashBytes.Length * 2)
    for ($index = 0; $index -lt $HashBytes.Length; $index++) {
        $value = [int]$HashBytes[$index]
        $characters[$index * 2] = $script:ResourceManagementInventoryUpperHexDigits[$value -shr 4]
        $characters[($index * 2) + 1] = $script:ResourceManagementInventoryUpperHexDigits[$value -band 0x0F]
    }
    return [string]::new($characters)
}

function Get-ResourceManagementFileSha256 {
    param([Parameter(Mandatory)][string]$Path)

    $hash = [Security.Cryptography.IncrementalHash]::CreateHash(
        [Security.Cryptography.HashAlgorithmName]::SHA256
    )
    $stream = [IO.FileStream]::new($Path, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
    [byte[]]$buffer = [byte[]]::new(81920)
    try {
        while (($read = $stream.Read($buffer, 0, $buffer.Length)) -gt 0) {
            $hash.AppendData($buffer, 0, $read)
        }
        return ConvertTo-ResourceManagementInventorySha256 -HashBytes $hash.GetHashAndReset()
    }
    finally {
        $stream.Dispose()
        $hash.Dispose()
    }
}

function Get-ResourceManagementScaleInventorySha256 {
    param(
        [Parameter(Mandatory)][string]$DataRoot,
        [Parameter(Mandatory)][ValidateRange(1, 100000)][int]$DataAssetCount
    )

    $encoding = [Text.UTF8Encoding]::new($false)
    $separator = [byte[]]@(0)
    $hash = [Security.Cryptography.IncrementalHash]::CreateHash(
        [Security.Cryptography.HashAlgorithmName]::SHA256
    )
    [byte[]]$buffer = [byte[]]::new(81920)
    try {
        for ($index = 1; $index -le $DataAssetCount; $index++) {
            $fileName = 'catalog_{0:D6}.json' -f $index
            $virtualPath = 'res://data/' + $fileName
            $sourcePath = [IO.Path]::GetFullPath([IO.Path]::Combine($DataRoot, $fileName))
            if (-not [IO.File]::Exists($sourcePath)) {
                throw "Resource-management scale inventory source does not exist: $virtualPath"
            }
            $hash.AppendData($encoding.GetBytes($virtualPath))
            $hash.AppendData($separator)
            $stream = [IO.FileStream]::new($sourcePath, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
            try {
                while (($read = $stream.Read($buffer, 0, $buffer.Length)) -gt 0) {
                    $hash.AppendData($buffer, 0, $read)
                }
            }
            finally {
                $stream.Dispose()
            }
            $hash.AppendData($separator)
        }
        return ConvertTo-ResourceManagementInventorySha256 -HashBytes $hash.GetHashAndReset()
    }
    finally {
        $hash.Dispose()
    }
}

Export-ModuleMember -Function Get-ResourceManagementFileSha256, Get-ResourceManagementScaleInventorySha256
