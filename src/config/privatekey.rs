use base64;
use failure;
use rand_os::OsRng;
use std::fmt;
use std::str::FromStr;
use x25519_dalek::StaticSecret;

pub struct PrivateKey(StaticSecret);

impl PrivateKey {
    pub fn new() -> Result<Self, failure::Error> {
        let mut os_rng = OsRng::new()?;
        Ok(Self(StaticSecret::new(&mut os_rng)))
    }
}

impl FromStr for PrivateKey {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut decoded = [0u8; 32];
        let written = base64::decode_config_slice(&s, base64::STANDARD, &mut decoded)?;

        if written != 32 {
            return Err(InvalidLengthError.into());
        }

        Ok(Self(x25519_dalek::StaticSecret::from(decoded)))
    }
}

impl fmt::Display for PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", base64::encode(&self.0.to_bytes()))
    }
}

#[derive(Debug, failure::Fail)]
#[fail(display = "private keys must be exactly 32 bytes long")]
struct InvalidLengthError;
