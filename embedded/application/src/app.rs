use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, MutexGuard, mpsc::SyncSender},
};

use anyhow::Result;
use common::SpiFlash;
use esp_idf_hal::sys::twai_message_t;
use esp_idf_svc::{log::EspIdfLogger, nvs::EspDefaultNvsPartition};
use log::{Log, Metadata, Record, info};
use nmea2000::packets::RawPacket;

use crate::{
    ble::{Bluetooth, characteristics::Characteristic},
    util::{ForceLock, RollingAverage},
    wifi::WirelessClient,
};

type Soon<T> = Mutex<Option<T>>;

pub struct App {
    pub logs: Mutex<VecDeque<String>>,
    pub bt: Soon<Arc<Bluetooth>>,
    pub indicator: Soon<SyncSender<IndicatorEvent>>,
    pub wireless: Mutex<Vec<WirelessClient>>,
    pub packets: Mutex<Vec<RawPacket>>,

    pub nvs: EspDefaultNvsPartition,
    pub flash: Mutex<SpiFlash>,

    boat: Mutex<Boat>,
}

#[derive(Default)]
pub struct Boat {
    pub latitude: i32,
    pub longitude: i32,
    pub wind_speed: RollingAverage<16>,
    pub wind_angle: RollingAverage<16>,
    pub speed_over_ground: RollingAverage<16>,
}

pub enum IndicatorEvent {
    CanOnline,
}

pub struct MemoryLogger {
    pub app: Arc<App>,
    pub uart: EspIdfLogger<()>,
}

impl App {
    pub fn new(flash: SpiFlash) -> Result<Self> {
        let id = flash.read_id()?;
        let size = flash.size()?;
        info!("Connected to flash {{ id=0x{id:#08x}, size={size} }}");

        Ok(Self {
            logs: Default::default(),
            bt: Default::default(),
            indicator: Default::default(),
            wireless: Default::default(),
            packets: Default::default(),

            nvs: EspDefaultNvsPartition::take().unwrap(),
            flash: Mutex::new(flash),

            boat: Default::default(),
        })
    }

    pub fn boat(&self) -> MutexGuard<'_, Boat> {
        self.boat.force_lock()
    }

    pub fn enqueue_packet(&self, packet: RawPacket) {
        self.packets.force_lock().push(packet);
    }

    pub fn on_can_frame(&self, frame: twai_message_t) {
        let mut wireless = self.wireless.force_lock();

        let mut i = 0;
        while i < wireless.len() {
            if wireless[i].write(frame) {
                wireless.remove(i);
            } else {
                i += 1;
            }
        }
    }

    pub fn indicator(&self, event: IndicatorEvent) {
        let mut channel = self.indicator.force_lock();
        channel.as_mut().unwrap().send(event).unwrap();
    }

    pub fn position_update(&self, lat: i32, lon: i32) {
        let mut boat = self.boat();
        boat.latitude = lat;
        boat.longitude = lon;
    }

    pub fn speed_update(&self, speed: u16) {
        let mut boat = self.boat();
        boat.speed_over_ground.push(speed as f32);
    }

    pub fn wind_update(&self, speed: u16, angle: u16) {
        let mut boat = self.boat();
        boat.wind_speed.push(speed as f32);
        boat.wind_angle.push(angle as f32);
    }
}

impl Boat {
    pub fn notify(&self, bt: &Arc<Bluetooth>, characteristic: Characteristic) {
        bt.notify(characteristic, &self.packet(characteristic));
    }

    pub fn packet(&self, characteristic: Characteristic) -> Vec<u8> {
        match characteristic {
            Characteristic::WindScreen => self.wind_screen_packet(),
        }
    }

    fn wind_screen_packet(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend((self.speed_over_ground.avg() as u16).to_le_bytes());
        out.extend((self.wind_speed.avg() as u16).to_le_bytes());
        out.extend((self.wind_angle.avg() as u16).to_le_bytes());
        out
    }
}

impl MemoryLogger {
    pub fn new(app: &Arc<App>) -> Self {
        Self {
            app: app.clone(),
            uart: EspIdfLogger::new(()),
        }
    }
}

impl Log for MemoryLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let mut logs = self.app.logs.force_lock();
        logs.push_back(format!("{} {}", record.level(), record.args()));
        (logs.len() > 30).then(|| logs.pop_front());
        drop(logs);

        self.uart.log(record);
    }

    fn flush(&self) {}
}
