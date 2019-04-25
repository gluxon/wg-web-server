use base64;
use colored::*;
use failure;
use std::env;
use wireguard_uapi::get::{AllowedIp, Device, Peer};
use wireguard_uapi::socket;

fn main() -> Result<(), failure::Error> {
    let mut args = env::args();
    let _prog_name = args.next();
    let ifname = args.next().expect("Please provide an interface name");

    let mut wg = socket::Socket::connect()?;
    let device = wg.get_device(socket::GetDeviceArg::Ifname(&ifname))?;

    print_device(&device);

    Ok(())
}

fn print_device(device: &Device) {
    println!("{}: {}", "interface".green(), device.ifname.green());
    println!(
        "  {}: {}",
        "public key".black().bold(),
        base64::encode(&device.public_key)
    );
    println!("  {}: {}", "listen port".black().bold(), device.listen_port);

    for peer in &device.peers {
        println!();
        print_peer(&peer);
    }
}

fn print_peer(peer: &Peer) {
    println!(
        "{}: {}",
        "peer".yellow(),
        base64::encode(&peer.public_key).yellow()
    );
    if let Some(endpoint) = peer.endpoint {
        println!("  {}: {}", "endpoint".black().bold(), endpoint);
    }

    print!("  {}: ", "allowed ips".black().bold());
    for (i, allowed_ip) in peer.allowed_ips.iter().enumerate() {
        print_allowed_ip(allowed_ip);
        if i < peer.allowed_ips.len() - 1 {
            print!(", ");
        }
    }
    println!();
}

fn print_allowed_ip(allowed_ip: &AllowedIp) {
    print!(
        "{}{}{}",
        allowed_ip.ipaddr,
        "/".cyan(),
        allowed_ip.cidr_mask
    );
}
