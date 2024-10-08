use crate::mac::MACAddress;
use hidapi::{HidApi, HidDevice};
use std::error::Error;

/// A struct representing a USB device ID.
pub struct USBDeviceId {
    /// USB Vendor ID.
    pub vendor: u16,

    /// USB Product ID.
    pub product: u16,
}

struct KnownDeviceRecord {
    /// Display name of the device. Used for logging.
    name: &'static str,

    /// USB Device ID.
    id: USBDeviceId,
}

/// List of known devices supported by sixaxispairer.
const KNOWN_DEVICES: [KnownDeviceRecord; 2] = [
    KnownDeviceRecord {
        name: "Sony Sixaxis",
        id: USBDeviceId {
            vendor: 0x054c,
            product: 0x0268,
        },
    },
    KnownDeviceRecord {
        name: "Sony Move Motion",
        id: USBDeviceId {
            vendor: 0x054c,
            product: 0x042f,
        },
    },
];

/// HID feature report ID for getting or setting the paired MAC address.
/// Note: hidapi automatically ors this with (3 << 8), making the report id 0x03f5.
const PAIRED_MAC_REPORT_ID: u8 = 0xf5;

/// A struct representing a Sony Sixaxis controller.
/// This struct encapsulates the HID device and provides methods to interact with it.
pub struct SixAxisController {
    device: HidDevice,
}

impl SixAxisController {
    /// Connect to a Sony Sixaxis controller, creating a new SixAxisController instance.
    /// If a device ID is provided, only devices with a matching ID will be opened.
    pub fn open(device_id: Option<USBDeviceId>) -> Result<SixAxisController, Box<dyn Error>> {
        // initialize hidapi
        let api = HidApi::new();
        if api.is_err() {
            return Err(Box::from(api.err().unwrap()));
        }

        let api: HidApi = api.unwrap();

        // iterate over all devices
        for device in api.device_list() {
            let mut should_open = false;

            if let Some(device_id) = &device_id {
                // if a device ID was provided, check if the current device matches
                if device.vendor_id() == device_id.vendor
                    && device.product_id() == device_id.product
                {
                    println!(
                        "Found device: (VID={:04X}, PID={:04X}",
                        device_id.vendor, device_id.product
                    );
                    should_open = true;
                }
            } else {
                // no device ID provided, check if the current device is a known device
                for known_device in KNOWN_DEVICES.iter() {
                    if device.vendor_id() == known_device.id.vendor
                        && device.product_id() == known_device.id.product
                    {
                        println!(
                            "Found device: {} (VID={:04X}, PID={:04X}",
                            known_device.name, known_device.id.vendor, known_device.id.product
                        );
                        should_open = true;
                    }
                }
            }

            // if this is a supported device, open it
            if should_open {
                let device = api.open(device.vendor_id(), device.product_id());
                if device.is_err() {
                    return Err(Box::from(device.err().unwrap()));
                }

                // all good, instantiate struct and return it
                let device = device.unwrap();
                return Ok(SixAxisController { device });
            }
        }

        return Err(Box::from("No supported devices found."));
    }

    /// Get the display name of the controller.
    /// This is a combination of the manufacturer and product strings.
    /// The serial number is also included if include_serial is true.
    pub fn get_display_name(&self, include_serial: Option<bool>) -> String {
        // get manufacturer and product strings, default to "Unknown" if not available
        let manufacturer = self
            .device
            .get_manufacturer_string()
            .unwrap_or(None)
            .unwrap_or("?".to_owned());

        let product = self
            .device
            .get_product_string()
            .unwrap_or(None)
            .unwrap_or("?".to_owned());

        if include_serial.unwrap_or(false) {
            let serial = self
                .device
                .get_serial_number_string()
                .unwrap_or(None)
                .unwrap_or("?".to_owned());

            return format!("{} {} ({})", manufacturer, product, serial);
        }

        return format!("{} {}", manufacturer, product);
    }

    /// Get the MAC address of the controller.
    pub fn get_paired_mac(&self) -> Result<MACAddress, Box<dyn Error>> {
        // prepare report buffer
        let mut report = [0; 8];
        report[0] = PAIRED_MAC_REPORT_ID;

        // query the device
        let result = self.device.get_feature_report(&mut report);
        if result.is_err() {
            return Err(Box::from(result.err().unwrap()));
        }

        // result is 2 bytes header, then the currently paired mac address (6 bytes)
        let result_len = result.unwrap();
        if result_len != 8 {
            return Err(Box::from("Invalid response length."));
        }

        // validate header bytes
        if report[0] != PAIRED_MAC_REPORT_ID || report[1] != 0 {
            return Err(Box::from("Invalid response header."));
        }

        // extract mac address
        let mac_bytes = report[2..8].try_into().unwrap();
        return Ok(MACAddress::from_bytes(mac_bytes));
    }

    /// Set the MAC address of the controller.
    pub fn set_paired_mac(&self, mac: &MACAddress) -> Result<(), Box<dyn Error>> {
        // prepare report buffer
        let mut report = [0; 8];
        report[0] = PAIRED_MAC_REPORT_ID;
        report[1] = 0;
        report[2..8].copy_from_slice(&mac.as_bytes());

        // send the report
        let result = self.device.send_feature_report(&report);
        if result.is_err() {
            return Err(Box::from(result.err().unwrap()));
        }

        Ok(())
    }
}
