use crate::config::Config;
use std::sync::{Mutex, MutexGuard};
use wireguard_uapi::err::ConnectError;
use wireguard_uapi::get::Device;
use wireguard_uapi::{DeviceInterface, WgSocket};

pub struct WgState {
    // Currently, a call to wireguard_uapi::Socket's get_device() method requires a mutable
    // reference to itself. There are 2 things that need to be resolved before we can remove the
    // mutable reference requirement and thus remove the Mutex below.
    //
    //   1. The wireguard_uapi::Socket current keeps track of an auto-incrementing sequence id. A
    //      stateless alternative would be to use the time, since these sequence ids just have to
    //      be increasing.
    //   2. The neli library socket object currently requires a mutable reference to itself when
    //      sending and receiving. This mutable reference isn't required an can be easily removed
    //      however. (In version 0.3.1)
    //
    // I was able to remove the Mutex by patching neli and removing sequence ids, but this was done
    // in a hacky way (and we need sequence ids).
    pub socket: Mutex<WgSocket>,
    interface_config: Config,
}

impl WgState {
    pub fn init(interface_config: Config) -> Result<Self, ConnectError> {
        Ok(Self {
            socket: Mutex::new(WgSocket::connect()?),
            interface_config,
        })
    }

    pub fn apply_config(&self) -> Result<(), failure::Error> {
        let mut guard = self.get_socket_guard()?;
        let socket = &mut *guard;

        socket.set_device((&self.interface_config).into())?;

        Ok(())
    }

    fn get_socket_guard(&self) -> Result<MutexGuard<WgSocket>, ConnectError> {
        match self.socket.lock() {
            Ok(guard) => Ok(guard),
            // If the mutex for the socket is poisoned, let's just grab a fresh new socket.
            Err(poisoned) => {
                let mut guard = poisoned.into_inner();
                *guard = WgSocket::connect()?;
                Ok(guard)
            }
        }
    }

    pub fn get_device(&self) -> Result<Device, failure::Error> {
        let mut guard = self.get_socket_guard()?;
        let socket = &mut *guard;
        let device = socket.get_device(DeviceInterface::from_name(&self.interface_config.name))?;
        Ok(device)
    }
}
