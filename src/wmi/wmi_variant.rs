use napi::{
  bindgen_prelude::{check_status, ToNapiValue},
  sys,
};
use std::ptr;
use windows::Win32::System::Variant::{
  VARIANT, VT_BOOL, VT_BSTR, VT_EMPTY, VT_I2, VT_I4, VT_I8, VT_NULL, VT_R4, VT_R8, VT_UI1, VT_UINT,
};

// Is there a faster method? This is not any faster than just using serde_json::to_string
impl ToNapiValue for WMIVariant {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> napi::Result<sys::napi_value> {
    let mut raw_value = ptr::null_mut();

    match val {
      WMIVariant::BStr(s) => {
        let utf16_data: Vec<u16> = s.encode_utf16().collect();
        check_status!(unsafe {
          sys::napi_create_string_utf16(env, utf16_data.as_ptr(), utf16_data.len(), &mut raw_value)
        })?;

        Ok(raw_value)
      }
      WMIVariant::I2(i) => {
        check_status!(unsafe { sys::napi_create_int32(env, i.into(), &mut raw_value) })?;
        Ok(raw_value)
      }
      WMIVariant::I4(i) => {
        check_status!(unsafe { sys::napi_create_int32(env, i, &mut raw_value) })?;
        Ok(raw_value)
      }
      WMIVariant::I8(i) => {
        check_status!(unsafe { sys::napi_create_int64(env, i, &mut raw_value) })?;
        Ok(raw_value)
      }
      WMIVariant::R4(i) => {
        check_status!(unsafe { sys::napi_create_int32(env, i, &mut raw_value) })?;
        Ok(raw_value)
      }
      WMIVariant::R8(i) => {
        check_status!(unsafe { sys::napi_create_double(env, i, &mut raw_value) })?;
        Ok(raw_value)
      }
      WMIVariant::Uint(i) => {
        check_status!(unsafe { sys::napi_create_uint32(env, i, &mut raw_value) })?;
        Ok(raw_value)
      }
      WMIVariant::UI1(i) => {
        check_status!(unsafe { sys::napi_create_uint32(env, i.into(), &mut raw_value) })?;
        Ok(raw_value)
      }
      WMIVariant::Bool(b) => {
        check_status!(unsafe { sys::napi_get_boolean(env, b, &mut raw_value) })?;
        Ok(raw_value)
      }
    }
  }
}

#[derive(Debug)]
pub enum WMIVariant {
  BStr(String),
  I2(i16),
  I4(i32),
  I8(i64),
  R4(i32),
  R8(f64),
  Uint(u32),
  UI1(u8),
  Bool(bool),
}

/// Convert a Windows Variant type into a type we can work with
pub fn process_variant(value_data: &VARIANT) -> Option<WMIVariant> {
  let variant_type = unsafe { value_data.Anonymous.Anonymous.vt };
  match variant_type {
    VT_BSTR => {
      let bstr = unsafe { (*(value_data.Anonymous.Anonymous.Anonymous.bstrVal)).to_string() };
      Some(WMIVariant::BStr(bstr.to_string()))
    }
    VT_I2 => {
      let int_val = unsafe { value_data.Anonymous.Anonymous.Anonymous.iVal };
      Some(WMIVariant::I2(int_val))
    }
    VT_I4 => {
      let int_val = unsafe { value_data.Anonymous.Anonymous.Anonymous.lVal };
      Some(WMIVariant::I4(int_val))
    }
    VT_I8 => {
      let int_val = unsafe { value_data.Anonymous.Anonymous.Anonymous.llVal };
      Some(WMIVariant::I8(int_val))
    }
    VT_R4 => {
      let int_val = unsafe { value_data.Anonymous.Anonymous.Anonymous.intVal };
      Some(WMIVariant::R4(int_val))
    }
    VT_R8 => {
      let int_val = unsafe { value_data.Anonymous.Anonymous.Anonymous.dblVal };
      Some(WMIVariant::R8(int_val))
    }
    VT_UINT => {
      let int_val = unsafe { value_data.Anonymous.Anonymous.Anonymous.uintVal };
      Some(WMIVariant::Uint(int_val))
    }
    VT_UI1 => {
      let int_val = unsafe { value_data.Anonymous.Anonymous.Anonymous.bVal };
      Some(WMIVariant::UI1(int_val))
    }
    VT_BOOL => {
      let bool_val = unsafe { value_data.Anonymous.Anonymous.Anonymous.boolVal.as_bool() };
      Some(WMIVariant::Bool(bool_val))
    }
    VT_EMPTY => None,
    VT_NULL => None,
    _ => None,
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use std::mem::ManuallyDrop;
  use windows::{
    core::BSTR,
    Win32::{
      Foundation::VARIANT_BOOL,
      System::Variant::{VARENUM, VARIANT_0, VARIANT_0_0, VARIANT_0_0_0},
    },
  };

  fn create_variant(vt: VARENUM, value: WMIVariant) -> VARIANT {
    let inner_value = match value {
      WMIVariant::BStr(_bstr) => VARIANT_0_0_0 {
        bstrVal: ManuallyDrop::new(BSTR::new()),
      },
      WMIVariant::I2(i) => VARIANT_0_0_0 { iVal: i },
      WMIVariant::I4(i) => VARIANT_0_0_0 { lVal: i },
      WMIVariant::I8(i) => VARIANT_0_0_0 { llVal: i },
      WMIVariant::R4(i) => VARIANT_0_0_0 { intVal: i },
      WMIVariant::R8(i) => VARIANT_0_0_0 { dblVal: i },
      WMIVariant::Uint(i) => VARIANT_0_0_0 { uintVal: i },
      WMIVariant::UI1(i) => VARIANT_0_0_0 { bVal: i },
      WMIVariant::Bool(_i) => VARIANT_0_0_0 {
        boolVal: VARIANT_BOOL::default(),
      },
    };

    VARIANT {
      Anonymous: VARIANT_0 {
        Anonymous: ManuallyDrop::new(VARIANT_0_0 {
          vt,
          Anonymous: inner_value,
          ..Default::default()
        }),
      },
    }
  }

  #[test]
  fn test_process_variant_bstr() {
    let mock_variant = create_variant(VT_BSTR, WMIVariant::BStr("hello".to_string()));
    let result = process_variant(&mock_variant);
    assert!(result.is_some());
  }
  #[test]
  fn test_process_variant_i2() {
    let mock_variant = create_variant(VT_I2, WMIVariant::I2(32));
    let result = process_variant(&mock_variant);
    assert!(result.is_some());
  }
  #[test]
  fn test_process_variant_i4() {
    let mock_variant = create_variant(VT_I4, WMIVariant::I4(32));
    let result = process_variant(&mock_variant);
    assert!(result.is_some());
  }
  #[test]
  fn test_process_variant_i8() {
    let mock_variant = create_variant(VT_I8, WMIVariant::I8(32));
    let result = process_variant(&mock_variant);
    assert!(result.is_some());
  }
  #[test]
  fn test_process_variant_r4() {
    let mock_variant = create_variant(VT_R4, WMIVariant::R4(32));
    let result = process_variant(&mock_variant);
    assert!(result.is_some());
  }
  #[test]
  fn test_process_variant_r8() {
    let mock_variant = create_variant(VT_R8, WMIVariant::R8(32.0));
    let result = process_variant(&mock_variant);
    assert!(result.is_some());
  }
  #[test]
  fn test_process_variant_uint() {
    let mock_variant = create_variant(VT_UINT, WMIVariant::Uint(32));
    let result = process_variant(&mock_variant);
    assert!(result.is_some());
  }
  #[test]
  fn test_process_variant_ui1() {
    let mock_variant = create_variant(VT_UI1, WMIVariant::UI1(32));
    let result = process_variant(&mock_variant);
    assert!(result.is_some());
  }
  #[test]
  fn test_process_variant_bool() {
    let mock_variant = create_variant(VT_BOOL, WMIVariant::Bool(true));
    let result = process_variant(&mock_variant);
    assert!(result.is_some());
  }

  #[test]
  fn test_process_variant_empty() {
    let mock_variant = VARIANT {
      Anonymous: VARIANT_0 {
        Anonymous: ManuallyDrop::new(VARIANT_0_0 {
          vt: VT_EMPTY,
          ..Default::default()
        }),
      },
    };
    let result = process_variant(&mock_variant);
    assert!(result.is_none());
  }
  #[test]
  fn test_process_variant_null() {
    let mock_variant = VARIANT {
      Anonymous: VARIANT_0 {
        Anonymous: ManuallyDrop::new(VARIANT_0_0 {
          vt: VT_NULL,
          ..Default::default()
        }),
      },
    };
    let result = process_variant(&mock_variant);
    assert!(result.is_none());
  }
  #[test]
  fn test_process_variant_other() {
    let mock_variant = VARIANT {
      Anonymous: VARIANT_0 {
        Anonymous: ManuallyDrop::new(VARIANT_0_0 {
          vt: VARENUM(999),
          ..Default::default()
        }),
      },
    };
    let result = process_variant(&mock_variant);
    assert!(result.is_none());
  }
}
