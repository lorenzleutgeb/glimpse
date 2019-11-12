extern crate cgmath;
extern crate enigo;
extern crate tobii_sys;

mod gyro_input;
mod inputs;
mod tobii_input;

use cgmath::prelude::MetricSpace;
use cgmath::{vec2, vec3, Vector2};
use enigo::{Enigo, MouseControllable};

use std::cmp::{max, min};
use std::mem;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, Instant};

use inputs::{Input, InputPool};
use std::process::Command;

// TODO: Do not hardcode display height and width, but
// make it configurable or auto-detect.

const display_laptop_x: f32 = 1920f32;
const display_laptop_y: f32 = 1080f32;
const display_dell_x: f32 = 2560f32;
const display_dell_y: f32 = 1440f32;

const offset_laptop_x: f32 = display_dell_x / 2f32 - display_laptop_x / 2f32;
const offset_laptop_y: f32 = display_dell_y;
const offset_dell_x: f32 = 0f32;
const offset_dell_y: f32 = 0f32;

const display_x: f32 = display_dell_x;
const display_y: f32 = display_dell_y;
const offset_x: f32 = offset_dell_x;
const offset_y: f32 = offset_dell_y;

const distance_to_center_max: f32 = 0.7071067811865476f32;

fn fmax(a: f32, b: f32) -> f32 {
    if b.is_nan() || b <= a {
        a
    } else {
        b
    }
}

fn fmin(a: f32, b: f32) -> f32 {
    if b.is_nan() || b >= a {
        a
    } else {
        b
    }
}

fn calc_dt(tick: Instant, last_tick: &mut Instant) -> f32 {
    let dur = tick.duration_since(*last_tick);
    let dt = dur.as_secs() as f32 + dur.subsec_nanos() as f32 * 1.0e-9;
    mem::replace(last_tick, tick);
    dt
}

/*
fn is_warp(cx: i32, cy: i32, x: i32, y: i32) -> bool {
    // First prototype, worked quite nice but does not adapt,
    // better accuracy in the middle of the screen.
    //euclidean_distance(x, y, cx, cy) > (screen_x / 16f32)
    let (xf, yf) = (x as f64, y as f64);
    let (sx, sy) = (screen_x as f64, screen_y as f64);

    let (dx, dy) = ((xf - (cx as f64)).powi(2), (yf - (cy as f64)).powi(2));
    let d = (dx + dy).sqrt();

    if (xf < sx / 3f64 || xf > 2f64 * sx / 3f64) && (yf < sy / 3f64 || yf > 2f64 * sy / 3f64) {
        if dx > dy {
            //println!("Area 2");
            d > (sx / 24f64)
        } else {
            //println!("Area 1");
            d > (sy / 16f64)
        }
    } else {
        //println!("Area 3");
        d > 16f64
    }
}
*/

fn current_location() -> (i32, i32) {
    // TODO(lorenz.leutgeb): Look for a faster version to obtain the current
    // cursor position. Spawning xdotool adds a dependency and is quite heavy.
    let mut current = String::from_utf8(
        Command::new("xdotool")
            .arg("getmouselocation")
            .arg("--shell")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .split("\n")
    .take(2)
    .map(|s| -> i32 { s[2..].parse::<i32>().unwrap() })
    .collect::<Vec<i32>>();

    let y = current.pop().unwrap();
    let x = current.pop().unwrap();
    (x, y)
}

fn move_cursor_relative(x: i32, y: i32) {
    // TODO(lorenz.leutgeb): Look for a native solution to move the cursor.
    // Spawning xdotool adds a dependency and is quite heavy.
    Command::new("xdotool")
        .arg("mousemove_relative")
        .arg("--")
        .arg((x).to_string())
        .arg((y).to_string())
        .status();
}

fn run_pipeline(rx: Receiver<Input>) {
    let mut raw_head_angular_velocity: Vector2<f32> = vec2(0.0, 0.0);
    let mut raw_gaze: Vector2<f32> = vec2(0.0, 0.0);

    let mut last_head_move = Instant::now();

    let mut gaze_pt: Vector2<f32> = vec2(0.0, 0.0);
    let mut anchor: Vector2<f32> = vec2(0.0, 0.0); // [px]
    let mut px_gaze: Vector2<f32>; // [px]

    let mut enigo = Enigo::new();

    let mut distance_to_screen = 0.50f32; // [m]

    loop {
        // update input state =========================
        let mut tick_gaze = false;
        let mut tick_head = false;
        match rx.recv().unwrap() {
            Input::TobiiGaze { x, y } => {
                raw_gaze = vec2(x, y);
                tick_gaze = true;
            }
            Input::TobiiGazeOrigin {
                rx,
                ry,
                rz,
                lx,
                ly,
                lz,
            } => {
                // z is distance to screen moving closer and further away
                // x is left (-) and right (+)
                // y is up (+) and down (-)
                //println!("{0:8.3} {1:8.3} {2:8.3}", rx - lx, ry - ly, rz - lz);
                //println!("{0:8.3} {1:8.3} {2:8.3}", rx, ry, rz);
                let distance_to_left_eye = vec3(0f32, 0f32, 0f32).distance(vec3(lx, ly, lz));
                let distance_to_right_eye = vec3(0f32, 0f32, 0f32).distance(vec3(rx, ry, rz));
                // Distances reported by Tobii are in millimeters. We take the average and convert
                // to meters at the same time.
                distance_to_screen = (distance_to_left_eye + distance_to_right_eye) / 2000f32;
                //println!("{0:8.3}", distance_to_screen);
            }
            Input::Gyro { x, y } => {
                raw_head_angular_velocity = vec2(x, y);
                tick_head = true;
            }
            Input::Shutdown => break,
            _ => {
                panic!("got some input that will not be handled");
            }
        }

        let tick = Instant::now();

        if tick_head {
            // Rotate to account for mounting offset.
            raw_head_angular_velocity = rotate(raw_head_angular_velocity, 13f32);

            let mut angular = vec2(
                (raw_head_angular_velocity.x * distance_to_screen * 1.7) as i32,
                (raw_head_angular_velocity.y * distance_to_screen) as i32,
            );

            if angular.x.abs() > 0 || angular.y.abs() > 0 {
                if tick.duration_since(last_head_move) < Duration::from_millis(100)
                    && angular.x.abs() + angular.y.abs() > 5
                {
                    //println!("BOOST");
                    angular.x = (angular.x * 3) / 2;
                    angular.y = (angular.y * 3) / 2;
                    last_head_move = tick;
                } else if angular.x.abs() + angular.y.abs() > 2 {
                    last_head_move = tick;
                }
                anchor.x = anchor.x + (angular.x as f32);
                anchor.y = anchor.y + (angular.y as f32);
                move_cursor_relative(angular.x, angular.y);
            }
        }

        if tick_gaze {
            let dt = tick.duration_since(last_head_move);

            let distance_to_center = denormalize(vec2(0.5f32, 0.5f32)).distance(anchor);

            // Ratio that increases as the distance between gaze point and center of screen
            // increases, but within (0;1).
            let distance_to_center_ratio =
                fmin(1.0f32, distance_to_center / distance_to_center_max);

            px_gaze = denormalize(raw_gaze);

            let d = euclidean_distance(
                px_gaze.x as i32,
                px_gaze.y as i32,
                anchor.x as i32,
                anchor.y as i32,
            );

            // User is still looking at anchor, so do nothing.
            if d < 5 {
                continue;
            }

            // Threshold for making an absolute jump:
            //   - Inverse to the distance of the anchor. This way, if the gaze is far away from
            //     the anchor because the user has changed their attention to something else, the
            //     jump will be allowed, but if the user is just adjusting for precision there
            //     will be no jump.
            //   - Larger the greater the distance to the center of the screen. Since the
            //     eye-tracker shows best accuracy in the center, there are more corrective actions
            //     to be expected at the edges.

            // Maximum distance of a jump on the display.
            // TODO: Avoid recomputation of this number over and over again?
            let display_max: f32 = (display_x.powi(2) + display_y.powi(2)).sqrt();

            // Ratio that increases proportional to the distance between the gaze point and the
            // anchor point, but within (0;1).
            let max_ratio: f32 = fmin(1.0f32, (d as f32) / display_max);

            let threshold_as_millis = ((100.0 + 1100.0
                - fmin(600.0, (max_ratio * distance_to_center_ratio * 24000.0)))
                as u64);
            let threshold = Duration::from_millis(threshold_as_millis);
            //println!("threshold: {}", threshold_as_millis );
            if dt < threshold {
                //println!("cannot move... tooo close");
                continue;
            }

            px_gaze = px_gaze + vec2(offset_x, offset_y);

            if d > 30 || (anchor.x == 0f32 && anchor.y == 0f32) {
                anchor = px_gaze;
                enigo.mouse_move_to(px_gaze.x as i32, px_gaze.y as i32);
            } else if dt > Duration::from_millis(1000) {
                if d > 20 {
                anchor = px_gaze;
                enigo.mouse_move_to(px_gaze.x as i32, px_gaze.y as i32);
                }
            else  {
                px_gaze = vec2((px_gaze.x + anchor.x) / 2f32, (px_gaze.y + anchor.y) / 2f32);
                enigo.mouse_move_to(px_gaze.x as i32, px_gaze.y as i32);
            }
            }
        }
    }
}

fn rotate(v: Vector2<f32>, degrees: f32) -> Vector2<f32> {
    let bias = degrees.to_radians();
    vec2(
        v.x * bias.cos() - v.y * bias.sin(),
        v.x * bias.sin() + v.y * bias.cos(),
    )
}

fn denormalize(p: Vector2<f32>) -> Vector2<f32> {
    if p.x.is_nan() || p.y.is_nan() {
        println!("encountered NaN!");
        return vec2(0f32, 0f32);
    }
    vec2(p.x * display_x, p.y * display_y)
}

fn euclidean_distance(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
    (((x1 - x2) as f64).powi(2) + ((y1 - y2) as f64).powi(2)).sqrt() as i32
}

fn main() {
    let (mut pool, rx) = InputPool::new();
    pool.spawn(tobii_input::listen);
    pool.spawn(gyro_input::listen);

    let handle = thread::spawn(|| run_pipeline(rx));
    handle.join().unwrap();
}
