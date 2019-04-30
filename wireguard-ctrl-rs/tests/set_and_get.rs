use base64;
use failure;
use libc;
use std::time::Duration;
use wireguard_ctrl_rs;
use wireguard_ctrl_rs::socket::{GetDeviceArg, Socket};
use wireguard_ctrl_rs::{get, set};

fn parse_device_key(buf: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    key.copy_from_slice(&buf);
    key
}

#[test]
/// This test requires that the "wgtest0" interface already exists.
///
///   - Create it with: ip link add wgtest0 type wireguard
///   - Remove it with: ip link del wgtest0
fn simple() -> Result<(), failure::Error> {
    let mut test_device = get::Device {
        ifindex: 0,
        ifname: "wgtest0".to_string(),
        private_key: parse_device_key(&base64::decode(
            "EHhtoXVXpnXz31cx8nrAxQfvaRqe1vf343GVSyEtqUU=",
        )?),
        public_key: parse_device_key(&base64::decode(
            "MhBzmIBrzw8b8iF2FH4ejh/7Vumn6Q/KoR0H5+o7mlY=",
        )?),
        listen_port: 1234,
        fwmark: 0,
        peers: vec![
            get::Peer {
                public_key: parse_device_key(&base64::decode(
                    "DNeiCuVE2CuDy9QH3K3/egRK1rdn/oThlPtWNc4FfSw=",
                )?),
                preshared_key: [0u8; 32],
                endpoint: Some("[::1]:8080".parse()?),
                persistent_keepalive_interval: 0,
                last_handshake_time: Duration::new(0, 0),
                rx_bytes: 0,
                tx_bytes: 0,
                allowed_ips: vec![
                    get::AllowedIp {
                        family: libc::AF_INET as u16,
                        ipaddr: "10.24.24.1".parse()?,
                        cidr_mask: 32,
                    },
                    get::AllowedIp {
                        family: libc::AF_INET as u16,
                        ipaddr: "10.24.25.0".parse()?,
                        cidr_mask: 24,
                    },
                ],
                protocol_version: 1,
            },
            get::Peer {
                public_key: parse_device_key(&base64::decode(
                    "6KUaqULa+M6JI+b6DP3p0ZZZWyClN7ioMpYJp0kNFxQ=",
                )?),
                preshared_key: parse_device_key(&base64::decode(
                    "cMeE5GWUzUbvxbnBKco2MwAnW78nsk8vr04+KupVFkQ=",
                )?),
                endpoint: Some("127.0.0.1:12345".parse()?),
                persistent_keepalive_interval: 60,
                last_handshake_time: Duration::new(0, 0),
                rx_bytes: 0,
                tx_bytes: 0,
                allowed_ips: vec![get::AllowedIp {
                    family: libc::AF_INET6 as u16,
                    ipaddr: "::1".parse()?,
                    cidr_mask: 128,
                }],
                protocol_version: 1,
            },
        ],
    };

    let response_device = {
        let mut wg = Socket::connect()?;

        let set_device_args = set::Device::from_ifname(&test_device.ifname)
            .private_key(&test_device.private_key)
            .listen_port(test_device.listen_port)
            .flags(vec![set::WgDeviceF::ReplacePeers])
            .peers(vec![
                set::Peer::from_public_key(&test_device.peers[0].public_key)
                    .endpoint(test_device.peers[0].endpoint.as_ref().unwrap())
                    .allowed_ips(test_device.peers[0].allowed_ips.clone()),
                set::Peer::from_public_key(&test_device.peers[1].public_key)
                    .preshared_key(&test_device.peers[1].preshared_key)
                    .endpoint(test_device.peers[1].endpoint.as_ref().unwrap())
                    .persistent_keepalive_interval(
                        test_device.peers[1].persistent_keepalive_interval,
                    )
                    .allowed_ips(test_device.peers[1].allowed_ips.clone()),
            ]);

        wg.set_device(set_device_args)?;
        wg.get_device(GetDeviceArg::Ifname(&test_device.ifname))?
    };

    // The ifindex can't be determined before response_device is set. So we'll just copy over the
    // newly generated index before testing that what we sent matches what was returned.
    test_device.ifindex = response_device.ifindex;

    assert_eq!(test_device, response_device);

    Ok(())
}
