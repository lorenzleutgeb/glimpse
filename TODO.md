# TODO

## Essentials

 - Stabilize eye-tracking input.
 - Smooth mode-switch between absolute and relative positioning.

## Usability
 - Monitor input on other devices. Can this be done using `evdev`?
   - Do not move cursor if user is clicking (for shorter than some milliseconds).
   - Do not move cursor if another mouse/touchpad was used within the last 5 seconds.

## Nice to Have
 - Change cursor to crosshair when enabled.
 - Use Tobii to estimate distance between head and screen, then adjust relative movement
   to be larger with greater distance.
