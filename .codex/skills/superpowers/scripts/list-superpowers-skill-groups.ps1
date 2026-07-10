param(
    [string]$Root = ".codex/skills/superpowers"
)

$resolvedRoot = (Resolve-Path -Path $Root).Path

Write-Output "superpowers/"
foreach ($category in Get-ChildItem $resolvedRoot -Directory | Where-Object { $_.Name -ne "scripts" } | Sort-Object Name) {
    $items = Get-ChildItem $category.FullName -Directory | Sort-Object Name
    Write-Output ("  {0}/ [{1}]" -f $category.Name, $items.Count)
    foreach ($item in $items) {
        Write-Output ("    {0}/" -f $item.Name)
    }
}

Write-Output "  scripts/"
foreach ($script in Get-ChildItem (Join-Path $resolvedRoot "scripts") -File | Sort-Object Name) {
    Write-Output ("    {0}" -f $script.Name)
}
