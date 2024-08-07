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

#![doc(html_logo_url = "https://iot.bzh/images/defaults/company/512-479-max-transp.png", html_favicon_url = "https://iot.bzh/images/defaults/favicon.ico")]
extern crate bitflags;

#[cfg(test)]
#[path = "../test/jsonc-test.rs"]
mod test;

// cglue is exported as private
#[path = "../capi/cglue-mod.rs"]
mod cglue;

#[path = "./apiv4-mod.rs"]
pub mod apiv4;

#[path = "./datav4-mod.rs"]
pub mod datav4;

#[path = "./jsonc-mod.rs"]
pub mod jsonc;

#[path = "./utilv4-mod.rs"]
pub mod utilv4;

#[path = "./testv4-mod.rs"]
pub mod testv4;

pub mod prelude {
    pub(crate) use crate::cglue;
    pub use crate::jsonc::*;
    pub use crate::apiv4::*;
    pub use crate::datav4::*;
    pub use crate::utilv4::*;
    pub use crate::testv4::*;
    pub use std::any::Any;
}
