![License: MIT](https://img.shields.io/badge/License-MIT-brightgreen.svg)

# Whimy

A collection of low level Windows tools exposed to node

## Features

- Easy querying of WMI services.
- JSON-parsable response for WMI queries.
- File signature verification based on publisher names.

## Usage

### WMI Querying

```typescript
import { Wmi } from 'whimy';

const wmi = new Wmi(`root\\cimv2`);
const queryString = wmi.query("Select * From Win32_processor");
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

## Installation

`npm install whimy`

## Contributing

Any contributions are welcome! You can either make an issue or you can create a pull request yourself.
