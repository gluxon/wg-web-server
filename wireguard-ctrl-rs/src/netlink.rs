/// Netlink Route packages on crates.io aren't very mature at the moment. This is a quick and dirty
/// private module for sending some simple messages.
///
/// The rtnetlink crate was pretty good, but the async design isn't necessary at the moment (since
/// the rocket framework that uses this crate isn't async yet) and there's an issue on its GitHub
/// repo about abandoning rtnetlink (as of 2019-05-02).

use nix;
use nix::sys::socket::{
    connect, recv, send, socket, AddressFamily, MsgFlags, SockAddr, SockFlag, SockType,
};
use std::mem::size_of;
use std::os::unix::io::RawFd;

pub use nix::Result;

pub struct Socket {
    fd: RawFd,
}

impl Socket {
    pub fn connect(pid: u32, groups: u32) -> nix::Result<Self> {
        let fd = {
            let domain = AddressFamily::Netlink;
            let ty = SockType::Raw;
            let flags = SockFlag::empty();
            let protocol = None;
            socket(domain, ty, flags, protocol)?
        };
        let addr = SockAddr::new_netlink(pid, groups);
        connect(fd, &addr)?;
        Ok(Self { fd })
    }

    pub fn send<P: Clone + Into<Vec<u8>>>(
        &self,
        message: &NetlinkMessage<P>,
    ) -> nix::Result<usize> {
        let buf: Vec<u8> = message.into();
        send(self.fd, &buf[..], MsgFlags::empty())
    }

    pub fn recv(&self) -> nix::Result<Vec<u8>> {
        let mut buf = vec![];
        let size = recv(self.fd, &mut buf, MsgFlags::MSG_PEEK | MsgFlags::MSG_TRUNC)?;
        buf.resize(size, 0);
        recv(self.fd, &mut buf, MsgFlags::empty())?;
        Ok(buf)
    }
}

// https://www.infradead.org/~tgr/libnl/doc/core.html#core_msg_format
#[derive(Clone)]
pub struct NetlinkMessage<P> {
    pub r#type: u16,
    pub flags: u16,
    pub sequence: u32,
    pub port: u32,
    pub payload: P,
}

const NETLINK_MESSAGE_HEADER_LEN: u32 = 16;

impl<P: Clone + Into<Vec<u8>>> From<&NetlinkMessage<P>> for Vec<u8> {
    fn from(message: &NetlinkMessage<P>) -> Self {
        let payload: Vec<u8> = message.payload.clone().into();
        let length: u32 = NETLINK_MESSAGE_HEADER_LEN + (payload.len() as u32);
        [
            &length.to_ne_bytes()[..],
            &message.r#type.to_ne_bytes()[..],
            &message.flags.to_ne_bytes()[..],
            &message.sequence.to_ne_bytes()[..],
            &message.port.to_ne_bytes()[..],
            &payload[..],
        ]
        .concat()
    }
}

#[derive(Clone)]
pub struct NetlinkAttribute {
    pub r#type: u16,
    pub payload: NetlinkAttributePayload,
}

#[derive(Clone)]
pub struct NetlinkAttributePayload(pub Vec<u8>);

impl From<String> for NetlinkAttributePayload {
    fn from(string: String) -> Self {
        let mut bytes = string.into_bytes();
        bytes.push(0);
        Self(bytes)
    }
}

impl From<Vec<NetlinkAttribute>> for NetlinkAttributePayload {
    fn from(attributes: Vec<NetlinkAttribute>) -> Self {
        let mut bytes = vec![];
        for attribute in &attributes {
            let attribute_length = (size_of::<u16>() * 2 + attribute.payload.0.len()) as u16;
            bytes.extend(attribute_length.to_ne_bytes().iter());
            bytes.extend(attribute.r#type.to_ne_bytes().iter());
            bytes.extend(attribute.payload.0.iter());
            bytes.extend([0u8; 3][..((4 - attribute.payload.0.len() % 4) % 4)].iter());
        }
        Self(bytes)
    }
}
