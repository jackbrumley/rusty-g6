//! Firmware version query: `5a 07 01 02`, answered with `5a 07 [len]` and
//! the version as ASCII (e.g. "2.1.250903.1324").
use super::{packet, CommandFamily};

pub fn build_firmware_query() -> Vec<u8> {
    packet(CommandFamily::FirmwareQuery, &[0x01, 0x02])
}

/// The firmware version in a `0x07` answer, if it carries one.
pub fn parse_firmware_response(response: &[u8]) -> Option<String> {
    if response.get(1) != Some(&0x07) {
        return None;
    }
    let length = usize::from(*response.get(2)?);
    let text = response.get(3..3 + length)?;
    let end = text
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(text.len());
    let version = String::from_utf8_lossy(&text[..end]).trim().to_string();
    (!version.is_empty()).then_some(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_read_from_the_answer() {
        let mut response = b"\x5a\x07\x102.1.250903.1324\x00".to_vec();
        response.resize(64, 0);
        assert_eq!(
            parse_firmware_response(&response),
            Some("2.1.250903.1324".to_string())
        );
        assert_eq!(parse_firmware_response(&[0x5a, 0x2c, 0x05]), None);
    }
}
