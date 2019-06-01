use failure::Fail;
use crate::OctetsError;

pub mod packet;
pub mod report_block;

pub mod sender_report;
pub mod receiver_report;
pub mod source_description;

pub type Result<T> = std::result::Result<T, RtcpError>;

#[derive(Fail, Debug)]
pub enum RtcpError {
    #[fail(display = "Octets manipulate failed: {:?}", error)]
    OctetsError{
        error : OctetsError,
    },

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

    #[fail(display = "padding size calculation is failed.")]
    InvalidPaddingSize,
}

impl From<OctetsError> for RtcpError{
    fn from(error : OctetsError) -> Self {
        RtcpError::OctetsError { error }
    }
}