$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$devA = Join-Path (Split-Path -Parent $root) "Dev A"
if (-not (Test-Path $devA)) {
    $devA = Join-Path $root "..\Dev A"
}
$assets = Join-Path $root "android\app\src\main\assets\models"
New-Item -ItemType Directory -Force -Path $assets | Out-Null

# Vocab: export bundle first, then training output
$vocabSources = @(
    (Join-Path $devA "export\vocab.txt"),
    (Join-Path $devA "models\final_money_date_model\vocab.txt")
)
$copiedVocab = $false
foreach ($src in $vocabSources) {
    if (Test-Path $src) {
        Copy-Item $src (Join-Path $assets "vocab.txt") -Force
        Write-Host "Copied vocab from $src"
        $copiedVocab = $true
        break
    }
}
if (-not $copiedVocab) {
    Write-Warning "vocab.txt not found under Dev A/export or models/final_money_date_model"
}

# TFLite: find canonical Dev A exports/model.tflite (and fall back to other candidates).
$modelDest = Join-Path $assets "model.tflite"
$modelCandidates = @(
    (Join-Path $devA "export\model.tflite"),
    (Join-Path $devA "exports\model.tflite"),
    (Join-Path $devA "models\model.tflite"),
    (Join-Path $devA "models\model_int8.tflite")
)

$tflitePath = $null
foreach ($cand in $modelCandidates) {
    if (Test-Path $cand) {
        $tflitePath = $cand
        break
    }
}

if (-not $tflitePath) {
    # Last resort: recursive search (keeps script robust to directory naming).
    $found = Get-ChildItem -Path $devA -Recurse -Filter "model.tflite" -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($found) {
        $tflitePath = $found.FullName
    }
}

if ($tflitePath -and (Test-Path $tflitePath)) {
    Copy-Item $tflitePath $modelDest -Force
    Write-Host "Copied model.tflite from $tflitePath"
} else {
    Write-Error "model.tflite not found under Dev A. Expected Dev A/exports/model.tflite or equivalent. " +
                "Generate it via Dev A/scripts/onnx_to_tflite.py, then rerun this script."
    exit 1
}

Write-Host "Assets directory: $assets"
