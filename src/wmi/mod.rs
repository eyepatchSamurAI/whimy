mod wmi_query_handler;
mod wmi_variant;

use napi::Result;
use wmi_query_handler::{QueryResult, WMIQueryHandler};

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

  /// Query the COM service previously initialized and get back a json response
  ///
  /// ```
  /// import {WMI} from 'wmi'
  /// const wmi = new WMI(`root\\cimv2`);
  /// const queryObject = wmi.query("Select * From Win32_processor");
  /// wmi.stop();
  /// ```
  ///
  #[napi]
  pub fn query(&self, query: String) -> Result<QueryResult> {
    self.query_handler.execute_query(query)
  }

  /// Change the namespace you are querying without having to make a new instance of Wmi
  ///
  /// ```
  /// import {WMI} from 'wmi'
  /// const wmi = new WMI(`root\\cimv2`);
  /// const queryObject = wmi.query("Select * From Win32_processor");
  /// wmi.changeNamespace(`root\\wmi`);
  /// const newQueryObject = wmi.query("SELECT * FROM MSMouse"); // cimv2 cannot make this query
  /// wmi.stop();
  /// ```
  ///
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
