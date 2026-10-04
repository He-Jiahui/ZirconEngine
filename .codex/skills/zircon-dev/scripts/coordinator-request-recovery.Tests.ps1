$recoveryScript = Join-Path $PSScriptRoot 'coordinator-request-recovery.ps1'
if (Test-Path -LiteralPath $recoveryScript) { . $recoveryScript }
Set-StrictMode -Version Latest

Describe 'Unaccepted coordinator request retries' {
    BeforeEach {
        $script:attempts = 0
        Mock Start-Sleep {}
    }

    It 'retries a capacity rejection before returning the successful response' {
        $raw = Invoke-CoordinatorUnacceptedRequest -Invoke {
            $script:attempts++
            $global:LASTEXITCODE = if ($script:attempts -eq 1) { 1 } else { 0 }
            if ($script:attempts -eq 1) { return '{"error":{"code":"request_overloaded"}}' }
            '{"job":{"job_id":"one-job"}}'
        } -Parse { param($raw) ($raw -join "`n") | ConvertFrom-Json }
        ($raw | ConvertFrom-Json).job.job_id | Should Be 'one-job'
        $LASTEXITCODE | Should Be 0
        $script:attempts | Should Be 2
    }

    It 'bounds stale acquire retries and preserves the final exit code' {
        $raw = Invoke-CoordinatorUnacceptedRequest -CargoAcquire -Invoke {
            $script:attempts++
            $global:LASTEXITCODE = 2
            '{"error":{"code":"admission_checkpoint_stale","details":{"operation":"cargo.acquire@owner"}}}'
        } -Parse { param($raw) ($raw -join "`n") | ConvertFrom-Json }
        ($raw | ConvertFrom-Json).error.code | Should Be 'admission_checkpoint_stale'
        $LASTEXITCODE | Should Be 2
        $script:attempts | Should Be 3
    }

    It 'does not resubmit accepted mutations or another operation stale failure' {
        foreach ($document in @(
            '{"error":{"code":"command_post_timeout","details":{"requestId":"0123456789abcdef0123456789abcdef"}}}',
            '{"error":{"code":"admission_checkpoint_stale","details":{"operation":"cargo.start"}}}',
            '{"request":{"status":"failed"},"error":{"code":"request_overloaded"}}'
        )) {
            $script:attempts = 0
            $raw = Invoke-CoordinatorUnacceptedRequest -CargoAcquire -Invoke {
                $script:attempts++
                $global:LASTEXITCODE = 2
                $document
            } -Parse { param($raw) ($raw -join "`n") | ConvertFrom-Json }
            $raw | Should Be $document
            $script:attempts | Should Be 1
        }
    }

    It 'keeps stale admission failures final outside cargo acquire' {
        $raw = Invoke-CoordinatorUnacceptedRequest -Invoke {
            $script:attempts++
            $global:LASTEXITCODE = 2
            '{"error":{"code":"admission_checkpoint_stale","details":{"operation":"cargo.acquire@owner"}}}'
        } -Parse { param($raw) ($raw -join "`n") | ConvertFrom-Json }
        $script:attempts | Should Be 1
    }
}

Describe 'Accepted coordinator request recovery' {
    BeforeEach {
        $script:requestId = '0123456789abcdef0123456789abcdef'
        $script:failure = [pscustomobject]@{
            error = [pscustomobject]@{
                code = 'command_post_timeout'
                details = [pscustomobject]@{ requestId = $script:requestId; phase = 'post_response' }
            }
        }
        Mock Start-Sleep {}
    }

    It 'recovers the original accepted request after a late pool allocation' {
        $script:queries = 0
        $query = {
            param($requestId)
            if ($requestId -cne $script:requestId) { throw 'wrong request identity' }
            $script:queries++
            if ($script:queries -eq 1) {
                return [pscustomobject]@{ request = [pscustomobject]@{ requestId = $requestId; status = 'accepted' } }
            }
            return [pscustomobject]@{
                request = [pscustomobject]@{ requestId = $requestId; status = 'completed' }
                result = [pscustomobject]@{ job = [pscustomobject]@{ job_id = 'original-job' } }
            }
        }
        $result = Wait-CoordinatorAcceptedRequest -Failure $script:failure -Query $query
        $result.job.job_id | Should Be 'original-job'
        $script:queries | Should Be 2
    }

    It 'keeps an original terminal failure instead of submitting a replacement' {
        $query = {
            param($requestId)
            [pscustomobject]@{
                request = [pscustomobject]@{ requestId = $requestId; status = 'failed' }
                error = [pscustomobject]@{ code = 'cargo_reuse_pool_busy'; message = 'Pool is leased' }
            }
        }
        { Wait-CoordinatorAcceptedRequest -Failure $script:failure -Query $query } | Should Throw 'cargo_reuse_pool_busy'
    }

    It 'retries an overloaded status query without losing the accepted request' {
        $script:queries = 0
        $result = Wait-CoordinatorAcceptedRequest -Failure $script:failure -Query {
            param($requestId)
            if ($requestId -cne $script:requestId) { throw 'wrong request identity' }
            $script:queries++
            if ($script:queries -eq 1) {
                return [pscustomobject]@{ error = [pscustomobject]@{ code = 'request_overloaded' } }
            }
            [pscustomobject]@{
                request = [pscustomobject]@{ requestId = $requestId; status = 'completed' }
                result = [pscustomobject]@{ job = 'original-job' }
            }
        }
        $result.job | Should Be 'original-job'
        $script:queries | Should Be 2
    }

    It 'keeps an explicit timeout when status queries remain overloaded' {
        $query = { [pscustomobject]@{ error = [pscustomobject]@{ code = 'request_overloaded' } } }
        { Wait-CoordinatorAcceptedRequest -Failure $script:failure -Query $query -TimeoutSeconds 0.01 } |
            Should Throw $script:requestId
    }

    It 'does not retry a terminal request failure with an overload code' {
        $query = {
            param($requestId)
            [pscustomobject]@{
                request = [pscustomobject]@{ requestId = $requestId; status = 'failed' }
                error = [pscustomobject]@{ code = 'request_overloaded'; message = 'terminal failure' }
            }
        }
        { Wait-CoordinatorAcceptedRequest -Failure $script:failure -Query $query } | Should Throw 'terminal failure'
    }

    It 'does not query preflight failures or malformed request identities' {
        $query = { throw 'unexpected query' }
        $script:failure.error.code = 'command_preflight_timeout'
        Wait-CoordinatorAcceptedRequest -Failure $script:failure -Query $query | Should BeNullOrEmpty
        $script:failure.error.code = 'command_post_timeout'
        $script:failure.error.details.requestId = '../invalid'
        Wait-CoordinatorAcceptedRequest -Failure $script:failure -Query $query | Should BeNullOrEmpty
    }

    It 'rejects a result for another request' {
        $query = {
            [pscustomobject]@{
                request = [pscustomobject]@{ requestId = 'different'; status = 'completed' }
                result = [pscustomobject]@{ job = 'wrong' }
            }
        }
        { Wait-CoordinatorAcceptedRequest -Failure $script:failure -Query $query } | Should Throw 'identity'
    }

    It 'retains the request identity when bounded reconciliation expires' {
        $query = {
            param($requestId)
            [pscustomobject]@{ request = [pscustomobject]@{ requestId = $requestId; status = 'accepted' } }
        }
        { Wait-CoordinatorAcceptedRequest -Failure $script:failure -Query $query -TimeoutSeconds 0.01 } | Should Throw $script:requestId
    }

    It 'allows unbounded recovery to consume a late result without resubmitting' {
        $script:queries = 0
        $query = {
            param($requestId)
            $script:queries++
            if ($script:queries -lt 3) {
                return [pscustomobject]@{
                    request = [pscustomobject]@{ requestId = $requestId; status = 'accepted' }
                }
            }
            [pscustomobject]@{
                request = [pscustomobject]@{ requestId = $requestId; status = 'completed' }
                result = [pscustomobject]@{ job = [pscustomobject]@{ job_id = 'late-original-job' } }
            }
        }
        $result = Wait-CoordinatorAcceptedRequest -Failure $script:failure -Query $query -TimeoutSeconds 0
        $result.job.job_id | Should Be 'late-original-job'
        $script:queries | Should Be 3
    }
}

Describe 'Offline coordinator request recovery' {
    BeforeEach {
        $script:queued = [pscustomobject]@{
            status = 'queued'
            queueId = '0123456789abcdef0123456789abcdef'
            command = 'session.register'
        }
        Mock Start-Sleep {}
    }

    It 'waits for offline replay before resuming an idempotent registration' {
        $script:queries = 0
        $script:resumed = 0
        $result = Wait-CoordinatorQueuedRequest -Response $script:queued -Query {
            $script:queries++
            if ($script:queries -eq 1) { return [pscustomobject]@{ status = 'offline' } }
            [pscustomobject]@{ status = 'ok'; offlineReplay = [pscustomobject]@{ retained = 0 } }
        } -Resume {
            $script:resumed++
            [pscustomobject]@{ session = [pscustomobject]@{ session_id = 'original-session' } }
        }
        $result.session.session_id | Should Be 'original-session'
        $script:queries | Should Be 2
        $script:resumed | Should Be 1
    }

    It 'does not resubmit resource allocation or malformed queue receipts' {
        $query = { throw 'unexpected query' }
        $resume = { throw 'unexpected resume' }
        $script:queued.command = 'cargo.acquire'
        { Wait-CoordinatorQueuedRequest -Response $script:queued -Query $query -Resume $resume } | Should Throw 'replay-safe'
        $script:queued.command = 'session.register'
        $script:queued.queueId = '../invalid'
        { Wait-CoordinatorQueuedRequest -Response $script:queued -Query $query -Resume $resume } | Should Throw 'queue identity'
    }

    It 'preserves the queue identity if the service remains offline' {
        $query = { [pscustomobject]@{ status = 'offline' } }
        $resume = { throw 'unexpected resume' }
        { Wait-CoordinatorQueuedRequest -Response $script:queued -Query $query -Resume $resume -TimeoutSeconds 0.01 } | Should Throw $script:queued.queueId
    }

    It 'waits while replay is retained and rejects unhealthy service responses' {
        $script:queries = 0
        $result = Wait-CoordinatorQueuedRequest -Response $script:queued -Query {
            $script:queries++
            [pscustomobject]@{ status = 'ok'; offlineReplay = [pscustomobject]@{ retained = $(if ($script:queries -eq 1) { 1 } else { 0 }) } }
        } -Resume { [pscustomobject]@{ resumed = $true } }
        $result.resumed | Should Be $true
        $script:queries | Should Be 2
        { Wait-CoordinatorQueuedRequest -Response $script:queued -Query { [pscustomobject]@{ status = 'error' } } -Resume { throw 'unexpected resume' } } | Should Throw 'unhealthy'
    }

    It 'does not change a normal completed response' {
        Wait-CoordinatorQueuedRequest -Response ([pscustomobject]@{ session = 'completed' }) -Query { throw 'unexpected query' } -Resume { throw 'unexpected resume' } | Should BeNullOrEmpty
    }
}
