use crate::OctetsError;
use failure::Fail;

pub mod packet;
pub mod report_block;

pub mod good_bye;
pub mod payload_specific_feedback;
pub mod receiver_report;
pub mod rtp_feedback;
pub mod sender_report;
pub mod source_description;

pub type Result<T> = std::result::Result<T, RtcpError>;

#[derive(Fail, Debug, PartialEq)]
pub enum RtcpError {
    #[fail(display = "Octets manipulate failed: {:?}", error)]
    OctetsError { error: OctetsError },

    /// The provided packet cannot be parsed because its version is unknown.
    #[fail(display = "This RTP packet version is not 2.")]
    UnknownVersion,

    /// The provided packet cannot be parsed because its version is unknown.
    #[fail(display = "Unknown Packet Type.")]
    UnknownPacketType,

    /// InvalidPacketHeader
    #[fail(display = "Invalid packet header")]
    InvalidPacketHeader,

    #[fail(display = "Packet header size is too short.")]
    PacketHeaderTooShort,

    #[fail(display = "Packet extension is broken.")]
    InvalidPacketHeaderExtensionSize,

    #[fail(display = "padding size calculation is failed.")]
    InvalidPaddingSize,

    #[fail(display = "Packet size is invalid.")]
    InvalidPacketLength,

    #[fail(display = "RTCP payload-specific feedback length is invalid")]
    InvalidPsfbPacketLength,

    #[fail(display = "RTCP receiver report length is invalid")]
    InvalidRrPacketLength,

    #[fail(display = "Not implemented.")]
    NotImplemented,
}

impl From<OctetsError> for RtcpError {
    fn from(error: OctetsError) -> Self {
        RtcpError::OctetsError { error }
    }
}
