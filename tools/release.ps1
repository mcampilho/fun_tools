<#
.SYNOPSIS
Builds the release packages locally, like the GitHub workflow: a .zip for Windows,
a .tar.gz for Linux (built in WSL) and SHA256SUMS.txt, all in dist\.

.DESCRIPTION
The version comes from Cargo.toml (tag v<version>). Local packages have no build
provenance attestation (only GitHub Actions can sign one), and their hashes differ
from the workflow's, so they should not replace the files of a published release.

.EXAMPLE
tools\release.ps1                  # build and test both packages
tools\release.ps1 -NoTest -NoLinux # quick Windows package only
tools\release.ps1 -Publish         # also create a draft release on GitHub
#>
param(
    # Create a draft GitHub release with the packages (needs gh).
    [switch]$Publish,
    # Skip the tests.
    [switch]$NoTest,
    # Only the Windows package.
    [switch]$NoLinux,
    # Only the Linux package.
    [switch]$NoWindows,
    # Script that puts the MSVC tools in the environment, when cl.exe is not found.
    [string]$DevCmd = $env:RTOOLS_DEVCMD
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

function Fail([string]$msg) {
    Write-Host "error: $msg" -ForegroundColor Red
    exit 1
}

function Run([string]$what, [scriptblock]$cmd) {
    Write-Host "== $what" -ForegroundColor Cyan
    & $cmd
    if ($LASTEXITCODE -ne 0) { Fail "$what failed (exit code $LASTEXITCODE)" }
}

$version = (Select-String -Path Cargo.toml -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1).Matches.Groups[1].Value
if (-not $version) { Fail 'no version in Cargo.toml' }
$tag = "v$version"
$dist = Join-Path $root 'dist'
New-Item -ItemType Directory -Force $dist | Out-Null
# Only the packages being made again are replaced (-NoWindows keeps the .zip, ...).
if (-not $NoWindows) { Remove-Item (Join-Path $dist "fun_tools-$tag-windows-*") -Force -ErrorAction SilentlyContinue }
if (-not $NoLinux) { Remove-Item (Join-Path $dist "fun_tools-$tag-linux-*") -Force -ErrorAction SilentlyContinue }
Write-Host "fun_tools $tag -> $dist"

if (-not $NoWindows) {
    if (-not (Get-Command cl.exe -ErrorAction SilentlyContinue)) {
        $candidates = @(@($DevCmd, 'e:\portable\tools\dev\msvc\devcmd.ps1') | Where-Object { $_ -and (Test-Path $_) })
        if ($candidates) {
            . $candidates[0] *> $null
        }
        if (-not (Get-Command cl.exe -ErrorAction SilentlyContinue)) {
            Fail 'the MSVC tools (cl.exe, link.exe) are not in the environment: open a Developer PowerShell or pass -DevCmd <script>'
        }
    }
    $target = 'x86_64-pc-windows-msvc'
    # The C runtime goes inside the .exe: no Visual C++ redistributable needed.
    $env:RUSTFLAGS = '-C target-feature=+crt-static'
    try {
        if (-not $NoTest) { Run 'Windows tests' { cargo test --release --locked --target $target } }
        Run 'Windows build' { cargo build --release --locked --target $target }
    } finally {
        Remove-Item env:RUSTFLAGS -ErrorAction SilentlyContinue
    }
    $name = "fun_tools-$tag-windows-x86_64"
    $stage = Join-Path ([IO.Path]::GetTempPath()) "rtools-$([guid]::NewGuid().ToString('N'))"
    $folder = Join-Path $stage $name
    New-Item -ItemType Directory -Force $folder | Out-Null
    try {
        Copy-Item "target\$target\release\r*.exe" $folder
        Copy-Item README.md, README.pt.md, LICENSE $folder
        Compress-Archive -Path $folder -DestinationPath (Join-Path $dist "$name.zip") -CompressionLevel Optimal
        $n = (Get-ChildItem $folder -Filter 'r*.exe').Count
        Write-Host "dist\$name.zip: $n tools"
    } finally {
        Remove-Item -Recurse -Force $stage
    }
}

if (-not $NoLinux) {
    if (-not (Get-Command wsl.exe -ErrorAction SilentlyContinue)) { Fail 'WSL is needed for the Linux package (or use -NoLinux)' }
    $extra = if ($NoTest) { 'notest' } else { '' }
    # WSL starts in the same folder (as /mnt/...), so relative paths work.
    Run 'Linux build (WSL)' { wsl.exe sh tools/release-linux.sh $tag dist $extra }
}

# SHA256SUMS.txt in the format of sha256sum (checked with: sha256sum -c SHA256SUMS.txt).
$packages = Get-ChildItem $dist -File | Where-Object { $_.Name -like "fun_tools-$tag-*" } | Sort-Object Name
if (-not $packages) { Fail 'no packages were made' }
$lines = $packages | ForEach-Object { '{0}  {1}' -f (Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLower(), $_.Name }
[IO.File]::WriteAllText((Join-Path $dist 'SHA256SUMS.txt'), (($lines -join "`n") + "`n"))
Write-Host "== SHA256SUMS.txt" -ForegroundColor Cyan
$lines | ForEach-Object { Write-Host $_ }

if ($Publish) {
    if (-not (Get-Command gh -ErrorAction SilentlyContinue)) { Fail 'the GitHub CLI (gh) is needed to publish' }
    gh release view $tag *> $null
    if ($LASTEXITCODE -eq 0) {
        Fail "release $tag already exists on GitHub: raise the version in Cargo.toml (its files are not replaced)"
    }
    $files = @($packages.FullName) + (Join-Path $dist 'SHA256SUMS.txt')
    Run "Draft release $tag" { gh release create $tag @files --draft --title "fun_tools $tag" --generate-notes }
    Write-Host 'The release is a draft: check it on GitHub and publish it there.'
}
