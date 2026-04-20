use std::sync::{Arc, MappedMutexGuard, Mutex, MutexGuard};

use crate::ble::{Bluetooth, characteristics::Characteristic};

#[derive(Default)]
pub struct App {
    pub bt: Mutex<Option<Arc<Bluetooth>>>,
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

    pub fn bt(&self) -> MappedMutexGuard<'_, Arc<Bluetooth>> {
        MutexGuard::map(self.bt.lock().unwrap(), |x| x.as_mut().unwrap())
    }

    pub fn position_update(&self, lat: i32, lon: i32) {
        let mut boat = self.boat();
        boat.latitude = lat;
        boat.longitude = lon;
        self.bt()
            .notify(Characteristic::Position, &boat.position_packet());
    }

    pub fn speed_update(&self, speed: u16) {
        let mut boat = self.boat();
        boat.speed_over_ground = speed;
        self.bt()
            .notify(Characteristic::Speed, &boat.speed_packet());
    }
}

impl Boat {
    pub fn packet(&self, characteristic: Characteristic) -> Vec<u8> {
        match characteristic {
            Characteristic::Position => self.position_packet(),
            Characteristic::Speed => self.speed_packet(),
        }
    }

    fn position_packet(&self) -> Vec<u8> {
        format!("{}, {}", self.latitude, self.longitude)
            .as_bytes()
            .to_vec()
    }

    fn speed_packet(&self) -> Vec<u8> {
        self.speed_over_ground.to_string().as_bytes().to_vec()
    }
}
