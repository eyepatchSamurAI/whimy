# Must be in admin
$domainName = "TotallyFakeTestDomain.com"
$signTool = "C:\Program Files (x86)\Windows Kits\10\App Certification Kit\signtool.exe"
$password = "password"
$name = "customCert"
$certificatePath = ".\$name.pfx"
$secureStringPassword = ConvertTo-SecureString -String $password -Force -AsPlainText

# Check if running as administrator
if (-not ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
    Write-Host "This script requires elevated permissions. Run as Administrator."
    exit
}
cargo build --release

# Generate the PFX
openssl ecparam -out "$name.key" -name prime256v1 -genkey
openssl req -new -sha256 -key "$name.key" -out "$name.csr" --config ".\openssl.cnf" -batch
openssl x509 -req -sha256 -days 1 -in "$name.csr" -signkey "$name.key" -out "$name.crt"
openssl pkcs12 -export -out "$name.pfx" -inkey "$name.key" -in "$name.crt"

# Import a PFX certificate into a certificate store on the local computer
$cert = Import-PfxCertificate -FilePath $certificatePath -CertStoreLocation Cert:\LocalMachine\Root -Password $secureStringPassword

# Sign exe
&$signTool sign /fd SHA256 /td sha256 /tr http://timestamp.digicert.com /f .\$name.pfx /p $password .\target\release\signed_exe.exe
