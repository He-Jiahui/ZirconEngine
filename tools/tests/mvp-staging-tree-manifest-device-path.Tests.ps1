$modulePath = Join-Path $PSScriptRoot '..\mvp\MvpAcceptanceStagingTreeManifest.psm1'
Import-Module $modulePath -Force -ErrorAction Stop

Describe 'MVP staging tree manifest device paths' {
    It 'writes and reads the manifest through the operational device path' {
        $stageRoot = Join-Path $TestDrive 'stage'
        [IO.Directory]::CreateDirectory($stageRoot) | Out-Null
        [IO.File]::WriteAllText(
            [IO.Path]::Combine($stageRoot, 'payload.txt'),
            'payload',
            [Text.UTF8Encoding]::new($false))
        $deviceRoot = "\\?\$([IO.Path]::GetFullPath($stageRoot))"

        $manifestPath = Write-MvpAcceptanceStagingTreeManifest -StagingRoot $deviceRoot
        $entries = @(Read-MvpAcceptanceStagingTreeManifest -StagingRoot $deviceRoot)

        $manifestPath | Should Be ([IO.Path]::Combine($deviceRoot, 'staging-tree-manifest.json'))
        [IO.File]::Exists($manifestPath) | Should Be $true
        $entries.Count | Should Be 1
        $entries[0].relative_path | Should Be 'payload.txt'
    }

    It 'rejects every rooted or drive-relative manifest entry' {
        foreach ($entryPath in @(
                'C:\escape.txt',
                '\rooted.txt',
                '\\server\share\escape.txt',
                'C:drive-relative.txt',
                '\\?\C:\escape.txt')) {
            $rejected = $false
            try {
                Resolve-MvpAcceptanceStagingTreeManifestEntryPath `
                    -Root $TestDrive `
                    -RelativePath $entryPath | Out-Null
            }
            catch {
                $rejected = $_.Exception.Message -match 'is not a relative path'
            }

            $rejected | Should Be $true
        }
    }
}
