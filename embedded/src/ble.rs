use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use clone_macro::clone;
use esp_idf_hal::modem::Modem;
use esp_idf_svc::{
    bt::{
        Ble, BtDriver, BtUuid,
        ble::{
            gap::{AdvConfiguration, AppearanceCategory, BleGapEvent, EspBleGap},
            gatt::{
                AutoResponse, GattCharacteristic, GattId, GattResponse, GattServiceId, GattStatus,
                Permission, Property,
                server::{EspGatts, GattsEvent},
            },
        },
    },
    nvs::EspDefaultNvsPartition,
};
use log::info;
use uuid::uuid;

use characteristics::CharacteristicHandles;

use crate::{app::App, ble::characteristics::Characteristic};

const APP_ID: u16 = 0;
const SERVICE: u128 = uuid!("76e20500-da73-4971-bb03-6105e39db3d6").as_u128();

mod characteristics {
    use std::sync::atomic::{AtomicU16, Ordering};

    use esp_idf_svc::bt::BtUuid;
    use uuid::uuid;

    pub const ALL: &[u128] = &[POSITION, SPEED];

    pub const POSITION: u128 = uuid!("300b2aec-a094-43fb-98ff-04917cf7a2fb").as_u128();
    pub const SPEED: u128 = uuid!("d948b9e5-6626-4d41-8967-c4dca26db1fd").as_u128();

    pub enum Characteristic {
        Position,
        Speed,
    }

    #[derive(Default)]
    pub struct CharacteristicHandles {
        position: AtomicU16,
        speed: AtomicU16,
    }

    impl CharacteristicHandles {
        pub fn init(&self, char_uuid: BtUuid, attr_handle: u16) {
            let uuid = u128::from_ne_bytes(*char_uuid.as_bytes().as_array::<16>().unwrap());
            match uuid {
                POSITION => self.position.store(attr_handle, Ordering::Relaxed),
                SPEED => self.speed.store(attr_handle, Ordering::Relaxed),
                _ => unreachable!(),
            }
        }

        pub fn characteristic(&self, handle: u16) -> Option<Characteristic> {
            if handle == self.position.load(Ordering::Relaxed) {
                Some(Characteristic::Position)
            } else if handle == self.speed.load(Ordering::Relaxed) {
                Some(Characteristic::Speed)
            } else {
                None
            }
        }
    }
}

#[derive(Clone)]
pub struct Bluetooth {
    gap: Arc<EspBleGap<'static, Ble, Arc<BtDriver<'static, Ble>>>>,
    gatts: Arc<EspGatts<'static, Ble, Arc<BtDriver<'static, Ble>>>>,
    handles: Arc<CharacteristicHandles>,
    clients: Arc<Mutex<HashSet<u16>>>,
}

pub fn init(app: Arc<App>, modem: Modem<'static>) -> Result<()> {
    let nvs = EspDefaultNvsPartition::take()?;
    let driver = Arc::new(BtDriver::<Ble>::new(modem, Some(nvs))?);

    let bt = Bluetooth {
        gap: Arc::new(EspBleGap::new(driver.clone())?),
        gatts: Arc::new(EspGatts::new(driver.clone())?),
        handles: Arc::new(CharacteristicHandles::default()),
        clients: Arc::new(Mutex::new(HashSet::new())),
    };

    bt.gap.subscribe(clone!([bt], move |event| {
        if let BleGapEvent::AdvertisingConfigured(_) = event {
            bt.gap.start_advertising().unwrap();
            info!("Advertising started");
        }
    }))?;

    bt.gap.set_device_name("windlink").unwrap();
    bt.gap
        .set_adv_conf(&AdvConfiguration {
            include_name: true,
            include_txpower: true,
            flag: 0x06,
            service_uuid: Some(BtUuid::uuid128(SERVICE)),
            appearance: AppearanceCategory::NetworkDevice,
            ..Default::default()
        })
        .unwrap();

    bt.gatts
        .subscribe(clone!([app, bt], move |(gatt_if, event)| match event {
            GattsEvent::ServiceRegistered { app_id, .. } if app_id == APP_ID => {
                let service = GattServiceId {
                    id: GattId {
                        uuid: BtUuid::uuid128(SERVICE),
                        inst_id: 0,
                    },
                    is_primary: true,
                };
                bt.gatts.create_service(gatt_if, &service, 10).unwrap();
            }
            GattsEvent::ServiceCreated { service_handle, .. } => {
                bt.gatts.start_service(service_handle).unwrap();

                for &uuid in characteristics::ALL {
                    let characteristic = GattCharacteristic {
                        uuid: BtUuid::uuid128(uuid),
                        permissions: Permission::Read.into(),
                        properties: Property::Read | Property::Notify,
                        max_len: 20,
                        auto_rsp: AutoResponse::ByApp,
                    };
                    bt.gatts
                        .add_characteristic(service_handle, &characteristic, &[])
                        .unwrap();
                }
            }
            GattsEvent::CharacteristicAdded {
                attr_handle,
                char_uuid,
                ..
            } => {
                bt.handles.init(char_uuid, attr_handle);
            }
            GattsEvent::PeerConnected { conn_id, .. } => {
                bt.clients.lock().unwrap().insert(conn_id);
            }
            GattsEvent::PeerDisconnected { conn_id, .. } => {
                bt.clients.lock().unwrap().remove(&conn_id);
            }
            GattsEvent::Read {
                conn_id,
                trans_id,
                handle,
                ..
            } => {
                let Some(characteristic) = bt.handles.characteristic(handle) else {
                    return;
                };

                let mut response = GattResponse::new();
                response.attr_handle(handle);

                let boat = app.boat();
                match characteristic {
                    Characteristic::Position => {
                        let msg = format!("{}, {}", boat.longitude, boat.latitude);
                        response.value(msg.as_bytes()).unwrap();
                    }
                    Characteristic::Speed => {
                        response
                            .value(boat.speed_over_ground.to_string().as_bytes())
                            .unwrap();
                    }
                }

                bt.gatts
                    .send_response(gatt_if, conn_id, trans_id, GattStatus::Ok, Some(&response))
                    .unwrap();
            }
            _ => {}
        }))?;

    bt.gatts.register_app(APP_ID)?;
    app.bt.lock().unwrap().replace(bt);
    info!("Initialized BLE");
    Ok(())
}
