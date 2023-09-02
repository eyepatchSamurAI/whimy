mod wmi_query_handler;
mod wmi_variant;

use napi::{Result, bindgen_prelude::AsyncTask};
use wmi_query_handler::{QueryResult, WMIQueryHandler};

use self::wmi_query_handler::AsyncWMIQuery;

#[napi]
pub struct Wmi {
  query_handler: WMIQueryHandler,
}

#[napi]
impl Wmi {
  /// Allows the querying of an internal server that WMI can connect to
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
  /// Note: when passing in the service you want to query use string literals
  ///
  #[napi(constructor)]
  pub fn new(namespace: String) -> Result<Self> {
    Ok(Wmi {
      query_handler: WMIQueryHandler::new(namespace)?,
    })
  }

  /// Creates an open connection with the Windows Management Instrumentation (WMI) service
  /// and executes a synchronous query on the specified WMI namespace.
  /// 
  /// Note: Make sure to call `stop()` to release resources when you're done with WMI operations.
  /// ```
  /// import { Wmi } from 'wmi'
  /// const wmi = new Wmi(`root\\cimv2`);
  /// const queryObject = wmi.syncQuery("Select * From Win32_processor");
  /// wmi.stop();
  /// ```
  ///
  #[napi]
  pub fn sync_query(&self, query: String) -> Result<QueryResult> {
    self.query_handler.execute_query(query)
  }
  
  /// Asynchronously query the WMI service within a specified namespace to retrieve JSON-formatted management data.
  /// Does not open a continuous connection. Once queried the WMI resources are released
  ///
  /// This function is non-blocking and returns an `AsyncTask` that you can await on the JavaScript side.
  /// Ideal for running WMI queries that might take some time to complete, without blocking the main thread.
  ///
  /// ```javascript
  /// import { Wmi } from 'wmi';
  ///
  /// async function fetchData() {
  ///   const asyncQueryTask = Wmi.asyncQuery(`root\\cimv2`, "SELECT * FROM Win32_Processor");
  ///   const queryResult = await asyncQueryTask;
  ///   // Process queryResult
  /// }
  ///
  /// fetchData();
  /// ```
  ///
  #[napi]
  pub fn async_query(namespace: String, query: String) -> AsyncTask<AsyncWMIQuery> {
    AsyncTask::new(AsyncWMIQuery {namespace, query})
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
