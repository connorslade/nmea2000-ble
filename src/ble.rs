use std::sync::{Arc, Mutex};

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
                Handle, Permission, Property,
                server::{EspGatts, GattsEvent},
            },
        },
    },
    nvs::EspDefaultNvsPartition,
};
use log::info;
use uuid::uuid;

const APP_ID: u16 = 0;
const SERVICE_UUID: u128 = uuid!("69da49f6-9646-4231-9d79-02b08a41e5b6").as_u128();
const CHAR_UUID: u128 = uuid!("300b2aec-a094-43fb-98ff-04917cf7a2fb").as_u128();

pub fn init(modem: Modem<'static>) -> Result<()> {
    let nvs = EspDefaultNvsPartition::take()?;

    let bt = Arc::new(BtDriver::<Ble>::new(modem, Some(nvs))?);
    let gap = Arc::new(EspBleGap::new(bt.clone())?);
    let gatts = Arc::new(EspGatts::new(bt.clone())?);

    let char_handle: Arc<Mutex<Option<Handle>>> = Arc::new(Mutex::new(None));

    gap.subscribe(clone!([gap], move |event| {
        if let BleGapEvent::AdvertisingConfigured(_) = event {
            gap.start_advertising().unwrap();
            info!("Advertising started");
        }
    }))?;

    gatts.subscribe(clone!([gap, gatts, char_handle], move |(
        gatt_if,
        event,
    )| match event {
        GattsEvent::ServiceRegistered { app_id, .. } if app_id == APP_ID => {
            gap.set_device_name("NMEA2000").unwrap();
            gap.set_adv_conf(&AdvConfiguration {
                include_name: true,
                include_txpower: true,
                flag: 0x06,
                service_uuid: Some(BtUuid::uuid128(SERVICE_UUID)),
                appearance: AppearanceCategory::NetworkDevice,
                ..Default::default()
            })
            .unwrap();

            let service = GattServiceId {
                id: GattId {
                    uuid: BtUuid::uuid128(SERVICE_UUID),
                    inst_id: 0,
                },
                is_primary: true,
            };
            gatts.create_service(gatt_if, &service, 3).unwrap();
        }
        GattsEvent::ServiceCreated { service_handle, .. } => {
            gatts.start_service(service_handle).unwrap();
            gatts
                .add_characteristic(
                    service_handle,
                    &GattCharacteristic {
                        uuid: BtUuid::uuid128(CHAR_UUID),
                        permissions: Permission::Read.into(),
                        properties: Property::Read.into(),
                        max_len: 20,
                        auto_rsp: AutoResponse::ByApp,
                    },
                    &[],
                )
                .unwrap();
        }
        GattsEvent::CharacteristicAdded { attr_handle, .. } => {
            *char_handle.lock().unwrap() = Some(attr_handle);
            info!("characteristic handle: {attr_handle}");
        }
        GattsEvent::PeerConnected { addr, .. } => {
            info!("Client connected: {addr}");
        }
        GattsEvent::PeerDisconnected { addr, .. } => {
            info!("Client disconnected: {addr}");
            gap.start_advertising().unwrap();
        }
        GattsEvent::Read {
            conn_id,
            trans_id,
            handle,
            ..
        } => {
            let mut rsp = GattResponse::new();
            rsp.attr_handle(handle).value(b"hello world!").unwrap();
            gatts
                .send_response(gatt_if, conn_id, trans_id, GattStatus::Ok, Some(&rsp))
                .unwrap();
        }
        _ => {}
    }))?;

    gatts.register_app(APP_ID)?;
    info!("Initialized BLE");
    Ok(())
}
