use tokio::task;
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED, CoInitializeSecurity, RPC_C_AUTHN_LEVEL_DEFAULT, RPC_C_IMP_LEVEL_IMPERSONATE, EOAC_NONE};

pub struct AsyncQueryHandler {}

impl AsyncQueryHandler {
  pub fn new() -> napi::Result<Self> {
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED)
          .map_err(|_error| napi::Error::from_reason("Failed to Initialize COM"))?;
      };
  
      AsyncQueryHandler::initialize_security()?;
  
    //   let server = WMIQueryHandler::connect_to_wmi_namespace(&namespace)?;
  
    //   unsafe {
    //     let _ = CoSetProxyBlanket(
    //       &server,
    //       RPC_C_AUTHN_WINNT,
    //       RPC_C_AUTHN_NONE,
    //       None,
    //       RPC_C_AUTHN_LEVEL_CALL,
    //       RPC_C_IMP_LEVEL_IMPERSONATE,
    //       None,
    //       EOAC_NONE,
    //     );
    //   };
  
    //   Ok(WMIQueryHandler {
    //     server: Some(server),
    //   })
    Ok(AsyncQueryHandler {})
  }

  pub fn initialize_security() -> napi::Result<()> {
    unsafe {
      CoInitializeSecurity(
        None,
        -1,
        None,
        None,
        RPC_C_AUTHN_LEVEL_DEFAULT,
        RPC_C_IMP_LEVEL_IMPERSONATE,
        None,
        EOAC_NONE,
        None,
      )
    }
    .map_err(|_error| napi::Error::from_reason("Failed to initialize security"))?;
    Ok(())
  }







  async fn called_function(id: i32) {
    println!("Number: {}", id);
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
  }

  pub async fn my_async_function(&self) {
    let mut handles = vec![];

    for i in 1..=50 {
      let handle = task::spawn(Self::called_function(i));
      handles.push(handle);
    }

    for handle in handles {
      handle.await.unwrap();
    }
  }
}
