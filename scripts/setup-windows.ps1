$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot
$mpvDir = Join-Path $repoRoot '.libmpv'
if (-not [Environment]::Is64BitOperatingSystem) { throw 'Windows x64 is required.' }
New-Item -ItemType Directory -Path $mpvDir -Force | Out-Null
$archive = Join-Path $mpvDir 'mpv-dev.7z'
$archiveUrl = 'https://github.com/SimoHypers/limusic/releases/download/deps/libmpv-2-x86_64-20260610-git-304426c.7z'
$expectedHash = 'af708edcdf6ed12ac7b2bb31b4cbeba0bf7d4eef345a715ef5de782c0ddd6671'
if (-not (Test-Path -LiteralPath $archive)) {
    Invoke-WebRequest -UseBasicParsing -Uri $archiveUrl -OutFile $archive
}
if ((Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant() -ne $expectedHash) {
    throw 'libmpv archive checksum mismatch. Remove .libmpv/mpv-dev.7z and retry.'
}
$sevenZip = (Get-Command 7z -ErrorAction SilentlyContinue).Source
if (-not $sevenZip) { $sevenZip = Join-Path $env:ProgramFiles '7-Zip/7z.exe' }
if (-not (Test-Path -LiteralPath $sevenZip)) { throw 'Install 7-Zip or add 7z to PATH.' }
& $sevenZip x $archive "-o$mpvDir" -y | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'libmpv extraction failed.' }
$dll = Join-Path $mpvDir 'libmpv-2.dll'
if (-not (Test-Path -LiteralPath $dll)) { throw 'libmpv-2.dll is absent from the archive.' }

if (-not (Get-Command dumpbin -ErrorAction SilentlyContinue)) {
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
    if (-not (Test-Path -LiteralPath $vswhere)) { throw 'Install Visual Studio C++ Build Tools.' }
    $vsInstall = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if (-not $vsInstall) { throw 'Visual Studio C++ x64 tools are required.' }
    Import-Module (Join-Path $vsInstall 'Common7/Tools/Microsoft.VisualStudio.DevShell.dll')
    Enter-VsDevShell -VsInstallPath $vsInstall -SkipAutomaticLocation -DevCmdArguments '-arch=x64 -host_arch=x64'
}
$exports = & dumpbin /exports $dll
if ($LASTEXITCODE -ne 0) { throw 'Could not read libmpv exports.' }
$symbols = @($exports | ForEach-Object {
    if ($_ -match '^\s+\d+\s+[0-9A-Fa-f]+\s+[0-9A-Fa-f]+\s+(\w+)') { $Matches[1] }
})
if ($symbols.Count -lt 50) { throw 'Unexpected libmpv export table.' }
$definition = Join-Path $mpvDir 'mpv.def'
@('EXPORTS') + $symbols | Set-Content -LiteralPath $definition -Encoding ASCII
Push-Location $mpvDir
try {
    & lib /def:mpv.def /name:libmpv-2.dll /out:mpv.lib /machine:x64
    if ($LASTEXITCODE -ne 0) { throw 'MSVC import library generation failed.' }
} finally { Pop-Location }
Copy-Item -LiteralPath $dll -Destination (Join-Path $repoRoot 'src-tauri/libmpv-2.dll') -Force
Write-Host 'libmpv is ready. Run scripts/build-windows.ps1 next.'
