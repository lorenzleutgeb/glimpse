# glimpse

A mouse replacement for eye-tracking and head-tracking. Work in progress.

## Hardware Requirements (at Runtime)

 * Tobii eyeX 4C
 * Gyroscope

As an alternative to using a gyroscope, you may use a webcam to track your head
position. Sofrware that does this is freely available (eviacam). However,
tracking is problematic in bad ambient light conditions (including direct light
and darkness). Also, running computer vision algorithms to process 60 frames
from the camera per second is expensive.

## Software Requirements (at Runtime)

 * Linux (with [`uinput`](uinput) module)
 * `tobiiusbserviced` must be running (must be acquired from Tobii)

## Software Requirements (at Compiletime)

 * Rust

## Architecture

Use Tobii's stream engine to interface with the Tobii eyeX 4C.

## FAQs

### Does this driver support interacting on multiple screens in parallel?

No. That would be possible in theory, but [Tobii hardware only supports one
screen](multiple). [Some users have requested this feature already
](multiple-forum).

### Does this driver support profiles for different screens?

No. But this could be done via different calibrations (one per screen).

### Does this driver support calibration?

No. The calibration flow is planned as follows:
 * `tobii_calibration_start`, `tobii_calibration_collect_data_2d`,
   `tobii_calibration_compute_and_apply` (for calibrating)
 * `tobii_calibration_retrieve` (and dump to a file)
 * `tobii_calibration_apply` (after reading from a file)

[Tobii also notes that you should recalibrate for glasses/lenses as
well as light/dark environment.](calibration)

## Related

 * https://www.omnicalculator.com/math/arc-length#arc-length-formula
 * https://thume.ca/2016/03/24/eye-tracker-reviews-pupil-labs-tobii-eyex-eye-tribe-tobii-x2-30/
 * https://github.com/trishume/tobii-sys

## Gyroscope

I am using a Wit-Motion JY901 module which contains a MPU9250 which
combines the MPU6500 (accelerometer, gyroscope) and the AK8963 (magnetometer).

See:
 * https://github.com/psiphi75/mpu9250-i2c
 * https://github.com/copterust/mpu9250
 * https://www.invensense.com/products/motion-tracking/9-axis/mpu-9250/
 * https://github.com/semaf/MPU-9250
 * https://www.hackster.io/30503/using-the-mpu9250-to-get-real-time-motion-data-08f011
 * https://github.com/brumster/EDTracker2
 * https://github.com/TheChapu/GY-91
 * https://github.com/eupn/bno055
 * https://github.com/pinetum/Sensor-JY901

[ch7]: https://www.embeddedlinux.org.cn/essentiallinuxdevicedrivers/final/ch07.html
[uinput]: https://www.kernel.org/doc/html/latest/input/uinput.html
[multiple]: https://help.tobii.com/hc/en-us/articles/209529429
[multiple-forum]: https://developer.tobii.com/community/forums/topic/dual-screen-multi-screen-support/
[calibration]: https://help.tobii.com/hc/en-us/articles/360023794433
