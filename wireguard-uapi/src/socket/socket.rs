use crate::attr::WgDeviceAttribute;
use crate::cmd::WgCmd;
use crate::consts::{WG_GENL_NAME, WG_GENL_VERSION};
use crate::err::{ConnectError, GetDeviceError};
use crate::get::Device;
use crate::socket::parse::*;
use crate::socket::NlWgMsgType;
use libc::IFNAMSIZ;
use neli::ffi::{CtrlCmd, GenlId, NlFamily, NlmF};
use neli::genlhdr::GenlHdr;
use neli::nlattr::NlAttrHdr;
use neli::nlhdr::NlHdr;
use neli::socket::NlSocket;

type NlWgSocket = NlSocket<NlWgMsgType, GenlHdr<WgCmd>>;

pub struct Socket {
    sock: NlWgSocket,
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
            nlsock.resolve_genl_family(WG_GENL_NAME)
                .map_err(|err| ConnectError::ResolveFamilyError(err))?
        };

        let mut wgsock = NlWgSocket::new(NlFamily::Generic)?;

        // Autoselect a PID
        let pid = None;
        let groups = vec![];
        wgsock.bind(pid, groups)?;

        Ok(Self {
            sock: wgsock,
            seq: 0,
            family_id,
        })
    }

    pub fn get_device(&mut self, interface: GetDeviceArg) -> Result<Device, GetDeviceError> {
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

        let res = self.sock.recv_nl(None)?;
        let handle = res.nl_payload.get_attr_handle::<WgDeviceAttribute>();
        Ok(parse_device(handle)?)
    }
}
