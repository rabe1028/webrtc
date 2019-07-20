// https://tools.ietf.org/html/rfc4585

/*
    0                   1                   2                   3
    0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |V=2|P|   FMT   |       PT      |          length               |
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |                  SSRC of packet sender                        |
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |                  SSRC of media source                         |
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   :            Feedback Control Information (FCI)                 :
   :                                                               :

           Figure 3: Common Packet Format for Feedback Messages

   
   The Feedback Control Information (FCI) field has the following Syntax
   (Figure 4):

    0                   1                   2                   3
    0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |            PID                |             BLP               |
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

               Figure 4: Syntax for the Generic NACK message
*/

use crate::rtcp::{Result,RtcpError};

//use crate::{Result,Error};
use crate::octets;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RtcpRtpFeedbackPacket {
    format : u8,        // 1bytes
    ssrc : u32,         // 4bytes
    media_ssrc : u32,   // 4bytes
    lost : Option<Vec<u16>>,
}

impl RtcpRtpFeedbackPacket{

    pub fn get_length(&self) -> u32 {
        let mut b_length = 4 + 4;

        if let Some(ref v) = self.lost {
            let mut pid : u16 = v[0];
            let mut _blp : u16 = 0;
            for p in v[1..].iter() {
                let d : u16 = *p - pid - 1;
                if d < 16 { _blp |= 1 << d }
                else{
                    b_length += 4;
                    pid = *p;
                    _blp = 0;
                }
            }
            b_length += 4
        }

        b_length
    }

    pub fn get_format(&self) -> u8 {
        self.format
    }

    pub fn to_bytes(&self, out: &mut octets::Octets) -> Result<()>{
        out.put_u32(self.ssrc)?;
        out.put_u32(self.media_ssrc)?;

        if let Some(ref v) = self.lost {
            let mut pid : u16 = v[0];
            let mut blp : u16 = 0;
            for p in v[1..].iter() {
                let d : u16 = *p - pid - 1;
                if d < 16 { blp |= 1 << d }
                else{
                    out.put_u16(pid)?;
                    out.put_u16(blp)?;
                    pid = *p;
                    blp = 0;
                }
            }
            out.put_u16(pid)?;
            out.put_u16(blp)?;
        }

        Ok(())
    }

    pub fn from_bytes(bytes : &mut octets::Octets, format : u8) -> Result<RtcpRtpFeedbackPacket>{
        // 8bytes = ssrc + media_ssrc
        // packet length = 8 + 4 * k [bytes]
        // k is feedback control information counts
        if bytes.len() < 8 || bytes.len() % 4 != 0 {
            return Err(RtcpError::InvalidPacketLength)
        }

        let ssrc = bytes.get_u32()?;
        let media_ssrc = bytes.get_u32()?;

        let fci_count = (bytes.len() - 8) / 4;

        let lost = if fci_count > 0 {
            let mut tmp_lost = Vec::new();

            for _ in 0..fci_count {
                let pid = bytes.get_u16()?;
                let blp = bytes.get_u16()?;

                tmp_lost.push(pid);
                for d in 0..16 {
                    if (blp >> d) & 1 != 0 {
                        tmp_lost.push(pid + d + 1);
                    }
                }
            }

            Some(tmp_lost)
        } else {
            None
        };

        Ok(RtcpRtpFeedbackPacket{format, ssrc, media_ssrc, lost})
    }
}