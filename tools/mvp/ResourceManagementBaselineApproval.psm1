Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Import-Module (Join-Path $PSScriptRoot 'ResourceManagementJsonEvidence.psm1') -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'ResourceManagementSchema.psm1') -ErrorAction Stop
Import-Module (Join-Path $PSScriptRoot 'ResourceManagementSchemaRegistry.psm1') -ErrorAction Stop

$script:ResourceManagementApprovalTrustRegistrySnapshot = $null

function ConvertTo-ResourceManagementApprovalUtc {
    param(
        [Parameter(Mandatory)]$Value,
        [Parameter(Mandatory)][string]$Label
    )

    $text = if ($Value -is [DateTimeOffset]) {
        ([DateTimeOffset]$Value).ToUniversalTime().ToString('O', [Globalization.CultureInfo]::InvariantCulture)
    }
    elseif ($Value -is [DateTime]) {
        ([DateTime]$Value).ToUniversalTime().ToString('O', [Globalization.CultureInfo]::InvariantCulture)
    }
    else {
        [string]$Value
    }
    $parsed = [DateTimeOffset]::MinValue
    if (-not [DateTimeOffset]::TryParse(
            $text,
            [Globalization.CultureInfo]::InvariantCulture,
            [Globalization.DateTimeStyles]::RoundtripKind,
            [ref]$parsed
        ) -or $parsed.Offset -ne [TimeSpan]::Zero) {
        throw "$Label must be an ISO-8601 UTC timestamp."
    }
    return [pscustomobject]@{
        text = $parsed.ToUniversalTime().ToString('O', [Globalization.CultureInfo]::InvariantCulture)
        value = $parsed.ToUniversalTime()
    }
}

function Assert-ResourceManagementApprovalIdentifier {
    param(
        [Parameter(Mandatory)][string]$Value,
        [Parameter(Mandatory)][string]$Label
    )

    if ($Value -notmatch '^[a-z0-9][a-z0-9._-]{0,127}$') {
        throw "$Label must be one stable lowercase identifier."
    }
    return $Value
}

function Resolve-ResourceManagementApprovalTrustRegistry {
    param(
        [Parameter(Mandatory)]$Registry,
        [Parameter(Mandatory)][ValidateRange(1, 65536)][int]$RegistryBytes,
        [Parameter(Mandatory)][string]$RegistrySha256
    )

    Assert-ResourceManagementSchemaSha256 `
        -Value $RegistrySha256 `
        -Label 'Resource-management approval trust registry SHA-256' | Out-Null
    Assert-ResourceManagementSchemaProperties `
        -Value $Registry `
        -RequiredNames @('schema_version', 'registry_kind', 'issuers', 'revoked_receipt_sha256') `
        -Label 'Resource-management approval trust registry'
    Assert-ResourceManagementRegisteredSchemaIdentity `
        -Value $Registry `
        -SchemaId 'zircon.resource-management.approval-trust-registry' `
        -Label 'Resource-management approval trust registry' | Out-Null
    $issuerIndex = @{}
    foreach ($issuer in @($Registry.issuers)) {
        Assert-ResourceManagementSchemaProperties `
            -Value $issuer `
            -RequiredNames @(
                'issuer_id', 'key_id', 'signature_algorithm', 'public_key_spki_base64',
                'not_before_utc', 'not_after_utc', 'status') `
            -Label 'Resource-management approval issuer'
        $issuerId = Assert-ResourceManagementApprovalIdentifier `
            -Value ([string]$issuer.issuer_id) `
            -Label 'Resource-management approval issuer_id'
        $keyId = Assert-ResourceManagementApprovalIdentifier `
            -Value ([string]$issuer.key_id) `
            -Label 'Resource-management approval key_id'
        if ([string]$issuer.signature_algorithm -cne 'rsa-pss-sha256') {
            throw "Resource-management approval issuer '$issuerId/$keyId' has an unsupported signature_algorithm."
        }
        try {
            [byte[]]$publicKeyBytes = [Convert]::FromBase64String([string]$issuer.public_key_spki_base64)
        }
        catch {
            throw "Resource-management approval issuer '$issuerId/$keyId' has invalid public_key_spki_base64."
        }
        $rsa = [Security.Cryptography.RSA]::Create()
        try {
            $bytesRead = 0
            $rsa.ImportSubjectPublicKeyInfo($publicKeyBytes, [ref]$bytesRead)
            if ($bytesRead -ne $publicKeyBytes.Length -or $rsa.KeySize -lt 2048) {
                throw 'The approval public key must be one complete RSA key of at least 2048 bits.'
            }
        }
        catch {
            throw "Resource-management approval issuer '$issuerId/$keyId' has an invalid RSA public key: $($_.Exception.Message)"
        }
        finally {
            $rsa.Dispose()
        }
        $notBefore = ConvertTo-ResourceManagementApprovalUtc `
            -Value $issuer.not_before_utc `
            -Label "Resource-management approval issuer '$issuerId/$keyId' not_before_utc"
        $notAfter = ConvertTo-ResourceManagementApprovalUtc `
            -Value $issuer.not_after_utc `
            -Label "Resource-management approval issuer '$issuerId/$keyId' not_after_utc"
        if ($notAfter.value -le $notBefore.value) {
            throw "Resource-management approval issuer '$issuerId/$keyId' has an invalid validity interval."
        }
        $status = [string]$issuer.status
        if ($status -cnotin @('active', 'disabled')) {
            throw "Resource-management approval issuer '$issuerId/$keyId' has unsupported status '$status'."
        }
        $indexKey = "$issuerId$([char]0)$keyId"
        if ($issuerIndex.ContainsKey($indexKey)) {
            throw "Resource-management approval trust registry has duplicate issuer key '$issuerId/$keyId'."
        }
        $issuerIndex[$indexKey] = [pscustomobject][ordered]@{
            issuer_id = $issuerId
            key_id = $keyId
            signature_algorithm = 'rsa-pss-sha256'
            public_key_spki_base64 = [Convert]::ToBase64String($publicKeyBytes)
            not_before_utc = $notBefore.text
            not_before = $notBefore.value
            not_after_utc = $notAfter.text
            not_after = $notAfter.value
            status = $status
        }
    }
    $revoked = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($entry in @($Registry.revoked_receipt_sha256)) {
        $sha256 = Assert-ResourceManagementSchemaSha256 `
            -Value ([string]$entry) `
            -Label 'Resource-management revoked approval receipt SHA-256'
        if (-not $revoked.Add($sha256)) {
            throw "Resource-management approval trust registry has duplicate revoked receipt '$sha256'."
        }
    }
    return [pscustomobject][ordered]@{
        receipt = [pscustomobject][ordered]@{
            schema_version = 1
            registry_kind = 'zircon.resource-management-approval-trust-registry'
            issuer_count = $issuerIndex.Count
            revoked_receipt_count = $revoked.Count
            bytes = $RegistryBytes
            sha256 = $RegistrySha256
        }
        issuers = $issuerIndex
        revoked_receipts = $revoked
    }
}

function Get-ResourceManagementApprovalTrustRegistrySnapshot {
    if ($null -ne $script:ResourceManagementApprovalTrustRegistrySnapshot) {
        return $script:ResourceManagementApprovalTrustRegistrySnapshot
    }
    $evidence = Get-ResourceManagementJsonEvidence `
        -Path (Join-Path $PSScriptRoot 'resource-management-approval-trust-registry.json') `
        -Label 'Resource-management approval trust registry' `
        -MaximumBytes 65536
    $script:ResourceManagementApprovalTrustRegistrySnapshot = Resolve-ResourceManagementApprovalTrustRegistry `
        -Registry $evidence.json `
        -RegistryBytes $evidence.bytes `
        -RegistrySha256 $evidence.sha256
    return $script:ResourceManagementApprovalTrustRegistrySnapshot
}

function Resolve-ResourceManagementApprovalReceiptFields {
    param([Parameter(Mandatory)]$Receipt)

    Assert-ResourceManagementSchemaProperties `
        -Value $Receipt `
        -RequiredNames @(
            'schema_version', 'receipt_kind', 'promotion_id', 'evidence_set_id', 'review_id',
            'issuer_id', 'key_id', 'issued_utc',
            'expires_utc', 'approved_baseline_report_sha256', 'workload_profile_id',
            'retention_class', 'retention_until_utc', 'legal_security_scrub_receipt_sha256',
            'supersedes_promotion_receipt_sha256', 'decision', 'signature_algorithm',
            'signature_base64') `
        -Label 'Resource-management approved baseline receipt'
    Assert-ResourceManagementRegisteredSchemaIdentity `
        -Value $Receipt `
        -SchemaId 'zircon.resource-management.approval-receipt' `
        -Label 'Resource-management approved baseline receipt' | Out-Null
    $promotionId = Assert-ResourceManagementApprovalIdentifier `
        -Value ([string]$Receipt.promotion_id) `
        -Label 'Resource-management approved baseline receipt promotion_id'
    $evidenceSetId = Assert-ResourceManagementSchemaSha256 `
        -Value ([string]$Receipt.evidence_set_id) `
        -Label 'Resource-management approved baseline receipt evidence_set_id'
    $reviewId = Assert-ResourceManagementApprovalIdentifier `
        -Value ([string]$Receipt.review_id) `
        -Label 'Resource-management approved baseline receipt review_id'
    $issuerId = Assert-ResourceManagementApprovalIdentifier `
        -Value ([string]$Receipt.issuer_id) `
        -Label 'Resource-management approved baseline receipt issuer_id'
    $keyId = Assert-ResourceManagementApprovalIdentifier `
        -Value ([string]$Receipt.key_id) `
        -Label 'Resource-management approved baseline receipt key_id'
    $issued = ConvertTo-ResourceManagementApprovalUtc `
        -Value $Receipt.issued_utc `
        -Label 'Resource-management approved baseline receipt issued_utc'
    $expires = ConvertTo-ResourceManagementApprovalUtc `
        -Value $Receipt.expires_utc `
        -Label 'Resource-management approved baseline receipt expires_utc'
    if ($expires.value -le $issued.value) {
        throw 'Resource-management approved baseline receipt has an invalid validity interval.'
    }
    $approvedSha256 = Assert-ResourceManagementSchemaSha256 `
        -Value ([string]$Receipt.approved_baseline_report_sha256) `
        -Label 'Resource-management approved baseline receipt approved_baseline_report_sha256'
    $workloadProfileId = Assert-ResourceManagementApprovalIdentifier `
        -Value ([string]$Receipt.workload_profile_id) `
        -Label 'Resource-management approved baseline receipt workload_profile_id'
    if ([string]$Receipt.retention_class -cne 'accepted-baseline') {
        throw "Resource-management approved baseline receipt has unsupported retention_class '$($Receipt.retention_class)'."
    }
    $retentionUntil = ConvertTo-ResourceManagementApprovalUtc `
        -Value $Receipt.retention_until_utc `
        -Label 'Resource-management approved baseline receipt retention_until_utc'
    if ($retentionUntil.value -lt $expires.value) {
        throw 'Resource-management approved baseline receipt retention cannot end before receipt expiry.'
    }
    $scrubReceiptSha256 = Assert-ResourceManagementSchemaSha256 `
        -Value ([string]$Receipt.legal_security_scrub_receipt_sha256) `
        -Label 'Resource-management approved baseline receipt legal_security_scrub_receipt_sha256'
    $supersedesReceiptSha256 = $null
    if ($null -ne $Receipt.supersedes_promotion_receipt_sha256) {
        $supersedesReceiptSha256 = Assert-ResourceManagementSchemaSha256 `
            -Value ([string]$Receipt.supersedes_promotion_receipt_sha256) `
            -Label 'Resource-management approved baseline receipt supersedes_promotion_receipt_sha256'
    }
    if ([string]$Receipt.decision -cne 'approved' -or
        [string]$Receipt.signature_algorithm -cne 'rsa-pss-sha256') {
        throw 'Resource-management approved baseline receipt has an unsupported decision or signature algorithm.'
    }
    try {
        [byte[]]$signatureBytes = [Convert]::FromBase64String([string]$Receipt.signature_base64)
    }
    catch {
        throw 'Resource-management approved baseline receipt signature_base64 is invalid.'
    }
    if ($signatureBytes.Length -eq 0 -or $signatureBytes.Length -gt 16384) {
        throw 'Resource-management approved baseline receipt signature size is invalid.'
    }
    return [pscustomobject][ordered]@{
        promotion_id = $promotionId
        evidence_set_id = $evidenceSetId
        review_id = $reviewId
        issuer_id = $issuerId
        key_id = $keyId
        issued_utc = $issued.text
        issued = $issued.value
        expires_utc = $expires.text
        expires = $expires.value
        approved_baseline_report_sha256 = $approvedSha256
        workload_profile_id = $workloadProfileId
        retention_class = 'accepted-baseline'
        retention_until_utc = $retentionUntil.text
        legal_security_scrub_receipt_sha256 = $scrubReceiptSha256
        supersedes_promotion_receipt_sha256 = $supersedesReceiptSha256
        signature_bytes = $signatureBytes
    }
}

function Get-ResourceManagementApprovalCanonicalPayloadBytes {
    param([Parameter(Mandatory)]$Receipt)

    $fields = Resolve-ResourceManagementApprovalReceiptFields -Receipt $Receipt
    $payload = [ordered]@{
        schema_version = 2
        receipt_kind = 'zircon.resource-management-baseline-approval'
        promotion_id = $fields.promotion_id
        evidence_set_id = $fields.evidence_set_id
        review_id = $fields.review_id
        issuer_id = $fields.issuer_id
        key_id = $fields.key_id
        issued_utc = $fields.issued_utc
        expires_utc = $fields.expires_utc
        approved_baseline_report_sha256 = $fields.approved_baseline_report_sha256
        workload_profile_id = $fields.workload_profile_id
        retention_class = $fields.retention_class
        retention_until_utc = $fields.retention_until_utc
        legal_security_scrub_receipt_sha256 = $fields.legal_security_scrub_receipt_sha256
        supersedes_promotion_receipt_sha256 = $fields.supersedes_promotion_receipt_sha256
        decision = 'approved'
        signature_algorithm = 'rsa-pss-sha256'
    }
    return [Text.UTF8Encoding]::new($false, $true).GetBytes(($payload | ConvertTo-Json -Depth 3 -Compress))
}

function New-ResourceManagementApprovalVerification {
    param(
        [Parameter(Mandatory)][ValidateSet('verified', 'unverified')][string]$Status,
        [Parameter(Mandatory)][string]$Reason,
        [Parameter(Mandatory)][string]$ReceiptSha256,
        [Parameter(Mandatory)]$Fields,
        [Parameter(Mandatory)]$TrustRegistrySnapshot,
        [Parameter(Mandatory)][DateTimeOffset]$VerificationTimeUtc
    )

    return [pscustomobject][ordered]@{
        schema_version = 2
        verification_kind = 'zircon.resource-management-baseline-approval-verification'
        verification_status = $Status
        verification_reason = $Reason
        approval_receipt_sha256 = $ReceiptSha256
        issuer_id = $Fields.issuer_id
        key_id = $Fields.key_id
        promotion_id = $Fields.promotion_id
        evidence_set_id = $Fields.evidence_set_id
        review_id = $Fields.review_id
        retention_class = $Fields.retention_class
        retention_until_utc = $Fields.retention_until_utc
        legal_security_scrub_receipt_sha256 = $Fields.legal_security_scrub_receipt_sha256
        supersedes_promotion_receipt_sha256 = $Fields.supersedes_promotion_receipt_sha256
        trust_registry_sha256 = $TrustRegistrySnapshot.receipt.sha256
        verified_utc = $VerificationTimeUtc.ToUniversalTime().ToString('O', [Globalization.CultureInfo]::InvariantCulture)
    }
}

function Resolve-ResourceManagementBaselineApproval {
    param(
        [Parameter(Mandatory)]$Receipt,
        [Parameter(Mandatory)][string]$ReceiptSha256,
        [Parameter(Mandatory)][string]$ApprovedBaselineReportSha256,
        [Parameter(Mandatory)][string]$WorkloadProfileId,
        [Parameter(Mandatory)]$TrustRegistrySnapshot,
        [Parameter(Mandatory)][DateTimeOffset]$VerificationTimeUtc
    )

    [void](Assert-ResourceManagementSchemaSha256 `
            -Value $ReceiptSha256 `
            -Label 'Resource-management approved baseline receipt SHA-256')
    [void](Assert-ResourceManagementSchemaSha256 `
            -Value $ApprovedBaselineReportSha256 `
            -Label 'Resource-management approved baseline report SHA-256')
    $fields = Resolve-ResourceManagementApprovalReceiptFields -Receipt $Receipt
    if ($null -ne $fields.supersedes_promotion_receipt_sha256 -and
        $fields.supersedes_promotion_receipt_sha256 -ceq $ReceiptSha256) {
        throw 'Resource-management approved baseline receipt cannot supersede itself.'
    }
    if ($fields.approved_baseline_report_sha256 -cne $ApprovedBaselineReportSha256) {
        throw 'Resource-management approved baseline receipt binds a different report SHA-256.'
    }
    if ($fields.workload_profile_id -cne $WorkloadProfileId) {
        throw 'Resource-management approved baseline receipt binds a different workload profile.'
    }
    if ($TrustRegistrySnapshot.revoked_receipts.Contains($ReceiptSha256)) {
        return New-ResourceManagementApprovalVerification `
            -Status 'unverified' -Reason 'approval-receipt-revoked' `
            -ReceiptSha256 $ReceiptSha256 -Fields $fields `
            -TrustRegistrySnapshot $TrustRegistrySnapshot -VerificationTimeUtc $VerificationTimeUtc
    }
    $issuerKey = "$($fields.issuer_id)$([char]0)$($fields.key_id)"
    if (-not $TrustRegistrySnapshot.issuers.ContainsKey($issuerKey)) {
        return New-ResourceManagementApprovalVerification `
            -Status 'unverified' -Reason 'approval-issuer-not-trusted' `
            -ReceiptSha256 $ReceiptSha256 -Fields $fields `
            -TrustRegistrySnapshot $TrustRegistrySnapshot -VerificationTimeUtc $VerificationTimeUtc
    }
    $issuer = $TrustRegistrySnapshot.issuers[$issuerKey]
    if ($issuer.status -ne 'active') {
        return New-ResourceManagementApprovalVerification `
            -Status 'unverified' -Reason 'approval-issuer-disabled' `
            -ReceiptSha256 $ReceiptSha256 -Fields $fields `
            -TrustRegistrySnapshot $TrustRegistrySnapshot -VerificationTimeUtc $VerificationTimeUtc
    }
    $verificationTime = $VerificationTimeUtc.ToUniversalTime()
    if ($verificationTime -lt $fields.issued -or $verificationTime -lt $issuer.not_before) {
        return New-ResourceManagementApprovalVerification `
            -Status 'unverified' -Reason 'approval-not-yet-valid' `
            -ReceiptSha256 $ReceiptSha256 -Fields $fields `
            -TrustRegistrySnapshot $TrustRegistrySnapshot -VerificationTimeUtc $VerificationTimeUtc
    }
    if ($verificationTime -gt $fields.expires -or $verificationTime -gt $issuer.not_after) {
        return New-ResourceManagementApprovalVerification `
            -Status 'unverified' -Reason 'approval-receipt-expired' `
            -ReceiptSha256 $ReceiptSha256 -Fields $fields `
            -TrustRegistrySnapshot $TrustRegistrySnapshot -VerificationTimeUtc $VerificationTimeUtc
    }
    [byte[]]$publicKeyBytes = [Convert]::FromBase64String($issuer.public_key_spki_base64)
    $rsa = [Security.Cryptography.RSA]::Create()
    try {
        $bytesRead = 0
        $rsa.ImportSubjectPublicKeyInfo($publicKeyBytes, [ref]$bytesRead)
        $verified = $rsa.VerifyData(
            (Get-ResourceManagementApprovalCanonicalPayloadBytes -Receipt $Receipt),
            $fields.signature_bytes,
            [Security.Cryptography.HashAlgorithmName]::SHA256,
            [Security.Cryptography.RSASignaturePadding]::Pss)
    }
    finally {
        $rsa.Dispose()
    }
    if (-not $verified) {
        throw 'Resource-management approved baseline receipt signature verification failed.'
    }
    return New-ResourceManagementApprovalVerification `
        -Status 'verified' -Reason 'signature-and-policy-verified' `
        -ReceiptSha256 $ReceiptSha256 -Fields $fields `
        -TrustRegistrySnapshot $TrustRegistrySnapshot -VerificationTimeUtc $VerificationTimeUtc
}

Export-ModuleMember -Function @(
    'Get-ResourceManagementApprovalCanonicalPayloadBytes',
    'Get-ResourceManagementApprovalTrustRegistrySnapshot',
    'Resolve-ResourceManagementApprovalTrustRegistry',
    'Resolve-ResourceManagementBaselineApproval'
)
