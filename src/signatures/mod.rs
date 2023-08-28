mod verify_signature;
use self::verify_signature::TrustStatus;
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

  /// Verifies a file given a list of publisher names
  /// ```
  /// import { verifySignatureByPublisherNames } from "whimy"
  /// const filePath = resolve(directoryName, '../../test_signed_data/signed_exes/microsoft_signed.exe');
  /// const output = verifySignatureByPublisherNames(filePath, ['CN="Microsoft Corporation",O="Microsoft Corporation",L=Redmond,S=Washington,C=US"'])
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
