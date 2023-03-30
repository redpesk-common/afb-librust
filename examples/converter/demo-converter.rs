/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk samples code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 */

#![doc(
    html_logo_url = "https://iot.bzh/images/defaults/company/512-479-max-transp.png",
    html_favicon_url = "https://iot.bzh/images/defaults/favicon.ico"
)]
extern crate jsonc;
extern crate libafb;
extern crate serde;
libafb::AfbModImport!();

// automatically generate json encoder/decoder for MySimpleData
AfbDataConverter!(simple_data, MySimpleData);
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MySimpleData {
    pub name: String,
    pub x: i32,
    pub y: i32,
}

pub fn register(binding: AfbApiV4) -> Result<(),AfbError> {
    // Custom type should be registered at binding startup time
    match simple_data::register() {
        Err(error) => {
            afb_log_msg!(
                Critical,
                binding,
                "fail to register converter error={}",
                error
            );
            Err(error)
        },
        Ok(_value) => Ok({}),
    }
}
