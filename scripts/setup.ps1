# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Stya Yur Open Source Studio
param([switch]$SkipModel)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$cacheDir = Join-Path $projectRoot '.cache'
$toolsDir = Join-Path $projectRoot 'tools'
New-Item -ItemType Directory -Force $cacheDir,$toolsDir | Out-Null

function Download-File([string]$Url, [string]$Destination) {
    if (!(Test-Path -LiteralPath $Destination)) {
        $partial = "$Destination.partial"
        & curl.exe -L --fail --retry 2 $Url -o $partial
        if ($LASTEXITCODE -ne 0) { throw "Download failed: $Url" }
        Move-Item -LiteralPath $partial -Destination $Destination
    }
}

Write-Host 'Preparing portable eSpeak-NG 1.52.0 (no system installation).'
$msiPath = Join-Path $cacheDir 'espeak-ng.msi'
Download-File 'https://github.com/espeak-ng/espeak-ng/releases/download/1.52.0/espeak-ng.msi' $msiPath
$lessmsiZip = Join-Path $cacheDir 'lessmsi.zip'
Download-File 'https://github.com/activescott/lessmsi/releases/download/v2.12.9/lessmsi-v2.12.9.zip' $lessmsiZip
$lessmsiDir = Join-Path $cacheDir 'lessmsi'
Expand-Archive -LiteralPath $lessmsiZip -DestinationPath $lessmsiDir -Force
$extractionDir = Join-Path $cacheDir 'espeak-extracted'
# lessmsi requires a trailing backslash; this is extraction, not installation.
& (Join-Path $lessmsiDir 'lessmsi.exe') x $msiPath ($extractionDir + '\') | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'Portable eSpeak extraction failed' }
$espeakDir = Join-Path $toolsDir 'espeak-ng'
New-Item -ItemType Directory -Force $espeakDir | Out-Null
Copy-Item -Path (Join-Path $extractionDir 'SourceDir/eSpeak NG/*') -Destination $espeakDir -Recurse -Force
$env:ESPEAK_NG = Join-Path $espeakDir 'espeak-ng.exe'
& $env:ESPEAK_NG ('--path=' + $espeakDir) -q --ipa=2 -v en-us hello
if ($LASTEXITCODE -ne 0) { throw 'eSpeak G2P check failed' }

if (!$SkipModel) {
    Write-Host 'Preparing Kokoro v1.0 model bundle (~334 MiB download).'
    $modelArchive = Join-Path $cacheDir 'kokoro-multi-lang-v1_0.tar.bz2'
    Download-File 'https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/kokoro-multi-lang-v1_0.tar.bz2' $modelArchive
    $modelsDir = Join-Path $projectRoot 'models'
    New-Item -ItemType Directory -Force $modelsDir | Out-Null
    & tar -xjf $modelArchive -C $modelsDir
    if ($LASTEXITCODE -ne 0) { throw 'Kokoro model extraction failed' }
}
Write-Host 'Setup complete. From the project root: cargo run -p ipa-desktop -- doctor'
