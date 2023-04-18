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
use apiv4::*;
use datav4::*;

use cgluev4::{self as cglue};
use jsonc::jsonc_mod::{JsoncObj, Jtype};
use std::collections::HashMap;
use std::ffi::CString;
use std::fmt;
use std::sync::{Arc, Condvar, Mutex};
use bitflags::bitflags;

const AUTOSTART: &str = "autostart";
const TIMEOUT: u32 = 5;

pub type AfbTmrV4 = cglue::afb_timer_t;

pub struct DbgInfo {
    pub name: &'static str,
    pub file: &'static str,
    pub line: i32,
}

pub struct AfbError {
    uid: String,
    info: String,
    dbg_info: Option<DbgInfo>,
}

pub use AfbAuthAllOf;
#[macro_export]
macro_rules! AfbAuthAllOf {
 ( $( $args:expr ),*) => {
    {
    let mut vect= Vec::new();
    $(
        vect.push(AfbPermission::from($args));
    )*
    libafb::utilv4::AfbPermission::new(libafb::utilv4::AfbPermission::AnyOf(vect))
    }
 };
}

pub use AfbAuthAnyOf;
#[macro_export]
macro_rules! AfbAuthAnyOf {
 ( $( $args:expr ),*) => {
    {
    let mut vect= Vec::new();
    $(
        vect.push(AfbPermission::from($args));
    )*
    libafb::utilv4::AfbPermission::new(libafb::utilv4::AfbPermission::AllOf(vect))
    }
 };
}

pub use AfbTimerRegister;
#[macro_export]
macro_rules! AfbTimerRegister {
    ($timer_name:ident, $callback:ident, $userdata:ident) => {
        #[allow(non_camel_case_types)]
        type $timer_name = $userdata;
        impl libafb::utilv4::AfbTimerControl for $userdata {
            fn timer_callback(&mut self, timer: &libafb::utilv4::AfbTimer, decount: u32) {
                $callback(timer, decount, self)
            }
        }
    };
    ($timer_name: ident, $callback:ident) => {
        #[allow(non_camel_case_types)]
        struct $timer_name;
        impl libafb::apiv4::AfbTimerControl for $timer_name {
            fn timer_callback(&mut self, timer: &libafb::utilv4::AfbTimer, decount: u32) {
                $callback(timer, decount)
            }
        }
    };
}

pub use AfbEvtFdRegister;
#[macro_export]
macro_rules! AfbEvtFdRegister {
    ($evtfd_name:ident, $callback:ident, $userdata:ident) => {
        #[allow(non_camel_case_types)]
        type $evtfd_name = $userdata;
        impl libafb::utilv4::AfbEvtFdControl for $userdata {
            fn evtfd_callback(&mut self, evtfd: &libafb::utilv4::AfbEvtFd, revents: u32) {
                $callback(evtfd, revents, self)
            }
        }
    };
    ($evtfd_name: ident, $callback:ident) => {
        #[allow(non_camel_case_types)]
        struct $evtfd_name;
        impl libafb::apiv4::AfbEvtFdControl for $evtfd_name {
            fn evtfd_callback(&mut self, evtfd: &libafb::utilv4::AfbEvtFd, revents: u32) {
                $callback(evtfd, revents)
            }
        }
    };
}

pub use AfbJobRegister;
#[macro_export]
macro_rules! AfbJobRegister {
    ($job_name:ident, $callback:ident, $userdata:ident) => {
        #[allow(non_camel_case_types)]
        type $job_name = $userdata;
        impl libafb::utilv4::AfbJobControl for $userdata {
            fn job_callback(&mut self, job: &libafb::utilv4::AfbSchedJob, signal: i32) {
                $callback(job, signal, self)
            }
        }
    };
    ($job_name: ident, $callback:ident) => {
        #[allow(non_camel_case_types)]
        struct $job_name;
        impl libafb::utilv4::AfbJobControl for $job_name {
            fn job_callback(&mut self, job: &libafb::utilv4::AfbSchedJob, signal: i32) {
                $callback(job, signal)
            }
        }
    };
}

pub use afb_log_msg;
#[macro_export]
macro_rules! afb_log_msg {
 ( $level:tt, $handle:expr,$format:expr, $( $args:expr ),*) => {
    let dbg_info = Some(DbgInfo {
        name: func_name!(),
        file: file!(),
        line: line!() as i32,
    });
    let message= format! ($format, $($args),*);
    AfbLogMsg::push_log (AfbLogLevel::$level, $handle, message, dbg_info)
 };
 ( $level:tt, $handle:expr,$format:expr) => {
    let dbg_info = Some(DbgInfo {
        name: func_name!(),
        file: file!(),
        line: line!() as i32,
    });
    AfbLogMsg::push_log (AfbLogLevel::$level, $handle, $format, dbg_info)
 }
}

pub use afb_log_raw;
#[macro_export]
macro_rules! afb_log_raw {
 ( $level:tt, $handle:expr,$format:expr, $( $args:expr ),*) => {
    let format= format! ($format, $($args),*);
    AfbLogMsg::push_log (AfbLogLevel::$level, $handle, format, None)
 }
}

pub use func_name;
#[doc(hidden)]
#[macro_export]
macro_rules! func_name {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}

pub use afb_add_trace;
#[macro_export]
macro_rules! afb_add_trace {
    ($afb_error:ident) => {
        $afb_error.add_trace(func_name!(), file!(), line!())
    };
}

pub trait MakeError<T> {
    fn make(uid: &str, msg: T) -> AfbError;
}

impl MakeError<&str> for AfbError {
    fn make(uid: &str, msg: &str) -> AfbError {
        AfbError {
            uid: uid.to_string(),
            info: msg.to_string(),
            dbg_info: None,
        }
    }
}

impl MakeError<String> for AfbError {
    fn make(uid: &str, msg: String) -> AfbError {
        AfbError {
            uid: uid.to_string(),
            info: msg,
            dbg_info: None,
        }
    }
}

impl AfbError {
    pub fn new<T>(uid: &str, msg: T) -> AfbError
    where
        AfbError: MakeError<T>,
    {
        Self::make(uid, msg)
    }
    pub fn get_uid(&self) -> String {
        self.uid.to_owned()
    }
    pub fn get_info(&self) -> String {
        self.info.to_owned()
    }

    pub fn add_trace(&mut self, name: &'static str, file: &'static str, line: u32) -> &Self {
        self.dbg_info = Some(DbgInfo {
            name: name,
            file: file,
            line: line as i32,
        });
        self
    }

    pub fn to_jsonc(&self) -> AfbJsonObj {
        let do_jerror = || -> Result<AfbJsonObj, &'static str> {
            let jobject = AfbJsonObj::new();
            jobject.add("uid", &self.uid)?.add("info", &self.info)?;
            Ok(jobject)
        };

        let do_jdebug = |info: &DbgInfo| -> Result<AfbJsonObj, &'static str> {
            let jobject = AfbJsonObj::new();
            jobject
                .add("name", info.name)?
                .add("file", info.file)?
                .add("line", info.line)?;
            Ok(jobject)
        };

        let jerror = match do_jerror() {
            Err(error) => AfbJsonObj::from(error),
            Ok(jobject) => {
                match &self.dbg_info {
                    None => (),
                    Some(info) => {
                        match do_jdebug(info) {
                            Err(error) => {
                                jobject
                                    .add("dbg", AfbJsonObj::from(error))
                                    .expect("(hoops: AfbError->jsonc fail");
                            }
                            Ok(jdebug) => {
                                jobject
                                    .add("dbg", jdebug)
                                    .expect("(hoops: AfbError->jsonc fail");
                            }
                        };
                    }
                }
                jobject
            }
        };
        jerror
    }
}

impl fmt::Display for AfbError {
    fn fmt(&self, format: &mut fmt::Formatter) -> fmt::Result {
        write!(format, "uid:{} info:{}", self.uid, self.info)
    }
}

impl fmt::Debug for AfbError {
    fn fmt(&self, format: &mut fmt::Formatter) -> fmt::Result {
        write!(format, "uid:{} info:{}", self.uid, self.info)
    }
}

pub struct AfbLogMsg {}

pub trait DoSendLog<T> {
    fn print_log(
        level: i32,
        handle: T,
        file: *mut i8,
        line: i32,
        funcname: *mut i8,
        format: *mut i8,
    );
}

impl<'a> DoSendLog<&AfbEventMsg<'a>> for AfbLogMsg {
    fn print_log(
        level: i32,
        event: &AfbEventMsg,
        file: *mut i8,
        line: i32,
        funcname: *mut i8,
        format: *mut i8,
    ) {
        unsafe {
            cglue::afb_api_verbose(
                (*event).get_api().get_apiv4(),
                level,
                file,
                line as i32,
                funcname,
                format,
            )
        }
    }
}

impl<'a> DoSendLog<&AfbTimer> for AfbLogMsg {
    fn print_log(
        level: i32,
        _timer: &AfbTimer,
        file: *mut i8,
        line: i32,
        funcname: *mut i8,
        format: *mut i8,
    ) {
        unsafe { cglue::afb_verbose(level, file, line as i32, funcname, format) }
    }
}

impl<'a> DoSendLog<&AfbSchedJob> for AfbLogMsg {
    fn print_log(
        level: i32,
        _timer: &AfbSchedJob,
        file: *mut i8,
        line: i32,
        funcname: *mut i8,
        format: *mut i8,
    ) {
        unsafe { cglue::afb_verbose(level, file, line as i32, funcname, format) }
    }
}

impl<'a> DoSendLog<&AfbRequest<'a>> for AfbLogMsg {
    fn print_log(
        level: i32,
        rqt: &AfbRequest<'a>,
        file: *mut i8,
        line: i32,
        funcname: *mut i8,
        format: *mut i8,
    ) {
        unsafe {
            cglue::afb_req_verbose(
                (*rqt).get_rqtv4(),
                level,
                file,
                line as i32,
                funcname,
                format,
            )
        }
    }
}

impl<'a> DoSendLog<&AfbApi> for AfbLogMsg {
    fn print_log(
        level: i32,
        api: &AfbApi,
        file: *mut i8,
        line: i32,
        funcname: *mut i8,
        format: *mut i8,
    ) {
        unsafe {
            cglue::afb_api_verbose(
                (*api).get_apiv4(),
                level,
                file,
                line as i32,
                funcname,
                format,
            )
        }
    }
}

impl DoSendLog<&AfbEvent> for AfbLogMsg {
    fn print_log(
        level: i32,
        event: &AfbEvent,
        file: *mut i8,
        line: i32,
        funcname: *mut i8,
        format: *mut i8,
    ) {
        unsafe {
            cglue::afb_api_verbose(
                (*event).get_apiv4(),
                level,
                file,
                line as i32,
                funcname,
                format,
            )
        }
    }
}

impl DoSendLog<Option<u32>> for AfbLogMsg {
    fn print_log(
        level: i32,
        _not_used: Option<u32>,
        file: *mut i8,
        line: i32,
        funcname: *mut i8,
        format: *mut i8,
    ) {
        unsafe { cglue::afb_verbose(level, file, line as i32, funcname, format) }
    }
}

impl DoSendLog<AfbRqtV4> for AfbLogMsg {
    fn print_log(
        level: i32,
        rqtv4: cglue::afb_req_t,
        file: *mut i8,
        line: i32,
        funcname: *mut i8,
        format: *mut i8,
    ) {
        unsafe { cglue::afb_req_verbose(rqtv4, level, file, line as i32, funcname, format) }
    }
}

impl DoSendLog<AfbApiV4> for AfbLogMsg {
    fn print_log(
        level: i32,
        apiv4: AfbApiV4,
        file: *mut i8,
        line: i32,
        funcname: *mut i8,
        format: *mut i8,
    ) {
        unsafe { cglue::afb_api_verbose(apiv4, level, file, line as i32, funcname, format) }
    }
}

pub trait DoMessage<T> {
    fn as_string(msg: T) -> String;
}

impl DoMessage<&AfbError> for AfbLogMsg {
    fn as_string(msg: &AfbError) -> String {
        format!("{}", msg)
    }
}

impl DoMessage<String> for AfbLogMsg {
    fn as_string(msg: String) -> String {
        msg
    }
}

impl DoMessage<&str> for AfbLogMsg {
    fn as_string(msg: &str) -> String {
        msg.to_string()
    }
}

pub enum AfbLogLevel {
    Error = cglue::afb_syslog_levels_AFB_SYSLOG_LEVEL_ERROR as isize,
    Debug = cglue::afb_syslog_levels_AFB_SYSLOG_LEVEL_DEBUG as isize,
    Notice = cglue::afb_syslog_levels_AFB_SYSLOG_LEVEL_NOTICE as isize,
    Critical = cglue::afb_syslog_levels_AFB_SYSLOG_LEVEL_CRITICAL as isize,
    Warning = cglue::afb_syslog_levels_AFB_SYSLOG_LEVEL_WARNING as isize,
    Emergency = cglue::afb_syslog_levels_AFB_SYSLOG_LEVEL_EMERGENCY as isize,
    Info = cglue::afb_syslog_levels_AFB_SYSLOG_LEVEL_INFO as isize,
    Unknown = -1,
}

impl AfbLogMsg {
    pub fn get_level(log_level: AfbLogLevel) -> i32 {
        let level = match log_level {
            AfbLogLevel::Debug => AfbLogLevel::Debug,
            AfbLogLevel::Info => AfbLogLevel::Info,
            AfbLogLevel::Notice => AfbLogLevel::Notice,
            AfbLogLevel::Critical => AfbLogLevel::Critical,
            AfbLogLevel::Error => AfbLogLevel::Error,
            AfbLogLevel::Warning => AfbLogLevel::Warning,
            AfbLogLevel::Emergency => AfbLogLevel::Emergency,
            _ => AfbLogLevel::Unknown,
        };
        level as i32
    }

    pub fn push_log<H, T>(level: AfbLogLevel, handle: H, format: T, info: Option<DbgInfo>)
    where
        AfbLogMsg: DoMessage<T>,
        AfbLogMsg: DoSendLog<H>,
    {
        //let level = Self::verbosity(handle);
        let log_level = AfbLogMsg::get_level(level);
        let message = Self::as_string(format);

        match info {
            Some(dbg) => {
                let line = dbg.line;
                let file = CString::new(dbg.file)
                    .expect("Invalid filename string")
                    .into_raw();
                let func = CString::new(dbg.name)
                    .expect("Invalid func_name string")
                    .into_raw();
                let format = CString::new(message)
                    .expect("Invalid message string")
                    .into_raw();
                Self::print_log(log_level as i32, handle, file, line, func, format);
            }
            None => {
                let line = 0;
                let file = 0 as *mut i8;
                let func = 0 as *mut i8;
                let format = CString::new(message)
                    .expect("Invalid message string")
                    .into_raw();
                Self::print_log(log_level as i32, handle, file, line, func, format);
            }
        };
    }
}

pub trait AfbTimerControl {
    fn timer_callback(&mut self, timer: &AfbTimer, decount: u32);
}

// Afb AfbTimerHandle implementation
// ------------------------
#[no_mangle]
pub extern "C" fn api_timers_cb(
    _timer: cglue::afb_timer_t,
    userdata: *mut std::os::raw::c_void,
    decount: u32,
) {
    // extract timer+api object from libafb internals
    let timer_ref = unsafe { &mut *(userdata as *mut AfbTimer) };

    // call timer calback
    match timer_ref.callback {
        Some(timer_control) => unsafe { (*timer_control).timer_callback(timer_ref, decount) },
        _ => panic!("timer={} no callback defined", timer_ref._uid),
    }

    // clean callback control box
    if decount == 1 {
        let _ctrlbox = unsafe { Box::from_raw(timer_ref) };
    }
}

pub struct AfbTimer {
    _uid: &'static str,
    _timerv4: AfbTmrV4,
    info: &'static str,
    callback: Option<*mut dyn AfbTimerControl>,
    decount: u32,
    period: u32,
    autounref: i32,
}

impl AfbTimer {
    pub fn new(uid: &'static str) -> &'static mut Self {
        let timer_box = Box::new(AfbTimer {
            _uid: uid,
            info: "",
            callback: None,
            decount: 0,
            period: 0,
            _timerv4: 0 as cglue::afb_timer_t,
            autounref: 0,
        });
        Box::leak(timer_box)
    }

    pub fn set_info(&mut self, value: &'static str) -> &mut Self {
        self.info = value;
        self
    }

    pub fn set_period(&mut self, ms: u32) -> &mut Self {
        self.period = ms;
        self
    }
    pub fn set_decount(&mut self, decount: u32) -> &mut Self {
        self.decount = decount;
        self
    }
    pub fn set_autounref(&mut self, value: i32) -> &mut Self {
        self.autounref = value;
        self
    }
    pub fn set_callback(&mut self, ctrlbox: Box<dyn AfbTimerControl>) -> &mut Self {
        self.callback = Some(Box::leak(ctrlbox));
        self
    }

    pub fn start(&mut self) -> Result<&Self, AfbError> {
        if self.period == 0 || self.callback == None {
            return Err(AfbError::new(
                self._uid,
                "Timer callback must be set and period should >0",
            ));
        }

        let status = unsafe {
            cglue::afb_timer_create(
                &mut self._timerv4,
                0,
                0,
                0,
                self.decount,
                self.period,
                0,
                Some(api_timers_cb),
                self as *const _ as *mut std::ffi::c_void,
                self.autounref,
            )
        };
        if status != 0 {
            return Err(AfbError::new(self._uid, "Afb_Timer creation fail"));
        }
        Ok(self)
    }

    pub fn get_uid(&self) -> &'static str {
        self._uid
    }

    pub fn unref(&self) {
        unsafe { cglue::afb_timer_unref(self._timerv4) };
    }

    pub fn addref(&self) {
        unsafe { cglue::afb_timer_addref(self._timerv4) };
    }

    pub fn get_info(&self) -> &'static str {
        self.info
    }
}

impl fmt::Display for AfbTimer {
    /// timer simple printing output
    /// format {} => print uid,name,info
    /// Examples
    /// ```compile_fail
    /// println!("timer={}", timer);
    /// ```
    fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(format, "uid:{} info:{}", self._uid, self.info)
    }
}

#[no_mangle]
pub extern "C" fn api_schedjob_cb(signal: i32, userdata: *mut std::os::raw::c_void) {
    // extract timer+api object from libafb internals
    let job_ref = unsafe { &mut *(userdata as *mut AfbSchedJob) };

    // call timer calback
    match job_ref.callback {
        Some(control) => unsafe { (*control).job_callback(job_ref, signal) },
        _ => panic!("schedjob={} no callback defined", job_ref._uid),
    }

    // clean callback control box
    let _ctrlbox = unsafe { Box::from_raw(job_ref) };
    if signal != 0 {
        unsafe { cglue::afb_job_abort(job_ref._jobv4) };
    }
}

pub trait AfbJobControl {
    fn job_callback(&mut self, jobs: &AfbSchedJob, signal: i32);
}
pub struct AfbSchedJob {
    _uid: &'static str,
    _jobv4: i32,
    info: &'static str,
    watchdog: i32,
    callback: Option<*mut dyn AfbJobControl>,
}

impl AfbSchedJob {
    pub fn new(uid: &'static str) -> &'static mut Self {
        let job_box = Box::new(AfbSchedJob {
            _uid: uid,
            _jobv4: 0,
            info: "",
            watchdog: 0,
            callback: None,
        });
        Box::leak(job_box)
    }

    pub fn set_exec_watchdog(&mut self, exec_watchdog: i32) -> &mut Self {
        self.watchdog = exec_watchdog;
        self
    }

    pub fn set_info(&mut self, info: &'static str) -> &mut Self {
        self.info = info;
        self
    }

    pub fn set_callback(&mut self, ctrlbox: Box<dyn AfbJobControl>) -> &mut Self {
        self.callback = Some(Box::leak(ctrlbox));
        self
    }

    pub fn get_jobid(&self) -> i32 {
        self._jobv4
    }

    pub fn get_uid(&self) -> &'static str {
        self._uid
    }

    pub fn post(&mut self, delay_ms: i64) -> Result<&mut Self, AfbError> {
        match self.callback {
            None => Err(AfbError::new(
                self._uid,
                "schedjob require callback setting",
            )),
            Some(_control) => {
                let jobv4 = unsafe {
                    cglue::afb_job_post(
                        delay_ms,
                        self.watchdog,
                        Some(api_schedjob_cb),
                        self as *const _ as *mut std::ffi::c_void,
                        0 as *mut std::ffi::c_void,
                    )
                };
                if jobv4 <= 0 {
                    return Err(AfbError::new(self._uid, "Afb_Timer creation fail"));
                }
                self._jobv4 = jobv4;
                Ok(self)
            }
        }
    }

    pub fn get_info(&self) -> &'static str {
        self.info
    }
    pub fn abort(&self) -> Result<(), AfbError> {
        let rc = unsafe { cglue::afb_job_abort(self._jobv4) };
        if rc < 0 {
            Err(AfbError::new(
                self._uid,
                format!("No job running id={}", self._jobv4),
            ))
        } else {
            Ok(())
        }
    }
}

impl fmt::Display for AfbSchedJob {
    /// AfbSchedjob simple printing output
    /// format {} => print uid,name,info
    /// Examples
    /// ```compile_fail
    /// println!("schedjob={}", timer);
    /// ```
    fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(format, "uid:{} info:{}", self._uid, self.info)
    }
}

pub const AFB_AUTH_DFLT_V4: *mut AfbAuthV4 = 0 as *mut AfbAuthV4;
pub type AfbAuthV4 = cglue::afb_auth;
pub struct AfbPermisionV4 {}
impl AfbPermisionV4 {
    /// translate 'rust' permissions into 'libafb' permissions
    pub fn new(permission: &'static AfbPermission, inherited: *const AfbAuthV4) -> *mut AfbAuthV4 {
        let auth = match permission {
            AfbPermission::None() => AFB_AUTH_DFLT_V4,
            AfbPermission::Loa(value) => {
                let auth_box = Box::new(AfbAuthV4 {
                    type_: cglue::afb_auth_type_afb_auth_LOA,
                    __bindgen_anon_1: cglue::afb_auth__bindgen_ty_1 { loa: *value as u32 },
                    next: AFB_AUTH_DFLT_V4,
                });
                Box::leak(auth_box) as *mut AfbAuthV4
            }
            AfbPermission::Require(value) => {
                let perm = CString::new(*value).expect("invalid permission string");
                let auth_box = Box::new(AfbAuthV4 {
                    type_: cglue::afb_auth_type_afb_auth_Permission,
                    __bindgen_anon_1: cglue::afb_auth__bindgen_ty_1 {
                        text: perm.into_raw(),
                    },
                    next: AFB_AUTH_DFLT_V4,
                });
                Box::leak(auth_box) as *mut AfbAuthV4
            }
            AfbPermission::AnyOf(values) => {
                let mut next = AFB_AUTH_DFLT_V4;
                for slot in values {
                    let auth_box = Box::new(AfbAuthV4 {
                        type_: cglue::afb_auth_type_afb_auth_And,
                        __bindgen_anon_1: cglue::afb_auth__bindgen_ty_1 {
                            first: AfbPermisionV4::new(&slot, AFB_AUTH_DFLT_V4),
                        },
                        next: next,
                    });
                    next = Box::leak(auth_box);
                }
                next
            }
            AfbPermission::AllOf(values) => {
                let mut next = AFB_AUTH_DFLT_V4;
                for slot in values {
                    let auth_box = Box::new(AfbAuthV4 {
                        type_: cglue::afb_auth_type_afb_auth_Or,
                        __bindgen_anon_1: cglue::afb_auth__bindgen_ty_1 {
                            first: AfbPermisionV4::new(&slot, AFB_AUTH_DFLT_V4),
                        },
                        next: next,
                    });
                    next = Box::leak(auth_box);
                }
                next as *mut AfbAuthV4
            }
            AfbPermission::Inner(value) => AfbPermisionV4::new(value, AFB_AUTH_DFLT_V4),
        };

        if inherited == AFB_AUTH_DFLT_V4 {
            auth
        } else if auth == AFB_AUTH_DFLT_V4 {
            inherited as *mut AfbAuthV4
        } else {
            let auth_box = Box::new(AfbAuthV4 {
                type_: cglue::afb_auth_type_afb_auth_And,
                __bindgen_anon_1: cglue::afb_auth__bindgen_ty_1 { first: auth },
                next: inherited,
            });
            Box::leak(auth_box)
        }
    }
}

pub enum AfbPermission {
    Loa(i32),
    Require(&'static str),
    AnyOf(Vec<AfbPermission>),
    AllOf(Vec<AfbPermission>),
    Inner(&'static AfbPermission),
    None(),
}

impl From<&'static str> for AfbPermission {
    fn from(value: &'static str) -> Self {
        AfbPermission::Require(value)
    }
}

impl From<&'static AfbPermission> for AfbPermission {
    fn from(value: &'static AfbPermission) -> Self {
        AfbPermission::Inner(value)
    }
}

impl From<i32> for AfbPermission {
    fn from(value: i32) -> Self {
        if value > 7 || value < -7 {
            panic!("LOA should be within [0-7] range");
        }
        if value != 0 {
            AfbPermission::Loa(value)
        } else {
            AfbPermission::None()
        }
    }
}

impl AfbPermission {
    pub fn from<T>(value: T) -> AfbPermission
    where
        T: Into<AfbPermission>,
    {
        value.into()
    }

    pub fn new<T>(permission: T) -> &'static mut Self
    where
        T: Into<AfbPermission>,
    {
        let boxe = Box::new(AfbPermission::from(permission));
        Box::leak(boxe)
    }
}

// TAP test
struct TapCtxData {
    test: *const AfbTapTest,
}

impl AfbJobControl for TapCtxData {
    fn job_callback(&mut self, _job: &AfbSchedJob, signal: i32) {
        let test = unsafe { &mut *(self.test as *mut AfbTapTest) };
        let suite = test.get_suite();
        let event = suite.get_event();

        let jsonc = AfbJsonObj::new();
        jsonc.add("index", test.index).unwrap();
        jsonc.add("test", test.uid).unwrap();
        event.push(jsonc);

        if signal == 0 {
            // subcall start
            let _ignore = test.call_sync();
        } else {
            // force a timeout response
            let no_data = [0 as cglue::afb_data_t; 0];
            let reply = AfbData::new(&no_data, 0, -62);
            let response = test.check_response(reply);
            test.done(response);
        }
    }
}
pub struct AfbTapResponse {
    pub status: i32,
    pub diagnostic: String,
}

pub struct AfbTapTest {
    pub uid: &'static str,
    pub info: &'static str,
    pub api: &'static str,
    pub verb: &'static str,
    pub status: i32,
    pub params: AfbParams,
    pub expect: Vec<AfbJsonObj>,
    pub onerror: Option<&'static str>,
    pub onsucess: Option<&'static str>,
    pub response: Option<AfbTapResponse>,
    pub timeout: u32,
    pub delay: u32,
    pub index: usize,
    semaphore: Arc<(Mutex<u32>, Condvar)>,
    group: *const AfbTapGroup,
}

impl AfbTapTest {
    pub fn new(
        uid: &'static str,
        api: &'static str,
        verb: &'static str,
    ) -> &'static mut AfbTapTest {
        let boxe = Box::new(AfbTapTest {
            uid: uid,
            info: "",
            api: api,
            verb: verb,
            status: 0,
            params: AfbParams::new(),
            expect: Vec::new(),
            onerror: None,
            onsucess: None,
            response: None,
            index: 0,
            timeout: 0,
            delay: 0,
            semaphore: Arc::new((Mutex::new(0), Condvar::new())),
            group: 0 as *mut AfbTapGroup,
        });
        Box::leak(boxe)
    }

    pub fn set_info(&mut self, value: &'static str) -> &mut Self {
        self.info = value;
        self
    }

    pub fn finalize(&mut self) -> Result<&mut Self, AfbError> {
        Ok(self)
    }

    pub fn set_response(&mut self, status: i32, diagnostic: &str) -> &mut Self {
        self.response = Some(AfbTapResponse {
            status: status,
            diagnostic: diagnostic.to_owned(),
        });
        self
    }

    pub fn set_timeout(&mut self, value: u32) -> &mut Self {
        self.timeout = value;
        self
    }

    pub fn set_delay(&mut self, value: u32) -> &mut Self {
        self.delay = value;
        self
    }

    pub fn set_onsuccess(&mut self, group: &'static str) -> &mut Self {
        self.onsucess = Some(group);
        self
    }

    pub fn set_onerror(&mut self, group: &'static str) -> &mut Self {
        self.onerror = Some(group);
        self
    }

    pub fn set_status(&mut self, value: i32) -> &mut Self {
        self.status = value;
        self
    }

    pub fn add_arg<T>(&mut self, param: T) -> Result<&mut Self, AfbError>
    where
        AfbParams: ConvertResponse<T>,
    {
        match self.params.push(param) {
            Err(error) => Err(error),
            Ok(_data) => Ok(self),
        }
    }

    pub fn add_expect<T>(&mut self, data: T) -> &mut Self
    where
        T: Into<AfbJsonObj>,
    {
        let jsonc = data.into();
        self.expect.push(jsonc);
        self
    }

    pub fn get_group(&self) -> &AfbTapGroup {
        unsafe { & *(self.group as *mut AfbTapGroup) }
    }

    pub fn get_suite(&self) -> &AfbTapSuite {
        let group= unsafe {& *(self.group as *mut AfbTapGroup)};
        group.get_suite()
    }

    fn check_response(&self, reply: AfbData) -> AfbTapResponse {
        let api = self.get_suite().get_api();

        if reply.get_status() != self.status {
            let msg = format!(
                "status={} info={}",
                reply.get_status(),
                afb_error_info(reply.get_status())
            );
            return AfbTapResponse {
                status: AFB_FAIL,
                diagnostic: msg,
            };
        }

        for idx in 0..self.expect.len() {
            let jexpect = self.expect[idx].clone();
            match reply.get::<AfbJsonObj>(idx) {
                // expect argument as no jsonc representation.
                Err(error) => {
                    let msg = error.to_jsonc().to_string();
                    return AfbTapResponse {
                        status: AFB_FAIL,
                        diagnostic: msg,
                    };
                }
                Ok(mut jvalue) => {
                    let jtest = if jexpect.is_type(Jtype::Object) {
                        jvalue.contains(jexpect.clone())
                    } else {
                        jvalue.equal(jexpect.clone())
                    };

                    match jtest {
                        Err(error) => {
                            afb_log_msg!(
                                Warning,
                                api,
                                "{} -> {} NotIn {}",
                                error,
                                jexpect,
                                jvalue
                            );
                            return AfbTapResponse {
                                status: AFB_FAIL,
                                diagnostic: error.to_owned(),
                            };
                        }
                        Ok(()) => {}
                    }
                }
            }
        }
        AfbTapResponse {
            status: AFB_OK,
            diagnostic: "".to_owned(),
        }
    }

    fn call_sync(&mut self) {
        let api = self.get_suite().get_api();

        afb_log_msg!(
            Info,
            api,
            "callsync idx:{} tap->uid:{} afb-api->'/{}/{}'",
            self.index,
            self.uid,
            self.api,
            self.verb
        );

        let result = AfbSubCall::call_sync(api, self.api, self.verb, self.params.clone());
        let response = match result {
            Err(error) => AfbTapResponse {
                status: -1,
                diagnostic: error.to_string(),
            },
            Ok(response) => self.check_response(response),
        };
        // decrease group test count and return result
        self.done(response);
    }

    // next_test should be executed within group thread context in order not to pile test execution
    fn get_next(&self) -> Option<&mut AfbTapTest> {
        let group = self.get_group();
        let suite = self.get_suite();

        // wait for job to be finish
        let (lock, cvar) = &*self.semaphore;
        let mut done = match lock.lock() {
            Err(_error) => return None,
            Ok(mutex) => mutex,
        };
        while *done != 0 {
            done = cvar.wait(done).unwrap();
        }

        let response = match &self.response {
            None => panic!("next-test require some response"),
            Some(value) => value,
        };

        if response.status == AFB_OK {
            match self.onsucess {
                None => group.get_test(self.index),
                Some(label) => match suite.get_group(label) {
                    None => None,
                    Some(next_group) => {
                        let next_test = next_group.get_test(0);
                        next_test
                    }
                },
            }
        } else {
            match self.onerror {
                None => group.get_test(self.index),
                Some(label) => match suite.get_group(label) {
                    None => None,
                    Some(group) => group.get_test(0),
                },
            }
        }
    }

    pub fn jobpost(&mut self) -> Result<(), AfbError> {
        let semaphore = Arc::clone(&self.semaphore);
        let (lock, cvar) = &*semaphore;
        let mut done = match lock.lock() {
            Err(_error) => {
                return Err(AfbError::new(
                    "fail-group-wait",
                    format!("fail waiting on tap group={}", self.uid),
                ))
            }
            Ok(mutex) => mutex,
        };
        *done = 1;
        cvar.notify_one();

        // use group timeout as default
        let timeout = if self.timeout == 0 {
            self.get_group().timeout
        } else {
            self.timeout
        };

        match AfbSchedJob::new(self.uid)
            .set_exec_watchdog(timeout as i32)
            .set_callback(Box::new(TapCtxData { test: self }))
            .post(self.delay as i64)
        {
            Err(_error) => {
                let response = AfbTapResponse {
                    status: -1,
                    diagnostic: "Fail to post job".to_owned(),
                };
                self.done(response);
            }
            Ok(_job) => {}
        };
        Ok(())
    }

    /// update test response and release semaphore
    pub fn done(&mut self, response: AfbTapResponse) {
        self.response = Some(response);
        let semaphore = Arc::clone(&self.semaphore);
        let (lock, cvar) = &*semaphore;
        let mut done = lock.lock().unwrap();
        *done = 0;
        cvar.notify_one();
    }

    pub fn get_report(&self) -> AfbJsonObj {
        let msg = match &self.response {
            None => {
                format!("ok {} - {} # SKIP", self.index, self.uid)
            }
            Some(response) => {
                if response.status == AFB_OK {
                    format!("ok {} - {}", self.index, self.uid)
                } else {
                    format!(
                        "not ok {} - {} # {}",
                        self.index, self.uid, response.diagnostic
                    )
                }
            }
        };
        AfbJsonObj::from(msg.as_str())
    }
}

pub struct AfbTapGroup {
    pub uid: &'static str,
    pub info: &'static str,
    pub tests: Vec<*mut AfbTapTest>,
    index: usize,
    pub timeout: u32,
    suite: *mut AfbTapSuite,
    api_group: *mut AfbGroup,
}

impl AfbTapGroup {
    pub fn new(uid: &'static str) -> &'static mut AfbTapGroup {
        let boxe = Box::new(AfbTapGroup {
            uid: uid,
            info: "",
            timeout: 0,
            tests: Vec::new(),
            index: 0,
            suite: 0 as *mut AfbTapSuite,
            api_group: AfbGroup::new(uid),
        });
        Box::leak(boxe)
    }

    pub fn finalize(&mut self) -> Result<&mut Self, AfbError> {
        Ok(self)
    }

    pub fn set_info(&'static mut self, value: &'static str) -> &'static mut Self {
        self.info = value;
        self
    }

    pub fn set_timeout(&'static mut self, value: u32) -> &'static mut Self {
        self.timeout = value;
        self
    }

    pub fn add_test(&'static mut self, test: &'static mut AfbTapTest) -> &'static mut Self {
        self.index += 1;
        test.index = self.index;
        test.group = self;
        self.tests.push(test);

        let verb = AfbVerb::new(test.uid)
            .set_info(test.info)
            .set_callback(Box::new(TapTestData { test: test }))
            .set_usage("no input")
            .finalize().unwrap();

        let api_group= unsafe {&mut *(self.api_group as *mut AfbGroup)};
        api_group.add_verb(verb);
        self
    }

    // return suite test until group end
    pub fn get_test(&self, index: usize) -> Option<&mut AfbTapTest> {
        if self.tests.len() <= index {
            None
        } else {
            let test= unsafe {&mut *(self.tests[index] as *mut AfbTapTest)};
            Some(test)
        }
    }

    pub fn get_suite(&self) -> &AfbTapSuite {
        unsafe { & *(self.suite as *const _ as *mut AfbTapSuite) }
    }

    pub fn launch(&self) -> Result<(), AfbError> {
        // get group 1st test
        let test = match self.get_test(0) {
            None => return Err(AfbError::new(self.uid, "no-test-found")),
            Some(value) => value,
        };

        // launch test and wait for completion
        test.jobpost().unwrap();

        // wait for jobpost completion before moving to next one
        let mut next = test.get_next();

        // callback return normaly or timeout
        while let Some(test) = next {
            test.jobpost().unwrap();
            next = test.get_next();
        }
        Ok(())
    }

    /// wait for group test to be done then print report
    pub fn get_report(&self) -> AfbJsonObj {
        let jsonc = AfbJsonObj::array();
        let count = self.tests.len();
        let msg = format!("1..{} # {}", count, self.uid);
        jsonc.insert(msg.as_str()).unwrap();
        for idx in 0..count {
            let test_ref = self.get_test(idx).unwrap();
            let test = &mut *(test_ref);
            jsonc.insert(test.get_report()).unwrap();
        }
        jsonc
    }
}

#[derive(Copy, Clone)]
pub enum AfbTapOutput {
    JSON,
    TAP,
    NONE,
}

pub struct AfbTapSuite {
    pub uid: &'static str,
    pub info: &'static str,
    pub autostart: *mut AfbTapGroup,
    pub autorun: bool,
    pub autoexit: bool,
    pub timeout: u32,
    pub output: AfbTapOutput,
    hashmap: HashMap<&'static str, *mut AfbTapGroup>,
    tap_api: *const AfbApi,
    event: *mut AfbEvent,
}

impl AfbTapSuite {
    pub fn new(api: &AfbApi, uid: &'static str) -> &'static mut AfbTapSuite {
        let mut hashmap: HashMap<&'static str, *mut AfbTapGroup> = HashMap::new();
        let autostart = AfbTapGroup::new(AUTOSTART);
        hashmap.insert(AUTOSTART, autostart);

        // register an event to notify test progression in api mode
        let event = AfbEvent::new(uid);
        event.register(api.get_apiv4());

        let boxe = Box::new(AfbTapSuite {
            uid: uid,
            info: "",
            autostart: autostart,
            autorun: true,
            autoexit: true,
            output: AfbTapOutput::TAP,
            timeout: TIMEOUT,
            hashmap: hashmap,
            tap_api: api,
            event: event,
        });

        // link autostart is default group to suite
        let suite = Box::leak(boxe);
        autostart.suite = suite;
        suite
    }

    pub fn set_timeout(&'static mut self, value: u32) -> &'static mut Self {
        self.timeout = value;
        self
    }

    pub fn add_test(&'static mut self, test: &'static mut AfbTapTest) -> &'static mut Self {
        let autostart = unsafe { &mut *(self.autostart) };
        if test.timeout == 0 {
            test.timeout = self.timeout;
        }
        autostart.add_test(test);
        self
    }

    pub fn add_group(&'static mut self, group: &mut AfbTapGroup) -> &'static mut Self {
        if group.timeout == 0 {
            group.timeout = self.timeout;
        }
        group.suite = self;
        self.hashmap.insert(group.uid, group);

        // create group test verb
        let vcbdata = TapGroupData { group: group };
        let api = unsafe { &mut *(self.tap_api as *mut AfbApi) };
        let verb = AfbVerb::new(group.uid);
        verb.set_callback(Box::new(vcbdata))
            .set_info(group.info)
            .set_usage("no_input")
            .register(api.get_apiv4(), AFB_NO_AUTH);
        api.add_verb(verb.finalize().unwrap());

        let api_group= unsafe {&mut *(group.api_group as *mut AfbGroup)};
        api_group.register(api.get_apiv4(), AFB_NO_AUTH);
        api.add_group(api_group);
        self
    }

    pub fn set_autorun(&'static mut self, value: bool) -> &'static mut Self {
        self.autorun = value;
        self
    }

    pub fn set_autoexit(&'static mut self, value: bool) -> &'static mut Self {
        self.autoexit = value;
        self
    }

    pub fn set_output(&'static mut self, value: AfbTapOutput) -> &'static mut Self {
        self.output = value;
        self
    }

    pub fn set_info(&'static mut self, value: &'static str) -> &'static mut Self {
        self.info = value;
        let api = unsafe { &mut *(self.tap_api as *mut AfbApi) };
        api.set_info(value);
        self
    }

    pub fn get_api(&self) -> &AfbApi {
        unsafe { & *(self.tap_api as *mut AfbApi) }
    }

    pub fn get_uid(&self) -> &'static str {
        self.uid
    }

    pub fn get_event(&self) -> &AfbEvent {
        unsafe { & *(self.event as *mut AfbEvent) }
    }

    pub fn get_group(&self, label: &'static str) -> Option<&AfbTapGroup> {
        let api = self.get_api();

        match self.hashmap.get(label) {
            Some(group) => {
                afb_log_msg!(Debug, api, "-- Get Tap group:{}", label);
                Some(unsafe { &mut *(*group as *mut AfbTapGroup) })
            }
            None => {
                afb_log_msg!(Critical, api, "Fail to find test-group:{}", label);
                None
            }
        }
    }

    // launch a group and return report as jsonc
    pub fn launch(&self, label: &'static str) -> Result<(), AfbError> {
        match self.get_group(label) {
            None => Err(AfbError::new("group-label-not-found", label)),
            Some(group) => group.launch(),
        }
    }

    pub fn finalize(&'static mut self) -> Result<(), AfbError> {
        // create autostart test verb and register notification event
        let vcbdata = TapGroupData {
            group: self.autostart,
        };
        let api = unsafe { &mut *(self.tap_api as *mut AfbApi) };
        let verb = AfbVerb::new(AUTOSTART);
        verb.set_callback(Box::new(vcbdata))
            .set_info("default tap autostart group")
            .set_usage("no_input")
            .register(api.get_apiv4(), AFB_NO_AUTH);
        api.add_verb(verb.finalize()?);

        // seal tap test api
        unsafe { cglue::afb_api_seal(api.get_apiv4()) };

        if self.autorun {
            match AfbSchedJob::new(self.uid)
                .set_callback(Box::new(TapSuiteAutoRun { suite: self }))
                .post(0)
            {
                Err(error) => Err(error),
                Ok(_job) => Ok(()),
            }
        } else {
            Ok(())
        }
    }

    pub fn get_autoexit(&self) -> bool {
        self.autoexit
    }

    pub fn get_autorun(&self) -> bool {
        self.autorun
    }

    pub fn get_report(&'static mut self) -> AfbJsonObj {
        let autostart = unsafe { &mut *(self.autostart) };
        let jreport = AfbJsonObj::new();
        jreport.add(AUTOSTART, autostart.get_report()).unwrap();

        for (uid, group) in self.hashmap.drain() {
            let group = unsafe { &mut (*group) };
            jreport.add(uid, group.get_report()).unwrap();
        }

        match self.output {
            AfbTapOutput::NONE => {}
            AfbTapOutput::JSON => {
                println!("{}", jreport);
            }
            AfbTapOutput::TAP => {
                println!("-- start:{} --", self.uid);
                let jvec = jreport.expand();
                for entry in &jvec {
                    let _key = entry.key.as_str();
                    println!();
                    match jreport.get::<JsoncObj>(entry.key.as_str()) {
                        Err(error) => {
                            afb_log_msg!(Critical, self.get_api().get_apiv4(), error);
                        }
                        Ok(jtest) => {
                            for idx in 0..jtest.count().unwrap() {
                                println!("{}", jtest.index::<String>(idx).unwrap().as_str());
                            }
                        }
                    }
                }
                println!("\n-- end:{} --", self.uid);
            }
        };
        jreport
    }
}

struct TapSuiteAutoRun {
    suite: *mut AfbTapSuite,
}

/// autostart is launched as job to complete API initialisation before effectively starting test suite
impl AfbJobControl for TapSuiteAutoRun {
    fn job_callback(&mut self, _jobs: &AfbSchedJob, _signal: i32) {
        let suite = unsafe {&mut *(self.suite as *mut AfbTapSuite)};
        let autostart = unsafe { &mut *(suite.autostart) };

        match autostart.launch() {
            Err(error) => {afb_log_msg!(Critical, suite.get_api().get_apiv4(), "Test fail {}:autostart error={}", suite.get_uid(), error);},
            Ok(()) => {},
        }

        let autoexit= suite.get_autoexit();
        suite.get_report();

        if autoexit {
            std::process::exit(0);
        }
    }
}

// implement TapTest API callback
struct TapTestData {
    test: *mut AfbTapTest,
}
impl AfbRqtControl for TapTestData {
    fn verb_callback(&mut self, rqt: &AfbRequest, _args: &AfbData) {
        // bypass Rust limitation that refuses to understand static object pointers
        let test = unsafe { &mut (*self.test) };
        match test.jobpost() {
            Err(error) => {
                afb_log_msg!(Error, rqt, "fail to launch test error={}", error);
                rqt.reply(error, 405);
            }
            Ok(_jreport) => {
                // wait for test to be completed
                let _next = test.get_next();
                rqt.reply(test.get_report(), 0);
            }
        }
    }
}

// implement TapGroup API callback
struct TapGroupData {
    group: *mut AfbTapGroup,
}

impl AfbRqtControl for TapGroupData {
    fn verb_callback(&mut self, rqt: &AfbRequest, _args: &AfbData) {
        // bypass Rust limitation that refuses to understand static object pointers
        let group = unsafe { &mut (*self.group) };
        let suite = unsafe { &mut (*group.suite) };
        let event = unsafe { &mut (*suite.event) };

        match event.subscribe(rqt) {
            Err(error) => {
                rqt.reply(error, 405);
                return;
            }
            Ok(_event) => {}
        }

        match group.launch() {
            Err(error) => {
                afb_log_msg!(Error, rqt, "fail to launch test error={}", error);
                rqt.reply(error, 405);
            }
            Ok(_jreport) => {
                rqt.reply(group.get_report(), 0);
            }
        }
    }
}


pub trait AfbEvtFdControl {
    fn evtfd_callback(&mut self, evfd: &AfbEvtFd, revents: u32);
}

// Afb AfbTimerHandle implementation
// ------------------------
#[no_mangle]
pub extern "C" fn api_evtfd_cb(
    _efd: cglue::afb_evfd_t,
    _fd: ::std::os::raw::c_int,
    revents: u32,
    userdata: *mut ::std::os::raw::c_void
) {
    // extract evtfd+api object from libafb internals
    let evtfd_ref = unsafe { &mut *(userdata as *mut AfbEvtFd) };

    // call evtfd calback
    match evtfd_ref.callback {
        Some(evtfd_control) => unsafe { (*evtfd_control).evtfd_callback(evtfd_ref, revents) },
        _ => panic!("evtfd={} no callback defined", evtfd_ref.uid),
    }

    // clean callback control box
    if revents ==  AfbEvtFdPoll::HUP.bits() {
        let _ctrlbox = unsafe { Box::from_raw(evtfd_ref) };
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct AfbEvtFdPoll: u32 {
        const IN = cglue::afb_epoll_epoll_IN;
        const OUT= cglue::afb_epoll_epoll_OUT;
        const HUP= cglue::afb_epoll_epoll_HUP;
        const ERR= cglue::afb_epoll_epoll_ERR;
    }
}

// Event FD add a filedescriptor to mainloop and connect a callback
pub struct AfbEvtFd {
    uid: &'static str,
    info: &'static str,
    efdv4: cglue::afb_evfd_t,
    fd: ::std::os::raw::c_int,
    events: u32,
    callback: Option<*mut dyn AfbEvtFdControl>,
    autounref: i32,
    autoclose: i32,
}

impl AfbEvtFd {
    pub fn new(uid: &'static str) -> &'static mut Self {
        let timer_box = Box::new(AfbEvtFd {
            uid: uid,
            fd: 0,
            info: "",
            callback: None,
            autounref: 0,
            autoclose: 0,
            efdv4: 0 as cglue::afb_evfd_t,
            events: AfbEvtFdPoll::IN.bits(),
        });
        Box::leak(timer_box)
    }

    pub fn set_info(&mut self, value: &'static str) -> &mut Self {
        self.info = value;
        self
    }

    pub fn set_fd(&mut self, sockfd: ::std::os::raw::c_int) -> &mut Self {
        self.fd = sockfd;
        self
    }

    pub fn set_autounref(&mut self, autounref: bool) -> &mut Self {
        if autounref {self.autounref = 1};
        self
    }

    pub fn set_autoclose(&mut self, autoclose: bool) -> &mut Self {
        if autoclose {self.autoclose = 1};
        self
    }

    pub fn set_events(&mut self, events: AfbEvtFdPoll) -> &mut Self {
        self.events= events.bits();
        self
    }

    pub fn set_callback(&mut self, ctrlbox: Box<dyn AfbEvtFdControl>) -> &mut Self {
        self.callback = Some(Box::leak(ctrlbox));
        self
    }

    pub fn start(&mut self) -> Result<&Self, AfbError> {
        if self.fd == 0 || self.callback == None {
            return Err(AfbError::new(
                self.uid,
                "EventFd callback must be set and fd should >0",
            ));
        }

        let status = unsafe {
            cglue::afb_evfd_create(
                &mut self.efdv4,
                self.fd,
                self.events,
                Some(api_evtfd_cb),
                self as *const _ as *mut std::ffi::c_void,
                self.autounref,
                self.autoclose,
            )
        };
        if status != 0 {
            return Err(AfbError::new(self.uid, "Afb_EvtFd creation fail"));
        }
        Ok(self)
    }

    pub fn get_uid(&self) -> &'static str {
        self.uid
    }

    pub fn unref(&self) {
        unsafe { cglue::afb_evfd_unref(self.efdv4) };
    }

    pub fn addref(&self) {
        unsafe { cglue::afb_evfd_addref(self.efdv4) };
    }

    pub fn get_fd(&self) -> i32{
        unsafe { cglue::afb_evfd_get_fd(self.efdv4) }
    }

    pub fn get_events(&self) -> u32{
        unsafe { cglue::afb_evfd_get_events(self.efdv4) }
    }

    pub fn get_info(&self) -> &'static str {
        self.info
    }

}