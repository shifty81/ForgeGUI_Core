param([string]$Root = "")
$ErrorActionPreference = "Stop"
if ([string]::IsNullOrWhiteSpace($Root)) { $Root = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path }
$Root = [IO.Path]::GetFullPath($Root.Trim().Trim('"').Trim("'"))
function Info($m){Write-Host "[INFO] $m" -ForegroundColor Cyan}
function Pass($m){Write-Host "[PASS] $m" -ForegroundColor Green}
function Warn($m){Write-Host "[WARN] $m" -ForegroundColor Yellow}
$manifestPath=Join-Path $Root "PACKAGE_MANIFEST.json"
$protected=@{}
foreach($n in @("Cargo.toml","Cargo.lock","README.md","VERSION","PROJECT_CONTROL_CENTER.cmd","project.control.json","forge.gui.toml","rust-toolchain.toml","PACKAGE_MANIFEST.json","DEPENDENCY_PROVENANCE.json","THIRD_PARTY_NOTICES.md","CHANGELOG.md","ROADMAP.md","CONTRIBUTING.md","SECURITY.md","LICENSE","LICENSE.md","LICENSE.txt","NOTICE","NOTICE.txt",".gitignore",".gitattributes",".editorconfig")){$protected[$n.ToLowerInvariant()]=$true}
$manifestLoaded=$false
if(Test-Path -LiteralPath $manifestPath){try{$m=Get-Content $manifestPath -Raw|ConvertFrom-Json;foreach($e in @($m.files)){$r=([string]$e.path).Replace('\','/');if($r -and $r -notmatch '/'){$protected[$r.ToLowerInvariant()]=$true}};$manifestLoaded=$true}catch{Warn "Manifest authority unavailable: $($_.Exception.Message)"}}
$stamp=Get-Date -Format "yyyyMMdd-HHmmss";$dest=Join-Path $Root "artifacts\maintenance\root-cleanup\$stamp";$moved=@();$warnings=@();$patches=@()
foreach($f in @(Get-ChildItem -LiteralPath $Root -File -Force)){ $name=$f.Name;$low=$name.ToLowerInvariant();if($f.Extension -ieq '.patch'){$patches+=$name;continue};if($protected.ContainsKey($low)){continue};$operational=($name -like 'APPLY_FORGEGUI_*') -or ($name -ieq 'README.txt') -or ($name -like 'ForgeGUI_*Repair*') -or ($name -like 'ForgeGUI_DebugBundle_*') -or ($f.Extension -in @('.zip','.7z','.rar','.log','.tmp','.bak','.orig','.rej','.sha256'));if($operational){New-Item -ItemType Directory -Force -Path $dest|Out-Null;$target=Join-Path $dest $name;Move-Item -LiteralPath $f.FullName -Destination $target -Force;$moved+=$name}else{$warnings+="Unknown root file left in place: $name"}}
foreach($base in @('crates','examples','tools','templates','spec')){ $scan=Join-Path $Root $base;if(Test-Path $scan){foreach($f in @(Get-ChildItem $scan -File -Recurse -Force -ErrorAction SilentlyContinue|Where-Object{$_.Name -like '*.pre-*.bak' -or $_.Extension -in @('.orig','.rej')})){ $rel=$f.FullName.Substring($Root.Length).TrimStart([char[]]'\\/');$target=Join-Path $dest (Join-Path 'source-backups' $rel);New-Item -ItemType Directory -Force -Path (Split-Path $target -Parent)|Out-Null;Move-Item $f.FullName $target -Force;$moved+=$rel }}}
New-Item -ItemType Directory -Force -Path (Join-Path $Root 'artifacts\maintenance\root-cleanup')|Out-Null;$report=Join-Path $Root ("artifacts\maintenance\root-cleanup\root-hygiene-$stamp.json");[ordered]@{schema='forge.root_hygiene.report.v1';timestamp=(Get-Date).ToString('o');manifest_authority_loaded=$manifestLoaded;preserved_patches=$patches;moved=$moved;warnings=$warnings}|ConvertTo-Json -Depth 6|Set-Content $report -Encoding UTF8
foreach($w in $warnings){Warn $w};Pass "Root hygiene: $($moved.Count) residue item(s) moved, $($patches.Count) patch transport(s) preserved.";Info "Report: $report"
