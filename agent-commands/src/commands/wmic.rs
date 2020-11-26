#[cfg(windows)]
use wmi::{COMLibrary, Variant, WMIConnection};
use std::collections::HashMap;

#[cfg(windows)]
pub fn wmic(cmd_arg: String) -> (String, String) {
    //println!("cmd_arg: {:#?}", cmd_arg);

    let com_con = COMLibrary::new().unwrap();
    let wmi_con = WMIConnection::new(com_con.into()).unwrap();

    let results: Vec<HashMap<String, Variant>> = wmi_con.raw_query(&cmd_arg).unwrap();
    
    /*for r in results {
        println!("{:#?}", r);
    }*/

    ("".to_string(), "".to_string())
    //(results.to_string(), results.to_string())

}
