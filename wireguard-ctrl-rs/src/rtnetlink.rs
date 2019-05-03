use crate::netlink::{NetlinkAttribute, NetlinkAttributePayload};

// For RTM_NEWLINK, RTM_DELLINK, RTM_GETLINK
#[derive(Clone)]
pub struct LinkMessage {
    pub family: u8,
    pub r#type: u16,
    pub index: u32,
    pub flags: u32,
    pub change: u32,
    pub attributes: Vec<NetlinkAttribute>,
}

impl From<LinkMessage> for Vec<u8> {
    fn from(message: LinkMessage) -> Self {
        let mut bytes = vec![];
        // https://www.infradead.org/~tgr/libnl/doc/route.html#_link_message_format
        bytes.extend(message.family.to_ne_bytes().iter());
        bytes.extend([0u8; 1].iter());
        bytes.extend(message.r#type.to_ne_bytes().iter());
        bytes.extend(message.index.to_ne_bytes().iter());
        bytes.extend(message.flags.to_ne_bytes().iter());
        bytes.extend(message.change.to_ne_bytes().iter());

        let payload: NetlinkAttributePayload = message.attributes.into();
        bytes.extend(payload.0.iter());

        bytes
    }
}

// https://github.com/torvalds/linux/blob/9a76aba0/include/uapi/linux/rtnetlink.h#L24
#[repr(u16)]
pub enum MessageType {
    NewLink = 16,
    DelLink = 17,
    GetLink = 18,
    SetLink = 19,
    NewAddr = 20,
    DelAddr = 21,
    GetAddr = 22,
    NewRoute = 24,
    DelRoute = 25,
    GetRoute = 26,
}

// https://github.com/torvalds/linux/blob/600d7258316d87cf9ecd58b6fdc8a35deca0870c/include/uapi/linux/if_link.h#L106
#[repr(u16)]
pub enum InterfaceLinkAttribute {
    Unspec = 0,
    InterfaceName = 3,
    LinkInfo = 18,
}

impl From<InterfaceLinkAttribute> for u16 {
    fn from(attr: InterfaceLinkAttribute) -> Self {
        attr as u16
    }
}

#[repr(u16)]
pub enum LinkInfoAttribute {
    Unspec = 0,
    Kind = 1,
}

impl From<LinkInfoAttribute> for u16 {
    fn from(attr: LinkInfoAttribute) -> Self {
        attr as u16
    }
}

// RTM_NEWADDR, RTM_DELADDR, RTM_GETADDR
#[derive(Clone)]
pub struct RouteMessage {
    pub family: u8,
    pub destination_len: u8,
    pub source_len: u8,
    pub tos: u8,
    pub table: u8,
    pub protocol: u8,
    pub scope: u8,
    pub r#type: u8,
    pub flags: u32,
    pub attributes: Vec<NetlinkAttribute>
}

impl From<RouteMessage> for Vec<u8> {
    fn from(message: RouteMessage) -> Self {
        let mut bytes = vec![
            message.family.clone(),
            message.destination_len.clone(),
            message.source_len.clone(),
            message.tos.clone(),
            message.table.clone(),
            message.protocol.clone(),
            message.scope.clone(),
            message.r#type.clone(),
        ];
        bytes.extend(message.flags.to_ne_bytes().iter());

        let payload: NetlinkAttributePayload = message.attributes.into();
        bytes.extend(payload.0.iter());

        bytes
    }
}

#[repr(u16)]
pub enum RouteMessageType {
    Unspec = 0,
    Unicast = 1,
    Local = 2,
    Broadcast = 3,
    Anycast = 4,
    Multicast = 5,
    BlackHole = 6,
    Unreachable = 7,
    Prohibit = 8,
    Throw = 9,
    Nat = 10,
    ExternalResolve = 11,
}

#[repr(u8)]
pub enum RouteMessageProtocol {
    Unspec = 0,
    Redirect = 1,
    Kernel = 2,
    Boot = 3,
    Static = 4,
}

#[repr(u8)]
pub enum RouteTable {
    Unspec = 0,
    Compat = 252,
    Default = 253,
    Main = 254,
    Local = 255,
}

#[repr(u8)]
pub enum RouteScope {
    Universe = 0,
    Site = 200,
    Link = 253,
    Host = 254,
    Nowhere = 255,
}

// https://github.com/torvalds/linux/blob/9a76aba0/include/uapi/linux/rtnetlink.h#L312
#[repr(u16)]
pub enum RouteMessageAttributeType {
    Unspec = 0,
    Destination = 1,
    Source = 2,
    InputInterface = 3,
    OutputInterface = 4,
    Gateway = 5,
    Priority = 6,
    PreferenceSource = 7,
    Metrics = 8,
    Multipath = 9,
    ProtocolInfo = 10,
    Flow = 11,
    CacheInfo = 12,
}
