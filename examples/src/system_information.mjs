import { Wmi } from "whimy"

// Get a list of the current processes
(()=> {
    const wmi = new Wmi(`root\\cimv2`);
    const currently_running_processes = wmi.syncQuery("select * from Win32_Process");
    console.log("result: ", currently_running_processes);
    wmi.stop();
})();


// Get various system information asynchronously
(async ()=> {
    const namespace = `root\\cimv2`;
    const commands = [
        "Select BatteryStatus from Win32_Battery", // If you have a laptop 
        "Select * from win32_networkadapter WHERE GUID IS NOT NULL",
        "Select FreeSpace, Size from Win32_logicaldisk",
        "Select Caption from Win32_OperatingSystem",
    ];
    const promiseResults = commands.map((command)=> {
        return Wmi.asyncQuery(namespace, command);
    });
    const results = await Promise.all(promiseResults);
    console.log(JSON.stringify(results, null, 2));
})();
