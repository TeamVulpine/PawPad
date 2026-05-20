use crate::backend::PawPadBackend;

pub mod backend;
pub mod gamepad;

pub fn wawa() {
    PawPadBackend::new().unwrap().update().unwrap();
}
