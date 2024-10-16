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

use std::any::Any;
use std::any::TypeId;
use std::boxed::Box;
use std::ops::{Deref, DerefMut};
use std::sync::{Mutex, MutexGuard};

use std::ffi::{c_void, CStr, CString};

// alias few external types
pub type AfbTypeV4 = cglue::afb_type_t;
pub type AfbDataV4 = cglue::afb_data_t;
pub type Cchar = ::std::os::raw::c_char;

// trick to create a null parameter
pub type AfbNoData = Option<std::ffi::c_void>;
pub const AFB_NO_AUTH: *const cglue::afb_auth = 0 as *const cglue::afb_auth;
pub const AFB_NO_DATA: AfbNoData = None;
pub const AFB_OK: i32 = 0;
pub const AFB_FAIL: i32 = 1;
pub const AFB_ABORT: i32 = -1;

pub struct ConverterBox(pub Option<&'static AfbConverter>);
unsafe impl Sync for ConverterBox {}

pub enum AfbBuiltinType {
    Opaque,
    StringZ,
    Json,
    JsoncObj,
    ByteArray,
    Bool,
    I32,
    U32,
    I64,
    U64,
    Double,
    None,
}

pub use crate::AfbDataConverter;
#[macro_export]
macro_rules! AfbDataConverter {
    ($uid:ident, $datat:ident) => {
        mod $uid {
            #![allow(non_upper_snake_case)]
            use super::*;
            use std::any::Any;
            pub static mut CONVERTER_BOX: ConverterBox = ConverterBox(None);

            #[track_caller]
            pub fn encode(cbuffer: *mut std::ffi::c_void) -> Result<String, AfbError> {
                let data = unsafe { &mut *(cbuffer as *mut $datat) };
                match serde_json::to_string(data) {
                    Ok(output) => Ok(output),
                    Err(error) => afb_error!(
                        stringify!($uid),
                        "{} {}",
                        stringify!($datat),
                        &error.to_string()
                    ),
                }
            }

            #[track_caller]
            pub fn decode(json_string: &str) -> Result<Box<dyn Any>, AfbError> {
                match serde_json::from_str::<$datat>(json_string) {
                    Ok(value) => Ok(Box::new(value)),
                    //Err(error) => Err(format!("{}::{} {}", stringify!($uid), stringify!($datat), error.to_string()))
                    Err(error) => afb_error!(
                        stringify!($uid),
                        "{} {}",
                        stringify!($datat),
                        &error.to_string()
                    ),
                }
            }

            #[track_caller]
            pub fn register(
            ) -> Result<&'static afbv4::datav4::AfbConverter, afbv4::utilv4::AfbError> {
                let converter =
                    afbv4::datav4::AfbConverter::new(stringify!($uid)).and_then(|obj| {
                        obj.add_encoder(afbv4::datav4::AfbBuiltinType::Json, encode, decode)
                    });

                match converter {
                    Ok(encoder) => {
                        unsafe {
                            CONVERTER_BOX =
                                ConverterBox(Some(encoder as &'static afbv4::datav4::AfbConverter))
                        };
                        Ok(encoder as &'static afbv4::datav4::AfbConverter)
                    }
                    Err(error) => Err(error),
                }
            }
        }

        impl ConvertQuery<&'static $datat> for afbv4::datav4::AfbRqtData {
            #[track_caller]
            fn import(&self, index: usize) -> Result<&'static $datat, afbv4::utilv4::AfbError> {
                let typev4 = match unsafe { &$uid::CONVERTER_BOX } {
                    ConverterBox(None) => {
                        afb_log_msg!(
                            Critical,
                            None,
                            "AfbConverter missing --> {}::register() <-- at binding init",
                            stringify!($uid)
                        );
                        panic!("fix missing converter");
                    }
                    ConverterBox(Some(value)) => value.typev4,
                };

                // retrieve c-buffer pointer to argument void* value
                match self.get_ro(typev4, index) {
                    None => {
                        let data = {
                            let converter = AfbBuiltinType::get(&AfbBuiltinType::StringZ).typev4;
                            match self.get_ro(converter, index) {
                                None => "no readable data found",
                                Some(cbuffer) => {
                                    let cstring = unsafe {
                                        std::ffi::CStr::from_ptr(&mut *(cbuffer as *mut Cchar))
                                    };
                                    cstring.to_str().unwrap()
                                }
                            }
                        };
                        afb_error!(
                            concat!("export:", stringify!($uid)),
                            "invalid custom converter format args[{}]={}",
                            index,
                            data
                        )
                    }
                    Some(cbuffer) => Ok(unsafe { &mut *(cbuffer as *mut $datat) }),
                }
            }
        }

        impl ConvertResponse<$datat> for afbv4::datav4::AfbParams {
            #[track_caller]
            fn export(data: $datat) -> afbv4::datav4::AfbExportResponse {
                let typev4 = match unsafe { &$uid::CONVERTER_BOX } {
                    ConverterBox(None) => {
                        afb_log_msg!(
                            Critical,
                            None,
                            "AfbConverter missing run --> {}::register() <-- at binding init",
                            stringify!($uid)
                        );
                        panic!("fix missing converter");
                    }
                    ConverterBox(Some(value)) => value.typev4,
                };
                let uid = concat!("export:", stringify!($user_type));
                let boxe = Box::new(data);
                afbv4::datav4::AfbExportResponse::Converter(AfbExportData {
                    uid: uid,
                    buffer_ptr: Box::leak(boxe) as *const _ as *mut std::ffi::c_void,
                    typev4: typev4,
                    buffer_len: 0, // auto
                    freecb: None,  // auto
                })
            }
        }
    };
}

pub struct AfbCtxData {
    raw: *mut std::ffi::c_void,
    typeid: TypeId,
    lock: Mutex<bool>,
}

pub struct AfbCtxLock<'a, T> {
    data: T,
    _lock: MutexGuard<'a, bool>,
}

impl<'a, T> Deref for AfbCtxLock<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a, T> DerefMut for AfbCtxLock<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl AfbCtxData {
    pub fn new<T>(ctx: T) -> Self
    where
        T: 'static,
    {
        let typeid = TypeId::of::<T>();
        Self {
            raw: Box::into_raw(Box::new(ctx)) as *mut std::ffi::c_void,
            typeid,
            lock: Mutex::new(true),
        }
    }

    #[track_caller]
    fn check_type(&self, tid: TypeId) -> Result<(), AfbError> {
        if self.typeid != tid {
            return afb_error!("afb-ctx-data", "source/destination incompatible data types",);
        }
        Ok(())
    }

    #[track_caller]
    pub fn get_lock<T>(&self) -> Result<AfbCtxLock<&mut T>, AfbError>
    where
        T: 'static,
    {
        self.check_type(TypeId::of::<T>())?;
        let value = AfbCtxLock {
            data: unsafe { &mut *(self.raw as *mut T) },
            _lock: self.lock.lock().unwrap(),
        };

        Ok(value)
    }

    #[track_caller]
    pub fn get_ref<T>(&self) -> Result<&T, AfbError>
    where
        T: 'static,
    {
        self.check_type(TypeId::of::<T>())?;
        let value: &mut T = unsafe { &mut *(self.raw as *mut T) };
        Ok(value)
    }

    #[track_caller]
    pub fn get_mut<T>(&self) -> Result<&mut T, AfbError>
    where
        T: 'static,
    {
        self.check_type(TypeId::of::<T>())?;
        let value: &mut T = unsafe { &mut *(self.raw as *mut T) };
        Ok(value)
    }

    pub fn get_type(&self) -> String {
        format!("{:?}", self.typeid)
    }

    pub fn free<T>(&self) {
        let mut lock = self.lock.lock().unwrap();
        if *lock {
            *lock = false;
            let boxe = unsafe { Box::from_raw(self.raw as *mut T) };
            drop(boxe)
        }
    }
}

impl Drop for AfbCtxData {
    fn drop(&mut self) {
        let mut lock = self.lock.lock().unwrap();
        if *lock {
            *lock = false;
            let boxe = unsafe { Box::from_raw(self.raw) };
            drop(boxe)
        }
    }
}

impl AfbBuiltinType {
    #[track_caller]
    pub fn get(builtin_type: &AfbBuiltinType) -> AfbConverter {
        unsafe {
            match builtin_type {
                AfbBuiltinType::None => AfbConverter {
                    _uid: "Builtin-None",
                    typev4: 0 as AfbTypeV4,
                },
                AfbBuiltinType::Opaque => AfbConverter {
                    _uid: "Builtin-Opaque",
                    typev4: (*cglue::afbBindingV4r1_itfptr).type_opaque,
                },
                AfbBuiltinType::StringZ => AfbConverter {
                    _uid: "Builtin-StringZ",
                    typev4: (*cglue::afbBindingV4r1_itfptr).type_stringz,
                },
                AfbBuiltinType::Json => AfbConverter {
                    _uid: "Builtin-Json",
                    typev4: (*cglue::afbBindingV4r1_itfptr).type_json,
                },
                AfbBuiltinType::JsoncObj => AfbConverter {
                    _uid: "Builtin-JsoncObj",
                    typev4: (*cglue::afbBindingV4r1_itfptr).type_json_c,
                },
                AfbBuiltinType::ByteArray => AfbConverter {
                    _uid: "Builtin-ByteArray",
                    typev4: (*cglue::afbBindingV4r1_itfptr).type_bytearray,
                },
                AfbBuiltinType::Bool => AfbConverter {
                    _uid: "Builtin-Bool",
                    typev4: (*cglue::afbBindingV4r1_itfptr).type_bool,
                },
                AfbBuiltinType::I32 => AfbConverter {
                    _uid: "Builtin-I32",
                    typev4: (*cglue::afbBindingV4r1_itfptr).type_i32,
                },
                AfbBuiltinType::U32 => AfbConverter {
                    _uid: "Builtin-U32",
                    typev4: (*cglue::afbBindingV4r1_itfptr).type_u32,
                },
                AfbBuiltinType::I64 => AfbConverter {
                    _uid: "Builtin-I64",
                    typev4: (*cglue::afbBindingV4r1_itfptr).type_i64,
                },
                AfbBuiltinType::U64 => AfbConverter {
                    _uid: "Builtin-U64",
                    typev4: (*cglue::afbBindingV4r1_itfptr).type_u64,
                },
                AfbBuiltinType::Double => AfbConverter {
                    _uid: "Builtin-Double",
                    typev4: (*cglue::afbBindingV4r1_itfptr).type_double,
                },
            }
        }
    }
}

// provide alias to make builtin type selection nicer
pub const AFB_BUILTIN: Option<&AfbConverter> = None;

// restore Rust Cstring, in order to make it disposable
#[no_mangle]
extern "C" fn free_cstring_cb(context: *mut std::ffi::c_void) {
    let cbuffer = context as *mut Cchar;
    let cstring = unsafe { CString::from_raw(cbuffer) };
    drop(cstring);
}

// restore Rust Cstring, in order to make it disposable
#[no_mangle]
pub extern "C" fn free_box_cb(context: *mut std::ffi::c_void) {
    let cbox = unsafe { Box::from_raw(context) };
    drop(cbox);
}

type EncoderCb = fn(*mut std::ffi::c_void) -> Result<String, AfbError>;
type DecoderCb = fn(&str) -> Result<Box<dyn Any>, AfbError>;

#[no_mangle]
// move from internal representation to json string
extern "C" fn afb_encoding_cb(
    ctx: *mut std::ffi::c_void,
    source: cglue::afb_data_t,
    _typev4: cglue::afb_type_t,
    dest: *mut cglue::afb_data_t,
) -> i32 {
    let encoder_ref = unsafe { &mut *(ctx as *mut AfbEncoder) };
    // map lib afb raw pointers to rust object require unsafe operation
    let result = unsafe {
        // retrieve raw data pointer and map it as a dyn any
        let cbuffer = cglue::afb_data_ro_pointer(source);

        // call user defined encoding logic
        encoder_ref.get_encoder()(cbuffer)
    };

    match result {
        Ok(encoded) => {
            let len = encoded.len();
            let cbuffer = CString::new(encoded)
                .expect("(hoops) invalid encoded string")
                .into_raw();
            let status = unsafe {
                cglue::afb_create_data_raw(
                    dest,
                    AfbBuiltinType::get(&AfbBuiltinType::Json).typev4,
                    cbuffer as *const _ as *mut std::ffi::c_void,
                    len + 1,
                    Some(free_cstring_cb),
                    cbuffer as *const _ as *mut std::ffi::c_void,
                )
            };
            status
        }
        Err(error) => {
            println!("encoding error={}", error);
            -1
        }
    }
}

#[no_mangle]
//int socialFromJsonObjB (void *ctx,  afb_data_t jsonD, afb_type_t socialT, afb_data_t *dest) {
extern "C" fn afb_decoding_cb(
    ctx: *mut std::ffi::c_void,
    source: cglue::afb_data_t,
    _typev4: cglue::afb_type_t,
    dest: *mut cglue::afb_data_t,
) -> i32 {
    let encoder_ref = unsafe { &mut *(ctx as *mut AfbEncoder) };

    // map lib afb raw pointers to rust object require unsafe operation
    let result = unsafe {
        // retrieve raw data pointer and map it as a Datatype object
        let cbuffer = cglue::afb_data_ro_pointer(source) as *const Cchar;
        let cstring = CStr::from_ptr(cbuffer);
        let data: &str = cstring.to_str().unwrap();

        // call user defined encoding logic
        encoder_ref.get_decoder()(data)
    };

    match result {
        Ok(decoded) => {
            let cbuffer = Box::leak(decoded);

            let status = unsafe {
                cglue::afb_create_data_raw(
                    dest,
                    encoder_ref.typev4,
                    cbuffer as *const _ as *mut std::ffi::c_void,
                    0,
                    Some(free_box_cb),
                    cbuffer as *const _ as *mut std::ffi::c_void,
                )
            };
            status
        }
        Err(error) => {
            println!("decoding error={}", error);
            -1
        }
    }
}

pub trait ConvertQuery<T> {
    fn import(&self, index: usize) -> Result<T, AfbError>;
}

struct AfbEncoder {
    _converter: *mut AfbConverter,
    typev4: cglue::afb_type_t,
    encoder: EncoderCb,
    decoder: DecoderCb,
}

impl AfbEncoder {
    #[track_caller]
    pub fn get_encoder(&mut self) -> EncoderCb {
        self.encoder
    }

    #[track_caller]
    pub fn get_decoder(&mut self) -> DecoderCb {
        self.decoder
    }
}

pub struct AfbConverter {
    _uid: &'static str,
    pub typev4: cglue::afb_type_t,
}

impl AfbConverter {
    // create a new converter type within libafb
    #[track_caller]
    pub fn new(uid: &'static str) -> Result<&mut Self, AfbError> {
        // register new type within libafb
        let cuid = CString::new(uid)
            .expect("Invalid converter uid key")
            .into_raw();
        let typev4 = 0 as cglue::afb_type_t;
        let status = unsafe {
            if cglue::afb_type_lookup(&typev4 as *const _ as *mut cglue::afb_type_t, cuid) == 0 {
                0
            } else {
                cglue::afb_type_register(&typev4 as *const _ as *mut cglue::afb_type_t, cuid, 0)
            }
        };

        if status == 0 {
            let converter = Box::new(AfbConverter {
                _uid: uid,
                typev4: typev4,
            });

            // freeze converter type in memory heap
            Ok(Box::leak(converter))
        } else {
            afb_error!(uid, "fail to register converter data type")
        }
    }

    #[track_caller]
    pub fn add_encoder(
        &mut self,
        encoder_type: AfbBuiltinType,
        encoder: EncoderCb,
        decoder: DecoderCb,
    ) -> Result<&mut Self, AfbError> {
        // froze callback into a leaked box
        let encoder_ref = Box::new(AfbEncoder {
            _converter: self,
            typev4: AfbBuiltinType::get(&AfbBuiltinType::Json).typev4,
            encoder: encoder,
            decoder: decoder,
        });

        let encoder_ref = Box::leak(encoder_ref);

        let mut status = unsafe {
            cglue::afb_type_add_convert_to(
                self.typev4,
                AfbBuiltinType::get(&encoder_type).typev4,
                Some(afb_encoding_cb),
                encoder_ref as *const _ as *mut std::ffi::c_void,
            )
        };

        if status == 0 {
            status = unsafe {
                cglue::afb_type_add_convert_from(
                    self.typev4,
                    AfbBuiltinType::get(&encoder_type).typev4,
                    Some(afb_decoding_cb),
                    encoder_ref as *const _ as *mut std::ffi::c_void,
                )
            }
        };

        if status != 0 {
            afb_error!(&self._uid, "Fail adding encoding converter")
        } else {
            Ok(self)
        }
    }

    // return object getter trait to prevent any malicious modification
    #[track_caller]
    pub fn finalize(&mut self) -> &AfbConverter {
        self
    }

    #[track_caller]
    pub fn get_uid(&self) -> &'static str {
        self._uid
    }

    #[track_caller]
    pub fn get_typev4(&self) -> cglue::afb_type_t {
        self.typev4
    }
}

#[track_caller]
pub fn get_type(uid: &'static str) -> Result<&mut AfbConverter, AfbError> {
    let typev4: cglue::afb_type_t = 0 as cglue::afb_type_t;
    let cuid = CString::new(uid).expect("Invalid converter uid key");

    let status = unsafe {
        cglue::afb_type_lookup(
            &typev4 as *const _ as *mut cglue::afb_type_t,
            cuid.into_raw(),
        )
    };

    if status < 0 {
        afb_error!(uid, "type lookup fail")
    } else {
        let converter_box = Box::new(AfbConverter {
            _uid: uid,
            typev4: typev4,
        });

        Ok(Box::leak(converter_box))
    }
}

macro_rules! _register_query_converter {
    ($rust_type:ty, $afb_builtin_type:ident) => {
        impl ConvertQuery<$rust_type> for AfbRqtData {
            #[track_caller]
            fn import(&self, index: usize) -> Result<$rust_type, AfbError> {
                let converter = unsafe { (*cglue::afbBindingV4r1_itfptr).$afb_builtin_type };
                match self.get_ro(converter, index) {
                    None => {
                        let data = {
                            let converter = AfbBuiltinType::get(&AfbBuiltinType::StringZ).typev4;
                            match self.get_ro(converter, index) {
                                None => "no readable data found",
                                Some(cbuffer) => {
                                    let cstring =
                                        unsafe { CStr::from_ptr(&mut *(cbuffer as *mut Cchar)) };
                                    cstring.to_str().unwrap()
                                }
                            }
                        };
                        afb_error!(
                            concat!("export:", stringify!($afb_builtin_type)),
                            "invalid converter format args[{}]={}",
                            index,
                            data
                        )
                    }
                    Some(cbuffer) => Ok(unsafe { *(cbuffer as *mut $rust_type) }),
                }
            }
        }
    };
}
_register_query_converter!(i64, type_i64);
_register_query_converter!(i32, type_i32);
_register_query_converter!(u64, type_u64);
_register_query_converter!(u32, type_u32);
_register_query_converter!(bool, type_bool);
_register_query_converter!(f64, type_double);

impl ConvertQuery<String> for AfbRqtData {
    fn import(&self, index: usize) -> Result<String, AfbError> {
        let uid = "builtin-string";
        let converter = unsafe { (*cglue::afbBindingV4r1_itfptr).type_stringz };
        match self.get_ro(converter, index) {
            None => afb_error!(uid, "invalid converter format args[{}]", index),
            Some(cbuffer) => {
                let cstring = unsafe { CStr::from_ptr(&mut *(cbuffer as *mut Cchar)) };
                let slice: &str = cstring.to_str().unwrap();
                Ok(slice.to_owned())
            }
        }
    }
}

impl ConvertQuery<JsoncObj> for AfbRqtData {
    fn import(&self, index: usize) -> Result<JsoncObj, AfbError> {
        // retrieve builtin converter from libafb
        let uid = "builtin-JsoncObj";
        let converter = unsafe { (*cglue::afbBindingV4r1_itfptr).type_json_c };

        // retrieve c-buffer pointer to argument void* value
        match self.get_ro(converter, index) {
            None => afb_error!(uid, "invalid converter format args[{}]", index),
            Some(cbuffer) => JsoncObj::import(cbuffer),
        }
    }
}

pub struct AfbRqtData {
    count: u32,
    status: i32,
    argsv4: Vec<AfbDataV4>,
}

impl AfbRqtData {
    #[track_caller]
    pub fn new(args: &[AfbDataV4], argc: u32, status: i32) -> Self {
        AfbRqtData {
            count: argc,
            status: status,
            argsv4: args.to_owned(),
        }
    }

    #[track_caller]
    pub fn without_data(status: i32) -> Self {
        AfbRqtData::new(&[0 as cglue::afb_data_t; 0], 0, status)
    }

    #[track_caller]
    pub fn unref(&self) {
        for idx in 0..self.count as usize {
            let data = self.argsv4[idx];
            unsafe { cglue::afb_data_unref(data) };
        }
    }

    #[track_caller]
    pub fn addref(&self) {
        for idx in 0..self.count as usize {
            let data = self.argsv4[idx];
            unsafe { cglue::afb_data_addref(data) };
        }
    }

    #[track_caller]
    pub fn get<T>(&self, index: usize) -> Result<T, AfbError>
    where
        AfbRqtData: ConvertQuery<T>,
    {
        match self.check(index as i32) {
            Err(max) => afb_error!(
                "AfbRqtData.get",
                "invalid argument index ask:{} max:{}",
                index + 1,
                max
            ),
            Ok(()) => Self::import(self, index),
        }
    }

    // return argument only if status > 0
    #[track_caller]
    pub fn get_on_status<T>(&self, index: usize) -> Result<T, AfbError>
    where
        AfbRqtData: ConvertQuery<T>,
    {
        if self.status < 0 {
            afb_error!(
                "AfbRqtData.status",
                "value:{} info:{}",
                self.status,
                afb_error_info(self.status)
            )
        } else {
            self.get(index)
        }
    }

    // return argument only if status > 0
    #[track_caller]
    pub fn get_onsuccess<T>(&self, index: usize) -> Result<T, AfbError>
    where
        AfbRqtData: ConvertQuery<T>,
    {
        if self.status < 0 {
            afb_error!(
                "AfbRqtData.status",
                "value:{} info:{}",
                self.status,
                afb_error_info(self.status)
            )
        } else {
            self.get(index)
        }
    }

    // return Null is max argument count is index too big
    #[track_caller]
    pub fn check(&self, index: i32) -> Result<(), u32> {
        let count = self.count as i32;
        let check = if (index >= 0) && (index < count) {
            Ok(())
        } else if index < 0 && index.abs() == (count - 1) {
            Ok(())
        } else {
            Err(self.count)
        };
        check
    }

    #[track_caller]
    pub fn get_count(&self) -> u32 {
        self.count
    }

    #[track_caller]
    pub fn get_status(&self) -> i32 {
        self.status
    }

    #[track_caller]
    pub fn get_v4(&self, index: u32) -> cglue::afb_data_t {
        self.argsv4[index as usize]
    }

    #[track_caller]
    pub fn get_ro(&self, typev4: AfbTypeV4, index: usize) -> Option<*mut std::ffi::c_void> {
        let result = unsafe {
            let source = self.argsv4[index];
            let mut argument = 0 as cglue::afb_data_t;
            let status = cglue::afb_data_convert(source, typev4, &mut argument);
            if status == 0 {
                // read argument as cbuffer
                let cbuffer = cglue::afb_data_ro_pointer(argument);
                Some(cbuffer)
            } else {
                None
            }
        };
        result
    }

    #[track_caller]
    pub fn to_jsonc(&self) -> JsoncObj {
        let jsonc = JsoncObj::new();
        let jdata = JsoncObj::array();
        for idx in 0..self.count {
            let data = self.get::<JsoncObj>(idx as usize);
            let jsonc = match data {
                Err(error) => error.to_jsonc().unwrap(),
                Ok(data) => data,
            };
            jdata.append(jsonc).unwrap();
        }
        jsonc.add("status", self.status).unwrap();
        jsonc.add("response", jdata).unwrap();
        jsonc
    }
}

impl Clone for AfbRqtData {
    fn clone(&self) -> Self {
        self.addref();
        AfbRqtData { count: self.count, status: self.status, argsv4:  self.argsv4.clone()}
    }
}

pub struct AfbExportData {
    pub uid: &'static str,
    pub typev4: AfbTypeV4,
    pub buffer_ptr: *const std::ffi::c_void,
    pub buffer_len: usize,
    pub freecb: ::std::option::Option<unsafe extern "C" fn(arg1: *mut ::std::os::raw::c_void)>,
}

pub enum AfbExportResponse {
    Converter(AfbExportData),
    Response(AfbParams),
}

pub trait ConvertResponse<T> {
    fn export(data: T) -> AfbExportResponse;
}

macro_rules! _register_response_converter {
    ($rust_type:ty, $afb_builtin_type:ident) => {
        impl ConvertResponse<$rust_type> for AfbParams {
            #[track_caller]
            fn export(data: $rust_type) -> AfbExportResponse {
                // cast integer to c-void*
                let boxe = Box::new(data);
                let raw_data = Box::into_raw(boxe) as *mut std::ffi::c_void;
                let export = AfbExportData {
                    uid: concat!("export:", stringify!($afb_builtin_type)),
                    typev4: unsafe { (*cglue::afbBindingV4r1_itfptr).$afb_builtin_type },
                    buffer_ptr: raw_data,
                    buffer_len: 0, // auto
                    freecb: Some(free_box_cb),
                };
                AfbExportResponse::Converter(export)
            }
        }
    };
}
// converters with Rust/C equal binary representation
_register_response_converter!(i64, type_i64);
_register_response_converter!(i32, type_i32);
_register_response_converter!(u64, type_u64);
_register_response_converter!(u32, type_u32);
_register_response_converter!(bool, type_bool);
_register_response_converter!(f64, type_double);

impl ConvertResponse<JsoncObj> for AfbParams {
    #[track_caller]
    fn export(data: JsoncObj) -> AfbExportResponse {
        let export = AfbExportData {
            uid: "export:builtin-JsoncObj",
            typev4: unsafe { (*cglue::afbBindingV4r1_itfptr).type_json_c },
            buffer_ptr: data.into_raw() as *const _ as *mut std::ffi::c_void,
            buffer_len: 0, // auto
            freecb: Some(free_jsonc_cb),
        };
        AfbExportResponse::Converter(export)
    }
}

impl ConvertResponse<&JsoncObj> for AfbParams {
    #[track_caller]
    fn export(data: &JsoncObj) -> AfbExportResponse {
        let export = AfbExportData {
            uid: "export:builtin-&JsoncObj",
            typev4: unsafe { (*cglue::afbBindingV4r1_itfptr).type_json_c },
            buffer_ptr: data.into_raw() as *const _ as *mut std::ffi::c_void,
            buffer_len: 0, // auto
            freecb: Some(free_jsonc_cb),
        };
        AfbExportResponse::Converter(export)
    }
}

fn export_raw_string(data: &str) -> AfbExportResponse {
    // build valid c-string from data
    let cstring = CString::new(data).expect("Invalid data string");
    let data_ptr = cstring.into_raw() as *const _ as *mut std::ffi::c_void;

    let export = AfbExportData {
        uid: "export:builtin-string",
        typev4: unsafe { (*cglue::afbBindingV4r1_itfptr).type_stringz },
        buffer_ptr: data_ptr,
        buffer_len: data.len() + 1,
        freecb: Some(free_cstring_cb),
    };
    AfbExportResponse::Converter(export)
}

impl ConvertResponse<&str> for AfbParams {
    #[track_caller]
    fn export(data: &str) -> AfbExportResponse {
        if data.starts_with('{') || data.starts_with('[') {
            match JsoncObj::parse(data) {
                Ok(jvalue) => Self::export(jvalue),
                Err(_) => export_raw_string(data),
            }
        } else {
            export_raw_string(data)
        }
    }
}

impl ConvertResponse<String> for AfbParams {
    #[track_caller]
    fn export(data: String) -> AfbExportResponse {
        export_raw_string(data.as_str())
    }
}

impl ConvertResponse<&AfbError> for AfbParams {
    #[track_caller]
    fn export(data: &AfbError) -> AfbExportResponse {
        let error = data.to_jsonc().unwrap();
        Self::export(error)
    }
}

impl ConvertResponse<AfbError> for AfbParams {
    #[track_caller]
    fn export(data: AfbError) -> AfbExportResponse {
        let error = data.to_jsonc().unwrap();
        Self::export(error)
    }
}

// dummy converter when AfbParams is push reply
impl ConvertResponse<AfbParams> for AfbParams {
    #[track_caller]
    fn export(data: AfbParams) -> AfbExportResponse {
        AfbExportResponse::Response(data)
    }
}

// proxy converter when AfbRqtData is push reply
impl ConvertResponse<AfbRqtData> for AfbParams {
    #[track_caller]
    fn export(data: AfbRqtData) -> AfbExportResponse {
        let mut param = AfbParams::new();
        for idx in 0..data.count {
            let datav4 = data.argsv4[idx as usize];
            param.arguments.push(datav4);
        }
        AfbExportResponse::Response(param)
    }
}
// dummy converter to allow recursive call
impl ConvertResponse<AfbNoData> for AfbParams {
    #[track_caller]
    fn export(_data: AfbNoData) -> AfbExportResponse {
        AfbExportResponse::Response(AfbParams::new())
    }
}

pub struct AfbParams {
    pub arguments: Vec<cglue::afb_data_t>,
}

impl Clone for AfbParams {
    fn clone(&self) -> Self {
        let mut clone = AfbParams {
            arguments: Vec::new(),
        };

        for idx in 0..self.arguments.len() {
            let param = self.arguments[idx];
            unsafe { cglue::afb_data_addref(param) };
            clone.arguments.push(param);
        }
        clone
    }
}

impl AfbParams {
    #[track_caller]
    pub fn new() -> Self {
        AfbParams {
            arguments: Vec::new(),
        }
    }

    #[track_caller]
    pub fn addref(&self) {
        for data in &self.arguments {
            unsafe { cglue::afb_data_addref(*data) };
        }
    }

    #[track_caller]
    pub fn unref(&self) {
        for data in &self.arguments {
            unsafe { cglue::afb_data_unref(*data) };
        }
    }

    #[track_caller]
    pub fn from<T>(data_in: T) -> Result<Self, AfbError>
    where
        AfbParams: ConvertResponse<T>,
    {
        let mut param = AfbParams::new();
        // convert response data depending on type
        let mut data = match AfbParams::export(data_in) {
            AfbExportResponse::Converter(export) => export,
            _ => return afb_error!("afb_response::push", "invalid data type"),
        };

        match Self::insert(&mut param, &mut data) {
            Ok(()) => Ok(param),
            Err(error) => Err(error),
        }
    }

    fn insert(&mut self, data: &mut AfbExportData) -> Result<(), AfbError> {
        // provide box free_cb is none defined
        if let None = data.freecb {
            data.freecb = Some(free_box_cb);
        }

        // push data into libafb and retrieve it's handle
        let data_handle: cglue::afb_data_t = 0 as cglue::afb_data_t;
        let status = unsafe {
            cglue::afb_create_data_raw(
                &data_handle as *const _ as *mut cglue::afb_data_t,
                data.typev4,
                data.buffer_ptr,
                data.buffer_len,
                data.freecb,
                data.buffer_ptr as *mut c_void,
            )
        };
        if status != 0 {
            afb_error!(data.uid, "Fail:{} data export", data.uid)
        } else {
            self.arguments.push(data_handle);
            Ok(())
        }
    }

    #[track_caller]
    pub fn convert<T>(data_in: T) -> Result<AfbParams, AfbError>
    where
        AfbParams: ConvertResponse<T>,
    {
        // convert response data depending on type
        let mut data = match AfbParams::export(data_in) {
            AfbExportResponse::Response(response) => return Ok(response),
            AfbExportResponse::Converter(export) => export,
        };

        let mut response = AfbParams::new();
        match AfbParams::insert(&mut response, &mut data) {
            Ok(()) => Ok(response),
            Err(error) => Err(error),
        }
    }

    #[track_caller]
    pub fn push<T>(&mut self, data_in: T) -> Result<&mut Self, AfbError>
    where
        AfbParams: ConvertResponse<T>,
    {
        // convert response data depending on type
        let mut data = match AfbParams::export(data_in) {
            AfbExportResponse::Converter(export) => export,
            _ => return afb_error!("afb_response::push", "invalid data type"),
        };

        match Self::insert(self, &mut data) {
            Ok(()) => Ok(self),
            Err(error) => Err(error),
        }
    }
}
