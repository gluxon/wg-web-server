use failure::Fail;
use neli::err::NlError;
use nix;

#[derive(Fail, Debug)]
pub enum ConnectError {
    #[fail(display = "{}", _0)]
    NlError(#[fail(cause)] NlError),

    #[fail(display = "{}", _0)]
    NixError(#[fail(cause)] nix::Error),

    #[fail(display = "Unable to connect to the WireGuard DKMS. Is WireGuard installed?")]
    ResolveFamilyError(#[fail(cause)] NlError),
}

impl From<NlError> for ConnectError {
    fn from(error: NlError) -> Self {
        ConnectError::NlError(error)
    }
}

impl From<nix::Error> for ConnectError {
    fn from(error: nix::Error) -> Self {
        ConnectError::NixError(error)
    }
}

impl From<std::io::Error> for ConnectError {
    fn from(error: std::io::Error) -> Self {
        ConnectError::NlError(error.into())
    }
}
