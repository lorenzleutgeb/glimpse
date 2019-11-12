use std::ffi::{CStr, CString};
use std::mem;
use std::os::raw;
use std::ptr;
use std::sync::mpsc::{Receiver, SyncSender};

use crate::inputs::{Input, InputAction};

use std::{env, io, str};
use tokio_io::codec::{Decoder, Encoder, Framed};

use bytes::{ByteOrder, BytesMut, LittleEndian};
use std::process::Command;

use crate::tokio::prelude::{Future, Stream};

#[cfg(unix)]
const DEFAULT_TTY: &str = "/dev/ttyUSB0";
#[cfg(windows)]
const DEFAULT_TTY: &str = "COM1";

// TODO:
//  - Set return content (7.2.8) to only contain the packages we are interested in.
//  - Set baud rate (7.2.10) to as high as possible.
//  - Set return rate (7.2.9) to as high as possible (200Hz?).

// All frames are 11 bytes long. The first is always FRAME_START, the second is
// a discriminator, and the last is a checksum.
//
//          Discri
// Section  minator Measurement       Unit
// -----------------------------------------------
//   7.1.1   0x50   Time              [hh:mm:ss]
//   7.1.2   0x51   Acceleration      [m / s²] * 3
//   7.1.3   0x52   Angular Velocity  [° / s ] * 3
//   7.1.4   0x53   Angle             [°     ] * 3
//   7.1.5   0x54   Magnetic          [ TODO ] * 3
//   7.1.10  0x59   Quaternion        [ TODO ] * ?
//   ...
//
// Other discriminators which I am not interested in:
//   0x55, 0x56, 0x57, 0x58, 0x5A

enum MeasurementFrameType {
    Acceleration,    // [m / s²]
    Angle,           // [°]
    Magnetic,        // [?] TODO
    AngularVelocity, // [° / s]
}

const FRAME_START: u8 = 0x55;

struct LineCodec {
    output: SyncSender<Input>,
}

impl Decoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() == 0 {
            return Ok(None);
        }

        match src.iter().position(|b| *b == FRAME_START) {
            None => Ok(None),
            Some(offset) => {
                let mut frame = src.split_to(offset);
                src.split_to(1); // Consume FRAME_START too.

                if frame.len() != 10 {
                    // TODO: Maybe warn that we got a frame that looks weird?
                    if frame.len() != 0 {
                        println!("ignoring frame of length {}", frame.len());
                    }
                    return Ok(None);
                }

                // TODO: Check against checksum.
                frame.split_off(9);

                let discriminator = frame.split_to(1)[0];

                match discriminator {
                    0x51 => {
                        // Acceleration
                        // Python script also multiplies by 16.
                        let x = (LittleEndian::read_i16(&mut frame) as f32) / 32768f32 * 16f32;
                        frame.advance(2);
                        let y = (LittleEndian::read_i16(&mut frame) as f32) / 32768f32 * 16f32;
                        frame.advance(2);
                        let z = (LittleEndian::read_i16(&mut frame) as f32) / 32768f32 * 16f32;
                        frame.advance(2);
                        //println!("acc {} {} {}", x, y, z);
                        //self.output.send(Input::Gyro{x: z - 0.315f32, y: y});
                        Ok(Some("Attempted decode".to_string()))
                    }
                    0x52 => {
                        // Angular Velocity
                        // Rotation of head left/right
                        let x = (LittleEndian::read_i16(&mut frame) as f32) / 32768f32 * 2000f32;
                        frame.advance(2);
                        let y = (LittleEndian::read_i16(&mut frame) as f32) / 32768f32 * 2000f32;
                        frame.advance(2);
                        // Rotation of head up/down
                        let z = (LittleEndian::read_i16(&mut frame) as f32) / 32768f32 * 2000f32;
                        frame.advance(2);
                        let temp = (LittleEndian::read_i16(&mut frame) as f32) / 100f32;
                        //println!("anv {0:6.3} {1:6.3} {2:6.3}", x, y, z);
                        self.output.send(Input::Gyro { x, y: z });
                        Ok(Some("Attempted decode".to_string()))
                    }
                    0x53 => {
                        // Angle
                        // Python Script also multiplies by 170
                        // x (east)
                        let roll = (LittleEndian::read_i16(&mut frame) as f32) / 32768f32 * 180f32;
                        frame.advance(2);
                        // y (north)
                        let pitch = (LittleEndian::read_i16(&mut frame) as f32) / 32768f32 * 180f32;
                        frame.advance(2);
                        // z (toward sky)
                        let yaw = (LittleEndian::read_i16(&mut frame) as f32) / 32768f32 * 180f32;
                        frame.advance(2);
                        let temp = (LittleEndian::read_i16(&mut frame) as f32) / 100f32;
                        // println!("yaw {0:6.3}    pitch {1:6.3}     ignore {2:6.3}", yaw, roll, pitch);
                        self.output.send(Input::HeadAngle {
                            roll,
                            pitch,
                            yaw,
                        });
                        //move_cursor_relative((z * -1.5f32) as i32, x as i32);
                        Ok(Some("Attempted decode".to_string()))
                    }
                    _ => {
                        // TODO: Maybe warn that we are dropping a frame?
                        //println!("ignoring frame with discriminator {:x}", discriminator);
                        Ok(None)
                    }
                }
            }
        }
    }
}

impl Encoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn encode(&mut self, _item: Self::Item, _dst: &mut BytesMut) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub fn listen(output: SyncSender<Input>, inbox: Receiver<InputAction>) {
    let tty_path: &str = DEFAULT_TTY.into();

    let settings = tokio_serial::SerialPortSettings {
        baud_rate: 921600, // default is 9200
        data_bits: tokio_serial::DataBits::Eight,
        flow_control: tokio_serial::FlowControl::None,
        parity: tokio_serial::Parity::None,
        stop_bits: tokio_serial::StopBits::One,
        timeout: std::time::Duration::from_millis(1000),
    };
    let mut port = tokio_serial::Serial::from_path(tty_path, &settings).unwrap();

    #[cfg(unix)]
    port.set_exclusive(false)
        .expect("Unable to set serial port exlusive");

    let (_, reader) = LineCodec{output}.framed(port).split();

    let printer = reader
        .for_each(|s| {
            println!("{:?}", s);
            Ok(())
        })
        .map_err(|e| eprintln!("{}", e));

    tokio::run(printer);
}
