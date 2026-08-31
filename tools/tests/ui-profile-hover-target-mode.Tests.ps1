$script:CaptureScript = Join-Path $PSScriptRoot "..\ui-profile-capture.ps1"
$script:NativeInteraction = Join-Path $PSScriptRoot "..\ui-profile-native-resize.ps1"

Describe "UI profile hover target mode contract" {
    BeforeAll {
        . $script:NativeInteraction
    }

    It "accepts two coordinates inside one stable target for same-target hover" {
        $targets = @(
            [pscustomobject]@{
                target_id = "template.left.Button"
                target_kind = "template_control"
                target_surface = "left"
                X = 120
                Y = 80
            },
            [pscustomobject]@{
                target_id = "template.left.Button"
                target_kind = "template_control"
                target_surface = "left"
                X = 140
                Y = 80
            }
        )

        Test-ZirconPointerTargetModeEvidence -Targets $targets -TargetMode "same_target" |
            Should Be $true

        $targets[1].X = 120
        Test-ZirconPointerTargetModeEvidence -Targets $targets -TargetMode "same_target" |
            Should Be $false

        $targets[1].X = 140
        $targets[1].target_id = "template.left.OtherButton"
        Test-ZirconPointerTargetModeEvidence -Targets $targets -TargetMode "same_target" |
            Should Be $false
    }

    It "requires distinct target identities for cross-target hover" {
        $targets = @(
            [pscustomobject]@{
                target_id = "template.left.Button"
                target_kind = "template_control"
                target_surface = "left"
                X = 120
                Y = 80
            },
            [pscustomobject]@{
                target_id = "template.document.Tab"
                target_kind = "document_tab"
                target_surface = "document"
                X = 420
                Y = 40
            }
        )

        Test-ZirconPointerTargetModeEvidence -Targets $targets -TargetMode "cross_target" |
            Should Be $true

        $targets[1].target_id = $targets[0].target_id
        $targets[1].target_kind = $targets[0].target_kind
        $targets[1].target_surface = $targets[0].target_surface
        Test-ZirconPointerTargetModeEvidence -Targets $targets -TargetMode "cross_target" |
            Should Be $false
    }

    It "keeps viewport spatial probes separate from hover target-transition evidence" {
        $targets = @(
            [pscustomobject]@{
                target_id = "scene_viewport.center"
                target_kind = "scene_viewport"
                target_surface = "document"
                X = 640
                Y = 360
            },
            [pscustomobject]@{
                target_id = "scene_viewport.corner"
                target_kind = "scene_viewport"
                target_surface = "document"
                X = 920
                Y = 620
            }
        )

        Test-ZirconPointerTargetModeEvidence -Targets $targets -TargetMode "spatial_probe" |
            Should Be $true

        $targets[1].X = $targets[0].X
        $targets[1].Y = $targets[0].Y
        Test-ZirconPointerTargetModeEvidence -Targets $targets -TargetMode "spatial_probe" |
            Should Be $false
    }

    It "binds the hover target mode to target generation, evidence, and the source manifest" {
        $captureSource = Get-Content -LiteralPath $script:CaptureScript -Raw
        $interactionSource = Get-Content -LiteralPath $script:NativeInteraction -Raw

        $captureSource | Should Match '\[ValidateSet\("cross_target", "same_target"\)\]'
        $captureSource | Should Match '\[string\]\$AutoHoverTargetMode = "cross_target"'
        $captureSource | Should Match 'Get-LiveGeometryInteractionTargets[\s\S]*-TargetMode \$AutoHoverTargetMode'
        $captureSource | Should Match '-TargetMode \$pointerTargetMode'
        $captureSource | Should Match 'auto_hover_target_mode = \$AutoHoverTargetMode'
        $captureSource | Should Match 'Test-ZirconPointerTargetModeEvidence'
        $interactionSource | Should Match 'target_mode = \$TargetMode'
    }
}
