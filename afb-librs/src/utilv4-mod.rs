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

use ::std::os::raw;
use bitflags::bitflags;
use std::ffi::CStr;
use std::ffi::CString;
use std::fmt;
use std::panic::Location;

const MAX_ERROR_LEN: usize = 256;
pub fn get_perror() -> String {
    get_strerror(unsafe { *cglue::__errno_location() })
}

pub fn get_strerror(code: i32) -> String {
    let mut buffer = [0 as ::std::os::raw::c_char; MAX_ERROR_LEN];
    unsafe { cglue::strerror_r(code, &mut buffer as *mut raw::c_char, MAX_ERROR_LEN) };
    let cstring = unsafe { CStr::from_ptr(&mut buffer as *const raw::c_char) };
    let slice: &str = cstring.to_str().unwrap();
    slice.to_owned()
}

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

#[derive(Clone)]
pub struct DbgInfo {
    pub name: &'static str,
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
}

#[derive(Clone)]
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
    pub fn get_dbg(&self) -> &DbgInfo {
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
                name,
                file,
                line,
                column,
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
        write!(
            format,
            "{}:{} {}:{}:{}",
            self.uid, self.info, self.dbg_info.file, self.dbg_info.line, self.dbg_info.column
        )
    }
}

// convert syslog (0-7) level to Afb verbosity mask model
pub fn verbosity_to_mask(level: i32) -> Result<u32, AfbError> {
    let mut mask = AfbLogLevel::from_syslog_level(level.unsigned_abs())? as u32;
    if level > 0 {
        for sys_level in [1, 2, 3, 4, 5, 6, 7] {
            if level as u32 >= mask {
                mask |= AfbLogLevel::from_syslog_level(sys_level)? as u32;
            } else {
                break;
            }
        }
    }
    Ok(mask)
}

pub struct AfbLogMsg {}
pub trait DoSendLog<T> {
    /// # Safety
    /// - `file`, `funcname` et `format` doivent pointer vers des C-strings valides et
    ///   null-terminées vivant au moins le temps de l'appel.
    /// - `handle` doit être un handle valide pour le backend concerné.
    /// - Cette fonction appelle des APIs C et peut déréférencer des pointeurs bruts.
    unsafe fn print_log(
        level: i32,
        handle: T,
        file: *const Cchar,
        line: u32,
        funcname: *const Cchar,
        format: *const Cchar,
    );
    /// # Safety
    /// - `handle` doit être un handle valide ; l’implémentation peut appeler du code C.
    unsafe fn get_verbosity(handle: T) -> u32;
}

impl<'a> DoSendLog<&AfbEventMsg<'a>> for AfbLogMsg {
    unsafe fn print_log(
        level: i32,
        event: &AfbEventMsg,
        file: *const Cchar,
        line: u32,
        funcname: *const Cchar,
        format: *const Cchar,
    ) {
        cglue::afb_api_verbose(
            (*event).get_api().get_apiv4(),
            level,
            file,
            line as i32,
            funcname,
            format,
        )
    }

    unsafe fn get_verbosity(event: &AfbEventMsg) -> u32 {
        let handle = event.get_handler();
        handle.get_verbosity()
    }
}

impl DoSendLog<&AfbTimer> for AfbLogMsg {
    unsafe fn print_log(
        level: i32,
        _timer: &AfbTimer,
        file: *const Cchar,
        line: u32,
        funcname: *const Cchar,
        format: *const Cchar,
    ) {
        cglue::afb_verbose(level, file, line as i32, funcname, format)
    }

    unsafe fn get_verbosity(timer: &AfbTimer) -> u32 {
        timer.get_verbosity()
    }
}

impl DoSendLog<&AfbSchedJob> for AfbLogMsg {
    unsafe fn print_log(
        level: i32,
        _timer: &AfbSchedJob,
        file: *const Cchar,
        line: u32,
        funcname: *const Cchar,
        format: *const Cchar,
    ) {
        cglue::afb_verbose(level, file, line as i32, funcname, format)
    }

    unsafe fn get_verbosity(job: &AfbSchedJob) -> u32 {
        job.get_verbosity()
    }
}

impl DoSendLog<&AfbRequest> for AfbLogMsg {
    unsafe fn print_log(
        level: i32,
        rqt: &AfbRequest,
        file: *const Cchar,
        line: u32,
        funcname: *const Cchar,
        format: *const Cchar,
    ) {
        cglue::afb_req_verbose(
            (*rqt).get_rqtv4(),
            level,
            file,
            line as i32,
            funcname,
            format,
        )
    }

    unsafe fn get_verbosity(rqt: &AfbRequest) -> u32 {
        let verb = rqt.get_verb();
        verb.get_verbosity(rqt)
    }
}

impl DoSendLog<&AfbApi> for AfbLogMsg {
    unsafe fn print_log(
        level: i32,
        api: &AfbApi,
        file: *const Cchar,
        line: u32,
        funcname: *const Cchar,
        format: *const Cchar,
    ) {
        cglue::afb_api_verbose(
            (*api).get_apiv4(),
            level,
            file,
            line as i32,
            funcname,
            format,
        )
    }

    unsafe fn get_verbosity(api: &AfbApi) -> u32 {
        api.get_verbosity()
    }
}

impl DoSendLog<Option<u32>> for AfbLogMsg {
    unsafe fn print_log(
        level: i32,
        _not_used: Option<u32>,
        file: *const Cchar,
        line: u32,
        funcname: *const Cchar,
        format: *const Cchar,
    ) {
        cglue::afb_verbose(level, file, line as i32, funcname, format)
    }

    unsafe fn get_verbosity(_unused: Option<u32>) -> u32 {
        255 // Fulup TBD should match binder -vvv
    }
}

impl DoSendLog<AfbRqtV4> for AfbLogMsg {
    unsafe fn print_log(
        level: i32,
        rqtv4: cglue::afb_req_t,
        file: *const Cchar,
        line: u32,
        funcname: *const Cchar,
        format: *const Cchar,
    ) {
        cglue::afb_req_verbose(rqtv4, level, file, line as i32, funcname, format)
    }

    unsafe fn get_verbosity(rqt: AfbRqtV4) -> u32 {
        cglue::afb_req_logmask(rqt) as u32
    }
}

impl DoSendLog<&AfbEvent> for AfbLogMsg {
    unsafe fn print_log(
        level: i32,
        event: &AfbEvent,
        file: *const Cchar,
        line: u32,
        funcname: *const Cchar,
        format: *const Cchar,
    ) {
        let apiv4 = event.get_apiv4();
        cglue::afb_api_verbose(apiv4, level, file, line as i32, funcname, format)
    }

    unsafe fn get_verbosity(event: &AfbEvent) -> u32 {
        event.get_verbosity()
    }
}

impl DoSendLog<AfbApiV4> for AfbLogMsg {
    unsafe fn print_log(
        level: i32,
        apiv4: AfbApiV4,
        file: *const Cchar,
        line: u32,
        funcname: *const Cchar,
        format: *const Cchar,
    ) {
        cglue::afb_api_verbose(apiv4, level, file, line as i32, funcname, format)
    }

    unsafe fn get_verbosity(apiv4: AfbApiV4) -> u32 {
        cglue::afb_api_logmask(apiv4) as u32
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

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum AfbLogLevel {
    Emergency = 1,
    Alert = 2,
    Critical = 4,
    Error = 8,
    Warning = 16,
    Notice = 32,
    Info = 64,
    Debug = 128,
}

impl AfbLogLevel {
    pub fn to_afb_mask(&self) -> u32 {
        *self as u32
    }

    pub fn from_syslog_level(level: u32) -> Result<AfbLogLevel, AfbError> {
        let level = match level {
            cglue::afb_syslog_levels_AFB_SYSLOG_LEVEL_ERROR => AfbLogLevel::Error,
            cglue::afb_syslog_levels_AFB_SYSLOG_LEVEL_DEBUG => AfbLogLevel::Debug,
            cglue::afb_syslog_levels_AFB_SYSLOG_LEVEL_NOTICE => AfbLogLevel::Notice,
            cglue::afb_syslog_levels_AFB_SYSLOG_LEVEL_CRITICAL => AfbLogLevel::Critical,
            cglue::afb_syslog_levels_AFB_SYSLOG_LEVEL_WARNING => AfbLogLevel::Warning,
            cglue::afb_syslog_levels_AFB_SYSLOG_LEVEL_EMERGENCY => AfbLogLevel::Emergency,
            cglue::afb_syslog_levels_AFB_SYSLOG_LEVEL_INFO => AfbLogLevel::Info,
            cglue::afb_syslog_levels_AFB_SYSLOG_LEVEL_ALERT => AfbLogLevel::Alert,
            _ => return afb_error!("afb-log-level", "invalid level:{} should be 0-7", level),
        };
        Ok(level)
    }

    pub fn to_syslog_level(&self) -> u32 {
        match self {
            AfbLogLevel::Error => cglue::afb_syslog_levels_AFB_SYSLOG_LEVEL_ERROR,
            AfbLogLevel::Debug => cglue::afb_syslog_levels_AFB_SYSLOG_LEVEL_DEBUG,
            AfbLogLevel::Notice => cglue::afb_syslog_levels_AFB_SYSLOG_LEVEL_NOTICE,
            AfbLogLevel::Critical => cglue::afb_syslog_levels_AFB_SYSLOG_LEVEL_CRITICAL,
            AfbLogLevel::Warning => cglue::afb_syslog_levels_AFB_SYSLOG_LEVEL_WARNING,
            AfbLogLevel::Emergency => cglue::afb_syslog_levels_AFB_SYSLOG_LEVEL_EMERGENCY,
            AfbLogLevel::Info => cglue::afb_syslog_levels_AFB_SYSLOG_LEVEL_INFO,
            AfbLogLevel::Alert => cglue::afb_syslog_levels_AFB_SYSLOG_LEVEL_ALERT,
        }
    }
}

impl AfbLogMsg {
    pub fn verbosity_satisfied<H>(level: AfbLogLevel, handle: H) -> bool
    where
        AfbLogMsg: DoSendLog<H>,
    {
        let log_level = level as u32;
        let verbosity = unsafe { <Self as DoSendLog<H>>::get_verbosity(handle) };
        (verbosity & log_level) != 0
    }

    pub fn push_log<H, T>(level: AfbLogLevel, handle: H, format: T, info: Option<&DbgInfo>)
    where
        AfbLogMsg: DoMessage<T>,
        AfbLogMsg: DoSendLog<H>,
    {
        let log_level = level.to_syslog_level();
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
                unsafe {
                    <Self as DoSendLog<H>>::print_log(
                        log_level as i32,
                        handle,
                        file,
                        line,
                        func,
                        format,
                    );
                }
            }
            None => {
                let line = 0;
                let file = std::ptr::null::<Cchar>();
                let func = std::ptr::null::<Cchar>();
                let format = CString::new(message)
                    .expect("Invalid message string")
                    .into_raw();
                unsafe {
                    <Self as DoSendLog<H>>::print_log(
                        log_level as i32,
                        handle,
                        file,
                        line,
                        func,
                        format,
                    );
                }
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
/// Timer callback invoked by the C AFB runtime.
///
/// # Safety
/// - This function is called from C (FFI). The `userdata` pointer must either be null
///   or point to a valid value of the expected type, and it must remain valid for the
///   whole duration of this call.
/// - `timer` is owned/managed by the runtime: do **not** free/close it here. Only call
///   functions that the AFB API documents as safe from a timer context.
/// - This function must **not** panic or unwind across the FFI boundary.
/// - Avoid long blocking operations and re-entrancy unless the AFB API explicitely
///   allows it; keep the handler short and non-blocking.
/// - `decount` is provided by the runtime; don’t assume it is > 0 and handle it
///   defensively.
#[no_mangle]
pub unsafe extern "C" fn api_timers_cb(
    _timer: cglue::afb_timer_t,
    userdata: *mut std::os::raw::c_void,
    decount: u32,
) {
    // extract timer+api object from libafb internals
    let timer_ref = &mut *(userdata as *mut AfbTimer);
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
    verbosity: u32,
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

    pub fn set_verbosity(&mut self, value: i32) -> Result<&mut Self, AfbError> {
        self.verbosity = verbosity_to_mask(value)?;
        Ok(self)
    }

    pub fn get_verbosity(&self) -> u32 {
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

/// Job scheduler callback invoked by the AFB runtime.
///
/// # Safety
/// - This function is invoked by the C/AFB runtime; **do not call it directly** from safe Rust.
/// - `userdata` must be a valid, properly aligned pointer to the expected context type and must
///   remain valid for the entire duration of the call.
/// - The pointed-to value must not be aliased mutably elsewhere while this function runs.
/// - `signal` must be the value provided by the scheduler (e.g. 0 or a valid POSIX signal code);
///   passing arbitrary values is undefined behaviour.
/// - All invariants required by the surrounding FFI/libafb code must be preserved.
#[no_mangle]
pub unsafe extern "C" fn api_schedjob_cb(signal: i32, userdata: *mut std::os::raw::c_void) {
    let handle = Box::from_raw(userdata as *mut SchedJobV4);
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
    info: &'static str,
    group: usize,
    watchdog: i32,
    verbosity: u32,
    callback: JobCallback,
    context: AfbCtxData,
}

impl AfbSchedJob {
    pub fn new(uid: &'static str) -> &'static mut Self {
        let job_box = Box::new(AfbSchedJob {
            _uid: uid,
            info: "",
            group: 0,
            watchdog: 0,
            verbosity: 0,
            callback: job_default_cb,
            context: AfbCtxData::new(AFB_NO_DATA),
        });
        Box::leak(job_box)
    }

    pub fn set_verbosity(&mut self, value: i32) -> Result<&mut Self, AfbError> {
        self.verbosity = verbosity_to_mask(value)?;
        Ok(self)
    }

    pub fn get_verbosity(&self) -> u32 {
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
        let _ = unsafe { Box::from_raw(self as *const _ as *mut AfbSchedJob) };
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

pub const AFB_AUTH_DFLT_V4: *mut AfbAuthV4 = std::ptr::null_mut::<AfbAuthV4>();
pub type AfbAuthV4 = cglue::afb_auth;
pub struct AfbPermisionV4 {}
impl AfbPermisionV4 {
    #[allow(clippy::new_ret_no_self)]
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
                            first: AfbPermisionV4::new(slot, AFB_AUTH_DFLT_V4),
                        },
                        next,
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
                            first: AfbPermisionV4::new(slot, AFB_AUTH_DFLT_V4),
                        },
                        next,
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
        if !(-7..=7).contains(&value) {
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
/// Event-FD callback invoked by the C AFB runtime.
///
/// # Safety
/// - This function is called from C (FFI). `userdata` must be either null or a valid
///   pointer to the expected type and remain valid for the duration of the call.
/// - `efd` is owned/managed by the runtime: do **not** free/close/unref it here unless
///   the AFB API explicitly allows it in an event-fd callback.
/// - Do **not** panic or unwind across the FFI boundary.
/// - Keep the handler short and non-blocking; avoid long blocking operations and
///   re-entrancy unless the AFB API states it is safe in this context.
/// - `revents` comes from the kernel/poller and may contain combinations of flags; treat
///   it defensively and validate before acting.
#[no_mangle]
pub unsafe extern "C" fn api_evtfd_cb(
    efd: cglue::afb_evfd_t,
    _fd: ::std::os::raw::c_int,
    revents: u32,
    userdata: *mut ::std::os::raw::c_void,
) {
    // extract evtfd+api object from libafb internals
    let evtfd_ref = &mut *(userdata as *mut AfbEvtFd);

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
    if (revents & AfbEvtFdPoll::RUP.bits()) != 0 || (revents & AfbEvtFdPoll::HUP.bits()) != 0 {
        let _ctrlbox = Box::from_raw(evtfd_ref);
        cglue::afb_evfd_unref(efd);
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
            uid,
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
                0, //self.autounref done by rust callback
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
