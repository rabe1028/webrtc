// https://tools.ietf.org/html/rfc3550

/*
BYE: Goodbye RTCP Packet

       0                   1                   2                   3
       0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
      +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
      |V=2|P|    SC   |   PT=BYE=203  |             length            |
      +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
      |                           SSRC/CSRC                           |
      +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
      :                              ...                              :
      +=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+
(opt) |     length    |               reason for leaving            ...
      +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

*/

use crate::rtcp::{Result,RtcpError};

//use crate::{Result,Error};
use crate::octets;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RtcpGoodByePacket(Vec<u32>);

impl RtcpGoodByePacket {
    pub fn to_bytes(&self, out: &mut octets::Octets) -> Result<()>{
        //out.put_bytes(&self.0)?;
        
        for item in &self.0 {
            out.put_u32(item)?;
        }
        Ok(())
    }

    pub fn from_bytes(bytes : &mut octets::Octets, count : u8) -> Result<RtcpGoodByePacket>{
        // ssrc length must be longer than 4 * count, because ssrc is 32bytes unsigned int.
        if bytes.len() < 4 * count as usize{
            return Err(RtcpError::InvalidPacketLength)
        }
        let source = if count > 0 {
            bytes.get_bytes(count as usize)?.to_vec()
        } else {
            Vec::new()
        };

        Ok(RtcpGoodByePacket(source))
    }
}