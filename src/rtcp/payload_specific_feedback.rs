// https://tools.ietf.org/html/rfc3550

/*
Receiver Report Block Format

        0                   1                   2                   3
        0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
header |V=2|P|    RC   |   PT=RR=201   |             length            |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |                     SSRC of packet sender                     |
       +=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+
report |                 SSRC_1 (SSRC of first source)                 |
block  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
  1    | fraction lost |       cumulative number of packets lost       |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |           extended highest sequence number received           |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |                      interarrival jitter                      |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |                         last SR (LSR)                         |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |                   delay since last SR (DLSR)                  |
       +=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+
report |                 SSRC_2 (SSRC of second source)                |
block  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
  2    :                               ...                             :
       +=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+
       |                  profile-specific extensions                  |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

*/

use crate::rtcp::{Result,RtcpError};

//use crate::{Result,Error};
use crate::octets;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RtcpPayloadSpecificFeedbackPacket {
    format : u8,
    ssrc : u32,         // 4bytes
    media_ssrc : u32,   // 4bytes
    fci : Vec<u8>
}

impl RtcpPayloadSpecificFeedbackPacket{
    pub fn get_length(&self) -> u32 {
        4 + 4 + self.fci.len() as u32
    }

    pub fn get_format(&self) -> u8 {
        self.format
    }

    pub fn to_bytes(&self, out: &mut octets::Octets) -> Result<()>{
        out.put_u32(self.ssrc)?;
        out.put_u32(self.media_ssrc)?;
        out.put_bytes(&self.fci)?;

        Ok(())
    }

    pub fn from_bytes(bytes : &mut octets::Octets, format : u8) -> Result<RtcpPayloadSpecificFeedbackPacket>{
        if bytes.len() < 8 {
            return Err(RtcpError::InvalidPacketLength)
        }

        let ssrc = bytes.get_u32()?;
        let media_ssrc = bytes.get_u32()?;

        let fci = bytes.to_vec();

        Ok(RtcpPayloadSpecificFeedbackPacket{format, ssrc, media_ssrc, fci})
    }
}