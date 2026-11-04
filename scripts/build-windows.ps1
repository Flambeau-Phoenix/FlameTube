param([switch]$Debug, [switch]$NoBundle, [switch]$KeepTarget)
$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot

# Ensure Cargo and Node are on PATH even when invoked from a shell without dev env vars loaded
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    foreach ($candidate in @('E:\DevCache\cargo\bin', "$env:CARGO_HOME\bin", "$HOME\.cargo\bin")) {
        if ($candidate -and (Test-Path -LiteralPath (Join-Path $candidate 'cargo.exe'))) {
            $env:Path = "$candidate;$env:Path"
            if (-not $env:CARGO_HOME -and $candidate -eq 'E:\DevCache\cargo\bin') {
                $env:CARGO_HOME = 'E:\DevCache\cargo'
            }
            if (-not $env:RUSTUP_HOME -and (Test-Path -LiteralPath 'E:\DevCache\rustup')) {
                $env:RUSTUP_HOME = 'E:\DevCache\rustup'
            }
            break
        }
    }
}
if (-not (Get-Command npx.cmd -ErrorAction SilentlyContinue)) {
    if (Test-Path -LiteralPath 'D:\node_LTS_docker\npx.cmd') {
        $env:Path = "D:\node_LTS_docker;$env:Path"
    }
}

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

    if (-not $NoBundle) {
        $profile = if ($Debug) { 'debug' } else { 'release' }
        $builtInstaller = Join-Path $repoRoot "target\$profile\bundle\nsis\FlameTube_0.2.0_x64-setup.exe"
        $rootInstaller = Join-Path $repoRoot 'FlameTube-Setup-0.2.0-x64.exe'
        $shaFile = Join-Path $repoRoot 'FlameTube-Setup-0.2.0-x64.sha256'
        if (Test-Path -LiteralPath $builtInstaller) {
            Copy-Item -LiteralPath $builtInstaller -Destination $rootInstaller -Force
            $hash = (Get-FileHash -LiteralPath $rootInstaller -Algorithm SHA256).Hash.ToLowerInvariant()
            "$hash  FlameTube-Setup-0.2.0-x64.exe" | Set-Content -LiteralPath $shaFile -Encoding ASCII
            Write-Host "Saved installer to $rootInstaller ($hash)"
        }
        if (-not $KeepTarget) {
            Remove-Item -LiteralPath (Join-Path $repoRoot 'target') -Recurse -Force -ErrorAction SilentlyContinue
            Remove-Item -LiteralPath (Join-Path $repoRoot 'ui\build') -Recurse -Force -ErrorAction SilentlyContinue
            Remove-Item -LiteralPath (Join-Path $repoRoot 'ui\.svelte-kit') -Recurse -Force -ErrorAction SilentlyContinue
            Write-Host 'Cleaned intermediate target/ and ui/build caches.'
        }
    }
} finally {
    $env:RUSTFLAGS = $savedFlags
    $env:CARGO_ENCODED_RUSTFLAGS = $savedEncoded
}
