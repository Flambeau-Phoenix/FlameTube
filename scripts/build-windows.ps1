param([switch]$Debug, [switch]$NoBundle)
$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot
$mpvDir = Join-Path $repoRoot '.libmpv'
if (-not (Test-Path -LiteralPath (Join-Path $mpvDir 'mpv.lib'))) {
    throw 'Run scripts/setup-windows.ps1 first.'
}
if (-not (Test-Path -LiteralPath (Join-Path $repoRoot 'src-tauri/libmpv-2.dll'))) {
    throw 'The runtime DLL is missing. Run scripts/setup-windows.ps1.'
}
$savedFlags = $env:RUSTFLAGS
$savedEncoded = $env:CARGO_ENCODED_RUSTFLAGS
try {
    # Cargo's encoded form preserves spaces in a checkout's absolute path.
    $env:RUSTFLAGS = $null
    $env:CARGO_ENCODED_RUSTFLAGS = '-L' + [char]31 + "native=$mpvDir"
    Push-Location (Join-Path $repoRoot 'ui')
    try {
        & npx.cmd --yes pnpm@10 install --frozen-lockfile
        if ($LASTEXITCODE -ne 0) { throw 'Frontend dependency install failed.' }
        foreach ($task in @('check', 'test', 'build')) {
            & npx.cmd --yes pnpm@10 $task
            if ($LASTEXITCODE -ne 0) { throw "Frontend $task failed." }
        }
    } finally { Pop-Location }
    Push-Location (Join-Path $repoRoot 'src-tauri')
    try {
        $tauriArgs = @('--yes', '@tauri-apps/cli@2.11.5', 'build', '--ci')
        if ($Debug) { $tauriArgs += '--debug' }
        if ($NoBundle) { $tauriArgs += '--no-bundle' } else { $tauriArgs += @('--bundles', 'nsis') }
        & npx.cmd @tauriArgs
        if ($LASTEXITCODE -ne 0) { throw 'Native application build failed.' }
    } finally { Pop-Location }
} finally {
    $env:RUSTFLAGS = $savedFlags
    $env:CARGO_ENCODED_RUSTFLAGS = $savedEncoded
}
