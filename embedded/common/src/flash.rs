use core::ptr::NonNull;
use std::mem::ManuallyDrop;

use esp_idf_hal::{
    gpio::{InputPin, OutputPin},
    spi::{SpiAnyPins, SpiDriver, config::DriverConfig},
    sys::{
        self, EspError, esp_flash_init, esp_flash_io_mode_t_SPI_FLASH_FASTRD as SPI_FLASH_FASTRD,
        esp_flash_speed_s_ESP_FLASH_40MHZ as ESP_FLASH_40MHZ, esp_flash_t,
        soc_periph_spi_clk_src_t_SPI_CLK_SRC_DEFAULT as SPI_CLK_SRC_DEFAULT,
        spi_bus_add_flash_device, spi_host_device_t_SPI2_HOST as SPI2_HOST,
    },
};

pub struct SpiFlash {
    chip: NonNull<sys::esp_flash_t>,
}

impl SpiFlash {
    pub fn new(
        spi: impl SpiAnyPins,
        sclk: impl OutputPin,
        sdo: impl OutputPin,
        sdi: impl InputPin,
        cs: impl OutputPin,
        freq_mhz: i32,
    ) -> Result<Self, EspError> {
        let _ = ManuallyDrop::new(SpiDriver::new(
            spi,
            sclk,
            sdo,
            Some(sdi),
            &DriverConfig::new(),
        )?);

        let cfg = sys::esp_flash_spi_device_config_t {
            host_id: SPI2_HOST,
            cs_io_num: cs.pin() as _,
            io_mode: SPI_FLASH_FASTRD,
            speed: ESP_FLASH_40MHZ,
            input_delay_ns: 0,
            cs_id: 0,
            freq_mhz,
            clock_source: SPI_CLK_SRC_DEFAULT,
        };

        let mut chip: *mut esp_flash_t = core::ptr::null_mut();
        unsafe {
            EspError::convert(spi_bus_add_flash_device(&mut chip, &cfg))?;
            EspError::convert(esp_flash_init(chip))?;
        }

        Ok(Self {
            chip: NonNull::new(chip).unwrap(),
        })
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut sys::esp_flash_t {
        self.chip.as_ptr()
    }

    pub fn read_id(&self) -> Result<u32, EspError> {
        let mut id: u32 = 0;
        unsafe {
            EspError::convert(sys::esp_flash_read_id(self.as_ptr(), &mut id))?;
        }
        Ok(id)
    }

    pub fn size(&self) -> Result<u32, EspError> {
        let mut size: u32 = 0;
        unsafe { EspError::convert(sys::esp_flash_get_physical_size(self.as_ptr(), &mut size))? };
        Ok(size)
    }

    pub fn read(&self, address: u32, buf: &mut [u8]) -> Result<(), EspError> {
        unsafe {
            EspError::convert(sys::esp_flash_read(
                self.as_ptr(),
                buf.as_mut_ptr().cast::<core::ffi::c_void>(),
                address,
                buf.len() as u32,
            ))?;
        }
        Ok(())
    }

    pub fn erase_region(&self, start: u32, len: u32) -> Result<(), EspError> {
        unsafe { EspError::convert(sys::esp_flash_erase_region(self.as_ptr(), start, len))? };
        Ok(())
    }

    pub fn write(&self, address: u32, data: &[u8]) -> Result<(), EspError> {
        unsafe {
            EspError::convert(sys::esp_flash_write(
                self.as_ptr(),
                data.as_ptr().cast::<core::ffi::c_void>(),
                address,
                data.len() as u32,
            ))?;
        }
        Ok(())
    }
}

impl Drop for SpiFlash {
    fn drop(&mut self) {
        let _ = unsafe { sys::spi_bus_remove_flash_device(self.as_ptr()) };
    }
}

unsafe impl Send for SpiFlash {}
