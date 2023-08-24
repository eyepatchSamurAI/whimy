import { Wmi } from "whimy"

(()=> {
    const wmi = new Wmi(`root\\cimv2`);
    const result = wmi.query("select Name, ProcessId from Win32_Process");
    console.log("result: ", JSON.parse(result));
    wmi.stop();
})()