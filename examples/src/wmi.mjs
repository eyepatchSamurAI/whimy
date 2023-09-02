import { Wmi } from "whimy"

// Sync Query
(()=> {
    const wmi = new Wmi(`root\\cimv2`);
    const result = wmi.syncQuery("select Name, ProcessId from Win32_Process");
    console.log("result: ", result);
    wmi.stop();
})()

// Async Query
(async ()=> {
    const result = await Wmi.asyncQuery(`root\\cimv2`, "select Name, ProcessId from Win32_Process");
    console.log("result: ", result);
})()