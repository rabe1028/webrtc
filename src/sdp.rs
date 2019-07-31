use failure::Fail;

use std::net::IpAddr;

pub type Result<T> = std::result::Result<T, SdpError>;

#[derive(Fail, Debug, PartialEq)]
pub enum SdpError {
    #[fail(display = "Not Implemented")]
    NotImplemented,

    #[fail(display = "SDP is empty.")]
    EmptySDP,

    #[fail(display = "sdp version parse failed.")]
    VersionParseFailed,

    #[fail(display = "sdp session split failed.")]
    SessionParseFailed,

    #[fail(display = "sdp session split failed.")]
    AttributeParseFailed,

    #[fail(display = "session name parse failed.")]
    SessionNameParseFailed,

    #[fail(display = "session name parse failed.")]
    OriginParseFailed,

    #[fail(display = "Line format is wrong.")]
    InvalidLineFormat,

    #[fail(display = "sdp origin ip is invalid.")]
    InvalidIpType,

    #[fail(display = "ip address parsing failed.")]
    IpAddrParseFailed,
}

pub fn make_groups(sdp : &str) -> (Vec<&str>, Vec<Vec<&str>>) {
    let mut lines = sdp.lines();

    let mut session_lines : Vec<&str> = vec![];
    let mut media_lines : Vec<Vec<&str>> = vec![];

    let mut media_count = 0;

    while let Some(item) = lines.next() {
        println!("{:?}",item);
        match &item[..2] {
            "m=" => {
                media_lines.push(vec![item]);
                media_count += 1;
            }
            _ => {
                if media_count > 0 {
                    media_lines[media_count - 1].push(item);
                } else {
                    session_lines.push(item);
                }
            }
        }
    }

    (session_lines, media_lines)
}

pub fn parse_version(p: &str) -> Result<u64> {
    let split: Vec<&str> = p.splitn(2, '=').collect();
    if split.len() != 2 {
        return Err(SdpError::InvalidLineFormat);
    }

    if split[0] != "v" {
        return Err(SdpError::VersionParseFailed);
    }

    Ok(split[1].parse::<u64>().unwrap())
}

pub fn parse_session_name(p: &str) -> Result<String> {
    let split: Vec<&str> = p.splitn(2, '=').collect();
    if split.len() != 2 {
        return Err(SdpError::InvalidLineFormat);
    }
    if split[0] != "s" {
        return Err(SdpError::SessionNameParseFailed);
    }

    Ok(split[1].to_string())
}

pub fn parse_attribute(p: &str) -> Result<SessionAttribute> {
    let split: Vec<&str> = p.splitn(2, '=').collect();
    if split.len() != 2 {
        return Err(SdpError::InvalidLineFormat);
    }
    if split[0] != "a" {
        return Err(SdpError::AttributeParseFailed);
    }

    let data: Vec<&str> = split[1].splitn(2, ':').collect();

    Ok(SessionAttribute {
        attribute_type: data[0].to_string(),
        data: data[1].to_string(),
    })
    /*
    match data[1] {
        "fingerprint" => None,
    }
    */
}

// TODO : enum に変更
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FingerPrint {
    pub algorithm: String,
    pub value: String,
}

// TODO : RtcIceTypeに変更
pub struct IceOptions {
    pub option_type: String,
}

/*
pub enum SessionAttribute {
    FingerPrint(FingerPrint),
    IceOptions(IceOptions),

}
*/

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SessionAttribute {
    attribute_type: String,
    data: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct OriginInfo {
    username: String,
    session_id: u64,
    session_version: u64,
    address: IpAddr,
}

impl OriginInfo {
    pub fn parse(sdp: &str) -> Result<Self> {
        let split: Vec<&str> = sdp.splitn(2, '=').collect();
        if split[0] != "o" {
            return Err(SdpError::OriginParseFailed);
        }

        let data: Vec<&str> = split[1].split_whitespace().collect();

        if data.len() != 6 {
            return Err(SdpError::InvalidLineFormat);
        }

        let username = data[0].to_string();
        let session_id = data[1].parse::<u64>().unwrap();
        let session_version = data[2].parse::<u64>().unwrap();
        let address = match data[3] {
            // nettype matching
            "IN" => {
                // address parse
                let addr: IpAddr = data[5].parse().map_err(|_| SdpError::IpAddrParseFailed)?;
                match data[4] {
                    // ip type matching
                    "IP4" => {
                        assert!(addr.is_ipv4());
                    }
                    "IP6" => {
                        assert!(addr.is_ipv6());
                    }
                    _ => {
                        println!("invalid ip type {:?}", data[4]);
                        return Err(SdpError::InvalidIpType);
                    }
                }
                addr
            }
            _ => {
                println!("invalid ip type {:?}", data[3]);
                return Err(SdpError::InvalidIpType);
            }
        };

        Ok(OriginInfo {
            username,
            session_id,
            session_version,
            address,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TimeDescription {
    start_time: u64,
    stop_time: u64,
}

impl TimeDescription {
    pub fn parse(sdp_payload: &str) -> Result<Self> {
        let split: Vec<&str> = sdp_payload.splitn(2, '=').collect();
        if split[0] != "t" {
            return Err(SdpError::InvalidLineFormat);
        }

        let mut s = split[1].split_whitespace();

        let start_time = match s.next() {
            Some(v) => {
                println!("start_time : {:?}", v);
                v.parse::<u64>().unwrap()
            }
            None => {
                return Err(SdpError::InvalidLineFormat);
            }
        };

        let stop_time = match s.next() {
            Some(v) => {
                println!("stop_time : {:?}", v);
                v.parse::<u64>().unwrap()
            }
            None => {
                return Err(SdpError::InvalidLineFormat);
            }
        };

        Ok(TimeDescription {
            start_time,
            stop_time,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SessionDescription {
    version: u64,
    origin: OriginInfo,
    session_name: String,
    time: TimeDescription,
    session_info: Option<String>,
    description_url: Option<String>,
    email_address: Option<String>,
    phone_number: Option<String>,
    attribute: Vec<SessionAttribute>,
}

impl SessionDescription {
    pub fn new(
        version: u64,
        origin: OriginInfo,
        session_name: String,
        time: TimeDescription,
    ) -> Self {
        SessionDescription {
            version,
            origin,
            session_name,
            time,
            session_info: None,
            description_url: None,
            email_address: None,
            phone_number: None,
            attribute: vec![],
        }
    }

    pub fn parse(split_sdp: &Vec<&str>) -> Result<Self> {
        if split_sdp.is_empty() {
            return Err(SdpError::EmptySDP);
        }

        // parse session
        //println!("{:?}", split_sdp);

        let version = parse_version(split_sdp[0])?;

        let origin = OriginInfo::parse(split_sdp[1])?;

        let session_name = parse_session_name(split_sdp[2])?;

        let time = TimeDescription::parse(split_sdp[3])?;

        let mut session = SessionDescription::new(version, origin, session_name, time);

        println!("{:?}", session);

        for item in &split_sdp[4..] {
            //println!("item :{:?}", item);
            session.attribute.push(parse_attribute(item)?);
        }

        Ok(session)
    }

    pub fn add_media(self, media_sdp : &Vec<Vec<&str>>) -> Result<Self> {
        Ok(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use webrtc_sdp;
    use webrtc_sdp::error::*;

    #[test]
    fn audio_chrome_test() {
        let d = "v=0
o=- 863426017819471768 2 IN IP4 127.0.0.1
s=-
t=0 0
a=group:BUNDLE audio
a=msid-semantic: WMS TF6VRif1dxuAfe5uefrV2953LhUZt1keYvxU
m=audio 45076 UDP/TLS/RTP/SAVPF 111 103 104 9 0 8 106 105 13 110 112 113 126
c=IN IP4 192.168.99.58
a=rtcp:9 IN IP4 0.0.0.0
a=candidate:2665802302 1 udp 2122262783 2a02:a03f:3eb0:e000:b0aa:d60a:cff2:933c 38475 typ host generation 0 network-id 2 network-cost 10
a=candidate:1039001212 1 udp 2122194687 192.168.99.58 45076 typ host generation 0 network-id 1 network-cost 10
a=candidate:3496416974 1 tcp 1518283007 2a02:a03f:3eb0:e000:b0aa:d60a:cff2:933c 9 typ host tcptype active generation 0 network-id 2 network-cost 10
a=candidate:1936595596 1 tcp 1518214911 192.168.99.58 9 typ host tcptype active generation 0 network-id 1 network-cost 10
a=ice-ufrag:5+Ix
a=ice-pwd:uK8IlylxzDMUhrkVzdmj0M+v
a=ice-options:trickle
a=fingerprint:sha-256 6B:8B:5D:EA:59:04:20:23:29:C8:87:1C:CC:87:32:BE:DD:8C:66:A5:8E:50:55:EA:8C:D3:B6:5C:09:5E:D6:BC
a=setup:actpass
a=mid:audio
a=extmap:1 urn:ietf:params:rtp-hdrext:ssrc-audio-level
a=sendrecv
a=rtcp-mux
a=rtpmap:111 opus/48000/2
a=rtcp-fb:111 transport-cc
a=fmtp:111 minptime=10;useinbandfec=1
a=rtpmap:103 ISAC/16000
a=rtpmap:104 ISAC/32000
a=rtpmap:9 G722/8000
a=rtpmap:0 PCMU/8000
a=rtpmap:8 PCMA/8000
a=rtpmap:106 CN/32000
a=rtpmap:105 CN/16000
a=rtpmap:13 CN/8000
a=rtpmap:110 telephone-event/48000
a=rtpmap:112 telephone-event/32000
a=rtpmap:113 telephone-event/16000
a=rtpmap:126 telephone-event/8000
a=ssrc:1944796561 cname:/vC4ULAr8vHNjXmq
a=ssrc:1944796561 msid:TF6VRif1dxuAfe5uefrV2953LhUZt1keYvxU ec1eb8de-8df8-4956-ae81-879e5d062d12
a=ssrc:1944796561 mslabel:TF6VRif1dxuAfe5uefrV2953LhUZt1keYvxU
a=ssrc:1944796561 label:ec1eb8de-8df8-4956-ae81-879e5d062d12";

        //let sdp = webrtc_sdp::parse_sdp(d, true);

        //let split_sdp: Vec<Vec<&str>> = d.split("m=").map(|s| s.lines().collect()).collect();

        let (session, media) = make_groups(d);
        
        let out = SessionDescription::parse(&session).unwrap().add_media(&media);

        println!("{:?}", out);

        //println!("{}", sdp.unwrap() );
        //assert_eq!(sdp);
        //assert!(sdp.is_some());
    }
}
