// for test run 'clear && cargo test jsonc'
// ----------------------------------------
// start test => cargo test --lib -- --exact

use crate::prelude::*;

// ------------------------------------------------
// testing jsonc parsing from string
// -------------------------------------------------
#[test]
fn parse_json() {
    let token = "{'a':1,'b':2}";
    let jvalue = JsoncObj::parse(token).unwrap();

    let a = jvalue.get::<u32>("a").unwrap();
    let b = jvalue.get::<u32>("b").unwrap();

    assert!(a + a == b)
}

#[test]
fn equal_full_json() -> Result <(), AfbError> {
    let query = "{
                'transfer_mode':'dc_extended',
                'dc_evparam':{
                  'status':{
                    'ready':true,
                    'error':'no_error',
                    'evress_soc':57
                  },
                  'max_voltage':{
                    'value':4230,
                    'multiplier':-1,
                    'unit':'volt'
                  },
                  'max_current':{
                    'value':5490,
                    'multiplier':-1,
                    'unit':'ampere'
                  },
                  'max_power':{
                    'value':2320,
                    'multiplier':2,
                    'unit':'watt'
                  }
                },
                'tagid':'param_discovery_req',
                'proto':'din',
                'msgid':17
              }";

    let jquery = JsoncObj::parse(query).unwrap();
    let jexpect = JsoncObj::parse(query).unwrap();

    jquery.equal ("tst_equal_full", jexpect, Jequal::Full )?;
    Ok(())

}

#[test]
fn equal_partial_json() -> Result <(), AfbError> {
    let query = "{
                'transfer_mode':'dc_extended',
                'dc_evparam':{
                  'status':{
                    'ready':true,
                    'error':'no_error',
                    'evress_soc':57
                  },
                  'max_voltage':{
                    'value':4230,
                    'multiplier':-1,
                    'unit':'volt'
                  },
                  'max_current':{
                    'value':5490,
                    'multiplier':-1,
                    'unit':'ampere'
                  },
                  'max_power':{
                    'value':2320,
                    'multiplier':2,
                    'unit':'watt'
                  }
                },
                'tagid':'param_discovery_req',
                'proto':'din',
                'msgid':17
              }";
    let expect = "{
                'transfer_mode':'dc_extended',
                'dc_evparam':{
                  'status':{
                    'ready':true,
                    'error':'no_error',
                    'evress_soc':57
                  },
                  'max_current':{
                    'value':5490,
                    'multiplier':-1,
                    'unit':'ampere'
                  },
                },
                'tagid':'param_discovery_req',
              }";

    let jquery = JsoncObj::parse(query).unwrap();
    let jexpect = JsoncObj::parse(expect).unwrap();


    if let Ok(_) = jquery.equal ("tst_equal_fail", jexpect.clone(), Jequal::Full) {
        return afb_error!("tst_equal_fail", "this test should have fail")
    }


    jquery.equal ("tst_equal_partial", jexpect.clone(), Jequal::Partial )?;
    Ok(())

}

#[test]
fn parse_hexa() {
    let token = "[00,7d,fa,07,5e,4a]";
    let mut buffer = [0u8; 6];
    hexa_to_bytes(token, &mut buffer).unwrap();
    let hexa = bytes_to_hexa(&buffer);

    assert!(hexa == token)
}

// ------------------------------------------------
// testing object creation from rust type
// -------------------------------------------------
#[test]
fn new_json() {
    let jsonc = JsoncObj::new();
    assert!(jsonc.is_type(Jtype::Object), "object invalid");

    let value = 4;
    let jsonc = JsoncObj::import(value).unwrap();
    assert!(jsonc.is_type(Jtype::Int), "object not an int");

    let value = 123.456;
    let jsonc = JsoncObj::import(value).unwrap();
    assert!(jsonc.is_type(Jtype::Float), "object not a float");

    let value = "toto titi tata";
    let jsonc = JsoncObj::import(value).unwrap();
    assert!(jsonc.is_type(Jtype::String), "object not a string");
}

// ------------------------------------------------
// testing object creation
// -------------------------------------------------
#[test]
fn add_object() {
    let jsonc = JsoncObj::new();
    assert!(jsonc.is_type(Jtype::Object), "object not a jsonc object");
    assert!(jsonc.count().unwrap() == 0, "object not empty");

    jsonc.add("slot1", 123).unwrap();
    assert!(jsonc.count().unwrap() == 1, "object count != 1");

    jsonc.add("slot2", 123.456).unwrap();
    assert!(jsonc.count().unwrap() == 2, "objectcount != 2");

    jsonc.add("slot3", "toto").unwrap();
    assert!(jsonc.count().unwrap() == 3, "object count != 3");

    // adding an object may fail is target is not an object
    let jobject = JsoncObj::parse("{'a':1,'b':2}");
    assert!(jobject.is_ok(), "Fail to parse jsonc string");
    let result = jsonc.add("slot4", jobject.unwrap());
    assert!(result.is_ok(), "Fail to add slot4");
    assert!(jsonc.count().unwrap() == 4, "object count != 4");

    match jsonc.get_type() {
        Jtype::Array => println!("jsonc is array"),
        Jtype::Object => println!("jsonc is object"),
        _ => panic!("Hoop: unknown jtype"),
    }
}

// ------------------------------------------------
// testing array
// -------------------------------------------------
#[test]
fn insert_array() {
    let jsonc = JsoncObj::array();
    assert!(jsonc.is_type(Jtype::Array), "object not a jsonc array");
    assert!(jsonc.count().unwrap() == 0, "object count != 0");

    jsonc.append(123).unwrap();
    assert!(jsonc.count().unwrap() == 1, "object count != 1");

    jsonc.append(123.456).unwrap();
    assert!(jsonc.count().unwrap() == 2, "object count != 2");

    jsonc.append("toto").unwrap();
    assert!(jsonc.count().unwrap() == 3, "object count != 3");

    // adding an object may fail is target is not an object
    let jobject = JsoncObj::parse("{'a':1,'b':2}");
    assert!(jobject.is_ok(), "Fail to parse jsonc string");
    let result = jsonc.append(jobject.unwrap());
    assert!(result.is_ok(), "Fail insert jsonc object in array");
    assert!(jsonc.count().unwrap() == 4, "object count != 4");

    match jsonc.get_type() {
        Jtype::Array => println!("jsonc is array"),
        Jtype::Object => println!("jsonc is object"),
        _ => panic!("Hoop: unknown jtype"),
    }
}

#[test]
fn expend_to_vec() {
    let token = "{'a':1,'b':'abc'}";
    let jvalue = JsoncObj::parse(token).unwrap();
    let entries = jvalue.expand().unwrap();
    for idx in 0..entries.len() {
        let entry = &entries[idx];
        println!("key={}, value:{}", entry.key, entry.obj);
    }
}

// ------------------------------------------------
// testing object type
// -------------------------------------------------
#[test]
fn hexa_converter() {
    let input = "[01,02,03,04,05,06]";
    let mut buffer: [u8; 6] = [0; 6];

    let buffer = hexa_to_bytes(input, &mut buffer).unwrap();

    let result = bytes_to_hexa(&buffer);
    assert!(input == result);
}

#[test]
fn get_from_object() {
    let value1 = 123;
    let value2 = 123.456;
    let value3 = "toto";

    // create a testing jsonc object
    let jsonc = JsoncObj::new();
    jsonc.add("slot1", value1).unwrap();
    jsonc.add("slot2", value2).unwrap();
    jsonc.add("slot3", value3).unwrap();
    jsonc.add("slot4", JsoncObj::new()).unwrap();
    jsonc.add("slot5", JsoncObj::array()).unwrap();
    assert!(
        matches!(jsonc.get_type(), Jtype::Object),
        "object not a jsonc object"
    );

    match jsonc.get::<i64>("slot1") {
        Ok(value) => assert!(value == value1, "slot1/value diverge"),
        Err(error) => panic!("fail getting 'slot1'={}", error),
    }

    match jsonc.get::<f64>("slot2") {
        Ok(value) => assert!(value == value2, "slot2/value diverge"),
        Err(error) => panic!("fail getting 'slot2'={}", error),
    }

    match jsonc.get::<String>("slot3") {
        Ok(value) => assert!(value == value3, "slot2/value diverge"),
        Err(error) => panic!("fail getting 'slot3'={}", error),
    }

    let labels = ["slot1", "slot2", "slot3", "slot4", "slot5"];
    println!("Loop on jsonc object= {}", jsonc);
    for key in 0..labels.len() {
        match jsonc.get(labels[key]).unwrap() {
            Jobject::Int(value) => assert!(value == value1, "slot1/value diverge"),
            Jobject::Float(value) => assert!(value == value2, "slot2/value diverge"),
            Jobject::String(value) => assert!(value == value3, "slot2/value diverge"),
            Jobject::Object(value) => {
                assert!(value.is_type(Jtype::Object), "object not a jsonc object")
            }
            Jobject::Array(value) => {
                assert!(value.is_type(Jtype::Array), "object not a jsonc array")
            }
            _ => panic!("invalid jsonc type"),
        }
    }
}

// ------------------------------------------------
// testing object type
// -------------------------------------------------
#[test]
fn get_from_array() {
    let value1 = 123;
    let value2 = 123.456;
    let value3 = "toto";

    // create a testing jsonc object
    let jsonc = JsoncObj::array();
    jsonc.append(value1).unwrap();
    jsonc.append(value2).unwrap();
    jsonc.append(value3).unwrap();
    jsonc.append(JsoncObj::new()).unwrap();
    jsonc.append(JsoncObj::array()).unwrap();
    assert!(
        matches!(jsonc.get_type(), Jtype::Array),
        "object not a jsonc array"
    );

    match jsonc.index::<i64>(0) {
        Ok(value) => assert!(value == value1, "slot1/value diverge"),
        Err(error) => panic!("fail getting 'slot1'={}", error),
    }

    match jsonc.index::<f64>(1) {
        Ok(value) => assert!(value == value2, "slot2/value diverge"),
        Err(error) => panic!("fail getting 'slot2'={}", error),
    }

    match jsonc.index::<String>(2) {
        Ok(value) => assert!(value == value3, "slot2/value diverge"),
        Err(error) => panic!("fail getting 'slot3'={}", error),
    }

    println!("Loop on jsonc array= {}", jsonc);
    for idx in 0..jsonc.count().unwrap() {
        match jsonc.index::<Jobject>(idx).unwrap() {
            Jobject::Int(value) => assert!(value == value1, "slot1/value diverge"),
            Jobject::Float(value) => assert!(value == value2, "slot2/value diverge"),
            Jobject::String(value) => assert!(value == value3, "slot2/value diverge"),
            Jobject::Object(value) => {
                assert!(value.is_type(Jtype::Object), "object not a jsonc object")
            }
            Jobject::Array(value) => {
                assert!(value.is_type(Jtype::Array), "object not a jsonc array")
            }
            _ => panic!("invalid jsonc type"),
        }
    }
}
