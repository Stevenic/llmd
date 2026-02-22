# bench.ps1 — Benchmark all LLMD compiler implementations
# Usage: pwsh tools/bench.ps1
# Requires: node (18+), python (3.10+), cargo build --release in tools/rust/

param(
    [int]$Runs = 5
)

$ErrorActionPreference = "SilentlyContinue"
$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Push-Location $root

$config = "config/llmdc.config.json"
$samples = @(
    @{ name = "api-spec.md"; path = "corpora/samples/api-spec.md" },
    @{ name = "fluentlm-components.md"; path = "corpora/samples/fluentlm-components.md" }
)

$tools = @(
    @{ name = "JS"; cmd = "node tools/js/llmdc.js {INPUT} --config $config -o {OUT}" },
    @{ name = "Python"; cmd = "python tools/py/llmdc.py {INPUT} --config $config -o {OUT}" },
    @{ name = "Rust"; cmd = "tools/rust/target/release/llmdc{EXE} {INPUT} --config $config -o {OUT}" }
)

$exe = if ($IsWindows -or $env:OS -match "Windows") { ".exe" } else { "" }
$rustBin = "tools/rust/target/release/llmdc$exe"

# Check prerequisites
$missing = @()
if (-not (Get-Command node -ErrorAction SilentlyContinue)) { $missing += "node" }
if (-not (Get-Command python -ErrorAction SilentlyContinue)) { $missing += "python" }
if (-not (Test-Path $rustBin)) { $missing += "Rust binary (run: cargo build --release --manifest-path tools/rust/Cargo.toml)" }
if ($missing.Count -gt 0) {
    Write-Host "Missing: $($missing -join ', ')" -ForegroundColor Red
    Pop-Location
    exit 1
}

$tmpOut = [System.IO.Path]::GetTempFileName()

function Measure-Tool($cmd, $runs) {
    $times = @()
    for ($i = 0; $i -lt $runs; $i++) {
        $sw = [System.Diagnostics.Stopwatch]::StartNew()
        Invoke-Expression $cmd 2>$null | Out-Null
        $sw.Stop()
        $times += $sw.ElapsedMilliseconds
    }
    $sorted = $times | Sort-Object
    $median = $sorted[[math]::Floor($runs / 2)]
    return @{ median = $median; all = $times }
}

Write-Host ""
Write-Host "LLMD Compiler Benchmark ($Runs runs per tool, median reported)" -ForegroundColor Cyan
Write-Host "=============================================================" -ForegroundColor Cyan

$results = @()

foreach ($sample in $samples) {
    $size = (Get-Item $sample.path).Length
    $sizeKB = [math]::Round($size / 1024, 1)
    Write-Host ""
    Write-Host "$($sample.name) ($sizeKB KB)" -ForegroundColor Yellow
    Write-Host ("-" * 50)

    foreach ($tool in $tools) {
        $cmd = $tool.cmd -replace "\{INPUT\}", $sample.path
        $cmd = $cmd -replace "\{OUT\}", $tmpOut
        $cmd = $cmd -replace "\{EXE\}", $exe

        $result = Measure-Tool $cmd $Runs
        $label = $tool.name.PadRight(8)
        Write-Host "  $label  $($result.median)ms  (runs: $($result.all -join ', '))"

        $results += @{
            file = $sample.name
            tool = $tool.name
            median = $result.median
            size = $sizeKB
        }
    }
}

Remove-Item $tmpOut -ErrorAction SilentlyContinue

# Summary table
Write-Host ""
Write-Host "Summary" -ForegroundColor Cyan
Write-Host "-------"
Write-Host ""
Write-Host ("| {0,-35} | {1,8} | {2,8} | {3,8} |" -f "File", "JS", "Python", "Rust")
Write-Host ("| {0,-35} | {1,8} | {2,8} | {3,8} |" -f ("-" * 35), ("-" * 8), ("-" * 8), ("-" * 8))

foreach ($sample in $samples) {
    $js = ($results | Where-Object { $_.file -eq $sample.name -and $_.tool -eq "JS" }).median
    $py = ($results | Where-Object { $_.file -eq $sample.name -and $_.tool -eq "Python" }).median
    $rs = ($results | Where-Object { $_.file -eq $sample.name -and $_.tool -eq "Rust" }).median
    $sizeKB = ($results | Where-Object { $_.file -eq $sample.name } | Select-Object -First 1).size
    $label = "$($sample.name) ($sizeKB KB)"
    Write-Host ("| {0,-35} | {1,5} ms | {2,5} ms | {3,5} ms |" -f $label, $js, $py, $rs)
}

Write-Host ""
Pop-Location
