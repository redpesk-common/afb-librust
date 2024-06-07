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

use std::any::Any;
use std::boxed::Box;
use std::cell::Cell;
use std::ffi::{CStr, CString};

use std::fmt;
// libafb dependencies
use crate::prelude::*;

// alias few external types
pub type AfbApiV4 = cglue::afb_api_t;
pub type AfbRqtV4 = cglue::afb_req_t;
pub type AfbEvtV4 = cglue::afb_event_t;
pub const NULLPTR: *mut std::ffi::c_void = 0 as *mut std::ffi::c_void;

// maximum argument return from call sync functions
const MAX_CALL_ARGS: u32 = 10;

pub trait AfbApiSubCallControl {
    fn api_callback(&mut self, api: &AfbApi, args: &AfbRqtData) -> Result<(), AfbError>;
}

pub type RqtCallback =
    fn(rqt: &AfbRequest, args: &AfbRqtData, ctx: &AfbCtxData) -> Result<(), AfbError>;

#[track_caller]
fn rqt_default_cb(rqt: &AfbRequest, _args: &AfbRqtData, _ctx: &AfbCtxData) -> Result<(), AfbError> {
    afb_error!(
        "afb-default-cb",
        "uid:{} no verb callback defined",
        rqt.get_verb().get_uid()
    )
}

pub type ApiCallback =
    fn(api: &AfbApi, args: &AfbRqtData, ctx: &AfbCtxData) -> Result<(), AfbError>;

pub use crate::AfbBindingRegister;
#[macro_export]
macro_rules! AfbBindingRegister {
    ($callback:expr) => {
        // afbBindingV4entry is called right after binder loads a binding
        // this mandatory callback pass control from C to Rust libafb glue
        #[no_mangle]
        pub extern "C" fn afbBindingV4entry(
            apiv4: AfbApiV4,
            ctlid: *mut std::ffi::c_void,
            ctlarg: *mut std::ffi::c_void,
            api_data: *mut std::ffi::c_void,
        ) -> i32 {
            let jconf = afbv4::apiv4::afb_binding_get_config(apiv4, ctlid, ctlarg, api_data);
            match $callback(apiv4, jconf) {
                Ok(api) => {
                    afb_log_msg!(Notice, apiv4, "RUST api uid={} started", api.get_uid());
                    AFB_OK
                }
                Err(error) => {
                    let dbg = error.get_dbg();
                    afb_log_raw!(
                        Notice,
                        apiv4,
                        "binding init fail {} file: {}:{}:{}",
                        error.get_info(),
                        dbg.file,
                        dbg.line,
                        dbg.column
                    );
                    AFB_ABORT
                }
            }
        }
    };
}

pub use crate::AfbSessionRegister;
#[macro_export]
macro_rules! AfbSessionRegister {
    ($userdata: ident, $callback: ident) => {
        use crate::afbv4::utilv4::afb_error;
        #[allow(non_camel_case_types)]
        impl AfbRqtSession for $userdata {
            fn as_any(&mut self) -> &mut dyn Any {
                self
            }
            fn closing(&mut self) {
                $callback(self)
            }
        }

        impl $userdata {
            fn get<'a>(request: &'a AfbRequest) -> Result<&'a mut Self, AfbError> {
                match request.get_session() {
                    Err(error) => Err(error),
                    Ok(any) => match any.as_any().downcast_mut::<$userdata>() {
                        None => afb_error!("session-any-cast", "fail to restore <$userdata>"),
                        Some(value) => Ok(value),
                    },
                }
            }

            fn set<'a>(
                request: &'a AfbRequest,
                userdata: $userdata,
            ) -> Result<&'a mut Self, AfbError> {
                match request.set_session(Box::new(userdata)) {
                    Err(error) => Err(error),
                    Ok(any) => match any.as_any().downcast_mut::<$userdata>() {
                        None => afb_error!("session-any-cast", "fail to restore <$userdata>"),
                        Some(value) => Ok(value),
                    },
                }
            }

            fn unref(request: &AfbRequest) -> Result<(), afbv4::utilv4::AfbError> {
                request.drop_session()
            }
        }
    };
    ($userdata: ident) => {
        use crate::afbv4::utilv4::MakeError;
        #[allow(non_camel_case_types)]
        impl AfbRqtSession for $userdata {
            fn as_any(&mut self) -> &mut dyn Any {
                self
            }
        }

        impl $userdata {
            fn get<'a>(request: &'a AfbRequest) -> Result<&'a mut Self, AfbError> {
                match request.get_session() {
                    Err(error) => Err(error),
                    Ok(any) => match any.as_any().downcast_mut::<$userdata>() {
                        None => afb_error!("session-any-cast", "fail to restore <$userdata>"),
                        Some(value) => Ok(value),
                    },
                }
            }

            fn set<'a>(
                request: &'a AfbRequest,
                userdata: $userdata,
            ) -> Result<&'a mut Self, AfbError> {
                match request.set_session(Box::new(userdata)) {
                    Err(error) => Err(error),
                    Ok(any) => match any.as_any().downcast_mut::<$userdata>() {
                        None => afb_error!("session-any-cast", "fail to restore <$userdata>"),
                        Some(value) => Ok(value),
                    },
                }
            }

            fn unref(request: &AfbRequest) -> Result<(), afbv4::utilv4::AfbError> {
                request.drop_session()
            }
        }
    };
}

#[doc(hidden)]
fn add_verbs_to_group(
    uid: &'static str,
    info: &'static str,
    verbs: &mut Vec<*const AfbVerb>,
) -> JsoncObj {
    let jgroup = JsoncObj::new();
    if uid.len() > 0 {
        jgroup.add("uid", uid).unwrap();
    }
    if info.len() > 0 {
        jgroup.add("info", info).unwrap();
    }
    let jverbs = JsoncObj::array();
    for apiverb in verbs {
        let verb_ref = unsafe { &mut *(*apiverb as *mut AfbVerb) };
        let jverb = JsoncObj::new();
        jverb.add("uid", verb_ref.get_uid()).unwrap();
        jverb.add("verb", verb_ref.get_name()).unwrap();
        jverb.add("info", verb_ref.get_info()).unwrap();

        let jactions = verb_ref.get_action();
        if let Ok(count) = jactions.count() {
            let jusages = JsoncObj::new();
            if count > 0 {
                jusages.add("action", jactions).unwrap();
            };
            if let Some(jusage) = verb_ref.get_usage() {
                jusages.add("data", jusage).unwrap();
            }
            jverb.add("usage", jusages).unwrap();
        } else {
            if let Some(jusage) = verb_ref.get_usage() {
                jverb.add("usage", jusage).unwrap();
            }
        };

        let jsamples = verb_ref.get_samples();
        if let Ok(count) = jsamples.count() {
            if count > 0 {
                jverb.add("sample", jsamples).unwrap();
            };
        };
        jverbs.append(jverb).unwrap();
    }
    jgroup.add("verbs", jverbs).unwrap();
    jgroup
}

// restore Rust Cstring, in order to make it disposable
#[no_mangle]
pub extern "C" fn free_session_cb(context: *mut std::ffi::c_void) {
    // Fulup why session drop is not called ????
    let wrap = unsafe { &mut *(context as *const _ as *mut AfbRqtSessionWrap) };
    wrap.inner.closing();
    let cbox = unsafe { Box::from_raw(context) };
    drop(cbox);
}

#[no_mangle]
pub extern "C" fn api_info_cb(
    rqtv4: cglue::afb_req_t,
    _argc: u32,
    _args: *const cglue::afb_data_t,
) {
    // extract api object from libafb vcbdata
    let api_ref = unsafe {
        let vcbdata = cglue::afb_req_get_vcbdata(rqtv4);
        &mut *(vcbdata as *mut AfbApi)
    };

    // build final jinfo object with metadata and groups
    let jinfo = JsoncObj::new();

    // create api introspection metadata
    let jmeta = JsoncObj::new();
    jmeta.add("uid", api_ref.get_uid()).unwrap();
    jmeta.add("info", api_ref.get_info()).unwrap();
    jmeta.add("version", api_ref.get_version()).unwrap();
    jinfo.add("metadata", jmeta).unwrap();

    // create groups array to host verbs
    let jgroups = JsoncObj::array();
    jgroups
        .append(add_verbs_to_group("", "", &mut api_ref.verbs))
        .unwrap();

    for slot in &api_ref.groups {
        let group_ref = unsafe { &mut *(*slot as *mut AfbGroup) };
        jgroups
            .append(add_verbs_to_group(
                group_ref._uid,
                group_ref.info,
                &mut group_ref.verbs,
            ))
            .unwrap();
    }

    jinfo.add("groups", jgroups).unwrap();

    // create a dummy Rust request and send jinfo response (Fulup: Rust is unfriendly with void*=NULL)
    let nullptr: *mut std::ffi::c_void = 0 as *mut std::ffi::c_void;
    let nullapi = unsafe { &mut *(nullptr as *mut AfbApi) };
    let nullverb = unsafe { &mut *(nullptr as *mut AfbVerb) };
    let request = AfbRequest::new(rqtv4, nullapi, nullverb);
    request.reply(jinfo, 0);
}

#[no_mangle]
pub extern "C" fn api_ping_cb(
    rqtv4: cglue::afb_req_t,
    _argc: u32,
    _args: *const cglue::afb_data_t,
) {
    // increment counter for each ping
    static mut COUNTER: u32 = 0;
    unsafe { COUNTER += 1 };

    // build final jinfo object with metadata and groups
    let jpong = JsoncObj::new();
    jpong.add("pong", unsafe { COUNTER }).unwrap();

    // create a dummy Rust request and send jinfo response (Fulup: Rust is unfriendly with void*=NULL)
    let nullptr: *mut std::ffi::c_void = 0 as *mut std::ffi::c_void;
    let nullapi = unsafe { &mut *(nullptr as *mut AfbApi) };
    let nullverb = unsafe { &mut *(nullptr as *mut AfbVerb) };
    let request = AfbRequest::new(rqtv4, nullapi, nullverb);
    request.reply(jpong, 0);
}

pub trait AfbApiControls {
    fn config(&mut self, api: &AfbApi, config: JsoncObj) -> Result<(), AfbError> {
        afb_log_msg!(
            Notice,
            api,
            "api preinit: uid:{}, name:{}, config:{}",
            api._uid,
            api.name,
            config
        );
        Ok(())
    }

    fn start(&mut self, api: &AfbApi) -> Result<(), AfbError> {
        afb_log_msg!(Debug, api, "api init uid:{}", api._uid);
        Ok(())
    }

    #[doc(hidden)]
    fn ready(&mut self, api: &AfbApi) -> Result<(), AfbError> {
        afb_log_msg!(Debug, api, "api ready uid:{}", api._uid);
        Ok(())
    }

    fn orphan(&mut self, api: &AfbApi, signal: &str) {
        afb_log_msg!(Info, api, "orphan event api:{} event: {}", api._uid, signal);
    }

    fn exit(&mut self, api: &AfbApi, code: i32) -> i32 {
        afb_log_msg!(Debug, api, "api exit: uid:{} code:{}", api._uid, code);
        return code;
    }

    fn as_any(&mut self) -> &mut dyn Any;
}

#[doc(hidden)]
fn binding_parse_config(apiv4: cglue::afb_api_t, ctlarg: cglue::afb_ctlarg_t) -> JsoncObj {
    assert!(ctlarg.is_null() != true);
    let jso: *mut std::ffi::c_void =
        unsafe { (*ctlarg).root_entry.config } as *mut _ as *mut std::ffi::c_void;

    // extract config rust object from C void* ctrlbox
    if jso != 0 as *mut std::ffi::c_void {
        JsoncObj::from(jso)
    } else {
        // libafb may not pass api config as expected
        let jso = unsafe { cglue::afb_api_settings(apiv4) };
        JsoncObj::from(jso as *mut std::ffi::c_void)
    }
}

#[doc(hidden)]
pub fn afb_binding_get_config(
    apiv4: AfbApiV4,
    _ctlid_v4: *mut std::ffi::c_void,
    ctlarg_v4: *mut std::ffi::c_void,
    _apidata: *mut std::ffi::c_void,
) -> JsoncObj {
    // return Rust binding config
    let ctlarg = ctlarg_v4 as cglue::afb_ctlarg_t;
    let apiv4 = apiv4 as cglue::afb_api_t;
    binding_parse_config(apiv4, ctlarg)
}

#[no_mangle]
pub extern "C" fn api_controls_cb(
    apiv4: cglue::afb_api_t,
    ctlid: cglue::afb_ctlid_t,
    ctlarg_v4: cglue::afb_ctlarg_t,
    apictx: *mut std::ffi::c_void,
) -> i32 {
    // extract config rust object from C void* ctrlbox
    //let ctlid = ctlid_v4 as cglue::afb_ctlid_t;
    let ctlarg = ctlarg_v4 as cglue::afb_ctlarg_t;
    let api_ref = unsafe { &*(apictx as *mut AfbApi) };

    let status = match ctlid {
        cglue::afb_ctlid_afb_ctlid_Pre_Init => {
            // reference native afb apiv4 within rust api object
            api_ref.set_apiv4(apiv4);
            let mut status = match api_ref.ctrlbox {
                Some(ctrlbox) => {
                    match unsafe { (*ctrlbox).config(api_ref, binding_parse_config(apiv4, ctlarg)) }
                    {
                        Err(error) => {
                            let dbg = error.get_dbg();
                            afb_log_raw!(
                                Critical,
                                apiv4,
                                "binding config fail:{} file: {}:{}:{}",
                                error.get_info(),
                                dbg.file,
                                dbg.line,
                                dbg.column
                            );
                            AFB_FAIL
                        }
                        Ok(()) => AFB_OK,
                    }
                }
                None => 0,
            };

            if status >= 0 {
                for slot in &api_ref.require_apis {
                    let name = CString::new(*slot).expect("invalid api name");
                    let rc = unsafe { cglue::afb_api_require_api(apiv4, name.as_ptr(), 0) };
                    if rc < 0 {
                        afb_log_msg!(Critical, apiv4, "Require on api:{} fail", slot);
                    }
                }
            }

            if status >= 0 {
                for slot in &api_ref.require_classes {
                    let name = CString::new(*slot).expect("invalid api name");
                    let rc = unsafe { cglue::afb_api_require_class(apiv4, name.as_ptr()) };
                    if rc < 0 {
                        afb_log_msg!(Critical, apiv4, "Require on api class:{} fail", slot);
                    }
                }
            }
            let api_auth = AfbPermisionV4::new(api_ref.permission, AFB_AUTH_DFLT_V4);

            // pre_init config ok, let's loop on api verb array registration
            if status >= 0 {
                for slot in &api_ref.verbs {
                    let verb_ref = unsafe { &mut *(*slot as *mut AfbVerb) };

                    // use api verbosity is higger than verb one
                    if verb_ref.verbosity < 0 {
                        verb_ref.verbosity = verb_ref.verbosity * -1;
                    } else if api_ref.verbosity > verb_ref.verbosity {
                        verb_ref.verbosity = api_ref.verbosity;
                    }

                    verb_ref.register(apiv4, api_auth);
                    if status < 0 {
                        afb_log_msg!(
                            Critical,
                            api_ref._apiv4.get(),
                            "Fail to register verb={}",
                            verb_ref.get_uid()
                        );
                        break;
                    };
                }
            }

            if status >= 0 {
                for slot in &api_ref.groups {
                    let group_ref = unsafe { &mut *(*slot as *mut AfbGroup) };
                    status = group_ref.register(apiv4, api_auth);
                    if status < 0 {
                        afb_log_msg!(
                            Critical,
                            api_ref._apiv4.get(),
                            "Fail to register group={}",
                            group_ref.get_uid()
                        );
                        break;
                    };
                }
            };

            if status >= 0 {
                for slot in &api_ref.evthandlers {
                    let event_ref = unsafe { &mut *(*slot as *mut AfbEvtHandler) };
                    // use api verbosity is higger than verb one
                    if event_ref.verbosity < 0 {
                        event_ref.verbosity = event_ref.verbosity * -1;
                    } else if api_ref.verbosity > event_ref.verbosity {
                        event_ref.verbosity = api_ref.verbosity;
                    }
                    status = event_ref.register(apiv4);
                    if status < 0 {
                        afb_log_msg!(
                            Critical,
                            api_ref._apiv4.get(),
                            "Fail to register event={}",
                            event_ref.get_uid()
                        );
                        break;
                    };
                }
            };

            if status >= 0 {
                for slot in &api_ref.events {
                    let event_ref = unsafe { &mut *(*slot as *mut AfbEvent) };
                    // use api verbosity is higger than verb one
                    if event_ref.verbosity < 0 {
                        event_ref.verbosity = event_ref.verbosity * -1;
                    } else if api_ref.verbosity > event_ref.verbosity {
                        event_ref.verbosity = api_ref.verbosity;
                    }
                    status = event_ref.register(apiv4);
                    if status < 0 {
                        afb_log_msg!(
                            Critical,
                            api_ref._apiv4.get(),
                            "Fail to register event={}",
                            event_ref.get_uid()
                        );
                        break;
                    };
                }
            };

            // add verb ping
            if status >= 0 && api_ref.do_ping == true {
                let verb_name = CString::new("ping").unwrap();
                let verb_info = CString::new("libafb default api check").unwrap();
                status = unsafe {
                    cglue::afb_api_add_verb(
                        apiv4,
                        verb_name.as_ptr(),
                        verb_info.as_ptr(),
                        Some(api_ping_cb),
                        api_ref as *const _ as *mut std::ffi::c_void,
                        AFB_NO_AUTH, // const struct afb_auth *auth,
                        0,
                        0,
                    )
                }
            };

            // add verb info
            if status >= 0 && api_ref.do_info == true {
                let verb_name = CString::new("info").unwrap();
                let verb_info =
                    CString::new("libafb automatic introspection of api verbs").unwrap();
                status = unsafe {
                    cglue::afb_api_add_verb(
                        apiv4,
                        verb_name.as_ptr(),
                        verb_info.as_ptr(),
                        Some(api_info_cb),
                        api_ref as *const _ as *mut std::ffi::c_void,
                        AFB_NO_AUTH, // const struct afb_auth *auth,
                        0,
                        0,
                    )
                }
            };
            if status >= 0 && api_ref.do_seal {
                unsafe { cglue::afb_api_seal(apiv4) }
            }
            status
        }

        cglue::afb_ctlid_afb_ctlid_Init => match api_ref.ctrlbox {
            Some(ctrlbox) => match unsafe { (*ctrlbox).start(api_ref) } {
                Ok(()) => AFB_OK,
                Err(error) => {
                    let dbg = error.get_dbg();
                    afb_log_raw!(
                        Critical,
                        apiv4,
                        "binding start fail:{} file: {}:{}:{}",
                        error.get_info(),
                        dbg.file,
                        dbg.line,
                        dbg.column
                    );
                    AFB_FAIL
                }
            },
            None => AFB_OK,
        },

        cglue::afb_ctlid_afb_ctlid_Class_Ready => match api_ref.ctrlbox {
            Some(ctrlbox) => match unsafe { (*ctrlbox).ready(api_ref) } {
                Ok(()) => AFB_OK,
                Err(error) => {
                    let dbg = error.get_dbg();
                    afb_log_raw!(
                        Critical,
                        apiv4,
                        "binding class fail:{} file: {}:{}:{}",
                        error.get_info(),
                        dbg.file,
                        dbg.line,
                        dbg.column
                    );
                    AFB_FAIL
                }
            },
            None => AFB_OK,
        },

        cglue::afb_ctlid_afb_ctlid_Orphan_Event => match api_ref.ctrlbox {
            Some(ctrlbox) => {
                let cbuffer = unsafe { (*ctlarg).orphan_event.name };
                let cname = unsafe { CStr::from_ptr(cbuffer) };
                unsafe {
                    (*ctrlbox).orphan(api_ref, cname.to_str().unwrap());
                };
                AFB_OK
            }
            None => AFB_OK,
        },

        cglue::afb_ctlid_afb_ctlid_Exiting => match api_ref.ctrlbox {
            Some(ctrlbox) => unsafe { (*ctrlbox).exit(api_ref, (*ctlarg).exiting.code) },
            None => AFB_OK,
        },

        _ => {
            panic!("Rust ApiControl unknown ctlid (hoop!!!)");
        }
    };

    return status;
}

pub struct AfbApi {
    _uid: &'static str,
    _count: usize,
    _apiv4: Cell<cglue::afb_api_t>,
    name: &'static str,
    info: &'static str,
    version: &'static str,
    permission: &'static AfbPermission,
    class: &'static str,
    verbosity: i32,
    do_info: bool,
    do_ping: bool,
    do_seal: bool,
    do_concurrency: bool,
    verbs: Vec<*const AfbVerb>,
    evthandlers: Vec<*const AfbEvtHandler>,
    events: Vec<*const AfbEvent>,
    groups: Vec<*const AfbGroup>,
    ctrlbox: Option<*mut dyn AfbApiControls>,
    require_apis: Vec<&'static str>,
    require_classes: Vec<&'static str>,
}

impl AfbApi {
    pub fn new(uid: &'static str) -> &'static mut Self {
        let api_box = Box::new(AfbApi {
            _uid: uid,
            _count: 0,
            _apiv4: Cell::new(0 as cglue::afb_api_t),
            do_info: true,
            do_seal: true,
            do_ping: true,
            name: uid,
            info: "",
            class: "",
            version: "",
            verbosity: AfbLogLevel::Notice as i32,
            permission: AfbPermission::new(0),
            do_concurrency: true,
            ctrlbox: None,
            verbs: Vec::new(),
            events: Vec::new(),
            evthandlers: Vec::new(),
            groups: Vec::new(),
            require_apis: Vec::new(),
            require_classes: Vec::new(),
        });
        Box::leak(api_box)
    }

    pub fn set_name(&mut self, value: &'static str) -> &mut Self {
        self.name = value;
        self
    }

    pub fn set_info(&mut self, value: &'static str) -> &mut Self {
        self.info = value;
        self
    }

    pub fn set_class(&mut self, value: &'static str) -> &mut Self {
        self.class = value;
        self
    }

    pub fn add_info_cb(&mut self, value: bool) -> &mut Self {
        self.do_info = value;
        self
    }

    pub fn add_ping_cb(&mut self, value: bool) -> &mut Self {
        self.do_ping = value;
        self
    }

    pub fn seal(&mut self, value: bool) -> &mut Self {
        self.do_seal = value;
        self
    }

    pub fn set_version(&mut self, value: &'static str) -> &mut Self {
        self.version = value;
        self
    }

    pub fn set_permission(&mut self, value: &'static AfbPermission) -> &mut Self {
        self.permission = value;
        self
    }

    pub fn set_concurrency(&mut self, value: bool) -> &mut Self {
        self.do_concurrency = value;
        self
    }

    pub fn set_verbosity(&mut self, value: i32) -> &mut Self {
        self.verbosity = value;
        self
    }

    pub fn get_verbosity(&self) -> i32 {
        self.verbosity
    }

    pub fn add_verb(&mut self, verb: &AfbVerb) -> &mut Self {
        self.verbs.push(verb);
        self
    }

    pub fn add_event(&mut self, event: &'static AfbEvent) -> &mut Self {
        self.events.push(event);
        self
    }

    pub fn add_group(&mut self, group: &AfbGroup) -> &mut Self {
        self.groups.push(group);
        self
    }

    pub fn add_evt_handler(&mut self, handler: &AfbEvtHandler) -> &mut Self {
        self.evthandlers.push(handler);
        self
    }

    pub fn set_callback(&mut self, ctrlbox: Box<dyn AfbApiControls>) -> &mut Self {
        self.ctrlbox = Some(Box::leak(ctrlbox));
        self
    }

    pub fn require_api(&mut self, value: &'static str) -> &mut Self {
        if value != "" {
            self.require_apis.push(value);
        }
        self
    }

    pub fn require_class(&mut self, value: &'static str) -> &mut Self {
        self.require_classes.push(value);
        self
    }

    #[doc(hidden)]
    // hack to update apiv4 after api object creation
    pub fn set_apiv4(&self, apiv4: cglue::afb_api_t) {
        self._apiv4.set(apiv4);
    }

    pub fn finalize(&mut self) -> Result<&AfbApi, AfbError> {
        let api_name = CString::new(self.name).expect("invalid api name");
        let api_info = CString::new(self.info).expect("invalid api info");

        let api_concurrency: i32 = if self.do_concurrency == true { 0 } else { 1 };
        let status = unsafe {
            let mut _newapi: cglue::afb_api_t = 0 as cglue::afb_api_t;
            cglue::afb_create_api(
                &mut _newapi as *const _ as *mut cglue::afb_api_t,
                api_name.as_ptr(),
                api_info.as_ptr(),
                api_concurrency,
                Some(api_controls_cb),
                self as *const _ as *mut std::ffi::c_void,
            )
        };

        if status < 0 {
            afb_error!(
                self._uid,
                "Fail to register api uid={} status={} info={} ",
                self._uid,
                status,
                afb_error_info(status)
            )
        } else {
            Ok(self)
        }
    }

    pub fn get_uid(&self) -> &'static str {
        self._uid
    }
    pub fn get_name(&self) -> &'static str {
        self.name
    }
    pub fn get_info(&self) -> &'static str {
        self.info
    }
    pub fn get_apiv4(&self) -> cglue::afb_api_t {
        self._apiv4.get()
    }
    pub fn get_version(&self) -> &'static str {
        self.version
    }
    pub fn getctrlbox(&self) -> &mut dyn AfbApiControls {
        match self.ctrlbox {
            None => panic!(
                "(hoops) no userdata attach to api={} require --> .set_callback(Box::new(ApiuserData{{...}}) <--",
                self._uid
            ),
            Some(ctrlbox) => unsafe { &mut (*ctrlbox) },
        }
    }
}

impl fmt::Display for AfbApi {
    fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            format,
            "uid:{} name:{} info:{}",
            self._uid, self.name, self.info
        )
    }
}

#[no_mangle]
pub extern "C" fn api_verbs_cb(rqtv4: cglue::afb_req_t, argc: u32, args: *const cglue::afb_data_t) {
    // extract verb+api object from libafb internals
    let verb_ctx = unsafe { cglue::afb_req_get_vcbdata(rqtv4) };
    let verb_ref = unsafe { &mut *(verb_ctx as *mut AfbVerb) };

    // extract api_ref from libafb
    let api_ref = unsafe {
        let apiv4 = cglue::afb_req_get_api(rqtv4);
        let api_data = cglue::afb_api_get_userdata(apiv4);
        &mut *(api_data as *mut AfbApi)
    };
    // build new request reference count object
    // fulup to be done RUST and libafb refcount should be aligned
    api_ref._count += 1;
    verb_ref._count += 1;

    // move const **array in something Rust may understand
    let arguments = AfbRqtData::new(
        unsafe { std::slice::from_raw_parts(args as *const cglue::afb_data_t, argc as usize) },
        argc,
        0,
    );

    let request = AfbRequest::new(rqtv4, api_ref, verb_ref);
    let result = (verb_ref.callback)(&request, &arguments, &verb_ref.context);
    match result {
        Ok(()) => {}
        Err(error) => {
            let dbg = error.get_dbg();
            afb_log_raw!(
                Notice,
                &request,
                "{} file: {}:{}:{}",
                error,
                dbg.file,
                dbg.line,
                dbg.column
            );
            request.reply(error, -100);
        }
    }
}

pub struct AfbVerb {
    _uid: &'static str,
    _count: usize,
    name: &'static str,
    info: &'static str,
    permission: &'static AfbPermission,
    verbosity: i32,
    usage: Option<&'static str>,
    samples: JsoncObj,
    actions: JsoncObj,
    callback: RqtCallback,
    context: AfbCtxData,
}

impl AfbVerb {
    pub fn new(uid: &'static str) -> &'static mut Self {
        let verb_box = Box::new(AfbVerb {
            _uid: uid,
            _count: 0,
            name: uid,
            info: "",
            verbosity: 0,
            permission: AfbPermission::new(0),
            usage: None,
            samples: JsoncObj::array(),
            actions: JsoncObj::array(),
            callback: rqt_default_cb,
            context: AfbCtxData::new(AFB_NO_DATA),
        });
        Box::leak(verb_box)
    }
    pub fn set_name(&mut self, value: &'static str) -> &mut Self {
        self.name = value;
        self
    }

    pub fn set_info(&mut self, value: &'static str) -> &mut Self {
        self.info = value;
        self
    }

    pub fn set_permission(&mut self, value: &'static AfbPermission) -> &mut Self {
        self.permission = value;
        self
    }

    pub fn set_usage(&mut self, value: &'static str) -> &mut Self {
        self.usage = Some(value);
        self
    }

    #[track_caller]
    pub fn set_sample(&mut self, value: &'static str) -> Result<&mut Self, AfbError> {
        let jparse = JsoncObj::parse(value);
        match jparse {
            Err(_error) => afb_error!("jsonc-parsing-error", value.to_string()),
            Ok(jvalue) => {
                self.samples.append(jvalue).unwrap();
                Ok(self)
            }
        }
    }

    #[track_caller]
    pub fn set_action(&mut self, value: &'static str) -> Result<&mut Self, AfbError> {
        let jparse = JsoncObj::parse(value);
        match jparse {
            Err(error) => Err(error),
            Ok(jvalue) => {
                if jvalue.is_type(Jtype::Array) {
                    self.actions = jvalue;
                    Ok(self)
                } else {
                    afb_error!("verb-set-action", "not a valid json array")
                }
            }
        }
    }

    pub fn set_verbosity(&mut self, value: i32) -> &mut Self {
        self.verbosity = value;
        self
    }

    pub fn get_verbosity(&self) -> i32 {
        self.verbosity
    }

    pub fn set_callback(&mut self, callback: RqtCallback) -> &mut Self {
        self.callback = callback;
        self
    }

    pub fn set_context<T>(&mut self, ctx: T) -> &mut Self
    where
        T: 'static,
    {
        self.context = AfbCtxData::new(ctx);
        self
    }

    pub fn register(&self, apiv4: cglue::afb_api_t, inherited_auth: *const AfbAuthV4) -> i32 {
        let verb_name = CString::new(self.name).expect("invalid verb name");
        let verb_info = CString::new(self.info).expect("invalid verb info");

        let verb_permission: *mut AfbAuthV4 = AfbPermisionV4::new(self.permission, inherited_auth);

        unsafe {
            cglue::afb_api_add_verb(
                apiv4,
                verb_name.as_ptr(),
                verb_info.as_ptr(),
                Some(api_verbs_cb),
                self as *const _ as *mut std::ffi::c_void,
                verb_permission,
                0,
                0,
            )
        }
    }

    pub fn finalize(&mut self) -> Result<&Self, AfbError> {
        Ok(self)
    }

    pub fn get_uid(&self) -> &'static str {
        self._uid
    }
    pub fn get_name(&self) -> &'static str {
        self.name
    }
    pub fn get_info(&self) -> &'static str {
        self.info
    }
    pub fn get_usage(&self) -> Option<&'static str> {
        self.usage.clone()
    }
    pub fn get_samples(&self) -> JsoncObj {
        self.samples.clone()
    }
    pub fn get_action(&self) -> JsoncObj {
        self.actions.clone()
    }
}

impl fmt::Display for AfbVerb {
    fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            format,
            "uid:{} name:{} info:{}",
            self._uid, self.name, self.info
        )
    }
}

#[derive(Clone)]
pub struct AfbRequest {
    _rqtv4: cglue::afb_req_t,
    api: &'static AfbApi,
    verb: &'static AfbVerb,
}

// Rust dynamic object are fat pointer and should be encapculated before passing to C
struct AfbRqtSessionWrap {
    inner: Box<dyn AfbRqtSession>,
}
pub trait AfbRqtSession {
    fn as_any(&mut self) -> &mut dyn Any;
    fn closing(&mut self) {}
}

impl AfbRequest {
    pub fn new(rqtv4: cglue::afb_req_t, api: &'static AfbApi, verb: &'static AfbVerb) -> Self {
        AfbRequest {
            _rqtv4: unsafe { cglue::afb_req_addref(rqtv4) },
            verb: verb,
            api: api,
        }
    }

    pub fn set_session(
        &self,
        value: Box<dyn AfbRqtSession>,
    ) -> Result<&mut dyn AfbRqtSession, AfbError> {
        let wrapper = Box::new(AfbRqtSessionWrap { inner: value });
        let session = Box::leak(wrapper);
        let status = unsafe {
            cglue::afb_req_context_set(
                self.get_rqtv4(),
                session as *const _ as *mut ::std::os::raw::c_void,
                Some(free_session_cb),
                session as *const _ as *mut ::std::os::raw::c_void,
            )
        };
        if status < 0 {
            afb_error!("rqt-session-exist", "request fail to create session")
        } else {
            Ok(session.inner.as_mut())
        }
    }

    pub fn drop_session(&self) -> Result<(), AfbError> {
        let status = unsafe { cglue::afb_req_context_drop(self.get_rqtv4()) };
        if status < 0 {
            afb_error!("rqt-session-missing", "request session not defined")
        } else {
            Ok(())
        }
    }
    pub fn get_session(&self) -> Result<&mut dyn AfbRqtSession, AfbError> {
        let session = 0 as *mut ::std::os::raw::c_void;
        let status = unsafe {
            cglue::afb_req_context_get(
                self.get_rqtv4(),
                &session as *const _ as *mut *mut ::std::os::raw::c_void,
            )
        };
        if status < 0 {
            afb_error!("rqt-session-missing", "request session does not exit")
        } else {
            let session = unsafe { &mut *(session as *mut AfbRqtSessionWrap) };
            Ok(session.inner.as_mut())
        }
    }

    pub fn from_raw(rqtv4: AfbRqtV4) -> Self {
        // extract api_ref from libafb
        let api_ref = unsafe {
            let apiv4 = cglue::afb_req_get_api(rqtv4);
            let api_data = cglue::afb_api_get_userdata(apiv4);
            &mut *(api_data as *mut AfbApi)
        };

        // retreive source verb object
        let verb_ctx = unsafe { cglue::afb_req_get_vcbdata(rqtv4) };
        let verb_ref = unsafe { &mut *(verb_ctx as *mut AfbVerb) };

        AfbRequest {
            _rqtv4: rqtv4,
            verb: verb_ref,
            api: api_ref,
        }
    }

    pub fn get_uid(&self) -> String {
        format!("rqt:{:p}", self)
    }

    pub fn get_verb(&self) -> &'static AfbVerb {
        self.verb
    }

    pub fn get_api(&self) -> &'static AfbApi {
        self.api
    }

    pub fn get_apiv4(&self) -> AfbApiV4 {
        self.api._apiv4.get()
    }

    pub fn get_rqtv4(&self) -> cglue::afb_req_t {
        self._rqtv4
    }

    pub fn get_apidata(&self) -> &mut dyn Any {
        self.get_api().getctrlbox().as_any()
    }

    pub fn set_loa(&self, loa: u32) -> Result<u32, AfbError> {
        let status = unsafe { cglue::afb_req_session_set_LOA(self._rqtv4, loa) };
        if status < 0 {
            afb_error!(
                &self.get_uid(),
                "invalid LOA={} api={} verb={}",
                loa,
                self.get_api().get_uid(),
                self.get_verb().get_uid()
            )
        } else {
            Ok(loa)
        }
    }

    pub fn get_client_info(&self) -> JsoncObj {
        let jso = unsafe { cglue::afb_req_get_client_info(self._rqtv4) as *mut std::ffi::c_void };
        JsoncObj::from(jso)
    }

    pub fn add_ref(&self) -> Self {
        unsafe {
            cglue::afb_req_addref(self._rqtv4);
        }
        self.clone()
    }

    pub fn un_ref(&self) -> Self {
        unsafe {
            cglue::afb_req_unref(self._rqtv4);
        }
        self.clone()
    }

    pub fn reply<T>(&self, args: T, status: i32)
    where
        AfbParams: ConvertResponse<T>,
    {
        let response = AfbParams::convert(args);
        let params = match response {
            Err(error) => {
                afb_log_msg!(Critical, self, &error);
                return;
            }
            Ok(data) => data,
        };
        unsafe {
            cglue::afb_req_reply(
                self._rqtv4,
                status,
                params.arguments.len() as u32,
                params.arguments.as_slice().as_ptr(),
            )
        };
    }
}

#[doc(hidden)]
impl<'a> Drop for AfbRequest {
    fn drop(&mut self) {
        unsafe {
            cglue::afb_req_unref(self._rqtv4);
        }
    }
}

impl<'a> fmt::Display for AfbRequest {
    fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            let self_ref = &*(self as *const _ as *mut AfbRequest);
            let api_ref = & *(self_ref.get_api() as *const _ as *mut AfbApi);
            let verb_ref = & *(self_ref.get_verb() as *const _ as *mut AfbVerb);

            let api_uid = api_ref.get_uid();
            let verb_uid = verb_ref.get_uid();

            write!(format, "api:'{}', verb:'{}'}}", api_uid, verb_uid,)
        }
    }
}

pub struct AfbEventMsg<'a> {
    _uid: String,
    name: &'a str,
    api: &'a AfbApi,
    handler: &'a AfbEvtHandler,
}

impl<'a> AfbEventMsg<'a> {
    pub fn new(uid: String, name: &'a str, api: &'a AfbApi, handler: &'a AfbEvtHandler) -> Self {
        AfbEventMsg {
            _uid: uid,
            api: api,
            name: name,
            handler: handler,
        }
    }

    pub fn get_verbosity(&self) -> i32 {
        self.handler.get_verbosity()
    }

    pub fn get_uid(&'a self) -> &'a str {
        self._uid.as_str()
    }

    pub fn get_name(&'a self) -> &'a str {
        self.name
    }

    pub fn get_api(&'a self) -> &'a AfbApi {
        self.api
    }

    pub fn get_apiv4(&'a self) -> cglue::afb_api_t {
        self.api.get_apiv4()
    }

    pub fn get_handler(&'a self) -> &'a AfbEvtHandler {
        self.handler
    }
}

impl fmt::Display for AfbEventMsg<'_> {
    fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            let self_ref = &*(self as *const _ as *mut AfbEventMsg);
            let api_ref = &*(self_ref.get_api() as *const _ as *mut AfbApi);
            let handler_ref = &*(self_ref.get_handler() as *const _ as *mut AfbEvtHandler);

            let api_uid = api_ref.get_uid();
            let handler_uid = handler_ref.get_uid();

            write!(
                format,
                "{{uid:[{}], name:{} handler:{}, api:{:?}}}",
                self._uid, self.name, handler_uid, api_uid
            )
        }
    }
}

// Afb AfbEventMsg implementation
// ------------------------
#[no_mangle]
pub extern "C" fn api_events_cb(
    event_ctx: *mut std::ffi::c_void,
    evtname: *const std::os::raw::c_char,
    argc: u32,
    args: *const cglue::afb_data_t,
    apiv4: cglue::afb_api_t,
) {
    // extract event+api object from libafb internals
    let handler_ref = unsafe { &mut *(event_ctx as *mut AfbEvtHandler) };

    // extract api_ref from libafb
    let api_ref = unsafe {
        let api_data = cglue::afb_api_get_userdata(apiv4);
        &mut *(api_data as *mut AfbApi)
    };

    // build new request reference count object
    // fulup to be done RUST and libafb refcount should be aligned
    api_ref._count += 1;
    handler_ref._count += 1;
    let name = unsafe { CStr::from_ptr(evtname) }
        .to_str()
        .expect("hoops invalid internal event name");

    let uid = format!(
        "{}|{:04X}|{:04X}",
        api_ref._uid, api_ref._count, handler_ref._count
    );
    let event = AfbEventMsg::new(uid, name, api_ref, handler_ref);

    // move const **array in something Rust may understand
    let mut arguments = AfbRqtData::new(
        unsafe { std::slice::from_raw_parts(args as *const cglue::afb_data_t, argc as usize) },
        argc,
        0,
    );

    // call event calback
    let result = (handler_ref.callback)(&event, &mut arguments, &handler_ref.context);
    match result {
        Ok(()) => {}
        Err(error) => {
            let dbg = error.get_dbg();
            afb_log_raw!(
                Notice,
                apiv4,
                "{}:{} file: {}:{}:{}",
                handler_ref._uid,
                error,
                dbg.file,
                dbg.line,
                dbg.column
            );
        }
    }
}

pub type EvtCallback =
    fn(evt: &AfbEventMsg, args: &AfbRqtData, ctx: &AfbCtxData) -> Result<(), AfbError>;

#[track_caller]
fn evt_default_cb(
    evt: &AfbEventMsg,
    _args: &AfbRqtData,
    _ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    afb_error!(
        "afb-default-cb",
        "uid:{} no event callback defined",
        evt.get_uid()
    )
}

pub struct AfbEvtHandler {
    _uid: &'static str,
    _count: usize,
    verbosity: i32,
    pattern: &'static str,
    info: &'static str,
    callback: EvtCallback,
    context: AfbCtxData,
}

impl AfbEvtHandler {
    pub fn new(uid: &'static str) -> &'static mut Self {
        let event_box = Box::new(AfbEvtHandler {
            _uid: uid,
            _count: 0,
            verbosity: 0,
            pattern: uid,
            info: "",
            callback: evt_default_cb,
            context: AfbCtxData::new(AFB_NO_DATA),
        });
        Box::leak(event_box)
    }

    pub fn set_pattern(&mut self, value: &'static str) -> &mut Self {
        self.pattern = value;
        self
    }

    pub fn set_info(&mut self, value: &'static str) -> &mut Self {
        self.info = value;
        self
    }

    pub fn set_verbosity(&mut self, value: i32) -> &mut Self {
        self.verbosity = value;
        self
    }

    pub fn get_verbosity(&self) -> i32 {
        self.verbosity
    }

    pub fn set_callback(&mut self, callback: EvtCallback) -> &mut Self {
        self.callback = callback;
        self
    }

    pub fn set_context<T>(&mut self, ctx: T) -> &mut Self
    where
        T: 'static,
    {
        self.context = AfbCtxData::new(ctx);
        self
    }

    pub fn register(&mut self, apiv4: cglue::afb_api_t) -> i32 {
        let event_pattern = CString::new(self.pattern).expect("invalid event pattern");

        unsafe {
            cglue::afb_api_event_handler_add(
                apiv4,
                event_pattern.as_ptr(),
                Some(api_events_cb),
                self as *const _ as *mut std::ffi::c_void,
            )
        }
    }

    // return object getter trait to prevent any malicious modification
    pub fn finalize(&mut self) -> Result<&AfbEvtHandler, AfbError> {
        Ok(self)
    }
    pub fn get_uid(&self) -> &'static str {
        self._uid
    }
    pub fn get_pattern(&self) -> &'static str {
        self.pattern
    }
    pub fn get_info(&self) -> &'static str {
        self.info
    }
}

impl fmt::Display for AfbEvtHandler {
    fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            format,
            "uid:{} name:{} info:{}",
            self._uid, self.pattern, self.info
        )
    }
}

pub trait GetApiV4<T> {
    fn set_apiv4(api: T) -> AfbApiV4;
}

impl GetApiV4<AfbApiV4> for AfbEvent {
    fn set_apiv4(api: AfbApiV4) -> AfbApiV4 {
        api
    }
}

impl GetApiV4<&&mut AfbApi> for AfbEvent {
    fn set_apiv4(api: &&mut AfbApi) -> AfbApiV4 {
        (*api).get_apiv4()
    }
}

pub struct AfbEvent {
    _uid: &'static str,
    _evtv4: AfbEvtV4,
    _apiv4: AfbApiV4,
    verbosity: i32,
}

impl AfbEvent {
    pub fn new(uid: &'static str) -> &'static mut Self {
        let evt_box = Box::new(AfbEvent {
            _uid: uid,
            _evtv4: 0 as AfbEvtV4,
            _apiv4: 0 as AfbApiV4,
            verbosity: 0,
        });
        Box::leak(evt_box)
    }

    pub fn get_verbosity(&self) -> i32 {
        self.verbosity
    }

    pub fn register<T>(&mut self, api: T) -> i32
    where
        AfbEvent: GetApiV4<T>,
    {
        let apiv4 = Self::set_apiv4(api);
        let mut evt_id = 0 as AfbEvtV4;
        let evt_uid = CString::new(self._uid).unwrap();

        let status = unsafe { cglue::afb_api_new_event(apiv4, evt_uid.as_ptr(), &mut evt_id) };
        self._evtv4 = evt_id;
        self._apiv4 = apiv4;

        status
    }

    pub fn subscribe(&self, rqt: &AfbRequest) -> Result<&Self, AfbError> {
        if self._evtv4 == 0 as AfbEvtV4 {
            return afb_error!(self._uid, "should register before usage");
        }

        let status = unsafe { cglue::afb_req_subscribe(rqt.get_rqtv4(), self._evtv4) };
        if status != 0 {
            afb_error!(self._uid, "fail to subscribe event")
        } else {
            Ok(self)
        }
    }

    pub fn unsubscribe(&self, rqt: &AfbRequest) -> Result<&Self, AfbError> {
        if self._evtv4 == 0 as AfbEvtV4 {
            return afb_error!(self._uid, "should register before usage");
        }

        let status = unsafe { cglue::afb_req_unsubscribe(rqt.get_rqtv4(), self._evtv4) };
        if status != 0 {
            afb_error!(self._uid, "fail to unsubscribe event")
        } else {
            Ok(self)
        }
    }

    pub fn addref(&self) -> &Self {
        unsafe { cglue::afb_event_addref(self._evtv4) };
        self
    }

    pub fn unref(&self) -> &Self {
        unsafe { cglue::afb_event_unref(self._evtv4) };
        self
    }

    pub fn get_uid<'a>(&self) -> &'a str {
        self._uid
    }

    pub fn finalize(&self) -> Result<&Self, AfbError> {
        Ok(self)
    }

    pub fn get_apiv4(&self) -> AfbApiV4 {
        self._apiv4
    }

    #[track_caller]
    pub fn push<T>(&self, args: T) -> i32
    where
        AfbParams: ConvertResponse<T>,
    {
        if self._evtv4 == 0 as AfbEvtV4 {
            afb_log_msg!(
                Critical,
                None,
                format!(
                    "Not register event:{} should register before use",
                    self._uid
                )
            );
            return -1;
        }

        let response = AfbParams::convert(args);
        let params = match response {
            Err(error) => {
                afb_log_msg!(Critical, self.get_apiv4(), &error);
                return -1;
            }
            Ok(data) => data,
        };
        unsafe {
            cglue::afb_event_push(
                self._evtv4,
                params.arguments.len() as u32,
                params.arguments.as_slice().as_ptr(),
            )
        }
    }

    #[track_caller]
    pub fn broadcast<T>(&self, args: T) -> i32
    where
        AfbParams: ConvertResponse<T>,
    {
        if self._evtv4 == 0 as AfbEvtV4 {
            afb_log_msg!(
                Critical,
                self.get_apiv4(),
                format!(
                    "Not register event:{} should register before use",
                    self._uid
                )
            );
            return -1;
        }

        let response = AfbParams::convert(args);
        let params = match response {
            Err(error) => {
                afb_log_msg!(Critical, self.get_apiv4(), &error);
                return -1;
            }
            Ok(data) => data,
        };
        unsafe {
            cglue::afb_event_broadcast(
                self._evtv4,
                params.arguments.len() as u32,
                params.arguments.as_slice().as_ptr(),
            )
        }
    }
}

pub struct AfbGroup {
    _uid: &'static str,
    prefix: &'static str,
    info: &'static str,
    permission: &'static AfbPermission,
    verbosity: i32,
    separator: &'static str,
    verbs: Vec<*const AfbVerb>,
    events: Vec<*const AfbEvent>,
    evthandlers: Vec<*const AfbEvtHandler>,
}

impl AfbGroup {
    pub fn new(uid: &'static str) -> &'static mut Self {
        let group_box = Box::new(AfbGroup {
            _uid: uid,
            info: "",
            verbosity: 0,
            permission: AfbPermission::new(0),
            prefix: "",
            separator: "/",
            verbs: Vec::new(),
            evthandlers: Vec::new(),
            events: Vec::new(),
        });
        Box::leak(group_box)
    }

    pub fn set_prefix(&mut self, value: &'static str) -> &mut Self {
        self.prefix = value;
        self
    }

    pub fn set_permission(&mut self, value: &'static AfbPermission) -> &mut Self {
        self.permission = value;
        self
    }

    pub fn set_info(&mut self, value: &'static str) -> &mut Self {
        self.info = value;
        self
    }

    pub fn set_separator(&mut self, value: &'static str) -> &mut Self {
        self.separator = value;
        self
    }

    pub fn set_verbosity(&mut self, value: i32) -> &mut Self {
        self.verbosity = value;
        self
    }

    pub fn add_verb(&mut self, verb: &AfbVerb) -> &mut Self {
        self.verbs.push(verb);
        self
    }

    pub fn add_event(&mut self, event: &'static AfbEvent) -> &mut Self {
        self.events.push(event);
        self
    }

    pub fn add_evt_handler(&mut self, handler: &AfbEvtHandler) -> &mut Self {
        self.evthandlers.push(handler);
        self
    }

    pub fn register(&self, apiv4: cglue::afb_api_t, inherited_auth: *const AfbAuthV4) -> i32 {
        let mut status = 0;
        for slot in &self.verbs {
            let verb_ref = unsafe { &mut *(*slot as *mut AfbVerb) };

            // use group verbosity is higger than verb one
            if verb_ref.verbosity < 0 {
                verb_ref.verbosity = verb_ref.verbosity * -1;
            } else if self.verbosity > verb_ref.verbosity {
                verb_ref.verbosity = self.verbosity;
            }

            // add prefix to verb name and rebuild a static str string
            if self.prefix.len() > 0 {
                verb_ref.name = Box::leak(
                    (self.prefix.to_owned() + self.separator + verb_ref.name).into_boxed_str(),
                );
            }

            let group_permission: *mut AfbAuthV4 =
                AfbPermisionV4::new(self.permission, inherited_auth);

            //call verb registration method
            status = verb_ref.register(apiv4, group_permission);
            if status < 0 {
                afb_log_msg!(
                    Critical,
                    apiv4,
                    "Fail to register verb={}",
                    verb_ref.get_uid()
                );
                break;
            };
        }
        if status >= 0 {
            for slot in &self.evthandlers {
                let event_ref = unsafe { &mut *(*slot as *mut AfbEvtHandler) };
                status = event_ref.register(apiv4);
                if status < 0 {
                    afb_log_msg!(
                        Critical,
                        apiv4,
                        "Fail to register event_handler={}",
                        event_ref.get_uid()
                    );
                    break;
                };
            }
        }
        if status >= 0 {
            for slot in &self.events {
                let event_ref = unsafe { &mut *(*slot as *mut AfbEvent) };
                status = event_ref.register(apiv4);
                if status < 0 {
                    afb_log_msg!(
                        Critical,
                        apiv4,
                        "Fail to register event={}",
                        event_ref.get_uid()
                    );
                    break;
                };
            }
        }
        status
    }

    // return object getter trait to prevent any malicious modification
    pub fn finalize(&mut self) -> Result<&AfbGroup, AfbError> {
        Ok(self)
    }

    pub fn as_any(&self) -> &dyn Any {
        self
    }
    pub fn get_uid(&self) -> &'static str {
        self._uid
    }
    pub fn get_info(&self) -> &'static str {
        self.info
    }
}

#[no_mangle]
//fn afb_async_rqt_callback(userdata: *mut std::os::raw::c_void, status: i32, argc: u32, args: *mut cglue::afb_data_t, rqtv4: cglue::afb_req_t)
pub extern "C" fn afb_async_rqt_callback(
    userdata: *mut std::os::raw::c_void,
    status: i32,
    argc: u32,
    args: *const cglue::afb_data_t,
    rqtv4: cglue::afb_req_t,
) {
    // extract api_ref from libafb
    let api_ref = unsafe {
        let apiv4 = cglue::afb_req_get_api(rqtv4);
        let api_data = cglue::afb_api_get_userdata(apiv4);
        &mut *(api_data as *mut AfbApi)
    };

    // retreive source verb object
    let verb_ctx = unsafe { cglue::afb_req_get_vcbdata(rqtv4) };
    let verb_ref = unsafe { &mut *(verb_ctx as *mut AfbVerb) };

    // move const **array in something Rust may understand
    let arguments = AfbRqtData::new(
        unsafe { std::slice::from_raw_parts(args as *const cglue::afb_data_t, argc as usize) },
        argc,
        status,
    );
    // remap request on a valid Rust object
    let request = AfbRequest::new(rqtv4, api_ref, verb_ref);

    // extract verb+api object from libafb internals
    let subcall_ref = unsafe { &mut *(userdata as *mut AfbSubCall) };
    let result = (subcall_ref.rqt_cb.unwrap())(&request, &arguments, &subcall_ref.context);

    match result {
        Ok(()) => {}
        Err(error) => {
            let dbg = error.get_dbg();
            afb_log_raw!(
                Notice,
                &request,
                "{} file: {}:{}:{}",
                error,
                dbg.file,
                dbg.line,
                dbg.column
            );
            request.reply(error, -100);
        }
    }
}

#[no_mangle]
//fn afb_async_rqt_callback(userdata: *mut std::os::raw::c_void, status: i32, argc: u32, args: *mut cglue::afb_data_t, rqtv4: cglue::afb_req_t)
pub extern "C" fn afb_async_api_callback(
    userdata: *mut std::os::raw::c_void,
    status: i32,
    argc: u32,
    args: *const cglue::afb_data_t,
    apiv4: cglue::afb_api_t,
) {
    // extract api_ref from libafb
    let api_ref = unsafe {
        let api_data = cglue::afb_api_get_userdata(apiv4);
        &mut *(api_data as *mut AfbApi)
    };

    // move const **array in something Rust may understand
    let mut arguments = AfbRqtData::new(
        unsafe { std::slice::from_raw_parts(args as *const cglue::afb_data_t, argc as usize) },
        argc,
        status,
    );

    // extract verb+api object from libafb internals
    let subcall_ref = unsafe { &mut *(userdata as *mut AfbSubCall) };
    let result = (subcall_ref.api_cb.unwrap())(api_ref, &mut arguments, &subcall_ref.context);
    match result {
        Ok(()) => {}
        Err(error) => {
            let dbg = error.get_dbg();
            afb_log_raw!(
                Notice,
                apiv4,
                "{} file: {}:{}:{}",
                error,
                dbg.file,
                dbg.line,
                dbg.column
            );
        }
    }
}

pub fn afb_error_info(errcode: i32) -> &'static str {
    match errcode {
        0 => "Success",
        -9 => "Invalid Scope",
        -11 => "No Reply",
        -17 => "Api already exist",
        -62 => "Watchdog expire",
        -110 => "Connection timeout",
        -2 => "File exist",
        -3 => "Api not found",
        -4 => "Verb not found",
        -99 => "Invalid data type",
        -100 => "subcall application error",
        _ => "Unknown",
    }
}

pub trait DoSubcallAsync<H, K, C> {
    fn subcall_async(
        handle: H,
        apiname: CString,
        verbname: CString,
        params: &AfbParams,
        callback: K,
        context: C,
    );
}

pub trait DoSubcallSync<H> {
    fn subcall_sync(
        handle: H,
        apiname: CString,
        verbname: CString,
        params: &AfbParams,
    ) -> Result<AfbRqtData, AfbError>;
}

impl<C: 'static> DoSubcallAsync<&AfbApi, ApiCallback, C> for AfbSubCall {
    #[track_caller]
    fn subcall_async(
        api: &AfbApi,
        apiname: CString,
        verbname: CString,
        params: &AfbParams,
        callback: ApiCallback,
        context: C,
    ) {
        AfbSubCall::subcall_async(
            api.get_apiv4(),
            apiname,
            verbname,
            &params,
            callback,
            context,
        )
    }
}
impl DoSubcallSync<&AfbApi> for AfbSubCall {
    #[track_caller]
    fn subcall_sync(
        api: &AfbApi,
        apiname: CString,
        verbname: CString,
        params: &AfbParams,
    ) -> Result<AfbRqtData, AfbError> {
        AfbSubCall::subcall_sync(api.get_apiv4(), apiname, verbname, params)
    }
}

impl<C: 'static> DoSubcallAsync<AfbApiV4, ApiCallback, C> for AfbSubCall {
    #[track_caller]
    fn subcall_async(
        apiv4: AfbApiV4,
        apiname: CString,
        verbname: CString,
        params: &AfbParams,
        callback: ApiCallback,
        context: C,
    ) {
        let cbhandle = Box::into_raw(Box::new(AfbSubCall {
            api_cb: Some(callback),
            rqt_cb: None,
            context: AfbCtxData::new(context),
        }));

        unsafe {
            cglue::afb_api_call(
                apiv4,
                apiname.into_raw(),
                verbname.into_raw(),
                params.arguments.len() as u32,
                params.arguments.as_slice().as_ptr(),
                Some(afb_async_api_callback),
                cbhandle as *mut std::ffi::c_void,
            )
        };
    }
}

impl DoSubcallSync<AfbApiV4> for AfbSubCall {
    #[track_caller]
    fn subcall_sync(
        apiv4: AfbApiV4,
        apiname: CString,
        verbname: CString,
        params: &AfbParams,
    ) -> Result<AfbRqtData, AfbError> {
        let mut status = 0 as i32;
        let mut nreplies = MAX_CALL_ARGS;
        let replies = [0 as cglue::afb_data_t; MAX_CALL_ARGS as usize];

        let rc = unsafe {
            cglue::afb_api_call_sync(
                apiv4,
                apiname.into_raw(),
                verbname.into_raw(),
                params.arguments.len() as u32,
                params.arguments.as_slice().as_ptr(),
                &mut status,
                &mut nreplies,
                replies.as_ref() as *const _ as *mut cglue::afb_data_t,
            )
        };
        if rc < 0 || nreplies > MAX_CALL_ARGS || status < 0 {
            let replies = AfbRqtData::new(&replies, nreplies, status);
            let error = match replies.get::<JsoncObj>(0) {
                Ok(jerror) => jerror.to_string(),
                Err(_) => format!("status:{}({})", status, afb_error_info(status)),
            };
            return Err(AfbError::new("api-subcalls", status, error));
        }
        let datas = AfbRqtData::new(&replies, nreplies, status);
        Ok(datas)
    }
}

impl<'a, C: 'static> DoSubcallAsync<&AfbRequest, RqtCallback, C> for AfbSubCall {
    #[track_caller]
    fn subcall_async(
        rqt: &AfbRequest,
        apiname: CString,
        verbname: CString,
        params: &AfbParams,
        callback: RqtCallback,
        context: C,
    ) {
        AfbSubCall::subcall_async(
            (*rqt).get_rqtv4(),
            apiname,
            verbname,
            params,
            callback,
            context,
        )
    }
}

impl<'a> DoSubcallSync<&AfbRequest> for AfbSubCall {
    #[track_caller]
    fn subcall_sync(
        rqt: &AfbRequest,
        apiname: CString,
        verbname: CString,
        params: &AfbParams,
    ) -> Result<AfbRqtData, AfbError> {
        AfbSubCall::subcall_sync(rqt.get_rqtv4(), apiname, verbname, params)
    }
}

impl<C: 'static> DoSubcallAsync<AfbRqtV4, RqtCallback, C> for AfbSubCall {
    #[track_caller]
    fn subcall_async(
        rqtv4: AfbRqtV4,
        apiname: CString,
        verbname: CString,
        params: &AfbParams,
        callback: RqtCallback,
        context: C,
    ) {
        let cbhandle = Box::into_raw(Box::new(AfbSubCall {
            api_cb: None,
            rqt_cb: Some(callback),
            context: AfbCtxData::new(context),
        }));
        unsafe {
            cglue::afb_req_subcall(
                rqtv4,
                apiname.into_raw(),
                verbname.into_raw(),
                params.arguments.len() as u32,
                params.arguments.as_slice().as_ptr(),
                cglue::afb_req_subcall_flags_afb_req_subcall_catch_events as i32,
                Some(afb_async_rqt_callback),
                cbhandle as *mut std::ffi::c_void,
            )
        };
    }
}
impl DoSubcallSync<AfbRqtV4> for AfbSubCall {
    #[track_caller]
    fn subcall_sync(
        rqtv4: AfbRqtV4,
        apiname: CString,
        verbname: CString,
        params: &AfbParams,
    ) -> Result<AfbRqtData, AfbError> {
        let mut status = 0 as i32;
        let mut nreplies = MAX_CALL_ARGS;
        let replies = [0 as cglue::afb_data_t; MAX_CALL_ARGS as usize];

        let rc = unsafe {
            // err= afb_req_subcall_sync (glue->rqt.afb, apiname, verbname, (int)index, params, afb_req_subcall_catch_events, &status, &nreplies, replies);
            cglue::afb_req_subcall_sync(
                rqtv4,
                apiname.into_raw(),
                verbname.into_raw(),
                params.arguments.len() as u32,
                params.arguments.as_slice().as_ptr(),
                cglue::afb_req_subcall_flags_afb_req_subcall_catch_events as i32,
                &mut status,
                &mut nreplies,
                replies.as_ref() as *const _ as *mut cglue::afb_data_t,
            )
        };
        if rc < 0 || nreplies > MAX_CALL_ARGS || status < 0 {
            let replies = AfbRqtData::new(&replies, nreplies, status);
            let error = match replies.get::<JsoncObj>(0) {
                Ok(jerror) => jerror.to_string(),
                Err(_) => format!("status:{}({})", status, afb_error_info(status)),
            };
            return Err(AfbError::new("api-subcalls", status, error));
        }
        // move const **array in something Rust may understand
        let datas = AfbRqtData::new(&replies, nreplies, status);
        Ok(datas)
    }
}

pub struct AfbSubCall {
    context: AfbCtxData,
    api_cb: Option<ApiCallback>,
    rqt_cb: Option<RqtCallback>,
}

impl AfbSubCall {
    #[track_caller]
    pub fn call_sync<H, T>(
        handle: H,
        apiname: &str,
        verbname: &str,
        args: T,
    ) -> Result<AfbRqtData, AfbError>
    where
        AfbParams: ConvertResponse<T>,
        AfbSubCall: DoSubcallSync<H>,
    {
        let response = AfbParams::convert(args);
        let params = match response {
            Err(error) => {
                return Err(error);
            }
            Ok(data) => data,
        };

        let apistr = CString::new(apiname).expect("Invalid apiname");
        let verbstr = CString::new(verbname).expect("Invalid verbname");
        AfbSubCall::subcall_sync(handle, apistr, verbstr, &params)
    }

    #[track_caller]
    pub fn call_async<H, T, K, C>(
        handle: H,
        apiname: &str,
        verbname: &str,
        args: T,
        callback: K,
        context: C,
    ) -> Result<(), AfbError>
    where
        AfbParams: ConvertResponse<T>,
        AfbSubCall: DoSubcallAsync<H, K, C>,
    {
        let response = AfbParams::convert(args);
        let params = match response {
            Err(error) => {
                return Err(error);
            }
            Ok(data) => data,
        };

        let apistr = CString::new(apiname).expect("Invalid apiname");
        let verbstr = CString::new(verbname).expect("Invalid verbname");

        //let handle = 0 as *mut cglue::afb_api_x4;

        Ok(AfbSubCall::subcall_async(
            handle, apistr, verbstr, &params, callback, context,
        ))
    }
}
