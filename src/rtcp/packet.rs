use crate::rtcp::{Result, RtcpError};

// rtcp block format
use crate::rtcp::good_bye::RtcpGoodByePacket;
use crate::rtcp::payload_specific_feedback::RtcpPayloadSpecificFeedbackPacket;
use crate::rtcp::receiver_report::RtcpReceiverReportPacket;
use crate::rtcp::rtp_feedback::RtcpRtpFeedbackPacket;
use crate::rtcp::sender_report::RtcpSenderReportPacket;
use crate::rtcp::source_description::RtcpSourceDescriptionPacket;

use crate::octets;

// https://www.geekpage.jp/technology/rtp/rtcp.php
// http://www.ttc.or.jp/files/6112/8763/7422/2007_3Q_01.pdf

const RTCP_SR: u8 = 200;
const RTCP_RR: u8 = 201;
const RTCP_SDES: u8 = 202;
const RTCP_BYE: u8 = 203;
const RTCP_APP: u8 = 204;
const RTCP_RTPFB: u8 = 205;
const RTCP_PSFB: u8 = 206;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum RtcpPacketType {
    SenderReport(RtcpSenderReportPacket),
    ReceiverReport(RtcpReceiverReportPacket),
    SourceDescription(RtcpSourceDescriptionPacket),
    Goodbye(RtcpGoodByePacket),
    ApplicationDefined,
    RTPFeedback(RtcpRtpFeedbackPacket),
    PayloadSpecificFeedback(RtcpPayloadSpecificFeedbackPacket),
}

fn pack_rtcp_packet(packet: &RtcpPacketType, out: &mut octets::Octets) -> Result<()> {
    match packet {
        RtcpPacketType::SenderReport(v) => {
            let len = v.get_length() as u16;
            let count = v.get_reports_count();
            pack_rtcp_header(out, RTCP_SR, count, len)?;
            v.to_bytes(out)?;
        }
        RtcpPacketType::ReceiverReport(v) => {
            let len = v.get_length() as u16;
            let count = v.get_reports_count();
            pack_rtcp_header(out, RTCP_RR, count, len)?;
        }
        RtcpPacketType::SourceDescription(v) => {
            let len = v.get_length() as u16;
            let count = v.get_chunks_length();
            pack_rtcp_header(out, RTCP_SDES, count, len)?;
            v.to_bytes(out)?;
        }
        RtcpPacketType::Goodbye(v) => {
            let len = v.get_length() as u16;
            let count = v.get_sources_count();
            pack_rtcp_header(out, RTCP_BYE, count, len)?;
            v.to_bytes(out)?;
        }
        RtcpPacketType::ApplicationDefined => return Err(RtcpError::NotImplemented),
        RtcpPacketType::RTPFeedback(v) => {
            let len = v.get_length() as u16;
            let fmt = v.get_format();
            pack_rtcp_header(out, RTCP_RTPFB, fmt, len)?;
            v.to_bytes(out)?;
        }
        RtcpPacketType::PayloadSpecificFeedback(v) => {
            let len = v.get_length() as u16;
            let fmt = v.get_format();
            pack_rtcp_header(out, RTCP_PSFB, fmt, len)?;
            v.to_bytes(out)?;
        }
    }

    Ok(())
}

fn pack_rtcp_header(
    out: &mut octets::Octets,
    packet_type: u8,
    count: u8,
    payload_len: u16,
) -> Result<()> {
    out.put_u8((2 << 6) | count)?;
    out.put_u8(packet_type)?;
    out.put_u16(payload_len)?;
    Ok(())
}

//struct RtcpPacket(Vec<RtcpPacketType>);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RtcpPacket {
    version: u8,
    packet: RtcpPacketType,
}

impl RtcpPacket {
    pub fn to_bytes(&self, out: &mut octets::Octets) -> Result<()> {
        pack_rtcp_packet(&self.packet, out)?;
        Ok(())
    }

    pub fn from_bytes(bytes: &mut octets::Octets) -> Result<RtcpPacket> {
        let first = bytes.get_u8()?;

        let version = first >> 6;
        let padding = (first & 0b00100000) > 0;
        let count = first & 0b00011111;

        if version != 2 {
            return Err(RtcpError::UnknownVersion);
        }

        let packet_type = bytes.get_u8()?;
        let length = bytes.get_u16()?;

        // rtcp packet bytes length
        let mut tmp_payload = bytes.get_bytes(length as usize * 4)?;

        let mut payload = if padding {
            let last_index: usize =
                tmp_payload.len() - tmp_payload.get_val(tmp_payload.len() - 1)? as usize;
            tmp_payload.get_bytes(last_index)?
        } else {
            tmp_payload
        };

        let packet = match packet_type {
            RTCP_SR => RtcpPacketType::SenderReport(RtcpSenderReportPacket::from_bytes(
                &mut payload,
                count,
            )?),
            RTCP_RR => RtcpPacketType::ReceiverReport(RtcpReceiverReportPacket::from_bytes(
                &mut payload,
                count,
            )?),
            RTCP_SDES => RtcpPacketType::SourceDescription(
                RtcpSourceDescriptionPacket::from_bytes(&mut payload, count)?,
            ),
            RTCP_BYE => {
                RtcpPacketType::Goodbye(RtcpGoodByePacket::from_bytes(&mut payload, count)?)
            }
            RTCP_APP => RtcpPacketType::ApplicationDefined,
            RTCP_RTPFB => {
                RtcpPacketType::RTPFeedback(RtcpRtpFeedbackPacket::from_bytes(&mut payload, count)?)
            }
            RTCP_PSFB => RtcpPacketType::PayloadSpecificFeedback(
                RtcpPayloadSpecificFeedbackPacket::from_bytes(&mut payload, count)?,
            ),
            _ => return Err(RtcpError::UnknownPacketType),
        };

        Ok(RtcpPacket {
            version,
            packet: packet,
        })
    }
}

type RtcpPacketList = Vec<RtcpPacket>;

pub fn parse(bytes: &mut octets::Octets) -> Result<RtcpPacketList> {
    let mut packet_list = Vec::new();

    while bytes.off() < bytes.len() {
        packet_list.push(RtcpPacket::from_bytes(bytes)?);
    }

    Ok(packet_list)
}

pub fn serialize(packets: RtcpPacketList, out: &mut octets::Octets) -> Result<()> {
    for packet in &packets {
        packet.to_bytes(out)?;
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::octets;
    use crate::rtcp::good_bye::RtcpGoodByePacket;
    /*
    use crate::rtcp::payload_specific_feedback::RtcpPayloadSpecificFeedbackPacket;
    use crate::rtcp::receiver_report::RtcpReceiverReportPacket;
    use crate::rtcp::rtp_feedback::RtcpRtpFeedbackPacket;
    use crate::rtcp::sender_report::RtcpSenderReportPacket;
    use crate::rtcp::source_description::RtcpSourceDescriptionPacket;
    */

    #[test]
    fn rtcp_bye_parse_test() {
        // from aiortc
        let mut raw_packet = [0x81, 0xCB, 0x00, 0x01, 0xAE, 0x52, 0x8B, 0x43];

        let mut raw_octet = octets::Octets::with_slice(&mut raw_packet);

        let parse_bye = RtcpPacket::from_bytes(&mut raw_octet);

        let bye_buf = RtcpGoodByePacket(vec![2924645187]);

        let ref_bye = RtcpPacket {
            version: 2,
            packet: RtcpPacketType::Goodbye(bye_buf),
        };

        assert_eq!(parse_bye, Ok(ref_bye));
    }

    #[test]
    fn rtcp_bye_invalid_test() {
        // from aiortc
        let mut raw_packet = [0x91, 0xCB, 0x00, 0x01, 0xAE, 0x52, 0x8B, 0x43];

        let mut raw_octet = octets::Octets::with_slice(&mut raw_packet);

        let parse_bye = RtcpPacket::from_bytes(&mut raw_octet);

        assert_eq!(parse_bye, Err(RtcpError::InvalidPacketLength));
    }

    #[test]
    fn rtcp_bye_no_sources_test() {
        // from aiortc
        let mut raw_packet = [0x80, 0xCB, 0x00, 0x00];

        let mut raw_octet = octets::Octets::with_slice(&mut raw_packet);

        let parse_bye = RtcpPacket::from_bytes(&mut raw_octet);

        let bye_buf = RtcpGoodByePacket(vec![]);

        let ref_bye = RtcpPacket {
            version: 2,
            packet: RtcpPacketType::Goodbye(bye_buf),
        };

        assert_eq!(parse_bye, Ok(ref_bye));
    }

    #[test]
    fn rtcp_bye_only_padding_test() {
        // from aiortc
        let mut raw_packet = [0xA0, 0xCB, 0x00, 0x01, 0x04, 0x04, 0x04, 0x04];

        let mut raw_octet = octets::Octets::with_slice(&mut raw_packet);

        let parse_bye = RtcpPacket::from_bytes(&mut raw_octet);

        let bye_buf = RtcpGoodByePacket(vec![]);

        let ref_bye = RtcpPacket {
            version: 2,
            packet: RtcpPacketType::Goodbye(bye_buf),
        };

        assert_eq!(parse_bye, Ok(ref_bye));
    }
}
