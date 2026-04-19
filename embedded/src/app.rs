use std::sync::{Mutex, MutexGuard};

use crate::ble::Bluetooth;

#[derive(Default)]
pub struct App {
    pub bt: Mutex<Option<Bluetooth>>,
    boat: Mutex<Boat>,
}

#[derive(Default)]
pub struct Boat {
    pub latitude: i32,
    pub longitude: i32,
    pub speed_over_ground: u16,
}

impl App {
    pub fn boat(&self) -> MutexGuard<'_, Boat> {
        self.boat.lock().unwrap()
    }
}
