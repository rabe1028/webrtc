// https://tools.ietf.org/html/rfc4585

use crate::rtcp::{Result,RtcpError};
use crate::rtcp::report_block::RtcpReportBlock;

//use crate::{Result,Error};
use crate::octets;

pub struct RtcpRtpFeedbackPacket {

}

impl RtcpReceiverReportPacket{
    pub fn to_bytes(&self, out: &mut octets::Octets) -> Result<()>{
        out.put_u32(self.ssrc)?;
        for item in &self.reports{
            item.to_bytes(out)?;
        }
        Ok(())
    }

    pub fn from_bytes(bytes : &mut octets::Octets, count : u8) -> Result<RtcpReceiverReportPacket>{
        if bytes.len() < 8 or bytes.len() % 4 != 0 {
            return Err(RtcpError::InvalidPacketLength);
        }
        let ssrc = bytes.get_u32()?;
        let mut reports : Vec<RtcpReportBlock> = Vec::new();
        for i in 0..count{
            match RtcpReportBlock::from_bytes(bytes){
                Ok(v) => {reports.push(v);}
                Err(v) => {return Err(v)}
            }
            //reports.push(RtcpReportBlock::from_bytes(bytes)?);
        }
        Ok(RtcpReceiverReportPacket{ssrc, reports})
    }
}