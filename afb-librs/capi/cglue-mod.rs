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

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(clippy::ptr_offset_with_cast)]
#![allow(clippy::type_complexity)]
#![allow(clippy::useless_transmute)]
include!("_libafb-map.rs");

// hack to force RUST to export afbBinding mandatory entry points
#[export_name = "afbBindingV4r1_itfptr"]
pub static mut rustBindingV4r1_itfptr: usize = 0;

#[export_name = "afbBindingV4root"]
pub static mut rustBindingV4root: usize = 0;

#[export_name = "afbBindingV4_itf_revision"]
pub static mut rustBindingV4_itf_revision: u16 = afbBindingV4_itf_revision as u16; // => AFB_BINDING_X4R1_ITF_REVISION

pub const JSON_C_TO_STRING_PLAIN: u32 = 0;
pub const JSON_C_TO_STRING_SPACED: u32 = 1;
pub const JSON_C_TO_STRING_PRETTY: u32 = 2;
pub const JSON_C_TO_STRING_PRETTY_TAB: u32 = 8;
pub const JSON_C_TO_STRING_NOZERO: u32 = 4;
pub const JSON_C_TO_STRING_NOSLASHESCAPE: u32 = 16;

pub type json_tokener_srec = ::std::os::raw::c_int;
pub type printbuf = ::std::os::raw::c_int;
pub type json_tokener = ::std::os::raw::c_int;

include!("_jsonc-map.rs");
