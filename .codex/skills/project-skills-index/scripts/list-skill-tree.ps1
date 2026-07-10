param(
    [string]$SkillsRoot = ".codex/skills"
)

$resolvedRoot = Resolve-Path -Path $SkillsRoot
$skillsRootPath = $resolvedRoot.Path

Write-Output ("Skills root: {0}" -f $skillsRootPath)
Write-Output ""

$skillDirs = Get-ChildItem -Path $skillsRootPath -Directory | Sort-Object Name

foreach ($skillDir in $skillDirs) {
    Write-Output ("{0}/" -f $skillDir.Name)

    $children = Get-ChildItem -Path $skillDir.FullName -Force |
        Sort-Object @{ Expression = { -not $_.PSIsContainer } }, Name

    foreach ($child in $children) {
        $suffix = ""
        if ($child.PSIsContainer) {
            $suffix = "/"
        }
        Write-Output ("  {0}{1}" -f $child.Name, $suffix)
    }

    $skillMdPath = Join-Path $skillDir.FullName "SKILL.md"
    if (Test-Path -Path $skillMdPath) {
        $frontmatter = Get-Content -Path $skillMdPath -TotalCount 12
        $skillName = $null
        $description = $null

        foreach ($line in $frontmatter) {
            if (-not $skillName -and $line -match '^name:\s*(.+)$') {
                $skillName = $Matches[1].Trim()
                continue
            }
            if (-not $description -and $line -match '^description:\s*(.+)$') {
                $description = $Matches[1].Trim()
                continue
            }
        }

        if (-not $skillName) {
            $skillName = $skillDir.Name
        }

        if ($description) {
            Write-Output ("  summary: {0} | {1}" -f $skillName, $description)
        } else {
            Write-Output ("  summary: {0}" -f $skillName)
        }
    } else {
        Write-Output "  summary: collection directory with no top-level SKILL.md"
    }

    Write-Output ""
}
