[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0)]
    [ValidateSet('ValidateArtifacts', 'Configure', 'ImportRealm', 'StartKeycloak', 'StartService', 'Readiness', 'Backup', 'Restore')]
    [string]$Action,
    [string]$KeycloakHome,
    [string]$JavaHome,
    [string]$StateRoot,
    [string]$IntrospectionSecretFile,
    [string]$CloudKeyFile,
    [string]$HubServiceExecutable,
    [string]$BackupPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$artifactRoot = Split-Path -Parent $PSCommandPath
$deploymentPath = Join-Path $artifactRoot 'deployment.json'
$realmPath = Join-Path $artifactRoot 'zircon-local-realm.json'
$deployment = Get-Content -Raw -Encoding UTF8 -LiteralPath $deploymentPath | ConvertFrom-Json

function Require-Value([string]$Name, [string]$Value) {
    if ([string]::IsNullOrWhiteSpace($Value)) {
        throw "$Name is required for action $Action."
    }
}

function Resolve-AbsolutePath([string]$Name, [string]$Value) {
    Require-Value $Name $Value
    if (-not [IO.Path]::IsPathFullyQualified($Value)) {
        throw "$Name must be an absolute path."
    }
    return [IO.Path]::GetFullPath($Value)
}

function Read-SecretFile([string]$Path) {
    $resolved = Resolve-AbsolutePath 'IntrospectionSecretFile' $Path
    if (-not (Test-Path -LiteralPath $resolved -PathType Leaf)) {
        throw 'The introspection secret file does not exist.'
    }
    $secret = [IO.File]::ReadAllText($resolved, [Text.Encoding]::UTF8)
    if ($secret -notmatch '^[A-Za-z0-9_-]{32,4096}$') {
        throw 'The introspection secret must contain 32-4096 base64url characters without padding.'
    }
    return $secret
}

function Read-CloudKeyFile([string]$Path) {
    $resolved = Resolve-AbsolutePath 'CloudKeyFile' $Path
    if (-not (Test-Path -LiteralPath $resolved -PathType Leaf)) {
        throw 'The cloud key file does not exist.'
    }
    $key = [IO.File]::ReadAllBytes($resolved)
    if ($key.Length -ne 32) {
        throw 'The cloud key file must contain exactly 32 raw bytes.'
    }
    return $key
}

function Write-Utf8([string]$Path, [string]$Value) {
    $parent = Split-Path -Parent $Path
    [IO.Directory]::CreateDirectory($parent) | Out-Null
    [IO.File]::WriteAllText($Path, $Value, [Text.UTF8Encoding]::new($false))
}

function New-PrivateDirectory([string]$Path) {
    $userSid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
    $trusted = @($userSid, 'S-1-5-18', 'S-1-5-32-544')
    $ancestor = [IO.DirectoryInfo]::new($Path)
    while ($null -ne $ancestor) {
        if ($ancestor.Exists -and ($ancestor.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
            throw 'State and backup directories must not contain reparse points in their path.'
        }
        $ancestor = $ancestor.Parent
    }
    if (-not (Test-Path -LiteralPath $Path)) {
        $security = [Security.AccessControl.DirectorySecurity]::new()
        $security.SetSecurityDescriptorSddlForm("O:${userSid}D:P(A;OICI;FA;;;${userSid})(A;OICI;FA;;;SY)")
        [IO.FileSystemAclExtensions]::Create([IO.DirectoryInfo]::new($Path), $security)
    }
    $current = [IO.DirectoryInfo]::new($Path)
    while ($null -ne $current) {
        if (-not $current.Exists -or ($current.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
            throw 'State and backup directories must be local directories without reparse points.'
        }
        $acl = Get-Acl -LiteralPath $current.FullName
        if ($acl.GetOwner([Security.Principal.SecurityIdentifier]).Value -notin $trusted) {
            throw 'State and backup directories must be owned by the service user, SYSTEM, or Administrators.'
        }
        foreach ($rule in $acl.GetAccessRules($true, $true, [Security.Principal.SecurityIdentifier])) {
            if ($rule.AccessControlType -eq [Security.AccessControl.AccessControlType]::Allow -and
                $rule.IdentityReference.Value -notin $trusted) {
                throw 'State and backup directories must grant access only to the service user, SYSTEM, or Administrators. Choose a new private directory.'
            }
        }
        if ($acl.AreAccessRulesProtected) { return }
        $current = $current.Parent
    }
    throw 'State and backup directories need a protected private ACL in their inheritance chain.'
}

function Assert-TomlLiteralSafe([string]$Name, [string]$Value) {
    if ($Value.Contains("'") -or $Value.Contains("`r") -or $Value.Contains("`n")) {
        throw "$Name cannot contain a single quote or newline."
    }
}

function Write-HubConfigs([string]$Root, [string]$SecretFile, [string]$CloudKey) {
    $resolvedRoot = Resolve-AbsolutePath 'StateRoot' $Root
    $resolvedSecret = Resolve-AbsolutePath 'IntrospectionSecretFile' $SecretFile
    $resolvedCloudKey = Resolve-AbsolutePath 'CloudKeyFile' $CloudKey
    Read-SecretFile $resolvedSecret | Out-Null
    Read-CloudKeyFile $resolvedCloudKey | Out-Null
    $rootPrefix = $resolvedRoot.TrimEnd([IO.Path]::DirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
    if ($resolvedSecret.StartsWith($rootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'IntrospectionSecretFile must be outside StateRoot so data backups cannot capture it.'
    }
    if ($resolvedCloudKey.StartsWith($rootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'CloudKeyFile must be outside StateRoot so data backups cannot capture it.'
    }
    $database = Join-Path $resolvedRoot 'data\zircon-hub.sqlite3'
    $cloudRoot = Join-Path $resolvedRoot 'data\cloud'
    Assert-TomlLiteralSafe 'StateRoot' $database
    Assert-TomlLiteralSafe 'IntrospectionSecretFile' $resolvedSecret
    Assert-TomlLiteralSafe 'CloudKeyFile' $resolvedCloudKey
    New-PrivateDirectory $resolvedRoot
    New-PrivateDirectory (Join-Path $resolvedRoot 'data')
    $serviceConfig = @"
bind = "$($deployment.hubService.bind)"
database = '$database'
issuer = "$($deployment.keycloak.issuer)"
audience = "$($deployment.clients.service)"
introspection_client_id = "$($deployment.clients.service)"
introspection_secret_file = '$resolvedSecret'
allow_loopback_http = true

[cloud]
root = '$cloudRoot'
key_file = '$resolvedCloudKey'
"@
    Write-Utf8 (Join-Path $resolvedRoot 'config\service.toml') ($serviceConfig.Trim() + "`n")
    $accountConfig = [ordered]@{
        issuer = [string]$deployment.keycloak.issuer
        client_id = [string]$deployment.clients.desktop
        service_url = [string]$deployment.hubService.url
        callback_port = [int]$deployment.clients.callbackPort
        allow_loopback_http = $true
    } | ConvertTo-Json
    Write-Utf8 (Join-Path $resolvedRoot 'config\account.json') ($accountConfig + "`n")
    $state = [ordered]@{
        schemaVersion = 1
        keycloakVersion = [string]$deployment.keycloak.version
        stateRoot = $resolvedRoot
        introspectionSecretFile = $resolvedSecret
        cloudKeyFile = $resolvedCloudKey
    } | ConvertTo-Json
    Write-Utf8 (Join-Path $resolvedRoot 'deployment-state.json') ($state + "`n")
    Write-Output "Wrote Hub configuration under $resolvedRoot."
}

function Resolve-Keycloak([string]$InstallHome, [string]$JdkHome) {
    $resolvedHome = Resolve-AbsolutePath 'KeycloakHome' $InstallHome
    $resolvedJava = Resolve-AbsolutePath 'JavaHome' $JdkHome
    $kc = Join-Path $resolvedHome 'bin\kc.bat'
    $java = Join-Path $resolvedJava 'bin\java.exe'
    if (-not (Test-Path -LiteralPath $kc -PathType Leaf)) {
        throw 'KeycloakHome does not contain bin\kc.bat.'
    }
    if (-not (Test-Path -LiteralPath $java -PathType Leaf)) {
        throw 'JavaHome does not contain bin\java.exe.'
    }
    return [pscustomobject]@{ Home = $resolvedHome; JavaHome = $resolvedJava; Kc = $kc }
}

function Invoke-Keycloak([pscustomobject]$Runtime, [string[]]$Arguments) {
    $oldJavaHome = $env:JAVA_HOME
    $oldPath = $env:Path
    try {
        $env:JAVA_HOME = $Runtime.JavaHome
        $env:Path = (Join-Path $Runtime.JavaHome 'bin') + [IO.Path]::PathSeparator + $oldPath
        & $Runtime.Kc @Arguments
        if ($LASTEXITCODE -ne 0) {
            throw "Keycloak exited with code $LASTEXITCODE."
        }
    }
    finally {
        $env:JAVA_HOME = $oldJavaHome
        $env:Path = $oldPath
    }
}

function Invoke-JsonEndpoint([string]$Uri) {
    $response = Invoke-WebRequest -UseBasicParsing -TimeoutSec 5 -Uri $Uri
    if ($response.StatusCode -ne 200) {
        throw "$Uri returned HTTP $($response.StatusCode)."
    }
    return $response.Content | ConvertFrom-Json
}

function Test-PortOpen([int]$Port) {
    $client = [Net.Sockets.TcpClient]::new()
    try {
        $task = $client.ConnectAsync([string]$deployment.keycloak.httpHost, $Port)
        return $task.Wait(300) -and $client.Connected
    }
    catch {
        return $false
    }
    finally {
        $client.Dispose()
    }
}

function Assert-StackStopped {
    foreach ($port in @(
        [int]$deployment.keycloak.httpPort,
        [int]$deployment.keycloak.managementPort,
        [int](([string]$deployment.hubService.bind).Split(':')[-1])
    )) {
        if (Test-PortOpen $port) {
            throw "TCP port $port is open. Stop Keycloak and Zircon Hub service before this operation."
        }
    }
}

function Assert-EmptyDestination([string]$Path, [string]$Name) {
    if (Test-Path -LiteralPath $Path) {
        if (-not (Test-Path -LiteralPath $Path -PathType Container) -or
            (Get-ChildItem -Force -LiteralPath $Path | Select-Object -First 1)) {
            throw "$Name must not exist or must be an empty directory."
        }
    }
}

function Assert-OutsideDirectory([string]$Candidate, [string]$Container, [string]$Name) {
    $containerPrefix = $Container.TrimEnd([IO.Path]::DirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
    if ($Candidate.StartsWith($containerPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "$Name must not be inside $Container."
    }
}

function Copy-DirectoryContents([string]$Source, [string]$Destination) {
    if (-not (Test-Path -LiteralPath $Source -PathType Container)) {
        throw "Backup source directory does not exist: $Source"
    }
    [IO.Directory]::CreateDirectory($Destination) | Out-Null
    Get-ChildItem -Force -LiteralPath $Source | Copy-Item -Force -Recurse -Destination $Destination
}

switch ($Action) {
    'ValidateArtifacts' {
        if ([int]$deployment.schemaVersion -ne 1) { throw 'Unsupported deployment schema.' }
        $realm = Get-Content -Raw -Encoding UTF8 -LiteralPath $realmPath | ConvertFrom-Json
        if ($realm.realm -ne $deployment.keycloak.realm) { throw 'Realm name differs from deployment manifest.' }
        if ($realm.defaultSignatureAlgorithm -ne 'RS256') { throw 'Realm must use RS256.' }
        $desktop = @($realm.clients | Where-Object clientId -eq $deployment.clients.desktop)
        $service = @($realm.clients | Where-Object clientId -eq $deployment.clients.service)
        if ($desktop.Count -ne 1 -or -not $desktop[0].publicClient -or
            -not $desktop[0].standardFlowEnabled -or $desktop[0].implicitFlowEnabled -or
            $desktop[0].directAccessGrantsEnabled -or $desktop[0].serviceAccountsEnabled -or
            $desktop[0].attributes.'pkce.code.challenge.method' -ne 'S256') {
            throw 'Desktop client is not a unique public S256 PKCE client.'
        }
        if ('basic' -notin @($desktop[0].defaultClientScopes)) {
            throw 'Desktop client needs the basic default client scope for the access token subject.'
        }
        if ($service.Count -ne 1 -or $service[0].publicClient -or
            $service[0].clientAuthenticatorType -ne 'client-secret' -or
            $service[0].standardFlowEnabled -or $service[0].implicitFlowEnabled -or
            $service[0].directAccessGrantsEnabled -or $service[0].serviceAccountsEnabled -or
            $service[0].secret -ne '${ZIRCON_INTROSPECTION_CLIENT_SECRET}') {
            throw 'Service client is not a unique environment-secret confidential client.'
        }
        $expectedRedirect = "http://127.0.0.1:$($deployment.clients.callbackPort)/callback"
        if (@($desktop[0].redirectUris).Count -ne 1 -or $desktop[0].redirectUris[0] -ne $expectedRedirect) {
            throw 'Desktop callback does not match the deployment manifest.'
        }
        $audience = @($desktop[0].protocolMappers | Where-Object protocolMapper -eq 'oidc-audience-mapper')
        if ($audience.Count -ne 1 -or
            $audience[0].config.'included.client.audience' -ne $deployment.clients.service -or
            $audience[0].config.'access.token.claim' -ne 'true') {
            throw 'Desktop access token audience mapper does not target the Hub service.'
        }
        Write-Output 'Deployment JSON and realm contracts are valid.'
    }
    'Configure' {
        Write-HubConfigs $StateRoot $IntrospectionSecretFile $CloudKeyFile
    }
    'ImportRealm' {
        $runtime = Resolve-Keycloak $KeycloakHome $JavaHome
        $secret = Read-SecretFile $IntrospectionSecretFile
        $oldSecret = $env:ZIRCON_INTROSPECTION_CLIENT_SECRET
        try {
            $env:ZIRCON_INTROSPECTION_CLIENT_SECRET = $secret
            Invoke-Keycloak $runtime @('import', '--file', $realmPath, '--override', 'false')
        }
        finally {
            $env:ZIRCON_INTROSPECTION_CLIENT_SECRET = $oldSecret
            $secret = $null
        }
    }
    'StartKeycloak' {
        $runtime = Resolve-Keycloak $KeycloakHome $JavaHome
        Invoke-Keycloak $runtime @(
            'start-dev',
            '--http-host', [string]$deployment.keycloak.httpHost,
            '--http-port', [string]$deployment.keycloak.httpPort,
            '--hostname', "http://$($deployment.keycloak.httpHost):$($deployment.keycloak.httpPort)",
            '--health-enabled', 'true',
            '--metrics-enabled', 'true',
            '--http-management-host', [string]$deployment.keycloak.httpHost,
            '--http-management-port', [string]$deployment.keycloak.managementPort,
            '--http-management-scheme', 'http',
            '--server-async-bootstrap', 'false'
        )
    }
    'StartService' {
        $root = Resolve-AbsolutePath 'StateRoot' $StateRoot
        $executable = Resolve-AbsolutePath 'HubServiceExecutable' $HubServiceExecutable
        if (-not (Test-Path -LiteralPath $executable -PathType Leaf)) { throw 'Hub service executable does not exist.' }
        $ready = Invoke-JsonEndpoint "http://$($deployment.keycloak.httpHost):$($deployment.keycloak.managementPort)/health/ready"
        if ($ready.status -ne 'UP') { throw 'Keycloak is not ready.' }
        $config = Join-Path $root 'config\service.toml'
        if (-not (Test-Path -LiteralPath $config -PathType Leaf)) { throw 'Run Configure before StartService.' }
        & $executable $config
        if ($LASTEXITCODE -ne 0) { throw "Zircon Hub service exited with code $LASTEXITCODE." }
    }
    'Readiness' {
        $keycloak = Invoke-JsonEndpoint "http://$($deployment.keycloak.httpHost):$($deployment.keycloak.managementPort)/health/ready"
        if ($keycloak.status -ne 'UP') { throw 'Keycloak is not ready.' }
        $service = Invoke-JsonEndpoint ([string]$deployment.hubService.healthUrl)
        if ($service.status -ne 'alive' -or [int]$service.protocolVersion -ne 1) {
            throw 'Zircon Hub service health contract did not match protocol version 1.'
        }
        Write-Output 'Keycloak and Zircon Hub service are ready.'
    }
    'Backup' {
        Assert-StackStopped
        $runtime = Resolve-Keycloak $KeycloakHome $JavaHome
        $root = Resolve-AbsolutePath 'StateRoot' $StateRoot
        $backup = Resolve-AbsolutePath 'BackupPath' $BackupPath
        Assert-OutsideDirectory $backup $root 'BackupPath'
        Assert-OutsideDirectory $backup (Join-Path $runtime.Home 'data') 'BackupPath'
        Assert-EmptyDestination $backup 'BackupPath'
        if (-not (Test-Path -LiteralPath (Join-Path $runtime.Home 'data') -PathType Container)) { throw 'Keycloak data directory does not exist.' }
        if (-not (Test-Path -LiteralPath $root -PathType Container)) { throw 'StateRoot does not exist.' }
        New-PrivateDirectory $backup
        Copy-DirectoryContents (Join-Path $runtime.Home 'data') (Join-Path $backup 'keycloak-data')
        Copy-DirectoryContents $root (Join-Path $backup 'hub-state')
        $metadata = [ordered]@{
            schemaVersion = 1
            createdAtUtc = [DateTime]::UtcNow.ToString('O')
            keycloakVersion = [string]$deployment.keycloak.version
            realm = [string]$deployment.keycloak.realm
        } | ConvertTo-Json
        Write-Utf8 (Join-Path $backup 'backup.json') ($metadata + "`n")
        Write-Output "Backup completed at $backup. Protect it as sensitive identity data."
    }
    'Restore' {
        Assert-StackStopped
        $runtime = Resolve-Keycloak $KeycloakHome $JavaHome
        $root = Resolve-AbsolutePath 'StateRoot' $StateRoot
        $backup = Resolve-AbsolutePath 'BackupPath' $BackupPath
        $keycloakData = Join-Path $runtime.Home 'data'
        $resolvedSecret = Resolve-AbsolutePath 'IntrospectionSecretFile' $IntrospectionSecretFile
        $resolvedCloudKey = Resolve-AbsolutePath 'CloudKeyFile' $CloudKeyFile
        Read-SecretFile $resolvedSecret | Out-Null
        Read-CloudKeyFile $resolvedCloudKey | Out-Null
        Assert-OutsideDirectory $root $backup 'StateRoot'
        Assert-OutsideDirectory $keycloakData $backup 'Keycloak data directory'
        Assert-OutsideDirectory $resolvedSecret $root 'IntrospectionSecretFile'
        Assert-OutsideDirectory $resolvedCloudKey $root 'CloudKeyFile'
        Assert-EmptyDestination $keycloakData 'Keycloak data directory'
        Assert-EmptyDestination $root 'StateRoot'
        $metadata = Get-Content -Raw -Encoding UTF8 -LiteralPath (Join-Path $backup 'backup.json') | ConvertFrom-Json
        if ($metadata.keycloakVersion -ne $deployment.keycloak.version -or $metadata.realm -ne $deployment.keycloak.realm) {
            throw 'Backup Keycloak version or realm does not match this deployment artifact.'
        }
        New-PrivateDirectory $root
        New-PrivateDirectory $keycloakData
        Copy-DirectoryContents (Join-Path $backup 'keycloak-data') $keycloakData
        Copy-DirectoryContents (Join-Path $backup 'hub-state') $root
        Write-HubConfigs $root $IntrospectionSecretFile $CloudKeyFile
        Write-Output 'Restore completed. Run Readiness only after both foreground services have started.'
    }
}
