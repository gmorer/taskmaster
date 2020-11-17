# taskmaster
Job control daemon in rust

```bash
cargo run ./test.conf
````

you can nc on it:
```bash
nc -v localhost 6061
```

available commands:
 - __conf__: show available programs from the conf file
 - __start $program_name__: launch a program
 - __ls__: list running programs
 - __stop $program_name__: stop a runing program
