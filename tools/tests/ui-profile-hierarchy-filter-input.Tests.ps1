$script:HierarchyFilterInput = Join-Path $PSScriptRoot "..\ui-profile-hierarchy-filter-input.ps1"

. $script:HierarchyFilterInput

Describe "ui-profile hierarchy filter input" {
    It "selects exactly the live hierarchy search control from profile geometry" {
        $geometry = [pscustomobject]@{
            template_controls = @(
                [pscustomobject]@{
                    id = 'template.left.HierarchySearchQuery'
                    kind = 'template_control'
                    surface = 'left'
                    frame = [pscustomobject]@{ x = 12.0; y = 48.0; width = 160.0; height = 28.0 }
                },
                [pscustomobject]@{
                    id = 'template.left.Hierarchy/AddEntity'
                    kind = 'template_control'
                    surface = 'left'
                    frame = [pscustomobject]@{ x = 176.0; y = 48.0; width = 28.0; height = 28.0 }
                }
            )
        }

        $target = Find-ZirconHierarchyFilterProfileTarget -Geometry $geometry

        $target.id | Should Be 'template.left.HierarchySearchQuery'
        $target.kind | Should Be 'template_control'
        $target.surface | Should Be 'left'
    }

    It "rejects ambiguous or invisible hierarchy search geometry" {
        $ambiguousGeometry = [pscustomobject]@{
            template_controls = @(
                [pscustomobject]@{
                    id = 'template.left.HierarchySearchQuery'
                    kind = 'template_control'
                    surface = 'left'
                    frame = [pscustomobject]@{ x = 12.0; y = 48.0; width = 160.0; height = 28.0 }
                },
                [pscustomobject]@{
                    id = 'template.left.HierarchySearchQuery'
                    kind = 'template_control'
                    surface = 'left'
                    frame = [pscustomobject]@{ x = 12.0; y = 80.0; width = 160.0; height = 28.0 }
                }
            )
        }

        { Find-ZirconHierarchyFilterProfileTarget -Geometry $ambiguousGeometry } |
            Should Throw 'Hierarchy filter profiling requires exactly one visible template.left.HierarchySearchQuery control.'
    }

    It "preserves UTF-16 units for Unicode text injection" {
        $query = 'Scene ' + [char]0x4E2D + [char]0xD83D + [char]0xDE00
        $units = @(Get-ZirconProfileUtf16CodeUnits -Text $query)

        $units.Count | Should Be 9
        $units[0] | Should Be ([uint16][char]'S')
        $units[5] | Should Be ([uint16][char]' ')
        $units[6] | Should Be ([uint16]0x4E2D)
        $units[7] | Should Be ([uint16]0xD83D)
        $units[8] | Should Be ([uint16]0xDE00)
    }

    It "matches the native Windows INPUT ABI before injecting Unicode text" {
        Initialize-ZirconProfileUnicodeInputApi

        $expectedInputSize = if ([IntPtr]::Size -eq 8) { 40 } else { 28 }
        [ZirconProfileUnicodeInputNative]::GetInputSize() | Should Be $expectedInputSize
    }

    It "defines a complete native text reset before each configured query" {
        Initialize-ZirconProfileUnicodeInputApi

        [ZirconProfileUnicodeInputNative]::GetTextResetInputCount() | Should Be 6
    }
}
