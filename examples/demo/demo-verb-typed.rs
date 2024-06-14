/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk samples code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 */

// import libafb dependencies
use afbv4::prelude::*;

// mySimpleData type is within an external crate to allow sharing with other crate/binding/binder
extern crate demo_converter;
use self::demo_converter::MySimpleData;

fn typed_callback(request: &AfbRequest, args: &AfbRqtData, _ctx: &AfbCtxData) ->Result <(), AfbError>{
    // check arg0 match MySimpleData grammar
    let input = args.get::<&MySimpleData>(0)?;

    // create a sample simple-data object as response
    let output = MySimpleData {
        name: input.name.to_uppercase(),
        x: input.x + 1,
        y: input.y - 1,
    };

    // closure is call from following 'if let' with reply()
    let reply = || -> Result<(), AfbError> {
        let mut response = AfbParams::new();
        response.push(output)?;
        request.reply(response, 0);
        Ok(())
    };

    // if data export fail send an error report
    if let Err(error) = reply() {
        request.reply(afb_add_trace!(error), 405);
    }
    Ok(())
}

pub fn register(rootv4: AfbApiV4) -> Result<&'static AfbVerb, AfbError> {

    // custom type should register once per binder
    demo_converter::register(rootv4).expect("must register custom type");

    // build verb name from Rust module name
    let mod_name = module_path!().split(':').last().unwrap();
    afb_log_msg!(Notice, rootv4, "Registering verb={}", mod_name);

    let group= AfbVerb::new(mod_name)
        .set_callback(typed_callback)
        .set_info("My 2nd demo verb")
        .set_usage("any json string")
        .add_sample("{'x': 1, 'y':99, 'name':'IoT.bzh'}")?
        .finalize()?;

    Ok(group)
}

