Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$treeManifestModule = Join-Path $repoRoot 'tools\mvp\MvpAcceptanceStagingTreeManifest.psm1'
Import-Module $treeManifestModule -Force -ErrorAction Stop

Describe 'MVP acceptance staging tree manifest' {
    It 'encodes manifest SHA-256 values through one fixed-size uppercase buffer' {
        $module = Get-Module -Name MvpAcceptanceStagingTreeManifest -ErrorAction Stop
        $bytes = [byte[]]@(0x00, 0x0F, 0x10, 0x7F, 0x80, 0xF0, 0xFF)

        $encoded = & $module {
            param([byte[]]$Value)

            ConvertTo-MvpAcceptanceStagingTreeManifestUpperHex -Bytes $Value
        } $bytes

        $encoded | Should Be '000F107F80F0FF'
        $moduleSource = Get-Content -LiteralPath $treeManifestModule -Raw
        $moduleSource | Should Match '\[char\[\]\]::new\(\$Bytes\.Length \* 2\)'
        $moduleSource | Should Not Match 'ForEach-Object \{ \$_.ToString\(''X2''\) \}'
    }

    It 'reuses one SHA-256 instance across the complete manifest inventory' {
        $tokens = $null
        $errors = $null
        $ast = [Management.Automation.Language.Parser]::ParseFile(
            $treeManifestModule,
            [ref]$tokens,
            [ref]$errors)
        $errors.Count | Should Be 0
        $inventory = $ast.Find(
            { param($node) $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -eq 'Get-MvpAcceptanceStagingTreeManifestEntries' },
            $true)
        $hashFile = $ast.Find(
            { param($node) $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -eq 'Get-MvpAcceptanceStagingTreeManifestSha256' },
            $true)

        @([regex]::Matches($inventory.Extent.Text, '\[Security\.Cryptography\.SHA256\]::Create\(\)')).Count |
            Should Be 1
        $inventory.Extent.Text | Should Match 'Get-MvpAcceptanceStagingTreeManifestSha256[\s\S]+-Hasher \$hasher'
        $inventory.Extent.Text | Should Match 'finally[\s\S]+\$hasher\.Dispose\(\)'
        $hashFile.Extent.Text | Should Match '\[Security\.Cryptography\.SHA256\]\$Hasher'
    }

    It 'streams each directory through a typed queue without per-directory child arrays' {
        $source = Get-Content -LiteralPath $treeManifestModule -Raw
        $inventory = [regex]::Match(
            $source,
            '(?s)function Get-MvpAcceptanceStagingTreeManifestEntries \{.*?(?=\r?\nfunction Write-MvpAcceptanceStagingTreeManifest)')

        $inventory.Success | Should Be $true
        $inventory.Value | Should Match '\[System\.Collections\.Generic\.Queue\[IO\.DirectoryInfo\]\]::new\(\)'
        $inventory.Value | Should Match '\.EnumerateFileSystemInfos\(\)'
        $inventory.Value | Should Match '\$directories\.Enqueue\(\[IO\.DirectoryInfo\]\$child\)'
        $inventory.Value | Should Not Match 'Get-ChildItem'
        $inventory.Value | Should Not Match 'foreach \(\$child in @\('
    }

    It 'precomputes manifest root containment once for the complete inventory' {
        $source = Get-Content -LiteralPath $treeManifestModule -Raw
        $inventory = [regex]::Match(
            $source,
            '(?s)function Get-MvpAcceptanceStagingTreeManifestEntries \{.*?(?=\r?\nfunction Write-MvpAcceptanceStagingTreeManifest)')

        $inventory.Value | Should Match '\$rootPrefix = \$absoluteRoot\.TrimEnd'
        $inventory.Value | Should Match '\$childPath\.StartsWith\(\$rootPrefix, \[StringComparison\]::OrdinalIgnoreCase\)'
        $inventory.Value | Should Match '\$childPath\.Substring\(\$rootPrefix\.Length\)\.Replace'
        $inventory.Value | Should Not Match 'ConvertTo-MvpAcceptanceStagingTreeManifestRelativePath'
    }

    It 'validates normalized manifest paths without split or filtered segment arrays' {
        $source = Get-Content -LiteralPath $treeManifestModule -Raw
        $resolver = [regex]::Match(
            $source,
            '(?s)function Resolve-MvpAcceptanceStagingTreeManifestNormalizedEntryPath \{.*?(?=\r?\nfunction Resolve-MvpAcceptanceStagingTreeManifestEntryPath)')

        $resolver.Success | Should Be $true
        $resolver.Value | Should Match '\$NormalizedRelativePath\.IndexOf\(''/\.\/'', \[StringComparison\]::Ordinal\)'
        $resolver.Value | Should Match '\[IO\.Path\]::Combine\(\$AbsoluteRoot, \$platformRelativePath\)'
        $resolver.Value | Should Not Match '\.Split\('
        $resolver.Value | Should Not Match 'Where-Object'
        $resolver.Value | Should Not Match '\$segments'
    }

    It 'precomputes path depth once and removes the internal sort key before returning' {
        $source = Get-Content -LiteralPath $treeManifestModule -Raw
        $reader = [regex]::Match(
            $source,
            '(?s)function Read-MvpAcceptanceStagingTreeManifest \{.*?(?=\r?\nExport-ModuleMember)')

        $reader.Value | Should Match '\$sortDepth = 1'
        $reader.Value | Should Match '\$normalizedRelativePath\[\$index\] -eq ''/'''
        $reader.Value | Should Match 'sort_depth = \$sortDepth'
        $reader.Value | Should Match 'Sort-Object -Property sort_depth, relative_path'
        $reader.Value | Should Match '\.Properties\.Remove\(''sort_depth''\)'
        $reader.Value | Should Not Match 'relative_path\.Split\('
    }

    It 'uses the parsed entries directly without copying the complete reference array' {
        $source = Get-Content -LiteralPath $treeManifestModule -Raw
        $reader = [regex]::Match(
            $source,
            '(?s)function Read-MvpAcceptanceStagingTreeManifest \{.*?(?=\r?\nExport-ModuleMember)')

        $reader.Value | Should Match '\$manifestEntries = \$manifest\.entries'
        $reader.Value | Should Match '\$manifestEntries -is \[array\]'
        $reader.Value | Should Not Match '\$manifestEntries = @\(\$manifest\.entries\)'
    }

    It 'validates each entry kind without a per-entry literal array' {
        $source = Get-Content -LiteralPath $treeManifestModule -Raw
        $reader = [regex]::Match(
            $source,
            '(?s)function Read-MvpAcceptanceStagingTreeManifest \{.*?(?=\r?\nExport-ModuleMember)')

        $reader.Value | Should Match '\$kind -cne ''file'' -and \$kind -cne ''directory'''
        $reader.Value | Should Not Match '\$kind -notin @\('
    }
}
