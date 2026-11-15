//! fork: what stands in for backend.rs without the `joystick` feature: no
//! controller library, no thread, never a device.

use super::Snapshot;

/// What the frame needs from the devices (as backend.rs).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Use {
    Off = 0,
    List = 1,
    On = 2,
    Gamepad = 3,
}

#[derive(Default)]
pub struct Backend;

impl Backend {
    pub fn poll(&self, _mode: Use, _latest: &mut Snapshot) {}
}
