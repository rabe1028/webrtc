// https://tools.ietf.org/html/rfc3550
use crate::rtcp::{Result,RtcpError};
use crate::rtcp::report_block::RtcpReportBlock;

//use crate::{Result,Error};
use crate::octets;

const RTCP_HEADER_LENGTH : usize = 8;
const RTCP_SR_INFO_LENGTH : usize = 20;
const RTCP_REPORT_BLOCK_LENGTH : usize = 24;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RtcpReceiverReportPacket{
    ssrc : u32,
    reports : Vec<RtcpReportBlock>,
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
        if bytes.len() != RTCP_HEADER_LENGTH + RTCP_REPORT_BLOCK_LENGTH * count as usize{
            return Err(RtcpError::InvalidPacketHeader);
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