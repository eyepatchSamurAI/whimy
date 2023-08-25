# Must be in admin
$domainName = "TotallyFakeTestDomain.com"
$signTool = "C:\Program Files (x86)\Windows Kits\10\App Certification Kit\signtool.exe"
$name = "customCert"
$certificatePath = ".\$name.pfx"
$customExePath = ".\target\release\custom_signed_exe.exe"
$destinationPath = "..\signed_exes\custom_signed_exe.exe"


cargo build --release

# Generate the PFX
openssl ecparam -out "$name.key" -name prime256v1 -genkey
openssl req -new -sha256 -key "$name.key" -out "$name.csr" --config ".\openssl.cnf" -batch
openssl x509 -req -sha256 -days 1 -in "$name.csr" -signkey "$name.key" -out "$name.crt"
openssl pkcs12 -export -out "$name.pfx" -inkey "$name.key" -in "$name.crt" -passout pass:

# Import a PFX certificate into a certificate store on the local computer
$cert = Import-PfxCertificate -FilePath $certificatePath -CertStoreLocation Cert:\LocalMachine\Root

# Sign exe
&$signTool sign /fd SHA256 /td sha256 /tr http://timestamp.digicert.com /f .\$name.pfx $customExePath

# Move signed exe 
Move-Item -Path $customExePath -Destination $destinationPath -Force

