function Invoke-CoordinatorUnacceptedRequest {
    param(
        [scriptblock]$Invoke,
        [scriptblock]$Parse,
        [switch]$CargoAcquire
    )

    for ($attempt = 1; $attempt -le 3; $attempt++) {
        $raw = @(& $Invoke)
        $exitCode = $LASTEXITCODE
        if ($exitCode -eq 0) { return $raw }
        $response = & $Parse $raw
        $errorProperty = $response.PSObject.Properties['error']
        $retry = $false
        if ($null -ne $errorProperty -and $null -eq $response.PSObject.Properties['request'] -and
            $null -eq $response.PSObject.Properties['job']) {
            $issue = $errorProperty.Value
            # Capacity rejection precedes admission; acquire's fence precedes its job insert.
            $retry = $issue.code -eq 'request_overloaded'
            if ($CargoAcquire -and $issue.code -eq 'admission_checkpoint_stale') {
                $details = $issue.PSObject.Properties['details']
                if ($null -ne $details -and $null -ne $details.Value.PSObject.Properties['operation']) {
                    $retry = $details.Value.operation -like 'cargo.acquire@*'
                }
            }
        }
        if (-not $retry -or $attempt -eq 3) {
            $global:LASTEXITCODE = $exitCode
            return $raw
        }
        Start-Sleep -Milliseconds (250 * $attempt)
    }
}

function Wait-CoordinatorAcceptedRequest {
    param(
        [object]$Failure,
        [scriptblock]$Query,
        [ValidateRange(0, 300)]
        [double]$TimeoutSeconds = 0
    )

    $errorProperty = $Failure.PSObject.Properties['error']
    if ($null -eq $errorProperty) { return $null }
    $issue = $errorProperty.Value
    if ($issue.code -ne 'command_post_timeout') { return $null }
    $requestId = [string]$issue.details.requestId
    if ($issue.details.phase -ne 'post_response' -or $requestId -cnotmatch '^[a-f0-9]{32}$') {
        return $null
    }

    # An accepted allocation may finish late; retain its identity until terminal.
    $timer = [Diagnostics.Stopwatch]::StartNew()
    do {
        $status = & $Query $requestId
        $queryError = $status.PSObject.Properties['error']
        if ($null -eq $status.PSObject.Properties['request'] -and $null -ne $queryError -and
            $queryError.Value.code -eq 'request_overloaded') {
            Start-Sleep -Milliseconds 500
            continue
        }
        if ($status.request.requestId -cne $requestId) {
            throw "Coordinator request recovery returned a different identity for $requestId."
        }
        switch ($status.request.status) {
            'completed' {
                $result = $status.PSObject.Properties['result']
                if ($null -eq $result -or $null -eq $result.Value) {
                    throw "Coordinator request $requestId completed without a result."
                }
                return $result.Value
            }
            'failed' {
                throw "Coordinator request $requestId failed: $($status.error.code): $($status.error.message)"
            }
            'accepted' { }
            default { throw "Coordinator request $requestId has an invalid recovery status." }
        }
        Start-Sleep -Milliseconds 250
    } while ($TimeoutSeconds -eq 0 -or $timer.Elapsed.TotalSeconds -lt $TimeoutSeconds)
    throw "Coordinator request $requestId remains accepted after $TimeoutSeconds seconds; reconcile this request before retrying."
}

function Wait-CoordinatorQueuedRequest {
    param(
        [object]$Response,
        [scriptblock]$Query,
        [scriptblock]$Resume,
        [ValidateRange(0.001, 300)]
        [double]$TimeoutSeconds = 120
    )

    if ($null -eq $Response) { return $null }
    $responseStatus = $Response.PSObject.Properties['status']
    if ($null -eq $responseStatus -or $responseStatus.Value -ne 'queued') {
        return $null
    }

    $queueId = [string]$Response.queueId
    $command = [string]$Response.command
    if ($queueId -cnotmatch '^[a-f0-9]{32}$') {
        throw "Coordinator offline queue identity '$queueId' is invalid."
    }
    if ($command -notin @('session.register', 'session.heartbeat', 'lease.heartbeat')) {
        throw "Coordinator queued command '$command' is not replay-safe."
    }

    $timer = [Diagnostics.Stopwatch]::StartNew()
    do {
        $status = & $Query
        if ($null -eq $status -or $status.status -eq 'offline') {
            Start-Sleep -Milliseconds 250
            continue
        }
        if ($status.status -ne 'ok') {
            throw "Coordinator service is unhealthy while replaying queue $queueId."
        }
        $replay = $status.PSObject.Properties['offlineReplay']
        if ($null -eq $replay -or $null -eq $replay.Value) {
            throw "Coordinator status omitted offline replay state for queue $queueId."
        }
        foreach ($field in @('failed', 'quarantined')) {
            $count = $replay.Value.PSObject.Properties[$field]
            if ($null -ne $count -and [int]$count.Value -gt 0) {
                throw "Coordinator offline queue $queueId failed during replay."
            }
        }
        if ([int]$replay.Value.retained -eq 0) {
            $resumed = & $Resume
            if ($null -eq $resumed) {
                throw "Coordinator offline queue $queueId completed without a result."
            }
            $resumedStatus = $resumed.PSObject.Properties['status']
            if ($null -ne $resumedStatus -and $resumedStatus.Value -eq 'queued') {
                throw "Coordinator offline queue $queueId remained queued after replay."
            }
            return $resumed
        }
        Start-Sleep -Milliseconds 250
    } while ($timer.Elapsed.TotalSeconds -lt $TimeoutSeconds)
    throw "Coordinator offline queue $queueId remains pending after $TimeoutSeconds seconds."
}
