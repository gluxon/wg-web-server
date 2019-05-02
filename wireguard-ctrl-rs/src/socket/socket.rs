use crate::attr::WgDeviceAttribute;
use crate::cmd::WgCmd;
use crate::consts::{WG_GENL_NAME, WG_GENL_VERSION};
use crate::err::{ConnectError, GetDeviceError, SetDeviceError};
use crate::get;
use crate::netlink;
use crate::rtnetlink;
use crate::set;
use crate::socket::parse::*;
use crate::socket::NlWgMsgType;
use libc::{IFNAMSIZ, NLMSG_ERROR};
use neli::ffi::{CtrlCmd, GenlId, NlFamily, NlmF};
use neli::genlhdr::GenlHdr;
use neli::nlattr::NlAttrHdr;
use neli::nlhdr::NlHdr;
use neli::socket::NlSocket;
use std::convert::TryInto;

type NlWgSocket = NlSocket<NlWgMsgType, GenlHdr<WgCmd>>;

pub struct Socket {
    sock: NlWgSocket,
    route_sock: netlink::Socket,
    seq: u32,
    family_id: NlWgMsgType,
}

pub enum GetDeviceArg<'a> {
    Ifindex(u32),
    Ifname(&'a str),
}

impl Socket {
    pub fn connect() -> Result<Self, ConnectError> {
        let family_id = {
            let mut nlsock = NlSocket::<GenlId, GenlHdr<CtrlCmd>>::new_genl()?;
            nlsock
                .resolve_genl_family(WG_GENL_NAME)
                .map_err(|err| ConnectError::ResolveFamilyError(err))?
        };

        let mut wgsock = NlWgSocket::new(NlFamily::Generic)?;

        // Autoselect a PID
        let pid = None;
        let groups = vec![];
        wgsock.bind(pid, groups)?;

        let route_sock = netlink::Socket::connect(0, 0)?;

        Ok(Self {
            sock: wgsock,
            route_sock,
            seq: 0,
            family_id,
        })
    }

    pub fn get_device(&mut self, interface: GetDeviceArg) -> Result<get::Device, GetDeviceError> {
        let attr = match interface {
            GetDeviceArg::Ifname(name) => {
                Some(name.len())
                    .filter(|&len| 0 < len && len < IFNAMSIZ)
                    .ok_or_else(|| GetDeviceError::InvalidInterfaceName)?;
                NlAttrHdr::new_str_payload(None, WgDeviceAttribute::Ifname, name)?
            }
            GetDeviceArg::Ifindex(index) => {
                NlAttrHdr::new_nl_payload(None, WgDeviceAttribute::Ifindex, index)?
            }
        };
        let genlhdr = {
            let cmd = WgCmd::GetDevice;
            let version = WG_GENL_VERSION;
            let attrs = vec![attr];
            GenlHdr::new::<WgDeviceAttribute>(cmd, version, attrs)?
        };
        let nlhdr = {
            let size = None;
            let nl_type = self.family_id;
            let flags = vec![NlmF::Request, NlmF::Ack, NlmF::Dump];
            let seq = Some(self.seq);
            let pid = None;
            let payload = genlhdr;
            NlHdr::new(size, nl_type, flags, seq, pid, payload)
        };

        self.seq += 1;
        self.sock.send_nl(nlhdr)?;

        // In the future, neli will return multiple Netlink messages. We have to go through each
        // message and coalesce peers in the way described by the WireGuard UAPI when this change
        // happens. For now, parsing is broken if the entire response doesn't fit in a single
        // payload.
        //
        // See: https://github.com/jbaublitz/neli/issues/15
        let res = self.sock.recv_nl(None)?;

        if i32::from(res.nl_type) == NLMSG_ERROR {
            return Err(GetDeviceError::AccessError);
        }

        let handle = res.nl_payload.get_attr_handle::<WgDeviceAttribute>();
        Ok(parse_device(handle)?)
    }

    pub fn set_device(&mut self, device: set::Device) -> Result<(), SetDeviceError> {
        let genlhdr = {
            let cmd = WgCmd::SetDevice;
            let version = WG_GENL_VERSION;
            let attrs = (&device).try_into()?;
            GenlHdr::new::<WgDeviceAttribute>(cmd, version, attrs)?
        };
        let nlhdr = {
            let size = None;
            let nl_type = self.family_id;
            let flags = vec![NlmF::Request, NlmF::Ack];
            let seq = Some(self.seq);
            let pid = None;
            let payload = genlhdr;
            NlHdr::new(size, nl_type, flags, seq, pid, payload)
        };

        self.seq += 1;
        self.sock.send_nl(nlhdr)?;
        self.sock.recv_ack(None)?;

        Ok(())
    }

    pub fn add_device(&self, name: &str) -> nix::Result<()> {
        let msg = netlink::NetlinkMessage {
            r#type: rtnetlink::RouteMessageType::NewLink as u16,
            flags: (libc::NLM_F_REQUEST + libc::NLM_F_EXCL + libc::NLM_F_CREATE) as u16,
            sequence: 0,
            port: 0,
            payload: rtnetlink::LinkMessage {
                family: 0,
                r#type: 0,
                index: 0,
                flags: 0,
                change: 0,
                attributes: vec![
                    netlink::NetlinkAttribute {
                        r#type: rtnetlink::InterfaceLinkAttribute::InterfaceName.into(),
                        payload: name.to_string().into(),
                    },
                    netlink::NetlinkAttribute {
                        r#type: rtnetlink::InterfaceLinkAttribute::LinkInfo.into(),
                        payload: vec![netlink::NetlinkAttribute {
                            r#type: rtnetlink::LinkInfoAttribute::Kind.into(),
                            payload: "wireguard".to_string().into(),
                        }]
                        .into(),
                    },
                ],
            },
        };

        self.route_sock.send(&msg)?;

        Ok(())
    }

    pub fn delete_device(&self, name: &str) -> nix::Result<()> {
        let msg = netlink::NetlinkMessage {
            r#type: rtnetlink::RouteMessageType::DelLink as u16,
            flags: libc::NLM_F_REQUEST as u16,
            sequence: 0,
            port: 0,
            payload: rtnetlink::LinkMessage {
                family: 0,
                r#type: 0,
                index: 0,
                flags: 0,
                change: 0,
                attributes: vec![netlink::NetlinkAttribute {
                    r#type: rtnetlink::InterfaceLinkAttribute::InterfaceName.into(),
                    payload: name.to_string().into(),
                }],
            },
        };

        self.route_sock.send(&msg)?;

        Ok(())
    }
}
