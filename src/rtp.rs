pub mod packet;
pub mod packetizer;

use crate::OctetsError;
use failure::Fail;

pub type Result<T> = std::result::Result<T, RtpError>;

#[derive(Fail, Debug, PartialEq)]
pub enum RtpError {
    #[fail(display = "Octets manipulate failed: {:?}", error)]
    OctetsError { error: OctetsError },

    /// The provided packet cannot be parsed because its version is unknown.
    #[fail(display = "This RTP packet version is not 2.")]
    UnknownVersion,

    /// InvalidPacketHeader
    #[fail(display = "Invalid packet header")]
    InvalidPacketHeader,

    #[fail(display = "Packet header size is too short.")]
    PacketHeaderTooShort,

    #[fail(display = "This RTP packet version is not 2.")]
    InvalidPacketVersion,

    #[fail(display = "Packet extension is broken.")]
    InvalidPacketHeaderExtensionSize,

    #[fail(display = "RTP packet padding length is invalid.")]
    InvalidPacketPaddingLength,

    #[fail(display = "rtp header extension profile is wrong.")]
    InvalidHeaderExtensionProfile,

    #[fail(display = "rtp header extension value is truncated.")]
    TruncatedHeaderExtensionValue,

    #[fail(display = "rtp two-byte header extension is truncated.")]
    TruncatedTwoByteHeaderExtension,
}

impl From<OctetsError> for RtpError {
    fn from(error: OctetsError) -> Self {
        RtpError::OctetsError { error }
    }
}
