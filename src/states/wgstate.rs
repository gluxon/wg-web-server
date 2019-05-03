use std::net::{IpAddr, Ipv4Addr};
use std::sync::{Mutex, MutexGuard};
use wireguard_ctrl_rs;
use wireguard_ctrl_rs::err::ConnectError;
use wireguard_ctrl_rs::get::Device;
use wireguard_ctrl_rs::socket::GetDeviceArg;

pub struct WgState {
    // Currently, a call to wireguard_ctrl_rs::Socket's get_device() method requires a mutable
    // reference to itself. There are 2 things that need to be resolved before we can remove the
    // mutable reference requirement and thus remove the Mutex below.
    //
    //   1. The wireguard_ctrl_rs::Socket current keeps track of an auto-incrementing sequence id. A
    //      stateless alternative would be to use the time, since these sequence ids just have to
    //      be increasing.
    //   2. The neli library socket object currently requires a mutable reference to itself when
    //      sending and receiving. This mutable reference isn't required an can be easily removed
    //      however. (In version 0.3.1)
    //
    // I was able to remove the Mutex by patching neli and removing sequence ids, but this was done
    // in a hacky way (and we need sequence ids).
    pub socket: Mutex<wireguard_ctrl_rs::Socket>,
    interface_name: String,
}

impl WgState {
    pub fn init(interface_name: String) -> Result<Self, ConnectError> {
        Ok(Self {
            socket: Mutex::new(wireguard_ctrl_rs::Socket::connect()?),
            interface_name,
        })
    }

    fn get_socket_guard(&self) -> Result<MutexGuard<wireguard_ctrl_rs::Socket>, ConnectError> {
        match self.socket.lock() {
            Ok(guard) => Ok(guard),
            // If the mutex for the socket is poisoned, let's just grab a fresh new socket.
            Err(poisoned) => {
                let mut guard = poisoned.into_inner();
                *guard = wireguard_ctrl_rs::Socket::connect()?;
                Ok(guard)
            }
        }
    }

    pub fn get_device(&self) -> Result<Device, failure::Error> {
        let mut guard = self.get_socket_guard()?;
        let socket = &mut *guard;
        let device = socket.get_device(GetDeviceArg::Ifname(&self.interface_name))?;
        Ok(device)
    }

    pub fn add_peer(&self, public_key: [u8; 32], ipaddr: Ipv4Addr) -> Result<(), failure::Error> {
        use wireguard_ctrl_rs::set::{Device, Peer, AllowedIp};
        let mut guard = self.get_socket_guard()?;
        let socket = &mut *guard;

        socket.set_device(Device::from_ifname(&self.interface_name)
            .peers(vec![
                Peer::from_public_key(&public_key)
                    .allowed_ips(vec![
                        AllowedIp {
                            ipaddr: &IpAddr::V4(ipaddr.clone()),
                            cidr_mask: None
                        }
                    ])
            ]))?;

        socket.new_route(&self.interface_name, ipaddr)?;

        Ok(())
    }
}
