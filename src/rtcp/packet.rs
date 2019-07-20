use crate::rtcp::{Result,RtcpError};

// rtcp block format
use crate::rtcp::sender_report::RtcpSenderReportPacket;
use crate::rtcp::receiver_report::RtcpReceiverReportPacket;
use crate::rtcp::source_description::RtcpSourceDescriptionPacket;
use crate::rtcp::good_bye::RtcpGoodByePacket;
use crate::rtcp::rtp_feedback::RtcpRtpFeedbackPacket;
use crate::rtcp::payload_specific_feedback::RtcpPayloadSpecificFeedbackPacket;

use crate::octets;

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
    Goodbye(RtcpGoodByePacket),
    ApplicationDefined,
    RTPFeedback(RtcpRtpFeedbackPacket),
    PayloadSpecificFeedback(RtcpPayloadSpecificFeedbackPacket),
    None
}

fn pack_rtcp_packet(packet : &RtcpPacketType, out: &mut octets::Octets) -> Result<()>{
    match packet {
        RtcpPacketType::SenderReport(v) => {
            let len = v.get_length() as u16;
            let count = v.get_reports_count();
            pack_rtcp_header(out, RTCP_SR,   count, len)?;
            v.to_bytes(out)?;
        }
        RtcpPacketType::ReceiverReport(v) => {
            let len = v.get_length() as u16;
            let count = v.get_reports_count();
            pack_rtcp_header(out, RTCP_RR,   count, len)?;
            v.to_bytes(out)?;
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
            pack_rtcp_header(out, RTCP_BYE,  count, len)?;
            v.to_bytes(out)?;
        }
        RtcpPacketType::ApplicationDefined => {
            return Err(RtcpError::NotImplemented)
        }
        RtcpPacketType::RTPFeedback(v) => {
            let len = v.get_length() as u16;
            let fmt = v.get_format();
            pack_rtcp_header(out, RTCP_RTPFB, fmt  , len)?;
            v.to_bytes(out)?;
        }
        RtcpPacketType::PayloadSpecificFeedback(v) => {
            let len = v.get_length() as u16;
            let fmt = v.get_format();
            pack_rtcp_header(out, RTCP_PSFB,  fmt  , len)?;
            v.to_bytes(out)?;
        }
        RtcpPacketType::None => {}
    }
    Ok(())
}

fn pack_rtcp_header(out: &mut octets::Octets, packet_type : u8, count : u8, payload_len : u16) -> Result<()> {
    out.put_u8( (2 << 6) | count )?;
    out.put_u8(packet_type)?;
    out.put_u16(payload_len)?;
    Ok(())
}

struct RtcpPacket(Vec<RtcpPacketType>);

impl RtcpPacket{

    pub fn to_bytes(&self, out: &mut octets::Octets) -> Result<()>{
        for packet in &self.0 {
            pack_rtcp_packet(packet, out)?;
        }
        Ok(())
    }

    pub fn from_bytes(bytes: &mut octets::Octets) -> Result<RtcpPacket>{

        let mut packet_list : Vec<RtcpPacketType> = Vec::new();

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
            
            // rtcp packet bytes length
            let mut tmp_payload = bytes.get_bytes(length as usize * 4)?;

            let last_index : usize = tmp_payload.len() - tmp_payload.get_val(tmp_payload.len() - 1)? as usize;

            let mut payload = if padding {
                tmp_payload.get_bytes(last_index)?
            } else {   
                tmp_payload
            };

            let packet = match packet_type {
                RTCP_SR    => { RtcpPacketType::SenderReport(            RtcpSenderReportPacket::from_bytes(           &mut payload, count)? )}
                RTCP_RR    => { RtcpPacketType::ReceiverReport(          RtcpReceiverReportPacket::from_bytes(         &mut payload, count)? )}
                RTCP_SDES  => { RtcpPacketType::SourceDescription(       RtcpSourceDescriptionPacket::from_bytes(      &mut payload, count)? )}
                RTCP_BYE   => { RtcpPacketType::Goodbye(                 RtcpGoodByePacket::from_bytes(                &mut payload, count)? )}
                RTCP_APP   => { RtcpPacketType::ApplicationDefined}
                RTCP_RTPFB => { RtcpPacketType::RTPFeedback(             RtcpRtpFeedbackPacket::from_bytes(            &mut payload, count)? )}
                RTCP_PSFB  => { RtcpPacketType::PayloadSpecificFeedback( RtcpPayloadSpecificFeedbackPacket::from_bytes(&mut payload, count)? )}
                _ => {RtcpPacketType::None}
            };

            packet_list.push(packet);
        }

        Ok( RtcpPacket(packet_list) )
    }
}