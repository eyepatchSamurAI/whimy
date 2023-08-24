import { verifySignatureByPublishName } from "whimy"
import { fileURLToPath } from 'url';
import {resolve, dirname } from "path"

(()=> {
    const filename = fileURLToPath(import.meta.url);
    const directoryName = dirname(filename);
    const filePath = resolve(directoryName, '../../test_signed_data/signed_exes/microsoft_signed.exe');
    const output = verifySignatureByPublishName(filePath, ['CN="Microsoft Corporation",O="Microsoft Corporation",L=Redmond,S=Washington,C=US"'])
    console.log(output); 
})()