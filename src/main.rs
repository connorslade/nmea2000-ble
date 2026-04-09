use esp_idf_svc::log::EspLogger;

fn main() {
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();

    log::info!("Hello, world!");
}
