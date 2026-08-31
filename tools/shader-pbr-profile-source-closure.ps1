function Get-ZirconShaderPbrSourceClosureRelativePath {
    param(
        [Parameter(Mandatory = $true)][string]$Root,
        [Parameter(Mandatory = $true)][string]$Path
    )

    $resolvedRoot = [System.IO.Path]::GetFullPath($Root).TrimEnd(
        [char[]]@(
            [System.IO.Path]::DirectorySeparatorChar,
            [System.IO.Path]::AltDirectorySeparatorChar
        )
    )
    $resolvedPath = [System.IO.Path]::GetFullPath($Path)
    $prefix = $resolvedRoot + [System.IO.Path]::DirectorySeparatorChar
    if (-not $resolvedPath.StartsWith($prefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Shader PBR profile source closure path escapes its root: $resolvedPath"
    }
    return $resolvedPath.Substring($prefix.Length).Replace("\", "/")
}

function Get-ZirconShaderPbrViewerProductionSourceClosure {
    param([Parameter(Mandatory = $true)][string]$RepoRoot)

    $resolvedRepoRoot = [System.IO.Path]::GetFullPath($RepoRoot)
    $relativeViewerRoot = "zircon_app/src/bin/zircon_shader_pbr_viewer"
    $viewerRoot = [System.IO.Path]::GetFullPath((Join-Path $resolvedRepoRoot $relativeViewerRoot))
    if (-not (Test-Path -LiteralPath $viewerRoot -PathType Container)) {
        throw "Shader PBR profile source closure cannot find the viewer source root: $viewerRoot"
    }
    $viewerPrefix = $viewerRoot.TrimEnd(
        [char[]]@(
            [System.IO.Path]::DirectorySeparatorChar,
            [System.IO.Path]::AltDirectorySeparatorChar
        )
    ) + [System.IO.Path]::DirectorySeparatorChar
    $entryPath = Join-Path $viewerRoot "main.rs"
    if (-not (Test-Path -LiteralPath $entryPath -PathType Leaf)) {
        throw "Shader PBR profile source closure cannot find the viewer entry module: $entryPath"
    }

    $visited = [System.Collections.Generic.HashSet[string]]::new(
        [System.StringComparer]::OrdinalIgnoreCase
    )
    $relativePaths = [System.Collections.Generic.List[string]]::new()
    $visitModule = $null
    $visitModule = {
        param([Parameter(Mandatory = $true)][string]$SourcePath)

        $resolvedSourcePath = [System.IO.Path]::GetFullPath($SourcePath)
        if (-not $resolvedSourcePath.StartsWith($viewerPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
            throw "Shader PBR profile source closure escapes the viewer root: $resolvedSourcePath"
        }
        if (-not (Test-Path -LiteralPath $resolvedSourcePath -PathType Leaf)) {
            throw "Shader PBR profile source closure is missing a declared module: $resolvedSourcePath"
        }
        $sourceItem = Get-Item -LiteralPath $resolvedSourcePath -Force
        if (($sourceItem.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw "Shader PBR profile source closure does not permit reparse-point modules: $resolvedSourcePath"
        }
        if (-not $visited.Add($resolvedSourcePath)) {
            return
        }

        $relativePath = Get-ZirconShaderPbrSourceClosureRelativePath `
            -Root $resolvedRepoRoot `
            -Path $resolvedSourcePath
        if ($relativePath -notlike "$relativeViewerRoot/*") {
            throw "Shader PBR profile source closure produced an invalid viewer path: $relativePath"
        }
        $relativePaths.Add($relativePath)

        $leafName = [System.IO.Path]::GetFileName($resolvedSourcePath)
        $moduleDirectory = if ($leafName -in @("main.rs", "lib.rs", "mod.rs")) {
            Split-Path -Parent $resolvedSourcePath
        }
        else {
            Join-Path (Split-Path -Parent $resolvedSourcePath) ([System.IO.Path]::GetFileNameWithoutExtension($resolvedSourcePath))
        }
        $cfgTestAttributePending = $false
        $pathAttributePending = $null
        foreach ($line in @(Get-Content -LiteralPath $resolvedSourcePath)) {
            $declarationLine = $line
            while ($true) {
                $attributeMatch = [regex]::Match(
                    $declarationLine,
                    '^\s*#\s*\[\s*(.*?)\s*\]\s*(.*)$'
                )
                if (-not $attributeMatch.Success) {
                    break
                }
                $attribute = $attributeMatch.Groups[1].Value
                $declarationLine = $attributeMatch.Groups[2].Value
                if ($attribute -match '^cfg\s*\(\s*test\s*\)$') {
                    $cfgTestAttributePending = $true
                }
                $pathAttributeMatch = [regex]::Match(
                    $attribute,
                    '^path\s*=\s*"([^"]+)"$'
                )
                if ($pathAttributeMatch.Success) {
                    $pathAttributePending = $pathAttributeMatch.Groups[1].Value
                }
                if ([string]::IsNullOrWhiteSpace($declarationLine)) {
                    break
                }
            }
            if ([string]::IsNullOrWhiteSpace($declarationLine)) {
                continue
            }
            $moduleMatch = [regex]::Match(
                $declarationLine,
                '^\s*(?:pub(?:\s*\([^)]*\))?\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;\s*(?://.*)?$'
            )
            if ($moduleMatch.Success) {
                if (-not $cfgTestAttributePending) {
                    $moduleName = $moduleMatch.Groups[1].Value
                    $moduleCandidates = @(
                        if ($null -ne $pathAttributePending) {
                            [System.IO.Path]::GetFullPath(
                                (Join-Path (Split-Path -Parent $resolvedSourcePath) $pathAttributePending)
                            )
                        }
                        else {
                            @(
                                (Join-Path $moduleDirectory "$moduleName.rs"),
                                (Join-Path (Join-Path $moduleDirectory $moduleName) "mod.rs")
                            ) | Where-Object { Test-Path -LiteralPath $_ -PathType Leaf }
                        }
                    )
                    if ($moduleCandidates.Count -eq 0) {
                        throw "Shader PBR profile source closure cannot resolve production module '$moduleName' declared by $resolvedSourcePath"
                    }
                    if ($moduleCandidates.Count -ne 1) {
                        throw "Shader PBR profile source closure found ambiguous production module '$moduleName' declared by $resolvedSourcePath"
                    }
                    & $visitModule $moduleCandidates[0]
                }
                $cfgTestAttributePending = $false
                $pathAttributePending = $null
                continue
            }
            if ($declarationLine -notmatch '^\s*(?://.*)?$') {
                $cfgTestAttributePending = $false
                $pathAttributePending = $null
            }
        }
    }

    & $visitModule $entryPath
    return @($relativePaths | Sort-Object -Unique)
}

function Get-ZirconShaderPbrViewerProductionSourceManifest {
    param([Parameter(Mandatory = $true)][string]$RepoRoot)

    $resolvedRepoRoot = [System.IO.Path]::GetFullPath($RepoRoot)
    $sourcePaths = @(Get-ZirconShaderPbrViewerProductionSourceClosure -RepoRoot $resolvedRepoRoot)
    $records = foreach ($relativePath in $sourcePaths) {
        $sourcePath = [System.IO.Path]::GetFullPath((Join-Path $resolvedRepoRoot $relativePath))
        if (-not $sourcePath.StartsWith($resolvedRepoRoot.TrimEnd(
                [char[]]@(
                    [System.IO.Path]::DirectorySeparatorChar,
                    [System.IO.Path]::AltDirectorySeparatorChar
                )
            ) + [System.IO.Path]::DirectorySeparatorChar, [System.StringComparison]::OrdinalIgnoreCase)) {
            throw "Shader PBR profile source manifest escapes the repository root: $relativePath"
        }
        $sourceItem = Get-Item -LiteralPath $sourcePath -Force
        if (($sourceItem.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw "Shader PBR profile source manifest does not permit reparse-point files: $sourcePath"
        }

        $stream = [System.IO.File]::OpenRead($sourcePath)
        $hasher = [System.Security.Cryptography.SHA256]::Create()
        try {
            $sha256 = -join ($hasher.ComputeHash($stream) | ForEach-Object { $_.ToString("x2") })
        }
        finally {
            $hasher.Dispose()
            $stream.Dispose()
        }
        [pscustomobject]@{
            relative_path = $relativePath
            sha256 = $sha256
            byte_length = [int64]$sourceItem.Length
        }
    }
    return @($records)
}
