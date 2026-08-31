Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$projectionModule = Join-Path $repoRoot 'tools\mvp\MvpAcceptanceStagingProjection.psm1'
Import-Module $projectionModule -Force -ErrorAction Stop

Describe 'MVP acceptance staging projection' {
    It 'encodes projection SHA-256 values through one fixed-size uppercase buffer' {
        $module = Get-Module -Name MvpAcceptanceStagingProjection -ErrorAction Stop
        $bytes = [byte[]]@(0x00, 0x0F, 0x10, 0x7F, 0x80, 0xF0, 0xFF)

        $encoded = & $module {
            param([byte[]]$Value)

            ConvertTo-MvpAcceptanceProjectionUpperHex -Bytes $Value
        } $bytes

        $encoded | Should Be '000F107F80F0FF'
        $moduleSource = Get-Content -LiteralPath $projectionModule -Raw
        $moduleSource | Should Match '\[char\[\]\]::new\(\$Bytes\.Length \* 2\)'
        $moduleSource | Should Not Match 'ForEach-Object \{ \$_.ToString\(''X2''\) \}'
    }

    It 'streams typed no-follow projection traversal without a whole-tree array' {
        $source = Get-Content -LiteralPath $projectionModule -Raw
        $assertFunction = [regex]::Match(
            $source,
            '(?s)function Assert-MvpAcceptanceStagingProjection \{.*?(?=\r?\nExport-ModuleMember)')

        $assertFunction.Success | Should Be $true
        $assertFunction.Value | Should Match '\[System\.Collections\.Generic\.Stack\[IO\.DirectoryInfo\]\]::new\(\)'
        $assertFunction.Value | Should Match '\.EnumerateFileSystemInfos\(\)'
        $assertFunction.Value | Should Match '\$actualIsDirectory = \$item -is \[IO\.DirectoryInfo\]'
        $assertFunction.Value | Should Not Match 'Get-MvpAcceptanceProjectionDescriptor -Item \$item'
        $assertFunction.Value | Should Not Match 'Get-ChildItem'
        $assertFunction.Value | Should Not Match '\[IO\.Path\]::GetFullPath\(\$item\.FullName\)'
    }

    It 'precomputes projection root containment once for the complete inventory' {
        $source = Get-Content -LiteralPath $projectionModule -Raw
        $assertFunction = [regex]::Match(
            $source,
            '(?s)function Assert-MvpAcceptanceStagingProjection \{.*?(?=\r?\nExport-ModuleMember)')

        $assertFunction.Value | Should Match '\$rootPrefix = \[string\]\$Projection\.root_prefix'
        $assertFunction.Value | Should Match '\$absolutePath\.StartsWith\(\$rootPrefix, \[StringComparison\]::OrdinalIgnoreCase\)'
        $assertFunction.Value | Should Match '\$absolutePath\.Substring\(\$rootPrefix\.Length\)\.Replace'
        $assertFunction.Value | Should Not Match 'Get-MvpAcceptanceProjectionRelativePath'
    }

    It 'reuses one normalized root prefix across source and owned projection writers' {
        $source = Get-Content -LiteralPath $projectionModule -Raw
        $newFunction = [regex]::Match(
            $source,
            '(?s)function New-MvpAcceptanceStagingProjection \{.*?(?=\r?\nfunction Add-MvpAcceptanceStagingProjectionSourceEntry)')
        $writerFunctions = [regex]::Match(
            $source,
            '(?s)function Add-MvpAcceptanceStagingProjectionSourceEntry \{.*?(?=\r?\nfunction Assert-MvpAcceptanceStagingProjection)')

        $newFunction.Value | Should Match 'root_prefix = \$rootPrefix'
        $writerFunctions.Value | Should Match '-RootPrefix \$Projection\.root_prefix'
        $writerFunctions.Value | Should Not Match '\[IO\.Path\]::GetFullPath\(\$Projection\.root\)'
        $writerFunctions.Value | Should Not Match 'Get-MvpAcceptanceProjectionRelativePath'
    }

    It 'stores immutable typed descriptors and shares one directory descriptor' {
        $root = Join-Path $TestDrive 'typed-projection'
        $nested = Join-Path $root 'nested\deeper'
        [IO.Directory]::CreateDirectory($nested) | Out-Null
        $path = Join-Path $nested 'fixture.bin'
        $content = [byte[]](1..16)
        [IO.File]::WriteAllBytes($path, $content)

        $projection = New-MvpAcceptanceStagingProjection -Root $root
        Add-MvpAcceptanceStagingProjectionOwnedFile `
            -Projection $projection `
            -Path $path `
            -ContentBytes $content

        ($projection.entries -is [Collections.Generic.Dictionary[
                string, Tuple[bool, Nullable[Int64], string]]]) | Should Be $true
        [Object]::ReferenceEquals(
            $projection.entries['nested'],
            $projection.entries['nested/deeper']) | Should Be $true
        ($null -eq $projection.entries['nested'].Item2) | Should Be $true
        ($null -eq $projection.entries['nested'].Item3) | Should Be $true
        { Assert-MvpAcceptanceStagingProjection -Root $root -Projection $projection } | Should Not Throw

        [IO.File]::WriteAllText(
            (Join-Path $nested 'unexpected.bin'),
            'unexpected',
            [Text.UTF8Encoding]::new($false))
        $unexpectedRejected = $false
        try {
            Assert-MvpAcceptanceStagingProjection -Root $root -Projection $projection
        }
        catch {
            $unexpectedRejected = $_.Exception.Message -match 'unexpected entry'
        }
        $unexpectedRejected | Should Be $true
        { [IO.Directory]::Delete($root, $true) } | Should Not Throw

        $source = Get-Content -LiteralPath $projectionModule -Raw
        $source | Should Match '\$script:MvpAcceptanceProjectionDirectoryDescriptor = \[Tuple'
        $source | Should Not Match '\[pscustomobject\]@\{\s*is_directory'
    }
}
