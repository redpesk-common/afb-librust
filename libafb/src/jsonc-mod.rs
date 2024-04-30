/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * $RP_BEGIN_LICENSE$
 * Commercial License Usage
 *  Licensees holding valid commercial IoT.bzh licenses may use this file in
 *  accordance with the commercial license agreement provided with the
 *  Software or, alternatively, in accordance with the terms contained in
 *  a written agreement between you and The IoT.bzh Company. For licensing terms
 *  and conditions see https://www.iot.bzh/terms-conditions. For further
 *  information use the contact form at https://www.iot.bzh/contact.
 *
 * GNU General Public License Usage
 *  Alternatively, this file may be used under the terms of the GNU General
 *  Public license version 3. This license is as published by the Free Software
 *  Foundation and appearing in the file LICENSE.GPLv3 included in the packaging
 *  of this file. Please review the following information to ensure the GNU
 *  General Public License requirements will be met
 *  https://www.gnu.org/licenses/gpl-3.0.html.
 * $RP_END_LICENSE$
 */

use crate::prelude::*;

use std::ffi::{CStr, CString};
use std::fmt;
use std::os::raw::c_char;

#[derive(Debug)]
pub enum Jtype {
    Array = cglue::json_type_json_type_array as isize,
    String = cglue::json_type_json_type_string as isize,
    Bool = cglue::json_type_json_type_boolean as isize,
    Int = cglue::json_type_json_type_int as isize,
    Float = cglue::json_type_json_type_double as isize,
    Object = cglue::json_type_json_type_object as isize,
    Null = cglue::json_type_json_type_null as isize,
    Unknown = -1,
}
pub struct JsonStr(pub &'static str);
pub enum Jobject {
    String(String),
    Int(i64),
    Bool(bool),
    Float(f64),
    Object(JsoncObj),
    Array(JsoncObj),
    Null(),
    Unknown(&'static str),
}

#[track_caller]
pub fn to_static_str(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}

// convert an hexadecimal string "01:02:...:xx" into an &[u8] slice
#[track_caller]
pub fn hexa_to_bytes<'a>(input: &str, buffer: &'a mut [u8]) -> Result<&'a[u8], AfbError> {
    let mut idx = 0;
    for hexa in input[1..input.len() - 1].split(',') {
        if idx == buffer.len() {
            return afb_error!(
                "string-decode-hexa",
                "destination buffer too small size:{}",
                buffer.len()
            );
        }
        match u8::from_str_radix(hexa, 16) {
            Ok(value) => buffer[idx] = value,
            Err(_) => {
                return afb_error!(
                    "string-ecode-hexa",
                    "invalid hexa encoding syntax: '[01,ff,...]'"
                )
            }
        }
        idx = idx + 1;
    }
    Ok(&buffer[0..idx])
}

#[track_caller]
pub fn bytes_to_hexa(buffer: &[u8]) -> String {
    format!("{:02x?}", buffer).replace(" ", "")
}

pub struct JsoncObj {
    jso: *mut cglue::json_object,
}

// minimal internal jsonc object to external crates
pub type JsoncJso = cglue::json_object;

pub struct Jentry {
    pub key: String,
    pub obj: JsoncObj,
}

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn free_jsonc_cb(jso: *mut std::ffi::c_void) {
    unsafe { cglue::json_object_put(jso as *mut cglue::json_object) };
}

#[doc(hidden)]
impl Drop for JsoncObj {
    fn drop(&mut self) {
        unsafe {
            cglue::json_object_put(self.jso);
        }
    }
}

#[doc(hidden)]
impl Clone for JsoncObj {
    fn clone(&self) -> Self {
        unsafe {
            let jsonc = JsoncObj {
                jso: cglue::json_object_get(self.jso),
            };
            return jsonc;
        }
    }
}

impl fmt::Display for JsoncObj {
    fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cstring;
        unsafe {
            // warning: no ';'
            let jso = &mut *(self.jso as *mut cglue::json_object);
            let cbuffer = if format.alternate() {
                // {:#}
                cglue::json_object_to_json_string_ext(jso, cglue::JSON_C_TO_STRING_PRETTY as i32)
            } else {
                // {}
                cglue::json_object_to_json_string_ext(
                    jso,
                    (cglue::JSON_C_TO_STRING_NOSLASHESCAPE | cglue::JSON_C_TO_STRING_NOZERO) as i32,
                )
            };
            cstring = CStr::from_ptr(cbuffer);
        };

        // pas de ; fait sur le write fait return
        write!(format, "{}", cstring.to_str().unwrap())

        // Fulup should free cbuffer
    }
}

pub trait DoPutJso<T> {
    fn put_jso(jso: *mut cglue::json_object) -> Result<T, AfbError>;
}

impl DoPutJso<String> for JsoncObj {
    #[track_caller]
    fn put_jso(jso: *mut cglue::json_object) -> Result<String, AfbError> {
        if unsafe { cglue::json_object_get_type(jso) } != cglue::json_type_json_type_string {
            afb_error!("jsonc-get-type", "jsonc object is not a string",)
        } else {
            Ok(JsoncObj::to_string(jso))
        }
    }
}

impl DoPutJso<&'static str> for JsoncObj {
    #[track_caller]
    fn put_jso(jso: *mut cglue::json_object) -> Result<&'static str, AfbError> {
        if unsafe { cglue::json_object_get_type(jso) } != cglue::json_type_json_type_string {
            afb_error!("jsonc-get-type", "jsonc object is not a string",)
        } else {
            Ok(to_static_str(JsoncObj::to_string(jso)))
        }
    }
}

impl DoPutJso<i64> for JsoncObj {
    #[track_caller]
    fn put_jso(jso: *mut cglue::json_object) -> Result<i64, AfbError> {
        if unsafe { cglue::json_object_get_type(jso) } != cglue::json_type_json_type_int {
            afb_error!("jsonc-get-type", "jsonc object is not an integer",)
        } else {
            Ok(unsafe { cglue::json_object_get_int64(jso) })
        }
    }
}

impl DoPutJso<u64> for JsoncObj {
    #[track_caller]
    fn put_jso(jso: *mut cglue::json_object) -> Result<u64, AfbError> {
        if unsafe { cglue::json_object_get_type(jso) } != cglue::json_type_json_type_int {
            afb_error!("jsonc-get-type", "jsonc object is not an unsigned",)
        } else {
            Ok(unsafe { cglue::json_object_get_int64(jso) } as u64)
        }
    }
}

impl DoPutJso<i32> for JsoncObj {
    #[track_caller]
    fn put_jso(jso: *mut cglue::json_object) -> Result<i32, AfbError> {
        if unsafe { cglue::json_object_get_type(jso) } != cglue::json_type_json_type_int {
            afb_error!("jsonc-get-type", "jsonc object is not an integer",)
        } else {
            Ok(unsafe { cglue::json_object_get_int(jso) })
        }
    }
}

impl DoPutJso<u32> for JsoncObj {
    #[track_caller]
    fn put_jso(jso: *mut cglue::json_object) -> Result<u32, AfbError> {
        if unsafe { cglue::json_object_get_type(jso) } != cglue::json_type_json_type_int {
            afb_error!("jsonc-get-type", "jsonc object is not integer",)
        } else {
            Ok(unsafe { cglue::json_object_get_int(jso) as u32 })
        }
    }
}

impl DoPutJso<i16> for JsoncObj {
    #[track_caller]
    fn put_jso(jso: *mut cglue::json_object) -> Result<i16, AfbError> {
        if unsafe { cglue::json_object_get_type(jso) } != cglue::json_type_json_type_int {
            afb_error!("jsonc-get-type", "jsonc object is not an integer",)
        } else {
            let value = unsafe { cglue::json_object_get_int(jso) };
            if value > std::i16::MAX as i32 || value < std::i16::MIN as i32 {
                return afb_error!("jsonc::get<i16>", "multiplier should be i16 get:{}", value);
            }
            Ok(value as i16)
        }
    }
}

impl DoPutJso<u8> for JsoncObj {
    #[track_caller]
    fn put_jso(jso: *mut cglue::json_object) -> Result<u8, AfbError> {
        if unsafe { cglue::json_object_get_type(jso) } != cglue::json_type_json_type_int {
            afb_error!("jsonc-get-type", "jsonc object is not an integer",)
        } else {
            let value = unsafe { cglue::json_object_get_int(jso) };
            if value > std::u8::MAX as i32 || value < std::u8::MIN as i32 {
                return afb_error!("jsonc::get<u8>", "multiplier should be u8 get:{}", value);
            }
            Ok(value as u8)
        }
    }
}

impl DoPutJso<i8> for JsoncObj {
    #[track_caller]
    fn put_jso(jso: *mut cglue::json_object) -> Result<i8, AfbError> {
        if unsafe { cglue::json_object_get_type(jso) } != cglue::json_type_json_type_int {
            afb_error!("jsonc-get-type", "jsonc object is not an integer",)
        } else {
            let value = unsafe { cglue::json_object_get_int(jso) };
            if value > std::i8::MAX as i32 || value < std::i8::MIN as i32 {
                return afb_error!("jsonc::get<i8>", "multiplier should be i8 get:{}", value);
            }
            Ok(value as i8)
        }
    }
}

impl DoPutJso<u16> for JsoncObj {
    #[track_caller]
    fn put_jso(jso: *mut cglue::json_object) -> Result<u16, AfbError> {
        if unsafe { cglue::json_object_get_type(jso) } != cglue::json_type_json_type_int {
            afb_error!("jsonc-get-type", "jsonc object is not an integer",)
        } else {
            let value = unsafe { cglue::json_object_get_int(jso) };
            if value > std::u16::MAX as i32 || value < std::u16::MIN as i32 {
                return afb_error!("jsonc::get<u16>", "multiplier should be u16 get:{}", value);
            }
            Ok(value as u16)
        }
    }
}

impl DoPutJso<bool> for JsoncObj {
    #[track_caller]
    fn put_jso(jso: *mut cglue::json_object) -> Result<bool, AfbError> {
        if unsafe { cglue::json_object_get_type(jso) } != cglue::json_type_json_type_boolean {
            afb_error!("jsonc-get-type", "jsonc object is not boolean")
        } else {
            Ok(unsafe { cglue::json_object_get_boolean(jso) != 0 })
        }
    }
}

impl DoPutJso<f64> for JsoncObj {
    #[track_caller]
    fn put_jso(jso: *mut cglue::json_object) -> Result<f64, AfbError> {
        if unsafe { cglue::json_object_get_type(jso) } != cglue::json_type_json_type_double {
            afb_error!("jsonc-get-type", "jsonc object is not a float",)
        } else {
            Ok(unsafe { cglue::json_object_get_double(jso) })
        }
    }
}

impl DoPutJso<Jobject> for JsoncObj {
    #[track_caller]
    fn put_jso(jso: *mut cglue::json_object) -> Result<Jobject, AfbError> {
        if jso as usize == 0 {
            afb_error!("jsonc-get-key", "not object at key")
        } else {
            Ok(JsoncObj::get_jso_value(jso))
        }
    }
}

#[doc(hidden)]
pub trait DoGetJso<K> {
    #[track_caller]
    fn get_jso(key: K, jso: *const cglue::json_object)
        -> Result<*mut cglue::json_object, AfbError>;
}

impl DoPutJso<JsoncObj> for JsoncObj {
    #[track_caller]
    fn put_jso(jso: *mut cglue::json_object) -> Result<JsoncObj, AfbError> {
        Ok(JsoncObj::from(jso as *mut std::ffi::c_void))
    }
}

impl DoGetJso<&str> for JsoncObj {
    #[track_caller]
    fn get_jso(
        key: &str,
        jso: *const cglue::json_object,
    ) -> Result<*mut cglue::json_object, AfbError> {
        let skey = CString::new(key).expect("Invalid jsonc key string");
        let result;
        unsafe {
            let jslot: *mut cglue::json_object = 0 as *mut cglue::json_object;

            if cglue::json_object_object_get_ex(
                jso,
                skey.into_raw(),
                &jslot as *const _ as *mut *mut cglue::json_object,
            ) == 0
            {
                result = afb_error!("jconc-key-missing", key.to_string());
            } else {
                result = Ok(jslot);
            }
        }
        return result;
    }
}

impl DoGetJso<usize> for JsoncObj {
    #[track_caller]
    fn get_jso(
        idx: usize,
        jso: *const cglue::json_object,
    ) -> Result<*mut cglue::json_object, AfbError> {
        unsafe {
            if idx > cglue::json_object_array_length(jso) {
                afb_error!("jsonc-array-size", "jsonc array index out of bound")
            } else {
                Ok(cglue::json_object_array_get_idx(jso, idx))
            }
        }
    }
}

#[doc(hidden)]
pub trait DoAddon<T> {
    #[track_caller]
    fn add(&self, key: &str, value: T);
    fn insert(&self, value: T);
}


impl DoAddon<bool> for JsoncObj {
    #[track_caller]
    fn add(&self, key: &str, value: bool) {
        unsafe {
            let object = cglue::json_object_new_boolean(value as i32);
            self.add_to_object(key, object);
        }
    }
    #[track_caller]
    fn insert(&self, value: bool) {
        unsafe {
            let object = cglue::json_object_new_boolean(value as i32);
            self.add_to_array(object);
        }
    }
}

impl DoAddon<f64> for JsoncObj {
    #[track_caller]
    fn add(&self, key: &str, value: f64) {
        unsafe {
            let object = cglue::json_object_new_double(value);
            self.add_to_object(key, object);
        }
    }

    #[track_caller]
    fn insert(&self, value: f64) {
        unsafe {
            let object = cglue::json_object_new_double(value);
            self.add_to_array(object);
        }
    }
}

impl DoAddon<i64> for JsoncObj {
    #[track_caller]
    fn add(&self, key: &str, value: i64) {
        unsafe {
            let object = cglue::json_object_new_int64(value);
            self.add_to_object(key, object);
        }
    }
    #[track_caller]
    fn insert(&self, value: i64) {
        unsafe {
            let object = cglue::json_object_new_int64(value);
            self.add_to_array(object);
        }
    }
}

impl DoAddon<u64> for JsoncObj {
    #[track_caller]
    fn add(&self, key: &str, value: u64) {
        unsafe {
            let object = cglue::json_object_new_int64(value as i64);
            self.add_to_object(key, object);
        }
    }
    #[track_caller]
    fn insert(&self, value: u64) {
        unsafe {
            let object = cglue::json_object_new_int64(value as i64);
            self.add_to_array(object);
        }
    }
}

impl DoAddon<i32> for JsoncObj {
    #[track_caller]
    fn add(&self, key: &str, value: i32) {
        unsafe {
            let object = cglue::json_object_new_int(value);
            self.add_to_object(key, object);
        }
    }
    #[track_caller]
    fn insert(&self, value: i32) {
        unsafe {
            let object = cglue::json_object_new_int(value);
            self.add_to_array(object);
        }
    }
}

impl DoAddon<u32> for JsoncObj {
    #[track_caller]
    fn add(&self, key: &str, value: u32) {
        DoAddon::add(self, key, value as i64)
    }
    #[track_caller]
    fn insert(&self, value: u32) {
        DoAddon::insert(self, value as i64)
    }
}

impl DoAddon<u16> for JsoncObj {
    #[track_caller]
    fn add(&self, key: &str, value: u16) {
        DoAddon::add(self, key, value as u32)
    }
    #[track_caller]
    fn insert(&self, value: u16) {
        DoAddon::insert(self, value as u32)
    }
}

impl DoAddon<i16> for JsoncObj {
    #[track_caller]
    fn add(&self, key: &str, value: i16) {
        DoAddon::add(self, key, value as i32)
    }
    #[track_caller]
    fn insert(&self, value: i16) {
        DoAddon::insert(self, value as i32)
    }
}

impl DoAddon<u8> for JsoncObj {
    #[track_caller]
    fn add(&self, key: &str, value: u8) {
        DoAddon::add(self, key, value as u32)
    }
    #[track_caller]
    fn insert(&self, value: u8) {
        DoAddon::insert(self, value as u32)
    }
}

impl DoAddon<i8> for JsoncObj {
    #[track_caller]
    fn add(&self, key: &str, value: i8) {
        DoAddon::add(self, key, value as i32)
    }
    #[track_caller]
    fn insert(&self, value: i8) {
        DoAddon::insert(self, value as i32)
    }
}

impl DoAddon<usize> for JsoncObj {
    #[track_caller]
    fn add(&self, key: &str, value: usize) {
        DoAddon::add(self, key, value as i64)
    }
    fn insert(&self, value: usize) {
        DoAddon::insert(self, value as i64)
    }
}

impl DoAddon<&str> for JsoncObj {
    #[track_caller]
    fn add(&self, key: &str, value: &str) {
        let sval = CString::new(value).expect("Invalid jsonc key string");
        unsafe {
            let object = cglue::json_object_new_string(sval.into_raw());
            self.add_to_object(key, object);
        }
    }
    #[track_caller]
    fn insert(&self, value: &str) {
        let sval = CString::new(value).expect("Invalid jsonc key string");
        unsafe {
            let object = cglue::json_object_new_string(sval.into_raw());
            self.add_to_array(object);
        }
    }
}

impl DoAddon<&String> for JsoncObj {
    #[track_caller]
    fn add(&self, key: &str, value: &String) {
        DoAddon::add(self, key, value.as_str())
    }
    #[track_caller]
    fn insert(&self, value: &String) {
        DoAddon::insert(self, value.as_str())
    }
}

impl DoAddon<&JsoncObj> for JsoncObj {
    #[track_caller]
    fn add(&self, key: &str, object: &JsoncObj) {
        self.add_to_object(key, object.jso);
    }
    #[track_caller]
    fn insert(&self, object: &JsoncObj) {
        self.add_to_array(object.jso);
    }
}

impl DoAddon<JsoncObj> for JsoncObj {
    #[track_caller]
    fn add(&self, key: &str, object: JsoncObj) {
        self.add_to_object(key, object.jso);
    }
    #[track_caller]
    fn insert(&self, object: JsoncObj) {
        self.add_to_array(object.jso);
    }
}

impl From<i64> for JsoncObj {
    #[track_caller]
    fn from(value: i64) -> Self {
        unsafe {
            let jsonc = JsoncObj {
                jso: cglue::json_object_new_int64(value),
            };
            return jsonc;
        }
    }
}

impl From<&JsonStr> for JsoncObj {
    #[track_caller]
    fn from(value: &JsonStr) -> Self {
        match JsoncObj::parse(value.0) {
            Err(error) => panic!("(hoops: invalid json error={}", error),
            Ok(jsonc) => jsonc,
        }
    }
}

impl From<*mut std::ffi::c_void> for JsoncObj {
    #[track_caller]
    fn from(value: *mut std::ffi::c_void) -> Self {
        let jso: &mut cglue::json_object = unsafe { &mut *(value as *mut cglue::json_object) };
        unsafe {
            let jsonc = JsoncObj { jso: jso };
            cglue::json_object_get(jsonc.jso);
            return jsonc;
        }
    }
}

impl From<f64> for JsoncObj {
    #[track_caller]
    fn from(value: f64) -> Self {
        unsafe {
            let jsonc = JsoncObj {
                jso: cglue::json_object_new_double(value),
            };
            cglue::json_object_get(jsonc.jso);
            return jsonc;
        }
    }
}

impl From<&str> for JsoncObj {
    #[track_caller]
    fn from(value: &str) -> Self {
        let sval = CString::new(value).expect("Invalid jsonc key string");
        unsafe {
            let jsonc = JsoncObj {
                jso: cglue::json_object_new_string(sval.into_raw()),
            };
            cglue::json_object_get(jsonc.jso);
            return jsonc;
        }
    }
}

impl From<&String> for JsoncObj {
    #[track_caller]
    fn from(value: &String) -> Self {
        let sval = CString::new(value.as_str()).expect("Invalid jsonc key string");
        unsafe {
            let jsonc = JsoncObj {
                jso: cglue::json_object_new_string(sval.into_raw()),
            };
            cglue::json_object_get(jsonc.jso);
            return jsonc;
        }
    }
}

impl JsoncObj {
    #[track_caller]
    pub fn new() -> JsoncObj {
        unsafe {
            let jsonc = JsoncObj {
                jso: cglue::json_object_new_object(),
            };
            cglue::json_object_get(jsonc.jso);
            return jsonc;
        }
    }
    #[track_caller]
    pub fn array() -> JsoncObj {
        unsafe {
            let jsonc = JsoncObj {
                jso: cglue::json_object_new_array(),
            };
            cglue::json_object_get(jsonc.jso);
            return jsonc;
        }
    }

    #[track_caller]
    pub fn from<T>(args: T) -> JsoncObj
    where
        T: Into<JsoncObj>,
    {
        args.into()
    }

    #[track_caller]
    pub fn get_as<T>(&self) -> Result<T, AfbError>
    where
        JsoncObj: DoPutJso<T>,
    {
        Self::put_jso(self.jso)
    }

    #[track_caller]
    pub fn add<T>(&self, key: &str, value: T) -> Result<&Self, AfbError>
    where
        JsoncObj: DoAddon<T>,
    {
        unsafe {
            let result =
                if cglue::json_object_is_type(self.jso, cglue::json_type_json_type_object) == 0 {
                    afb_error!("jsonc-add-fail", "jsonc target is not an object")
                } else {
                    DoAddon::add(self, key, value);
                    Ok(self)
                };
            return result;
        }
    }

    #[track_caller]
    pub fn get_jso_value(jso: *mut cglue::json_object) -> Jobject {
        let result;
        unsafe {
            match JsoncObj::get_jso_type(jso) {
                Jtype::Bool => {
                    result = Jobject::Bool(if cglue::json_object_get_boolean(jso) == 0 {
                        false
                    } else {
                        true
                    })
                }
                Jtype::Int => result = Jobject::Int(cglue::json_object_get_int64(jso)),
                Jtype::Float => result = Jobject::Float(cglue::json_object_get_double(jso)),
                Jtype::String => {
                    let cbuffer = cglue::json_object_get_string(jso);
                    let cstring = CStr::from_ptr(cbuffer);
                    let slice: &str = cstring.to_str().unwrap();
                    result = Jobject::String(slice.to_owned());
                }
                Jtype::Array => result = Jobject::Array(JsoncObj { jso: jso }),
                Jtype::Object => result = Jobject::Object(JsoncObj { jso: jso }),
                Jtype::Null => result = Jobject::Null(),
                _ => result = Jobject::Unknown("jsonc unknown type"),
            }
        }
        return result;
    }

    #[track_caller]
    pub fn to_string(jso: *mut cglue::json_object) -> String {
        let result;
        unsafe {
            let cbuffer = cglue::json_object_get_string(jso);
            let cstring = CStr::from_ptr(cbuffer);
            let slice: &str = cstring.to_str().unwrap();
            result = slice.to_owned();
        }
        return result;
    }

    #[track_caller]
    pub fn get<T>(&self, key: &str) -> Result<T, AfbError>
    where
        JsoncObj: DoPutJso<T>,
    {
        match Self::get_jso(key, self.jso) {
            Err(error) => Err(error),
            Ok(jso) => Self::put_jso(jso),
        }
    }

    #[track_caller]
    pub fn default<T>(&self, key: &str, default: T) -> Result<T, AfbError>
    where
        JsoncObj: DoPutJso<T>,
    {
        match Self::get_jso(key, self.jso) {
            Err(_error) => Ok(default),
            Ok(jso) => Self::put_jso(jso),
        }
    }

    #[track_caller]
    pub fn optional<T>(&self, key: &str) -> Result<Option<T>, AfbError>
    where
        JsoncObj: DoPutJso<T>,
    {
        match Self::get_jso(key, self.jso) {
            Err(_error) => Ok(None),
            Ok(jso) => Ok(Some(Self::put_jso(jso)?)),
        }
    }

    #[track_caller]
    pub fn index<T>(&self, index: usize) -> Result<T, AfbError>
    where
        JsoncObj: DoPutJso<T>,
    {
        match Self::get_jso(index, self.jso) {
            Err(error) => Err(error),
            Ok(jso) => Self::put_jso(jso),
        }
    }

    #[track_caller]
    pub fn count(&self) -> Result<usize, AfbError> {
        unsafe {
            match JsoncObj::get_jso_type(self.jso) {
                Jtype::Array => Ok(cglue::json_object_array_length(self.jso)),
                Jtype::Object => Ok(cglue::json_object_object_length(self.jso) as usize),
                _ => afb_error!("jsonc-count-fail", "jsonc is neither object or array",),
            }
        }
    }

    #[track_caller]
    pub fn insert<T>(&self, value: T) -> Result<&Self, AfbError>
    where
        JsoncObj: DoAddon<T>,
    {
        unsafe {
            let result =
                if cglue::json_object_is_type(self.jso, cglue::json_type_json_type_array) == 0 {
                    afb_error!("jsonc-insert-fail", "jsonc target is not an array",)
                } else {
                    DoAddon::insert(self, value);
                    Ok(self)
                };
            return result;
        }
    }

    #[track_caller]
    pub fn into_raw(&self) -> *mut cglue::json_object {
        unsafe { cglue::json_object_get(self.jso) };
        self.jso
    }

    #[track_caller]
    fn add_to_object(&self, key: &str, jval: *mut cglue::json_object) {
        let skey = CString::new(key).expect("Invalid jsonc key string");
        unsafe {
            cglue::json_object_object_add(self.jso, skey.into_raw(), jval);
        }
    }

    #[track_caller]
    fn add_to_array(&self, jval: *mut cglue::json_object) {
        unsafe {
            cglue::json_object_array_add(self.jso, jval);
        }
    }

    #[track_caller]
    pub fn is_type(&self, jtype: Jtype) -> bool {
        unsafe {
            if cglue::json_object_is_type(self.jso, jtype as u32) != 0 {
                true
            } else {
                false
            }
        }
    }

    #[track_caller]
    fn get_jso_type(jso: *const cglue::json_object) -> Jtype {
        unsafe {
            let value = cglue::json_object_get_type(jso);
            match value {
                value if value == Jtype::Array as u32 => Jtype::Array,
                value if value == Jtype::String as u32 => Jtype::String,
                value if value == Jtype::Bool as u32 => Jtype::Bool,
                value if value == Jtype::Int as u32 => Jtype::Int,
                value if value == Jtype::Object as u32 => Jtype::Object,
                value if value == Jtype::Float as u32 => Jtype::Float,
                value if value == Jtype::Null as u32 => Jtype::Null,
                _ => Jtype::Unknown,
            }
        }
    }

    #[track_caller]
    pub fn get_type(&self) -> Jtype {
        JsoncObj::get_jso_type(self.jso)
    }

    #[track_caller]
    pub fn expand(&self) -> Vec<Jentry> {
        // if not object return now
        if !self.is_type(Jtype::Object) {
            return Vec::new();
        }

        let mut jvec = Vec::new();
        let mut entry = unsafe { (*cglue::json_object_get_object(self.jso)).head };
        while entry != 0 as *mut cglue::lh_entry {
            let key = unsafe {
                CStr::from_ptr((*entry).k as *const Cchar)
                    .to_owned()
                    .to_str()
                    .unwrap()
                    .to_owned()
            };
            let obj = unsafe { JsoncObj::from((*entry).v as *mut std::ffi::c_void) };
            jvec.push(Jentry { key: key, obj: obj });
            entry = unsafe { (*entry).next };
        }
        jvec
    }

    #[track_caller]
    pub fn equal(&mut self, jsonc: JsoncObj) -> Result<(), AfbError> {
        if unsafe { cglue::json_object_get_type(self.jso) }
            != unsafe { cglue::json_object_get_type(jsonc.jso) }
        {
            afb_error!("jsonc::equal", "jtype diverge")
        } else if self.to_string() != jsonc.to_string() {
            afb_error!(",jsonc::equal", "jtype not equal")
        } else {
            Ok(())
        }
    }

    #[track_caller]
    pub fn contains(&mut self, jtok: JsoncObj) -> Result<(), AfbError> {
        let jvec = jtok.expand();
        for entry in &jvec {
            match self.get::<JsoncObj>(entry.key.as_str()) {
                Err(error) => {
                    return Err(error);
                }
                Ok(value) => {
                    let kval = entry.obj.to_string();
                    let tval = value.to_string();
                    if kval != tval {
                        return afb_error!("jsonc-contains-fail", "json-token-not-found");
                    }
                }
            }
        }
        Ok(())
    }
    #[track_caller]
    pub fn parse(json_str: &str) -> Result<JsoncObj, AfbError> {
        unsafe {
            let tok = cglue::json_tokener_new();
            let jsonc = JsoncObj {
                jso: cglue::json_tokener_parse_ex(
                    tok,
                    json_str.as_bytes().as_ptr() as *mut c_char,
                    json_str.len() as i32,
                ),
            };

            let jerr = cglue::json_tokener_get_error(tok);

            // warning no ';'
            let result = if jerr != cglue::json_tokener_error_json_tokener_success {
                afb_error!("jsonc-parse-fail", json_str)
            } else {
                cglue::json_object_get(jsonc.jso);
                Ok(jsonc)
            };

            cglue::json_tokener_free(tok);
            return result;
        }
    }
}
