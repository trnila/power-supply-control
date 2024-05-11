# Power supply control
Power supply control GUI for AIM-TTI MX100QP over USB UART.


## Build
```shell-session
$ cargo install dioxus-cli
$ dx serve
$ dx build --profile release
```

## Testing with arduino stub
Flash `MX100QP/MX100QP.ino` into your arduino for testing with fake device if you dont have real power supply.
