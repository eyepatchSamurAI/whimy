[![npm version](https://badge.fury.io/js/whimy.svg)](https://badge.fury.io/js/whimy)
![License: MIT](https://img.shields.io/badge/License-MIT-brightgreen.svg)
[![whimy CI/CD](https://github.com/eyepatchSamurAI/whimy/actions/workflows/ci-prod.yml/badge.svg)](https://github.com/eyepatchSamurAI/whimy/actions/workflows/ci-prod.yml)
[![codecov](https://codecov.io/gh/eyepatchSamurAI/whimy/graph/badge.svg?token=WCSPL1LGEF)](https://codecov.io/gh/eyepatchSamurAI/whimy)

# Whimy

In the landscape of Node.js projects that interface with Windows systems, it's common to rely on PowerShell commands for retrieving essential system information. However, this approach poses a security risk and is usually much slower.

Underneath the hood, Whimy leverages Windows Management Instrumentation (WMI) to fetch system data, bypassing the need for potentially hazardous PowerShell commands.

By adopting Whimy, developers gain:

- 🔒 Enhanced Security: Eliminate the security vulnerabilities associated with executing PowerShell commands.
- 🚀 Improved Performance: Benefit from the efficiency of direct WMI calls, significantly speeding up data retrieval.

## Features

- Easy querying of WMI services.
- JSON-parsable response for WMI queries.
- File signature verification based on publisher names.

## Usage

### WMI Querying

```typescript
import { Wmi } from 'whimy';

const wmi = new Wmi(`root\\cimv2`);
const queryString = wmi.syncQuery("Select * From Win32_processor");
const query = JSON.parse(queryString);
console.log(query);
wmi.stop();
```

### File Signature Verification

```typescript
import { verifySignatureByPublishName } from "whimy";

const filePath = resolve(directoryName, 'path/to/file.exe');
const output = verifySignatureByPublishName(filePath, ['CN="Microsoft Corporation",O="Microsoft Corporation",L=Redmond,S=Washington,C=US']);
console.log(output);
```

### Use Cases

The most obvious use case is to gather system information, see the examples on how it can be done.
## Installation

`npm install whimy`

## Contributing

Any contributions are welcome! You can either create an issue or you can create a pull request yourself.
