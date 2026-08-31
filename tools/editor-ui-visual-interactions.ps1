Set-StrictMode -Version Latest

function Get-ZirconEditorVisualProfileControlCenter {
    param(
        [Parameter(Mandatory = $true)][object]$ProfileGeometry,
        [Parameter(Mandatory = $true)][string]$ControlId
    )

    $matches = @(
        foreach ($collectionName in @('template_controls', 'viewport_toolbar_controls')) {
            $collection = $ProfileGeometry.PSObject.Properties[$collectionName]
            if ($null -eq $collection) {
                continue
            }
            @($collection.Value) | Where-Object { [string]$_.id -eq $ControlId }
        }
    )
    if ($matches.Count -ne 1) {
        throw "Expected one source-bound profile frame for '$ControlId', got $($matches.Count)."
    }

    $frame = $matches[0].frame
    $x = [double]$frame.x
    $y = [double]$frame.y
    $width = [double]$frame.width
    $height = [double]$frame.height
    if ([double]::IsNaN($x) -or [double]::IsInfinity($x) -or
        [double]::IsNaN($y) -or [double]::IsInfinity($y) -or
        [double]::IsNaN($width) -or [double]::IsInfinity($width) -or
        [double]::IsNaN($height) -or [double]::IsInfinity($height) -or
        $width -le 0.0 -or
        $height -le 0.0) {
        throw "Profile frame for '$ControlId' is not a finite positive rectangle."
    }

    [pscustomobject]@{
        X = [int][Math]::Floor($x + $width * 0.5)
        Y = [int][Math]::Floor($y + $height * 0.5)
        Frame = $frame
    }
}

function ConvertTo-ZirconEditorVisualPointerLParam {
    param(
        [Parameter(Mandatory = $true)][int]$X,
        [Parameter(Mandatory = $true)][int]$Y
    )

    if ($X -lt 0 -or $X -gt [uint16]::MaxValue -or
        $Y -lt 0 -or $Y -gt [uint16]::MaxValue) {
        throw "Pointer coordinate is outside the Win32 client-message domain: ($X,$Y)."
    }
    $packed = ([long]($Y -band 0xffff) -shl 16) -bor [long]($X -band 0xffff)
    [IntPtr]::new($packed)
}

function Invoke-ZirconEditorVisualPointerMove {
    param(
        [Parameter(Mandatory = $true)][IntPtr]$Window,
        [Parameter(Mandatory = $true)][int]$X,
        [Parameter(Mandatory = $true)][int]$Y,
        [int]$WaitMilliseconds = 0
    )

    $screenPoint = New-Object ZirconEditorVisualCapturePoint
    $screenPoint.X = $X
    $screenPoint.Y = $Y
    if (-not [ZirconEditorVisualCaptureNative]::ClientToScreen($Window, [ref]$screenPoint)) {
        throw "Could not resolve the screen point for client position ($X,$Y)."
    }
    [ZirconEditorVisualCaptureNative]::SetForegroundWindow($Window) | Out-Null
    if (-not [ZirconEditorVisualCaptureNative]::SetCursorPos($screenPoint.X, $screenPoint.Y)) {
        throw "Could not position the pointer at client position ($X,$Y)."
    }
    if (-not [ZirconEditorVisualCaptureNative]::PostMessage(
            $Window,
            0x0200,
            [IntPtr]::Zero,
            (ConvertTo-ZirconEditorVisualPointerLParam -X $X -Y $Y))) {
        throw "Could not post a native pointer move at client position ($X,$Y)."
    }
    [ZirconEditorVisualCaptureNative]::DwmFlush() | Out-Null
    if ($WaitMilliseconds -gt 0) {
        Start-Sleep -Milliseconds $WaitMilliseconds
    }

    [pscustomobject]@{
        client_x = $X
        client_y = $Y
        screen_x = $screenPoint.X
        screen_y = $screenPoint.Y
    }
}

function Invoke-ZirconEditorVisualControlHover {
    param(
        [Parameter(Mandatory = $true)][IntPtr]$Window,
        [Parameter(Mandatory = $true)][object]$ProfileGeometry,
        [Parameter(Mandatory = $true)][string]$ControlId,
        [int]$WaitMilliseconds = 350
    )

    $center = Get-ZirconEditorVisualProfileControlCenter `
        -ProfileGeometry $ProfileGeometry `
        -ControlId $ControlId
    $pointer = Invoke-ZirconEditorVisualPointerMove `
        -Window $Window `
        -X $center.X `
        -Y $center.Y `
        -WaitMilliseconds $WaitMilliseconds
    [pscustomobject]@{
        control_id = $ControlId
        client_x = $pointer.client_x
        client_y = $pointer.client_y
        screen_x = $pointer.screen_x
        screen_y = $pointer.screen_y
        wait_milliseconds = $WaitMilliseconds
        frame = $center.Frame
    }
}

function Invoke-ZirconEditorVisualControlClick {
    param(
        [Parameter(Mandatory = $true)][IntPtr]$Window,
        [Parameter(Mandatory = $true)][object]$ProfileGeometry,
        [Parameter(Mandatory = $true)][string]$ControlId
    )

    $center = Get-ZirconEditorVisualProfileControlCenter `
        -ProfileGeometry $ProfileGeometry `
        -ControlId $ControlId
    $pointer = Invoke-ZirconEditorVisualPointerMove `
        -Window $Window `
        -X $center.X `
        -Y $center.Y
    $lParam = ConvertTo-ZirconEditorVisualPointerLParam -X $center.X -Y $center.Y
    foreach ($message in @(
            @{ Id = 0x0201; WParam = 1 },
            @{ Id = 0x0202; WParam = 0 }
        )) {
        if (-not [ZirconEditorVisualCaptureNative]::PostMessage(
                $Window,
                $message.Id,
                [IntPtr]::new([long]$message.WParam),
                $lParam)) {
            throw "Could not post native pointer message $($message.Id) to '$ControlId'."
        }
    }
    [ZirconEditorVisualCaptureNative]::DwmFlush() | Out-Null
    Start-Sleep -Milliseconds 350

    [pscustomobject]@{
        control_id = $ControlId
        client_x = $center.X
        client_y = $center.Y
        screen_x = $pointer.screen_x
        screen_y = $pointer.screen_y
        frame = $center.Frame
    }
}

function Measure-ZirconEditorVisualRegionDifference {
    param(
        [Parameter(Mandatory = $true)][string]$BeforePath,
        [Parameter(Mandatory = $true)][string]$AfterPath,
        [Parameter(Mandatory = $true)][int]$RegionLeft,
        [Parameter(Mandatory = $true)][int]$RegionTop,
        [int]$RegionRight = -1,
        [int]$RegionBottom = -1,
        [int]$Stride = 2,
        [int]$MinimumChannelDelta = 12
    )

    if ($Stride -le 0) {
        throw 'Visual difference stride must be positive.'
    }
    $before = [System.Drawing.Bitmap]::new($BeforePath)
    $after = [System.Drawing.Bitmap]::new($AfterPath)
    try {
        if ($before.Width -ne $after.Width -or $before.Height -ne $after.Height) {
            throw 'Visual difference images must have the same physical extent.'
        }
        $left = [Math]::Max(0, [Math]::Min($RegionLeft, $before.Width))
        $top = [Math]::Max(0, [Math]::Min($RegionTop, $before.Height))
        $right = if ($RegionRight -lt 0) {
            $before.Width
        }
        else {
            [Math]::Max(0, [Math]::Min($RegionRight, $before.Width))
        }
        $bottom = if ($RegionBottom -lt 0) {
            $before.Height
        }
        else {
            [Math]::Max(0, [Math]::Min($RegionBottom, $before.Height))
        }
        if ($right -le $left -or $bottom -le $top) {
            throw 'Visual difference region must have positive width and height.'
        }
        $sampled = 0
        $different = 0
        $maximumChannelDelta = 0
        for ($y = $top; $y -lt $bottom; $y += $Stride) {
            for ($x = $left; $x -lt $right; $x += $Stride) {
                $beforePixel = $before.GetPixel($x, $y)
                $afterPixel = $after.GetPixel($x, $y)
                $delta = [Math]::Max(
                    [Math]::Abs([int]$beforePixel.R - [int]$afterPixel.R),
                    [Math]::Max(
                        [Math]::Abs([int]$beforePixel.G - [int]$afterPixel.G),
                        [Math]::Abs([int]$beforePixel.B - [int]$afterPixel.B)))
                $maximumChannelDelta = [Math]::Max($maximumChannelDelta, $delta)
                $sampled += 1
                if ($delta -ge $MinimumChannelDelta) {
                    $different += 1
                }
            }
        }
        if ($sampled -eq 0) {
            throw 'Visual difference region contains no sampled pixels.'
        }

        [pscustomobject]@{
            region_left = $left
            region_top = $top
            region_right = $right
            region_bottom = $bottom
            stride = $Stride
            minimum_channel_delta = $MinimumChannelDelta
            sampled_pixels = $sampled
            different_pixels = $different
            different_pixel_ratio = $different / [double]$sampled
            maximum_channel_delta = $maximumChannelDelta
        }
    }
    finally {
        $before.Dispose()
        $after.Dispose()
    }
}
