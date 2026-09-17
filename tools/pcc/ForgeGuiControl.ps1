param([string]$Operation = "")

$ErrorActionPreference = "Stop"
$Root = [IO.Path]::GetFullPath((Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path)
Set-Location $Root

$ProviderVersion = "0.4.13"
$CanonicalRepository = "https://github.com/shifty81/ForgeGUI_Core.git"
$SessionStamp = Get-Date -Format "yyyyMMdd-HHmmss"
$LogDir = Join-Path $Root "artifacts\logs\sessions"
$CertDir = Join-Path $Root "artifacts\certification"
$HandoffDir = Join-Path $Root "artifacts\handoff"
$GreenPath = Join-Path $CertDir "current-green.json"
$LatestHandoffPath = Join-Path $HandoffDir "latest-handoff.json"
$PatchStateDir = Join-Path $Root "artifacts\patches\state"
$NeedsGatePath = Join-Path $PatchStateDir "needs-gate.json"
New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
New-Item -ItemType Directory -Force -Path $CertDir | Out-Null
New-Item -ItemType Directory -Force -Path $HandoffDir | Out-Null
New-Item -ItemType Directory -Force -Path $PatchStateDir | Out-Null
$LogPath = Join-Path $LogDir "forgegui-pcc-$SessionStamp.log"
$script:Utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[IO.File]::WriteAllText($LogPath, "", $script:Utf8NoBom)
$script:LastCompletedAction = ""

function Log {
    param([string]$Level, [string]$Message)
    $line = "[{0}] [{1}] {2}" -f (Get-Date -Format "yyyy-MM-dd HH:mm:ss"), $Level, $Message
    Write-Host $line
    [IO.File]::AppendAllText($LogPath, $line + [Environment]::NewLine, $script:Utf8NoBom)
}

function Format-Arguments {
    param([string[]]$Arguments)
    if ($null -eq $Arguments -or $Arguments.Count -eq 0) { return "" }
    return (($Arguments | ForEach-Object {
        if ($_ -match '\s') { '"' + $_ + '"' } else { $_ }
    }) -join " ")
}

function Add-SessionLogLine {
    param([string]$Line)
    if ($null -eq $Line) { $Line = "" }
    [IO.File]::AppendAllText($LogPath, $Line + [Environment]::NewLine, $script:Utf8NoBom)
}

function Convert-NativeOutputLine {
    param($Value)
    if ($null -eq $Value) { return "" }

    if ($Value -is [System.Management.Automation.ErrorRecord]) {
        # Native stderr is represented as ErrorRecord objects by Windows PowerShell 5.1.
        # Prefer the actual native message and never leak the wrapper type name into logs.
        $message = $null
        if ($null -ne $Value.ErrorDetails -and -not [string]::IsNullOrWhiteSpace($Value.ErrorDetails.Message)) {
            $message = $Value.ErrorDetails.Message
        } elseif ($null -ne $Value.Exception -and -not [string]::IsNullOrWhiteSpace($Value.Exception.Message)) {
            $message = $Value.Exception.Message
        } else {
            $message = $Value.ToString()
        }

        if ($message -eq "System.Management.Automation.RemoteException") { return "" }
        return [string]$message
    }

    return [string]$Value
}

function Invoke-NativeProcess {
    param(
        [Parameter(Mandatory=$true)][string]$Label,
        [Parameter(Mandatory=$true)][string]$Executable,
        [string[]]$Arguments = @(),
        [bool]$EchoOutput = $true,
        [bool]$LogCommand = $true,
        [bool]$LogOutput = $true
    )

    if ($LogCommand) {
        $rendered = Format-Arguments $Arguments
        Log "RUN" (("{0} :: {1} {2}" -f $Label, $Executable, $rendered).Trim())
    }

    try {
        Get-Command $Executable -ErrorAction Stop | Out-Null
    } catch {
        $message = "Executable not found: $Executable"
        if ($EchoOutput) { Write-Host $message }
        Add-SessionLogLine $message
        return [pscustomobject]@{ ExitCode = 127; Output = $message }
    }

    $lines = New-Object System.Collections.Generic.List[string]
    $previousPreference = $ErrorActionPreference
    $global:LASTEXITCODE = 0
    $code = 0

    try {
        # PowerShell 5.1 can promote native stderr records to terminating errors when
        # ErrorActionPreference is Stop. Native tools such as Cargo routinely use
        # stderr for normal progress output, so native success/failure must be decided
        # by the native exit code only.
        $ErrorActionPreference = "Continue"
        & $Executable @Arguments 2>&1 | ForEach-Object {
            $line = Convert-NativeOutputLine $_
            if (-not [string]::IsNullOrWhiteSpace($line)) {
                if ($EchoOutput) { Write-Host $line }
                if ($LogOutput) { Add-SessionLogLine $line }
                [void]$lines.Add($line)
            }
        }
        $code = $LASTEXITCODE
        if ($null -eq $code) { $code = 0 }
    } catch {
        $code = if ($null -ne $LASTEXITCODE -and $LASTEXITCODE -ne 0) { [int]$LASTEXITCODE } else { 1 }
        $line = $_.Exception.Message
        if ($EchoOutput) { Write-Host $line }
        if ($LogOutput) { Add-SessionLogLine $line }
        [void]$lines.Add($line)
    } finally {
        $ErrorActionPreference = $previousPreference
    }

    return [pscustomobject]@{
        ExitCode = [int]$code
        Output = ($lines -join "`n")
    }
}

function Get-NativeFailureTail {
    param([string]$Output, [int]$Count = 16)
    if ([string]::IsNullOrWhiteSpace($Output)) { return "" }
    $items = @($Output -split "`r?`n" | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    if ($items.Count -eq 0) { return "" }
    return (($items | Select-Object -Last $Count) -join "`n")
}

function Run-Native {
    param(
        [Parameter(Mandatory=$true)][string]$Label,
        [Parameter(Mandatory=$true)][string]$Executable,
        [string[]]$Arguments = @()
    )

    $result = Invoke-NativeProcess $Label $Executable $Arguments $true $true
    if ($result.ExitCode -ne 0) {
        $tail = Get-NativeFailureTail $result.Output
        if ([string]::IsNullOrWhiteSpace($tail)) {
            throw "$Label failed with exit code $($result.ExitCode)"
        }
        throw "$Label failed with exit code $($result.ExitCode)`n$tail"
    }
    Log "PASS" $Label
}

function Capture-Native {
    param([string]$Executable, [string[]]$Arguments = @())
    $result = Invoke-NativeProcess "capture $Executable" $Executable $Arguments $false $false $false
    if ($result.ExitCode -ne 0) { return $null }
    return ([string]$result.Output).Trim()
}

function Invoke-NativeResult {
    param(
        [Parameter(Mandatory=$true)][string]$Label,
        [Parameter(Mandatory=$true)][string]$Executable,
        [string[]]$Arguments = @()
    )
    return Invoke-NativeProcess $Label $Executable $Arguments $true $true
}

function Test-LockRefreshFailure {
    param([string]$Text)
    if ([string]::IsNullOrWhiteSpace($Text)) { return $false }
    return (
        $Text -match 'cannot update the lock file' -or
        $Text -match 'lock file .* needs to be updated' -or
        $Text -match 'the lock file .* needs to be updated' -or
        $Text -match 'Cargo\.lock .* needs to be updated'
    )
}

function Test-PathSetTouchesRust {
    param([string[]]$Paths)
    foreach ($rel in @($Paths)) {
        if (([string]$rel).ToLowerInvariant().EndsWith(".rs")) { return $true }
    }
    return $false
}

function Test-PathSetTouchesCargoGraph {
    param([string[]]$Paths)
    foreach ($rel in @($Paths)) {
        $lower = ([string]$rel).Replace('\','/').ToLowerInvariant()
        if ($lower -eq "cargo.toml" -or $lower.EndsWith("/cargo.toml") -or $lower -eq "cargo.lock") {
            return $true
        }
    }
    return $false
}

function Get-PackageManifestFormatState {
    $manifestPath = Join-Path $Root "PACKAGE_MANIFEST.json"
    if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) { return $null }
    try {
        $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
        return [string]$manifest.format_state
    } catch {
        throw "PACKAGE_MANIFEST.json is unreadable during patch normalization."
    }
}

function Invoke-ControlledCargoLockReconcile {
    param([bool]$AllowRefresh = $true)

    $lockPath = Join-Path $Root "Cargo.lock"
    if (-not (Test-Path -LiteralPath $lockPath -PathType Leaf)) {
        if (-not $AllowRefresh) {
            throw "Cargo.lock is missing; Full Gate or patch normalization must regenerate it."
        }
        Log "INFO" "Cargo.lock is missing after patch application. Generating the workspace lockfile."
        Run-Native "post-patch generate Cargo.lock" "cargo" @("generate-lockfile")
    }

    $probe = Invoke-NativeResult "post-patch Cargo metadata / locked probe" "cargo" @(
        "metadata","--locked","--format-version","1","--no-deps"
    )
    if ($probe.ExitCode -eq 0) {
        Log "PASS" "Cargo.lock / workspace metadata are synchronized"
        return
    }

    if (-not (Test-LockRefreshFailure $probe.Output)) {
        $tail = Get-NativeFailureTail $probe.Output
        if ([string]::IsNullOrWhiteSpace($tail)) {
            throw "Cargo metadata locked probe failed with exit code $($probe.ExitCode)"
        }
        throw "Cargo metadata locked probe failed with exit code $($probe.ExitCode)`n$tail"
    }

    if (-not $AllowRefresh) {
        throw "Cargo.lock is stale for the current workspace. Apply/update normalization or Full Gate is required."
    }

    Log "WARN" "Cargo.lock is stale after a governed patch. Performing one controlled lock refresh."
    Run-Native "post-patch refresh Cargo.lock" "cargo" @("generate-lockfile")
    Run-Native "post-patch validate Cargo.lock" "cargo" @(
        "metadata","--locked","--format-version","1","--no-deps"
    )
}

function Get-PatchTransactionBackupPaths {
    param([string[]]$TouchedPaths)

    $set = @{}
    foreach ($rel in @($TouchedPaths)) {
        if (-not [string]::IsNullOrWhiteSpace([string]$rel)) {
            $normalized = ([string]$rel).Replace('\','/')
            $set[$normalized.ToLowerInvariant()] = $normalized
        }
    }

    # Normalization may rewrite the package manifest and Cargo.lock.
    $set["package_manifest.json"] = "PACKAGE_MANIFEST.json"
    $set["cargo.lock"] = "Cargo.lock"

    # rustfmt --all is intentionally project-wide. Back up every governed Rust file
    # for every patch transaction so a normalization or manifest-state transition
    # can always roll the complete source transformation back safely.
    foreach ($rel in @(Get-GovernedPackageFilePaths)) {
        if ($rel.ToLowerInvariant().EndsWith(".rs")) {
            $set[$rel.ToLowerInvariant()] = $rel
        }
    }

    return @($set.Values | Sort-Object)
}

function Invoke-PostPatchNormalization {
    param(
        [string[]]$TouchedPaths,
        [bool]$StartupRecovery = $false
    )

    $manifestState = Get-PackageManifestFormatState
    $rustRequired = (Test-PathSetTouchesRust $TouchedPaths) -or ($manifestState -eq "requires-canonicalization")
    $cargoRequired = (Test-PathSetTouchesCargoGraph $TouchedPaths) -or $StartupRecovery

    if ($rustRequired) {
        Log "INFO" "Post-patch normalization: canonicalizing Rust source automatically."
        Run-Native "post-patch cargo fmt apply" "cargo" @("fmt","--all")
        Run-Native "post-patch cargo fmt check" "cargo" @("fmt","--all","--check")
    }

    if ($cargoRequired) {
        Invoke-ControlledCargoLockReconcile $true
    }

    # Any controlled transformation becomes the new governed source authority.
    if ($rustRequired -or $cargoRequired -or $manifestState -eq "requires-canonicalization") {
        Update-PackageManifestHashes
        Test-PackageManifest
    }

    return [pscustomobject]@{
        rustfmt = [bool]$rustRequired
        cargoLock = [bool]$cargoRequired
        manifestState = (Get-PackageManifestFormatState)
    }
}

function Repair-PendingPatchNormalizationIfNeeded {
    $manifestState = Get-PackageManifestFormatState
    $state = Read-NeedsGateState

    $requiresRecovery = ($manifestState -eq "requires-canonicalization")
    if ($null -ne $state -and -not [bool]$state.normalized) {
        $requiresRecovery = $true
    }

    # Provider-transition recovery must trust rustfmt itself, not only the package
    # manifest flag. An older provider can reconcile post-image hashes and leave
    # format_state=canonical even though newly-written Rust has not been formatted.
    $rustfmtDrift = $false
    if (-not $requiresRecovery) {
        $fmtProbe = Invoke-NativeResult "startup rustfmt canonical probe" "cargo" @("fmt","--all","--check")
        if ($fmtProbe.ExitCode -ne 0) {
            $rustfmtDrift = $true
            $requiresRecovery = $true
            Log "WARN" "Rust source is not rustfmt-canonical; automatic startup normalization is required."
        }
    }

    if (-not $requiresRecovery) { return }

    $latest = @(Get-ActivePatchReceipts | Sort-Object appliedAt | Select-Object -Last 1)
    $touched = @()
    $patchId = $null
    if ($latest.Count -gt 0) {
        $touched = @($latest[0].touchedPaths)
        $patchId = [string]$latest[0].id
    }

    # If rustfmt itself detected drift, force the Rust branch even when the newest
    # receipt is only a provider/documentation patch.
    if ($rustfmtDrift) {
        $touched = @($touched) + @("__startup_rustfmt_recovery__.rs")
    }

    Log "WARN" "A previously applied patch requires automatic normalization before normal PCC use."
    [void](Invoke-PostPatchNormalization $touched $true)
    Write-NeedsGateState "Patch/update normalization completed automatically; explicit Full Gate is required." $patchId $true
    Clear-Green
    Log "PASS" "Patch/update normalization is complete. Project is ready for Full Gate."
}

function Assert-RunReady {
    $pendingCount = @(Get-ChildItem -LiteralPath $Root -File -Filter "*.patch" -ErrorAction SilentlyContinue).Count
    if ($pendingCount -gt 0) {
        throw "Run blocked: $pendingCount root patch transport(s) are still pending."
    }

    $state = Read-NeedsGateState
    if ($null -ne $state) {
        throw "Run blocked: $($state.state). Run FULL QUALITY GATE / CERTIFY GREEN first."
    }

    Test-PackageManifest
    if ((Get-PackageManifestFormatState) -ne "canonical") {
        throw "Run blocked: package manifest is not canonical. Run Full Gate."
    }

    $green = Read-Green
    if ($null -eq $green) {
        throw "Run blocked: no current GREEN certification exists. Run Full Gate first."
    }
    $currentFingerprint = Get-SourceFingerprint
    if ([string]$green.sourceFingerprint -ne $currentFingerprint) {
        throw "Run blocked: source changed after GREEN certification. Run Full Gate again."
    }

    Invoke-ControlledCargoLockReconcile $false
}

function Run-LockedWorkspaceCheck {
    $arguments = @("check","--locked","--workspace","--all-targets")
    $first = Invoke-NativeResult "cargo check workspace / locked preflight" "cargo" $arguments
    if ($first.ExitCode -eq 0) {
        Log "PASS" "cargo check workspace"
        return
    }

    if (-not (Test-LockRefreshFailure $first.Output)) {
        $tail = Get-NativeFailureTail $first.Output
        if ([string]::IsNullOrWhiteSpace($tail)) {
            throw "cargo check workspace failed with exit code $($first.ExitCode)"
        }
        throw "cargo check workspace failed with exit code $($first.ExitCode)`n$tail"
    }

    Log "WARN" "Cargo.lock is stale for the complete workspace/all-targets graph. Performing one controlled lock refresh; Cargo stderr progress is informational unless the native exit code is nonzero."
    Clear-Green
    Run-Native "refresh Cargo.lock" "cargo" @("generate-lockfile")
    Run-Native "Cargo workspace metadata / refreshed lock" "cargo" @("metadata","--locked","--format-version","1","--no-deps")
    Run-Native "cargo check workspace / refreshed lock" "cargo" $arguments
}

function Get-RelativePath {
    param([string]$Path)
    $full = [IO.Path]::GetFullPath($Path)
    if ($full.StartsWith($Root, [StringComparison]::OrdinalIgnoreCase)) {
        return $full.Substring($Root.Length).TrimStart([char[]]'\/').Replace('\','/')
    }
    return $full.Replace('\','/')
}

function Get-GovernedPackageFilePaths {
    $paths = New-Object System.Collections.Generic.List[string]
    foreach ($file in @(Get-ChildItem -LiteralPath $Root -Recurse -Force -File -ErrorAction SilentlyContinue)) {
        $rel = Get-RelativePath $file.FullName
        $lower = $rel.ToLowerInvariant()
        if ($lower -match '^(target|artifacts|\.git)/') { continue }
        if ($lower -eq 'package_manifest.json' -or $lower -eq 'cargo.lock') { continue }
        if ($lower -match '(^|/)[^/]+\.patch$') { continue }
        if ($lower -match '\.(bak|orig|rej|tmp|log|zip|7z|rar)$') { continue }
        if ($lower -match '\.pre-[^/]*\.bak$') { continue }
        [void]$paths.Add($rel)
    }
    return @($paths | Sort-Object -Unique)
}

function Get-SourceFingerprint {
    $records = New-Object System.Collections.Generic.List[string]
    $files = Get-ChildItem -LiteralPath $Root -Recurse -Force -File -ErrorAction SilentlyContinue
    foreach ($file in $files) {
        $rel = Get-RelativePath $file.FullName
        $lower = $rel.ToLowerInvariant()

        if ($lower -match '^(target|artifacts|\.git)/') { continue }
        if ($lower -match '(^|/)[^/]+\.patch$') { continue }
        if ($lower -match '\.(bak|orig|rej|tmp|log|zip|7z|rar)$') { continue }
        if ($lower -match '\.pre-[^/]*\.bak$') { continue }

        $hash = (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
        [void]$records.Add("$rel|$hash")
    }

    $ordered = @($records | Sort-Object)
    $payload = [Text.Encoding]::UTF8.GetBytes(($ordered -join "`n"))
    $sha = [Security.Cryptography.SHA256]::Create()
    try {
        return ([BitConverter]::ToString($sha.ComputeHash($payload))).Replace('-','').ToLowerInvariant()
    } finally {
        $sha.Dispose()
    }
}

function Clear-Green {
    if (Test-Path -LiteralPath $GreenPath) {
        Remove-Item -LiteralPath $GreenPath -Force
    }
}

function Write-NeedsGateState {
    param(
        [Parameter(Mandatory=$true)][string]$Reason,
        [string]$PatchId = $null,
        [bool]$Normalized = $false
    )

    New-Item -ItemType Directory -Force -Path $PatchStateDir | Out-Null
    $payload = [ordered]@{
        schema = "forge.patch_state.v1"
        state = "PATCH_APPLIED_NEEDS_GATE"
        reason = $Reason
        patchId = $PatchId
        normalized = $Normalized
        updatedAt = (Get-Date).ToString("o")
    }
    [IO.File]::WriteAllText(
        $NeedsGatePath,
        ($payload | ConvertTo-Json -Depth 5) + [Environment]::NewLine,
        $script:Utf8NoBom
    )
}

function Read-NeedsGateState {
    if (-not (Test-Path -LiteralPath $NeedsGatePath -PathType Leaf)) { return $null }
    try {
        return Get-Content -LiteralPath $NeedsGatePath -Raw | ConvertFrom-Json
    } catch {
        Log "WARN" "Patch state is unreadable and will be treated as requiring a Full Gate."
        return [pscustomobject]@{
            state = "PATCH_APPLIED_NEEDS_GATE"
            reason = "patch state unreadable"
            patchId = $null
            normalized = $false
        }
    }
}

function Clear-NeedsGateState {
    if (Test-Path -LiteralPath $NeedsGatePath) {
        Remove-Item -LiteralPath $NeedsGatePath -Force
    }
}

function Get-PatchStateText {
    $state = Read-NeedsGateState
    if ($null -eq $state) { return "READY" }
    $normalization = if ([bool]$state.normalized) { "normalized / Full Gate required" } else { "normalization required" }
    if (-not [string]::IsNullOrWhiteSpace([string]$state.patchId)) {
        return "$($state.state) / $normalization / $($state.patchId)"
    }
    return "$($state.state) / $normalization"
}

function Read-Green {
    if (-not (Test-Path -LiteralPath $GreenPath)) { return $null }
    try { return (Get-Content -LiteralPath $GreenPath -Raw | ConvertFrom-Json) }
    catch { return $null }
}

function Get-GreenStatus {
    $green = Read-Green
    if ($null -eq $green) { return "NONE" }
    try {
        $current = Get-SourceFingerprint
        if ($current -eq [string]$green.sourceFingerprint) {
            return "GREEN $($green.gateId) / CURRENT"
        }
        return "STALE $($green.gateId) / SOURCE CHANGED"
    } catch {
        return "UNKNOWN / fingerprint failed"
    }
}

function Get-GitStatusText {
    $inside = Capture-Native "git" @("rev-parse","--is-inside-work-tree")
    if ($inside -ne "true") { return "STANDALONE / NO-REPO" }

    $branch = Capture-Native "git" @("branch","--show-current")
    if ([string]::IsNullOrWhiteSpace($branch)) { $branch = "DETACHED" }
    $head = Capture-Native "git" @("rev-parse","--short","HEAD")
    if ([string]::IsNullOrWhiteSpace($head)) { $head = "NO-COMMIT" }
    $dirty = Capture-Native "git" @("status","--porcelain")
    $state = if ([string]::IsNullOrWhiteSpace($dirty)) { "Clean" } else { "Modified" }
    return "$branch / $state @ $head"
}

function Open-HandoffFolder {
    try {
        New-Item -ItemType Directory -Force -Path $HandoffDir | Out-Null
        Start-Process -FilePath "explorer.exe" -ArgumentList ('"{0}"' -f $HandoffDir) | Out-Null
        Log "INFO" "Opened handoff folder: $HandoffDir"
    } catch {
        Log "WARN" "Could not open handoff folder: $($_.Exception.Message)"
    }
}

function Open-HandoffBundle {
    param([string]$Path)
    try {
        if (-not (Test-Path -LiteralPath $Path)) {
            Open-HandoffFolder
            return
        }
        $argument = '/select,"{0}"' -f $Path
        Start-Process -FilePath "explorer.exe" -ArgumentList $argument | Out-Null
        Log "INFO" "Opened Explorer at handoff bundle: $Path"
    } catch {
        Log "WARN" "Could not open Explorer at handoff bundle: $($_.Exception.Message)"
    }
}

function Package-Debug {
    param(
        [string]$Reason,
        [string]$Kind = "FAIL",
        [bool]$OpenAfter = $true
    )

    New-Item -ItemType Directory -Force -Path $HandoffDir | Out-Null
    $bundleId = "DBG-" + (Get-Date -Format "yyyyMMdd-HHmmss") + "-" + $Kind + "-" + ([guid]::NewGuid().ToString("N").Substring(0,8))
    $zip = Join-Path $HandoffDir ("ForgeGUI_DebugBundle_{0}.zip" -f $bundleId)
    $tmp = Join-Path $env:TEMP ("forgegui-debug-" + [guid]::NewGuid())
    New-Item -ItemType Directory -Force -Path $tmp | Out-Null

    try {
        foreach ($file in @(
            "Cargo.toml","Cargo.lock","project.control.json","forge.gui.toml",
            "PACKAGE_MANIFEST.json","README.md","VERSION","PROJECT_CONTROL_CENTER.cmd"
        )) {
            $source = Join-Path $Root $file
            if (Test-Path -LiteralPath $source) {
                Copy-Item -LiteralPath $source -Destination (Join-Path $tmp ([IO.Path]::GetFileName($file))) -Force
            }
        }

        $providerDir = Join-Path $tmp "pcc"
        New-Item -ItemType Directory -Force -Path $providerDir | Out-Null
        foreach ($provider in @(
            "tools\\pcc\\ForgeGuiControl.ps1",
            "tools\\pcc\\ForgeGuiRootHygiene.ps1",
            "docs\\INTERNAL_PCC_STANDARD.md"
        )) {
            $source = Join-Path $Root $provider
            if (Test-Path -LiteralPath $source) {
                Copy-Item -LiteralPath $source -Destination (Join-Path $providerDir ([IO.Path]::GetFileName($provider))) -Force
            }
        }

        $logsDir = Join-Path $tmp "logs"
        New-Item -ItemType Directory -Force -Path $logsDir | Out-Null
        if (Test-Path -LiteralPath $LogPath) {
            Copy-Item -LiteralPath $LogPath -Destination (Join-Path $logsDir ([IO.Path]::GetFileName($LogPath))) -Force
        }
        $recentLogs = @(Get-ChildItem -LiteralPath $LogDir -File -Filter "forgegui-pcc-*.log" -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending | Select-Object -First 5)
        foreach ($recent in $recentLogs) {
            $dest = Join-Path $logsDir $recent.Name
            if (-not (Test-Path -LiteralPath $dest)) {
                Copy-Item -LiteralPath $recent.FullName -Destination $dest -Force
            }
        }

        if (Test-Path -LiteralPath $GreenPath) {
            Copy-Item -LiteralPath $GreenPath -Destination (Join-Path $tmp "current-green.json") -Force
        }

        $receiptDir = Join-Path $tmp "patch-receipts"
        New-Item -ItemType Directory -Force -Path $receiptDir | Out-Null
        $transactions = Join-Path $Root "artifacts\\patches\\transactions"
        if (Test-Path -LiteralPath $transactions) {
            $receipts = @(Get-ChildItem -LiteralPath $transactions -Recurse -File -Filter "receipt.json" -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending | Select-Object -First 10)
            $index = 0
            foreach ($receipt in $receipts) {
                $index++
                Copy-Item -LiteralPath $receipt.FullName -Destination (Join-Path $receiptDir ("receipt-{0:D2}.json" -f $index)) -Force
            }
        }

        $patchEvidenceDir = Join-Path $tmp "patch-evidence"
        New-Item -ItemType Directory -Force -Path $patchEvidenceDir | Out-Null
        $evidencePatchFiles = New-Object System.Collections.Generic.List[IO.FileInfo]

        $pendingPatchNames = @()
        foreach ($pendingPatch in @(Get-ChildItem -LiteralPath $Root -File -Filter "*.patch" -ErrorAction SilentlyContinue | Sort-Object Name)) {
            $pendingPatchNames += $pendingPatch.Name
            [void]$evidencePatchFiles.Add($pendingPatch)
            Copy-Item -LiteralPath $pendingPatch.FullName -Destination (Join-Path $patchEvidenceDir $pendingPatch.Name) -Force
        }

        $rebaseNeededNames = @()
        $rebaseNeededDir = Join-Path $Root "artifacts\patches\rebase-needed"
        if (Test-Path -LiteralPath $rebaseNeededDir) {
            $rebaseCandidates = @(
                Get-ChildItem -LiteralPath $rebaseNeededDir -File -Filter "*.patch" -ErrorAction SilentlyContinue |
                Sort-Object LastWriteTime -Descending |
                Select-Object -First 5
            )
            foreach ($candidate in $rebaseCandidates) {
                $rebaseNeededNames += $candidate.Name
                [void]$evidencePatchFiles.Add($candidate)
                Copy-Item -LiteralPath $candidate.FullName -Destination (Join-Path $patchEvidenceDir $candidate.Name) -Force
                $record = $candidate.FullName + ".rebase.json"
                if (Test-Path -LiteralPath $record -PathType Leaf) {
                    Copy-Item -LiteralPath $record -Destination (Join-Path $patchEvidenceDir ([IO.Path]::GetFileName($record))) -Force
                }
            }
        }

        $currentSourceDir = Join-Path $patchEvidenceDir "current-source"
        New-Item -ItemType Directory -Force -Path $currentSourceDir | Out-Null
        $evidencePaths = @{}
        foreach ($evidencePatch in @($evidencePatchFiles)) {
            try {
                foreach ($rel in @(Get-PatchTouchedPaths $evidencePatch)) {
                    $key = $rel.ToLowerInvariant()
                    if ($evidencePaths.ContainsKey($key)) { continue }

                    $source = Join-Path $Root $rel.Replace('/', '\')
                    $exists = Test-Path -LiteralPath $source -PathType Leaf
                    $hash = $null
                    if ($exists) {
                        $destination = Join-Path $currentSourceDir $rel.Replace('/', '\')
                        $parent = Split-Path -Parent $destination
                        if ($parent) { New-Item -ItemType Directory -Force -Path $parent | Out-Null }
                        Copy-Item -LiteralPath $source -Destination $destination -Force
                        $hash = (Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant()
                    }

                    $evidencePaths[$key] = [ordered]@{
                        path = $rel
                        exists = [bool]$exists
                        sha256 = $hash
                    }
                }
            } catch {
                Log "WARN" "Unable to collect current-source evidence for $($evidencePatch.Name): $($_.Exception.Message)"
            }
        }

        @($evidencePaths.Values | Sort-Object path) |
            ConvertTo-Json -Depth 6 |
            Set-Content -LiteralPath (Join-Path $patchEvidenceDir "current-source-index.json") -Encoding UTF8

        $rustc = Capture-Native "rustc" @("--version")
        $cargo = Capture-Native "cargo" @("--version")
        $gitVersion = Capture-Native "git" @("--version")
        $fingerprint = $null
        try { $fingerprint = Get-SourceFingerprint } catch { $fingerprint = "unavailable: $($_.Exception.Message)" }

        Set-Content -LiteralPath (Join-Path $tmp "failure.txt") -Value $Reason -Encoding UTF8
        [ordered]@{
            schema = "forge.debug.context.v3"
            bundleId = $bundleId
            generatedAt = (Get-Date).ToString("o")
            reason = $Reason
            kind = $Kind
            root = $Root
            sessionLog = Get-RelativePath $LogPath
            sourceFingerprint = $fingerprint
            environment = [ordered]@{
                powershell = $PSVersionTable.PSVersion.ToString()
                rustc = $rustc
                cargo = $cargo
                git = $gitVersion
            }
            git = Get-GitStatusText
            green = Get-GreenStatus
            pendingPatches = $pendingPatchNames
            rebaseNeededPatches = $rebaseNeededNames
            patchEvidenceDirectory = "patch-evidence"
        } | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $tmp "context.json") -Encoding UTF8

        Compress-Archive -Path (Join-Path $tmp "*") -DestinationPath $zip -Force

        [ordered]@{
            schema = "forge.handoff.latest.v1"
            generatedAt = (Get-Date).ToString("o")
            bundleId = $bundleId
            kind = $Kind
            reason = $Reason
            bundle = Get-RelativePath $zip
            sessionLog = Get-RelativePath $LogPath
        } | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $LatestHandoffPath -Encoding UTF8

        Log "PASS" "Debug handoff bundle: $zip"
        Write-Host "[HANDOFF] $zip" -ForegroundColor Cyan
        if ($OpenAfter) { Open-HandoffBundle $zip }
        return $zip
    } finally {
        if (Test-Path -LiteralPath $tmp) {
            Remove-Item -LiteralPath $tmp -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
}
function Get-PatchTouchedPaths {
    param([IO.FileInfo]$Patch)
    $paths = New-Object System.Collections.Generic.List[string]
    foreach ($line in Get-Content -LiteralPath $Patch.FullName) {
        if ($line -notmatch '^(---|\+\+\+)\s+(.+)$') { continue }
        $raw = $Matches[2].Trim()
        if ($raw -eq '/dev/null') { continue }
        if ($raw.StartsWith('"')) { throw "Quoted/escaped patch paths are not supported by forge.patch.v1: $raw" }
        if ($raw.StartsWith('a/') -or $raw.StartsWith('b/')) { $raw = $raw.Substring(2) }
        if ([string]::IsNullOrWhiteSpace($raw) -or $raw.Contains("`t")) { throw "Invalid patch path header: $raw" }
        $segments = @($raw -split '[\\/]')
        if ([IO.Path]::IsPathRooted($raw) -or $raw -match '^[A-Za-z]:' -or ($segments -contains '..')) {
            throw "Unsafe patch path: $raw"
        }
        $normalized = $raw.Replace('\','/')
        if (-not $paths.Contains($normalized)) { [void]$paths.Add($normalized) }
    }
    if ($paths.Count -eq 0) { throw "Patch contains no safe file paths" }
    return @($paths)
}

function Backup-PatchTouchedPaths {
    param([string[]]$Paths,[string]$TransactionDir)
    $backupRoot = Join-Path $TransactionDir 'backup'
    New-Item -ItemType Directory -Force -Path $backupRoot | Out-Null
    $records = @()
    foreach ($rel in $Paths) {
        $source = Join-Path $Root $rel.Replace('/', '\')
        $backup = Join-Path $backupRoot $rel.Replace('/', '\')
        $existed = Test-Path -LiteralPath $source -PathType Leaf
        if ($existed) {
            $parent = Split-Path -Parent $backup
            if ($parent) { New-Item -ItemType Directory -Force -Path $parent | Out-Null }
            Copy-Item -LiteralPath $source -Destination $backup -Force
        }
        $records += [ordered]@{ path=$rel; existed=[bool]$existed }
    }
    $records | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $TransactionDir 'backup-manifest.json') -Encoding UTF8
}

function Restore-PatchTransaction {
    param([string]$TransactionDir)
    $manifestPath = Join-Path $TransactionDir 'backup-manifest.json'
    if (-not (Test-Path -LiteralPath $manifestPath)) { throw "Patch backup manifest missing: $TransactionDir" }
    $records = @(Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json)
    $backupRoot = Join-Path $TransactionDir 'backup'
    foreach ($record in $records) {
        $rel = [string]$record.path
        $target = Join-Path $Root $rel.Replace('/', '\')
        $backup = Join-Path $backupRoot $rel.Replace('/', '\')
        if ([bool]$record.existed) {
            if (-not (Test-Path -LiteralPath $backup -PathType Leaf)) { throw "Patch backup file missing: $rel" }
            $parent = Split-Path -Parent $target
            if ($parent) { New-Item -ItemType Directory -Force -Path $parent | Out-Null }
            Copy-Item -LiteralPath $backup -Destination $target -Force
        } elseif (Test-Path -LiteralPath $target -PathType Leaf) {
            Remove-Item -LiteralPath $target -Force
        }
    }
}

function Get-PatchPostImages {
    param([string[]]$Paths)
    $records = @()
    foreach ($rel in $Paths) {
        $target = Join-Path $Root $rel.Replace('/', '\')
        if (Test-Path -LiteralPath $target -PathType Leaf) {
            $records += [ordered]@{ path=$rel; exists=$true; sha256=(Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash.ToLowerInvariant() }
        } else {
            $records += [ordered]@{ path=$rel; exists=$false; sha256=$null }
        }
    }
    return @($records)
}

function Assert-PatchPostImagesCurrent {
    param($PostImages)
    foreach ($record in @($PostImages)) {
        $rel = [string]$record.path
        $target = Join-Path $Root $rel.Replace('/', '\')
        $exists = Test-Path -LiteralPath $target -PathType Leaf
        if ([bool]$record.exists) {
            if (-not $exists) { throw "Rollback blocked: post-image file is missing: $rel" }
            $hash = (Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash.ToLowerInvariant()
            if ($hash -ne ([string]$record.sha256).ToLowerInvariant()) { throw "Rollback blocked: $rel changed after patch application" }
        } elseif ($exists) {
            throw "Rollback blocked: $rel was created/changed after patch application"
        }
    }
}


function Get-ActivePatchReceipts {
    $transactions = Join-Path $Root "artifacts\patches\transactions"
    if (-not (Test-Path -LiteralPath $transactions)) { return @() }

    $result = @()
    $files = @(Get-ChildItem -LiteralPath $transactions -Recurse -File -Filter "receipt.json" -ErrorAction SilentlyContinue | Sort-Object LastWriteTime)
    foreach ($file in $files) {
        try {
            $data = Get-Content -LiteralPath $file.FullName -Raw | ConvertFrom-Json
            if ($null -ne $data.rolledBackAt) { continue }
            if ($null -eq $data.postImages) { continue }
            $result += $data
        } catch {
            Log "WARN" "Ignoring unreadable patch receipt during reconciliation: $(Get-RelativePath $file.FullName)"
        }
    }
    return @($result)
}

function Test-PatchShaAlreadyApplied {
    param([string]$Sha)
    if ([string]::IsNullOrWhiteSpace($Sha)) { return $false }

    foreach ($receipt in @(Get-ActivePatchReceipts)) {
        if (([string]$receipt.sha256).ToLowerInvariant() -ne $Sha.ToLowerInvariant()) { continue }
        try {
            Assert-PatchPostImagesCurrent $receipt.postImages
            return $true
        } catch {
            # A receipt whose post-image no longer matches current source is historical,
            # not proof that this transport is already represented by current source.
        }
    }
    return $false
}

function Resolve-AlreadyAppliedRootPatches {
    $duplicates = Join-Path $Root "artifacts\patches\duplicates"
    New-Item -ItemType Directory -Force -Path $duplicates | Out-Null

    foreach ($patch in @(Get-ChildItem -LiteralPath $Root -File -Filter "*.patch" -ErrorAction SilentlyContinue | Sort-Object Name)) {
        try {
            $sha = (Get-FileHash -LiteralPath $patch.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
            if (-not (Test-PatchShaAlreadyApplied $sha)) { continue }

            $name = "{0}-{1}" -f (Get-Date -Format "yyyyMMdd-HHmmssfff"), $patch.Name
            $destination = Join-Path $duplicates $name
            Move-Item -LiteralPath $patch.FullName -Destination $destination -Force
            Log "PASS" "Archived already-applied duplicate root patch: $($patch.Name) sha256=$sha"
        } catch {
            Log "WARN" "Unable to resolve duplicate root patch $($patch.Name): $($_.Exception.Message)"
        }
    }
}

function Reconcile-PackageManifestFromAppliedReceipts {
    $manifestPath = Join-Path $Root "PACKAGE_MANIFEST.json"
    if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) { return }

    $receipts = @(Get-ActivePatchReceipts)
    if ($receipts.Count -eq 0) { return }

    $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
    if ($manifest.schema -ne "forgegui.package_manifest.v1") {
        Log "WARN" "Patch receipt reconciliation skipped: unsupported package manifest schema."
        return
    }

    $governedSet = @{}
    foreach ($rel in @(Get-GovernedPackageFilePaths)) {
        $governedSet[$rel.ToLowerInvariant()] = $rel
    }

    $entries = @{}
    foreach ($entry in @($manifest.files)) {
        $rel = ([string]$entry.path).Replace('\','/')
        $entries[$rel.ToLowerInvariant()] = $entry
    }

    $changed = $false
    foreach ($receipt in $receipts) {
        try {
            # Reconciliation is allowed only when every recorded post-image for the
            # receipt still matches current source exactly.
            Assert-PatchPostImagesCurrent $receipt.postImages
        } catch {
            continue
        }

        foreach ($post in @($receipt.postImages)) {
            $rel = ([string]$post.path).Replace('\','/')
            if ([string]::IsNullOrWhiteSpace($rel)) { continue }
            $key = $rel.ToLowerInvariant()

            if ($governedSet.ContainsKey($key) -and [bool]$post.exists) {
                $path = Join-Path $Root $rel.Replace('/', '\')
                if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { continue }
                $bytes = [int64](Get-Item -LiteralPath $path).Length
                $hash = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()

                if ($entries.ContainsKey($key)) {
                    $entry = $entries[$key]
                    if ([int64]$entry.bytes -ne $bytes -or ([string]$entry.sha256).ToLowerInvariant() -ne $hash) {
                        $entry.bytes = $bytes
                        $entry.sha256 = $hash
                        $changed = $true
                    }
                } else {
                    $entries[$key] = [pscustomobject][ordered]@{
                        path = $rel
                        bytes = $bytes
                        sha256 = $hash
                    }
                    $changed = $true
                }
            } elseif ($entries.ContainsKey($key) -and -not [bool]$post.exists) {
                [void]$entries.Remove($key)
                $changed = $true
            }
        }
    }

    if (-not $changed) { return }

    $manifest.files = @($entries.Values | Sort-Object { ([string]$_.path).ToLowerInvariant() })
    if ($manifest.PSObject.Properties.Name -contains "file_count") {
        $manifest.file_count = @($manifest.files).Count
    } else {
        $manifest | Add-Member -NotePropertyName file_count -NotePropertyValue @($manifest.files).Count
    }

    $json = $manifest | ConvertTo-Json -Depth 8
    [IO.File]::WriteAllText($manifestPath, $json + [Environment]::NewLine, $script:Utf8NoBom)
    Log "PASS" "PACKAGE_MANIFEST.json reconciled from verified active patch receipt post-images"
}

function Rollback-LastPatch {
    $transactions = Join-Path $Root 'artifacts\patches\transactions'
    if (-not (Test-Path -LiteralPath $transactions)) { Log "INFO" "No patch transactions exist."; return }
    $receipt = Get-ChildItem -LiteralPath $transactions -Recurse -File -Filter 'receipt.json' -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending | Where-Object {
        try { $r=Get-Content -LiteralPath $_.FullName -Raw | ConvertFrom-Json; -not $r.rolledBackAt } catch { $false }
    } | Select-Object -First 1
    if ($null -eq $receipt) { Log "INFO" "No applied patch transaction is available to roll back."; return }
    $txn = Split-Path -Parent $receipt.FullName
    $data = Get-Content -LiteralPath $receipt.FullName -Raw | ConvertFrom-Json
    if ($null -eq $data.postImages) { throw "Rollback blocked: transaction predates post-image safety metadata" }
    $safetyImages = if ($null -ne $data.transactionPostImages) { $data.transactionPostImages } else { $data.postImages }
    Assert-PatchPostImagesCurrent $safetyImages
    Restore-PatchTransaction $txn
    $data | Add-Member -NotePropertyName rolledBackAt -NotePropertyValue (Get-Date).ToString('o') -Force
    $data | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $receipt.FullName -Encoding UTF8
    Clear-Green
    Write-NeedsGateState "Patch rollback changed governed source; explicit Full Gate is required." $data.id $true
    Log "PASS" "Rolled back patch transaction $($data.id). GREEN invalidated."
}

function Resolve-SupersededRootPatches {
    param(
        [string[]]$AppliedPaths,
        [datetime]$AppliedTransportWriteTimeUtc
    )

    $appliedSet = @{}
    foreach ($rel in @($AppliedPaths)) {
        $normalized = ([string]$rel).Replace('\','/').ToLowerInvariant()
        $appliedSet[$normalized] = $true
    }

    $supersededDir = Join-Path $Root "artifacts\patches\superseded"
    foreach ($candidate in @(Get-ChildItem -LiteralPath $Root -File -Filter "*.patch" -ErrorAction SilentlyContinue | Sort-Object Name)) {
        try {
            # Never auto-classify a newer transport as stale.
            if ($candidate.LastWriteTimeUtc -gt $AppliedTransportWriteTimeUtc) { continue }

            $candidatePaths = @(Get-PatchTouchedPaths $candidate)
            if ($candidatePaths.Count -eq 0) { continue }

            $covered = $true
            foreach ($rel in $candidatePaths) {
                if (-not $appliedSet.ContainsKey(([string]$rel).Replace('\','/').ToLowerInvariant())) {
                    $covered = $false
                    break
                }
            }
            if (-not $covered) { continue }

            # A still-applicable patch is not stale. Only archive a covered older
            # transport that no longer applies after the newer cumulative patch.
            $probe = Invoke-NativeProcess "superseded patch probe" "git" @(
                "apply","--no-index","--check","--whitespace=error-all",$candidate.FullName
            ) $false $false $false
            if ($probe.ExitCode -eq 0) { continue }

            New-Item -ItemType Directory -Force -Path $supersededDir | Out-Null
            $destination = Join-Path $supersededDir (
                "{0}-{1}" -f (Get-Date -Format "yyyyMMdd-HHmmssfff"), $candidate.Name
            )
            Move-Item -LiteralPath $candidate.FullName -Destination $destination -Force
            Log "PASS" "Archived superseded older root patch: $($candidate.Name)"
        } catch {
            Log "WARN" "Superseded-patch classification skipped for $($candidate.Name): $($_.Exception.Message)"
        }
    }
}

function Move-PatchToRebaseNeeded {
    param(
        [Parameter(Mandatory=$true)][IO.FileInfo]$Patch,
        [Parameter(Mandatory=$true)][string]$Reason,
        [string[]]$TouchedPaths = @()
    )

    $rebaseDir = Join-Path $Root "artifacts\patches\rebase-needed"
    New-Item -ItemType Directory -Force -Path $rebaseDir | Out-Null

    $stamp = Get-Date -Format "yyyyMMdd-HHmmssfff"
    $destination = Join-Path $rebaseDir ("{0}-{1}" -f $stamp, $Patch.Name)
    $sha = (Get-FileHash -LiteralPath $Patch.FullName -Algorithm SHA256).Hash.ToLowerInvariant()

    Move-Item -LiteralPath $Patch.FullName -Destination $destination -Force

    $recordPath = $destination + ".rebase.json"
    [ordered]@{
        schema = "forge.patch.rebase_needed.v1"
        file = $Patch.Name
        sha256 = $sha
        detectedAt = (Get-Date).ToString("o")
        sourceFingerprint = (Get-SourceFingerprint)
        touchedPaths = @($TouchedPaths)
        reason = $Reason
        archivedPatch = (Get-RelativePath $destination)
        disposition = "preserved-for-rebase"
    } | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath $recordPath -Encoding UTF8

    Log "WARN" "Patch no longer applies to current source and was preserved for rebase: $($Patch.Name)"
    Log "INFO" "Rebase-needed patch: $(Get-RelativePath $destination)"
    return $destination
}

function Apply-Patch {
    param([IO.FileInfo]$Patch)
    $sha = (Get-FileHash -LiteralPath $Patch.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    $transportWriteTimeUtc = $Patch.LastWriteTimeUtc
    Log "INFO" "Validating patch: $($Patch.Name) sha256=$sha"

    $touched = @(Get-PatchTouchedPaths $Patch)
    $transactionPaths = @(Get-PatchTransactionBackupPaths $touched)
    Log "INFO" "Patch touched paths: $($touched.Count); transactional safety paths: $($transactionPaths.Count)"

    $preflight = Invoke-NativeResult "git apply --check $($Patch.Name)" "git" @(
        "apply","--no-index","--check","--whitespace=error-all",$Patch.FullName
    )
    if ($preflight.ExitCode -ne 0) {
        $tail = Get-NativeFailureTail $preflight.Output 32
        if ([string]::IsNullOrWhiteSpace($tail)) {
            $tail = "git apply --check failed with exit code $($preflight.ExitCode)"
        }

        $conflictLike = (
            $preflight.Output -match '(?im)^error:\s+patch failed:' -or
            $preflight.Output -match '(?im)^error:\s+.+patch does not apply'
        )
        if (-not $conflictLike) {
            throw "git apply preflight failed without a source-conflict signature; patch remains pending.`n$tail"
        }

        $reason = "Patch is stale against the current governed source and requires rebase.`n$tail"
        $archived = Move-PatchToRebaseNeeded $Patch $reason $touched
        throw ("REBASE_REQUIRED::{0}::{1}" -f $Patch.Name, (Get-RelativePath $archived))
    }
    Log "PASS" "git apply --check $($Patch.Name)"

    $id = "PATCH-" + (Get-Date -Format "yyyyMMdd-HHmmss") + "-" + $sha.Substring(0,8)
    $txn = Join-Path $Root "artifacts\patches\transactions\$id"
    New-Item -ItemType Directory -Force -Path $txn | Out-Null
    Copy-Item -LiteralPath $Patch.FullName -Destination (Join-Path $txn $Patch.Name) -Force
    Backup-PatchTouchedPaths $transactionPaths $txn

    $archive = Join-Path $Root "artifacts\patches\applied"
    New-Item -ItemType Directory -Force -Path $archive | Out-Null
    $archivedPatch = Join-Path $archive $Patch.Name

    try {
        Run-Native "apply patch $($Patch.Name)" "git" @(
            "apply","--no-index","--whitespace=error-all",$Patch.FullName
        )

        # Formatting, lock synchronization, and governance are part of patch
        # application. Users should never need a separate rustfmt/lock repair step.
        $normalization = Invoke-PostPatchNormalization $touched $false

        Move-Item -LiteralPath $Patch.FullName -Destination $archivedPatch -Force

        $postImages = @(Get-PatchPostImages $touched)
        $transactionPostImages = @(Get-PatchPostImages $transactionPaths)
        [ordered]@{
            schema = "forge.patch.receipt.v3"
            id = $id
            file = $Patch.Name
            sha256 = $sha
            touchedPaths = $touched
            transactionPaths = $transactionPaths
            postImages = $postImages
            transactionPostImages = $transactionPostImages
            normalization = [ordered]@{
                rustfmt = [bool]$normalization.rustfmt
                cargoLock = [bool]$normalization.cargoLock
                manifestState = [string]$normalization.manifestState
            }
            appliedAt = (Get-Date).ToString("o")
            rolledBackAt = $null
        } | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $txn "receipt.json") -Encoding UTF8

        # Reconcile only verified post-images for the actual patch paths. The full
        # manifest was already regenerated if normalization changed source.
        Reconcile-PackageManifestFromAppliedReceipts

        Write-NeedsGateState "Patch applied and normalized automatically; explicit Full Gate is required." $id $true
        Resolve-SupersededRootPatches $touched $transportWriteTimeUtc
    } catch {
        try { Restore-PatchTransaction $txn } catch { Log "FAIL" "Automatic patch rollback also failed: $($_.Exception.Message)" }
        if ((Test-Path -LiteralPath $archivedPatch) -and -not (Test-Path -LiteralPath $Patch.FullName)) {
            Move-Item -LiteralPath $archivedPatch -Destination $Patch.FullName -Force -ErrorAction SilentlyContinue
        }
        throw
    }

    Clear-Green
    Log "PASS" "Patch applied, normalized, receipted, archived, and prior GREEN invalidated: $id"
}


function Stage-RootPatchArchive {
    $archives = @(Get-ChildItem -LiteralPath $Root -File -Filter "*.zip" -ErrorAction SilentlyContinue | Sort-Object Name)
    if ($archives.Count -eq 0) { return $false }
    $stagedAny = $false

    try {
        Add-Type -AssemblyName System.IO.Compression.FileSystem -ErrorAction Stop
    } catch {
        Log "WARN" "ZIP patch intake unavailable: $($_.Exception.Message)"
        return $false
    }

    foreach ($archiveFile in $archives) {
        $zip = $null
        $tempPatch = $null
        $leafName = $null

        try {
            $zip = [System.IO.Compression.ZipFile]::OpenRead($archiveFile.FullName)
            $patchEntries = @(
                $zip.Entries |
                Where-Object {
                    -not [string]::IsNullOrWhiteSpace($_.Name) -and
                    $_.Name.ToLowerInvariant().EndsWith(".patch")
                }
            )

            if ($patchEntries.Count -eq 0) { continue }
            if ($patchEntries.Count -ne 1) {
                Log "WARN" "Root ZIP ignored because it contains $($patchEntries.Count) patch payloads; exactly one is required: $($archiveFile.Name)"
                continue
            }

            $entry = $patchEntries[0]
            $leafName = [IO.Path]::GetFileName($entry.Name)
            if ([string]::IsNullOrWhiteSpace($leafName) -or -not $leafName.ToLowerInvariant().EndsWith(".patch")) {
                Log "WARN" "Root ZIP patch entry has an invalid file name and was ignored: $($archiveFile.Name)"
                continue
            }

            $stageDir = Join-Path $Root "artifacts\patches\intake-stage"
            New-Item -ItemType Directory -Force -Path $stageDir | Out-Null
            $tempPatch = Join-Path $stageDir (([Guid]::NewGuid().ToString("N")) + ".patch")
            [System.IO.Compression.ZipFileExtensions]::ExtractToFile($entry, $tempPatch, $false)
        } catch {
            Log "WARN" "Root ZIP was not staged as a patch transport: $($archiveFile.Name) :: $($_.Exception.Message)"
            continue
        } finally {
            # Windows will not allow Move-Item on an open ZIP. Always release the
            # FileStream before archiving or otherwise moving the transport.
            if ($null -ne $zip) {
                try { $zip.Dispose() } catch { }
                $zip = $null
            }
        }

        if ($null -eq $tempPatch -or -not (Test-Path -LiteralPath $tempPatch -PathType Leaf)) {
            continue
        }

        try {
            $incomingHash = (Get-FileHash -LiteralPath $tempPatch -Algorithm SHA256).Hash.ToLowerInvariant()
            $archiveDir = Join-Path $Root "artifacts\patches\transports"
            New-Item -ItemType Directory -Force -Path $archiveDir | Out-Null

            if (Test-PatchShaAlreadyApplied $incomingHash) {
                Remove-Item -LiteralPath $tempPatch -Force -ErrorAction SilentlyContinue
                $transportName = "{0}-{1}" -f (Get-Date -Format "yyyyMMdd-HHmmssfff"), $archiveFile.Name
                $transportDestination = Join-Path $archiveDir $transportName
                Move-Item -LiteralPath $archiveFile.FullName -Destination $transportDestination -Force
                Log "PASS" "Archived already-applied ZIP transport without restaging: $($archiveFile.Name) sha256=$incomingHash"
                $stagedAny = $true
                continue
            }

            $destination = Join-Path $Root $leafName
            if (Test-Path -LiteralPath $destination) {
                $existingHash = (Get-FileHash -LiteralPath $destination -Algorithm SHA256).Hash.ToLowerInvariant()
                if ($existingHash -ne $incomingHash) {
                    Remove-Item -LiteralPath $tempPatch -Force -ErrorAction SilentlyContinue
                    Log "FAIL" "ZIP patch intake collision: root patch $leafName already exists with different contents. Resolve manually before retrying."
                    return $false
                }
                Remove-Item -LiteralPath $tempPatch -Force -ErrorAction SilentlyContinue
                Log "INFO" "ZIP transport contains the same patch already staged at root: $leafName"
            } else {
                Move-Item -LiteralPath $tempPatch -Destination $destination -Force
                Log "PASS" "Staged root patch from ZIP transport: $($archiveFile.Name) -> $leafName sha256=$incomingHash"
            }

            $transportName = "{0}-{1}" -f (Get-Date -Format "yyyyMMdd-HHmmssfff"), $archiveFile.Name
            $transportDestination = Join-Path $archiveDir $transportName
            Move-Item -LiteralPath $archiveFile.FullName -Destination $transportDestination -Force
            Log "PASS" "Archived consumed ZIP transport: $(Get-RelativePath $transportDestination)"
            $stagedAny = $true
            continue
        } catch {
            if ($null -ne $tempPatch -and (Test-Path -LiteralPath $tempPatch)) {
                Remove-Item -LiteralPath $tempPatch -Force -ErrorAction SilentlyContinue
            }
            Log "WARN" "Root ZIP transport finalization failed: $($archiveFile.Name) :: $($_.Exception.Message)"
        }
    }

    return $stagedAny
}

function Test-PatchTouchesPccProvider {
    param([string[]]$Touched)
    foreach ($rel in @($Touched)) {
        $normalized = ([string]$rel).Replace('\','/').ToLowerInvariant()
        if ($normalized -eq 'tools/pcc/forgeguicontrol.ps1' -or
            $normalized -eq 'project_control_center.cmd' -or
            $normalized -eq 'project.control.json') {
            return $true
        }
    }
    return $false
}

function Restart-PccProvider {
    param([string]$OperationToResume = '')
    Log "INFO" "PCC provider source changed; reloading the updated provider automatically in the same console."

    $arguments = @(
        '-NoLogo',
        '-NoProfile',
        '-ExecutionPolicy', 'Bypass',
        '-File', $PSCommandPath
    )
    if (-not [string]::IsNullOrWhiteSpace($OperationToResume)) {
        $arguments += @('-Operation', $OperationToResume)
    }

    # Run the newly written provider as a child in the current console. The old
    # provider waits and then exits with the child's code, so users never need to
    # close/reopen the PCC manually and no second console window is created.
    & powershell.exe @arguments
    $childExit = $LASTEXITCODE
    if ($null -eq $childExit) { $childExit = 0 }
    exit $childExit
}

function Test-PatchAppliesCleanly {
    param([IO.FileInfo]$Patch)
    $probe = Invoke-NativeProcess "patch preflight probe" "git" @(
        "apply","--no-index","--check","--whitespace=error-all",$Patch.FullName
    ) $false $false $false
    return ($probe.ExitCode -eq 0)
}

function Resolve-PreflightSupersededRootPatches {
    $patches = @(Get-ChildItem -LiteralPath $Root -File -Filter "*.patch" -ErrorAction SilentlyContinue)
    if ($patches.Count -lt 2) { return }

    # Find the newest patch that cleanly applies to the current governed source.
    # It may be a cumulative/rebased replacement for older root transports that
    # can no longer apply. We only auto-archive an older failing patch when every
    # path it touches is covered by that newer viable patch.
    $viable = @()
    foreach ($candidate in @($patches | Sort-Object @{Expression={$_.LastWriteTimeUtc}; Descending=$true}, Name)) {
        if (-not (Test-PatchAppliesCleanly $candidate)) { continue }
        $paths = @(Get-PatchTouchedPaths $candidate)
        if ($paths.Count -eq 0) { continue }
        $viable += [pscustomobject]@{
            Patch = $candidate
            Paths = $paths
            PathCount = $paths.Count
        }
    }
    if ($viable.Count -eq 0) { return }

    $authority = @($viable | Sort-Object @{Expression={$_.Patch.LastWriteTimeUtc}; Descending=$true}, @{Expression={$_.PathCount}; Descending=$true}, @{Expression={$_.Patch.Name}; Descending=$true})[0]
    $authoritySet = @{}
    foreach ($rel in @($authority.Paths)) {
        $authoritySet[([string]$rel).Replace('\','/').ToLowerInvariant()] = $true
    }

    $supersededDir = Join-Path $Root "artifacts\patches\superseded"
    foreach ($candidate in @($patches | Sort-Object Name)) {
        if ($candidate.FullName -eq $authority.Patch.FullName) { continue }
        if ($candidate.LastWriteTimeUtc -gt $authority.Patch.LastWriteTimeUtc) { continue }

        $candidatePaths = @(Get-PatchTouchedPaths $candidate)
        if ($candidatePaths.Count -eq 0) { continue }

        $covered = $true
        foreach ($rel in $candidatePaths) {
            $key = ([string]$rel).Replace('\','/').ToLowerInvariant()
            if (-not $authoritySet.ContainsKey($key)) {
                $covered = $false
                break
            }
        }
        if (-not $covered) { continue }
        if (Test-PatchAppliesCleanly $candidate) { continue }

        New-Item -ItemType Directory -Force -Path $supersededDir | Out-Null
        $destination = Join-Path $supersededDir (
            "{0}-{1}" -f (Get-Date -Format "yyyyMMdd-HHmmssfff"), $candidate.Name
        )
        Move-Item -LiteralPath $candidate.FullName -Destination $destination -Force
        Log "PASS" "Archived stale root patch superseded by $($authority.Patch.Name): $($candidate.Name)"
    }
}

function Scan-Patches {
    Resolve-AlreadyAppliedRootPatches
    Resolve-PreflightSupersededRootPatches

    while ($true) {
        Resolve-PreflightSupersededRootPatches
        $patches = @(Get-ChildItem -LiteralPath $Root -File -Filter "*.patch" | Sort-Object Name)
        if ($patches.Count -eq 0) { return }

        $patch = $patches[0]
        $answer = Read-Host "Patch found: $($patch.Name)`nApply this patch now? [y/N]"
        if ($answer -notmatch '^(y|yes)$') {
            Log "INFO" "Patch declined; ordered intake stops with the remaining queue untouched."
            return
        }

        $touched = @(Get-PatchTouchedPaths $patch)
        $providerTouched = Test-PatchTouchesPccProvider $touched

        try {
            Apply-Patch $patch
            Log "PASS" "Patch applied and normalized without requiring a manual PCC restart."

            if ($providerTouched) {
                Restart-PccProvider $Operation
            }

            Resolve-AlreadyAppliedRootPatches
            $remaining = @(Get-ChildItem -LiteralPath $Root -File -Filter "*.patch" | Sort-Object Name)
            if ($remaining.Count -eq 0) {
                Log "INFO" "Patch queue drained. The current PCC session remains active in NEEDS_GATE state."
                return
            }

            $continue = Read-Host "$($remaining.Count) ordered patch(es) remain. Continue patch intake now? [Y/n]"
            if ($continue -match '^(n|no)$') {
                Log "INFO" "Patch queue paused by user; no restart is required."
                return
            }
        } catch {
            $message = [string]$_.Exception.Message
            if ($message.StartsWith("REBASE_REQUIRED::", [StringComparison]::Ordinal)) {
                $parts = @($message -split "::", 3)
                $archivePath = if ($parts.Count -ge 3) { $parts[2] } else { "artifacts/patches/rebase-needed" }
                $reason = "Patch requires rebase and has been removed from active intake: $($patch.Name) -> $archivePath"
                Log "WARN" $reason
                [void](Package-Debug $reason "PATCH_REBASE" $true)
                return
            }

            $reason = "Patch intake failed; ordered intake stops and patch remains pending: $($patch.Name) :: $message"
            Log "FAIL" $reason
            [void](Package-Debug $reason "PATCH_FAIL" $true)
            return
        }
    }
}

function Root-Hygiene {
    $script = Join-Path $Root "tools\pcc\ForgeGuiRootHygiene.ps1"
    if (-not (Test-Path -LiteralPath $script)) {
        Log "WARN" "Root hygiene provider missing: $script"
        return
    }

    try {
        & powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File $script -Root $Root
        $code = $LASTEXITCODE
        if ($null -ne $code -and $code -ne 0) {
            Log "WARN" "Root hygiene exited $code (non-fatal)"
        }
    } catch {
        Log "WARN" "Root hygiene failed non-fatally: $($_.Exception.Message)"
    }
}

function Test-PccSelf {
    $probe = @("alpha","beta")
    $roundTrip = $probe | ConvertTo-Json -Compress | ConvertFrom-Json
    if (@($roundTrip).Count -ne 2) { throw "Internal PCC self-test failed: array/JSON persistence" }
    $native = Invoke-NativeProcess "PCC native argument self-test" "cmd.exe" @("/d","/c","exit","0") $false $false $false
    if ($native.ExitCode -ne 0) { throw "Internal PCC self-test failed: native argument/exit-code handling" }
    Log "PASS" "Internal PCC self-test (arrays / JSON / native argument forwarding)"
}

function Test-PackageManifest {
    $manifestPath = Join-Path $Root "PACKAGE_MANIFEST.json"
    if (-not (Test-Path -LiteralPath $manifestPath)) { throw "PACKAGE_MANIFEST.json is missing" }
    $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
    if ($manifest.schema -ne "forgegui.package_manifest.v1") { throw "Unsupported package manifest schema: $($manifest.schema)" }

    $declared = @{}
    foreach ($entry in @($manifest.files)) {
        $rel = ([string]$entry.path).Replace('\','/')
        $key = $rel.ToLowerInvariant()
        if ($declared.ContainsKey($key)) { throw "Package manifest contains duplicate path: $rel" }
        $declared[$key] = $true
        $path = Join-Path $Root $rel.Replace('/', '\')
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Package manifest file missing: $rel" }
        $file = Get-Item -LiteralPath $path
        if ([int64]$entry.bytes -ne [int64]$file.Length) { throw "Package manifest byte-size mismatch: $rel" }
        $hash = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($hash -ne ([string]$entry.sha256).ToLowerInvariant()) { throw "Package manifest hash mismatch: $rel" }
    }

    $governed = @(Get-GovernedPackageFilePaths)
    foreach ($rel in $governed) {
        if (-not $declared.ContainsKey($rel.ToLowerInvariant())) {
            throw "Package manifest coverage gap: governed file is not declared: $rel"
        }
    }
    if ($declared.Count -ne $governed.Count) {
        throw "Package manifest coverage mismatch: declared=$($declared.Count) governed=$($governed.Count)"
    }

    Log "PASS" "PACKAGE_MANIFEST.json / $($declared.Count) governed file(s), complete coverage"
}

function Update-PackageManifestHashes {
    $manifestPath = Join-Path $Root "PACKAGE_MANIFEST.json"
    if (-not (Test-Path -LiteralPath $manifestPath)) { throw "PACKAGE_MANIFEST.json is missing" }

    $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
    $newEntries = @()
    foreach ($rel in @(Get-GovernedPackageFilePaths)) {
        $path = Join-Path $Root $rel.Replace('/', '\')
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
            throw "Cannot refresh manifest; governed file disappeared: $rel"
        }
        $file = Get-Item -LiteralPath $path
        $newEntries += [pscustomobject][ordered]@{
            path = $rel
            bytes = [int64]$file.Length
            sha256 = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
        }
    }

    $manifest.files = @($newEntries)
    if ($manifest.PSObject.Properties.Name -contains "file_count") {
        $manifest.file_count = @($newEntries).Count
    } else {
        $manifest | Add-Member -NotePropertyName file_count -NotePropertyValue @($newEntries).Count
    }
    $manifest.format_state = "canonical"

    $json = $manifest | ConvertTo-Json -Depth 8
    [IO.File]::WriteAllText($manifestPath, $json + [Environment]::NewLine, $script:Utf8NoBom)
    Log "PASS" "PACKAGE_MANIFEST.json regenerated for $(@($newEntries).Count) governed file(s)"
}

function Ensure-CanonicalFormat {
    # Full Gate is read-only for normal development. A pristine full-source package
    # may be delivered before rustfmt is available in the artifact builder; because
    # Test-PackageManifest has already proven every governed byte matches the package,
    # one bootstrap-only canonicalization is safe. Arbitrary edited source cannot
    # reach this repair path because it fails manifest verification first.
    $check = Invoke-NativeResult "cargo fmt --check / bootstrap probe" "cargo" @("fmt","--all","--check")
    if ($check.ExitCode -eq 0) {
        Log "PASS" "cargo fmt --check"
        return
    }

    $manifest = Get-Content -LiteralPath (Join-Path $Root "PACKAGE_MANIFEST.json") -Raw | ConvertFrom-Json
    if ([string]$manifest.format_state -ne "requires-canonicalization") {
        $tail = Get-NativeFailureTail $check.Output
        throw "cargo fmt --check failed on canonical source. Use Advanced -> Format source / rustfmt repair explicitly.`n$tail"
    }
    Log "WARN" "Pristine recovery package requires its one-time manifest-authorized rustfmt canonicalization before certification."
    Run-Native "bootstrap cargo fmt apply" "cargo" @("fmt","--all")
    Update-PackageManifestHashes
    Test-PackageManifest
    Run-Native "cargo fmt --check" "cargo" @("fmt","--all","--check")
}

function Ensure-CargoLock {
    if (-not (Test-Path -LiteralPath (Join-Path $Root "Cargo.lock"))) {
        Log "INFO" "Cargo.lock is absent; generating a reproducible workspace lockfile."
        Run-Native "generate Cargo.lock" "cargo" @("generate-lockfile")
    }
}

function Write-GreenReceipt {
    param([string]$GateId)
    $fingerprint = Get-SourceFingerprint
    $gitHead = Capture-Native "git" @("rev-parse","HEAD")
    $gitBranch = Capture-Native "git" @("branch","--show-current")

    $receipt = [ordered]@{
        schema = "forge.green.v1"
        project = "ForgeGUI_Core"
        version = "0.4.8"
        gateId = $GateId
        certifiedAt = (Get-Date).ToString("o")
        sourceFingerprint = $fingerprint
        gitCommit = $gitHead
        gitBranch = $gitBranch
        gitRemote = $null
        pushedAt = $null
        operations = @(
            "cargo fmt --all --check",
            "cargo check --locked --workspace --all-targets (stale-lock repair allowed only on explicit Cargo diagnostic)",
            "cargo metadata --locked --format-version 1 --no-deps",
            "cargo test --locked --workspace --all-targets",
            "cargo clippy --locked --workspace --all-targets -- -D warnings",
            "cargo build --locked --release -p forge_gui_lab",
            "native tool PASS/FAIL determined exclusively by process exit code"
        )
    }

    $receipt | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $GreenPath -Encoding UTF8
    Log "PASS" "GREEN receipt: $GreenPath"
}

function Full-Gate {
    Log "INFO" "Operation started: gate.full"
    Clear-Green
    $gateId = "QG-" + (Get-Date -Format "yyyyMMdd-HHmmss") + "-full-" + ([guid]::NewGuid().ToString("N").Substring(0,8))

    try {
        $pendingCount = @(Get-ChildItem -LiteralPath $Root -File -Filter "*.patch" -ErrorAction SilentlyContinue).Count
        if ($pendingCount -gt 0) {
            throw "Full Gate blocked: $pendingCount root patch transport(s) are still pending."
        }
        $patchState = Read-NeedsGateState
        if ($null -ne $patchState -and -not [bool]$patchState.normalized) {
            throw "Full Gate blocked: patch/update normalization has not completed."
        }

        Get-Content -LiteralPath (Join-Path $Root "project.control.json") -Raw | ConvertFrom-Json | Out-Null
        Log "PASS" "project.control.json"
        Test-PccSelf
        Test-PackageManifest

        Run-Native "Rust toolchain" "rustc" @("--version")
        Run-Native "Cargo toolchain" "cargo" @("--version")
        Ensure-CargoLock

        # Certification is read-only after a manifest-proven package bootstrap.
        Ensure-CanonicalFormat

        # The full all-target graph is the authoritative lock freshness test.
        # A stale lock is repaired only for Cargo's explicit lock-refresh diagnostic.
        Run-LockedWorkspaceCheck
        Run-Native "Cargo workspace metadata / locked" "cargo" @("metadata","--locked","--format-version","1","--no-deps")
        Run-Native "cargo test workspace" "cargo" @("test","--locked","--workspace","--all-targets")
        Run-Native "cargo clippy workspace" "cargo" @("clippy","--locked","--workspace","--all-targets","--","-D","warnings")
        Run-Native "release build canonical GUI Lab" "cargo" @("build","--locked","--release","-p","forge_gui_lab")

        $releaseExe = Join-Path $Root "target\release\forge_gui_lab.exe"
        if (-not (Test-Path -LiteralPath $releaseExe)) {
            throw "release build completed but canonical GUI Lab executable is missing: $releaseExe"
        }
        Log "PASS" "Canonical GUI Lab executable present: $releaseExe"

        Write-GreenReceipt $gateId
        Clear-NeedsGateState
        Log "PASS" "FULL QUALITY GATE GREEN :: $gateId"
        [void](Package-Debug "FULL QUALITY GATE GREEN :: $gateId" "PASS" $false)
        return $true
    } catch {
        Clear-Green
        $reason = $_.Exception.Message
        Log "FAIL" "FULL QUALITY GATE FAILED :: $gateId :: $reason"
        [void](Package-Debug $reason "FAIL" $true)
        return $false
    }
}

function Fmt-Fix {
    Run-Native "cargo fmt apply" "cargo" @("fmt","--all")
    Run-Native "cargo fmt check" "cargo" @("fmt","--all","--check")
    Update-PackageManifestHashes
    Clear-Green
    Write-NeedsGateState "Manual rustfmt repair changed/revalidated governed source; explicit Full Gate is required." $null $true
    Log "INFO" "Source formatting changed or was revalidated; run Full Gate before commit/push."
}

function Run-Lab {
    Assert-RunReady

    $generation = Get-LabSourceGeneration
    if ($generation -ne "M1-M6 modular shell") {
        Log "WARN" "ForgeGUI Lab source does not report the M1-M6 modular shell marker. Running current source anyway so the mismatch is visible."
    }

    # Source is authoritative: cargo run rebuilds the current Lab when required and
    # cannot silently launch a stale pre-patch executable.
    Run-Native "ForgeGUI Universal Modular Lab" "cargo" @("run","--locked","-p","forge_gui_lab")
}

function Patch-Status {
    $pending = @(Get-ChildItem -LiteralPath $Root -File -Filter "*.patch" | Sort-Object Name)
    Write-Host "Pending root patches: $($pending.Count)"
    foreach ($patch in $pending) { Write-Host "  - $($patch.Name)" }

    $zipTransports = @(Get-ChildItem -LiteralPath $Root -File -Filter "*.zip" -ErrorAction SilentlyContinue | Sort-Object Name)
    Write-Host "Root ZIP transports: $($zipTransports.Count)"
    foreach ($transport in $zipTransports) { Write-Host "  - $($transport.Name)" }

    $rebaseDir = Join-Path $Root "artifacts\patches\rebase-needed"
    $rebaseNeeded = @()
    if (Test-Path -LiteralPath $rebaseDir) {
        $rebaseNeeded = @(Get-ChildItem -LiteralPath $rebaseDir -File -Filter "*.patch" -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending)
    }
    Write-Host "Rebase-needed patches: $($rebaseNeeded.Count)"
    foreach ($candidate in ($rebaseNeeded | Select-Object -First 10)) {
        Write-Host "  - $(Get-RelativePath $candidate.FullName)"
    }

    $transactions = Join-Path $Root "artifacts\patches\transactions"
    if (Test-Path -LiteralPath $transactions) {
        $receipts = @(Get-ChildItem -LiteralPath $transactions -Recurse -File -Filter "receipt.json" | Sort-Object LastWriteTime -Descending)
        Write-Host "Receipts: $($receipts.Count)"
        foreach ($receipt in ($receipts | Select-Object -First 10)) {
            Write-Host "  - $(Get-RelativePath $receipt.FullName)"
        }
    } else {
        Write-Host "Receipts: 0"
    }
}

function Status {
    Write-Host "Repository : $Root"
    Write-Host "Git        : $(Get-GitStatusText)"
    Write-Host "Gate       : $(Get-GreenStatus)"
    Write-Host "PatchState : $(Get-PatchStateText)"
    Write-Host "Patches    : $(@(Get-ChildItem -LiteralPath $Root -File -Filter '*.patch').Count) root .patch file(s)"
    Write-Host "PowerShell : $($PSVersionTable.PSVersion)"
    Write-Host "Authority  : Project-owned PCC / forge.project.v1"
    Write-Host "Provider   : forge.internal_pcc.v1 / $ProviderVersion"
    Write-Host "Intake     : all root ZIPs staged + superseded cleanup + ordered queue + same-console provider reload"
}

function Get-ProviderOperations {
    return @(
        "project.status","project.health","provider.capabilities","provider.operations",
        "gate.full","git.commit_push_green","run.lab","patch.scan","patch.status",
        "debug.bundle","fmt.fix","root.hygiene","git.setup","handoff.open",
        "cargo.lock.sync","patch.rollback_last"
    )
}

function Get-ProviderCapabilities {
    return @(
        "forge.internal_pcc.v1","forge.project.v1","forge.patch.v1",
        "forge.project_status.v1","forge.project_health.v1","forge.quality_gate.v1",
        "standalone-source","root-patch-intake","root-zip-patch-intake","transactional-patch-backup",
        "patch-receipts","patch-rollback","receipt-manifest-reconcile","duplicate-transport-suppression","superseded-transport-classification","rebase-needed-patch-quarantine","debug-patch-evidence","debug-current-source-rebase-evidence","auto-debug-handoff","green-commit-push",
        "automatic-post-patch-rustfmt","controlled-post-patch-lock-refresh","patch-needs-gate-state","run-green-guard",
        "ordered-patch-queue","stage-all-root-zip-transports","preflight-superseded-patch-cleanup","in-process-patch-continue","same-console-provider-reload","automatic-main-menu-return","source-authoritative-lab-run","lab-generation-status","package-manifest-verification","exit-code-native-authority",
        "provider-json-discovery","structural-rails","center-dock-workspace",
        "forgegui-sdk","infinite-canvas","pie-contract","notifications",
        "notification-center","embedded-console","semantic-icons","virtual-tables","oss-smoke-certification"
    )
}

function Write-ProviderStatusJson {
    $pending = @((Get-ChildItem -LiteralPath $Root -File -Filter '*.patch' -ErrorAction SilentlyContinue) | ForEach-Object { $_.Name })
    $gitStatus = Get-GitStatusText
    $greenStatus = Get-GreenStatus
    $fingerprint = Get-SourceFingerprint
    [ordered]@{
        schema = "forge.project_status.v1"
        project = "ForgeGUI_Core"
        projectId = "forgegui.core"
        version = "0.4.8"
        root = $Root
        provider = [ordered]@{ id="forge.internal_pcc.v1"; version=$ProviderVersion }
        git = $gitStatus
        green = $greenStatus
        patchState = Get-PatchStateText
        pendingPatches = $pending
        sourceFingerprint = $fingerprint
    } | ConvertTo-Json -Depth 7
}

function Write-ProviderHealthJson {
    $green = Read-Green
    $fingerprint = Get-SourceFingerprint
    $pendingCount = @(Get-ChildItem -LiteralPath $Root -File -Filter '*.patch' -ErrorAction SilentlyContinue).Count
    $greenCurrent = $false
    if ($null -ne $green) { $greenCurrent = ([string]$green.sourceFingerprint -eq $fingerprint) }
    $patchState = Read-NeedsGateState
    $state = if ($pendingCount -gt 0) { "attention" } elseif ($null -ne $patchState) { "needs-gate" } elseif ($greenCurrent) { "green" } elseif ($null -ne $green) { "stale-green" } else { "uncertified" }
    $handoff = $null
    if (Test-Path -LiteralPath $LatestHandoffPath) { $handoff = Get-RelativePath $LatestHandoffPath }
    [ordered]@{
        schema = "forge.project_health.v1"
        project = "ForgeGUI_Core"
        version = "0.4.8"
        state = $state
        greenCurrent = $greenCurrent
        patchState = if ($null -ne $patchState) { [string]$patchState.state } else { "READY" }
        pendingPatchCount = $pendingCount
        handoff = $handoff
    } | ConvertTo-Json -Depth 6
}

function Write-ProviderCapabilitiesJson {
    [ordered]@{ schema="forge.provider_capabilities.v1"; provider="forge.internal_pcc.v1"; version=$ProviderVersion; capabilities=@(Get-ProviderCapabilities) } | ConvertTo-Json -Depth 6
}

function Write-ProviderOperationsJson {
    [ordered]@{ schema="forge.provider_operations.v1"; provider="forge.internal_pcc.v1"; version=$ProviderVersion; operations=@(Get-ProviderOperations) } | ConvertTo-Json -Depth 6
}

function Project-Health {
    Status
    Write-Host ""
    Write-Host "Paths:"
    Write-Host "  Session log : $LogPath"
    Write-Host "  Handoff dir : $HandoffDir"
    if (Test-Path -LiteralPath $LatestHandoffPath) {
        try {
            $latest = Get-Content -LiteralPath $LatestHandoffPath -Raw | ConvertFrom-Json
            Write-Host "  Latest handoff : $($latest.bundle)"
        } catch {
            Write-Host "  Latest handoff : metadata unreadable"
        }
    } else {
        Write-Host "  Latest handoff : none"
    }
    Write-Host "  GREEN state : $GreenPath"
    Write-Host "  Cargo.lock  : $(if (Test-Path -LiteralPath (Join-Path $Root 'Cargo.lock')) { 'present' } else { 'missing (generated by gate/run)' })"
    Write-Host "  Release EXE : $(if (Test-Path -LiteralPath (Join-Path $Root 'target\release\forge_gui_lab.exe')) { 'present' } else { 'not built' })"
}

function Ensure-GitSetup {
    $inside = Capture-Native "git" @("rev-parse","--is-inside-work-tree")
    if ($inside -ne "true") {
        $answer = Read-Host "No Git repository exists in this extracted source. Initialize one here? [y/N]"
        if ($answer -notmatch '^(y|yes)$') { return $false }
        try {
            Run-Native "git init" "git" @("init","-b","main")
        } catch {
            Run-Native "git init" "git" @("init")
            Run-Native "git branch main" "git" @("branch","-M","main")
        }
    }

    $remotesText = Capture-Native "git" @("remote")
    $remotes = @()
    if (-not [string]::IsNullOrWhiteSpace($remotesText)) {
        $remotes = @($remotesText -split "`r?`n" | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    }

    if ($remotes.Count -eq 0) {
        $url = Read-Host "No Git remote is configured. Paste repository URL or press Enter for $CanonicalRepository"
        if ([string]::IsNullOrWhiteSpace($url)) { $url = $CanonicalRepository }
        Run-Native "git remote add origin" "git" @("remote","add","origin",$url.Trim())
    }

    return $true
}

function Ensure-GitIdentity {
    $name = Capture-Native "git" @("config","user.name")
    $email = Capture-Native "git" @("config","user.email")

    if ([string]::IsNullOrWhiteSpace($name)) {
        $name = Read-Host "Git user.name is not configured. Enter the commit author name (blank cancels)"
        if ([string]::IsNullOrWhiteSpace($name)) { return $false }
        Run-Native "configure git user.name" "git" @("config","user.name",$name.Trim())
    }

    if ([string]::IsNullOrWhiteSpace($email)) {
        $email = Read-Host "Git user.email is not configured. Enter the commit author email (blank cancels)"
        if ([string]::IsNullOrWhiteSpace($email)) { return $false }
        Run-Native "configure git user.email" "git" @("config","user.email",$email.Trim())
    }

    return $true
}

function Commit-Push-Green {
    Log "INFO" "Operation started: git.commit_push_green"
    $green = Read-Green
    if ($null -eq $green) {
        Log "FAIL" "No certified GREEN state exists. Run option 1 first."
        return $false
    }

    $pending = @(Get-ChildItem -LiteralPath $Root -File -Filter "*.patch")
    if ($pending.Count -gt 0) {
        Log "FAIL" "Commit/push blocked: $($pending.Count) root patch transport(s) are still pending."
        return $false
    }

    $currentFingerprint = Get-SourceFingerprint
    if ($currentFingerprint -ne [string]$green.sourceFingerprint) {
        Log "FAIL" "Commit/push blocked: source changed after GREEN certification. Run option 1 again."
        return $false
    }

    if (-not (Ensure-GitSetup)) {
        Log "FAIL" "Commit/push cancelled because Git repository/remote setup is incomplete."
        return $false
    }
    if (-not (Ensure-GitIdentity)) {
        Log "FAIL" "Commit/push cancelled because Git commit identity is incomplete."
        return $false
    }

    # .git creation/configuration is intentionally outside the source fingerprint.
    $afterSetupFingerprint = Get-SourceFingerprint
    if ($afterSetupFingerprint -ne [string]$green.sourceFingerprint) {
        Log "FAIL" "Commit/push blocked: source changed during Git setup. Run option 1 again."
        return $false
    }

    $branch = Capture-Native "git" @("branch","--show-current")
    if ([string]::IsNullOrWhiteSpace($branch)) {
        $branch = "main"
        Run-Native "select main branch" "git" @("branch","-M",$branch)
    }

    $remotesText = Capture-Native "git" @("remote")
    $remoteList = @($remotesText -split "`r?`n" | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    $remote = if ($remoteList -contains "origin") { "origin" } elseif ($remoteList.Count -gt 0) { $remoteList[0] } else { $null }
    if ([string]::IsNullOrWhiteSpace($remote)) {
        Log "FAIL" "Commit/push blocked: no Git remote is configured."
        return $false
    }

    Run-Native "git add" "git" @("add","-A")
    $staged = Capture-Native "git" @("diff","--cached","--name-only")
    if (-not [string]::IsNullOrWhiteSpace($staged)) {
        $message = "ForgeGUI GREEN $($green.gateId)"
        Run-Native "git commit GREEN" "git" @("commit","-m",$message)
    } else {
        Log "INFO" "No new source changes to commit; pushing currently certified HEAD."
    }

    $upstream = Capture-Native "git" @("rev-parse","--abbrev-ref","--symbolic-full-name","@{u}")
    if ([string]::IsNullOrWhiteSpace($upstream)) {
        Run-Native "git push / establish upstream" "git" @("push","-u",$remote,$branch)
    } else {
        Run-Native "git push" "git" @("push")
    }

    $green.gitCommit = Capture-Native "git" @("rev-parse","HEAD")
    $green.gitBranch = $branch
    $green.gitRemote = $remote
    $green.pushedAt = (Get-Date).ToString("o")
    $green | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $GreenPath -Encoding UTF8

    Log "PASS" "CURRENT GREEN committed and pushed."
    return $true
}

function Show-GreenReceipt {
    if (Test-Path -LiteralPath $GreenPath) {
        Get-Content -LiteralPath $GreenPath
    } else {
        Write-Host "No GREEN receipt exists."
    }
}

function Git-Setup {
    if (Ensure-GitSetup) {
        Write-Host "Git: $(Get-GitStatusText)"
        $remotes = Capture-Native "git" @("remote","-v")
        if (-not [string]::IsNullOrWhiteSpace($remotes)) { Write-Host $remotes }
    }
}


function Get-LabSourceGeneration {
    $labPath = Join-Path $Root "examples\forge_gui_lab\src\main.rs"
    if (-not (Test-Path -LiteralPath $labPath -PathType Leaf)) { return "MISSING" }
    $text = Get-Content -LiteralPath $labPath -Raw
    if ($text -match 'Universal modular ForgeGUI shell ready' -and
        $text -match 'workspace\.application' -and
        $text -match 'show_modular_surfaces') {
        return "M1-M6 modular shell"
    }
    return "legacy / modular pass not present"
}

function Return-ToMainMenu {
    param([string]$Completed = "")
    if ([string]::IsNullOrWhiteSpace($Completed)) { return }
    $script:LastCompletedAction = $Completed
    Log "INFO" "Action complete; returning to main menu: $Completed"
}

function Advanced-Menu {
    Write-Host ""
    Write-Host "------------------------------------------------------------------------"
    Write-Host " ForgeGUI Advanced / Provider Operations"
    Write-Host "------------------------------------------------------------------------"
    Write-Host " 1. Format source / rustfmt repair (manual fallback)"
    Write-Host " 2. Root hygiene"
    Write-Host " 3. Show current GREEN receipt"
    Write-Host " 4. Git setup / remote status"
    Write-Host " 5. Open handoff folder"
    Write-Host " 6. Synchronize Cargo.lock (manual fallback)"
    Write-Host " 7. Roll back last applied patch"
    Write-Host " 8. Scan / apply pending patch queue"
    Write-Host " 0. Back"
    $choice = Read-Host "Select"

    switch ($choice) {
        "1" { try { Fmt-Fix } catch { Log "FAIL" $_.Exception.Message } }
        "2" { Root-Hygiene }
        "3" { Show-GreenReceipt }
        "4" { try { Git-Setup } catch { Log "FAIL" $_.Exception.Message } }
        "5" { Open-HandoffFolder }
        "6" {
            try {
                Run-Native "synchronize Cargo.lock" "cargo" @("generate-lockfile")
                Run-Native "validate synchronized Cargo.lock" "cargo" @("metadata","--locked","--format-version","1","--no-deps")
                Clear-Green
                Write-NeedsGateState "Manual Cargo.lock synchronization changed/revalidated dependency state; explicit Full Gate is required." $null $true
                Log "INFO" "Cargo.lock synchronized; run Full Gate before commit/push."
            } catch { Log "FAIL" $_.Exception.Message }
        }
        "7" { try { Rollback-LastPatch } catch { Log "FAIL" $_.Exception.Message } }
        "8" {
            [void](Stage-RootPatchArchive)
            Scan-Patches
        }
        "0" { return }
        default { Write-Host "Unknown option" }
    }
}

function Menu {
    while ($true) {
        try { Clear-Host } catch { }

        Write-Host "========================================================================"
        Write-Host " FORGEGUI_CORE PROJECT CONTROL CENTER"
        Write-Host "========================================================================"
        Status
        if (-not [string]::IsNullOrWhiteSpace($script:LastCompletedAction)) {
            Write-Host "LastAction : $script:LastCompletedAction" -ForegroundColor Green
        }
        Write-Host "LabSource  : $(Get-LabSourceGeneration)"
        Write-Host "RunMode    : cargo run --locked -p forge_gui_lab (source-authoritative)"
        Write-Host "------------------------------------------------------------------------"
        Write-Host " 1. FULL QUALITY GATE / CERTIFY GREEN"
        Write-Host " 2. COMMIT + PUSH CURRENT GREEN"
        Write-Host ""
        Write-Host " 3. Run & play / ForgeGUI Universal Modular Lab"
        Write-Host " 4. Patch status / receipts"
        Write-Host " 5. Project status / health"
        Write-Host " 6. Package debug handoff + open folder"
        Write-Host " 7. Advanced / provider operations"
        Write-Host " 8. Scan / apply pending patch queue"
        Write-Host " 0. Exit"

        $choice = Read-Host "Select"
        $completed = $null

        switch ($choice) {
            "1" {
                [void](Full-Gate)
                $completed = "Full Gate"
            }
            "2" {
                try {
                    if (-not (Commit-Push-Green)) {
                        [void](Package-Debug "Commit/push did not complete; inspect the authoritative session log for the refusal reason." "GIT_FAIL" $true)
                    }
                } catch {
                    $reason=$_.Exception.Message
                    Log "FAIL" $reason
                    [void](Package-Debug $reason "GIT_FAIL" $true)
                }
                $completed = "Commit / Push"
            }
            "3" {
                try { Run-Lab }
                catch {
                    $reason = $_.Exception.Message
                    Log "FAIL" "ForgeGUI Universal Lab failed :: $reason"
                    [void](Package-Debug $reason "RUN_FAIL" $true)
                }
                $completed = "Run Lab"
            }
            "4" {
                Patch-Status
                $completed = "Patch status"
            }
            "5" {
                Project-Health
                $completed = "Project health"
            }
            "6" {
                [void](Package-Debug "manual handoff requested" "MANUAL" $true)
                $completed = "Debug handoff"
            }
            "7" {
                Advanced-Menu
                $completed = "Advanced operation"
            }
            "8" {
                [void](Stage-RootPatchArchive)
                Scan-Patches
                $completed = "Patch queue"
            }
            "0" { return }
            default {
                Write-Host "Unknown option"
                $completed = "Menu input"
            }
        }

        Return-ToMainMenu $completed
    }
}


# Provider discovery queries must be non-interactive and emit JSON only.
if ($Operation -eq "status-json") { Write-ProviderStatusJson; exit 0 }
if ($Operation -eq "health-json") { Write-ProviderHealthJson; exit 0 }
if ($Operation -eq "capabilities-json") { Write-ProviderCapabilitiesJson; exit 0 }
if ($Operation -eq "operations-json") { Write-ProviderOperationsJson; exit 0 }

# Reconcile source governance only from still-valid transactional patch receipts.
Reconcile-PackageManifestFromAppliedReceipts
Resolve-AlreadyAppliedRootPatches

# Transition recovery: an update applied by an older PCC may have left the
# package in requires-canonicalization state. Normalize that state automatically
# before any normal menu, run, gate, or later patch operation.
try {
    Repair-PendingPatchNormalizationIfNeeded
} catch {
    $reason = "Automatic patch/update normalization failed: $($_.Exception.Message)"
    Log "FAIL" $reason
    [void](Package-Debug $reason "PATCH_NORMALIZE_FAIL" $true)
    exit 1
}

Log "INFO" "Operation started: startup"
if ([string]::IsNullOrWhiteSpace($Operation)) {
    [void](Stage-RootPatchArchive)
    Scan-Patches
    Root-Hygiene
} elseif ($Operation -eq "patch.scan") {
    [void](Stage-RootPatchArchive)
    Scan-Patches
} else {
    # Machine-invoked provider operations never prompt for patch approval.
    $pendingCount = @(Get-ChildItem -LiteralPath $Root -File -Filter '*.patch' -ErrorAction SilentlyContinue).Count
    if ($pendingCount -gt 0) { Log "WARN" "$pendingCount root patch transport(s) pending; non-interactive operation will not apply them." }
}

if ($Operation) {
    switch ($Operation) {
        "gate.full" { if (-not (Full-Gate)) { exit 1 } }
        "git.commit_push_green" {
            try {
                if (-not (Commit-Push-Green)) {
                    [void](Package-Debug "Commit/push did not complete; inspect the authoritative session log for the refusal reason." "GIT_FAIL" $false)
                    exit 1
                }
            } catch {
                $reason=$_.Exception.Message
                Log "FAIL" $reason
                [void](Package-Debug $reason "GIT_FAIL" $false)
                exit 1
            }
        }
        "run.lab" { Run-Lab }
        "patch.scan" { }
        "patch.status" { Patch-Status }
        "project.status" { Project-Health }
        "debug.bundle" { [void](Package-Debug "manual handoff requested" "MANUAL" $true) }
        "handoff.open" { Open-HandoffFolder }
        "fmt.fix" { Fmt-Fix }
        "root.hygiene" { Root-Hygiene }
        "git.setup" { Git-Setup }
        "cargo.lock.sync" {
            Run-Native "synchronize Cargo.lock" "cargo" @("generate-lockfile")
            Run-Native "validate synchronized Cargo.lock" "cargo" @("metadata","--locked","--format-version","1","--no-deps")
            Clear-Green
            Write-NeedsGateState "Cargo.lock synchronization changed/revalidated dependency state; explicit Full Gate is required." $null $true
        }
        "patch.rollback_last" { Rollback-LastPatch }
        "project.health" { Write-ProviderHealthJson }
        "provider.capabilities" { Write-ProviderCapabilitiesJson }
        "provider.operations" { Write-ProviderOperationsJson }
        default { throw "Unknown operation $Operation" }
    }
} else {
    Menu
}
