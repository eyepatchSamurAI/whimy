use std::collections::HashMap;
use std::ffi::{c_void, OsStr};
use std::fs;
use std::os::windows::prelude::OsStrExt;
use std::path::Path;
use windows::core::{GUID, HRESULT, PCSTR, PCWSTR};
use windows::Win32::Foundation::{
  CERT_E_CHAINING, CRYPT_E_FILE_ERROR, CRYPT_E_SECURITY_SETTINGS, ERROR_SUCCESS, HANDLE, HWND,
  TRUST_E_EXPLICIT_DISTRUST, TRUST_E_NOSIGNATURE, TRUST_E_PROVIDER_UNKNOWN,
  TRUST_E_SUBJECT_FORM_UNKNOWN, TRUST_E_SUBJECT_NOT_TRUSTED,
};
use windows::Win32::Security::Cryptography::{
  szOID_COMMON_NAME, szOID_COUNTRY_NAME, szOID_DEVICE_SERIAL_NUMBER, szOID_DOMAIN_COMPONENT,
  szOID_GIVEN_NAME, szOID_INITIALS, szOID_LOCALITY_NAME, szOID_ORGANIZATIONAL_UNIT_NAME,
  szOID_ORGANIZATION_NAME, szOID_RSA_emailAddr, szOID_STATE_OR_PROVINCE_NAME, szOID_STREET_ADDRESS,
  szOID_SUR_NAME, szOID_TITLE, CertGetNameStringW, CERT_CHAIN_CONTEXT, CERT_NAME_ATTR_TYPE,
};
use windows::Win32::Security::WinTrust::{
  WTHelperGetProvSignerFromChain, WTHelperProvDataFromStateData, WinVerifyTrust,
  WINTRUST_ACTION_GENERIC_VERIFY_V2, WINTRUST_DATA, WINTRUST_FILE_INFO, WTD_CHOICE_FILE,
  WTD_REVOCATION_CHECK_CHAIN_EXCLUDE_ROOT, WTD_REVOKE_WHOLECHAIN, WTD_STATEACTION_CLOSE,
  WTD_STATEACTION_VERIFY, WTD_UICONTEXT_EXECUTE, WTD_UI_NONE,
};

#[napi(object)]
#[derive(Debug, Clone, PartialEq)]
pub struct TrustStatus {
  pub signed: bool,
  pub message: String,
  pub subject: String,
}

impl TrustStatus {
  pub fn new() -> Self {
    TrustStatus {
      signed: false,
      message: String::new(),
      subject: String::new(),
    }
  }
}

impl Default for TrustStatus {
  fn default() -> Self {
    Self::new()
  }
}

pub fn verify_signature_by_publisher(
  file_path: String,
  publish_names: Vec<String>,
) -> napi::Result<TrustStatus> {
  let trimmed_path = file_path.trim_end();
  validate_signed_file(trimmed_path)?;
  let result = verify_signature_from_path(trimmed_path)?;
  if !result.signed {
    return Ok(result);
  }
  let parsed_subject = parse_dn(&result.subject);
  if publish_names
    .iter()
    .any(|name| check_dn_match(&parsed_subject, name).is_ok_and(|keys_match| keys_match))
  {
    return Ok(result.clone());
  }
  let final_subject = result.subject;

  Ok(TrustStatus {
    signed: false,
    message: "Publisher name does not match.".to_string(),
    subject: final_subject,
  })
}

pub fn verify_signature_from_path(file_path: &str) -> napi::Result<TrustStatus> {
  let mut trust_status = TrustStatus::default();

  let constant_wstring_bytes = OsStr::new(Path::new(&file_path))
    .encode_wide()
    .chain(Some(0)) // Add null terminating character
    .collect::<Vec<u16>>();

  let policy_guid: *mut GUID = &WINTRUST_ACTION_GENERIC_VERIFY_V2 as *const _ as *mut _;
  let mut win_trust_file_info = WINTRUST_FILE_INFO {
    cbStruct: std::mem::size_of::<WINTRUST_FILE_INFO>() as u32,
    pcwszFilePath: PCWSTR::from_raw(constant_wstring_bytes.as_ptr()), // You have to put the ptr in here otherwise  is pointing to invalid memory by the time WinVerifyTrust is called.
    // The as_ptr() method will give you a pointer to the data, but it won't prevent the data from being dropped when it goes out of scope.
    hFile: HANDLE::default(),
    ..Default::default()
  };
  let mut win_trust_data = WINTRUST_DATA {
    cbStruct: std::mem::size_of::<WINTRUST_DATA>() as u32,
    dwProvFlags: WTD_REVOCATION_CHECK_CHAIN_EXCLUDE_ROOT,
    dwUIChoice: WTD_UI_NONE,
    fdwRevocationChecks: WTD_REVOKE_WHOLECHAIN,
    dwUnionChoice: WTD_CHOICE_FILE,
    dwStateAction: WTD_STATEACTION_VERIFY,
    dwUIContext: WTD_UICONTEXT_EXECUTE,
    hWVTStateData: HANDLE::default(),
    ..Default::default()
  };
  win_trust_data.Anonymous.pFile = &mut win_trust_file_info as *mut _;
  let signature_status = unsafe {
    WinVerifyTrust(
      HWND::default(),
      policy_guid,
      &mut win_trust_data as *mut _ as *mut std::ffi::c_void,
    )
  };

  match HRESULT(signature_status) {
    x if x.0 == ERROR_SUCCESS.0 as i32 => {
      trust_status.signed = true;
      trust_status.message = "Verification succeeded!".to_string();
    }
    TRUST_E_NOSIGNATURE | TRUST_E_SUBJECT_FORM_UNKNOWN | TRUST_E_PROVIDER_UNKNOWN => {
      trust_status.message = "The file is not signed.".to_string();
      return Ok(trust_status);
    }
    TRUST_E_EXPLICIT_DISTRUST => {
      trust_status.message =
        "Signature is present but is specifically disallowed by admin or user.".to_string();
      return Ok(trust_status);
    }
    TRUST_E_SUBJECT_NOT_TRUSTED => {
      trust_status.message = "Signature is present but subject not trusted.".to_string();
      return Ok(trust_status);
    }
    CRYPT_E_SECURITY_SETTINGS => {
      trust_status.message = "Signature was not explictly trusted by admin, and user trust has been disabled. No signature, publisher, or timestamp error.".to_string();
      return Ok(trust_status);
    }
    CRYPT_E_FILE_ERROR => {
      trust_status.message = format!("CRYPT_E_FILE_ERROR: Signature was not explictly trusted by admin, and user trust has been disabled. No signature, publisher, or timestamp error. Original Error Code: {signature_status}");
      return Ok(trust_status);
    }
    CERT_E_CHAINING => {
      trust_status.message = format!("CERT_E_CHAINING: There was an error relating to the certificate chain for the signed file. Check if your certificate is in Root storage. Original Error Code: {signature_status}");
      return Ok(trust_status);
    }

    _ => {
      trust_status.message =
        format!("Unexpected error. Verification failed. Original Error Code: {signature_status}")
    }
  }
  let crypt_provider_data = unsafe { WTHelperProvDataFromStateData(win_trust_data.hWVTStateData) };

  if crypt_provider_data.is_null() {
    trust_status.signed = false;
    trust_status.message = "pProvData is null".to_string();
    return Ok(trust_status);
  }

  let crypt_provider_signer =
    unsafe { WTHelperGetProvSignerFromChain(crypt_provider_data, 0, false, 0) };
  if crypt_provider_signer.is_null() {
    trust_status.signed = false;
    trust_status.message = "sign subject is empty".to_string();
    return Ok(trust_status);
  }
  let sign_subject = unsafe {
    crypt_provider_signer
      .as_ref()
      .and_then(|signer| signer.pChainContext.as_ref())
      .map(|chain_context| get_certificate_subject(*chain_context))
  };
  trust_status.subject = sign_subject
    .as_ref()
    .filter(|s| !s.is_empty())
    .map_or("Sign subject info is empty.".to_string(), |s| s.clone());

  // Any hWVTStateData must be released by a call with close.
  win_trust_data.dwStateAction = WTD_STATEACTION_CLOSE;
  unsafe {
    WinVerifyTrust(
      HWND::default(),
      policy_guid,
      &mut win_trust_data as *mut WINTRUST_DATA as *mut c_void,
    );
  };

  Ok(trust_status)
}

pub fn allowed_extensions() -> Vec<String> {
  vec![
    "exe".to_string(),
    "cab".to_string(),
    "dll".to_string(),
    "ocx".to_string(),
    "msi".to_string(),
    "msix".to_string(),
    "xpi".to_string(),
  ]
}

/// x500 key refers wincrpyt.h
///
///  Key         Object Identifier               RDN Value Type(s)
///  ---         -----------------               -----------------
///  CN          szOID_COMMON_NAME               Printable, Unicode
///  L           szOID_LOCALITY_NAME             Printable, Unicode
///  O           szOID_ORGANIZATION_NAME         Printable, Unicode
///  OU          szOID_ORGANIZATIONAL_UNIT_NAME  Printable, Unicode
///  E           szOID_RSA_emailAddr             Only IA5
///  C           szOID_COUNTRY_NAME              Only Printable
///  S           szOID_STATE_OR_PROVINCE_NAME    Printable, Unicode
///  STREET      szOID_STREET_ADDRESS            Printable, Unicode
///  T           szOID_TITLE                     Printable, Unicode
///  G           szOID_GIVEN_NAME                Printable, Unicode
///  I           szOID_INITIALS                  Printable, Unicode
///  SN          szOID_SUR_NAME                  Printable, Unicode
///  DC          szOID_DOMAIN_COMPONENT          IA5, UTF8
///  SERIALNUMBER szOID_DEVICE_SERIAL_NUMBER     Only Printable
fn create_publisher_mapping() -> HashMap<String, PCSTR> {
  let publisher_attribute_key = vec![
    "CN".to_string(),
    "L".to_string(),
    "O".to_string(),
    "OU".to_string(),
    "E".to_string(),
    "C".to_string(),
    "S".to_string(),
    "STREET".to_string(),
    "T".to_string(),
    "G".to_string(),
    "I".to_string(),
    "SN".to_string(),
    "DC".to_string(),
    "SERIALNUMBER".to_string(),
  ];
  // Distinguished Name
  let dn_attribute_identifiers = [
    szOID_COMMON_NAME,
    szOID_LOCALITY_NAME,
    szOID_ORGANIZATION_NAME,
    szOID_ORGANIZATIONAL_UNIT_NAME,
    szOID_RSA_emailAddr,
    szOID_COUNTRY_NAME,
    szOID_STATE_OR_PROVINCE_NAME,
    szOID_STREET_ADDRESS,
    szOID_TITLE,
    szOID_GIVEN_NAME,
    szOID_INITIALS,
    szOID_SUR_NAME,
    szOID_DOMAIN_COMPONENT,
    szOID_DEVICE_SERIAL_NUMBER,
  ];
  let mut map = HashMap::new();
  for (key, value) in publisher_attribute_key
    .iter()
    .zip(dn_attribute_identifiers.iter())
  {
    let _ = &map.insert(key.to_owned(), value.to_owned());
  }
  map
}

fn parse_dn(seq: &str) -> HashMap<String, String> {
  let mut quoted: bool = false;
  let mut key: Option<String> = None;
  let mut token = String::new();
  let mut next_non_space = 0;

  let seq = seq.trim();
  let mut result = HashMap::new();
  let mut chars = seq.chars().peekable();

  while let Some(ch) = chars.next() {
    if quoted {
      if ch == '"' {
        quoted = false;
        continue;
      }
    } else {
      if ch == '"' {
        quoted = true;
        continue;
      }

      if ch == '\\' {
        if let Some(first) = chars.next() {
          // Only consider the next two characters as a hex sequence
          if let (Some(second), Some(third)) = (chars.next(), chars.next()) {
            let hex_str = format!("{}{}", second, third);
            if let Ok(ord) = u8::from_str_radix(&hex_str, 16) {
              token.push(char::from(ord));
              continue;
            }
          }
          // Not a hex sequence, put back the characters we took out
          token.push('\\');
          token.push(first);
        }
        continue;
      }

      if key.is_none() && ch == '=' {
        key = Some(token.clone());
        token.clear();
        continue;
      }

      if ch == ',' || ch == ';' || ch == '+' {
        if let Some(k) = key {
          result.insert(k, token.clone());
          key = None;
          token.clear();
        }
        continue;
      }
    }

    if ch == ' ' && !quoted {
      if token.is_empty() {
        continue;
      }

      if next_non_space > 0 {
        let mut j = next_non_space;
        while chars.peek() == Some(&' ') {
          j += 1;
          chars.next();
        }
        next_non_space = j;
      }

      let next_char = chars.peek().cloned().unwrap_or_default();
      if next_char == ','
        || next_char == ';'
        || (key.is_none() && next_char == '=')
        || (key.is_some() && next_char == '+')
      {
        next_non_space -= 1;
        continue;
      }
    }

    token.push(ch);
  }

  if let Some(k) = key {
    result.insert(k, token);
  }

  result
}

fn check_dn_match(subject: &HashMap<String, String>, name: &str) -> napi::Result<bool> {
  let distingusihed_names_map = parse_dn(name);
  if !distingusihed_names_map.is_empty() {
    Ok(distingusihed_names_map.keys().all(|attribute_type| {
      distingusihed_names_map.get(attribute_type) == subject.get(attribute_type)
    }))
  } else {
    Ok(Some(name) == subject.get("CN").map(|x| x.as_str()))
  }
}

fn validate_signed_file(path: &str) -> napi::Result<()> {
  let path = Path::new(path);
  let allowed_extensions = allowed_extensions();

  if !fs::metadata(path)?.is_file() {
    return Err(napi::Error::from_reason(format!(
      "Unable to locate target file {:?}",
      path
    )));
  }

  let file_extension = path
    .extension()
    .and_then(|s| s.to_str())
    .ok_or(napi::Error::from_reason("Failed to get file extension"))?;

  if !allowed_extensions.contains(&file_extension.to_string()) {
    return Err(napi::Error::from_reason(format!(
      "Accepted file types are: {}",
      allowed_extensions.join(",")
    )));
  }
  Ok(())
}

fn get_certificate_subject(cert_chain_context: CERT_CHAIN_CONTEXT) -> String {
  let mut subject: String = String::new();

  let publisher_chain = unsafe {
    cert_chain_context
      .rgpChain
      .as_ref()
      .and_then(|f| f.as_ref())
      .map(|b| b.rgpElement)
  };

  if let Some(publisher_chain_data) = publisher_chain {
    let publisher_cert_context = unsafe {
      publisher_chain_data
        .as_ref()
        .and_then(|f| f.as_ref())
        .map(|f| f.pCertContext)
    };
    if let Some(publisher_cert_context_data) = publisher_cert_context {
      let publisher_mapping = create_publisher_mapping();
      publisher_mapping
        .iter()
        .for_each(|(distingusihed_name_attribute, &distingusihed_name)| {
          // Get the length of the subject name attribute
          let subject_name_att_length = unsafe {
            CertGetNameStringW(
              publisher_cert_context_data,
              CERT_NAME_ATTR_TYPE,
              0,
              Some(distingusihed_name.as_ptr() as *const c_void),
              None,
            )
          };
          if subject_name_att_length <= 1 {
            return;
          }
          let mut buffer: Vec<u16> = vec![0; subject_name_att_length as usize];
          let attr_string_lengtht = unsafe {
            CertGetNameStringW(
              publisher_cert_context_data,
              CERT_NAME_ATTR_TYPE,
              0,
              Some(distingusihed_name.as_ptr() as *const c_void),
              Some(buffer.as_mut_slice()),
            )
          };
          if attr_string_lengtht <= 1 {
            return;
          }
          let mut attribute_string = String::from_utf16_lossy(&buffer);
          attribute_string = attribute_string.trim_end_matches('\0').to_string();
          attribute_string = format!("=\"{}\",", attribute_string);

          subject += &format!("{}{}", distingusihed_name_attribute, attribute_string);
        });
    }
  }
  subject
}

#[cfg(test)]
mod test {

  use super::*;

  const SIGNED_PATH: &str = "./test_signed_data/signed_exes";

  fn assert_string_hashmap_eq(
    map1: HashMap<String, String>,
    map2: HashMap<String, String>,
  ) -> bool {
    if map1.len() != map2.len() {
      return false;
    }
    for (key, value1) in &map1 {
      match map2.get(key) {
        Some(value2) => {
          if value1 != value2 {
            return false;
          }
        }
        None => return false,
      }
    }

    true
  }

  fn assert_trust_status_eq(trusted_status: &TrustStatus, expected: &TrustStatus) -> bool {
    println!("expected {:#?}", expected);
    println!("trusted_status {:#?}", trusted_status);
    if trusted_status.subject.len() != expected.subject.len() {
      return false;
    }
    if trusted_status.signed != expected.signed || trusted_status.message != expected.message {
      return false;
    }
    let expected_subject_map = parse_dn(&expected.subject);
    let trusted_subject_map = parse_dn(&trusted_status.subject);

    println!("expected_subject_map {:?}", expected_subject_map);
    println!("trusted_subject_map {:?}", trusted_subject_map);

    assert_string_hashmap_eq(expected_subject_map, trusted_subject_map)
  }

  #[test]
  fn parse_test() {
    let dn_multiple_names = "CN=TestGroup,OU=Groups,OU=UT-SLC,OU=US,DC=Company,DC=com";

    let mut dn_multiple_names_expected = HashMap::new();
    dn_multiple_names_expected.insert("CN".to_string(), "TestGroup".to_string());
    dn_multiple_names_expected.insert("DC".to_string(), "com".to_string());
    dn_multiple_names_expected.insert("OU".to_string(), "US".to_string());

    let dn_multiple_names_result = parse_dn(&dn_multiple_names);
    assert_eq!(dn_multiple_names_result, dn_multiple_names_expected);

    let dn_with_quotes = String::from(
      r#"CN="Microsoft Corporation",L="Redmond",O="Microsoft Corporation",OU="Microsoft Corporation",C="US",S="Washington""#,
    );
    let mut dn_with_quotes_expected = HashMap::new();
    dn_with_quotes_expected.insert("S".to_string(), "Washington".to_string());
    dn_with_quotes_expected.insert("L".to_string(), "Redmond".to_string());
    dn_with_quotes_expected.insert("CN".to_string(), "Microsoft Corporation".to_string());
    dn_with_quotes_expected.insert("OU".to_string(), "Microsoft Corporation".to_string());
    dn_with_quotes_expected.insert("O".to_string(), "Microsoft Corporation".to_string());
    dn_with_quotes_expected.insert("C".to_string(), "US".to_string());

    let dn_with_quotes_result = parse_dn(&dn_with_quotes);
    assert_eq!(dn_with_quotes_result, dn_with_quotes_expected);

    let dn_multiple_names =
      r#"CN="TestGroup",OU="Groups",O="TotallyFakeTestDomain , Inc.",DC="Company""#;
    let mut dn_multiple_names_expected = HashMap::new();
    dn_multiple_names_expected.insert("CN".to_string(), "TestGroup".to_string());
    dn_multiple_names_expected.insert("DC".to_string(), "Company".to_string());
    dn_multiple_names_expected.insert("OU".to_string(), "Groups".to_string());
    dn_multiple_names_expected.insert("O".to_string(), "TotallyFakeTestDomain , Inc.".to_string());

    let dn_multiple_names_result = parse_dn(&dn_multiple_names);

    assert_eq!(dn_multiple_names_result, dn_multiple_names_expected);
  }

  #[test]
  fn test_parse_dn_with_escaped_characters() {
    // Test case where hex representation is valid
    let dn_hex_valid = r#"CN=Test\x20Group,OU=Groups,DC=Company"#;
    let mut dn_hex_valid_expected = HashMap::new();
    dn_hex_valid_expected.insert("CN".to_string(), "Test Group".to_string());
    dn_hex_valid_expected.insert("OU".to_string(), "Groups".to_string());
    dn_hex_valid_expected.insert("DC".to_string(), "Company".to_string());

    let dn_hex_valid_result = parse_dn(&dn_hex_valid);
    assert_eq!(dn_hex_valid_result, dn_hex_valid_expected);
  }

  #[test]
  fn test_signature_for_exe() {
    let file_path = format!("{SIGNED_PATH}/microsoft_signed.exe");
    let publisher_names = vec![String::from(
      r#"CN="Microsoft Corporation",L="Redmond",O="Microsoft Corporation",OU="Microsoft Corporation",C="US",S="Washington""#,
    )];
    let signature_status = verify_signature_by_publisher(file_path.to_string(), publisher_names);
    let expected_subject = String::from(
      r#"S="Washington",L="Redmond",OU="Microsoft Corporation",C="US",CN="Microsoft Corporation",SERIALNUMBER="230865+470561",O="Microsoft Corporation","#,
    );
    let expected = TrustStatus {
      message: "Verification succeeded!".to_string(),
      subject: expected_subject,
      signed: true,
    };
    assert!(signature_status.is_ok());
    assert!(assert_trust_status_eq(
      &signature_status.unwrap(),
      &expected
    ));
  }

  #[test]
  fn test_signature_for_dll() {
    let file_path = format!("{SIGNED_PATH}/signed.dll");
    let publisher_names = vec![String::from(
      r#"CN="Valve",L="Bellevue",O="Valve",C="US",S="WA""#,
    )];
    let signature_status = verify_signature_by_publisher(file_path.to_string(), publisher_names);
    let expected_subject = String::from(r#"S="WA",L="Bellevue",C="US",CN="Valve",O="Valve","#);
    let expected = TrustStatus {
      message: "Verification succeeded!".to_string(),
      subject: expected_subject,
      signed: true,
    };
    assert!(signature_status.is_ok());
    assert!(assert_trust_status_eq(
      &signature_status.unwrap(),
      &expected
    ));
  }

  #[test]
  fn test_unsigned() {
    let file_path = format!("{SIGNED_PATH}/unsigned.dll");
    let publisher_names = vec![String::from(
      r#"CN="Microsoft Corporation",L="Redmond",O="Microsoft Corporation",OU="Microsoft Corporation",C="US",S="Washington""#,
    )];
    let signature_status = verify_signature_by_publisher(file_path.to_string(), publisher_names);

    let expected = TrustStatus {
      message: "The file is not signed.".to_string(),
      subject: "".to_string(),
      signed: false,
    };
    assert!(signature_status.is_ok());
    assert!(assert_trust_status_eq(
      &signature_status.unwrap(),
      &expected
    ));
  }
  #[test]
  fn test_incorrect_extension() {
    let file_path = format!("{SIGNED_PATH}/empty.txt");
    let publisher_names = vec![String::from(r#"CN="Microsoft Corporation""#)];
    let signature_status = verify_signature_by_publisher(file_path.to_string(), publisher_names);
    let expected_error_string = "Accepted file types are: exe,cab,dll,ocx,msi,msix,xpi";

    assert!(&signature_status.is_err());
    assert_eq!(signature_status.unwrap_err().reason, expected_error_string);
  }

  #[test]
  fn test_not_a_file() {
    let file_path = SIGNED_PATH;
    let publisher_names = vec![String::from(r#"CN="Microsoft Corporation""#)];
    let signature_status = verify_signature_by_publisher(file_path.to_string(), publisher_names);
    let expected_error_string = r#"Unable to locate target file "./test_signed_data/signed_exes""#;

    assert!(&signature_status.is_err());
    assert_eq!(signature_status.unwrap_err().reason, expected_error_string);
  }

  #[test]
  fn test_custom_signature() {
    println!("If if the custom signature test fails, try running setting_up_cert_testing.ps1"); // Maybe don't keep because this is more about setup than the test. Prob write a dev setup
                                                                                                // If this test is failing try running setting_up_cert_testing.ps1
    let file_path = format!("{SIGNED_PATH}/custom_signed_exe.exe");
    let publisher_names = vec![String::from(
      r#"O="TotallyFakeTestDomain, Inc.",C=US,CN=TotallyFakeTestDomain.com"#,
    )];
    let signature_status = verify_signature_by_publisher(file_path.to_string(), publisher_names);
    let expected_subject =
      String::from(r#"O="TotallyFakeTestDomain, Inc.",C="US",CN="TotallyFakeTestDomain.com","#);
    let expected = TrustStatus {
      message: "Verification succeeded!".to_string(),
      subject: expected_subject,
      signed: true,
    };
    assert!(signature_status.is_ok());
    assert!(assert_trust_status_eq(
      &signature_status.unwrap(),
      &expected
    ));
  }

  #[test]
  fn test_signature_does_not_match() {
    let correct_publisher_subject = r#"S="Washington",L="Redmond",OU="Microsoft Corporation",C="US",CN="Microsoft Corporation",SERIALNUMBER="230865+470561",O="Microsoft Corporation","#;
    let incorrect_publisher_subject = r#"CN="Microsoft Corporationn",L="Redmondd",O="Microsoft Corporationn",OU="Microsoft Corporationn",C="US",S="Washington""#;

    let file_path = format!("{SIGNED_PATH}/microsoft_signed.exe");
    let publisher_names = vec![String::from(incorrect_publisher_subject)];
    let signature_status = verify_signature_by_publisher(file_path.to_string(), publisher_names);
    let expected = TrustStatus {
      message: "Publisher name does not match.".to_string(),
      // message: format!("Publisher name does not match.\n\n Given: {correct_publisher_subject} \n\n Expected: {incorrect_publisher_subject}"),
      subject: correct_publisher_subject.to_string(),
      signed: false,
    };
    assert!(signature_status.is_ok());
    assert!(assert_trust_status_eq(
      &signature_status.unwrap(),
      &expected
    ));
  }
}
