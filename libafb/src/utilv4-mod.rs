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

use bitflags::bitflags;
use std::collections::HashMap;
use std::ffi::CString;
use std::fmt;
use std::panic::Location;
use std::sync::{Arc, Condvar, Mutex};

const AUTOSTART: &str = "autostart";
const TIMEOUT: u32 = 5;

pub type AfbTmrV4 = cglue::afb_timer_t;

pub use crate::AfbAuthAllOf;
#[macro_export]
macro_rules! AfbAuthAllOf {
 ( $( $args:expr ),*) => {
    {
    let mut vect= Vec::new();
    $(
        vect.push(AfbPermission::from($args));
    )*
    afbv4::utilv4::AfbPermission::new(afbv4::utilv4::AfbPermission::AnyOf(vect))
    }
 };
}

pub use crate::AfbAuthAnyOf;
#[macro_export]
macro_rules! AfbAuthAnyOf {
 ( $( $args:expr ),*) => {
    {
    let mut vect= Vec::new();
    $(
        vect.push(AfbPermission::from($args));
    )*
    afbv4::utilv4::AfbPermission::new(afbv4::utilv4::AfbPermission::AllOf(vect))
    }
 };
}

pub use crate::afb_log_msg;
#[macro_export]
macro_rules! afb_log_msg {
 ( $level:tt, $handle:expr,$format:expr, $( $args:expr ),*) => {
    let dbg_info = DbgInfo {
        name: func_name!(),
        file: file!(),
        line: line!(),
        column: column!(),
    };
    if AfbLogMsg::verbosity_satisfied(AfbLogLevel::$level, $handle) {
        let message= format! ($format, $($args),*);
        AfbLogMsg::push_log (AfbLogLevel::$level, $handle, message, Some(&dbg_info))
    }
 };
 ( $level:tt, $handle:expr,$format:expr) => {
    let dbg_info = DbgInfo {
        name: func_name!(),
        file: file!(),
        line: line!(),
        column: column!(),
    };
    if AfbLogMsg::verbosity_satisfied(AfbLogLevel::$level, $handle) {
    AfbLogMsg::push_log (AfbLogLevel::$level, $handle, $format, Some(&dbg_info))
    }
 }
}

pub use crate::afb_error;
#[macro_export]
macro_rules! afb_error {
 ( $label:expr, $format:expr, $( $args:expr ),*) => {
    {
    Err(AfbError::new ($label, 0, format! ($format, $($args),*)))
    }
 };
 ( $label:expr, $format:expr) => {
    {
     Err(AfbError::new ($label, 0, $format))
    }
 }
}

pub use crate::afb_log_raw;
#[macro_export]
macro_rules! afb_log_raw {
 ( $level:tt, $handle:expr,$format:expr, $( $args:expr ),*) => {
    let format= format! ($format, $($args),*);
    AfbLogMsg::push_log (AfbLogLevel::$level, $handle, format, None)
 }
}

pub use crate::func_name;
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

pub use crate::afb_add_trace;
#[macro_export]
macro_rules! afb_add_trace {
    ($afb_error:ident) => {
        $afb_error.add_trace(func_name!(), file!(), line!(), column!())
    };
}

pub trait MakeError<T> {
    fn make(uid: &str, status: i32, msg: T, location: &'static Location<'static>) -> AfbError;
}

impl MakeError<&str> for AfbError {
    fn make(uid: &str, status: i32, msg: &str, caller: &'static Location<'static>) -> AfbError {
        AfbError {
            uid: uid.to_string(),
            info: msg.to_string(),
            status,
            dbg_info: DbgInfo {
                name: func_name!(),
                file: caller.file(),
                line: caller.line(),
                column: caller.column(),
            },
        }
    }
}

impl MakeError<String> for AfbError {
    fn make(uid: &str, status: i32, msg: String, caller: &'static Location<'static>) -> AfbError {
        AfbError {
            uid: uid.to_string(),
            status,
            info: msg,
            dbg_info: DbgInfo {
                name: func_name!(),
                file: caller.file(),
                line: caller.line(),
                column: caller.column(),
            },
        }
    }
}

pub struct DbgInfo {
    pub name: &'static str,
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
}

pub struct AfbError {
    uid: String,
    info: String,
    status: i32,
    dbg_info: DbgInfo,
}

impl AfbError {
    #[track_caller]
    pub fn new<T>(uid: &str, status: i32, msg: T) -> AfbError
    where
        AfbError: MakeError<T>,
    {
        Self::make(uid, status, msg, Location::caller())
    }
    pub fn get_uid(&self) -> String {
        self.uid.to_owned()
    }
    pub fn get_status(&self) -> i32 {
        self.status
    }
    pub fn get_info(&self) -> String {
        self.info.to_owned()
    }
    pub fn get_dbg<'a>(&'a self) -> &DbgInfo {
        &self.dbg_info
    }

    pub fn add_trace(
        &self,
        name: &'static str,
        file: &'static str,
        line: u32,
        column: u32,
    ) -> Self {
        AfbError {
            uid: self.uid.to_owned(),
            info: self.info.to_owned(),
            status: self.get_status(),
            dbg_info: DbgInfo {
                name: name,
                file: file,
                line: line,
                column: column,
            },
        }
    }

    #[track_caller]
    pub fn to_jsonc(&self) -> Result<JsoncObj, AfbError> {
        let jobject = JsoncObj::new();
        jobject.add("uid", &self.uid)?.add("info", &self.info)?;
        Ok(jobject)
    }
}

impl fmt::Display for AfbError {
    fn fmt(&self, format: &mut fmt::Formatter) -> fmt::Result {
        write!(format, "{}:{}", self.uid, self.info)
    }
}

impl fmt::Debug for AfbError {
    fn fmt(&self, format: &mut fmt::Formatter) -> fmt::Result {
        write!(format, "{}:{}", self.uid, self.info)
    }
}

pub struct AfbLogMsg {}
pub trait DoSendLog<T> {
    fn print_log(
        level: i32,
        handle: T,
        file: *const Cchar,
        line: u32,
        funcname: *const Cchar,
        format: *const Cchar,
    );
    fn get_verbosity(handle: T) -> i32;
}

impl<'a> DoSendLog<&AfbEventMsg<'a>> for AfbLogMsg {
    fn print_log(
        level: i32,
        event: &AfbEventMsg,
        file: *const Cchar,
        line: u32,
        funcname: *const Cchar,
        format: *const Cchar,
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

    fn get_verbosity(event: &AfbEventMsg) -> i32 {
        let handle = event.get_handler();
        handle.get_verbosity()
    }
}

impl<'a> DoSendLog<&AfbTimer> for AfbLogMsg {
    fn print_log(
        level: i32,
        _timer: &AfbTimer,
        file: *const Cchar,
        line: u32,
        funcname: *const Cchar,
        format: *const Cchar,
    ) {
        unsafe { cglue::afb_verbose(level, file, line as i32, funcname, format) }
    }

    fn get_verbosity(timer: &AfbTimer) -> i32 {
        timer.get_verbosity()
    }
}

impl<'a> DoSendLog<&AfbSchedJob> for AfbLogMsg {
    fn print_log(
        level: i32,
        _timer: &AfbSchedJob,
        file: *const Cchar,
        line: u32,
        funcname: *const Cchar,
        format: *const Cchar,
    ) {
        unsafe { cglue::afb_verbose(level, file, line as i32, funcname, format) }
    }

    fn get_verbosity(job: &AfbSchedJob) -> i32 {
        job.get_verbosity()
    }
}

impl<'a> DoSendLog<&AfbRequest<'a>> for AfbLogMsg {
    fn print_log(
        level: i32,
        rqt: &AfbRequest<'a>,
        file: *const Cchar,
        line: u32,
        funcname: *const Cchar,
        format: *const Cchar,
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

    fn get_verbosity(rqt: &AfbRequest) -> i32 {
        let verb = rqt.get_verb();
        verb.get_verbosity()
    }
}

impl<'a> DoSendLog<&AfbApi> for AfbLogMsg {
    fn print_log(
        level: i32,
        api: &AfbApi,
        file: *const Cchar,
        line: u32,
        funcname: *const Cchar,
        format: *const Cchar,
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

    fn get_verbosity(api: &AfbApi) -> i32 {
        api.get_verbosity()
    }
}

impl DoSendLog<Option<u32>> for AfbLogMsg {
    fn print_log(
        level: i32,
        _not_used: Option<u32>,
        file: *const Cchar,
        line: u32,
        funcname: *const Cchar,
        format: *const Cchar,
    ) {
        unsafe { cglue::afb_verbose(level, file, line as i32, funcname, format) }
    }

    fn get_verbosity(_unused: Option<u32>) -> i32 {
        255 // Fulup TBD should match binder -vvv
    }
}

impl DoSendLog<AfbRqtV4> for AfbLogMsg {
    fn print_log(
        level: i32,
        rqtv4: cglue::afb_req_t,
        file: *const Cchar,
        line: u32,
        funcname: *const Cchar,
        format: *const Cchar,
    ) {
        unsafe { cglue::afb_req_verbose(rqtv4, level, file, line as i32, funcname, format) }
    }
    fn get_verbosity(_rqt: AfbRqtV4) -> i32 {
        255 // Fulup TBD
    }
}

impl DoSendLog<&AfbEvent> for AfbLogMsg {
    fn print_log(
        level: i32,
        event: &AfbEvent,
        file: *const Cchar,
        line: u32,
        funcname: *const Cchar,
        format: *const Cchar,
    ) {
        let apiv4 = event.get_apiv4();
        unsafe { cglue::afb_api_verbose(apiv4, level, file, line as i32, funcname, format) }
    }
    fn get_verbosity(event: &AfbEvent) -> i32 {
        event.get_verbosity()
    }
}

impl DoSendLog<AfbApiV4> for AfbLogMsg {
    fn print_log(
        level: i32,
        apiv4: AfbApiV4,
        file: *const Cchar,
        line: u32,
        funcname: *const Cchar,
        format: *const Cchar,
    ) {
        unsafe { cglue::afb_api_verbose(apiv4, level, file, line as i32, funcname, format) }
    }
    fn get_verbosity(_unused: AfbApiV4) -> i32 {
        255 // Fulup TBD should match binder -vvv
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

    pub fn verbosity_satisfied<H>(level: AfbLogLevel, handle: H) -> bool
    where
        AfbLogMsg: DoSendLog<H>,
    {
        let log_level = AfbLogMsg::get_level(level);
        let verbosity = Self::get_verbosity(handle);
        verbosity >= log_level
    }

    pub fn push_log<H, T>(level: AfbLogLevel, handle: H, format: T, info: Option<&DbgInfo>)
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
                let file = 0 as *const Cchar;
                let func = 0 as *const Cchar;
                let format = CString::new(message)
                    .expect("Invalid message string")
                    .into_raw();
                Self::print_log(log_level as i32, handle, file, line, func, format);
            }
        };
    }
}

// AfbTimer callback api signature
pub type TimerCallback =
    fn(timer: &AfbTimer, decount: u32, ctx: &AfbCtxData) -> Result<(), AfbError>;
#[track_caller]
fn timer_default_cb(timer: &AfbTimer, _decount: u32, _ctx: &AfbCtxData) -> Result<(), AfbError> {
    afb_error!(
        "afb-default-cb",
        "uid:{} no timer callback defined",
        timer.get_uid()
    )
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
    let result = (timer_ref.callback)(timer_ref, decount, &timer_ref.context);

    match result {
        Ok(()) => {}
        Err(error) => {
            let dbg = error.get_dbg();
            afb_log_raw!(
                Notice,
                None,
                "{}:{} file: {}:{}:{}",
                timer_ref._uid,
                error,
                dbg.file,
                dbg.line,
                dbg.column
            );
        }
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
    callback: TimerCallback,
    context: AfbCtxData,
    decount: u32,
    period: u32,
    autounref: i32,
    verbosity: i32,
}

impl AfbTimer {
    pub fn new(uid: &'static str) -> &'static mut Self {
        let timer_box = Box::new(AfbTimer {
            _uid: uid,
            info: "",
            decount: 0,
            period: 0,
            verbosity: 255, // Fulup TBD should inherit from API
            _timerv4: 0 as cglue::afb_timer_t,
            autounref: 0,
            callback: timer_default_cb,
            context: AfbCtxData::new(AFB_NO_DATA),
        });
        Box::leak(timer_box)
    }

    pub fn set_verbosity(&mut self, value: i32) -> &mut Self {
        self.verbosity = value;
        self
    }

    pub fn get_verbosity(&self) -> i32 {
        self.verbosity
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
    pub fn set_callback(&mut self, callback: TimerCallback) -> &mut Self {
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

    #[track_caller]
    pub fn start(&mut self) -> Result<&Self, AfbError> {
        if self.period == 0 {
            return afb_error!(self._uid, "Timer period should >0",);
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
            return afb_error!(self._uid, "Afb_Timer creation fail");
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
    fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(format, "uid:{} info:{}", self._uid, self.info)
    }
}

// AfbJob callback api signature
pub type JobCallback = fn(
    job: &AfbSchedJob,
    decount: i32,
    args: &AfbCtxData,
    ctx: &AfbCtxData,
) -> Result<(), AfbError>;
#[track_caller]
fn job_default_cb(
    job: &AfbSchedJob,
    _signal: i32,
    _args: &AfbCtxData,
    _ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    afb_error!(
        "afb-default-cb",
        "uid:{} no job callback defined",
        job.get_uid()
    )
}

struct SchedJobV4 {
    job: *const AfbSchedJob,
    args: AfbCtxData,
}

#[no_mangle]
pub extern "C" fn api_schedjob_cb(signal: i32, userdata: *mut std::os::raw::c_void) {
    let handle = unsafe { Box::from_raw(userdata as *mut SchedJobV4) };
    let job_ref = unsafe { &mut *(handle.job as *mut AfbSchedJob) };
    let result = (job_ref.callback)(job_ref, signal, &handle.args, &job_ref.context);

    match result {
        Ok(()) => {}
        Err(error) => {
            let dbg = error.get_dbg();
            afb_log_raw!(
                Notice,
                None,
                "{}:{} file: {}:{}:{}",
                job_ref._uid,
                error,
                dbg.file,
                dbg.line,
                dbg.column
            );
        }
    }
}

pub struct AfbSchedJob {
    _uid: &'static str,
    _jobv4: i32,
    info: &'static str,
    group: usize,
    watchdog: i32,
    verbosity: i32,
    callback: JobCallback,
    context: AfbCtxData,
}

impl AfbSchedJob {
    pub fn new(uid: &'static str) -> &'static mut Self {
        let job_box = Box::new(AfbSchedJob {
            _uid: uid,
            _jobv4: 0,
            info: "",
            group: 0,
            watchdog: 0,
            verbosity: 255,
            callback: job_default_cb,
            context: AfbCtxData::new(AFB_NO_DATA),
        });
        Box::leak(job_box)
    }

    pub fn set_verbosity(&mut self, value: i32) -> &mut Self {
        self.verbosity = value;
        self
    }

    pub fn get_verbosity(&self) -> i32 {
        self.verbosity
    }

    pub fn set_exec_watchdog(&mut self, exec_watchdog: i32) -> &mut Self {
        self.watchdog = exec_watchdog;
        self
    }

    pub fn set_info(&mut self, info: &'static str) -> &mut Self {
        self.info = info;
        self
    }

    pub fn set_group(&mut self, group: usize) -> &mut Self {
        self.group = group;
        self
    }

    pub fn set_callback(&mut self, callback: JobCallback) -> &mut Self {
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

    pub fn get_jobid(&self) -> i32 {
        self._jobv4
    }

    pub fn get_uid(&self) -> &'static str {
        self._uid
    }

    #[track_caller]
    pub fn post<T>(&self, delay_ms: i64, args: T) -> Result<i32, AfbError>
    where
        T: 'static,
    {
        let handle = Box::into_raw(Box::new(SchedJobV4 {
            job: self as *const AfbSchedJob,
            args: AfbCtxData::new(args),
        }));

        let jobv4 = unsafe {
            cglue::afb_job_post(
                delay_ms,
                self.watchdog,
                Some(api_schedjob_cb),
                handle as *const _ as *mut std::ffi::c_void,
                self.group as *mut std::ffi::c_void,
            )
        };
        if jobv4 <= 0 {
            return afb_error!(self._uid, "Job_post launch fail");
        }
        Ok(jobv4)
    }

    pub fn get_info(&self) -> &'static str {
        self.info
    }
    #[track_caller]
    pub fn abort(&self, jobv4: i32) -> Result<(), AfbError> {
        let rc = unsafe { cglue::afb_job_abort(jobv4) };
        if rc < 0 {
            afb_error!(self._uid, "No job running id={}", jobv4)
        } else {
            Ok(())
        }
    }

    // delete job post handle and attached callback context
    pub fn terminate(&self) {
        let _ = unsafe { Box::from_raw(self as *const _ as *mut std::ffi::c_void) };
    }

    pub fn finalize(&mut self) -> &Self {
        self
    }
}

impl fmt::Display for AfbSchedJob {
    fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(format, "uid:{} info:{}", self._uid, self.info)
    }
}

pub const AFB_AUTH_DFLT_V4: *mut AfbAuthV4 = 0 as *mut AfbAuthV4;
pub type AfbAuthV4 = cglue::afb_auth;
pub struct AfbPermisionV4 {}
impl AfbPermisionV4 {
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
struct TapJobData {
    test: *const AfbTapTest,
}

fn tap_job_callback(
    job: &AfbSchedJob,
    signal: i32,
    _args: &AfbCtxData,
    ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    let context = ctx.get::<TapJobData>()?;

    let test = unsafe { &mut *(context.test as *mut AfbTapTest) };
    let suite = test.get_suite();
    let event = suite.get_event();

    let jsonc = JsoncObj::new();
    jsonc.add("index", test.index).unwrap();
    jsonc.add("test", test.uid).unwrap();
    event.push(jsonc);

    if signal == 0 {
        // subcall start
        let _ignore = test.call_sync();
    } else {
        // force a timeout response
        let no_data = [0 as cglue::afb_data_t; 0];
        let reply = AfbRqtData::new(&no_data, 0, -62);
        let response = test.check_response(reply);
        test.done(response);
    }
    job.terminate(); // jobpost is rebuilt for each test
    Ok(())
}

// AfbEvtFdControl callback api signature
pub type EvtFdCallback =
    fn(evfd: &AfbEvtFd, revents: u32, ctx: &AfbCtxData) -> Result<(), AfbError>;
#[track_caller]
fn evtfd_default_cb(evfd: &AfbEvtFd, _revents: u32, _ctx: &AfbCtxData) -> Result<(), AfbError> {
    afb_error!(
        "afb-default-cb",
        "uid:{} no evtfd callback defined",
        evfd.get_uid()
    )
}

// Afb EvtFdHandle implementation
// ------------------------
#[no_mangle]
pub extern "C" fn api_evtfd_cb(
    _efd: cglue::afb_evfd_t,
    _fd: ::std::os::raw::c_int,
    revents: u32,
    userdata: *mut ::std::os::raw::c_void,
) {
    // extract evtfd+api object from libafb internals
    let evtfd_ref = unsafe { &mut *(userdata as *mut AfbEvtFd) };

    // call evtfd calback
    let result = (evtfd_ref.callback)(evtfd_ref, revents, &evtfd_ref.context);

    match result {
        Ok(()) => {}
        Err(error) => {
            let dbg = error.get_dbg();
            afb_log_raw!(
                Notice,
                None,
                "{}:{} file: {}:{}:{}",
                evtfd_ref.uid,
                error,
                dbg.file,
                dbg.line,
                dbg.column
            );
        }
    }

    // clean callback control box
    if revents == AfbEvtFdPoll::HUP.bits() {
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
        const RUP= cglue::afb_epoll_epoll_RDH;
        const ALL= 0xffff;
    }
}

// Event FD add a filedescriptor to mainloop and connect a callback
pub struct AfbEvtFd {
    uid: &'static str,
    info: &'static str,
    efdv4: cglue::afb_evfd_t,
    fd: ::std::os::raw::c_int,
    events: u32,
    callback: EvtFdCallback,
    context: AfbCtxData,
    autounref: i32,
    autoclose: i32,
}

impl AfbEvtFd {
    pub fn new(uid: &'static str) -> &'static mut Self {
        let evt_box = Box::new(AfbEvtFd {
            uid: uid,
            fd: 0,
            info: "",
            callback: evtfd_default_cb,
            context: AfbCtxData::new(AFB_NO_DATA),
            autounref: 0,
            autoclose: 0,
            efdv4: 0 as cglue::afb_evfd_t,
            events: AfbEvtFdPoll::IN.bits(),
        });
        Box::leak(evt_box)
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
        if autounref {
            self.autounref = 1
        };
        self
    }

    pub fn set_autoclose(&mut self, autoclose: bool) -> &mut Self {
        if autoclose {
            self.autoclose = 1
        };
        self
    }

    pub fn set_events(&mut self, events: AfbEvtFdPoll) -> &mut Self {
        self.events = events.bits();
        self
    }

    pub fn set_callback(&mut self, callback: EvtFdCallback) -> &mut Self {
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

    #[track_caller]
    pub fn start(&mut self) -> Result<&Self, AfbError> {
        if self.fd == 0 {
            return afb_error!(self.uid, "EventFd fd should >0",);
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
            return afb_error!(self.uid, "Afb_EvtFd creation fail");
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

    pub fn get_fd(&self) -> i32 {
        unsafe { cglue::afb_evfd_get_fd(self.efdv4) }
    }

    pub fn get_events(&self) -> u32 {
        unsafe { cglue::afb_evfd_get_events(self.efdv4) }
    }

    pub fn get_info(&self) -> &'static str {
        self.info
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
    pub expect: Vec<JsoncObj>,
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

    #[track_caller]
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

    #[track_caller]
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
        T: Into<JsoncObj>,
    {
        let jsonc = data.into();
        self.expect.push(jsonc);
        self
    }

    pub fn get_group(&self) -> &AfbTapGroup {
        unsafe { &*(self.group as *mut AfbTapGroup) }
    }

    pub fn get_suite(&self) -> &AfbTapSuite {
        let group = unsafe { &*(self.group as *mut AfbTapGroup) };
        group.get_suite()
    }

    fn check_response(&self, reply: AfbRqtData) -> AfbTapResponse {
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
            match reply.get::<JsoncObj>(idx) {
                // expect argument as no jsonc representation.
                Err(error) => {
                    let msg = error.to_jsonc().unwrap().to_string();
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
                            afb_log_msg!(Warning, api, "{} -> {} NotIn {}", error, jexpect, jvalue);
                            return AfbTapResponse {
                                status: AFB_FAIL,
                                diagnostic: error.to_string(),
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
            // Err(error) => AfbTapResponse {
            //     status: error.get_status(),
            //     diagnostic: error.to_string(),
            // }
            Err(error) => {
                let no_data = [0 as cglue::afb_data_t; 0];
                let response = AfbRqtData::new(&no_data, 0, error.get_status());
                self.check_response(response)
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
    #[track_caller]
    pub fn jobpost(&mut self) -> Result<(), AfbError> {
        let semaphore = Arc::clone(&self.semaphore);
        let (lock, cvar) = &*semaphore;
        let mut done = match lock.lock() {
            Err(_error) => {
                return afb_error!("fail-group-wait", "fail waiting on tap group={}", self.uid)
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
            .set_callback(tap_job_callback)
            .set_context(TapJobData { test: self })
            .post(self.delay as i64, AFB_NO_DATA)
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

    pub fn done(&mut self, response: AfbTapResponse) {
        self.response = Some(response);
        let semaphore = Arc::clone(&self.semaphore);
        let (lock, cvar) = &*semaphore;
        let mut done = lock.lock().unwrap();
        *done = 0;
        cvar.notify_one();
    }

    pub fn get_report(&self) -> JsoncObj {
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
        JsoncObj::from(msg.as_str())
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
    #[track_caller]
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
            .set_callback(tap_test_callback)
            .set_context(TapTestData { test: test })
            .set_usage("no input")
            .finalize()
            .unwrap();

        let api_group = unsafe { &mut *(self.api_group as *mut AfbGroup) };
        api_group.add_verb(verb);
        self
    }

    // return suite test until group end
    pub fn get_test(&self, index: usize) -> Option<&mut AfbTapTest> {
        if self.tests.len() <= index {
            None
        } else {
            let test = unsafe { &mut *(self.tests[index] as *mut AfbTapTest) };
            Some(test)
        }
    }

    pub fn get_suite(&self) -> &AfbTapSuite {
        unsafe { &*(self.suite as *const _ as *mut AfbTapSuite) }
    }
    #[track_caller]
    pub fn launch(&self) -> Result<(), AfbError> {
        // get group 1st test
        let test = match self.get_test(0) {
            None => return afb_error!(self.uid, "no-test-found"),
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

    pub fn get_report(&self) -> JsoncObj {
        let jsonc = JsoncObj::array();
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
        verb.set_callback(tap_group_callback)
            .set_context(vcbdata)
            .set_info(group.info)
            .set_usage("no_input")
            .register(api.get_apiv4(), AFB_NO_AUTH);
        api.add_verb(verb.finalize().unwrap());

        let api_group = unsafe { &mut *(group.api_group as *mut AfbGroup) };
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
        unsafe { &*(self.tap_api as *mut AfbApi) }
    }

    pub fn get_uid(&self) -> &'static str {
        self.uid
    }

    pub fn get_event(&self) -> &AfbEvent {
        unsafe { &*(self.event as *mut AfbEvent) }
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
    #[track_caller]
    // launch a group and return report as jsonc
    pub fn launch(&self, label: &'static str) -> Result<(), AfbError> {
        match self.get_group(label) {
            None => afb_error!("group-label-not-found", label),
            Some(group) => group.launch(),
        }
    }
    #[track_caller]
    pub fn finalize(&'static mut self) -> Result<(), AfbError> {
        let api = unsafe { &mut *(self.tap_api as *mut AfbApi) };
        let vcbdata = TapGroupData {
            group: self.autostart,
        };

        // add auto start group verbs
        let autostart_tap = unsafe { &mut *(self.autostart as *mut AfbTapGroup) };
        let autostart_afb = unsafe { &mut *(autostart_tap.api_group as *mut AfbGroup) };
        autostart_afb.register(api.get_apiv4(), AFB_NO_AUTH);
        api.add_group(autostart_afb);

        let verb = AfbVerb::new(AUTOSTART);
        verb.set_callback(tap_group_callback)
            .set_context(vcbdata)
            .set_info("default tap autostart group")
            .set_usage("no_input")
            .register(api.get_apiv4(), AFB_NO_AUTH);
        api.add_verb(verb.finalize()?);

        // seal tap test api
        unsafe { cglue::afb_api_seal(api.get_apiv4()) };

        if self.autorun {
            match AfbSchedJob::new(self.uid)
                .set_callback(tap_suite_callback)
                .set_context(TapSuiteAutoRun { suite: self })
                .post(0, AFB_NO_DATA)
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

    pub fn get_report(&'static mut self) -> JsoncObj {
        let autostart = unsafe { &mut *(self.autostart) };
        let jreport = JsoncObj::new();
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
                            afb_log_msg!(Critical, self.get_api().get_apiv4(), error.to_string());
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

fn tap_suite_callback(
    job: &AfbSchedJob,
    _signal: i32,
    _args: &AfbCtxData,
    ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    let context = ctx.get::<TapSuiteAutoRun>()?;
    let suite = unsafe { &mut *(context.suite as *mut AfbTapSuite) };
    let autostart = unsafe { &mut *(suite.autostart) };

    match autostart.launch() {
        Err(error) => {
            afb_log_raw!(
                Critical,
                suite.get_api().get_apiv4(),
                "Test fail {}:autostart error={}",
                suite.get_uid(),
                error
            );
        }
        Ok(()) => {}
    }

    let autoexit = suite.get_autoexit();
    suite.get_report();
    job.terminate();

    if autoexit {
        std::process::exit(0);
    }
    Ok(())
}

// implement TapTest API callback
struct TapTestData {
    test: *mut AfbTapTest,
}
#[track_caller]
fn tap_test_callback(
    rqt: &AfbRequest,
    _args: &AfbRqtData,
    ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    let context = ctx.get::<TapTestData>()?;

    // bypass Rust limitation that refuses to understand static object pointers
    let test = unsafe { &mut (*context.test) };
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
    Ok(())
}

// implement TapGroup API callback
struct TapGroupData {
    group: *mut AfbTapGroup,
}

#[track_caller]
fn tap_group_callback(
    rqt: &AfbRequest,
    _args: &AfbRqtData,
    ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    let context = ctx.get::<TapGroupData>()?;
    // bypass Rust limitation that refuses to understand static object pointers
    let group = unsafe { &mut (*context.group) };
    let suite = unsafe { &mut (*group.suite) };
    let event = unsafe { &mut (*suite.event) };

    event.subscribe(rqt)?;

    match group.launch() {
        Err(error) => {
            afb_log_msg!(Error, rqt, "fail to launch test error={}", error);
            rqt.reply(error, 405);
        }
        Ok(_jreport) => {
            rqt.reply(group.get_report(), 0);
        }
    }
    Ok(())
}
