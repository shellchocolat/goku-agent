use std::process::Command;

#[cfg(unix)]
pub fn psh(cmd_arg: String) -> std::process::Output {
    let output = Command::new("sh")
        .args(&["-c", &cmd_arg])
        .output()
        //.spawn() // to display the output -> use for test
        .expect("Command failed to start");

    output
}

#[cfg(windows)]
pub fn psh(cmd_arg: String) -> std::process::Output {
    let output = Command::new("powershell")
        .args(&["-Command", &cmd_arg])
        .output()
        //.spawn() // to display the output -> use for test
        .expect("Command failed to start");
    output
}
