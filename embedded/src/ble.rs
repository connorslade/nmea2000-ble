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

const APP_ID: u16 = 0;
const SERVICE: u128 = uuid!("69da49f6-9646-4231-9d79-02b08a41e5b6").as_u128();

mod characteristics {
    use std::sync::atomic::{AtomicU16, Ordering};

    use esp_idf_svc::bt::BtUuid;
    use uuid::uuid;

    pub const ALL: &[u128] = &[WIND, SPEED];

    pub const WIND: u128 = uuid!("300b2aec-a094-43fb-98ff-04917cf7a2fb").as_u128();
    pub const SPEED: u128 = uuid!("d948b9e5-6626-4d41-8967-c4dca26db1fd").as_u128();

    #[derive(Default)]
    pub struct CharacteristicHandles {
        wind: AtomicU16,
        speed: AtomicU16,
    }

    impl CharacteristicHandles {
        pub fn init(&self, char_uuid: BtUuid, attr_handle: u16) {
            let uuid = u128::from_ne_bytes(*char_uuid.as_bytes().as_array::<16>().unwrap());
            match uuid {
                WIND => self.wind.store(attr_handle, Ordering::Relaxed),
                SPEED => self.speed.store(attr_handle, Ordering::Relaxed),
                _ => unreachable!(),
            }
        }
    }
}

#[derive(Clone)]
struct Bluetooth {
    gap: Arc<EspBleGap<'static, Ble, Arc<BtDriver<'static, Ble>>>>,
    gatts: Arc<EspGatts<'static, Ble, Arc<BtDriver<'static, Ble>>>>,
    handles: Arc<CharacteristicHandles>,
    clients: Arc<Mutex<HashSet<u16>>>,
}

pub fn init(modem: Modem<'static>) -> Result<()> {
    let nvs = EspDefaultNvsPartition::take()?;
    let driver = Arc::new(BtDriver::<Ble>::new(modem, Some(nvs))?);

    let bt = Bluetooth {
        gap: Arc::new(EspBleGap::new(driver.clone())?),
        gatts: Arc::new(EspGatts::new(driver.clone())?),
        handles: Arc::new(CharacteristicHandles::default()),
        clients: Arc::new(Mutex::new(HashSet::new())),
    };

    bt.gap.set_device_name("NMEA2000").unwrap();
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
    bt.gap.subscribe(clone!([bt], move |event| {
        if let BleGapEvent::AdvertisingConfigured(_) = event {
            bt.gap.start_advertising().unwrap();
            info!("Advertising started");
        }
    }))?;

    bt.gatts
        .subscribe(clone!([bt], move |(gatt_if, event)| match event {
            GattsEvent::ServiceRegistered { app_id, .. } if app_id == APP_ID => {
                let service = GattServiceId {
                    id: GattId {
                        uuid: BtUuid::uuid128(SERVICE),
                        inst_id: 0,
                    },
                    is_primary: true,
                };
                bt.gatts.create_service(gatt_if, &service, 3).unwrap();
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
                let mut rsp = GattResponse::new();
                rsp.attr_handle(handle).value(b"hello world!").unwrap();
                bt.gatts
                    .send_response(gatt_if, conn_id, trans_id, GattStatus::Ok, Some(&rsp))
                    .unwrap();
            }
            _ => {}
        }))?;

    bt.gatts.register_app(APP_ID)?;
    info!("Initialized BLE");
    Ok(())
}
