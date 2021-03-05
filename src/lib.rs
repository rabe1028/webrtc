use failure::Fail;

pub mod octets;
pub mod rtcp;
pub mod rtp;
pub mod sdp;

pub mod rtcpeerconnection;

pub mod rtcrtpparameters;
pub mod rtcdtlstransport;

pub type Result<T> = std::result::Result<T, OctetsError>;
//pub type Result<T> = std::result::Result<T, WebrtcError>;

#[derive(Fail, Debug)]
pub enum WebrtcError {
    #[fail(display = "Octets manipulate failed: {:?}", error)]
    OctetsError { error: OctetsError },
    #[fail(display = "RTP failed: {:?}", error)]
    RtpError { error: rtp::RtpError },
    #[fail(display = "RTCP failed: {:?}", error)]
    RtcpError { error: rtcp::RtcpError },
}

impl From<OctetsError> for WebrtcError {
    fn from(error: OctetsError) -> Self {
        WebrtcError::OctetsError { error }
    }
}

impl From<rtp::RtpError> for WebrtcError {
    fn from(error: rtp::RtpError) -> Self {
        WebrtcError::RtpError { error }
    }
}

impl From<rtcp::RtcpError> for WebrtcError {
    fn from(error: rtcp::RtcpError) -> Self {
        WebrtcError::RtcpError { error }
    }
}

/// A Octets error.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub enum OctetsError {
    /// There is no more work to do.
    Done = -1,

    /// The provided buffer is too short.
    BufferTooShort = -2,
    /*
    /// The provided packet cannot be parsed because its version is unknown.
    UnknownVersion = -3,

    /// InvalidPacketHeader
    InvalidPacketHeader = -4,

    PacketHeaderTooShort = -5,

    InvalidPacketVersion = -6,

    InvalidPacketHeaderExtensionSize = -7,

    InvalidPacketLength = -8,
    */
}
