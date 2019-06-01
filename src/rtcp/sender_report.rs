// https://tools.ietf.org/html/rfc3550
use crate::rtcp::{Result,RtcpError};
use crate::rtcp::report_block::RtcpReportBlock;
//use crate::{Result,Error};
use crate::octets;

const RTCP_HEADER_LENGTH : usize = 8;
const RTCP_SR_INFO_LENGTH : usize = 20;
const RTCP_REPORT_BLOCK_LENGTH : usize = 24;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct RtcpSenderInfo{
    ntp_timestamp : u64,
    rtp_timestamp : u32,
    packet_count : u32,
    octet_count : u32,
}

impl RtcpSenderInfo{
    pub fn to_bytes(&self, out: &mut octets::Octets) -> Result<()>{
        out.put_u64(self.ntp_timestamp)?;
        out.put_u32(self.rtp_timestamp)?;
        out.put_u32(self.packet_count)?;
        out.put_u32(self.octet_count)?;
        Ok(())
    }

    pub fn from_bytes(bytes : &mut octets::Octets) -> Result<RtcpSenderInfo>{
        let ntp_timestamp = bytes.get_u64()?;
        let rtp_timestamp = bytes.get_u32()?;
        let packet_count = bytes.get_u32()?;
        let octet_count = bytes.get_u32()?;

        Ok(RtcpSenderInfo{ntp_timestamp,rtp_timestamp,packet_count,octet_count})
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RtcpSenderReportPacket{
    ssrc : u32,
    sender_info : RtcpSenderInfo,
    reports : Vec<RtcpReportBlock>,
}

impl RtcpSenderReportPacket{
    pub fn to_bytes(&self, out: &mut octets::Octets) -> Result<()>{
        out.put_u32(self.ssrc)?;
        self.sender_info.to_bytes(out)?;
        for item in &self.reports{
            item.to_bytes(out)?;
        }
        Ok(())
    }

    pub fn from_bytes(bytes : &mut octets::Octets, count : u8) -> Result<RtcpSenderReportPacket>{
        if bytes.len() != RTCP_HEADER_LENGTH + RTCP_SR_INFO_LENGTH + RTCP_REPORT_BLOCK_LENGTH * count as usize{
            return Err(RtcpError::InvalidPacketHeader);
            //return Err(Error::InvalidPacketLength);
        }
        let ssrc = bytes.get_u32()?;
        let sender_info = RtcpSenderInfo::from_bytes(bytes)?;
        let mut reports : Vec<RtcpReportBlock> = Vec::new();
        for i in 0..count{
            match RtcpReportBlock::from_bytes(bytes){
                Ok(v) => {reports.push(v);}
                Err(v) => {return Err(v)}
            }
        }
        Ok(RtcpSenderReportPacket{ssrc, sender_info, reports})
    }
}