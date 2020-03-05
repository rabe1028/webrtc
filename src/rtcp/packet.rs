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
            let len = v.get_length() as u16 >> 2;
            let count = v.get_reports_count();
            pack_rtcp_header(out, RTCP_SR, count, len)?;
            v.to_bytes(out)?;
        }
        RtcpPacketType::ReceiverReport(v) => {
            let len = v.get_length() as u16 >> 2;
            let count = v.get_reports_count();
            pack_rtcp_header(out, RTCP_RR, count, len)?;
            v.to_bytes(out)?;
        }
        RtcpPacketType::SourceDescription(v) => {
            let len = v.get_length() as u16 >> 2;
            let count = v.get_chunks_length();
            pack_rtcp_header(out, RTCP_SDES, count, len)?;
            v.to_bytes(out)?;
        }
        RtcpPacketType::Goodbye(v) => {
            let len = v.get_length() as u16 >> 2;
            let count = v.get_sources_count();
            pack_rtcp_header(out, RTCP_BYE, count, len)?;
            v.to_bytes(out)?;
        }
        RtcpPacketType::ApplicationDefined => return Err(RtcpError::NotImplemented),
        RtcpPacketType::RTPFeedback(v) => {
            let len = v.get_length() as u16 >> 2;
            let fmt = v.get_format();
            pack_rtcp_header(out, RTCP_RTPFB, fmt, len)?;
            v.to_bytes(out)?;
        }
        RtcpPacketType::PayloadSpecificFeedback(v) => {
            let len = v.get_length() as u16 >> 2;
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
        if bytes.len() < 4 { return Err(RtcpError::PacketHeaderTooShort) }
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
            if length == 0 {
                return Err(RtcpError::InvalidPaddingSize);
            }

            let padding_length = tmp_payload.get_val(tmp_payload.len() - 1)? as usize;

            // if padding flag is enable, padding_length must be greater than 0.
            if padding_length == 0 || padding_length > tmp_payload.len() {
                return Err(RtcpError::InvalidPaddingSize);
            }

            let last_index: usize = tmp_payload.len() - padding_length;
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
    use crate::OctetsError;
    use crate::rtcp::good_bye::RtcpGoodByePacket;
    use crate::rtcp::payload_specific_feedback::RtcpPayloadSpecificFeedbackPacket;
    use crate::rtcp::receiver_report::RtcpReceiverReportPacket;
    use crate::rtcp::report_block::RtcpReportBlock;
    use crate::rtcp::rtp_feedback::RtcpRtpFeedbackPacket;
    use crate::rtcp::sender_report::RtcpSenderReportPacket;
    use crate::rtcp::source_description::*;

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

        assert!(parse_bye.is_ok());
        assert_eq!(parse_bye.unwrap(), ref_bye);

        let mut buf = [0u8; 8];
        let mut ser = octets::Octets::with_slice(&mut buf);
        assert!(ref_bye.to_bytes(&mut ser).is_ok());
        assert_eq!(raw_octet, ser);
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

        assert!(parse_bye.is_ok());
        assert_eq!(parse_bye.unwrap(), ref_bye);

        let mut buf = [0u8; 4];
        let mut ser = octets::Octets::with_slice(&mut buf);
        assert!(ref_bye.to_bytes(&mut ser).is_ok());
        assert_eq!(raw_octet, ser);
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

        assert!(parse_bye.is_ok());
        assert_eq!(parse_bye.unwrap(), ref_bye);

        let raw_packet = [0x80, 0xCB, 0x00, 0x00];

        let mut buf = [0u8; 4];
        let mut ser = octets::Octets::with_slice(&mut buf);
        assert!(ref_bye.to_bytes(&mut ser).is_ok());
        assert_eq!(raw_packet, buf);
    }

    #[test]
    fn rtcp_bye_only_padding_zero_test() {
        // from aiortc
        let mut raw_packet = [0xA0, 0xCB, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00];

        let mut raw_octet = octets::Octets::with_slice(&mut raw_packet);

        let parse_bye = RtcpPacket::from_bytes(&mut raw_octet);

        assert_eq!(parse_bye, Err(RtcpError::InvalidPaddingSize));
    }

    #[test]
    fn rtcp_psfb_invalid_test() {
        // from aiortc
        let mut raw_packet = [0x81, 0xCE, 0x00, 0x01, 0xAE, 0x52, 0x8B, 0x43];

        let mut raw_octet = octets::Octets::with_slice(&mut raw_packet);

        let parse_psfb = RtcpPacket::from_bytes(&mut raw_octet);

        assert_eq!(parse_psfb, Err(RtcpError::InvalidPsfbPacketLength));
    }

    #[test]
    fn rtcp_psfb_pli_test() {
        // from aiortc
        let mut raw_packet = [
            0x81, 0xCE, 0x00, 0x02, 0x54, 0x50, 0x62, 0x65, 0x23, 0x01, 0x3F, 0xB9,
        ];

        let mut raw_octet = octets::Octets::with_slice(&mut raw_packet);

        let parse_psfb = RtcpPacket::from_bytes(&mut raw_octet);

        let psfb = RtcpPacket {
            version: 2,
            packet: RtcpPacketType::PayloadSpecificFeedback(
                RtcpPayloadSpecificFeedbackPacket::new(1, 1414554213, 587284409, vec![]),
            ),
        };

        assert!(parse_psfb.is_ok());
        assert_eq!(parse_psfb.unwrap(), psfb);

        let mut buf = [0u8; 12];
        let mut ser = octets::Octets::with_slice(&mut buf);
        assert!(psfb.to_bytes(&mut ser).is_ok());
        assert_eq!(raw_packet, buf);
    }

    #[test]
    fn rtcp_rr_test() {
        // from aiortc
        let mut raw_packet = [
            0x81, 0xC9, 0x00, 0x07, // header
            0x30, 0xB6, 0x84, 0x07, // ssrc
            0x47, 0x94, 0x37, 0xAF, // ssrc 1
            0x00, 0x00, 0x00, 0x00, // fraction lost etc..
            0x00, 0x00, 0x02, 0x76, // highest sequence
            0x00, 0x00, 0x07, 0x72, // jitter
            0x00, 0x00, 0x00, 0x00, // lsr
            0x00, 0x00, 0x00, 0x00, // dlsr
        ];

        let mut raw_octet = octets::Octets::with_slice(&mut raw_packet);

        let parse_rr = RtcpPacket::from_bytes(&mut raw_octet);

        let rr = RtcpPacket {
            version: 2,
            packet: RtcpPacketType::ReceiverReport(RtcpReceiverReportPacket::new(
                817267719,
                vec![RtcpReportBlock::new(
                    1200895919, // ssrc
                    0,          //fraction_lost
                    0,          //packets_lost_accumulation
                    630,        //highest_sequence
                    1906,       //jitter
                    0,          //last_sender_report_timestamp
                    0,          //delay
                )],
            )),
        };

        //assert_eq!(parse_rr, Ok(rr));
        assert!(parse_rr.is_ok());
        assert_eq!(parse_rr.unwrap(), rr);

        let mut buf = [0u8; 4 * 8];
        let mut ser = octets::Octets::with_slice(&mut buf);
        assert!(rr.to_bytes(&mut ser).is_ok());
        assert_eq!(raw_octet, ser);
        assert_eq!(raw_packet, buf);
    }

    #[test]
    fn rtcp_rr_invalid_test() {
        // from aiortc
        let mut raw_packet = [
            0x81, 0xC9, 0x00, 0x01, 
            0xAE, 0x52, 0x8B, 0x43];

        let mut raw_octet = octets::Octets::with_slice(&mut raw_packet);

        let parse_rr = RtcpPacket::from_bytes(&mut raw_octet);

        assert_eq!(parse_rr, Err(RtcpError::InvalidRrPacketLength));
    }

    #[test]
    fn rtcp_rr_truncated_test() {
        // from aiortc
        let mut raw_packet = [
            0x81, 0xC9, 0x00, 0x07, // header
            0x30, 0xB6, 0x84, 0x07, // ssrc
            0x47, 0x94, 0x37, 0xAF, // ssrc 1
            0x00, 0x00, 0x00, 0x00, // fraction lost etc..
            0x00, 0x00, 0x02, 0x76, // highest sequence
            0x00, 0x00, 0x07, 0x72, // jitter
            0x00, 0x00, 0x00, 0x00, // lsr
            0x00, 0x00, 0x00, 0x00, // dlsr
        ];

        for i in 0..4 {
            let mut raw_octet = octets::Octets::with_slice(&mut raw_packet[..i]);
            let parse_rr = RtcpPacket::from_bytes(&mut raw_octet);
            assert_eq!(parse_rr, Err(RtcpError::PacketHeaderTooShort));
        }

        for i in 4..32 {
            let mut raw_octet = octets::Octets::with_slice(&mut raw_packet[..i]);
            let parse_rr = RtcpPacket::from_bytes(&mut raw_octet);
            assert_eq!(parse_rr, Err(RtcpError::OctetsError { error: OctetsError::BufferTooShort}));
        }
    }

    #[test]
    fn rtsp_sdes_test() {
        let mut raw_packet = [
            0x81, 0xCA, 0x00, 0x0C, // header
            0x6D, 0x24, 0x53, 0xEA, // ssrc/csrc
            0x01, 0x26, 0x7B, 0x36, 
            0x33, 0x66, 0x34, 0x35,
            0x39, 0x65, 0x61, 0x2D,
            0x34, 0x31, 0x66, 0x65,
            0x2D, 0x34, 0x34, 0x37, 
            0x34, 0x2D, 0x39, 0x64, 
            0x33, 0x33, 0x2D, 0x39, 
            0x37, 0x30, 0x37, 0x63, 
            0x39, 0x65, 0x65, 0x37,
            0x39, 0x64, 0x31, 0x7D,
            0x00, 0x00, 0x00, 0x00,
        ];

        let mut raw_octet = octets::Octets::with_slice(&mut raw_packet);

        let parse_sdes = RtcpPacket::from_bytes(&mut raw_octet);

        let sdes = RtcpPacket {
            version: 2,
            packet: RtcpPacketType::SourceDescription(
                RtcpSourceDescriptionPacket::new(
                    vec![RtcpSourceDescriptionChunk::new(
                        1831097322, 
                        vec![RtcpSourceDescriptionItem {
                            item_type: 1,
                            data: "{63f459ea-41fe-4474-9d33-9707c9ee79d1}".as_bytes().to_vec(),
                        }]
                    )]
                )
            )
        };

        assert!(parse_sdes.is_ok());
        assert_eq!(parse_sdes.unwrap(), sdes);

        let mut buf = [0u8; 4 * 13];
        let mut ser = octets::Octets::with_slice(&mut buf);
        assert!(sdes.to_bytes(&mut ser).is_ok());
        assert_eq!(raw_octet, ser);
        assert_eq!(raw_packet[..], buf[..]); //長さを消すために，この様に書いている
    }

    #[test]
    fn rtcp_sdes_item_truncated_test() {
        let mut raw_packet = [
            0x81, 0xCA, 0x00, 0x07, 
            0x30, 0xB6, 0x84, 0x07, 
            0x47, 0x94, 0x37, 0xAF, 
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x02, 0x76, 
            0x00, 0x00, 0x07, 0x72, 
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let mut raw_octet = octets::Octets::with_slice(&mut raw_packet);

        let parse_sdes = RtcpPacket::from_bytes(&mut raw_octet);

        assert_eq!(parse_sdes, Err(RtcpError::OctetsError { error: OctetsError::BufferTooShort}));
    }
}
