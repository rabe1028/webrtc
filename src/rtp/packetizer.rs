use crate::octets;
use crate::rtp::packet::RtpHeader;
use rand::Rng;

// Payloadの詰め込みと新規StreamのSSRC発行などを行う．
struct RtpPacketizer {
    mtu: usize,
    payload_type: u8,
    ssrc: u32,
    timestamp: u32,
    clock_rate: u32,
}

impl RtpPacketizer {
    // NewPacketizer returns a new instance of a Packetizer for a specific payloader
    pub fn new(mtu: usize, payload_type: u8, ssrc: u32, clock_rate: u32) -> RtpPacketizer {
        let mut rng = rand::thread_rng();
        RtpPacketizer {
            mtu,
            payload_type,
            ssrc,
            timestamp: rng.gen(),
            clock_rate,
        }
    }

    pub fn pack() -> () {
        ()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rand() {
        let mut rng = rand::thread_rng();
        let rand1: u32 = rng.gen();
        let rand2: u32 = rng.gen();

        assert_ne!(rand1, rand2)
    }
}
