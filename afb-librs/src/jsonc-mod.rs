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
use std::str;

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Jequal {
    Full,
    Partial,
}

#[track_caller]
pub fn to_static_str(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}

pub fn bytes_to_str(data: &[u8]) -> Result<&str, AfbError> {
    let text = match str::from_utf8(data) {
        Ok(value) => value,
        Err(_) => return afb_error!("bytes_to_str", "not a valid UTF string"),
    };
    Ok(text)
}

// convert an hexadecimal string "[01,02,...,xx]" into an &[u8] slice
#[track_caller]
pub fn hexa_to_bytes<'a>(input: &str, buffer: &'a mut [u8]) -> Result<&'a [u8], AfbError> {
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
                    "invalid hexa encoding syntax: '[01,ff,...]' got:{}",
                    input
                )
            }
        }
        idx += 1;
    }
    Ok(&buffer[0..idx])
}

#[track_caller]
pub fn bytes_to_hexa(buffer: &[u8]) -> String {
    format!("{:02x?}", buffer).replace(" ", "")
}

fn cmp_entry<'a>(value: &'a Jentry, expect: &Jentry) -> Option<&'a Jentry> {
    if value.key == expect.key {
        Some(value)
    } else {
        None
    }
}

pub struct JsoncObj {
    jso: *mut cglue::json_object,
}

// minimal internal jsonc object to external crates
pub type JsoncJso = cglue::json_object;
pub type JsonSortCb = cglue::json_object_sort_cb;

pub struct Jentry {
    pub key: String,
    pub obj: JsoncObj,
}

#[no_mangle]
pub extern "C" fn free_jsonc_cb(jso: *mut std::ffi::c_void) {
    let jval = jso as *mut JsoncJso;
    //println! ("free_jsonc_cb:{}", JsoncObj{jso: jval});
    unsafe { cglue::json_object_put(jval) };
}

impl Drop for JsoncObj {
    fn drop(&mut self) {
        unsafe {
            cglue::json_object_put(self.jso);
        }
    }
}

impl Clone for JsoncObj {
    fn clone(&self) -> Self {
        unsafe {
            // Clone by bumping refcount
            JsoncObj {
                jso: cglue::json_object_get(self.jso),
            }
        }
    }
}

impl fmt::Display for JsoncObj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Convert the underlying json-c object to a C string buffer.
        //
        // Safety:
        // - `self.jso` must be a valid `*mut json_object` (owned/retained elsewhere).
        // - `json_object_to_json_string_ext` returns a pointer to an internal
        //   null-terminated buffer owned by json-c; it must NOT be freed by Rust.
        // - The buffer remains valid until the json-c object is modified or freed.
        let cstring = unsafe {
            let jso = self.jso;
            let flags = if f.alternate() {
                // `"{:#}"` → pretty-printed JSON, without slash escaping.
                (cglue::JSON_C_TO_STRING_PRETTY | cglue::JSON_C_TO_STRING_NOSLASHESCAPE) as i32
            } else {
                // `"{}"` → compact JSON, no slash escape and no trailing ".0" for integers.
                (cglue::JSON_C_TO_STRING_NOSLASHESCAPE | cglue::JSON_C_TO_STRING_NOZERO) as i32
            };

            let cbuffer = cglue::json_object_to_json_string_ext(jso, flags);
            // SAFETY: json-c guarantees a valid, null-terminated UTF-8 buffer here.
            CStr::from_ptr(cbuffer)
        };

        // Note: no trailing semicolon — `write!` returns `fmt::Result` directly.
        write!(f, "{}", cstring.to_str().expect("json-c produced non-UTF8"))
        // Do NOT free `cbuffer`: it is managed by json-c and tied to `self.jso`.
    }
}
pub trait JsoncExport<T> {
    /// # Safety
    /// Caller must ensure `jso` is a valid pointer coming from json-c.
    unsafe fn from_jso(jso: *mut cglue::json_object) -> Result<T, AfbError>;
}

impl JsoncExport<Jobject> for JsoncObj {
    #[track_caller]
    unsafe fn from_jso(jso: *mut cglue::json_object) -> Result<Jobject, AfbError> {
        if jso as usize == 0 {
            afb_error!("jsonc-get-key", "not object at key")
        } else {
            Ok(JsoncObj::get_jso_value(jso))
        }
    }
}

impl JsoncExport<&'static str> for JsoncObj {
    #[track_caller]
    unsafe fn from_jso(jso: *mut cglue::json_object) -> Result<&'static str, AfbError> {
        if cglue::json_object_get_type(jso) != cglue::json_type_json_type_string {
            afb_error!("jsonc-get-type", "jsonc object is not a string",)
        } else {
            Ok(to_static_str(JsoncObj::to_string(jso)))
        }
    }
}

impl JsoncExport<i64> for JsoncObj {
    #[track_caller]
    unsafe fn from_jso(jso: *mut cglue::json_object) -> Result<i64, AfbError> {
        if cglue::json_object_get_type(jso) != cglue::json_type_json_type_int {
            afb_error!("jsonc-get-type", "jsonc object is not an integer",)
        } else {
            Ok(cglue::json_object_get_int64(jso))
        }
    }
}

impl JsoncExport<String> for JsoncObj {
    #[track_caller]
    unsafe fn from_jso(jso: *mut cglue::json_object) -> Result<String, AfbError> {
        if cglue::json_object_get_type(jso) != cglue::json_type_json_type_string {
            afb_error!("jsonc-get-type", "jsonc object is not a string",)
        } else {
            Ok(JsoncObj::to_string(jso))
        }
    }
}

impl JsoncExport<Vec<u8>> for JsoncObj {
    #[track_caller]
    unsafe fn from_jso(jso: *mut cglue::json_object) -> Result<Vec<u8>, AfbError> {
        if cglue::json_object_get_type(jso) != cglue::json_type_json_type_string {
            afb_error!("jsonc-get-type", "jsonc object is not a string",)
        } else {
            Ok(JsoncObj::to_string(jso).as_bytes().to_vec())
        }
    }
}

impl JsoncExport<u64> for JsoncObj {
    #[track_caller]
    unsafe fn from_jso(jso: *mut cglue::json_object) -> Result<u64, AfbError> {
        if cglue::json_object_get_type(jso) != cglue::json_type_json_type_int {
            afb_error!("jsonc-get-type", "jsonc object is not an unsigned",)
        } else {
            Ok(cglue::json_object_get_int64(jso) as u64)
        }
    }
}

impl JsoncExport<i32> for JsoncObj {
    #[track_caller]
    unsafe fn from_jso(jso: *mut cglue::json_object) -> Result<i32, AfbError> {
        if cglue::json_object_get_type(jso) != cglue::json_type_json_type_int {
            afb_error!("jsonc-get-type", "jsonc object is not an integer",)
        } else {
            Ok(cglue::json_object_get_int(jso))
        }
    }
}

impl JsoncExport<u32> for JsoncObj {
    #[track_caller]
    unsafe fn from_jso(jso: *mut cglue::json_object) -> Result<u32, AfbError> {
        if cglue::json_object_get_type(jso) != cglue::json_type_json_type_int {
            afb_error!("jsonc-get-type", "jsonc object is not integer",)
        } else {
            Ok(cglue::json_object_get_int(jso) as u32)
        }
    }
}

impl JsoncExport<i16> for JsoncObj {
    #[track_caller]
    unsafe fn from_jso(jso: *mut cglue::json_object) -> Result<i16, AfbError> {
        if cglue::json_object_get_type(jso) != cglue::json_type_json_type_int {
            afb_error!("jsonc-get-type", "jsonc object is not an integer",)
        } else {
            let value = cglue::json_object_get_int(jso);
            if value > i16::MAX as i32 || value < i16::MIN as i32 {
                return afb_error!("jsonc::get<i16>", "multiplier should be i16 get:{}", value);
            }
            Ok(value as i16)
        }
    }
}

impl JsoncExport<u8> for JsoncObj {
    #[track_caller]
    unsafe fn from_jso(jso: *mut cglue::json_object) -> Result<u8, AfbError> {
        if cglue::json_object_get_type(jso) != cglue::json_type_json_type_int {
            afb_error!("jsonc-get-type", "jsonc object is not an integer",)
        } else {
            let value = cglue::json_object_get_int(jso);
            if value > u8::MAX as i32 || value < u8::MIN as i32 {
                return afb_error!("jsonc::get<u8>", "multiplier should be u8 get:{}", value);
            }
            Ok(value as u8)
        }
    }
}

impl JsoncExport<i8> for JsoncObj {
    #[track_caller]
    unsafe fn from_jso(jso: *mut cglue::json_object) -> Result<i8, AfbError> {
        if cglue::json_object_get_type(jso) != cglue::json_type_json_type_int {
            afb_error!("jsonc-get-type", "jsonc object is not an integer",)
        } else {
            let value = cglue::json_object_get_int(jso);
            if value > i8::MAX as i32 || value < i8::MIN as i32 {
                return afb_error!("jsonc::get<i8>", "multiplier should be i8 get:{}", value);
            }
            Ok(value as i8)
        }
    }
}

impl JsoncExport<u16> for JsoncObj {
    #[track_caller]
    unsafe fn from_jso(jso: *mut cglue::json_object) -> Result<u16, AfbError> {
        if cglue::json_object_get_type(jso) != cglue::json_type_json_type_int {
            afb_error!("jsonc-get-type", "jsonc object is not an integer",)
        } else {
            let value = cglue::json_object_get_int(jso);
            if value > u16::MAX as i32 || value < u16::MIN as i32 {
                return afb_error!("jsonc::get<u16>", "multiplier should be u16 get:{}", value);
            }
            Ok(value as u16)
        }
    }
}

impl JsoncExport<bool> for JsoncObj {
    #[track_caller]
    unsafe fn from_jso(jso: *mut cglue::json_object) -> Result<bool, AfbError> {
        if cglue::json_object_get_type(jso) != cglue::json_type_json_type_boolean {
            afb_error!("jsonc-get-type", "jsonc object is not boolean")
        } else {
            Ok(cglue::json_object_get_boolean(jso) != 0)
        }
    }
}

impl JsoncExport<f64> for JsoncObj {
    #[track_caller]
    unsafe fn from_jso(jso: *mut cglue::json_object) -> Result<f64, AfbError> {
        if cglue::json_object_get_type(jso) != cglue::json_type_json_type_double {
            afb_error!("jsonc-get-type", "jsonc object is not a float",)
        } else {
            Ok(cglue::json_object_get_double(jso))
        }
    }
}

#[doc(hidden)]
pub trait ImportJso<K> {
    #[track_caller]
    fn get_jso(key: K, jso: *const cglue::json_object)
        -> Result<*mut cglue::json_object, AfbError>;
}

impl JsoncExport<JsoncObj> for JsoncObj {
    #[track_caller]
    unsafe fn from_jso(jso: *mut cglue::json_object) -> Result<JsoncObj, AfbError> {
        // Safety: `jso` must come from json-c and be a valid pointer.
        JsoncObj::import(jso as *mut std::ffi::c_void)
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl ImportJso<&str> for JsoncObj {
    #[track_caller]
    fn get_jso(
        key: &str,
        jso: *const cglue::json_object,
    ) -> Result<*mut cglue::json_object, AfbError> {
        let skey = CString::new(key).expect("Invalid jsonc key string");
        let result;
        unsafe {
            let jslot: *mut cglue::json_object = std::ptr::null_mut();

            if cglue::json_object_object_get_ex(
                jso,
                skey.into_raw(),
                &jslot as *const _ as *mut *mut cglue::json_object,
            ) == 0
            {
                result = afb_error!("jsonc-key-missing", key.to_string());
            } else {
                result = Ok(jslot);
            }
        }
        result
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl ImportJso<usize> for JsoncObj {
    #[track_caller]
    fn get_jso(
        idx: usize,
        jso: *const cglue::json_object,
    ) -> Result<*mut cglue::json_object, AfbError> {
        unsafe {
            // Index must be strictly less than length
            if idx >= cglue::json_object_array_length(jso) {
                afb_error!("jsonc-array-size", "jsonc array index out of bound")
            } else {
                Ok(cglue::json_object_array_get_idx(jso, idx))
            }
        }
    }
}

#[doc(hidden)]
pub trait JsonImport<T> {
    #[track_caller]
    fn add(&self, key: &str, value: T);
    fn insert(&self, idx: usize, value: T);
    fn append(&self, value: T);
}

impl JsonImport<&JsoncObj> for JsoncObj {
    #[track_caller]
    fn add(&self, key: &str, object: &JsoncObj) {
        let jso = unsafe { cglue::json_object_get(object.jso) };
        self.add_to_object(key, jso);
    }
    #[track_caller]
    fn insert(&self, idx: usize, object: &JsoncObj) {
        let jso = unsafe { cglue::json_object_get(object.jso) };
        self.insert_to_array(idx, jso);
    }
    #[track_caller]
    fn append(&self, object: &JsoncObj) {
        let jso = unsafe { cglue::json_object_get(object.jso) };
        self.append_to_array(jso);
    }
}

impl JsonImport<JsoncObj> for JsoncObj {
    #[track_caller]
    fn add(&self, key: &str, object: JsoncObj) {
        let jso = unsafe { cglue::json_object_get(object.jso) };
        self.add_to_object(key, jso);
    }
    #[track_caller]
    fn append(&self, object: JsoncObj) {
        let jso = unsafe { cglue::json_object_get(object.jso) };
        self.append_to_array(jso);
    }
    #[track_caller]
    fn insert(&self, idx: usize, object: JsoncObj) {
        let jso = unsafe { cglue::json_object_get(object.jso) };
        self.insert_to_array(idx, jso);
    }
}

impl JsonImport<bool> for JsoncObj {
    #[track_caller]
    fn add(&self, key: &str, value: bool) {
        unsafe {
            let object = cglue::json_object_new_boolean(value as i32);
            self.add_to_object(key, object);
        }
    }
    #[track_caller]
    fn insert(&self, idx: usize, value: bool) {
        unsafe {
            let object = cglue::json_object_new_boolean(value as i32);
            self.insert_to_array(idx, object);
        }
    }

    #[track_caller]
    fn append(&self, value: bool) {
        unsafe {
            let object = cglue::json_object_new_boolean(value as i32);
            self.append_to_array(object);
        }
    }
}

impl JsonImport<f64> for JsoncObj {
    #[track_caller]
    fn add(&self, key: &str, value: f64) {
        unsafe {
            let object = cglue::json_object_new_double(value);
            self.add_to_object(key, object);
        }
    }

    #[track_caller]
    fn append(&self, value: f64) {
        unsafe {
            let object = cglue::json_object_new_double(value);
            self.append_to_array(object);
        }
    }

    #[track_caller]
    fn insert(&self, idx: usize, value: f64) {
        unsafe {
            let object = cglue::json_object_new_double(value);
            self.insert_to_array(idx, object);
        }
    }
}

impl JsonImport<i64> for JsoncObj {
    #[track_caller]
    fn add(&self, key: &str, value: i64) {
        unsafe {
            let object = cglue::json_object_new_int64(value);
            self.add_to_object(key, object);
        }
    }
    #[track_caller]
    fn append(&self, value: i64) {
        unsafe {
            let object = cglue::json_object_new_int64(value);
            self.append_to_array(object);
        }
    }
    #[track_caller]
    fn insert(&self, idx: usize, value: i64) {
        unsafe {
            let object = cglue::json_object_new_int64(value);
            self.insert_to_array(idx, object);
        }
    }
}

impl JsonImport<u64> for JsoncObj {
    #[track_caller]
    fn add(&self, key: &str, value: u64) {
        unsafe {
            let object = cglue::json_object_new_int64(value as i64);
            self.add_to_object(key, object);
        }
    }
    #[track_caller]
    fn append(&self, value: u64) {
        unsafe {
            let object = cglue::json_object_new_int64(value as i64);
            self.append_to_array(object);
        }
    }
    #[track_caller]
    fn insert(&self, idx: usize, value: u64) {
        unsafe {
            let object = cglue::json_object_new_int64(value as i64);
            self.insert_to_array(idx, object);
        }
    }
}

impl JsonImport<i32> for JsoncObj {
    #[track_caller]
    fn add(&self, key: &str, value: i32) {
        unsafe {
            let object = cglue::json_object_new_int(value);
            self.add_to_object(key, object);
        }
    }
    #[track_caller]
    fn append(&self, value: i32) {
        unsafe {
            let object = cglue::json_object_new_int(value);
            self.append_to_array(object);
        }
    }
    #[track_caller]
    fn insert(&self, idx: usize, value: i32) {
        unsafe {
            let object = cglue::json_object_new_int(value);
            self.insert_to_array(idx, object);
        }
    }
}

impl JsonImport<u32> for JsoncObj {
    #[track_caller]
    fn add(&self, key: &str, value: u32) {
        JsonImport::add(self, key, value as i64)
    }
    #[track_caller]
    fn insert(&self, idx: usize, value: u32) {
        JsonImport::insert(self, idx, value as i64)
    }
    #[track_caller]
    fn append(&self, value: u32) {
        JsonImport::append(self, value as i64)
    }
}

impl JsonImport<u16> for JsoncObj {
    #[track_caller]
    fn add(&self, key: &str, value: u16) {
        JsonImport::add(self, key, value as u32)
    }
    #[track_caller]
    fn insert(&self, idx: usize, value: u16) {
        JsonImport::insert(self, idx, value as u32)
    }
    #[track_caller]
    fn append(&self, value: u16) {
        JsonImport::append(self, value as u32)
    }
}

impl JsonImport<i16> for JsoncObj {
    #[track_caller]
    fn add(&self, key: &str, value: i16) {
        JsonImport::add(self, key, value as i32)
    }
    #[track_caller]
    fn insert(&self, idx: usize, value: i16) {
        JsonImport::insert(self, idx, value as i32)
    }
    #[track_caller]
    fn append(&self, value: i16) {
        JsonImport::append(self, value as i32)
    }
}

impl JsonImport<u8> for JsoncObj {
    #[track_caller]
    fn add(&self, key: &str, value: u8) {
        JsonImport::add(self, key, value as u32)
    }
    #[track_caller]
    fn insert(&self, idx: usize, value: u8) {
        JsonImport::insert(self, idx, value as u32)
    }
    #[track_caller]
    fn append(&self, value: u8) {
        JsonImport::append(self, value as u32)
    }
}

impl JsonImport<i8> for JsoncObj {
    #[track_caller]
    fn add(&self, key: &str, value: i8) {
        JsonImport::add(self, key, value as i32)
    }
    #[track_caller]
    fn insert(&self, idx: usize, value: i8) {
        JsonImport::insert(self, idx, value as i32)
    }
    #[track_caller]
    fn append(&self, value: i8) {
        JsonImport::append(self, value as i32)
    }
}

impl JsonImport<usize> for JsoncObj {
    #[track_caller]
    fn add(&self, key: &str, value: usize) {
        JsonImport::add(self, key, value as i64)
    }
    fn insert(&self, idx: usize, value: usize) {
        JsonImport::insert(self, idx, value as i64)
    }
    fn append(&self, value: usize) {
        JsonImport::append(self, value as i64)
    }
}
impl JsonImport<&[u8]> for JsoncObj {
    #[track_caller]
    fn add(&self, key: &str, value: &[u8]) {
        let sval = CString::new(value).expect("Invalid jsonc key bytes");
        unsafe {
            let object = cglue::json_object_new_string(sval.into_raw());
            self.add_to_object(key, object);
        }
    }
    #[track_caller]
    fn append(&self, value: &[u8]) {
        let sval = CString::new(value).expect("Invalid jsonc key bytes");
        unsafe {
            let object = cglue::json_object_new_string(sval.into_raw());
            self.append_to_array(object);
        }
    }

    fn insert(&self, idx: usize, value: &[u8]) {
        let sval = CString::new(value).expect("Invalid jsonc key bytes");
        unsafe {
            let object = cglue::json_object_new_string(sval.into_raw());
            self.insert_to_array(idx, object);
        }
    }
}

impl JsonImport<&str> for JsoncObj {
    #[track_caller]
    fn add(&self, key: &str, value: &str) {
        let sval = CString::new(value).expect("Invalid jsonc key string");
        unsafe {
            let object = cglue::json_object_new_string(sval.into_raw());
            self.add_to_object(key, object);
        }
    }
    #[track_caller]
    fn append(&self, value: &str) {
        let sval = CString::new(value).expect("Invalid jsonc key string");
        unsafe {
            let object = cglue::json_object_new_string(sval.into_raw());
            self.append_to_array(object);
        }
    }

    fn insert(&self, idx: usize, value: &str) {
        let sval = CString::new(value).expect("Invalid jsonc key string");
        unsafe {
            let object = cglue::json_object_new_string(sval.into_raw());
            self.insert_to_array(idx, object);
        }
    }
}

impl JsonImport<&String> for JsoncObj {
    #[track_caller]
    fn add(&self, key: &str, value: &String) {
        JsonImport::add(self, key, value.as_str())
    }
    #[track_caller]
    fn insert(&self, idx: usize, value: &String) {
        JsonImport::insert(self, idx, value.as_str())
    }

    #[track_caller]
    fn append(&self, value: &String) {
        JsonImport::append(self, value.as_str())
    }
}

pub trait JsoncImport<T> {
    fn import(args: T) -> Result<JsoncObj, AfbError>;
}

impl JsoncImport<JsoncObj> for JsoncObj {
    #[track_caller]
    fn import(value: JsoncObj) -> Result<Self, AfbError> {
        Ok(value)
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl JsoncImport<*const *const JsoncJso> for JsoncObj {
    #[track_caller]
    fn import(jso: *const *const JsoncJso) -> Result<Self, AfbError> {
        debug_assert!(!jso.is_null(), "Jsonc::import: jso must not be null");
        unsafe {
            let jsonc = JsoncObj {
                jso: cglue::json_object_get(*jso as *mut cglue::json_object),
            };
            Ok(jsonc)
        }
    }
}

impl JsoncImport<*mut std::ffi::c_void> for JsoncObj {
    #[track_caller]
    fn import(value: *mut std::ffi::c_void) -> Result<Self, AfbError> {
        let jso: &mut cglue::json_object = unsafe { &mut *(value as *mut cglue::json_object) };
        unsafe {
            let jsonc = JsoncObj {
                jso: cglue::json_object_get(jso),
            };
            Ok(jsonc)
        }
    }
}

impl JsoncImport<i64> for JsoncObj {
    #[track_caller]
    fn import(value: i64) -> Result<Self, AfbError> {
        unsafe {
            let jsonc = JsoncObj {
                jso: cglue::json_object_new_int64(value),
            };
            Ok(jsonc)
        }
    }
}

impl JsoncImport<f64> for JsoncObj {
    #[track_caller]
    fn import(value: f64) -> Result<Self, AfbError> {
        unsafe {
            let jsonc = JsoncObj {
                jso: cglue::json_object_new_double(value),
            };
            Ok(jsonc)
        }
    }
}

impl JsoncImport<&str> for JsoncObj {
    #[track_caller]
    fn import(value: &str) -> Result<Self, AfbError> {
        if value.starts_with('{') || value.starts_with('[') {
            JsoncObj::parse(value)
        } else {
            let sval = CString::new(value).expect("Invalid jsonc key string");
            unsafe {
                let jsonc = JsoncObj {
                    jso: cglue::json_object_new_string(sval.into_raw()),
                };
                Ok(jsonc)
            }
        }
    }
}

impl JsoncImport<&String> for JsoncObj {
    #[track_caller]
    fn import(value: &String) -> Result<Self, AfbError> {
        let sval = CString::new(value.as_str()).expect("Invalid jsonc key string");
        unsafe {
            let jsonc = JsoncObj {
                jso: cglue::json_object_new_string(sval.into_raw()),
            };
            Ok(jsonc)
        }
    }
}

impl Default for JsoncObj {
    fn default() -> Self {
        Self::new()
    }
}

impl JsoncObj {
    #[track_caller]
    pub fn new() -> JsoncObj {
        unsafe {
            // New empty object
            JsoncObj {
                jso: cglue::json_object_new_object(),
            }
        }
    }
    #[track_caller]
    pub fn array() -> JsoncObj {
        unsafe {
            // New empty array
            JsoncObj {
                jso: cglue::json_object_new_array(),
            }
        }
    }

    // #[track_caller]
    // pub fn import<T>(args: T) -> Result<JsoncObj, AfbError>
    // where
    //     JsoncObj: JsoncImport<T>,
    // {
    //     JsoncObj::import_from(args)
    // }

    #[track_caller]
    pub fn get_as<T>(&self) -> Result<T, AfbError>
    where
        JsoncObj: JsoncExport<T>,
    {
        // Safety: `self.jso` comes from json-c and lives as long as `self`
        unsafe { Self::from_jso(self.jso) }
    }

    #[track_caller]
    pub fn add<T>(&self, key: &str, value: T) -> Result<&Self, AfbError>
    where
        JsoncObj: JsonImport<T>,
    {
        unsafe {
            if cglue::json_object_is_type(self.jso, cglue::json_type_json_type_object) == 0 {
                afb_error!("jsonc-add-fail", "jsonc target is not an object")
            } else {
                JsonImport::add(self, key, value);
                Ok(self)
            }
        }
    }

    #[track_caller]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn get_jso_value(jso: *mut cglue::json_object) -> Jobject {
        let result;
        unsafe {
            match JsoncObj::get_jso_type(jso) {
                Jtype::Bool => result = Jobject::Bool(cglue::json_object_get_boolean(jso) != 0),
                Jtype::Int => result = Jobject::Int(cglue::json_object_get_int64(jso)),
                Jtype::Float => result = Jobject::Float(cglue::json_object_get_double(jso)),
                Jtype::String => {
                    let cbuffer = cglue::json_object_get_string(jso);
                    let cstring = CStr::from_ptr(cbuffer);
                    result = Jobject::String(cstring.to_str().unwrap().to_owned());
                }
                Jtype::Array => {
                    result = {
                        Jobject::Array(JsoncObj {
                            jso: cglue::json_object_get(jso),
                        })
                    }
                }
                Jtype::Object => {
                    result = Jobject::Object(JsoncObj {
                        jso: cglue::json_object_get(jso),
                    });
                }
                Jtype::Null => result = Jobject::Null(),
                _ => result = Jobject::Unknown("jsonc unknown type"),
            }
        }
        result
    }

    #[track_caller]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn to_string(jso: *mut cglue::json_object) -> String {
        unsafe {
            // Convert json-c string to owned Rust String
            let cbuffer = cglue::json_object_get_string(jso);
            let cstring = CStr::from_ptr(cbuffer);
            cstring.to_str().unwrap().to_owned()
        }
    }

    #[track_caller]
    pub fn get<T>(&self, key: &str) -> Result<T, AfbError>
    where
        JsoncObj: JsoncExport<T>,
    {
        match Self::get_jso(key, self.jso) {
            Err(error) => Err(error),
            Ok(jso) => unsafe { Self::from_jso(jso) },
        }
    }

    #[track_caller]
    pub fn default<T>(&self, key: &str, default: T) -> Result<T, AfbError>
    where
        JsoncObj: JsoncExport<T>,
    {
        match Self::get_jso(key, self.jso) {
            Err(_error) => Ok(default),
            Ok(jso) => unsafe { Self::from_jso(jso) },
        }
    }

    #[track_caller]
    pub fn optional<T>(&self, key: &str) -> Result<Option<T>, AfbError>
    where
        JsoncObj: JsoncExport<T>,
    {
        match Self::get_jso(key, self.jso) {
            Err(_error) => Ok(None),
            Ok(jso) => Ok(Some(unsafe { Self::from_jso(jso)? })),
        }
    }

    #[track_caller]
    pub fn index<T>(&self, index: usize) -> Result<T, AfbError>
    where
        JsoncObj: JsoncExport<T>,
    {
        match Self::get_jso(index, self.jso) {
            Err(error) => Err(error),
            Ok(jso) => unsafe { Self::from_jso(jso) },
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
    pub fn len(&self) -> Result<usize, AfbError> {
        self.count()
    }

    /// Convenience sibling for Clippy `len_without_is_empty`
    #[track_caller]
    pub fn is_empty(&self) -> Result<bool, AfbError> {
        Ok(self.count()? == 0)
    }

    #[track_caller]
    pub fn insert<T>(&self, idx: usize, value: T) -> Result<&Self, AfbError>
    where
        JsoncObj: JsonImport<T>,
    {
        unsafe {
            if cglue::json_object_is_type(self.jso, cglue::json_type_json_type_array) == 0 {
                afb_error!("jsonc-insert-fail", "jsonc target is not an array")
            } else {
                JsonImport::insert(self, idx, value);
                Ok(self)
            }
        }
    }

    #[track_caller]
    pub fn append<T>(&self, value: T) -> Result<&Self, AfbError>
    where
        JsoncObj: JsonImport<T>,
    {
        unsafe {
            if cglue::json_object_is_type(self.jso, cglue::json_type_json_type_array) == 0 {
                afb_error!("jsonc-append-fail", "jsonc target is not an array")
            } else {
                JsonImport::append(self, value);
                Ok(self)
            }
        }
    }

    #[track_caller]
    pub fn into_raw(&self) -> *mut cglue::json_object {
        unsafe { cglue::json_object_get(self.jso) }
    }

    #[track_caller]
    fn add_to_object(&self, key: &str, jval: *mut cglue::json_object) {
        let skey = CString::new(key).expect("Invalid jsonc key string");
        unsafe {
            cglue::json_object_object_add(self.jso, skey.into_raw(), jval);
        }
    }

    #[track_caller]
    fn append_to_array(&self, jval: *mut cglue::json_object) {
        unsafe {
            cglue::json_object_array_add(self.jso, jval);
        }
    }

    #[track_caller]
    fn insert_to_array(&self, idx: usize, jval: *mut cglue::json_object) {
        unsafe {
            cglue::json_object_array_put_idx(self.jso, idx, jval);
        }
    }

    #[track_caller]
    pub fn is_type(&self, jtype: Jtype) -> bool {
        unsafe { cglue::json_object_is_type(self.jso, jtype as u32) != 0 }
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
    pub fn expand(&self) -> Result<Vec<Jentry>, AfbError> {
        // if not object return now
        if !self.is_type(Jtype::Object) {
            return Ok(Vec::new());
        }

        let mut jvec = Vec::new();
        let mut entry = unsafe { (*cglue::json_object_get_object(self.jso)).head };
        while !entry.is_null() {
            let key = unsafe {
                CStr::from_ptr((*entry).k as *const Cchar)
                    .to_owned()
                    .to_str()
                    .unwrap()
                    .to_owned()
            };
            let obj = unsafe { JsoncObj::import((*entry).v as *mut std::ffi::c_void) }?;
            jvec.push(Jentry { key, obj });
            entry = unsafe { (*entry).next };
        }
        Ok(jvec)
    }

    #[track_caller]
    pub fn sort(&self, callback: JsonSortCb) -> Result<&Self, AfbError> {
        unsafe {
            if cglue::json_object_is_type(self.jso, cglue::json_type_json_type_array) == 0 {
                afb_error!("jsonc-sort-fail", "jsonc target is not an array")
            } else {
                cglue::json_object_array_sort(self.jso, callback);
                Ok(self)
            }
        }
    }

    #[track_caller]
    pub fn contains(&mut self, jtok: JsoncObj) -> Result<(), AfbError> {
        let jvec = jtok.expand()?;
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
    pub fn equal(&self, uid: &str, jexpected: JsoncObj, tag: Jequal) -> Result<(), AfbError> {
        match jexpected.get_type() {
            Jtype::Array => {
                // loop recursively on array slot
                if !self.is_type(Jtype::Array) {
                    return afb_error!(
                        uid,
                        "jsonc-match invalid type received:{:?} expected:{:?} value:{}",
                        self.get_type(),
                        jexpected.get_type(),
                        self
                    );
                }
                for idx in 0..self.count()? {
                    let receive_slot = self.index::<JsoncObj>(idx)?;
                    let expected_slot = jexpected.index(idx)?;
                    let uid_slot = format!("{}:{}", uid, idx);
                    receive_slot.equal(&uid_slot, expected_slot, tag)?;
                }
            }
            Jtype::Object => {
                // move jsonc into a rust array and iterate on key/value pairs
                if !self.is_type(Jtype::Object) {
                    return afb_error!(
                        uid,
                        "jsonc-match invalid type received:{:?} expected:{:?} value:{}",
                        self.get_type(),
                        jexpected.get_type(),
                        self
                    );
                }
                let received = self.expand()?;
                let expected = jexpected.expand()?;

                match tag {
                    Jequal::Partial => {
                        for expected_entry in &expected {
                            let received_entry =
                                match received.iter().find_map(|s| cmp_entry(s, expected_entry)) {
                                    None => {
                                        return afb_error!(
                                            uid,
                                            format!(
                                                "jsonc-match fail to find key:{} query:{}",
                                                expected_entry.key, self
                                            )
                                        )
                                    }
                                    Some(value) => value,
                                };
                            received_entry.obj.equal(
                                &expected_entry.key,
                                expected_entry.obj.clone(),
                                tag,
                            )?;
                        }
                    }

                    Jequal::Full => {
                        for received_entry in &received {
                            let expected_entry =
                                match expected.iter().find_map(|s| cmp_entry(s, received_entry)) {
                                    None => {
                                        return afb_error!(
                                            uid,
                                            format!(
                                                "jsonc-match fail to find key:{} expected:{}",
                                                received_entry.key, jexpected
                                            )
                                        )
                                    }
                                    Some(value) => value,
                                };
                            received_entry.obj.equal(
                                &expected_entry.key,
                                expected_entry.obj.clone(),
                                tag,
                            )?;
                        }
                    }
                }
            }

            expected_type => {
                if expected_type != self.get_type() {
                    return afb_error!(
                        uid,
                        "jsonc-match invalid type received:{:?} expected:{:?} value:{}",
                        self.get_type(),
                        jexpected.get_type(),
                        self
                    );
                }

                let equal = match expected_type {
                    Jtype::Bool => {
                        let rec = self.get_as::<bool>()?;
                        let exp = jexpected.get_as::<bool>()?;

                        rec == exp
                    }
                    Jtype::Int => {
                        let rec = self.get_as::<i64>()?;
                        let exp = jexpected.get_as::<i64>()?;

                        rec == exp
                    }
                    Jtype::Float => {
                        let rec = self.get_as::<f64>()?;
                        let exp = jexpected.get_as::<f64>()?;

                        rec == exp
                    }
                    Jtype::String => {
                        let rec = self.get_as::<String>()?;
                        let exp = jexpected.get_as::<String>()?;

                        rec == exp
                    }
                    _ => false,
                };

                if !equal {
                    return afb_error!(
                        uid,
                        "jsonc-match invalid value received:{} expected:{}",
                        self,
                        jexpected
                    );
                }
            }
        }

        Ok(())
    }

    #[track_caller]
    pub fn parse(json_str: &str) -> Result<JsoncObj, AfbError> {
        unsafe {
            let tok = cglue::json_tokener_new();
            let jso = cglue::json_tokener_parse_ex(
                tok,
                json_str.as_bytes().as_ptr() as *mut c_char,
                json_str.len() as i32,
            );
            let jsonc = JsoncObj {
                jso: cglue::json_object_get(jso),
            };

            let jerr = cglue::json_tokener_get_error(tok);

            // warning no ';'
            let result = if jerr != cglue::json_tokener_error_json_tokener_success {
                afb_error!("jsonc-parse-fail", json_str)
            } else {
                Ok(jsonc)
            };

            cglue::json_tokener_free(tok);
            result
        }
    }

    pub fn put(&self) {
        unsafe {
            cglue::json_object_put(self.jso);
        }
    }
}
