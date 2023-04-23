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

use cglue::{self as cglue};
use jsonc::*;
use std::any::Any;
use std::boxed::Box;
use std::ffi::{c_void, CStr, CString};
use utilv4::*;

// alias few external types
pub type AfbTypeV4 = cglue::afb_type_t;
pub type AfbDataV4 = cglue::afb_data_t;
pub type AfbJsonObj = JsoncObj;
pub type AfbJsonStr = JsonStr;

// trick to create a null parameter
pub type AfbNoData = Option<std::ffi::c_void>;
pub const AFB_NO_AUTH: *const cglue::afb_auth = 0 as *const cglue::afb_auth;
pub const AFB_NO_DATA: AfbNoData = None;
pub const AFB_OK: i32 = 0;
pub const AFB_FAIL: i32 = 1;
pub const AFB_FATAL: i32 = -1;

pub struct ConverterBox(pub Option<&'static AfbConverter>);
unsafe impl Sync for ConverterBox {}

/// Predefined libafb type
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

pub use AfbDataConverter;
#[macro_export]
macro_rules! AfbDataConverter {
    ($uid:ident, $datat:ident) => {
        mod $uid {
            use super::*;
            use std::any::Any;
            pub static mut converter_box: ConverterBox = ConverterBox(None);

            pub fn encode(cbuffer: *mut std::ffi::c_void) -> Result<String, String> {
                let data = unsafe { &mut *(cbuffer as *mut $datat) };
                match serde_json::to_string(data) {
                    Ok(output) => Ok(output),
                    Err(error) => Err(error.to_string()),
                }
            }

            pub fn decode(json_string: &str) -> Result<Box<dyn Any>, String> {
                match serde_json::from_str::<$datat>(json_string) {
                    Ok(value) => Ok(Box::new(value)),
                    Err(error) => Err(error.to_string()),
                }
            }

            pub fn register(
            ) -> Result<&'static libafb::datav4::AfbConverter, libafb::utilv4::AfbError> {
                let converter =
                    libafb::datav4::AfbConverter::new(stringify!($uid)).and_then(|obj| {
                        obj.add_encoder(libafb::datav4::AfbBuiltinType::Json, encode, decode)
                        //Fulup tobe check with Jose
                        //obj.add_encoder(libafb::datav4::AfbBuiltinType::StringZ, encode, decode)
                    });

                match converter {
                    Ok(encoder) => {
                        unsafe {
                            converter_box =
                                ConverterBox(Some(encoder as &'static libafb::datav4::AfbConverter))
                        };
                        Ok(encoder as &'static libafb::datav4::AfbConverter)
                    }
                    Err(error) => Err(error),
                }
            }
        }

        impl ConvertQuery<&'static $datat> for libafb::datav4::AfbData {
            fn import(
                &self,
                index: usize,
            ) -> Result<&'static $datat, libafb::utilv4::AfbError> {
                //let converter= $uid::converter_box;
                let typev4 = match unsafe { &$uid::converter_box } {
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
                    None => Err(libafb::utilv4::AfbError::new(
                        concat!("import-", stringify!($datat)),
                        format!("invalid converter format args[{}]", index),
                    )),
                    Some(cbuffer) => Ok(unsafe { &mut *(cbuffer as *mut $datat) }),
                }
            }
        }

        impl ConvertResponse<$datat> for libafb::datav4::AfbParams {
            fn export(data: $datat) -> libafb::datav4::AfbExportResponse {
                let typev4 = match unsafe { &$uid::converter_box } {
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
                let uid = concat!("import-", stringify!($user_type));
                let boxe = Box::new(data);
                libafb::datav4::AfbExportResponse::Converter(AfbExportData {
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
// make macro visible at module level

impl AfbBuiltinType {
    fn get(builtin_type: &AfbBuiltinType) -> AfbConverter {
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
    let cbuffer = context as *mut i8;
    let cstring = unsafe { CString::from_raw(cbuffer) };
    drop(cstring);
}

// restore Rust Cstring, in order to make it disposable
#[no_mangle]
pub extern "C" fn free_box_cb(context: *mut std::ffi::c_void) {
    let cbox = unsafe { Box::from_raw(context) };
    drop(cbox);
}

type EncoderCb = fn(*mut std::ffi::c_void) -> Result<String, String>;
type DecoderCb = fn(&str) -> Result<Box<dyn Any>, String>;

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
            let cbuffer = CString::new(encoded)
                .expect("(hoops) invalid encoded string")
                .into_raw();
            let status = unsafe {
                cglue::afb_create_data_raw(
                    dest,
                    AfbBuiltinType::get(&AfbBuiltinType::Json).typev4,
                    cbuffer as *const _ as *mut std::ffi::c_void,
                    0,
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
        let cbuffer = cglue::afb_data_ro_pointer(source) as *const i8;
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
    pub fn get_encoder(&mut self) -> EncoderCb {
        self.encoder
    }

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
            Err(AfbError::new(uid, "fail to register converter data type"))
        }
    }

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
            Err(AfbError::new(&self._uid, "Fail adding encoding converter"))
        } else {
            Ok(self)
        }
    }

    // return object getter trait to prevent any malicious modification
    pub fn finalize(&mut self) -> &AfbConverter {
        self
    }

    pub fn get_uid(&self) -> &'static str {
        self._uid
    }

    pub fn get_typev4(&self) -> cglue::afb_type_t {
        self.typev4
    }
}

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
        Err(AfbError::new(uid, "type lookup fail"))
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
        impl ConvertQuery<$rust_type> for AfbData {
            fn import(&self, index: usize) -> Result<$rust_type, AfbError> {
                let converter = unsafe { (*cglue::afbBindingV4r1_itfptr).$afb_builtin_type };
                match self.get_ro(converter, index) {
                    None => Err(AfbError::new(
                        concat!("export-", stringify!($afb_builtin_type)),
                        format!("invalid converter format args[{}]", index),
                    )),
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

impl ConvertQuery<String> for AfbData {
    fn import(&self, index: usize) -> Result<String, AfbError> {
        let uid = "builtin-string";
        let converter = unsafe { (*cglue::afbBindingV4r1_itfptr).type_stringz };
        match self.get_ro(converter, index) {
            None => Err(AfbError::new(
                uid,
                format!("invalid converter format args[{}]", index),
            )),
            Some(cbuffer) => {
                let cstring = unsafe { CStr::from_ptr(&mut *(cbuffer as *mut i8)) };
                let slice: &str = cstring.to_str().unwrap();
                Ok(slice.to_owned())
            }
        }
    }
}

impl ConvertQuery<AfbJsonObj> for AfbData {
    fn import(&self, index: usize) -> Result<AfbJsonObj, AfbError> {
        // retrieve builtin converter from libafb
        let uid = "builtin-JsoncObj";
        let converter = unsafe { (*cglue::afbBindingV4r1_itfptr).type_json_c };

        // retrieve c-buffer pointer to argument void* value
        match self.get_ro(converter, index) {
            None => Err(AfbError::new(
                uid,
                format!("invalid converter format args[{}]", index),
            )),
            Some(cbuffer) => Ok(AfbJsonObj::from(cbuffer)),
        }
    }
}

pub struct AfbData {
    count: u32,
    status: i32,
    argsv4: Vec<AfbDataV4>,
}

impl AfbData {
    pub fn new(args: &[AfbDataV4], argc: u32, status: i32) -> Self {
        AfbData {
            count: argc,
            status: status,
            argsv4: args.to_owned(),
        }
    }

    pub fn get<T>(&self, index: usize) -> Result<T, AfbError>
    where
        AfbData: ConvertQuery<T>,
    {
        match self.check(index as i32) {
            Err(max) => Err(AfbError::new(
                "AfbData.get",
                format!("invalid argument index ask:{} max:{}", index + 1, max),
            )),
            Ok(()) => Self::import(self, index),
        }
    }

    // return Null is max argument count is index too big
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

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn get_status(&self) -> i32 {
        self.status
    }

    pub fn get_v4(&self, index: u32) -> cglue::afb_data_t {
        self.argsv4[index as usize]
    }

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

    pub fn to_jsonc(&self) -> AfbJsonObj {
        let jsonc = JsoncObj::new();
        let jdata = JsoncObj::array();
        for idx in 0..self.count {
            let data = self.get::<AfbJsonObj>(idx as usize);
            let jsonc = match data {
                Err(error) => error.to_jsonc(),
                Ok(data) => data,
            };
            jdata.insert(jsonc).unwrap();
        }
        jsonc.add("status", self.status).unwrap();
        jsonc.add("response", jdata).unwrap();
        jsonc
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
            fn export(data: $rust_type) -> AfbExportResponse {
                // cast integer to c-void*
                let boxe = Box::new(data);
                let raw_data = Box::into_raw(boxe) as *mut std::ffi::c_void;
                let export = AfbExportData {
                    //uid: "export-builtin-number",
                    uid: concat!("export-", stringify!($afb_builtin_type)),
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

impl ConvertResponse<AfbJsonObj> for AfbParams {
    fn export(data: AfbJsonObj) -> AfbExportResponse {
        let export = AfbExportData {
            uid: "export-builtin-JsoncObj",
            typev4: unsafe { (*cglue::afbBindingV4r1_itfptr).type_json_c },
            buffer_ptr: data.into_raw() as *const _ as *mut std::ffi::c_void,
            buffer_len: 0, // auto
            freecb: Some(free_box_cb),
        };
        AfbExportResponse::Converter(export)
    }
}

impl ConvertResponse<&AfbJsonObj> for AfbParams {
    fn export(data: &AfbJsonObj) -> AfbExportResponse {
        let export = AfbExportData {
            uid: "export-builtin-JsoncObj",
            typev4: unsafe { (*cglue::afbBindingV4r1_itfptr).type_json_c },
            buffer_ptr: (*data).into_raw() as *const _ as *mut std::ffi::c_void,
            buffer_len: 0, // auto
            freecb: None,
        };
        AfbExportResponse::Converter(export)
    }
}

impl ConvertResponse<&AfbJsonStr> for AfbParams {
    fn export(data: &AfbJsonStr) -> AfbExportResponse {
        // extract string from AfbJsonStr
        let json_str = data.0;

        // try to parse json, if invalid pass it as a string
        let export = match AfbJsonObj::parse(json_str) {
            Err(_error) => {
                let cstring = CString::new(json_str as &str).expect("Invalid data string");
                let data_ptr = cstring.into_raw() as *const _ as *mut std::ffi::c_void;
                AfbExportData {
                    uid: "export-builtin-stringz",
                    typev4: unsafe { (*cglue::afbBindingV4r1_itfptr).type_stringz },
                    buffer_ptr: data_ptr,
                    buffer_len: json_str.len() + 1,
                    freecb: Some(free_cstring_cb),
                }
            }
            Ok(jsonc) => AfbExportData {
                uid: "export-builtin-jsonc",
                typev4: unsafe { (*cglue::afbBindingV4r1_itfptr).type_json_c },
                buffer_ptr: jsonc.into_raw() as *const _ as *mut std::ffi::c_void,
                buffer_len: 0,
                freecb: Some(free_jsonc_cb),
            },
        };
        AfbExportResponse::Converter(export)
    }
}

impl ConvertResponse<&str> for AfbParams {
    fn export(data: &str) -> AfbExportResponse {
        // build valid c-string from data
        let cstring = CString::new(data).expect("Invalid data string");
        let data_ptr = cstring.into_raw() as *const _ as *mut std::ffi::c_void;

        let export = AfbExportData {
            uid: "export-builtin-string",
            typev4: unsafe { (*cglue::afbBindingV4r1_itfptr).type_stringz },
            buffer_ptr: data_ptr,
            buffer_len: data.len() + 1,
            freecb: Some(free_cstring_cb),
        };
        AfbExportResponse::Converter(export)
    }
}

impl ConvertResponse<String> for AfbParams {
    fn export(data: String) -> AfbExportResponse {
        Self::export(data.as_str())
    }
}

impl ConvertResponse<&String> for AfbParams {
    fn export(data: &String) -> AfbExportResponse {
        Self::export(data.as_str())
    }
}

impl ConvertResponse<&AfbError> for AfbParams {
    fn export(data: &AfbError) -> AfbExportResponse {
        let error = data.to_jsonc();
        Self::export(error)
    }
}

impl ConvertResponse<AfbError> for AfbParams {
    fn export(data: AfbError) -> AfbExportResponse {
        let error = data.to_jsonc();
        Self::export(error)
    }
}

// dummy converter when AfbParams is push reply
impl ConvertResponse<AfbParams> for AfbParams {
    fn export(data: AfbParams) -> AfbExportResponse {
        AfbExportResponse::Response(data)
    }
}

// proxy converter when AfbData is push reply
impl ConvertResponse<AfbData> for AfbParams {
    fn export(data: AfbData) -> AfbExportResponse {
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
    pub fn new() -> Self {
        AfbParams {
            arguments: Vec::new(),
        }
    }

    pub fn from<T>(data_in: T) -> Result<Self, AfbError>
    where
        AfbParams: ConvertResponse<T>,
    {
        let mut param= AfbParams::new();
        // convert response data depending on type
        let mut data = match AfbParams::export(data_in) {
            AfbExportResponse::Converter(export) => export,
            _ => return Err(AfbError::new("afb_response::push", "invalid data type")),
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
                Some(free_box_cb),
                data.buffer_ptr as *mut c_void,
            )
        };
        if status != 0 {
            Err(AfbError::new(
                data.uid,
                format!("Fail:{} data export", data.uid),
            ))
        } else {
            self.arguments.push(data_handle);
            Ok(())
        }
    }

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

    pub fn push<T>(&mut self, data_in: T) -> Result<&mut Self, AfbError>
    where
        AfbParams: ConvertResponse<T>,
    {
        // convert response data depending on type
        let mut data = match AfbParams::export(data_in) {
            AfbExportResponse::Converter(export) => export,
            _ => return Err(AfbError::new("afb_response::push", "invalid data type")),
        };

        match Self::insert(self, &mut data) {
            Ok(()) => Ok(self),
            Err(error) => Err(error),
        }
    }
}
