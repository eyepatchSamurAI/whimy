This is for testing only. 

The microsoft signed folder contains some testing exe, dll's, and a text file for testing

The custom signed is made so we can easily generate and sign an exe to test various sigining cases.

### Setup custom signed
- Must run as admin
- cd into custom_signed
- run `cargo build --release`
- edit the openssl.cnf to contain the signatures you want
- run `.\setting_up_cert_testing.ps1` and put "password" for the password
- verify signing by running `.\check_cert.ps1`


### How to find custon Certificate
Once signed, you can find your certificate here: Cert:\LocalMachine\Root
As of now if the cert is stored somewhere like Cert:\LocalMachine\My then it will not be able to verify the certificate.
To get there from a GUI
- Press Windows + R
- type `mmc`
- File -> Add or Remove Snap-ins
- Click on certificates and press the "Add>" Button
- Manage the certificates for "Computer Account" and continue to Finish
- Open Trusted Root Certification Authorities
- Inside the Certificates folder you should see the fake domain name

Make sure to delete these fake certificates once you are done testing
