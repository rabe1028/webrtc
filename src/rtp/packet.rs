// https://tools.ietf.org/html/rfc3550
// TODO : Errorの定義

use crate::octets;
use crate::rtp::{Result, RtpError};
/*
    The RTP header has the following format:

    0                   1                   2                   3
    0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |V=2|P|X|  CC   |M|     PT      |       sequence number         |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                           timestamp                           |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |           synchronization source (SSRC) identifier            |
    +=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+
    |            contributing source (CSRC) identifiers             |
    |                             ....                              |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+


    0                   1                   2                   3
    0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |      defined by profile       |           length              |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                        header extension                       |
    |                             ....                              |
*/

/*
    0                   1                   2                   3
    0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |       0xBE    |    0xDE       |           length=3            |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |  ID   | L=0   |     data      |  ID   |  L=1  |   data...
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
        ...data     |    0 (pad)    |    0 (pad)    |  ID   | L=3   |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                          data                                 |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
*/
fn unpack_header_extension(
    bytes: &mut octets::Octets,
    profile: u16,
) -> Result<(Vec<(u8, Vec<u8>)>)> {
    // OctetsはRTP Header Extension で作り直されたものを使用する予定
    // こうすることで，RTP Header Extensionのサイズを超過しないようにする
    // TODO : Vec<(u8, octets::Octets)>で返せるようにする．．
    if profile != 0xBEDE && profile != 0x1000 {
        return Err(RtpError::InvalidHeaderExtensionProfile);
    }

    let mut extensions = vec![];

    while bytes.cap() > 0 {
        // skip padding byte
        let octet = bytes.get_u8()?;
        if octet == 0x00 {
            continue;
        }

        let (id, length) = match profile {
            0xBEDE => ((octet & 0xf0) >> 4, (octet & 0x0f) + 1),
            0x1000 => {
                let length = bytes
                    .get_u8()
                    .map_err(|_| RtpError::TruncatedTwoByteHeaderExtension)?;
                (octet, length)
            }
            _ => {
                return Err(RtpError::InvalidHeaderExtensionProfile);
            }
        };

        // length check
        if bytes.cap() < length as usize {
            return Err(RtpError::TruncatedHeaderExtensionValue);
        }

        let payload = bytes.get_bytes(length as usize)?.to_vec();

        extensions.push((id, payload));
    }

    Ok(extensions)
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct RtpHeaderExtension {
    profile: u16,
    //payload : Vec<u8>, // bytes array
    payload: Vec<u32>, // 4bytes array
}

impl RtpHeaderExtension {
    // 構造体に代入されたデータをBinaryに変換
    pub fn to_bytes(&self, out: &mut octets::Octets) -> Result<()> {
        out.put_u16(self.profile)?;
        out.put_u16(self.payload.len() as u16)?;
        //out.put_bytes(self.payload.as_slice())?;

        for item in &self.payload {
            out.put_u32(*item)?;
        }

        Ok(())
    }

    pub fn from_bytes(bytes: &mut octets::Octets) -> Result<RtpHeaderExtension> {
        let profile = bytes.get_u16()?;
        let length = bytes.get_u16()?;

        //let payload = bytes.get_bytes(length as usize * 4)?.to_vec();

        //let payload = bytes.get_bytes(length as usize * 4)?;

        let mut payload = vec![];
        for _ in 0..length {
            payload.push(bytes.get_u32()?);
        }

        Ok(RtpHeaderExtension { profile, payload })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RtpHeader {
    version: u8, // 2bit default value is 2
    //padding : bool, // 1bit
    padding: Option<u8>,
    //extension : bool, // 1bit
    extension: Option<RtpHeaderExtension>,
    //csrc_count : u8, // 4bit count of CSRC
    marker: bool,     // 1bit
    payload_type: u8, // 7bit
    sequence_number: u16,
    timestamp: u32,
    ssrc: u32,
    //csrc : Vec<u8>,
    csrc: Vec<u32>,
}

impl RtpHeader {
    // 構造体に代入されたデータをBinaryに変換
    pub fn to_bytes(&self, out: &mut octets::Octets) -> Result<()> {
        let csrc_count = self.csrc.len() as u8;
        let mut b0 = self.version << 6 | csrc_count;

        if self.padding.is_some() {
            b0 |= 1 << 5;
        }
        if self.extension.is_some() {
            b0 |= 1 << 4;
        }

        let mut b1 = self.payload_type;
        if self.marker {
            b1 |= 1 << 7;
        }

        out.put_u8(b0)?;
        out.put_u8(b1)?;
        out.put_u16(self.sequence_number)?;
        out.put_u32(self.timestamp)?;
        out.put_u32(self.ssrc)?;
        //out.put_bytes(self.csrc.as_slice())?;
        for item in &self.csrc {
            out.put_u32(*item)?;
        }

        // 神ブログ http://www.ameyalokare.com/rust/2017/10/23/rust-options.html
        match self.extension {
            Some(ref v) => {
                v.to_bytes(out)?;
                Ok(())
            }
            None => Ok(()),
        }
    }

    pub fn from_slice(buf: &mut [u8]) -> Result<RtpHeader> {
        let mut b = octets::Octets::with_slice(buf);
        RtpHeader::from_bytes(&mut b)
    }

    // BinaryをStructに変換
    pub fn from_bytes(bytes: &mut octets::Octets) -> Result<RtpHeader> {
        if bytes.len() < 4 * 4 {
            return Err(RtpError::PacketHeaderTooShort);
        }

        let first = bytes.get_u8()?;

        let version = first >> 6;
        let padding = if (first & 0b00100000) > 0 {
            Some(bytes.get_val(bytes.len() - 1)?)
        } else {
            None
        };

        let extension = (first & 0b00010000) > 0;
        let csrc_count = first & 0b00000111;

        let second = bytes.get_u8().map_err(|_| RtpError::PacketHeaderTooShort)?;

        let marker = (second & 0b10000000) > 0;
        let payload_type = second & 0b01111111;
        let sequence_number = bytes.get_u16()?;
        let timestamp = bytes.get_u32()?;

        if version != 2 {
            // Value Error : RTP Packet has invalid version. This RTP Packet Version is {:?}.
            return Err(RtpError::InvalidPacketVersion);
        }

        let ssrc = bytes.get_u32()?;

        //let csrc= bytes.get_bytes(csrc_count as usize * 4)?.to_vec();
        let mut csrc = Vec::new();

        if bytes.cap() < 4 * csrc_count as usize {
            return Err(RtpError::PacketHeaderTooShort);
        }
        for _ in 0..csrc_count {
            csrc.push(bytes.get_u32()?);
        }

        if !extension {
            return Ok(RtpHeader {
                version,
                padding,
                extension: None,
                marker,
                payload_type,
                sequence_number,
                timestamp,
                ssrc,
                csrc,
            });
        }

        //extension offset = header bytes length exclude headers extension.
        let extension_offset = ((3 + csrc_count) * 4) as usize; // 32bit row buffer * 4

        if bytes.cap() < (extension_offset) {
            return Err(RtpError::InvalidPacketHeaderExtensionSize);
        }

        Ok(RtpHeader {
            version,
            padding,
            extension: Some(RtpHeaderExtension::from_bytes(bytes)?),
            marker,
            payload_type,
            sequence_number,
            timestamp,
            ssrc,
            csrc,
        })
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct RtpPacket {
    header: RtpHeader,
    //padding_length : u8,
    payload: Vec<u8>, // byte array : data
}

impl RtpPacket {
    pub fn to_bytes(&self, out: &mut octets::Octets) -> Result<()> {
        self.header.to_bytes(out)?;

        for value in &self.payload {
            out.put_u8(*value)?;
        }

        if let Some(padding_length) = self.header.padding {
            for _ in 0..(padding_length as usize - 1) {
                out.put_u8(0)?; // padding
            }
            out.put_u8(padding_length)?;
        }

        Ok(())
    }

    pub fn from_slice(buf: &mut [u8]) -> Result<RtpPacket> {
        let mut b = octets::Octets::with_slice(buf);
        RtpPacket::from_bytes(&mut b)
    }

    pub fn from_bytes(bytes: &mut octets::Octets) -> Result<RtpPacket> {
        let header = RtpHeader::from_bytes(bytes).map_err(|_| RtpError::InvalidPacketHeader)?;

        let payload = match header.padding {
            Some(v) => {
                if v as usize > bytes.cap() {
                    return Err(RtpError::InvalidPacketPaddingLength);
                }
                bytes.get_bytes(bytes.cap() - v as usize)?.to_vec()
            }
            None => bytes.to_vec(),
        };

        Ok(RtpPacket { header, payload })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::octets;

    #[test]
    fn rtp_header_parse_test() {
        let mut raw_packet = [
            0x90, 0xe0, 0x69, 0x8f, 0xd9, 0xc2, 0x93, 0xda, 0x1c, 0x64, 0x27, 0x82, 0x00, 0x01,
            0x00, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0x98, 0x36, 0xbe, 0x88, 0x9e,
        ];

        let mut raw_octet = octets::Octets::with_slice(&mut raw_packet);

        let header = RtpHeader::from_bytes(&mut raw_octet).unwrap();

        let parsed_header = RtpHeader {
            version: 2,
            padding: None,
            extension: Option::Some(RtpHeaderExtension {
                profile: 1,
                payload: vec![0xFFFFFFFF], //vec![0xFF, 0xFF, 0xFF, 0xFF],
            }),
            marker: true,
            payload_type: 96,
            sequence_number: 27023,
            timestamp: 3653407706,
            ssrc: 476325762,
            csrc: vec![],
        };
        //assert_eq!(header, parsed_header);

        let mut pack_buf = [0; 1500];
        let mut offset = 0;
        {
            let mut packed_octets = octets::Octets::with_slice(&mut pack_buf);
            parsed_header.to_bytes(&mut packed_octets);
            offset = packed_octets.off();
        }

        assert_eq!(raw_packet[..offset], pack_buf[..offset]);
    }

    #[test]
    fn rtp_header_extension_test() {
        // TODO : Errorにto_strのメソッド加えたら,customメッセージを出力できるように変える．

        let mut missing = [
            0x90u8, 0x60, 0x69, 0x8f, 0xd9, 0xc2, 0x93, 0xda, 0x1c, 0x64, 0x27, 0x82,
        ];

        let mut missing_octets = octets::Octets::with_slice(&mut missing);

        //assert!(RtpHeader::from_bytes(&mut missing_octets).is_err());
        let header = RtpHeader::from_bytes(&mut missing_octets);
        assert_eq!(header, Err(RtpError::PacketHeaderTooShort));

        let mut invalid_length = [
            0x90, 0x60, 0x69, 0x8f, 0xd9, 0xc2, 0x93, 0xda, 0x1c, 0x64, 0x27, 0x82, 0x99, 0x99,
            0x99, 0x99,
        ];

        let mut invalid_length_octets = octets::Octets::with_slice(&mut invalid_length);

        let header = RtpHeader::from_bytes(&mut invalid_length_octets);
        assert_eq!(header, Err(RtpError::InvalidPacketHeaderExtensionSize));
    }

    #[test]
    fn rtp_packet_parse_test() {
        let mut raw_packet = [
            0x90, 0xe0, 0x69, 0x8f, 0xd9, 0xc2, 0x93, 0xda, 0x1c, 0x64, 0x27, 0x82, 0x00, 0x01,
            0x00, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0x98, 0x36, 0xbe, 0x88, 0x9e,
        ];

        let mut raw_octet = octets::Octets::with_slice(&mut raw_packet);

        let packet = RtpPacket::from_bytes(&mut raw_octet).unwrap();

        let parsed_packet = RtpPacket {
            header: RtpHeader {
                version: 2,
                padding: None,
                extension: Option::Some(RtpHeaderExtension {
                    profile: 1,
                    payload: vec![0xFFFFFFFF], //vec![0xFF, 0xFF, 0xFF, 0xFF],
                }),
                marker: true,
                payload_type: 96,
                sequence_number: 27023,
                timestamp: 3653407706,
                ssrc: 476325762,
                csrc: Vec::new(),
            },
            payload: raw_packet[20..].to_vec(),
        };
        assert_eq!(packet, parsed_packet);

        let mut pack_buf = [0; 1500];
        let mut offset = 0;
        {
            let mut packed_octets = octets::Octets::with_slice(&mut pack_buf);
            parsed_packet.to_bytes(&mut packed_octets);
            offset = packed_octets.off();
        }

        assert_eq!(raw_packet, pack_buf[..offset]);
    }

    #[test]
    fn rtp_packet_with_extension_test() {
        let mut missing = [
            0x90u8, 0x60, 0x69, 0x8f, 0xd9, 0xc2, 0x93, 0xda, 0x1c, 0x64, 0x27, 0x82,
        ];

        let mut missing_octets = octets::Octets::with_slice(&mut missing);

        assert!(RtpPacket::from_bytes(&mut missing_octets).is_err());

        let mut err = RtpPacket::from_bytes(&mut missing_octets);
        assert_eq!(err, Err(RtpError::InvalidPacketHeader));

        let mut invalid_length = [
            0x90, 0x60, 0x69, 0x8f, 0xd9, 0xc2, 0x93, 0xda, 0x1c, 0x64, 0x27, 0x82, 0x99, 0x99,
            0x99, 0x99,
        ];

        let mut invalid_length_octets = octets::Octets::with_slice(&mut invalid_length);

        assert!(RtpPacket::from_bytes(&mut invalid_length_octets).is_err());
    }
}
