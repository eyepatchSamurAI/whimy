$packageJson = Get-Content -Raw -Path .\package.json | ConvertFrom-Json
$version = $packageJson.version
Add-Content -Value "VERSION=$version" -Path $env:GITHUB_ENV
