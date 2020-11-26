use std::net::{TcpStream, /*UdpSocket*/};
use std::fmt::Write;

pub fn scan(cmd_arg: String) -> (String, String) {
    //println!("cmd_arg: {:#?}", cmd_arg);
    let mut iter = cmd_arg.split_whitespace();
    let scan_type = iter.next();
    let ip = iter.next();

    let addr: String = match ip {
        None => {"None".to_string()},
        Some(ref ip) => {ip.to_string()},
    };
  
    let mut scan_result: String = String::new();
    let scan_error: String = String::new();
    for port in iter {
        match scan_type {
            Some(scan_t) => {
                match scan_t {
                    "tcp" => {
                        let r: String = tcp_scan(addr.clone(), port.to_string());
                        //println!("{:#?} : {:#?}", port, r);
                        write!(&mut scan_result, "{:#?} : {:#?}\n", port, r).unwrap()
                    },
                    /*"UDP" => {
                        let r: String = udp_scan(addr.clone(), port.to_string());
                        write!(&mut scan_result, "{:#?} : {:#?}\n", port, r).unwrap()
                    },*/
                    _ => {
                        write!(&mut scan_result, "{:#?} : error (1) scanning\n", port).unwrap()
                    }    
                }
            },
            None => {
                write!(&mut scan_result, "{:#?} : error (2) scanning\n", port).unwrap()
            },
        };
        //println!("{:#?}", port);
    }

    (scan_result, scan_error)

}

fn tcp_scan(address: String, port: String) -> String {
    let full_address = [address, port].join(":");
    let r: String = match TcpStream::connect(full_address) {
        Ok(_) => {
            "OPENED".to_string()
        },
        Err(_) => {
            "closed".to_string()
        },
    };

    r
}

/*fn udp_scan(address: String, port: String) -> String {
    let full_address = [address, port.clone()].join(":");

    "".to_string()
}*/
