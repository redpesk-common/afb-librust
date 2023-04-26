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
use std::ffi::{CStr, CString};
use std::fmt;
// libafb dependencies
use cglue as cglue;
use datav4::*;
use jsonc::*;
use utilv4::*;

// alias few external types
pub type AfbApiV4 = cglue::afb_api_t;
pub type AfbRqtV4 = cglue::afb_req_t;
pub type AfbEvtV4 = cglue::afb_event_t;
pub const NULLPTR: *mut std::ffi::c_void = 0 as *mut std::ffi::c_void;

// maximum argument return from call sync functions
const MAX_CALL_ARGS: u32 = 10;

pub trait AfbRqtControl {
    fn verb_callback(&mut self, request: &AfbRequest, args: &AfbData) -> Result<(),AfbError>;
}

pub trait AfbSubcallControl {
    fn api_callback(&mut self, api: &mut AfbApi, args: &AfbData);
}

pub use AfbBindingRegister;
/// Register binding main entry callback to be called from afb_binder
/// Examples
/// ```
/// # use libafb::prelude::*;;
/// AfbBindingRegister!(binding_init);
/// pub fn binding_init(binding: AfbApiV4, jconf: JsoncObj) -> i32 {
///    afb_log_msg!(Notice, binding, "-- binding-init binding config={}", jconf);
///    // register verb,event,apis, ...
///    AFB_OK // or AFB_FAIL to abort binding load
/// }
/// ```
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
            let jconf = libafb::apiv4::afb_binding_get_config(apiv4, ctlid, ctlarg, api_data);
            match $callback(apiv4, jconf) {
                Ok(api) => {
                    afb_log_msg!(Notice, apiv4, "RUST api uid={} started", api.get_uid());
                    AFB_OK
                }
                Err(error) => {
                    afb_log_msg!(Critical, apiv4, "Binding init fail error={}",error.to_string());
                    AFB_FAIL
                }
            }
        }
    };
}

pub use AfbSessionRegister;
#[macro_export]
macro_rules! AfbSessionRegister {
    ($userdata: ident) => {
        use crate::libafb::utilv4::MakeError;
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
                        None => Err(AfbError::make(
                            "session-any-cast",
                            "fail to restore <$userdata>",
                        )),
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
                        None => Err(AfbError::make(
                            "session-any-cast",
                            "fail to restore <$userdata>",
                        )),
                        Some(value) => Ok(value),
                    },
                }
            }

            fn drop(request: &AfbRequest) -> Result<(), libafb::utilv4::AfbError> {
                request.drop_session()
            }
        }
    };
}
pub use AfbVerbRegister;
/// Register verb control handle.
///   - $verb_name: created verb object class
///   - $callback: user define verb callback
///   - $userdata: an option data structure class attach the verb
/// Examples
/// ```
///   # extern crate jsonc;
///   # use libafb::prelude::*;;
///   struct OptionalData {
///     // my verb private data
///   }
///   AfbVerbRegister!(VerbCtrl, verb_callback, OptionalData);
///   fn verb_callback(request: &mut AfbRequest, _args: &mut AfbData, userdata: &mut OptionalData) {
///      request.reply("my verb callback was called", 0);
///   }
///   // verb control should be use to create a new verb
///   let my_verb = AfbVerb::new("verb-uid")
///        .set_callback(Box::new(VerbCtrl{ /* private data initialization */}))
///        .finalize();
///   // Warning: finally verb_handler need to be added to an api or a group with .add_verb
///   // .add_verb(my_verb)
/// ```
#[macro_export]
macro_rules! AfbVerbRegister {
    ($verb_name: ident, $callback:ident, $userdata:ident) => {
        #[allow(non_camel_case_types)]
        type $verb_name = $userdata;
        impl libafb::apiv4::AfbRqtControl for $userdata {
            fn verb_callback(
                &mut self,
                request: &libafb::apiv4::AfbRequest,
                args: &libafb::datav4::AfbData,
            ) -> Result<(), AfbError>{
                $callback(request, args, self)
            }
        }
    };
    ($verb_name: ident, $callback:ident) => {
        #[allow(non_camel_case_types)]
        struct $verb_name;
        impl libafb::apiv4::AfbRqtControl for $verb_name {
            fn verb_callback(
                &mut self,
                request: &libafb::apiv4::AfbRequest,
                args: &libafb::datav4::AfbData,
            ) -> Result<(), AfbError>{
                $callback(request, args)
            }
        }
    };
}

pub use AfbEventRegister;
/// Register event control handle. Similar to verb-ctrl but receive an event in place of a request
///   - $event_name: created verb object class
///   - $callback: user define verb callback
///   - $userdata: an option data structure class attach the verb
/// Examples
/// ```
///   # extern crate jsonc;
///   # use libafb::prelude::*;;
/// struct OptionalData {
///     // my private data
///     counter: u32,
/// }
/// AfbEventRegister!(EventCtrl, event_callback, OptionalData);
/// fn event_callback(event: &AfbEventMsg, args: &mut AfbData, userdata: &mut OptionalData) {
///     userdata.counter += 1;
///     afb_log_msg!(Notice,&event,"--evt name={} counter={} api={}", event.get_name(),userdata.counter,event.get_api().get_uid());
/// }
/// // event handler is use to create an event handler for a given pattern
/// let event_handler = AfbEvtHandler::new("my_handler_uid")
/// .set_pattern("helloworld-event/timerCount")
/// .set_callback(Box::new(EventCtrl{counter: 0}))
/// .finalize();
///
/// // Warning: finally event_handler need to be added to an api or a group with .add_evt_handler
/// // .add_evt_handler(event_handler)
/// ```
#[macro_export]
macro_rules! AfbEventRegister {
    ($event_name:ident, $callback:ident, $userdata:ident) => {
        #[allow(non_camel_case_types)]
        type $event_name = $userdata;
        impl libafb::apiv4::AfbEventControl for $userdata {
            fn event_callback(
                &mut self,
                event: &libafb::apiv4::AfbEventMsg,
                args: &libafb::datav4::AfbData,
            ) {
                $callback(event, args, self)
            }
        }
    };
    ($event_name: ident, $callback:ident) => {
        #[allow(non_camel_case_types)]
        struct $event_name;
        impl libafb::apiv4::AfbEventControl for $event_name {
            fn event_callback(
                &mut self,
                event: &libafb::apiv4::AfbEventMsg,
                args: &libafb::datav4::AfbData,
            ) {
                $callback(event, args)
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
        jverbs.insert(jverb).unwrap();
    }
    jgroup.add("verbs", jverbs).unwrap();
    jgroup
}

/// Api introspection callback for http://localhost:1234/devtools/
///
///  * internal callback responding to api/info request. RustAfb build info verb automatically
///  * to remove automatic introspection use ```api.add_info_cb(false)```
/// Examples
/// ```no_run
///   # extern crate jsonc;
///   # use libafb::prelude::*;;
///   let api= AfbApi::new("rust-api")
///     .set_name("rust-api")
///     .set_permission(AfbPermission::new("acl:rust"))
///     // .... add_verb, group, ...
///     .add_info_cb(false) // default=true
///     .finalize()
///     ;
/// ```
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
        .insert(add_verbs_to_group("", "", &mut api_ref.verbs))
        .unwrap();

    for slot in &api_ref.groups {
        let group_ref = unsafe { &mut *(*slot as *mut AfbGroup) };
        jgroups
            .insert(add_verbs_to_group(
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

/// Api introspection callback for my-api-uid/ping
///
///  * internal callback responding to api/ping request. RustAfb build info verb automatically
///  * to remove automatic introspection use ```api.add_ping_cb(false)```
/// Examples
/// ```no_run
///   # extern crate jsonc;
///   # use libafb::prelude::*;;
///   let api= AfbApi::new("rust-api")
///     .set_name("rust-api")
///     .set_permission(AfbPermission::new("acl:rust"))
///      // .... add verb, group, ...
///     .add_ping_cb(false) // default=true
///     .finalize()
///     ;
/// ```
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

/// AfbApiControls trait might optionally implemented API control callbacks.
pub trait AfbApiControls {
    /// Called at binding load time, before api is ready.
    ///
    /// When defined config method receives binding configuration as a json-c object.
    /// At this level the API is not ready, when having external dependencies this is
    /// as a database this is typically the place to declare them.
    /// Examples:
    /// ```
    /// # use libafb::prelude::*;;
    /// struct ApiUserData{}
    /// impl AfbApiControls for ApiUserData {
    ///   fn config(&mut self, api: &mut AfbApi, config: JsoncObj) -> Result<(),AfbError> {
    ///     let _api_data = self; // self matches api_data
    ///     afb_log_msg!(Notice,api,"--api-config api={} config={}",api.get_uid(),config);
    ///     Ok(()) // returning -1 will abort binder process
    ///   }
    ///   fn as_any(&mut self) -> &mut dyn Any {self}
    /// }
    /// ```
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

    /// Called at when API is ready.
    ///
    /// When defined start method is used to create event, timers, request subcall, ... or anything that require a working API
    /// This is also the place from where you may launch automatic test. Nevertheless note that as an API cannot call itself
    /// in loopback mode. For test it is mandatory to register a dedicated testing API within your binding before shaking your primary API.
    /// Example:
    /// ```
    /// # use libafb::prelude::*;;
    /// struct EvtUserData {
    ///   counter: u32,
    /// }
    /// AfbEventRegister!(EventCtrl, event_callback, EvtUserData);
    /// fn event_callback(event: &mut AfbEventMsg, args: &mut AfbData, userdata: &mut EvtUserData) {
    ///    afb_log_msg!(Notice,&event,"--callback evt={} name={} counter={}",event.get_uid(), event.get_name(),userdata.counter);
    /// };
    /// pub struct ApiUserData {
    ///   my_event: &'static AfbEvent,
    /// }
    /// impl AfbApiControls for ApiUserData {
    ///   fn start(&mut self, api: &mut AfbApi) ->  Result<(),AfbError> {
    ///     let api_data = self; // self matches api_data
    ///     let event_handler = AfbEvtHandler::new("handler-1")
    ///       .set_pattern("helloworld-event/timerCount")
    ///       .set_callback(Box::new(EventCtrl{counter: 0}))
    ///       .finalize();
    ///     // store event_handler into api_data
    ///     Ok(())
    ///   }
    ///   fn as_any(&mut self) -> &mut dyn Any {self}
    /// }
    /// ```
    fn start(&mut self, api: &AfbApi) -> Result<(), AfbError> {
        afb_log_msg!(Debug, api, "api init uid:{}", api._uid);
        Ok(())
    }

    #[doc(hidden)]
    fn ready(&mut self, api: &AfbApi) -> Result<(), AfbError> {
        afb_log_msg!(Debug, api, "api ready uid:{}", api._uid);
        Ok(())
    }

    /// Called at unattended event is received.
    ///
    /// Usually uses default trait implementation. Default prints a log message when ever an attended event it received.
    fn orphan(&mut self, api: &AfbApi, signal: &str) {
        afb_log_msg!(Info, api, "orphan event api:{} event: {}", api._uid, signal);
    }

    /// Called at binding exit.
    ///
    /// This method usually keep default trait implementation. It simply print a log message.
    fn exit(&mut self, api: &AfbApi, code: i32) -> i32 {
        afb_log_msg!(Debug, api, "api exit: uid:{} code:{}", api._uid, code);
        return code;
    }

    /// Mandatory for api userdata downcasting
    ///
    /// This method is fully generic. Nevertheless Rust does not allow its automatic implementation from trait default values.
    /// Example
    /// ```
    /// # use libafb::prelude::*;;
    /// pub struct ApiUserData {
    ///   my_event: &'static AfbEvent,
    ///   my_timer: &'static mut dyn AfbTimerRef,
    /// }
    /// // as_any is returns a full object with a variable size and not a pointer on Self.
    /// impl AfbApiControls for ApiUserData {
    ///   fn as_any(&mut self) -> &mut dyn Any {
    ///     self
    ///   }
    /// }
    /// AfbVerbRegister!(SubscribeCtrl, subscribe_callback);
    /// fn subscribe_callback(request: &mut AfbRequest, _args: &mut AfbData) {
    ///   let apidata = request
    ///     .get_apidata()
    ///     .downcast_ref::<ApiUserData>()
    ///     .expect("invalid api-data");
    ///   match apidata.my_event.subscribe(request) {
    ///     Err(error) => request.reply(afb_add_trace!(error), 405),
    ///     Ok(_event) => request.reply(AFB_NO_DATA, 0),
    ///   }
    /// }
    /// ```
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

/// Rust/C callback hidden from Rust developers
///
/// This function is called by libafb framework each time a API control event pops up.
/// It acts as a proxy between C and Rust. During Api pre_init phase it registers the API+verbs.
/// Then if custom api control callbacks exit it call them.
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
    let api_ref = unsafe { &mut *(apictx as *mut AfbApi) };

    let status = match ctlid {
        cglue::afb_ctlid_afb_ctlid_Pre_Init => {
            // reference native afb apiv4 within rust api object
            api_ref.set_apiv4(apiv4);
            let mut status = match api_ref.ctrlbox {
                Some(ctrlbox) => {
                    match unsafe { (*ctrlbox).config(api_ref, binding_parse_config(apiv4, ctlarg)) }
                    {
                        Err(error) => {
                            afb_log_msg!(Critical, apiv4, error.to_string());
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
                    unsafe { cglue::afb_api_require_api(apiv4, name.as_ptr(), 0) };
                }
            }

            if status >= 0 {
                for slot in &api_ref.require_classes {
                    let name = CString::new(*slot).expect("invalid api name");
                    unsafe { cglue::afb_api_require_class(apiv4, name.as_ptr()) };
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
                            api_ref._apiv4,
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
                            api_ref._apiv4,
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
                    status = event_ref.register(apiv4);
                    if status < 0 {
                        afb_log_msg!(
                            Critical,
                            api_ref._apiv4,
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
                    status = event_ref.register(apiv4);
                    if status < 0 {
                        afb_log_msg!(
                            Critical,
                            api_ref._apiv4,
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
                    afb_log_msg!(Critical, apiv4, error.to_string());
                    AFB_FAIL
                }
            },
            None => AFB_OK,
        },

        cglue::afb_ctlid_afb_ctlid_Class_Ready => match api_ref.ctrlbox {
            Some(ctrlbox) => match unsafe { (*ctrlbox).ready(api_ref) } {
                Ok(()) => AFB_OK,
                Err(error) => {
                    afb_log_msg!(Critical, apiv4, error.to_string());
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

/// Rust AfbAPi internal object structure
///
/// User should use setters before finalizing and then only getters
/// Examples
/// ```
/// # use libafb::prelude::*;;
/// AfbVerbRegister!(VerbCtrl, verb_callback);
/// fn verb_callback(request: &mut AfbRequest, _args: &mut AfbData) {
///    request.reply("my verb callback was called", 0);
/// }
/// let verb1= AfbVerb::new("verb1")
///    .set_callback(Box::new(VerbCtrl {}))
///    .set_permission(AfbPermission::new("acl:verb1"))
///    .finalize();
/// let verb2= AfbVerb::new("verb2")
///    .set_callback(Box::new(VerbCtrl {}))
///    .set_permission(AfbPermission::new("acl:verb2"))
///    .finalize();
/// let api= AfbApi::new("my-api")
///   .set_permission(AfbPermission::new("acl:rust"))
///   .add_verb(verb1)
///   .add_verb(verb2)
///   //.finalize()
///   ;
/// ```
pub struct AfbApi {
    _uid: &'static str,
    _count: usize,
    _apiv4: cglue::afb_api_t,
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
    /// create a new api handle
    ///
    /// - uid: static str
    ///
    /// note:
    ///  - when not overloaded by 'name' uid is used as api name.
    ///  - until not finalize the api is not created within libafm framework
    pub fn new(uid: &'static str) -> &'static mut Self {
        let api_box = Box::new(AfbApi {
            _uid: uid,
            _count: 0,
            _apiv4: 0 as cglue::afb_api_t,
            do_info: true,
            do_seal: true,
            do_ping: true,
            name: uid,
            info: "",
            class: "",
            version: "",
            verbosity: 0,
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

    /// overload api name
    ///
    /// - value: static str
    /// - when not overloaded 'uid' is used for api name.
    pub fn set_name(&mut self, value: &'static str) -> &mut Self {
        self.name = value;
        self
    }

    /// add information metadata to api
    ///
    /// - value: static str
    pub fn set_info(&mut self, value: &'static str) -> &mut Self {
        self.info = value;
        self
    }

    /// attach api to a given class
    ///
    /// Class it used to handle API dependencies. A given api may depend from a specific API
    /// or from a class API that group many api. For example your may depend on "redis" api
    /// or you may choose to depend on class i.e: database class that group redis,mysql,...
    /// - value: static str
    pub fn set_class(&mut self, value: &'static str) -> &mut Self {
        self.class = value;
        self
    }

    /// set if introspection 'info' api should be implemented or not.
    ///
    /// by default api/info introspection verb is automatically implemented
    ///  use ```.add_info_cb(false)``` to remove info introspection verb from api
    pub fn add_info_cb(&mut self, value: bool) -> &mut Self {
        self.do_info = value;
        self
    }

    /// set if monitoring 'ping' api should be implemented or not.
    ///
    /// by default api/ping monotoring verb is automatically implemented
    ///  use ```.add_info_cb(false)``` to remove ping monitoring verb from api
    pub fn add_ping_cb(&mut self, value: bool) -> &mut Self {
        self.do_ping = value;
        self
    }

    /// set if api is sealed at finalize time or not.
    ///
    /// by default api is sealed during finalization.
    /// use ```.seal(false)``` to keep api open after finalization
    pub fn seal(&mut self, value: bool) -> &mut Self {
        self.do_seal = value;
        self
    }

    /// Declare api version.
    ///
    /// Version metadata is used only for api/info introspection
    pub fn set_version(&mut self, value: &'static str) -> &mut Self {
        self.version = value;
        self
    }

    /// Add one or many permission to the API.
    ///
    /// Internally LibAfb permission apply only to verbs. In order to simplify developper live
    /// libafb-rust allow to define permission at API or even group level. Those API are added
    /// to verb permission with a logical 'and'. In following example 'acl:rust' permission is added to both 'verb_basic' and 'verb_typed'
    /// Examples
    /// ```no_run
    ///  # extern crate jsonc;
    ///  # use libafb::prelude::*;;
    ///  AfbVerbRegister!(VerbCtrl, verb_cb);
    ///  fn verb_cb(request: &mut AfbRequest, _args: &mut AfbData) {
    ///    request.reply ("verb callback called",0);
    ///  }
    ///  let verb= AfbVerb::new("my-verb")
    ///    .set_callback(Box::new(VerbCtrl{}))
    ///    .set_permission(AfbPermission::new("acl:user"))
    ///    .finalize();
    ///  let api= AfbApi::new("rust-api")
    ///   .set_name ("toto")
    ///   .set_info ("toto")
    ///   .set_permission(AfbPermission::new("acl:rust"))
    ///   .add_verb(verb)
    ///   .finalize();
    /// ```
    pub fn set_permission(&mut self, value: &'static AfbPermission) -> &mut Self {
        self.permission = value;
        self
    }

    /// Allow api verbs run concurently.
    ///
    /// Internally libafb is both asynchronous and multi-threaded. Threads pool size is set at afb_binder level.
    /// Nevertheless it is possible to force an API independly of afb-binder threads pool size.
    /// Default is 'true', except for debuging purpose Rust binding should keep concurency==true.
    pub fn set_concurrency(&mut self, value: bool) -> &mut Self {
        self.do_concurrency = value;
        self
    }

    /// Allow to force request verbosity
    ///
    /// Global verbosity is set at afb-binding level with --verbose option. Nevertheless it is possible
    /// to force verbosity per request. Libafb-rust support the verbosity overloading at api, group and verb level.
    /// When defined it global verbosity is lower then speficied verbosity, then request verbosity is overloaded.
    /// When negative, request verbosity is overloaded independantly of global verbosity level.
    pub fn set_verbosity(&mut self, value: i32) -> &mut Self {
        self.verbosity = value;
        self
    }

    /// Add a verb to API
    ///
    /// Verb should be defined with AfbVerb::new() before behing added to an API
    /// Examples
    /// ```
    ///  # extern crate jsonc;
    ///  # use libafb::prelude::*;;
    ///  AfbVerbRegister!(VerbCtrl, verb_cb);
    ///  fn verb_cb(request: &mut AfbRequest, _args: &mut AfbData) {
    ///    request.reply ("verb callback called",0);
    ///  }
    /// let verb= AfbVerb::new("my-verb")
    ///    .set_callback(Box::new(VerbCtrl {}))
    ///    .set_permission(AfbPermission::new("acl:user"))
    ///    .finalize();
    /// let api= AfbApi::new("rust-api")
    ///    .set_name("rust-api")
    ///    .set_info("My first Rust API")
    ///    .set_permission(AfbPermission::new("acl:rust"))
    ///    .add_verb(verb)
    ///    ;// ... .finalize();
    /// ```
    pub fn add_verb(&mut self, verb: &AfbVerb) -> &mut Self {
        self.verbs.push(verb);
        self
    }

    /// Add a event to API
    ///
    /// event should be defined with AfbEvent::new() before behing added to an API
    /// Examples
    /// ```no_run
    ///  # extern crate jsonc;
    ///  # use libafb::prelude::*;;
    /// // create event
    /// let event= AfbEvent::new("my-event");
    ///
    /// // register event in api
    /// let api= AfbApi::new("rust-api")
    ///    .add_event(event)
    ///    .finalize();
    ///
    /// // send event params
    /// event.push("params data(s)");
    /// ```
    pub fn add_event(&mut self, event: &'static AfbEvent) -> &mut Self {
        self.events.push(event);
        self
    }

    /// Add a group of verb to API
    ///
    /// Group should be defined with AfbGroup::new() before behing added to an API.
    /// Groups allow to set a common api prefix and permission to a given set of verbs
    /// Examples
    /// ```
    ///  # extern crate jsonc;
    ///  # use libafb::prelude::*;;
    ///  AfbVerbRegister!(VerbCtrl, verb_cb);
    ///  fn verb_cb(request: &mut AfbRequest, _args: &mut AfbData) {
    ///    request.reply ("verb callback called",0);
    ///  }
    /// let verb1= AfbVerb::new("verb1")
    ///    .set_callback(Box::new(VerbCtrl {}))
    ///    .set_permission(AfbPermission::new("acl:verb1"))
    ///    .finalize();
    /// let verb2= AfbVerb::new("verb2")
    ///    .set_callback(Box::new(VerbCtrl {}))
    ///    .set_permission(AfbPermission::new("acl:verb2"))
    ///    .finalize();
    /// let group=AfbGroup::new("my-group")
    ///    .set_prefix("group-prefix")
    ///    .set_permission(AfbPermission::new("acl:my-group"))
    ///    .add_verb(verb1)
    ///    .add_verb(verb2)
    ///    .finalize();
    /// let api= AfbApi::new("rust-api")
    ///    .set_name("rust-api")
    ///    .set_info("My first Rust API")
    ///    .set_permission(AfbPermission::new("acl:rust"))
    ///    .add_group(group)
    ///    ; // ... .finalize();
    /// ```
    pub fn add_group(&mut self, group: &AfbGroup) -> &mut Self {
        self.groups.push(group);
        self
    }

    /// Add a callback to event reception
    ///
    /// Event are attached to a given API, except for broadcasted evthandlers, a subcription
    /// is mandatory before receving them. Event callback handler are attached to a given
    /// event name/pattern. Note that they is no permission attached to event reception.
    /// If a session can subscribe to a given event, then it is automatically allowed to receive it.
    /// Examples:
    /// ```
    ///  # extern crate jsonc;
    ///  # use libafb::prelude::*;;
    ///  AfbEventRegister!(EventCtrl, event_get_callback);
    ///  fn event_get_callback(event: &mut AfbEventMsg, args: &mut AfbData) {
    ///     afb_log_msg!(Notice,&event,"--callback evt={} name={}",event.get_uid(), event.get_name());
    ///  }
    ///  let event_handler = AfbEvtHandler::new("handler-1")
    ///    .set_pattern("helloworld-event/timerCount")
    ///    .set_callback(Box::new(EventCtrl{}))
    ///    .finalize();
    ///  let api= AfbApi::new("rust-api")
    ///    .set_name("rust-api")
    ///    .add_evt_handler(event_handler)
    ///    ; // ... .finalize();
    /// ```
    pub fn add_evt_handler(&mut self, handler: &AfbEvtHandler) -> &mut Self {
        self.evthandlers.push(handler);
        self
    }

    /// Define a set of control callback for the api
    ///
    /// For every major evthandlers (init, start, stop) of API; when defined LibAfb activates user callback .
    /// Api control callback are defined within AfbApiControls trait. Outside of 'as_any' API control
    /// trait provides a default callback implementation for every API control. Check AfbApiControls trait
    /// for further information
    ///
    /// Examples:
    /// ```
    ///  # extern crate jsonc;
    ///  # use libafb::prelude::*;;
    ///  pub struct ApiUserData {
    ///     /* my api data event_handle, timer_handle, ... */
    ///  }
    ///  impl AfbApiControls for ApiUserData {
    ///    fn config(&mut self, api: &mut AfbApi, config: JsoncObj) -> i32 {
    ///        let _api_data = self; // self matches api_data
    ///        afb_log_msg!(Notice, api,"--api-config api={} config={}", api.get_uid(), config);
    ///        AFB_OK // returning -1 will abort binder process
    ///    }
    ///    fn start(&mut self, api: &mut AfbApi) -> i32 {
    ///        let api_data = self; // self matches api_data
    ///        // this is where you create event or subcall some other micro-service api
    ///        AFB_OK
    ///    }
    ///    fn as_any(&mut self) -> &mut dyn Any {self}
    ///  }
    ///  let api= AfbApi::new("rust-api")
    ///    .set_name("rust-api")
    ///    .set_callback(Box::new(ApiUserData{}))
    ///    ; // ... .finalize();
    /// ```
    pub fn set_callback(&mut self, ctrlbox: Box<dyn AfbApiControls>) -> &mut Self {
        self.ctrlbox = Some(Box::leak(ctrlbox));
        self
    }

    /// Require an other microservice api before starting
    ///
    /// In order to garantie a correct order of microservice start. Each Api may declare dependencies
    /// Dependencies a symply declare from API name. The fact they are local or remote is handle by afb-binder
    /// Note: that micro-service startup ordering may also use require_class dependencies.
    /// Examples:
    /// ```no_run
    ///  # extern crate jsonc;
    ///  # use libafb::prelude::*;;
    ///  let api= AfbApi::new("rust-api")
    ///    .set_name("rust-api")
    ///    .require_api("api-1")
    ///    .require_api("api-2")
    ///    .finalize();
    /// ```
    pub fn require_api(&mut self, value: &'static str) -> &mut Self {
        self.require_apis.push(value);
        self
    }

    /// Require an other microservice api class before starting
    ///
    /// Api class are typically used to express dependencies on a generic micro-service.
    /// Typically: database, audio, canbus, ... any time you have common api with different
    /// low level implementation, class is a good candidate to order microservice starting order.
    /// Examples:
    /// ```no_run
    ///  # extern crate jsonc;
    ///  # use libafb::prelude::*;;
    ///  let api= AfbApi::new("rust-api")
    ///    .set_name("rust-api")
    ///    .require_class("audio")
    ///    .require_class("canbus")
    ///    .finalize();
    /// ```
    pub fn require_class(&mut self, value: &'static str) -> &mut Self {
        self.require_classes.push(value);
        self
    }

    #[doc(hidden)]
    // hack to update apiv4 after api object creation
    pub fn set_apiv4(&self, apiv4: cglue::afb_api_t) {
        let api_ref = unsafe { &mut *(self as *const _ as *mut AfbApi) };
        api_ref._apiv4 = apiv4;
    }

    /// Finalize an API and effectivly register API withing C/LibAFB framework
    ///
    /// Before finalization the API does not exist within C/LibAfb framework.
    /// This fonction do call the cglue::afb_create_api. Note that if the API
    /// if empty by default the framework will create an API with pîng verb.
    /// Examples:
    /// ```no_run
    ///  # extern crate jsonc;
    ///  # use libafb::prelude::*;;
    ///  let api= AfbApi::new("rust-api").finalize();
    /// ```
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
            let error = AfbError::new(
                self._uid,
                format!(
                    "Fail to register api uid={} status={} info={} ",
                    self._uid,
                    status,
                    afb_error_info(status)
                ),
            );
            Err(error)
        } else {
            Ok(self)
        }
    }

    pub fn as_mut(&self) -> &mut Self {
        unsafe { &mut *(self as *const _ as *mut AfbApi) }
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
        self._apiv4
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
    /// afb api simple printing output
    /// format {} => print uid,name,info
    /// format {:#name} => print only name
    /// Examples
    /// ```text
    /// println!("api={}", api);
    /// ```
    fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            format,
            "uid:{} name:{} info:{}",
            self._uid, self.name, self.info
        )
    }
}

/// Rust/C callback hidden from Rust developper
///
/// This function is call by libafb framework each time a registered rust API/VERB is requested.
/// It acts as a proxy between C and Rust and prepare Rust context before calling user defined callback.
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
    let mut arguments = AfbData::new(
        unsafe { std::slice::from_raw_parts(args as *const cglue::afb_data_t, argc as usize) },
        argc,
        0,
    );

    let mut request = AfbRequest::new(rqtv4, api_ref, verb_ref);

    if verb_ref.verbosity > 0 {
        unsafe { cglue::afb_req_wants_log_level(rqtv4, verb_ref.verbosity) };
    }

    // call verb calback
    let result= match verb_ref.ctrlbox {
        Some(verb_control) => unsafe {
            (*verb_control).verb_callback(&mut request, &mut arguments)
        },
        _ => panic!("verb={} no callback defined", verb_ref._uid),
    };

    match result {
        Ok(()) => {},
        Err(error) => request.reply(error, -100),
    }
}

/// Rust AfbVerb internal object structure
///
/// User should use setters before finalizing and then only getters
/// Examples
/// ```no_run
/// # use libafb::prelude::*;;
/// let verb1= AfbVerb::new("verb1")
///    .set_info("my first rust verb")
///    .set_callback(Box::new(VerbCtrl {}))
///    .set_permission(AfbPermission::new("acl:verb1"))
///    .finalize();
/// ```
pub struct AfbVerb {
    _uid: &'static str,
    _count: usize,
    ctrlbox: Option<*mut dyn AfbRqtControl>,
    name: &'static str,
    info: &'static str,
    permission: &'static AfbPermission,
    verbosity: i32,
    usage: Option<&'static str>,
    samples: JsoncObj,
    actions: JsoncObj,
}

impl AfbVerb {
    /// create a new verb handle
    ///
    /// - uid: static str
    ///
    /// note:
    ///  - when not overloaded by 'name' uid is used as verb name.
    ///  - verb are register automatically at api.finalyse() time
    ///    if API is not frozen(default) verb manual registration
    ///    remains posible with .register() method.
    pub fn new(uid: &'static str) -> &'static mut Self {
        let verb_box = Box::new(AfbVerb {
            _uid: uid,
            _count: 0,
            ctrlbox: None,
            name: uid,
            info: "",
            verbosity: 0,
            permission: AfbPermission::new(0),
            usage: None,
            samples: JsoncObj::array(),
            actions: JsoncObj::array(),
        });
        Box::leak(verb_box)
    }
    /// overload verb name
    ///
    /// - value: static str
    /// - when not overloaded 'uid' is used for api name.
    pub fn set_name(&mut self, value: &'static str) -> &mut Self {
        self.name = value;
        self
    }

    /// add information metadata to verb
    ///
    /// - value: static str
    pub fn set_info(&mut self, value: &'static str) -> &mut Self {
        self.info = value;
        self
    }

    /// Add one or many permission to the verb.
    ///
    /// This method should be called only once per verb. If parent permissions are defined at API or group level
    /// they are added. Final verb permission is ```AfbAuthAnyOf!(ApiPerm, GroupPerm, VerbPerm)```
    /// Examples
    /// ```no_run
    ///  # extern crate jsonc;
    ///  # use libafb::prelude::*;;
    ///  AfbVerbRegister!(VerbCtrl, verb_cb);
    ///  fn verb_cb(request: &mut AfbRequest, _args: &mut AfbData) {
    ///    request.reply ("verb callback called",0);
    ///  }
    ///  let verb= AfbVerb::new("my-verb")
    ///    .set_callback(Box::new(VerbCtrl{}))
    ///    .set_permission(AfbPermission::new("acl:user"))
    ///    .finalize();
    /// ```
    pub fn set_permission(&mut self, value: &'static AfbPermission) -> &mut Self {
        self.permission = value;
        self
    }

    /// Add metadata information for debugging tool.
    ///
    /// String place here will appear within input area of web debug tool
    pub fn set_usage(&mut self, value: &'static str) -> &mut Self {
        self.usage = Some(value);
        self
    }

    /// Add metadata information for debugging tool
    ///
    /// Example should be provided as json string. This method may be called multiple time when manu example are require
    /// This method return an AfbError in case of wrong json data.
    /// Examples
    /// ```no_run
    /// AfbVerb::new(mod_name)
    ///    .set_callback(Box::new(VerbCtrl {}))
    ///    .set_usage("{skipail': 'String', 'location':'string', 'zip': Integer}")
    ///    .set_sample("{'skipail': 'IoT.bzh', 'location':'Lorient', 'zip':56100}").expect("invalid json sample")
    ///    .set_sample("{'skipail': 'IoT.bzh', 'info':'missing location+zip'}").expect("invalid json sample")
    ///    .finalize()
    /// ```
    pub fn set_sample(&mut self, value: &'static str) -> Result<&mut Self, AfbError> {
        let jparse = JsoncObj::parse(value);
        match jparse {
            Err(_error) => Err(AfbError::new("jsonc-parsing-error", value.to_string())),
            Ok(jvalue) => {
                self.samples.insert(jvalue).unwrap();
                Ok(self)
            }
        }
    }

    /// Add metadata information for debugging tool
    ///
    /// Example should be provided as json string. This method may be called multiple time when manu example are require
    /// This method return an AfbError in case of wrong json data.
    /// Examples
    /// ```no_run
    /// AfbVerb::new(mod_name)
    ///    .set_callback(Box::new(VerbCtrl {}))
    ///    .set_action("['reset': 'subscribe', 'unsubscribe']").expect("a valid json array")
    ///    .finalize()
    /// ```
    pub fn set_action(&mut self, value: &'static str) -> Result<&mut Self, AfbError> {
        let jparse = JsoncObj::parse(value);
        match jparse {
            Err(error) => Err(error),
            Ok(jvalue) => {
                if jvalue.is_type(Jtype::Array) {
                    self.actions = jvalue;
                    Ok(self)
                } else {
                    Err(AfbError::new("verb-set-action","not a valid json array"))
                }
            }
        }
    }

    /// Allow to force request verbosity
    ///
    /// Global verbosity is set at afb-binding level with --verbose option. Nevertheless it is possible
    /// to force verbosity per request. Libafb-rust support the verbosity overloading at api, group and verb level.
    /// When defined it global verbosity is lower then speficied verbosity, then request verbosity is overloaded.
    /// When negative, request verbosity is overloaded independantly of global verbosity level.
    pub fn set_verbosity(&mut self, value: i32) -> &mut Self {
        self.verbosity = value;
        self
    }

    /// Define a set of control callback for the verb
    ///
    /// Verb MUST have a callback, if not defined verb registration will fail. Technically callback is implemented
    /// as AfbRqtControl trait. Nevertheless ```AfbVerbRegister!()``` is the magic want that hide this backmagic.
    /// Optionnaly callback may have a vcbdata. Note that even when multiple verbs share the same vcbdata type
    /// the instance of it remains private to each individual verb.
    /// Examples:
    /// ```
    /// AfbVerbRegister!(VerbCtrl, callback);
    /// fn callback(request: &mut AfbRequest, args: &mut AfbData) {
    ///     match args.get::<JsoncObj>(0) {
    ///         Ok(argument) => {
    ///             afb_log_msg!(Info,&request,"Got valid jsonc object argument={}",argument);
    ///             request.reply("done", 0);
    ///         },
    ///         Err(error) => {
    ///             afb_log_msg!(Error, &request, "hoop invalid json argument {}", error);
    ///             JsoncObj::from("invalid json input argument");
    ///             request.reply(afb_add_trace!(error), 405);
    ///         };
    ///     };
    /// };
    ///
    /// AfbVerb::new("my-verb")
    ///   .set_callback(Box::new(VerbCtrl{}))
    ///   .finalize()
    /// ```
    pub fn set_callback(&mut self, ctrlbox: Box<dyn AfbRqtControl>) -> &mut Self {
        self.ctrlbox = Some(Box::leak(ctrlbox));
        self
    }

    /// Manually register a verb within an API.
    ///
    /// This method is called automatically at API finalization and user should normally not use it.
    ///  * apiv4: is the internal libafb/C apihandle
    ///  * inherited_auth: is the internal libafb/C permission handle as inherited from Api and/or group.
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

    /// Freeze VERB definition and register it within libafb framework
    ///
    /// After using this method VERB object is not modifiable anymore and can only
    /// be requested through AfbVerbRef getter. Verb definition should be freeze before
    /// being added to an API or a group.
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
    /// verb simple printing output
    /// format {} => print uid,name,info
    /// Examples
    /// ```no_run
    /// println!("verb={}", verb);
    /// ```
    fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            format,
            "uid:{} name:{} info:{}",
            self._uid, self.name, self.info
        )
    }
}

/// AfbRequest is created automatically each time an API/Verb is invoqued
/// its main perpose is to carry internal request as well as a pointer to corresponding Verb+Api.
pub struct AfbRequest<'a> {
    _rqtv4: cglue::afb_req_t,
    api: &'a AfbApi,
    verb: &'a AfbVerb,
}

// Rust dynamic object are fat pointer and should be encapculated before passing to C
struct AfbRqtSessionWrap {
    inner: Box<dyn AfbRqtSession>,
}
pub trait AfbRqtSession {
    //fn new(request: AfbRequest, session: Self) -> Result<&mut Self, AfbError> ;
    //fn get(request: AfbRequest) -> Result<&mut Self, AfbError>;
    // fn get(&self) ->  Result<&mut dyn Any, AfbError>;
    fn as_any(&mut self) -> &mut dyn Any;
}

impl<'a> AfbRequest<'a> {
    /// new request is only created internal by the framework. User should never call this function.
    pub fn new(rqtv4: cglue::afb_req_t, api: &'a AfbApi, verb: &'a AfbVerb) -> Self {
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
                Some(free_box_cb),
                session as *const _ as *mut ::std::os::raw::c_void,
            )
        };
        if status < 0 {
            Err(AfbError::make(
                "rqt-session-exist",
                "request fail to create session",
            ))
        } else {
            Ok(session.inner.as_mut())
        }
    }

    pub fn drop_session(&self) -> Result<(), AfbError> {
        let status = unsafe { cglue::afb_req_context_drop(self.get_rqtv4()) };
        if status < 0 {
            Err(AfbError::make(
                "rqt-session-missing",
                "request session not defined",
            ))
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
            Err(AfbError::make(
                "rqt-session-exist",
                "request session already defined",
            ))
        } else {
            let session = unsafe { &mut *(session as *mut AfbRqtSessionWrap) };
            Ok(session.inner.as_mut())
        }
    }

    /// internal function that allow to transfert Rust request object livecycle to libafb/C framework
    /// it should never be called by user.
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

    /// Debug function return the hexa value of request pointer.
    ///
    /// It is moslty usefull to track requests
    /// during debugging session, when multiple asynchronous call are inbricated.
    pub fn get_uid(&'a self) -> String {
        format!("rqt:{:p}", self)
    }

    /// Return a reference on request corresponding verb handle
    ///
    /// Verb handle may then be use to retreive its uid and/or its vcbdata
    pub fn get_verb(&'a self) -> &'a AfbVerb {
        self.verb
    }

    /// Return a reference on request corresponding API
    ///
    /// Api handle may be use to retreive api name or api userdata.
    pub fn get_api(&'a self) -> &'a AfbApi {
        self.api
    }

    /// Return internal libAfb/C request handle
    ///
    /// Normally reserve to internal usage.
    pub fn get_rqtv4(&'a self) -> cglue::afb_req_t {
        self._rqtv4
    }

    /// Return api userdata as dyn Any
    ///
    /// Return a dyn Any pointer compatible with downcast. This is mandatory to remap
    /// api userdata without using unsafe code.
    /// Example
    /// ```no_run
    ///   # extern crate jsonc;
    ///   # use libafb::prelude::*;;
    /// let apidata = request
    ///     .get_apidata()
    ///     .downcast_mut::<ApiUserData>()
    ///     .expect("invalid api-data");
    ///     match apidata.my_event.subscribe(request) {
    ///        Err(_error) => {},
    ///        Ok(event) => {event.push("delay response should arrive in 3s");},
    ///     }
    /// ```
    pub fn get_apidata(&self) -> &mut dyn Any {
        self.get_api().getctrlbox().as_any()
    }

    /// Set LOA (Level Of Assurance) for active session
    ///
    /// LOA typically represent the level of trust we have in current session. It is typically used
    /// to check an level of autotication; but it may also be used to force an api order as Open before read.
    /// LOA level is typically express from 0=anonymous to 7=ultra-trust. But number is free until u32::max/2
    /// Examples
    /// ```no_run
    ///   # extern crate jsonc;
    ///   # use libafb::prelude::*;;
    /// fn set_loa_cb(request: &AfbRequest, _args: &AfbData) {
    /// match request.set_loa(1) {
    ///    Err(error) => request.reply (afb_add_trace!(error), -1),
    ///    Ok(loa) => request.reply(format!("LOA set to {}", loa), 0)
    ///  }
    ///}
    /// ```
    pub fn set_loa(&self, loa: u32) -> Result<u32, AfbError> {
        let status = unsafe { cglue::afb_req_session_set_LOA(self._rqtv4, loa) };
        if status < 0 {
            Err(AfbError::new(
                &self.get_uid(),
                format!(
                    "invalid LOA={} api={} verb={}",
                    loa,
                    self.get_api().get_uid(),
                    self.get_verb().get_uid()
                ),
            ))
        } else {
            Ok(loa)
        }
    }

    /// Return a jsonc object representing session/client metadata
    ///
    /// Rust version of [afb_req_get_client_info](https://docs.redpesk.bzh/docs/en/master/developer-guides/reference-v4/func-afb-req.html#function-afb_req_get_client_info)
    pub fn get_client_info(&self) -> JsoncObj {
        let jso = unsafe { cglue::afb_req_get_client_info(self._rqtv4) as *mut std::ffi::c_void };
        JsoncObj::from(jso)
    }

    /// Increment request reference count
    ///
    /// By default request is deleted when verb callback returns. When using asynchronous function request should leave longer than API initial callback
    /// Examples
    /// ```no_run
    ///   # extern crate jsonc;
    ///   # use libafb::prelude::*;;
    ///     match AfbSchedJob::new("demo-job-post-verb-cb")
    ///        .set_exec_watchdog(10) // limit exec time to 10s;
    ///        .set_callback(Box::new(UserPostData {
    ///            rqt: request.add_ref(),
    ///            jsonc: jquery.clone(),
    ///        }))
    ///        .post(3000)
    ///    {
    ///        // exec job in ~3s
    ///        Err(error) => {request.reply(afb_add_trace!(error), -1);},
    ///        Ok(job) => {afb_log_msg!(Info, request, "Job posted uid:{} jobid={}", job.get_uid(), job.get_jobid());},
    ///    }
    /// ```
    pub fn add_ref(&self) -> AfbRqtV4 {
        unsafe {
            cglue::afb_req_addref(self._rqtv4);
        }
        self._rqtv4
    }

    /// Reply to an API call.
    ///
    /// Reply is generally done at the end of an api/verb callback. nevertheless when using add_ref it is possible to delay the response
    /// Reply takes two parameters,
    ///  * data (AfbData)
    ///  * status (i32)
    /// In order to make developper live easier request.reply provde polymorphisme that access any afb-builtin type as well as the one
    /// register with AfbDataConverter!
    /// Example:
    /// ```no_run
    ///   # extern crate jsonc;
    ///   # use libafb::prelude::*;;
    ///   let reply = || -> Result<(), AfbError> {
    ///        let mut response = AfbParams::new();
    ///        response.push(JsoncObj::parse("{'label':'value'}"))?;
    ///        response.push(1234)?;
    ///        response.push(45.76)?;
    ///        response.push("skipail IoT.bzh")?;
    ///        request.reply(response, 0);
    ///        Ok(())
    ///    };
    /// ```
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
impl<'a> Drop for AfbRequest<'a> {
    /// decrease libafb request reference count anf free uid
    fn drop(&mut self) {
        unsafe {
            cglue::afb_req_unref(self._rqtv4);
        }
    }
}

impl<'a> fmt::Display for AfbRequest<'a> {
    /// afb simple printing output
    /// format {} => print uid,api,verb
    /// Examples
    /// ```compile_fail
    /// println!("request={}", request);
    /// ```
    fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            let self_ref = &mut *(self as *const _ as *mut AfbRequest);
            let api_ref = &mut *(self_ref.get_api() as *const _ as *mut AfbApi);
            let verb_ref = &mut *(self_ref.get_verb() as *const _ as *mut AfbVerb);

            let api_uid = api_ref.get_uid();
            let verb_uid = verb_ref.get_uid();

            write!(format, "api:'{}', verb:'{}'}}", api_uid, verb_uid,)
        }
    }
}

/// Event object as receive by its handler when suscribed
///
/// As one handler may receive multiple event with pattern match. The UID and NAME are no equal.
pub struct AfbEventMsg<'a> {
    _uid: String,
    name: &'a str,
    api: &'a AfbApi,
    handler: &'a AfbEvtHandler,
}

impl<'a> AfbEventMsg<'a> {
    /// create a new event handler
    ///
    ///
    pub fn new(uid: String, name: &'a str, api: &'a AfbApi, handler: &'a AfbEvtHandler) -> Self {
        AfbEventMsg {
            _uid: uid,
            api: api,
            name: name,
            handler: handler,
        }
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
    /// event simple printing output
    /// format {} => print uid,name,info
    /// Examples
    /// ```
    /// println!("event={}", event);
    /// ```
    fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            let self_ref = &mut *(self as *const _ as *mut AfbEventMsg);
            let api_ref = &mut *(self_ref.get_api() as *const _ as *mut AfbApi);
            let handler_ref = &mut *(self_ref.get_handler() as *const _ as *mut AfbEvtHandler);

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
    let mut event = AfbEventMsg::new(uid, name, api_ref, handler_ref);

    // move const **array in something Rust may understand
    let mut arguments = AfbData::new(
        unsafe { std::slice::from_raw_parts(args as *const cglue::afb_data_t, argc as usize) },
        argc,
        0,
    );

    // call event calback
    match handler_ref.ctrlbox {
        Some(event_control) => unsafe {
            (*event_control).event_callback(&mut event, &mut arguments)
        },
        _ => panic!("event={} no callback defined", handler_ref._uid),
    }
}

pub trait AfbEventControl {
    fn event_callback(&mut self, event: &AfbEventMsg, args: &AfbData);
}

pub struct AfbEvtHandler {
    _uid: &'static str,
    _count: usize,
    ctrlbox: Option<*mut dyn AfbEventControl>,
    pattern: &'static str,
    info: &'static str,
}

impl AfbEvtHandler {
    pub fn new(uid: &'static str) -> &'static mut Self {
        let event_box = Box::new(AfbEvtHandler {
            _uid: uid,
            _count: 0,
            ctrlbox: None,
            pattern: uid,
            info: "",
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

    pub fn set_callback(&mut self, ctrlbox: Box<dyn AfbEventControl>) -> &mut Self {
        self.ctrlbox = Some(Box::leak(ctrlbox));
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
    /// event simple printing output
    /// format {} => print uid,name,info
    /// Examples
    /// ```compile_fail
    /// println!("event={}", event);
    /// ```
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
}

impl AfbEvent {
    pub fn new(uid: &'static str) -> &'static mut Self {
        let evt_box = Box::new(AfbEvent {
            _uid: uid,
            _evtv4: 0 as AfbEvtV4,
            _apiv4: 0 as AfbApiV4,
        });
        Box::leak(evt_box)
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
            return Err(AfbError::new(self._uid, "should register before usage"));
        }

        let status = unsafe { cglue::afb_req_subscribe(rqt.get_rqtv4(), self._evtv4) };
        if status != 0 {
            Err(AfbError::new(self._uid, "fail to subscribe event"))
        } else {
            Ok(self)
        }
    }

    pub fn unsubscribe(&self, rqt: &AfbRequest) -> Result<&Self, AfbError> {
        if self._evtv4 == 0 as AfbEvtV4 {
            return Err(AfbError::new(self._uid, "should register before usage"));
        }

        let status = unsafe { cglue::afb_req_unsubscribe(rqt.get_rqtv4(), self._evtv4) };
        if status != 0 {
            Err(AfbError::new(self._uid, "fail to subscribe event"))
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

    pub fn push<T>(&self, args: T) -> i32
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
            cglue::afb_event_push(
                self._evtv4,
                params.arguments.len() as u32,
                params.arguments.as_slice().as_ptr(),
            )
        }
    }

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

    pub fn set_prefix(&'static mut self, value: &'static str) -> &'static mut Self {
        self.prefix = value;
        self
    }

    pub fn set_permission(&'static mut self, value: &'static AfbPermission) -> &'static mut Self {
        self.permission = value;
        self
    }

    pub fn set_info(&'static mut self, value: &'static str) -> &'static mut Self {
        self.info = value;
        self
    }

    pub fn set_separator(&'static mut self, value: &'static str) -> &'static mut Self {
        self.separator = value;
        self
    }

    pub fn set_verbosity(&'static mut self, value: i32) -> &'static mut Self {
        self.verbosity = value;
        self
    }

    pub fn add_verb(&'static mut self, verb: &AfbVerb) -> &'static mut Self {
        self.verbs.push(verb);
        self
    }

    pub fn add_event(&'static mut self, event: &'static AfbEvent) -> &'static mut Self {
        self.events.push(event);
        self
    }

    pub fn add_evt_handler(&'static mut self, handler: &AfbEvtHandler) -> &'static mut Self {
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
    pub fn finalize(&'static mut self) -> Result<&'static AfbGroup, AfbError> {
        Ok(self)
    }

    pub fn as_mut(&self) -> &mut Self {
        unsafe { &mut *(self as *const _ as *mut AfbGroup) }
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

pub struct AfbSubCall {
    verb_cb: Option<*mut dyn AfbRqtControl>,
    api_cb: Option<*mut dyn AfbSubcallControl>,
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
    let mut arguments = AfbData::new(
        unsafe { std::slice::from_raw_parts(args as *const cglue::afb_data_t, argc as usize) },
        argc,
        status,
    );
    // remap request on a valid Rust object
    let mut request = AfbRequest::new(rqtv4, api_ref, verb_ref);

    // extract verb+api object from libafb internals
    let subcall_ref = unsafe { &mut *(userdata as *mut AfbSubCall) };
    match subcall_ref.verb_cb {
        Some(callback) => {
            match unsafe { (*callback).verb_callback(&mut request, &mut arguments) } {
                Ok(()) => {},
                Err(error) => request.reply(error,101),
            }
        },
        _ => {
            afb_log_msg!(Critical, rqtv4, "(hoops invalid RQT callback context");
        }
    };

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
    let mut arguments = AfbData::new(
        unsafe { std::slice::from_raw_parts(args as *const cglue::afb_data_t, argc as usize) },
        argc,
        status,
    );

    // extract verb+api object from libafb internals
    let subcall_ref = unsafe { &mut *(userdata as *mut AfbSubCall) };
    match subcall_ref.api_cb {
        Some(callback) => unsafe { (*callback).api_callback(api_ref, &mut arguments) },
        _ => {
            afb_log_msg!(Critical, apiv4, "(hoops invalid RQT callback context");
        }
    };
}

pub fn afb_error_info(errcode: i32) -> &'static str {
    match errcode {
        0 => "Success",
        -9 => "Invalid Scope",
        -17 => "Api already exist",
        -62 => "Watchdog expire",
        -110 => "Connection timeout",
        -2 => "File exist",
        -3 => "Api not found",
        -4 => "Verb not found",
        -99 => "Invalid data type",
        _ => "Unknown",
    }
}

pub trait DoSubcall<H, C> {
    fn subcall_async(handle: H, apiname: &str, verbname: &str, params: &AfbParams, callback: C);

    fn subcall_sync(
        handle: H,
        apiname: &str,
        verbname: &str,
        params: &AfbParams,
    ) -> Result<AfbData, AfbError>;
}

impl DoSubcall<&AfbApi, Box<dyn AfbSubcallControl>> for AfbSubCall {
    fn subcall_async(
        api: &AfbApi,
        apiname: &str,
        verbname: &str,
        params: &AfbParams,
        callback: Box<dyn AfbSubcallControl>,
    ) {
        let apistr = CString::new(apiname).expect("Invalid apiname");
        let verbstr = CString::new(verbname).expect("Invalid verbname");

        // lock callback box into memory until AFB returns from subcall
        let cbbox = AfbSubCall {
            api_cb: Some(Box::leak(callback)),
            verb_cb: None,
        };
        let cbbox = Box::new(cbbox);

        unsafe {
            cglue::afb_api_call(
                (*api).get_apiv4(),
                apistr.into_raw(),
                verbstr.into_raw(),
                params.arguments.len() as u32,
                params.arguments.as_slice().as_ptr(),
                Some(afb_async_api_callback),
                Box::leak(cbbox) as *const _ as *mut std::ffi::c_void,
            )
        };
    }

    fn subcall_sync(
        api: &AfbApi,
        apiname: &str,
        verbname: &str,
        params: &AfbParams,
    ) -> Result<AfbData, AfbError> {
        let mut status = 0 as i32;
        let mut nreplies = MAX_CALL_ARGS;
        let replies = [0 as cglue::afb_data_t; MAX_CALL_ARGS as usize];
        let apistr = CString::new(apiname).expect("Invalid apiname");
        let verbstr = CString::new(verbname).expect("Invalid verbname");

        let rc = unsafe {
            cglue::afb_api_call_sync(
                (*api).get_apiv4(),
                apistr.into_raw(),
                verbstr.into_raw(),
                params.arguments.len() as u32,
                params.arguments.as_slice().as_ptr(),
                &mut status,
                &mut nreplies,
                replies.as_ref() as *const _ as *mut cglue::afb_data_t,
            )
        };
        if rc < 0 || nreplies > MAX_CALL_ARGS {
            return Err(AfbError::new(
                "api-subcall",
                format!(
                    "api:{} verb:{} rc={} info={}",
                    apiname,
                    verbname,
                    rc,
                    afb_error_info(rc)
                ),
            ));
        }
        let datas = AfbData::new(&replies, nreplies, status);
        Ok(datas)
    }
}

impl DoSubcall<AfbApiV4, Box<dyn AfbSubcallControl>> for AfbSubCall {
    fn subcall_async(
        apiv4: AfbApiV4,
        apiname: &str,
        verbname: &str,
        params: &AfbParams,
        callback: Box<dyn AfbSubcallControl>,
    ) {
        let apistr = CString::new(apiname).expect("Invalid apiname");
        let verbstr = CString::new(verbname).expect("Invalid verbname");

        // lock callback box into memory until AFB returns from subcall
        let cbbox = AfbSubCall {
            api_cb: Some(Box::leak(callback)),
            verb_cb: None,
        };
        let cbbox = Box::new(cbbox);

        unsafe {
            cglue::afb_api_call(
                apiv4,
                apistr.into_raw(),
                verbstr.into_raw(),
                params.arguments.len() as u32,
                params.arguments.as_slice().as_ptr(),
                Some(afb_async_api_callback),
                Box::leak(cbbox) as *const _ as *mut std::ffi::c_void,
            )
        };
    }

    fn subcall_sync(
        apiv4: AfbApiV4,
        apiname: &str,
        verbname: &str,
        params: &AfbParams,
    ) -> Result<AfbData, AfbError> {
        let mut status = 0 as i32;
        let mut nreplies = MAX_CALL_ARGS;
        let replies = [0 as cglue::afb_data_t; MAX_CALL_ARGS as usize];
        let apistr = CString::new(apiname).expect("Invalid apiname");
        let verbstr = CString::new(verbname).expect("Invalid verbname");

        let rc = unsafe {
            cglue::afb_api_call_sync(
                apiv4,
                apistr.into_raw(),
                verbstr.into_raw(),
                params.arguments.len() as u32,
                params.arguments.as_slice().as_ptr(),
                &mut status,
                &mut nreplies,
                replies.as_ref() as *const _ as *mut cglue::afb_data_t,
            )
        };
        if rc < 0 || nreplies > MAX_CALL_ARGS {
            return Err(AfbError::new(
                "api-subcall",
                format!(
                    "api:{} verb:{} rc={} info={}",
                    apiname,
                    verbname,
                    rc,
                    afb_error_info(rc)
                ),
            ));
        }
        let datas = AfbData::new(&replies, nreplies, status);
        Ok(datas)
    }
}

impl<'a> DoSubcall<&AfbRequest<'a>, Box<dyn AfbRqtControl>> for AfbSubCall {
    fn subcall_async(
        rqt: &AfbRequest,
        apiname: &str,
        verbname: &str,
        params: &AfbParams,
        callback: Box<dyn AfbRqtControl>,
    ) {
        let apistr = CString::new(apiname).expect("Invalid apiname");
        let verbstr = CString::new(verbname).expect("Invalid verbname");

        // lock callback box into memory until AFB returns from subcall
        let cbbox = AfbSubCall {
            verb_cb: Some(Box::leak(callback)),
            api_cb: None,
        };
        let cbbox = Box::new(cbbox);

        unsafe {
            cglue::afb_req_subcall(
                (*rqt).get_rqtv4(),
                apistr.into_raw(),
                verbstr.into_raw(),
                params.arguments.len() as u32,
                params.arguments.as_slice().as_ptr(),
                cglue::afb_req_subcall_flags_afb_req_subcall_catch_events as i32,
                Some(afb_async_rqt_callback),
                Box::leak(cbbox) as *const _ as *mut std::ffi::c_void,
            )
        };
    }

    fn subcall_sync(
        rqt: &AfbRequest,
        apiname: &str,
        verbname: &str,
        params: &AfbParams,
    ) -> Result<AfbData, AfbError> {
        let mut status = 0 as i32;
        let mut nreplies = MAX_CALL_ARGS;
        let replies = [0 as cglue::afb_data_t; MAX_CALL_ARGS as usize];
        let apistr = CString::new(apiname).expect("Invalid apiname");
        let verbstr = CString::new(verbname).expect("Invalid verbname");

        let rc = unsafe {
            // err= afb_req_subcall_sync (glue->rqt.afb, apiname, verbname, (int)index, params, afb_req_subcall_catch_events, &status, &nreplies, replies);
            cglue::afb_req_subcall_sync(
                (*rqt).get_rqtv4(),
                apistr.into_raw(),
                verbstr.into_raw(),
                params.arguments.len() as u32,
                params.arguments.as_slice().as_ptr(),
                cglue::afb_req_subcall_flags_afb_req_subcall_catch_events as i32,
                &mut status,
                &mut nreplies,
                replies.as_ref() as *const _ as *mut cglue::afb_data_t,
            )
        };
        if rc < 0 {
            return Err(AfbError::new(
                "api-subcall",
                format!(
                    "api:{} verb:{} rc={} info={}",
                    apiname,
                    verbname,
                    rc,
                    afb_error_info(rc)
                ),
            ));
        }
        // move const **array in something Rust may understand
        let datas = AfbData::new(&replies, nreplies, status);
        Ok(datas)
    }
}

impl DoSubcall<AfbRqtV4, Box<dyn AfbRqtControl>> for AfbSubCall {
    fn subcall_async(
        rqtv4: AfbRqtV4,
        apiname: &str,
        verbname: &str,
        params: &AfbParams,
        callback: Box<dyn AfbRqtControl>,
    ) {
        let apistr = CString::new(apiname).expect("Invalid apiname");
        let verbstr = CString::new(verbname).expect("Invalid verbname");

        // lock callback box into memory until AFB returns from subcall
        let cbbox = AfbSubCall {
            verb_cb: Some(Box::leak(callback)),
            api_cb: None,
        };
        let cbbox = Box::new(cbbox);

        unsafe {
            cglue::afb_req_subcall(
                rqtv4,
                apistr.into_raw(),
                verbstr.into_raw(),
                params.arguments.len() as u32,
                params.arguments.as_slice().as_ptr(),
                cglue::afb_req_subcall_flags_afb_req_subcall_catch_events as i32,
                Some(afb_async_rqt_callback),
                Box::leak(cbbox) as *const _ as *mut std::ffi::c_void,
            )
        };
    }

    fn subcall_sync(
        rqtv4: AfbRqtV4,
        apiname: &str,
        verbname: &str,
        params: &AfbParams,
    ) -> Result<AfbData, AfbError> {
        let mut status = 0 as i32;
        let mut nreplies = MAX_CALL_ARGS;
        let replies = [0 as cglue::afb_data_t; MAX_CALL_ARGS as usize];
        let apistr = CString::new(apiname).expect("Invalid apiname");
        let verbstr = CString::new(verbname).expect("Invalid verbname");

        let rc = unsafe {
            // err= afb_req_subcall_sync (glue->rqt.afb, apiname, verbname, (int)index, params, afb_req_subcall_catch_events, &status, &nreplies, replies);
            cglue::afb_req_subcall_sync(
                rqtv4,
                apistr.into_raw(),
                verbstr.into_raw(),
                params.arguments.len() as u32,
                params.arguments.as_slice().as_ptr(),
                cglue::afb_req_subcall_flags_afb_req_subcall_catch_events as i32,
                &mut status,
                &mut nreplies,
                replies.as_ref() as *const _ as *mut cglue::afb_data_t,
            )
        };
        if rc < 0 {
            return Err(AfbError::new(
                "api-subcall",
                format!(
                    "api:{} verb:{} rc={} info={}",
                    apiname,
                    verbname,
                    rc,
                    afb_error_info(rc)
                ),
            ));
        }
        // move const **array in something Rust may understand
        let datas = AfbData::new(&replies, nreplies, status);
        Ok(datas)
    }
}

impl AfbSubCall {
    pub fn call_sync<H, T, C>(
        handle: H,
        apiname: &str,
        verbname: &str,
        args: T,
    ) -> Result<AfbData, AfbError>
    where
        AfbParams: ConvertResponse<T>,
        AfbSubCall: DoSubcall<H, C>,
    {
        let response = AfbParams::convert(args);
        let mut params = match response {
            Err(error) => {
                return Err(error);
            }
            Ok(data) => data,
        };
        AfbSubCall::subcall_sync(handle, apiname, verbname, &mut params)
    }

    pub fn call_async<H, T, C>(
        handle: H,
        apiname: &str,
        verbname: &str,
        args: T,
        cbhandle: C,
    ) -> Result<(), AfbError>
    where
        AfbParams: ConvertResponse<T>,
        AfbSubCall: DoSubcall<H, C>,
    {
        let response = AfbParams::convert(args);
        let mut params = match response {
            Err(error) => {
                return Err(error);
            }
            Ok(data) => data,
        };

        // store cbhandle trait as a leaked box
        Ok(AfbSubCall::subcall_async(
            handle,
            apiname,
            verbname,
            &mut params,
            cbhandle,
        ))
    }
}
