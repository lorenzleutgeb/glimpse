# glimpse

A mouse driver for eye-tracking and head-tracking. Work in progress.

## Hardware Requirements (at Runtime)

 * Tobii eyeX 4C
 * A webcam (?)

## Software Requirements (at Runtime)

 * Linux (with [`uinput`](uinput) module)
 * OpenCV (?)
 * Tobii's stream engine (driver for the Tobii eyeX 4C, must be acquired from vendor)

## Software Requirements (at Compiletime)

 * Rust

## Architecture

Use Tobii's stream engine to interface with the Tobii eyeX 4C.

Publish events through `uinput`.

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

[ch7]: https://www.embeddedlinux.org.cn/essentiallinuxdevicedrivers/final/ch07.html
[uinput]: https://www.kernel.org/doc/html/latest/input/uinput.html
[multiple]: https://help.tobii.com/hc/en-us/articles/209529429
[multiple-forum]: https://developer.tobii.com/community/forums/topic/dual-screen-multi-screen-support/
[calibration]: https://help.tobii.com/hc/en-us/articles/360023794433
