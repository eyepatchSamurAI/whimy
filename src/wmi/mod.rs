mod wmi_query_handler;
mod wmi_variant;

use napi::Result;
use wmi_query_handler::WMIQueryHandler;

#[napi]
pub struct Wmi {
  query_handler: WMIQueryHandler,
}

#[napi]
impl Wmi {
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
  #[napi(constructor)]
  pub fn new(service_type: String) -> Result<Self> {
    Ok(Wmi {
      query_handler: WMIQueryHandler::new(service_type)?,
    })
  }

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
  #[napi]
  pub fn query(&self, query: Option<String>) -> Result<String> {
    match query {
      Some(query_string) => {
        let result = self.query_handler.execute_query(query_string)?;
        let json = serde_json::to_string(&result).map_err(|error| {
          napi::Error::from_reason(format!(
            "Failed to stringify query result, Original Error: {}",
            error
          ))
        })?;
        Ok(json)
      }
      None => Err(napi::Error::new(
        napi::Status::GenericFailure,
        "query parameter is not provided",
      )),
    }
  }

  #[napi]
  pub fn change_namespace(&mut self, namespace: String) -> napi::Result<()> {
    let _ = &self.query_handler.change_namespace(&namespace)?;
    Ok(())
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
  pub fn stop(&mut self) {
    self.query_handler.stop();
  }
}