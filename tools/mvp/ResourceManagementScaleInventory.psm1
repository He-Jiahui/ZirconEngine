Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

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
    try {
        for ($index = 1; $index -le $DataAssetCount; $index++) {
            $fileName = 'catalog_{0:D6}.json' -f $index
            $virtualPath = 'res://data/' + $fileName
            $sourcePath = Join-Path $DataRoot $fileName
            if (-not [IO.File]::Exists($sourcePath)) {
                throw "Resource-management scale inventory source does not exist: $virtualPath"
            }
            $hash.AppendData($encoding.GetBytes($virtualPath))
            $hash.AppendData($separator)
            $hash.AppendData([IO.File]::ReadAllBytes($sourcePath))
            $hash.AppendData($separator)
        }
        return [Convert]::ToHexString($hash.GetHashAndReset())
    }
    finally {
        $hash.Dispose()
    }
}

Export-ModuleMember -Function Get-ResourceManagementScaleInventorySha256
