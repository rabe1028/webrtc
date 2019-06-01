use crate::rtcp::{Result,RtcpError};
use crate::octets;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RtcpReportBlock{
    ssrc : u32,
    fraction_lost : u8,
    packets_lost_accumulation : u32,
    highest_sequence : u32,
    jitter : u32,
    last_sender_report_timestamp : u32,
    delay : u32,
}

impl RtcpReportBlock{
    pub fn to_bytes(&self, out: &mut octets::Octets) -> Result<()>{
        out.put_u32(self.ssrc)?;
        out.put_u8(self.fraction_lost)?;
        out.put_u24(self.packets_lost_accumulation)?;
        out.put_u32(self.highest_sequence)?;
        out.put_u32(self.jitter)?;
        out.put_u32(self.last_sender_report_timestamp)?;
        out.put_u32(self.delay)?;
        Ok(())
    }

    pub fn from_bytes(bytes : &mut octets::Octets) -> Result<RtcpReportBlock>{
        let ssrc = bytes.get_u32()?;
        let fraction_lost = bytes.get_u8()?;
        let packets_lost_accumulation = bytes.get_u24()?;
        let highest_sequence = bytes.get_u32()?;
        let jitter = bytes.get_u32()?;
        let last_sender_report_timestamp = bytes.get_u32()?;
        let delay = bytes.get_u32()?;
        Ok(RtcpReportBlock{ssrc, fraction_lost,
            packets_lost_accumulation,
            highest_sequence, jitter, last_sender_report_timestamp, delay})
    }
}
