use crate::mac::MACAddress;
use std::error::Error;

/// Get the MAC address of the DS4 controller.
pub fn get_mac() -> Result<MACAddress, Box<dyn Error>> {
    Ok(MACAddress::from_bytes([0, 0, 0, 0, 0, 0]))
}

/// Set the MAC address of the DS4 controller.
pub fn set_mac(mac: &MACAddress) -> Result<(), Box<dyn Error>> {
    Ok(())
}
