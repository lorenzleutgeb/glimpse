## How to interface with the "Stream Engine"?
 - Not C.
 - Node.js FFI?
 - Python FFI?
 - SWIG?
 - Rust bindgen!
 - There's the `tobii-sys` crate!

## OpenCV
 - Too heavy. Cheaper alternative is the gyro, which is also more accurate.

## Finding a Gyro
 - How modern MEMS work.
 - Soldering.

## How to interface with the Gryo?
 - SPI != IIC != TTL
 - Cannot use crate `mpu9250` as I initially thought.
 - Serial interface via Tokio!

## Asynchronous Programming in Rust
 - Tokio is complicated.

## Tobii USB Service Daemon
 - If it does not work, try `mkdir /var/run/tobiiusb`
