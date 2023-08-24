$domainName = "TotallyFakeTestDomain.com"

$thumbprint = Get-ChildItem -Path Cert:\LocalMachine\Root | Where-Object { $_.Subject -like "*$domainName*" } | Select-Object -ExpandProperty Thumbprint
$cert = Get-ChildItem Cert:\LocalMachine\Root | Where-Object { $_.Thumbprint -eq $thumbprint }
# Print the subject
Write-Output $cert.Subject