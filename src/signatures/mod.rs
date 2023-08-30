mod verify_signature;
use self::verify_signature::{verify_signature_from_path, TrustStatus};
use std::path::Path;
pub use verify_signature::{allowed_extensions, verify_signature_by_publisher};

#[napi(js_name = "signatures")]
pub struct Signatures {}

impl Default for Signatures {
  fn default() -> Self {
    Self::new()
  }
}

#[napi]
impl Signatures {
  #[napi(constructor)]
  pub fn new() -> Self {
    Signatures {}
  }

  /// Returns the allowed extensions of files that can have their signature verified
  #[napi]
  pub fn allowed_extensions() -> Vec<String> {
    allowed_extensions()
  }

  /// Checks if a file is signed, does not validate the signature.
  /// ```
  /// import { signatures } from "whimy"
  ///
  /// let signature_status = signatures.verifySignatureFromPath("./path/to/file.exe");
  /// let isSigned = signature_status.signed;
  /// ```
  ///
  #[napi]
  pub fn verify_signature_from_path(path: String) -> napi::Result<TrustStatus> {
    verify_signature_from_path(Path::new(&path))
  }

  /// Verifies a file given a list of publisher names
  /// ```
  /// import { signatures } from "whimy"
  /// const filePath = resolve(directoryName, '../../test_signed_data/signed_exes/microsoft_signed.exe');
  /// const output = signatures.verifySignatureByPublisher(filePath, ['CN="Microsoft Corporation",O="Microsoft Corporation",L=Redmond,S=Washington,C=US"'])
  /// console.log(output);
  /// ```
  ///
  #[napi]
  pub fn verify_signature_by_publisher(
    file_path: String,
    publish_names: Vec<String>,
  ) -> napi::Result<TrustStatus> {
    verify_signature_by_publisher(file_path, publish_names)
  }
}
