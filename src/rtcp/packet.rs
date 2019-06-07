use crate::rtcp::{Result,RtcpError};

// rtcp block format
use crate::rtcp::sender_report::RtcpSenderReportPacket;
use crate::rtcp::receiver_report::RtcpReceiverReportPacket;
use crate::rtcp::source_description::RtcpSourceDescriptionPacket;
use crate::rtcp::good_bye::RtcpGoodByePacket;
use crate::rtcp::rtp_feedback::RtcpRtpFeedbackPacket;
use crate::rtcp::payload_specific_feedback::RtcpPayloadSpecificFeedbackPacket;

use crate::octets;

use num::traits::FromPrimitive;

// https://www.geekpage.jp/technology/rtp/rtcp.php
// http://www.ttc.or.jp/files/6112/8763/7422/2007_3Q_01.pdf

/*

6.1 RTCP Packet Format

   This specification defines several RTCP packet types to carry a
   variety of control information:

   SR:   Sender report, for transmission and reception statistics from
         participants that are active senders

   RR:   Receiver report, for reception statistics from participants
         that are not active senders and in combination with SR for
         active senders reporting on more than 31 sources

   SDES: Source description items, including CNAME

   BYE:  Indicates end of participation

   APP:  Application-specific functions

   Each RTCP packet begins with a fixed part similar to that of RTP data
   packets, followed by structured elements that MAY be of variable
   length according to the packet type but MUST end on a 32-bit
   boundary.  The alignment requirement and a length field in the fixed
   part of each packet are included to make RTCP packets "stackable".
   Multiple RTCP packets can be concatenated without any intervening
   separators to form a compound RTCP packet that is sent in a single
   packet of the lower layer protocol, for example UDP.  There is no
   explicit count of individual RTCP packets in the compound packet
   since the lower layer protocols are expected to provide an overall
   length to determine the end of the compound packet.

   Each individual RTCP packet in the compound packet may be processed
   independently with no requirements upon the order or combination of
   packets.  However, in order to perform the functions of the protocol,
   the following constraints are imposed:

   o  Reception statistics (in SR or RR) should be sent as often as
      bandwidth constraints will allow to maximize the resolution of the
      statistics, therefore each periodically transmitted compound RTCP
      packet MUST include a report packet.

   o  New receivers need to receive the CNAME for a source as soon as
      possible to identify the source and to begin associating media for
      purposes such as lip-sync, so each compound RTCP packet MUST also
      include the SDES CNAME except when the compound RTCP packet is
      split for partial encryption as described in Section 9.1.

   o  The number of packet types that may appear first in the compound
      packet needs to be limited to increase the number of constant bits
      in the first word and the probability of successfully validating
      RTCP packets against misaddressed RTP data packets or other
      unrelated packets.

   Thus, all RTCP packets MUST be sent in a compound packet of at least
   two individual packets, with the following format:

   Encryption prefix:  If and only if the compound packet is to be
      encrypted according to the method in Section 9.1, it MUST be
      prefixed by a random 32-bit quantity redrawn for every compound
      packet transmitted.  If padding is required for the encryption, it
      MUST be added to the last packet of the compound packet.

   SR or RR:  The first RTCP packet in the compound packet MUST
      always be a report packet to facilitate header validation as
      described in Appendix A.2.  This is true even if no data has been
      sent or received, in which case an empty RR MUST be sent, and even
      if the only other RTCP packet in the compound packet is a BYE.

   Additional RRs:  If the number of sources for which reception
      statistics are being reported exceeds 31, the number that will fit
      into one SR or RR packet, then additional RR packets SHOULD follow
      the initial report packet.

   SDES:  An SDES packet containing a CNAME item MUST be included
      in each compound RTCP packet, except as noted in Section 9.1.
      Other source description items MAY optionally be included if
      required by a particular application, subject to bandwidth
      constraints (see Section 6.3.9).

   BYE or APP:  Other RTCP packet types, including those yet to be
      defined, MAY follow in any order, except that BYE SHOULD be the
      last packet sent with a given SSRC/CSRC.  Packet types MAY appear
      more than once.
*/

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

struct RtcpPacket(Vec<RtcpPacketType>);

impl RtcpPacket{
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

            let packet = match packet_type {
                RTCP_SR  => { RtcpPacketType::SenderReport(      RtcpSenderReportPacket::from_bytes(      bytes, count)? )}
                RTCP_RR  => { RtcpPacketType::ReceiverReport(    RtcpReceiverReportPacket::from_bytes(    bytes, count)? )}
                RTCP_SDES=> { RtcpPacketType::SourceDescription( RtcpSourceDescriptionPacket::from_bytes( bytes, count)? )}
                RTCP_BYE => { RtcpPacketType::Goodbye(           RtcpGoodByePacket::from_bytes(           bytes, count)? )}
                _ => {RtcpPacketType::None}
            };

            packet_list.push(packet);
        }

        Ok(RtcpPacket(packet_list) )
    }
}