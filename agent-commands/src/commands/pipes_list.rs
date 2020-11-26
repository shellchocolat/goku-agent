#[cfg(windows)]
use std::process::Command;

#[cfg(windows)]
pub fn pipes_list() -> std::process::Output {
    let output = Command::new("cmd")
        .args(&["/C", "dir", "\\\\.\\pipe\\\\"])
        .output()
        //.spawn() // to display the output -> use for test
        .expect("Command failed to start");
    output
}
