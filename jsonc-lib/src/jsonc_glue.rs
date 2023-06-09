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

mod cglue {

    #![allow(dead_code)]
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)] 

    pub const JSON_C_TO_STRING_PLAIN: u32 = 0;
    pub const JSON_C_TO_STRING_SPACED: u32 = 1;
    pub const JSON_C_TO_STRING_PRETTY: u32 = 2;
    pub const JSON_C_TO_STRING_PRETTY_TAB: u32 = 8;
    pub const JSON_C_TO_STRING_NOZERO: u32 = 4;
    pub const JSON_C_TO_STRING_NOSLASHESCAPE: u32 = 16;

    pub type json_tokener_srec = ::std::os::raw::c_int;
    pub type printbuf = ::std::os::raw::c_int;
    pub type json_tokener = ::std::os::raw::c_int;

    include!("./jsonc_map.rs");
}