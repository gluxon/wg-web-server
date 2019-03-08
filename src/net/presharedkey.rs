use base64;
use failure;
use std::fmt;
use std::str::FromStr;

pub struct PresharedKey([u8; 32]);

impl FromStr for PresharedKey {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut decoded = [0u8; 32];
        let written = base64::decode_config_slice(&s, base64::STANDARD, &mut decoded)?;

        if written != 32 {
            return Err(InvalidLengthError.into());
        }

        Ok(Self(decoded))
    }
}

impl fmt::Display for PresharedKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", base64::encode(&self.0))
    }
}

#[derive(Debug, failure::Fail)]
#[fail(display = "preshared keys must be exactly 32 bytes long")]
struct InvalidLengthError;
