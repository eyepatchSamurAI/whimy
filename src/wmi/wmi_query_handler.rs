use crate::wmi::wmi_variant::{process_variant, WMIVariant};
use std::{boxed::Box, collections::HashMap, ffi::OsStr, os::windows::prelude::OsStrExt};
use tokio::task;
use windows::{
  core::{BSTR, PCWSTR},
  Win32::System::{
    Com::{
      CoCreateInstance, CoInitializeEx, CoInitializeSecurity, CoSetProxyBlanket, CoUninitialize,
      CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, EOAC_NONE, RPC_C_AUTHN_LEVEL_CALL,
      RPC_C_AUTHN_LEVEL_DEFAULT, RPC_C_IMP_LEVEL_IMPERSONATE, SAFEARRAY,
    },
    Ole::{SafeArrayDestroy, SafeArrayLock},
    Rpc::{RPC_C_AUTHN_NONE, RPC_C_AUTHN_WINNT},
    Variant::{VariantClear, VARIANT},
    Wmi::{
      IEnumWbemClassObject, IWbemClassObject, IWbemLocator, IWbemServices, WbemLocator,
      WBEM_CONDITION_FLAG_TYPE, WBEM_FLAG_FORWARD_ONLY, WBEM_FLAG_RETURN_IMMEDIATELY,
      WBEM_INFINITE,
    },
  },
};

pub type QueryResult = HashMap<String, Vec<Option<WMIVariant>>>;

#[derive(Debug)]
pub struct WMIQueryHandler {
  server: Option<IWbemServices>, // Allows us to deref by setting to None
}

impl WMIQueryHandler {
  pub fn new(namespace: String) -> napi::Result<Self> {
    unsafe {
      CoInitializeEx(None, COINIT_MULTITHREADED)
        .map_err(|_error| napi::Error::from_reason("Failed to Initialize COM"))?;
    };

    WMIQueryHandler::initialize_security()?;

    let server = WMIQueryHandler::connect_to_wmi_namespace(&namespace)?;

    unsafe {
      let _ = CoSetProxyBlanket(
        &server,
        RPC_C_AUTHN_WINNT,
        RPC_C_AUTHN_NONE,
        None,
        RPC_C_AUTHN_LEVEL_CALL,
        RPC_C_IMP_LEVEL_IMPERSONATE,
        None,
        EOAC_NONE,
      );
    };

    Ok(WMIQueryHandler {
      server: Some(server),
    })
  }

  pub fn stop(&mut self) {
    self.server = None;
    unsafe {
      CoUninitialize();
    };
  }

  pub fn execute_query(&self, query: String) -> napi::Result<QueryResult> {
    let mut results: QueryResult = HashMap::new();
    let query_execution = self.execute_wmi_query(&query)?;

    while let Some(row_results) = WMIQueryHandler::get_row_results(&query_execution)? {
      WMIQueryHandler::extract_variant_data_and_update_results(&mut results, &row_results)?;
    }
    Ok(results)
  }

  pub fn change_namespace(&mut self, namespace: &str) -> napi::Result<()> {
    let new_server = WMIQueryHandler::connect_to_wmi_namespace(namespace)?;
    unsafe {
      let _ = CoSetProxyBlanket(
        &new_server,
        RPC_C_AUTHN_WINNT,
        RPC_C_AUTHN_NONE,
        None,
        RPC_C_AUTHN_LEVEL_CALL,
        RPC_C_IMP_LEVEL_IMPERSONATE,
        None,
        EOAC_NONE,
      );
    };
    self.server = Some(new_server);
    Ok(())
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

  fn connect_to_wmi_namespace(namespace: &str) -> napi::Result<IWbemServices> {
    let locator: IWbemLocator =
      unsafe { CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER) }
        .map_err(|_e| napi::Error::new(napi::Status::GenericFailure, "Failed CoCreateInstance"))?;

    let server: IWbemServices =
      unsafe { locator.ConnectServer(&BSTR::from(namespace), None, None, None, 0, None, None) }
        .map_err(|_e| {
          napi::Error::new(
            napi::Status::GenericFailure,
            format!("Failed to connect to {} server", namespace),
          )
        })?;
    Ok(server)
  }

  fn execute_wmi_query(&self, query: &str) -> napi::Result<IEnumWbemClassObject> {
    let server = self
      .server
      .as_ref()
      .ok_or(napi::Error::from_reason("Server is None"))?;

    unsafe {
      server
        .ExecQuery(
          &BSTR::from("WQL"),
          &BSTR::from(query),
          WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY,
          None,
        )
        .map_err(|_error| {
          napi::Error::from_reason(format!(
            "Failed to execute WMI query '{}'. Original error: {}",
            query, _error
          ))
        })
    }
  }

  fn get_row_results(
    query_results: &IEnumWbemClassObject,
  ) -> napi::Result<Option<IWbemClassObject>> {
    let mut row = [None; 1];
    unsafe {
      query_results
        .Next(WBEM_INFINITE, &mut row, &mut 0)
        .ok()
        .map_err(|_error| {
          napi::Error::from_reason(format!(
            "Failed to fetch results for the query. The query may be invalid. Original error: {}",
            _error
          ))
        })?
    }

    Ok(row[0].to_owned())
  }

  fn create_safe_array_from_wbem_object<'a>(
    row: &IWbemClassObject,
    variant_ptr: *mut VARIANT,
  ) -> napi::Result<&'a SAFEARRAY> {
    let safe_array = unsafe {
      row
        .GetNames(None, WBEM_CONDITION_FLAG_TYPE(64), variant_ptr)
        .map_err(|_error| {
          napi::Error::from_reason(format!(
            "Failed to retrieve property names for the current row. Original error: {}",
            _error
          ))
        })?
    };

    unsafe {
      SafeArrayLock(safe_array).map_err(|_error| {
        napi::Error::from_reason(format!("Failed to access data. Original error: {}", _error))
      })?
    };
    let safe_array_ref = unsafe { safe_array.as_ref() };
    match safe_array_ref {
      Some(safe_array) => Ok(safe_array),
      None => Err(napi::Error::from_reason("Failed to retrieve the data array from the system. This may be due to an internal inconsistency or memory issue, check your query.")),
    }
  }

  fn get_variant_data(
    property_name: &str,
    current_row: &IWbemClassObject,
  ) -> napi::Result<Option<WMIVariant>> {
    let variant_value_data = Default::default();
    let variant_data_ptr: *mut VARIANT = Box::into_raw(Box::new(variant_value_data));

    let encoded_property_name: Vec<u16> = OsStr::new(&property_name)
      .encode_wide()
      .chain(Some(0))
      .collect();
    let property_name_ptr: PCWSTR = PCWSTR(encoded_property_name.as_ptr());

    unsafe {
      current_row
        .Get(property_name_ptr, 0, variant_data_ptr, None, None)
        .map_err(|_error| {
          napi::Error::from_reason(format!(
            "Failed to retrieve the value for property '{}'. Original error: {}",
            &property_name, _error
          ))
        })?;
    };
    let wmi_variant = unsafe { process_variant(&*variant_data_ptr) };
    unsafe {
      VariantClear(variant_data_ptr).map_err(|_error| {
          napi::Error::from_reason(format!(
              "An issue occurred while cleaning up data related to the property '{}'. Original system error: {}",
              &property_name, _error
          ))
      })?;
      let _ = Box::from_raw(variant_data_ptr);
    }

    Ok(wmi_variant)
  }

  fn extract_variant_data_and_update_results(
    query_result_map: &mut QueryResult,
    row_results: &IWbemClassObject,
  ) -> napi::Result<()> {
    let variant_value: VARIANT = Default::default();
    let variant_ptr: *mut VARIANT = Box::into_raw(Box::new(variant_value));

    let safe_array: &SAFEARRAY =
      WMIQueryHandler::create_safe_array_from_wbem_object(row_results, variant_ptr)?;

    for i in 0..safe_array.rgsabound[0].cElements as isize {
      let property_name = safe_array_to_string(safe_array, i);
      let wmi_variant = WMIQueryHandler::get_variant_data(&property_name, row_results)?;

      query_result_map
        .entry(property_name)
        .or_default()
        .push(wmi_variant);
    }

    // Clean up Variant, safe array, and pointer
    // If a VARIANT was created uninitialized don't call VariantClear. The .vt field would be empty and thus have garbage data.
    // In this case it's not an issue because Default zero's all fields thus making vt VT_EMPTY
    unsafe {
      VariantClear(variant_ptr).map_err(|_error| {
        napi::Error::from_reason(format!(
          "Failed to clear VARIANT resources. Original error: {}",
          _error
        ))
      })?;

      let _ = SafeArrayDestroy(safe_array);

      // Once this goes in a Box and leaves scope it will clean up the pointer
      let _ = Box::from_raw(variant_ptr);
    }
    Ok(())
  }
}

fn safe_array_to_string(safe_array: &SAFEARRAY, offset: isize) -> String {
  let safe_array_data_ptr = safe_array.pvData as *const *const u16;
  let property_name: *const u16 = unsafe { *((safe_array_data_ptr).offset(offset)) };
  let property_name_str = unsafe {
    let len = (0..)
      .take_while(|index| *property_name.offset(*index) != 0)
      .count();
    let slice = std::slice::from_raw_parts(property_name, len);
    String::from_utf16_lossy(slice)
  };
  property_name_str
}

#[cfg(test)]
mod test {
  use super::*;
  use std::{ffi::OsStr, os::windows::prelude::OsStrExt};
  use windows::Win32::System::{
    Com::SAFEARRAYBOUND,
    Ole::{SafeArrayCreate, SafeArrayUnlock},
    Variant::VT_UI2,
  };

  #[test]
  fn test_new_success() {
    let valid_wmi_query_handler = WMIQueryHandler::new(r#"root\cimv2"#.to_string());
    assert!(&valid_wmi_query_handler.is_ok());
    let mut query_handler = valid_wmi_query_handler.unwrap();
    query_handler.stop();
    assert!(query_handler.server.is_none());
  }

  #[test]
  fn failed_new_initialize_security() {
    let failed_initialize_security = WMIQueryHandler::new(r#"root\cimv2"#.to_string());
    assert!(failed_initialize_security.is_ok());
    let mut failed_initialize_query_handler = failed_initialize_security.unwrap();

    let too_many_inits = WMIQueryHandler::new(r#"root\cimv2"#.to_string());
    assert!(too_many_inits.is_err());
    assert!(&too_many_inits.is_err_and(|error| error.reason == "Failed to initialize security"));
    failed_initialize_query_handler.stop();
  }

  #[test]
  fn failed_new_bad_namespace() {
    let bad_namespace = WMIQueryHandler::new(r#"bad\name"#.to_string());
    assert!(
      &bad_namespace.is_err_and(|error| error.reason == "Failed to connect to bad\\name server")
    );
  }

  #[test]
  fn success_execute_query() {
    let mut wmi_query_handler = WMIQueryHandler::new(r#"root\cimv2"#.to_string()).unwrap();
    let result = wmi_query_handler
      .execute_query(r#"SELECT Name, State FROM Win32_Service WHERE Name='Winmgmt'"#.to_string());
    assert!(&result.is_ok());
    wmi_query_handler.stop();
  }

  #[test]
  fn failed_execute_query_wrong_namespace_query() {
    let expected_error_message =
      "Failed to fetch results for the query. The query may be invalid. Original error: 0x80041010";
    let mut wmi_query_handler = WMIQueryHandler::new(r#"root\cimv2"#.to_string()).unwrap();
    let result =
      wmi_query_handler.execute_query("SELECT ProductState FROM AntiVirusProduct".to_string());
    assert!(result.is_err_and(|error| error.reason == expected_error_message.to_string()));
    wmi_query_handler.stop();
  }

  #[test]
  fn success_change_namespace() {
    let mut wmi_query_handler = WMIQueryHandler::new(r#"root\cimv2"#.to_string()).unwrap();
    let result =
      wmi_query_handler.execute_query("SELECT Model FROM Win32_ComputerSystem".to_string());
    assert!(result.is_ok());
    let change_result = wmi_query_handler.change_namespace(r#"root\wmi"#);
    assert!(change_result.is_ok());
    let result = wmi_query_handler.execute_query("SELECT * FROM MS_SystemInformation".to_string());
    assert!(result.is_ok());
  }

  #[test]
  fn test_safe_array_to_string() {
    let test_string = "hello";
    let test_utf16: Vec<u16> = OsStr::new(test_string)
      .encode_wide()
      .chain(std::iter::once(0))
      .collect();
    let test_ptr: *const u16 = test_utf16.as_ptr();

    let bounds: [SAFEARRAYBOUND; 1] = [SAFEARRAYBOUND {
      cElements: 1,
      lLbound: 0,
    }];

    unsafe {
      let psa = SafeArrayCreate(VT_UI2, 1, bounds.as_ptr() as *mut SAFEARRAYBOUND);
      if !psa.is_null() {
        let pv_data: *mut *const u16 = (*psa).pvData as *mut *const u16;
        *pv_data = test_ptr;
        let _ = SafeArrayLock(psa).unwrap();
        let result = safe_array_to_string(psa.as_ref().unwrap(), 0);
        assert_eq!(result, "hello");
        let _ = SafeArrayUnlock(psa);
        let _ = SafeArrayDestroy(psa);
      }
    }
  }
}
