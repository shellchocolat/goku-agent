use std::process::Command;

#[cfg(unix)]
pub fn cmd(cmd_arg: String) -> std::process::Output {
    let output = Command::new("sh")
        .args(&["-c", &cmd_arg])
        .output()
        //.spawn() // to display the output -> use for test
        .expect("Command failed to start");

    output
}

#[cfg(windows)]
pub fn cmd(cmd_arg: String) -> std::process::Output {
    let output = Command::new("cmd")
        .args(&["/C", &cmd_arg])
        .output()
        //.spawn() // to display the output -> use for test
        .expect("Command failed to start");
    output
}
