mod connect_error;
pub use connect_error::ConnectError;

mod get_device_error;
pub use get_device_error::GetDeviceError;

mod parse_device_error;
pub use parse_device_error::ParseDeviceError;

mod parse_attribute_error;
pub use parse_attribute_error::{ParseAttributeError, ParseIpAddrError, ParseSockAddrError};

pub use neli::err::NlError;
