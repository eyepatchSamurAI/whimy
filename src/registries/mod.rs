#[napi(js_name = "registries")]
pub struct Registries {}

impl Default for Registries {
  fn default() -> Self {
    Self::new()
  }
}

#[napi]
impl Registries {
  #[napi(constructor)]
  pub fn new() -> Self {
    Registries {}
  }

  #[napi]
  pub fn list() {

  }
}