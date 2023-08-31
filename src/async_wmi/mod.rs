use self::async_wmi_query_handler::AsyncQueryHandler;

mod async_wmi_query_handler;

#[napi]
pub struct AsyncWmi {
  query_handler: AsyncQueryHandler,
}

#[napi]
impl AsyncWmi {
  #[napi(constructor)]
  pub fn new() -> napi::Result<Self> {
    Ok(AsyncWmi {
      query_handler: AsyncQueryHandler::new()?,
    })
  }
  #[napi]
  pub async fn async_test(&self) {
    self.query_handler.my_async_function().await
  }
}
