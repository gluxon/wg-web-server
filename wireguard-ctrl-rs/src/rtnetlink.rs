use crate::netlink;

// For RTM_NEWLINK, RTM_DELLINK, RTM_GETLINK
#[derive(Clone)]
pub struct LinkMessage {
    pub family: u8,
    pub r#type: u16,
    pub index: u32,
    pub flags: u32,
    pub change: u32,
    pub attributes: Vec<netlink::NetlinkAttribute>,
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

        let payload: netlink::NetlinkAttributePayload = message.attributes.into();
        bytes.extend(payload.0.iter());

        bytes
    }
}

#[repr(u16)]
pub enum RouteMessageType {
    NewLink = 16,
    DelLink = 17,
    GetLink = 18,
    SetLink = 19,
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
