use crate::config;
use crate::config::Config;
use std::sync::{Mutex, MutexGuard};
use wireguard_uapi::err::ConnectError;
use wireguard_uapi::get::Device;
use wireguard_uapi::set;
use wireguard_uapi::set::WgPeerF;
use wireguard_uapi::{DeviceInterface, RouteSocket, WgSocket};

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
    pub wg_socket: Mutex<WgSocket>,
    pub route_socket: Mutex<RouteSocket>,
    interface_config: Config,
}

impl WgState {
    pub fn init(interface_config: Config) -> Result<Self, ConnectError> {
        Ok(Self {
            wg_socket: Mutex::new(WgSocket::connect()?),
            route_socket: Mutex::new(RouteSocket::connect()?),
            interface_config,
        })
    }

    pub fn apply_config(&self) -> Result<(), failure::Error> {
        let mut route_socket = self.get_route_socket_guard()?;
        let route_socket = &mut *route_socket;

        // TODO: Handle the result type and continue only if the error is because the device already
        // exists.
        let _ = route_socket.add_device(&self.interface_config.name);

        let mut wg_guard = self.get_wg_socket_guard()?;
        let wg_socket = &mut *wg_guard;

        wg_socket.set_device((&self.interface_config).into())?;

        Ok(())
    }

    fn get_wg_socket_guard(&self) -> Result<MutexGuard<WgSocket>, ConnectError> {
        match self.wg_socket.lock() {
            Ok(guard) => Ok(guard),
            // If the mutex for the socket is poisoned, let's just grab a fresh new socket.
            Err(poisoned) => {
                let mut guard = poisoned.into_inner();
                *guard = WgSocket::connect()?;
                Ok(guard)
            }
        }
    }

    fn get_route_socket_guard(&self) -> Result<MutexGuard<RouteSocket>, ConnectError> {
        match self.route_socket.lock() {
            Ok(guard) => Ok(guard),
            // If the mutex for the socket is poisoned, let's just grab a fresh new socket.
            Err(poisoned) => {
                let mut guard = poisoned.into_inner();
                *guard = RouteSocket::connect()?;
                Ok(guard)
            }
        }
    }

    pub fn get_device(&self) -> Result<Device, failure::Error> {
        let mut guard = self.get_wg_socket_guard()?;
        let socket = &mut *guard;
        let device = socket.get_device(DeviceInterface::from_name(&self.interface_config.name))?;
        Ok(device)
    }

    pub fn add_peer(&self, config_peer: config::Peer) -> Result<(), failure::Error> {
        let mut guard = self.get_wg_socket_guard()?;
        let socket = &mut *guard;

        let mut peer = set::Peer::from_public_key(config_peer.public_key.as_bytes())
            .flags(vec![WgPeerF::ReplaceAllowedIps]);

        if let Some(preshared_key) = &config_peer.preshared_key {
            peer = peer.preshared_key(preshared_key.as_bytes());
        }

        peer = peer.allowed_ips(
            config_peer
                .allowed_ips
                .0
                .iter()
                .map(|ip| ip.into())
                .collect(),
        );

        if let Some(endpoint) = &config_peer.endpoint {
            peer = peer.endpoint(endpoint);
        }

        if let Some(persistent_keepalive) = config_peer.persistent_keepalive {
            peer = peer.persistent_keepalive_interval(persistent_keepalive);
        }

        let device = wireguard_uapi::set::Device {
            interface: DeviceInterface::from_name(&self.interface_config.name),
            flags: vec![],
            private_key: None,
            listen_port: None,
            fwmark: None,
            peers: vec![peer],
        };
        socket.set_device(device)?;

        Ok(())
    }
}
