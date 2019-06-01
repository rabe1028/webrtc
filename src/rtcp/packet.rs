use crate::rtcp::{Result,RtcpError};

use crate::rtcp::sender_report::RtcpSenderReportPacket;
use crate::rtcp::receiver_report::RtcpReceiverReportPacket;

use crate::octets;

use num::traits::FromPrimitive;
use crate::rtcp::source_description::RtcpSourceDescriptionPacket;

// https://www.geekpage.jp/technology/rtp/rtcp.php
// http://www.ttc.or.jp/files/6112/8763/7422/2007_3Q_01.pdf

const RTCP_SR    : u8 = 200;
const RTCP_RR    : u8 = 201;
const RTCP_SDES  : u8 = 202;
const RTCP_BYE   : u8 = 203;
const RTCP_APP   : u8 = 204;
const RTCP_RTPFB : u8 = 205;
const RTCP_PSFB  : u8 = 206;

enum RtcpPacketType{
    SenderReport(RtcpSenderReportPacket),
    ReceiverReport(RtcpReceiverReportPacket),
    SourceDescription(RtcpSourceDescriptionPacket),
    Goodbye,
    ApplicationDefined,
    RTPFeedback,
    PayloadSpecificFeedback,
    None
}

struct RtcpPacket{}

impl RtcpPacket{
    pub fn from_bytes(bytes: &mut octets::Octets) -> Result<RtcpPacket>{

        let mut packets : Vec<RtcpPacketType> = Vec::new();

        while bytes.off() < bytes.len(){
            let first = bytes.get_u8()?;
            let version = first >> 6;
            let padding = (first & 0b00100000) > 0;
            let count = first & 0b00011111;
            if version != 2{
                return Err(RtcpError::InvalidPacketVersion);
            }

            let packet_type = bytes.get_u8()?;
            let length = bytes.get_u16()?;

            let packet = match packet_type {
                RTCP_SR  => { RtcpPacketType::SenderReport(      RtcpSenderReportPacket::from_bytes(      bytes, count)? )}
                RTCP_RR  => { RtcpPacketType::ReceiverReport(    RtcpReceiverReportPacket::from_bytes(    bytes, count)? )}
                RTCP_SDES=> { RtcpPacketType::SourceDescription( RtcpSourceDescriptionPacket::from_bytes( bytes, count)? )}
                _ => {RtcpPacketType::None}
            };

            packets.push(packet);
        }

        Ok(RtcpPacket{})
    }
}