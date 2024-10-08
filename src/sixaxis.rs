// Note: this code is heavily based on the following projects:
// - https://github.com/user-none/sixaxispairer/blob/main/main.c for SixAxis protocol
// - https://github.com/SveinIsdahl/PS4-controller-pairer/blob/master/main.c for DualShock 4 protocol

use crate::mac::MACAddress;
use hidapi::{HidApi, HidDevice};
use std::error::Error;

/// A struct representing a USB device ID.
#[derive(Debug, Clone, Copy)]
pub struct USBDeviceId {
    /// USB Vendor ID.
    pub vendor: u16,

    /// USB Product ID.
    pub product: u16,
}

#[derive(Debug, Clone, Copy)]
pub enum SixAxisProtocol {
    /// Protocol used by the PS3 controller (presumably also the Move controller).
    SixAxis,

    /// Protocol used by the PS4 controller.
    DualShock4,
}

struct KnownDeviceRecord {
    /// Display name of the device. Used for logging.
    name: &'static str,

    /// USB Device ID.
    id: USBDeviceId,

    /// Protocol used by the device.
    protocol: SixAxisProtocol,
}

/// List of known devices supported by sixaxispairer.
const KNOWN_DEVICES: [KnownDeviceRecord; 3] = [
    KnownDeviceRecord {
        name: "Sony PlayStation 3 Controller",
        protocol: SixAxisProtocol::SixAxis,
        id: USBDeviceId {
            vendor: 0x054c,
            product: 0x0268,
        },
    },
    KnownDeviceRecord {
        name: "Sony Move Motion Controller",
        protocol: SixAxisProtocol::SixAxis,
        id: USBDeviceId {
            vendor: 0x054c,
            product: 0x042f,
        },
    },
    KnownDeviceRecord {
        name: "Sony DualShock 4 Controller",
        protocol: SixAxisProtocol::DualShock4,
        id: USBDeviceId {
            vendor: 0x054c,
            product: 0x05c4,
        },
    },
];

/// A struct representing a Sony Sixaxis controller.
/// This struct encapsulates the HID device and provides methods to interact with it.
pub struct SixAxisController {
    device: HidDevice,
    protocol: SixAxisProtocol,
}

impl SixAxisController {
    /// Connect to a Sony Sixaxis controller, creating a new SixAxisController instance.
    /// If a device ID is provided, only devices with a matching ID will be opened.
    /// protocol must be provided if device_id is provided. Otherwise, it may be omitted.
    pub fn open(
        device_id: Option<USBDeviceId>,
        protocol: Option<SixAxisProtocol>,
    ) -> Result<SixAxisController, Box<dyn Error>> {
        // initialize hidapi
        let api = HidApi::new();
        if api.is_err() {
            return Err(Box::from(api.err().unwrap()));
        }

        let api: HidApi = api.unwrap();

        // iterate over all devices
        for device in api.device_list() {
            let mut should_open = false;
            let mut protocol = protocol;

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
                        protocol = Some(known_device.protocol);
                        should_open = true;
                    }
                }
            }

            // if this is a supported device, open it
            if should_open {
                // ensure a protocol was provided
                if protocol.is_none() {
                    return Err(Box::from("Device found, but no protocol specified."));
                }
                let protocol = protocol.unwrap();

                // open the device
                let device = api.open(device.vendor_id(), device.product_id());
                if device.is_err() {
                    return Err(Box::from(device.err().unwrap()));
                }

                // all good, instantiate struct and return it
                let device = device.unwrap();
                return Ok(SixAxisController { device, protocol });
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
        match self.protocol {
            SixAxisProtocol::SixAxis => {
                // prepare report buffer
                let mut report: [u8; 8] = [0; 8];
                report[0] = 0xf5;

                // query the device
                let result = self.device.get_feature_report(&mut report);
                if result.is_err() {
                    return Err(Box::from(result.err().unwrap()));
                }

                // validate result and extract mac address
                let mac_bytes: [u8; 6] = report[2..8].try_into().unwrap();
                return Ok(MACAddress::from_bytes(mac_bytes));
            }
            SixAxisProtocol::DualShock4 => {
                // prepare report buffer
                let mut report: [u8; 16] = [0; 16];
                report[0] = 0x12;

                // query the device
                let result = self.device.get_feature_report(&mut report);
                if result.is_err() {
                    return Err(Box::from(result.err().unwrap()));
                }

                // validate result and extract mac address
                let mut mac_bytes: [u8; 6] = report[10..16].try_into().unwrap();

                // mac address bytes need to be reversed, since PS4 uses little-endian
                mac_bytes.reverse();
                return Ok(MACAddress::from_bytes(mac_bytes));
            }
        };
    }

    /// Set the MAC address of the controller.
    pub fn set_paired_mac(&self, mac: &MACAddress) -> Result<(), Box<dyn Error>> {
        match self.protocol {
            SixAxisProtocol::SixAxis => {
                // prepare report buffer
                let mut report = [0; 8];
                report[0] = 0xf5;
                report[1] = 0;
                report[2..8].copy_from_slice(&mac.as_bytes());

                // send the report
                let result = self.device.send_feature_report(&report);
                if result.is_err() {
                    return Err(Box::from(result.err().unwrap()));
                }

                return Ok(());
            }
            SixAxisProtocol::DualShock4 => {
                // mac address bytes need to be reversed, since PS4 uses little-endian
                let mut mac_bytes = mac.as_bytes();
                mac_bytes.reverse();

                // prepare report buffer
                let mut report = [0; 23];
                report[0] = 0x13;
                report[1..7].copy_from_slice(&mac_bytes);

                // 7..23 is a key, seems to be optional...

                // send the report
                let result = self.device.send_feature_report(&report);
                if result.is_err() {
                    return Err(Box::from(result.err().unwrap()));
                }

                return Ok(());
            }
        };
    }
}
