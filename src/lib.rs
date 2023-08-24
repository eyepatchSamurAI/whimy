#![deny(clippy::all)]

mod verify_signature;
mod wmi_query_handler;
mod wmi_variant;

#[macro_use]
extern crate napi_derive;

use napi::Result;
use verify_signature::TrustStatus;
use wmi_query_handler::WMIQueryHandler;
use crate::verify_signature::verify_signature;

#[napi]
pub struct Wmi {
  /// Handles query calls to WMI
  query_handler: WMIQueryHandler,
}

#[napi]
impl Wmi {
  #[napi(constructor)]
  /// Allows the querying of an interal server that WMI can conenct to
  /// It's important that once you are done with making WMI calls you call stop()
  /// ```
  /// import {WMI} from 'wmi'
  /// const wmi = new WMI(`root\\cimv2`);
  ///
  /// const data1 = wmi.query('query...');
  /// const data2 = wmi.query('query...');
  /// // Done with making calls to WMI
  /// wmi.stop();
  /// ```
  ///
  /// Note: when passing in the service you watn to query use string literals
  ///
  pub fn new(service_type: String) -> Result<Self> {
    Ok(Wmi {
      query_handler: WMIQueryHandler::new(service_type)?,
    })
  }

  #[napi]
  /// Query the COM service previously initalted and get back a json parsable response
  ///
  /// ```
  /// import {WMI} from 'wmi'
  /// const wmi = new WMI(`root\\cimv2`);
  /// const queryString = wmi.query("Select * From Win32_processor");
  /// const query = JSON.parse(queryString);
  /// wmi.stop();
  /// ```
  ///
  pub fn query(&self, query: Option<String>) -> Result<String> {
    match query {
      Some(query_string) => {
        let result = self.query_handler.execute_query(query_string)?;
        let json = serde_json::to_string(&result).unwrap();
        Ok(json)
      }
      None => {
        return Err(napi::Error::new(
          napi::Status::GenericFailure,
          "query parameter is not provided",
        ));
      }
    }
  }
  
  /// Stop the connection between your server and COM
  /// Must be called after you are done making calls to Windows otherwise the COM library will remain locked
  /// This can result is resource leakage or degrade performance.
  /// ```
  /// import {Wmi} from 'whimy'
  /// const wmi = new Wmi(`root\\wmi`);
  ///
  /// const data = wmi.query('Select * from WmiMonitorConnectionParams');
  /// // Done with making calls to WMI
  /// wmi.stop(); // Very important
  /// ```
  /// 
  #[napi]
  pub fn stop(&self) {
    self.query_handler.stop();
  }
}

/// Verifies a file given a list of publisher names
/// ```
/// import { verifySignatureByPublishName } from "whimy"
/// const filePath = resolve(directoryName, '../../test_signed_data/signed_exes/microsoft_signed.exe');
/// const output = verifySignatureByPublishName(filePath, ['CN="Microsoft Corporation",O="Microsoft Corporation",L=Redmond,S=Washington,C=US"'])
/// console.log(output); 
/// ```
/// 
#[napi]
pub fn verify_signature_by_publish_name(file_path: String, publish_names: Vec<String>) -> Result<TrustStatus>{
  verify_signature(file_path, publish_names)
}