use base64;
use failure;
use std::fmt;
use std::str::FromStr;
use x25519_dalek;

pub struct PublicKey(x25519_dalek::PublicKey);

impl PublicKey {
    #[inline]
    pub fn as_bytes(&self) -> &[u8; 32] {
        self.0.as_bytes()
    }
}

impl FromStr for PublicKey {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut decoded = [0u8; 32];
        let written = base64::decode_config_slice(&s, base64::STANDARD, &mut decoded)?;

        if written != 32 {
            return Err(InvalidLengthError.into());
        }

        Ok(Self(x25519_dalek::PublicKey::from(decoded)))
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", base64::encode(&self.0.as_bytes()))
    }
}

#[derive(Debug, failure::Fail)]
#[fail(display = "public keys must be exactly 32 bytes long")]
struct InvalidLengthError;
