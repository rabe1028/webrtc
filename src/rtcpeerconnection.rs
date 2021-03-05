use std::net::*;
use webrtc_sdp::address::ExplicitlyTypedAddress;
use webrtc_sdp::*;

//https://developer.mozilla.org/en-US/docs/Web/API/RTCPeerConnection/iceConnectionState

enum IceConnectionState {
    New,
    Checking,
    Connected,
    Completed,
    Failed,
    Disconnected,
    Closed,
}

//https://developer.mozilla.org/en-US/docs/Web/API/RTCPeerConnection/iceGatheringState

enum IceGatheringState {
    New,
    Gathering,
    Completed,
}

enum RTCSessionDescription {
    Offer(String),
    Answer(String),
    Pranswer(String), //http://iwashi.co/2016/04/03/webrtc-pranswer
    Rollback,
}

enum RTCSignalingState {
    Stable,
    HaveLocalOffer,
    HaveRemoteOffer,
    HaveLocalPranswer,
    HaveRemotePranwer,
    Closed,
}

struct IceConnection;

struct RTCPeerConnection {
    ice_gathering_state: IceGatheringState,
    ice_connection: Option<IceConnection>,
}

impl RTCPeerConnection {
    pub fn new() -> RTCPeerConnection {
        RTCPeerConnection {
            ice_connection: None,
            ice_gathering_state: IceGatheringState::New,
        }
    }

    pub fn create_answer(&self) -> RTCSessionDescription {
        let sdp = self.create_sdp();
        RTCSessionDescription::Answer(sdp.to_string())
    }

    pub fn create_offer(&mut self) -> RTCSessionDescription {
        self.ice_gathering_state = IceGatheringState::Gathering;
        //self.gather_candidates();
        self.ice_gathering_state = IceGatheringState::Completed;

        let sdp = self.create_sdp();
        RTCSessionDescription::Offer(sdp.to_string())
    }

    pub fn create_pranswer(&self) -> RTCSessionDescription {
        //http://iwashi.co/2016/04/03/webrtc-pranswer
        let sdp = self.create_sdp();
        RTCSessionDescription::Pranswer(sdp.to_string().replace("a=sendrecv", "a=inactive"))
    }

    //fn gather_candidates(&mut self)

    fn create_sdp(&self) -> SdpSession {
        // TODO: implement
        let sdp_origin = SdpOrigin {
            username: "Test".to_string(),
            session_id: 0,
            session_version: 0,
            unicast_addr: ExplicitlyTypedAddress::Ip("127.0.0.1".parse().unwrap()),
        };
        SdpSession::new(0, sdp_origin, "".to_string())
    }
}
