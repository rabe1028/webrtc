use crate::rtcp::{Result};
use crate::octets;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RtcpReportBlock{
    ssrc : u32,                         // 4bytes
    fraction_lost : u8,                 // 1bytes
    packets_lost_accumulation : u32,    // 3bytes
    highest_sequence : u32,             // 4bytes
    jitter : u32,                       // 4bytes
    last_sender_report_timestamp : u32, // 4bytes
    delay : u32,                        // 4bytes
}

impl RtcpReportBlock{
    pub fn get_length() -> u32 {
        4 + 1 + 3 + 4 + 4 + 4 + 4
    }

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
