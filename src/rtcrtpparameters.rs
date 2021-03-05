
pub struct RtcRtpCodecCapability {
    mime_type: String,
    // "The codec MIME media type/subtype, for instance `'audio/PCMU'`."
    clock_rate: u64,
    // "The codec clock rate expressed in Hertz."
    channels: Option<usize>,
    // "The number of channels supported (e.g. two for stereo)."
    // parameters: OrderedDict = field(default_factory=OrderedDict)
    // "Codec-specific parameters available for signaling."
}

impl RtcRtpCodecCapability {
    fn name(&self) -> String {
        self.mime_type.split('/').collect::<Vec<&str>>()[1].to_string()
    }
}

pub struct RtcRtpCodecParameters {
    mime_type: String,
    // "The codec MIME media type/subtype, for instance `'audio/PCMU'`."
    clock_rate: u64,
    // "The codec clock rate expressed in Hertz."
    channels: Option<usize>,
    // "The number of channels supported (e.g. two for stereo)."
    payload_type: Option<usize>,
    // "The value that goes in the RTP Payload Type Field."
    rtcp_feedback: Vec<RtcRtcpFeedback>,
    // "Transport layer and codec-specific feedback messages for this codec."
    // parameters: OrderedDict = field(default_factory=OrderedDict)
    // "Codec-specific parameters available for signaling."
}

impl RtcRtpCodecParameters {
    fn name(&self) -> String {
        self.mime_type.split('/').collect::<Vec<&str>>()[1].to_string()
    }

    fn to_string(&self) -> String {
        let s = format!("{}/{}", self.name(), self.clock_rate);
        if self.channels == Some(2) {
            format!("{}/{}", s, 2)
        } else {
            s
        }
    }
}

pub struct RtcRtpRtxParameters {
    ssrc: u32,
}

pub struct RtcRtpCodingParameters{
    pub ssrc: u32,
    pub payload_type: usize,
    pub rtx: Option<RtcRtpRtxParameters>,
}

pub struct RtcRtpDecodingParameters(pub RtcRtpCodingParameters);
pub struct RtcRtpEncodingParameters(pub RtcRtpCodingParameters);

pub struct RtcRtpHeaderExtensionCapability {
    pub uri: String,
    // "The URI of the RTP header extension."
}

pub struct RtcRtpHeaderExtensionParameters {
    pub id: usize,
    // "The value that goes in the packet."
    pub uri: String,
    // "The URI of the RTP header extension."
}

pub struct RtcRtpCapabilities {
    pub codecs: Vec<RtcRtpCodecCapability>,
    pub header_extensions: Vec<RtcRtpHeaderExtensionCapability>,
}

pub struct RtcRtcpFeedback{
    pub kind: String,
    pub param: Option<String>,
}

pub struct RtcRtcpParameters {
    pub cname: Option<String>,
    // "The Canonical Name (CNAME) used by RTCP."
    pub mux: bool,
    // "Whether RTP and RTCP are multiplexed."
    pub ssrc: Option<u32>,
    // "The Synchronization Source identifier."
}

pub struct RtcRtpParameter {
    pub codecs: Vec<RtcRtpCodecParameters>,
    pub header_extensions: Vec<RtcRtpHeaderExtensionParameters>,
    pub mux_id: String,
    // "The muxId assigned to the RTP stream, if any, empty string if unset."
    pub rtcp: RtcRtcpParameters,
    // "Parameters to configure RTCP."
}

pub struct RtcRtpReceiveParameters {
    pub param: RtcRtpParameter,
    pub decoding: Vec<RtcRtpDecodingParameters>,
}

pub struct RtcRtpSendParameters {
    pub param: RtcRtpParameter,
    pub decoding: Vec<RtcRtpEncodingParameters>,
}
