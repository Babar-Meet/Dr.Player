# Download script for test video files
# Run this from the TESTING_videos directory
# Requires PowerShell 5.1+

$TargetDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
if (-not $TargetDir) { $TargetDir = "." }

Write-Host "Downloading test video files to: $TargetDir" -ForegroundColor Green

# Ensure target directory exists
New-Item -ItemType Directory -Path $TargetDir -Force | Out-Null

# ============================================================
# MP4 files
# ============================================================
Write-Host "`n--- Downloading MP4 files ---" -ForegroundColor Cyan

$mp4s = @(
    @{ Url = "https://truefilesize.com/files/mp4/sample-1mb.mp4"; Name = "sample-1mb.mp4" },
    @{ Url = "https://interactive-examples.mdn.mozilla.net/media/cc0-videos/flower.mp4"; Name = "flower.mp4" },
    @{ Url = "https://github.com/mdn/learning-area/raw/main/html/multimedia-and-embedding/video-and-audio-content/rabbit320.mp4"; Name = "rabbit320.mp4" }
)

foreach ($f in $mp4s) {
    $out = Join-Path $TargetDir $f.Name
    if (-not (Test-Path $out)) {
        Write-Host "  Downloading $($f.Name) ..." -NoNewline
        try {
            Invoke-WebRequest -Uri $f.Url -OutFile $out -UseBasicParsing -ErrorAction Stop
            Write-Host " OK ($([math]::Round((Get-Item $out).Length/1KB)) KB)" -ForegroundColor Green
        } catch {
            Write-Host " FAILED: $_" -ForegroundColor Red
        }
    } else {
        Write-Host "  $($f.Name) already exists, skipping" -ForegroundColor Yellow
    }
}

# ============================================================
# WebM files
# ============================================================
Write-Host "`n--- Downloading WebM files ---" -ForegroundColor Cyan

$webms = @(
    @{ Url = "https://interactive-examples.mdn.mozilla.net/media/cc0-videos/flower.webm"; Name = "flower.webm" },
    @{ Url = "https://github.com/mdn/learning-area/raw/main/html/multimedia-and-embedding/video-and-audio-content/rabbit320.webm"; Name = "rabbit320.webm" },
    @{ Url = "https://raw.githubusercontent.com/web-platform-tests/wpt/master/media-source/webm/test-vp9.webm"; Name = "test-vp9.webm" }
)

foreach ($f in $webms) {
    $out = Join-Path $TargetDir $f.Name
    if (-not (Test-Path $out)) {
        Write-Host "  Downloading $($f.Name) ..." -NoNewline
        try {
            Invoke-WebRequest -Uri $f.Url -OutFile $out -UseBasicParsing -ErrorAction Stop
            Write-Host " OK ($([math]::Round((Get-Item $out).Length/1KB)) KB)" -ForegroundColor Green
        } catch {
            Write-Host " FAILED: $_" -ForegroundColor Red
        }
    } else {
        Write-Host "  $($f.Name) already exists, skipping" -ForegroundColor Yellow
    }
}

# ============================================================
# AVI files
# ============================================================
Write-Host "`n--- Downloading AVI files ---" -ForegroundColor Cyan

$avis = @(
    @{ Url = "https://samplefile.com/samples/download/video/avi/avi_5s_sample_file_318KB.avi/"; Name = "sample-5s.avi" },
    @{ Url = "https://samplefile.com/samples/download/video/avi/avi_15s_sample_file_927KB.avi/"; Name = "sample-15s.avi" },
    @{ Url = "https://filesamples.com/samples/video/avi/sample_640x360.avi"; Name = "sample_640x360.avi" }
)

foreach ($f in $avis) {
    $out = Join-Path $TargetDir $f.Name
    if (-not (Test-Path $out)) {
        Write-Host "  Downloading $($f.Name) ..." -NoNewline
        try {
            Invoke-WebRequest -Uri $f.Url -OutFile $out -UseBasicParsing -ErrorAction Stop
            Write-Host " OK ($([math]::Round((Get-Item $out).Length/1KB)) KB)" -ForegroundColor Green
        } catch {
            Write-Host " FAILED: $_" -ForegroundColor Red
        }
    } else {
        Write-Host "  $($f.Name) already exists, skipping" -ForegroundColor Yellow
    }
}

# ============================================================
# MOV files
# ============================================================
Write-Host "`n--- Downloading MOV files ---" -ForegroundColor Cyan

$movs = @(
    @{ Url = "https://truefilesize.com/files/mov/sample-5mb.mov"; Name = "sample-5mb.mov" },
    @{ Url = "https://testfileorg.netwet.net/Sample%20Video%202/sample_1280x720.mov"; Name = "sample_1280x720.mov" }
)

foreach ($f in $movs) {
    $out = Join-Path $TargetDir $f.Name
    if (-not (Test-Path $out)) {
        Write-Host "  Downloading $($f.Name) ..." -NoNewline
        try {
            Invoke-WebRequest -Uri $f.Url -OutFile $out -UseBasicParsing -ErrorAction Stop
            Write-Host " OK ($([math]::Round((Get-Item $out).Length/1KB)) KB)" -ForegroundColor Green
        } catch {
            Write-Host " FAILED: $_" -ForegroundColor Red
        }
    } else {
        Write-Host "  $($f.Name) already exists, skipping" -ForegroundColor Yellow
    }
}

# ============================================================
# MKV files
# ============================================================
Write-Host "`n--- Downloading MKV files ---" -ForegroundColor Cyan

$mkvs = @(
    @{ Url = "https://truefilesize.com/files/mkv/sample-1mb.mkv"; Name = "sample-1mb.mkv" },
    @{ Url = "https://filesamples.com/samples/video/mkv/sample_640x360.mkv"; Name = "sample_640x360.mkv" },
    @{ Url = "https://github.com/Matroska-Org/matroska-test-files/raw/master/test_files/test1.mkv"; Name = "test1-matroskaconformance.mkv" }
)

foreach ($f in $mkvs) {
    $out = Join-Path $TargetDir $f.Name
    if (-not (Test-Path $out)) {
        Write-Host "  Downloading $($f.Name) ..." -NoNewline
        try {
            Invoke-WebRequest -Uri $f.Url -OutFile $out -UseBasicParsing -ErrorAction Stop
            Write-Host " OK ($([math]::Round((Get-Item $out).Length/1KB)) KB)" -ForegroundColor Green
        } catch {
            Write-Host " FAILED: $_" -ForegroundColor Red
        }
    } else {
        Write-Host "  $($f.Name) already exists, skipping" -ForegroundColor Yellow
    }
}

# ============================================================
# OGG/Theora files
# ============================================================
Write-Host "`n--- Downloading OGG/Theora files ---" -ForegroundColor Cyan

$ogvs = @(
    @{ Url = "https://upload.wikimedia.org/wikipedia/commons/3/33/320x240.ogv"; Name = "320x240_test_pattern.ogv" },
    @{ Url = "https://upload.wikimedia.org/wikipedia/commons/5/58/Big_buck_bunny_ecu.ogv"; Name = "big_buck_bunny_ecu.ogv" }
)

foreach ($f in $ogvs) {
    $out = Join-Path $TargetDir $f.Name
    if (-not (Test-Path $out)) {
        Write-Host "  Downloading $($f.Name) ..." -NoNewline
        try {
            Invoke-WebRequest -Uri $f.Url -OutFile $out -UseBasicParsing -ErrorAction Stop
            Write-Host " OK ($([math]::Round((Get-Item $out).Length/1KB)) KB)" -ForegroundColor Green
        } catch {
            Write-Host " FAILED: $_" -ForegroundColor Red
        }
    } else {
        Write-Host "  $($f.Name) already exists, skipping" -ForegroundColor Yellow
    }
}

# ============================================================
# Summary
# ============================================================
Write-Host "`n============================================" -ForegroundColor Green
Write-Host "DOWNLOAD COMPLETE" -ForegroundColor Green
Write-Host "============================================" -ForegroundColor Green
Get-ChildItem -Path $TargetDir -File | Sort-Object Length | ForEach-Object {
    Write-Host ("  {0,-40} {1,8:N0} KB" -f $_.Name, ($_.Length / 1KB))
}
Write-Host "============================================" -ForegroundColor Green
Write-Host "Total files: $(@(Get-ChildItem -Path $TargetDir -File).Length)" -ForegroundColor Green
Write-Host "Total size:  $([math]::Round((Get-ChildItem -Path $TargetDir -File | Measure-Object Length -Sum).Sum / 1MB, 2)) MB" -ForegroundColor Green
