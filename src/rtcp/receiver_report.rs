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

use crate::rtcp::report_block::RtcpReportBlock;
use crate::rtcp::{Result, RtcpError};

//use crate::{Result,Error};
use crate::octets;

const RTCP_HEADER_LENGTH: usize = 4; // ssrc size
const RTCP_SR_INFO_LENGTH: usize = 20;
const RTCP_REPORT_BLOCK_LENGTH: usize = 24;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RtcpReceiverReportPacket {
    ssrc: u32, // 4bytes
    reports: Vec<RtcpReportBlock>,
}

impl RtcpReceiverReportPacket {
    pub fn new(ssrc: u32, reports: Vec<RtcpReportBlock>) -> Self {
        RtcpReceiverReportPacket { ssrc, reports }
    }

    pub fn get_length(&self) -> u32 {
        4 + self.reports.len() as u32 * RtcpReportBlock::get_length()
    }

    pub fn get_reports_count(&self) -> u8 {
        self.reports.len() as u8
    }

    pub fn to_bytes(&self, out: &mut octets::Octets) -> Result<()> {
        out.put_u32(self.ssrc)?;

        self.reports
            .iter()
            .map(|item| item.to_bytes(out))
            .collect::<Result<Vec<_>>>()
            .map(|_| {})
    }

    pub fn from_bytes(bytes: &mut octets::Octets, count: u8) -> Result<RtcpReceiverReportPacket> {
        if bytes.len() != RTCP_HEADER_LENGTH + RTCP_REPORT_BLOCK_LENGTH * count as usize {
            return Err(RtcpError::InvalidRrPacketLength);
        }
        let ssrc = bytes.get_u32()?;

        let reports = (0..count)
            .map(|_| RtcpReportBlock::from_bytes(bytes))
            .collect::<Result<Vec<_>>>()?;

        Ok(RtcpReceiverReportPacket { ssrc, reports })
    }
}
