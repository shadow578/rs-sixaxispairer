use std::error::Error;

/// A struct representing a MAC address.
pub struct MACAddress([u8; 6]);

impl std::fmt::Display for MACAddress {
    /// Format the MAC address as a string.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = self.0;
        write!(
            f,
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]
        )
    }
}

impl MACAddress {
    /// Create a MACAddress struct from a byte array.
    pub fn from_bytes(bytes: [u8; 6]) -> MACAddress {
        MACAddress(bytes)
    }

    /// Parse a MAC address string of format "xx:xx:xx:xx:xx:xx" into a MACAddress struct.
    pub fn from_string(mac: &str) -> Result<MACAddress, Box<dyn Error>> {
        let mut bytes = [0; 6];
        let mut i = 0;

        let byte_strs: Vec<&str> = mac.split(':').collect();
        if byte_strs.len() != 6 {
            return Err(Box::from(format!(
                "Invalid number of bytes. Expected 6 bytes, got {}",
                byte_strs.len()
            )));
        }

        for byte in mac.split(':') {
            let b = u8::from_str_radix(byte, 16);
            if b.is_err() {
                return Err(Box::from(format!(
                    "Invalid character at position #{} ('{}')",
                    i + 1,
                    byte
                )));
            }

            bytes[i] = b.unwrap();
            i += 1;
        }

        Ok(MACAddress(bytes))
    }

    /// Get the MAC address as a byte array.
    pub fn as_bytes(&self) -> [u8; 6] {
        self.0
    }
}
