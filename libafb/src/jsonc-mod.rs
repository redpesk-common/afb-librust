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

#[cfg(test)]
#[path = "../../examples/test/jsonc-test.rs"]
mod jsonc_test;
use utilv4::AfbError;

use cglue; // restrict jsonc C-binding visible only internally
use std::ffi::{CStr, CString};
use std::fmt;
use std::os::raw::c_char;

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

/// safe jsonc-c Rust object wrapper
pub struct JsoncObj {
    /// internal jsonc-c native jsonc object
    jso: *mut cglue::json_object,
}

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
    /// decrease jsonc-c reference count
    fn drop(&mut self) {
        unsafe {
            cglue::json_object_put(self.jso);
        }
    }
}

#[doc(hidden)]
impl Clone for JsoncObj {
    /// increase jsonc-c reference count
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
    /// jsonc-c simple printing output
    /// format {} => JSON_C_TO_STRING_NOSLASHESCAPE | JSON_C_TO_STRING_NOZERO
    /// format {:#} => JSON_C_TO_STRING_PRETTY
    /// Examples
    /// ```
    /// let jsonc= JsoncObj:parse ("'a':1, 'b':123.456, c:'toto'");
    /// println!("jsonc={}", jsonc);
    /// ```
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
    fn put_jso(jso: *mut cglue::json_object) -> Result<String, AfbError> {
        if unsafe { cglue::json_object_get_type(jso) } != cglue::json_type_json_type_string {
            Err(AfbError::new(
                "jsonc-get-type",
                "jsonc object is not a string",
            ))
        } else {
            Ok(JsoncObj::to_string(jso))
        }
    }
}

impl DoPutJso<i64> for JsoncObj {
    fn put_jso(jso: *mut cglue::json_object) -> Result<i64, AfbError> {
        if unsafe { cglue::json_object_get_type(jso) } != cglue::json_type_json_type_int {
            Err(AfbError::new(
                "jsonc-get-type",
                "jsonc object is not an integer",
            ))
        } else {
            Ok(unsafe { cglue::json_object_get_int64(jso) })
        }
    }
}

impl DoPutJso<u64> for JsoncObj {
    fn put_jso(jso: *mut cglue::json_object) -> Result<u64, AfbError> {
        if unsafe { cglue::json_object_get_type(jso) } != cglue::json_type_json_type_int {
            Err(AfbError::new(
                "jsonc-get-type",
                "jsonc object is not an unsigned",
            ))
        } else {
            Ok(unsafe { cglue::json_object_get_int64(jso) } as u64)
        }
    }
}

impl DoPutJso<i32> for JsoncObj {
    fn put_jso(jso: *mut cglue::json_object) -> Result<i32, AfbError> {
        if unsafe { cglue::json_object_get_type(jso) } != cglue::json_type_json_type_int {
            Err(AfbError::new(
                "jsonc-get-type",
                "jsonc object is not an integer",
            ))
        } else {
            Ok(unsafe { cglue::json_object_get_int(jso) })
        }
    }
}

impl DoPutJso<bool> for JsoncObj {
    fn put_jso(jso: *mut cglue::json_object) -> Result<bool, AfbError> {
        if unsafe { cglue::json_object_get_type(jso) } != cglue::json_type_json_type_boolean {
            Err(AfbError::new(
                "jsonc-get-type",
                "jsonc object is not boolean",
            ))
        } else {
            Ok(unsafe { cglue::json_object_get_boolean(jso) != 0 })
        }
    }
}

impl DoPutJso<u32> for JsoncObj {
    fn put_jso(jso: *mut cglue::json_object) -> Result<u32, AfbError> {
        if unsafe { cglue::json_object_get_type(jso) } != cglue::json_type_json_type_int {
            Err(AfbError::new(
                "jsonc-get-type",
                "jsonc object is not integer",
            ))
        } else {
            Ok(unsafe { cglue::json_object_get_int(jso) as u32 })
        }
    }
}

impl DoPutJso<f64> for JsoncObj {
    fn put_jso(jso: *mut cglue::json_object) -> Result<f64, AfbError> {
        if unsafe { cglue::json_object_get_type(jso) } != cglue::json_type_json_type_double {
            Err(AfbError::new(
                "jsonc-get-type",
                "jsonc object is not a float",
            ))
        } else {
            Ok(unsafe { cglue::json_object_get_double(jso) })
        }
    }
}

impl DoPutJso<Jobject> for JsoncObj {
    fn put_jso(jso: *mut cglue::json_object) -> Result<Jobject, AfbError> {
        if jso as usize == 0 {
            Err(AfbError::new("jsonc-get-key", "not object at key"))
        } else {
            Ok(JsoncObj::get_jso_value(jso))
        }
    }
}

#[doc(hidden)]
pub trait DoGetJso<K> {
    fn get_jso(key: K, jso: *const cglue::json_object)
        -> Result<*mut cglue::json_object, AfbError>;
}

impl DoPutJso<JsoncObj> for JsoncObj {
    fn put_jso(jso: *mut cglue::json_object) -> Result<JsoncObj, AfbError> {
        Ok(JsoncObj::from(jso as *mut std::ffi::c_void))
    }
}

impl DoGetJso<&str> for JsoncObj {
    /// private function return internal unsafe jsonc-c object
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
                result = Err(AfbError::new("jconc-get_obj", "jsonc key not found"));
            } else {
                result = Ok(jslot);
            }
        }
        return result;
    }
}

impl DoGetJso<usize> for JsoncObj {
    /// private function return internal unsafe jsonc-c object
    fn get_jso(
        idx: usize,
        jso: *const cglue::json_object,
    ) -> Result<*mut cglue::json_object, AfbError> {
        unsafe {
            if idx > cglue::json_object_array_length(jso) {
                Err(AfbError::new("jsonc-array-size","jsonc array index out of bound"))
            } else {
                Ok(cglue::json_object_array_get_idx(jso, idx))
            }
        }
    }
}

#[doc(hidden)]
pub trait DoAddon<T> {
    fn add(&self, key: &str, value: T);
    fn insert(&self, value: T);
}

impl DoAddon<f64> for JsoncObj {
    fn add(&self, key: &str, value: f64) {
        unsafe {
            let object = cglue::json_object_new_double(value);
            self.add_to_object(key, object);
        }
    }

    fn insert(&self, value: f64) {
        unsafe {
            let object = cglue::json_object_new_double(value);
            self.add_to_array(object);
        }
    }
}

impl DoAddon<i64> for JsoncObj {
    fn add(&self, key: &str, value: i64) {
        unsafe {
            let object = cglue::json_object_new_int64(value);
            self.add_to_object(key, object);
        }
    }
    fn insert(&self, value: i64) {
        unsafe {
            let object = cglue::json_object_new_int64(value);
            self.add_to_array(object);
        }
    }
}

impl DoAddon<i32> for JsoncObj {
    fn add(&self, key: &str, value: i32) {
        unsafe {
            let object = cglue::json_object_new_int(value);
            self.add_to_object(key, object);
        }
    }
    fn insert(&self, value: i32) {
        unsafe {
            let object = cglue::json_object_new_int(value);
            self.add_to_array(object);
        }
    }
}

impl DoAddon<u32> for JsoncObj {
    fn add(&self, key: &str, value: u32) {
        DoAddon::add(self, key, value as i64)
    }
    fn insert(&self, value: u32) {
        DoAddon::insert(self, value as i64)
    }
}

impl DoAddon<usize> for JsoncObj {
    fn add(&self, key: &str, value: usize) {
        DoAddon::add(self, key, value as i64)
    }
    fn insert(&self, value: usize) {
        DoAddon::insert(self, value as i64)
    }
}

impl DoAddon<&str> for JsoncObj {
    fn add(&self, key: &str, value: &str) {
        let sval = CString::new(value).expect("Invalid jsonc key string");
        unsafe {
            let object = cglue::json_object_new_string(sval.into_raw());
            self.add_to_object(key, object);
        }
    }
    fn insert(&self, value: &str) {
        let sval = CString::new(value).expect("Invalid jsonc key string");
        unsafe {
            let object = cglue::json_object_new_string(sval.into_raw());
            self.add_to_array(object);
        }
    }
}

impl DoAddon<&String> for JsoncObj {
    fn add(&self, key: &str, value: &String) {
        DoAddon::add(self, key, value.as_str())
    }
    fn insert(&self, value: &String) {
        DoAddon::insert(self, value.as_str())
    }
}

impl DoAddon<&JsoncObj> for JsoncObj {
    fn add(&self, key: &str, object: &JsoncObj) {
        self.add_to_object(key, object.jso);
    }
    fn insert(&self, object: &JsoncObj) {
        self.add_to_array(object.jso);
    }
}

impl DoAddon<JsoncObj> for JsoncObj {
    fn add(&self, key: &str, object: JsoncObj) {
        self.add_to_object(key, object.jso);
    }
    fn insert(&self, object: JsoncObj) {
        self.add_to_array(object.jso);
    }
}

impl From<i64> for JsoncObj {
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
    fn from(value: &JsonStr) -> Self {
        match JsoncObj::parse(value.0) {
            Err(error) => panic!("(hoops: invalid json error={}", error),
            Ok(jsonc) => jsonc,
        }
    }
}

impl From<*mut std::ffi::c_void> for JsoncObj {
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

/// implement a Rust safe version of jsonc-c
/// Examples:
/// ```
/// // import jsonc-c methods and objects types
/// pub mod jsonc;
/// use JsoncObj::jsonc_mod::JsoncObj::{jsonc, Jtype, Jobject};
/// ```
///
impl JsoncObj {
    /// return an empty jsonc-c Rust safe object
    /// Examples:
    /// ```
    /// let jsonc = JsoncObj::object();
    /// ```
    pub fn new() -> JsoncObj {
        unsafe {
            let jsonc = JsoncObj {
                jso: cglue::json_object_new_object(),
            };
            cglue::json_object_get(jsonc.jso);
            return jsonc;
        }
    }
    /// return an empty jsonc-c Rust safe array
    /// Examples:
    /// ```
    /// let jsonc = JsoncObj::array();
    /// jsonc.insert(123).unwrap();
    /// jsonc.insert(123.456).unwrap();
    /// jsonc.insert("toto").unwrap();
    /// ```
    pub fn array() -> JsoncObj {
        unsafe {
            let jsonc = JsoncObj {
                jso: cglue::json_object_new_array(),
            };
            cglue::json_object_get(jsonc.jso);
            return jsonc;
        }
    }

    /// return from args type a jsonc-c Rust safe object
    /// Examples:
    /// ```
    /// let jint = JsoncObj::from(123);
    /// let jfloat = JsoncObj::from(123.456);
    /// let jstring = JsoncObj::from("toto");
    /// ```
    pub fn from<T>(args: T) -> JsoncObj
    where
        T: Into<JsoncObj>,
    {
        args.into()
    }

    /// add a Rust (int,float,...) to jsonc-c object
    /// Examples:
    /// ```
    /// let jsonc = JsoncObj::new();
    /// jsonc.add("slot1", 123).unwrap();
    /// jsonc.add("slot2", 123.456).unwrap();
    /// jsonc.add("slot3", "toto").unwrap();
    /// ```
    pub fn add<T>(&self, key: &str, value: T) -> Result<&Self, AfbError>
    where
        JsoncObj: DoAddon<T>,
    {
        unsafe {
            let result =
                if cglue::json_object_is_type(self.jso, cglue::json_type_json_type_object) == 0 {
                    Err(AfbError::new("jsonc-add-fail", "jsonc target is not an object"))
                } else {
                    DoAddon::add(self, key, value);
                    Ok(self)
                };
            return result;
        }
    }

    /// return a Jobject rust enum depending on jsonc-c object type
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

    /// return rust string from a jsonc-c object
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

    /// return rust Jobject slot from a rust jsonc-c safe object
    /// AfbData
    /// * key wanted jsonc slot
    /// # Examples
    /// ```
    /// let labels = ["slot1", "slot2", "slot3", "slot4"];
    /// for key in 0..labels.len() {
    /// match jsonc.get(labels[key]) {
    ///    Jobject::int(value) => println!("slot={} int={}", key, value),
    ///    Jobject::float(value) => println!("slot={} float={}", key, value),
    ///    Jobject::string(value) => println!("slot={} string={}",key, value),
    ///    Jobject::object(value) => println!("slot={} object={}",key, value),
    ///    Jobject::array(value) => println!("slot={} array={}",key, value),
    ///    _ => println!("Hoop: unknown Jtype"),
    /// }
    /// }
    /// ```

    pub fn get<T>(&self, key: &str) -> Result<T, AfbError>
    where
        JsoncObj: DoPutJso<T>,
    {
        match Self::get_jso(key, self.jso) {
            Err(error) => Err(error),
            Ok(jso) => Self::put_jso(jso),
        }
    }

    /// return rust Jobject slot from a rust jsonc-c safe object
    /// AfbData
    /// * key wanted jsonc slot
    /// # Examples
    /// ```
    /// let jobject = JsoncObj::parse ("['slot1', 1234, 4567.987, 'true']);
    /// let value= jobject.index::<String>(0);
    /// let value= jobject.index::<i64>(1);
    /// let value= jobject.index::<f64>(2);
    /// let value= jobject.index::<Bool>(3);
    /// ```

    pub fn index<T>(&self, index: usize) -> Result<T, AfbError>
    where
        JsoncObj: DoPutJso<T>,
    {
        match Self::get_jso(index, self.jso) {
            Err(error) => Err(error),
            Ok(jso) => Self::put_jso(jso),
        }
    }

    /// return jsonc-c object/array count
    /// # Examples
    /// ```
    /// match jsonc.count() {
    ///     Ok(count) => println! ("object count={}", count);
    ///     Err(error) => println!(error);
    /// }
    ///
    /// ```
    pub fn count(&self) -> Result<usize, AfbError> {
        unsafe {
            match JsoncObj::get_jso_type(self.jso) {
                Jtype::Array => Ok(cglue::json_object_array_length(self.jso)),
                Jtype::Object => Ok(cglue::json_object_object_length(self.jso) as usize),
                _ => Err(AfbError::new(
                    "jsonc-count-fail",
                    "jsonc is neither object or array",
                )),
            }
        }
    }

    /// insert a Rust (int,float,...) to jsonc-c array
    /// # Examples
    /// ```
    /// jsonc.insert("toto").unwrap();
    /// jsonc.insert(123).unwrap();
    /// jsonc.insert(123.456).unwrap();
    /// ```
    pub fn insert<T>(&self, value: T) -> Result<&Self, AfbError>
    where
        JsoncObj: DoAddon<T>,
    {
        unsafe {
            let result =
                if cglue::json_object_is_type(self.jso, cglue::json_type_json_type_array) == 0 {
                    Err(AfbError::new(
                        "jsonc-insert-fail",
                        "jsonc target is not an array",
                    ))
                } else {
                    DoAddon::insert(self, value);
                    Ok(self)
                };
            return result;
        }
    }

    /// Return an jsonc-c raw unsafe object
    /// Does not increment reference count
    /// Examples:
    /// ```
    /// // require bindgen cglue module to be imported
    /// let jso = jsonc.into_raw();
    /// unsafe {
    ///     let len = cglue::json_object_object_len(jso);
    /// }
    /// ```
    pub fn into_raw(&self) -> *mut cglue::json_object {
        unsafe { cglue::json_object_get(self.jso) };
        self.jso
    }

    fn add_to_object(&self, key: &str, jval: *mut cglue::json_object) {
        let skey = CString::new(key).expect("Invalid jsonc key string");
        unsafe {
            cglue::json_object_object_add(self.jso, skey.into_raw(), jval);
        }
    }

    fn add_to_array(&self, jval: *mut cglue::json_object) {
        unsafe {
            cglue::json_object_array_add(self.jso, jval);
        }
    }

    /// Assert that jsonc object is from a given Jtype
    /// # Examples
    /// ```
    ///if jsonc.is_type(Jtype::array) {
    ///    println!("jsonc is an array");
    ///} else {
    ///   println!("jsonc is not an array");
    /// }
    /// ```
    pub fn is_type(&self, jtype: Jtype) -> bool {
        unsafe {
            if cglue::json_object_is_type(self.jso, jtype as u32) != 0 {
                true
            } else {
                false
            }
        }
    }

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

    /// Examples:
    /// ```
    /// match jsonc.get_type() {
    ///    Jtype::array => println!("jsonc is array"),
    ///    Jtype::object => println!("jsonc is object"),
    ///    _ => println!("Hoop: unknown Jtype"),
    /// }
    /// ```
    pub fn get_type(&self) -> Jtype {
        JsoncObj::get_jso_type(self.jso)
    }

    /// expand as vector of entry(key,jsonc)
    /// Examples
    /// ```
    /// # use jsonc::JsoncObj;
    /// let jsonc= JsoncObj::parse("{'skipail':'IoT.bzh', 'location':'lorient'}").unwrap();
    /// let jvec=jsonc.expand();
    /// for entry in &jvec {
    ///     println! ("key:{} value:{}", entry.key, entry.obj);
    /// }
    /// ```
    pub fn expand(&self) -> Vec<Jentry> {
        // if not object return now
        if !self.is_type(Jtype::Object) {
            return Vec::new();
        }

        let mut jvec = Vec::new();
        let mut entry = unsafe { (*cglue::json_object_get_object(self.jso)).head };
        while entry != 0 as *mut cglue::lh_entry {
            let key = unsafe {
                CStr::from_ptr((*entry).k as *mut i8)
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

    /// check if two jsonc object are equal
    /// Example
    /// ```
    /// # use jsonc::JsoncObj;
    /// let mut jsonc= JsoncObj::from(12);
    /// let mut jtok= JsoncObj::from("12");
    /// match jsonc.equal(jtok.clone()) {
    ///     Ok(()) => println!("matches"),
    ///     Err(error) => println!("{}", error)
    /// }
    /// ```
    pub fn equal(&mut self, jsonc: JsoncObj) -> Result<(), AfbError> {
        if unsafe { cglue::json_object_get_type(self.jso) }
            != unsafe { cglue::json_object_get_type(jsonc.jso) }
        {
            Err(AfbError::new("jsonc::equal", "jtype diverge"))
        } else if self.to_string() != jsonc.to_string() {
            Err(AfbError::new(",jsonc::equal", "jtype not equal"))
        } else {
            Ok(())
        }
    }

    /// check if self contains jsonc
    /// Example
    /// ```
    /// # use jsonc::JsoncObj;
    /// let mut jsonc= JsoncObj::parse("{'skipail':'IoT.bzh', 'location':'lorient'}").unwrap();
    /// let mut jtok= JsoncObj::parse("{'skipail':'IoT.bzh'}").unwrap();
    /// match jsonc.contains(jtok.clone()) {
    ///     Ok(()) => println!("matches"),
    ///     Err(error) => println!("tokens:{} not found in jsonc:{}", jtok, jsonc)
    /// }
    /// ```
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
                        return Err(AfbError::new("jsonc-contains-fail", "json-token-not-found"));
                    }
                }
            }
        }
        Ok(())
    }
    /// create a new Rust safe jsonc-c object from a string
    /// # Examples
    /// ```
    /// let token = "{'a':1,'b':2}";
    /// let jsonc = JsoncObj::parse(token);
    /// let value= jsonc.get_int("a");
    /// ```
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
                Err(AfbError::new("jsonc-parse-fail", json_str))
            } else {
                cglue::json_object_get(jsonc.jso);
                Ok(jsonc)
            };

            cglue::json_tokener_free(tok);
            return result;
        }
    }
}
