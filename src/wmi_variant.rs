use serde::{Serialize, Serializer};
use windows::Win32::System::Variant::{
  VARIANT, VT_BOOL, VT_BSTR, VT_EMPTY, VT_I2, VT_I4, VT_I8, VT_NULL, VT_R4, VT_R8, VT_UI1, VT_UINT,
};

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

impl Serialize for WMIVariant {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    match self {
      WMIVariant::BStr(s) => serializer.serialize_str(s),
      WMIVariant::I2(i) => serializer.serialize_i16(*i),
      WMIVariant::I4(i) => serializer.serialize_i32(*i),
      WMIVariant::I8(i) => serializer.serialize_i64(*i),
      WMIVariant::R4(i) => serializer.serialize_i32(*i),
      WMIVariant::R8(f) => serializer.serialize_f64(*f),
      WMIVariant::Uint(u) => serializer.serialize_u32(*u),
      WMIVariant::UI1(u) => serializer.serialize_u8(*u),
      WMIVariant::Bool(b) => serializer.serialize_bool(*b),
    }
  }
}
