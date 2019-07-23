// https://tools.ietf.org/html/rfc3550
// TODO : Errorの定義
use crate::rtp::{Result,RtpError};
use crate::octets;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct RtpHeaderExtension{
    profile : u16,
    payload : Vec<u8>, // bytes array
}

impl RtpHeaderExtension{
    // 構造体に代入されたデータをBinaryに変換
    pub fn to_bytes(&self, out: &mut octets::Octets) -> Result<()>{
        out.put_u16(self.profile)?;
        out.put_u16(self.payload.len() as u16)?;
        out.put_bytes(self.payload.as_slice())?;

        Ok(())
    }

    pub fn from_bytes(bytes : &mut octets::Octets) -> Result<RtpHeaderExtension>{
        let profile = bytes.get_u16()?;
        let length = bytes.get_u16()?;

        let payload = bytes.get_bytes(length as usize * 4)?.to_vec();

        Ok(RtpHeaderExtension{ profile,payload })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RtpHeader {
    version : u8, // 2bit default value is 2
    //padding : bool, // 1bit
    padding : Option<u8>,
    //extension : bool, // 1bit
    extension : Option<RtpHeaderExtension>,
    //csrc_count : u8, // 4bit count of CSRC
    marker  : bool, // 1bit
    payload_type : u8, // 7bit
    sequence_number : u16,
    timestamp : u32,
    ssrc : u32,
    csrc : Vec<u8>,
}

impl RtpHeader {

    // 構造体に代入されたデータをBinaryに変換
    pub fn to_bytes(&self, out: &mut octets::Octets) -> Result<()>{
        let csrc_count = self.csrc.len() as u8;
        let mut b0 = self.version << 6 | csrc_count;

        if self.padding.is_some() {b0 |= 1 << 5;}
        if self.extension.is_some() {b0 |= 1 << 4;}

        let mut b1 = self.payload_type;
        if self.marker {b1 |= 1 << 7;}

        out.put_u8(b0)?;
        out.put_u8(b1)?;
        out.put_u16(self.sequence_number)?;
        out.put_u32(self.timestamp)?;
        out.put_u32(self.ssrc)?;
        out.put_bytes(self.csrc.as_slice())?;

        // 神ブログ http://www.ameyalokare.com/rust/2017/10/23/rust-options.html
        match self.extension {
            Some(ref v) => {
                v.to_bytes(out)?;
                Ok(())
            }
            None => {
                Ok(())
            }
        }

        /*
        out.put_u16(self.extension_profile)?;
        out.put_u16(self.extension_length)?;
        out.put_bytes(self.extension_payload.as_slice())?;
        */

        // Ok(())
    }

    pub fn from_slice(buf: &mut [u8]) -> Result<RtpHeader> {
        let mut b = octets::Octets::with_slice(buf);
        RtpHeader::from_bytes(&mut b)
    }

    // BinaryをStructに変換
    pub fn from_bytes(bytes : &mut octets::Octets) -> Result<RtpHeader>{
        if bytes.len() < 4 {
            return Err(RtpError::PacketHeaderTooShort)
            //return Err(Error::PacketHeaderTooShort)
        }

        let first = bytes.get_u8()?;

        let version = first >> 6;
        let padding = if (first & 0b00100000) > 0 {
            Some(bytes.get_val( bytes.len() - 1 )? )
        } else { None };

        let extension = (first & 0b00010000) > 0;
        let csrc_count = first & 0b00000111;

        let second = bytes.get_u8()?;

        let marker =  (second & 0b10000000) > 0;
        let payload_type = second & 0b01111111;
        //let sequence_number = ((bytes[2] as u16) << 8) & (bytes[3] as u16);
        let sequence_number = bytes.get_u16()?;
        //let timestamp = ((bytes[4] as u32) << 24) & ((bytes[5] as u32) << 16) & ((bytes[6] as u32) << 8) & (bytes[7] as u32);
        let timestamp = bytes.get_u32()?;

        if version != 2{
            // Value Error : RTP Packet has invalid version. This RTP Packet Version is {:?}.
            return Err(RtpError::InvalidPacketVersion);
            //Err(Error::InvalidPacketVersion);
        }

        //let ssrc = ((bytes[8] as u32) << 24) & ((bytes[9] as u32) << 16) & ((bytes[10] as u32) << 8) & (bytes[11] as u32);
        let ssrc= bytes.get_u32()?;

        let csrc= bytes.get_bytes(csrc_count as usize * 4)?.to_vec();

        if !extension{
            return Ok(RtpHeader{
                version,
                padding,
                extension : None,
                marker,
                payload_type,
                sequence_number,
                timestamp,
                ssrc,
                csrc,
            });
        }

        //extension offset = header bytes length exclude headers extension.
        let extension_offset =( (3 + csrc_count) * 4 ) as usize; // 32bit row buffer * 4

        if bytes.len() < (extension_offset as usize) {
            return Err(RtpError::InvalidPacketHeaderExtensionSize);
            //return Err(Error::InvalidPacketHeaderExtensionSize);
        }

        Ok(RtpHeader{
            version,
            padding,
            extension : Some(RtpHeaderExtension::from_bytes(bytes)?),
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
pub struct RtpPacket{
    header : RtpHeader,
    //padding_length : u8,
    payload : Vec<u8>, // byte array : data
}

impl RtpPacket{
    pub fn to_bytes(&self, out: &mut octets::Octets) -> Result<()>{
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

    pub fn from_bytes(bytes : &mut octets::Octets) -> Result<RtpPacket> {
        let header = RtpHeader::from_bytes(bytes)?;

        let payload = match header.padding {
            Some(v) => {
                if v as usize > bytes.cap() {
                    return Err(RtpError::InvalidPacketPaddingLength);
                }
                bytes.get_bytes(bytes.cap() - v as usize)?.to_vec()
            }
            None => {
                bytes.to_vec()
            }
        };

        Ok(RtpPacket{header, payload})
    }
}

#[cfg(test)]
mod test{
    use crate::octets;
    use super::{*};

    #[test]
    fn rtp_header_parse_test(){
        let mut raw_packet = [0x90, 0xe0, 0x69, 0x8f, 0xd9, 0xc2, 0x93, 0xda, 0x1c, 0x64,
            0x27, 0x82, 0x00, 0x01, 0x00, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0x98, 0x36, 0xbe, 0x88, 0x9e];

        let mut raw_octet = octets::Octets::with_slice(&mut raw_packet);

        let header = RtpHeader::from_bytes(&mut raw_octet).unwrap();

        let parsed_header = RtpHeader{
            version : 2,
            padding : None,
            extension : Option::Some(RtpHeaderExtension{
                profile : 1,
                payload : vec![0xFF, 0xFF, 0xFF, 0xFF],
            }),
            marker : true,
            payload_type : 96,
            sequence_number : 27023,
            timestamp : 3653407706,
            ssrc : 476325762,
            csrc : Vec::new(),
        };
        assert_eq!(header, parsed_header);
    }

    #[test]
    fn rtp_header_extension_test(){
        // TODO : Errorにto_strのメソッド加えたら,customメッセージを出力できるように変える．

        let mut missing = [0x90u8, 0x60, 0x69, 0x8f, 0xd9, 0xc2, 0x93, 0xda, 0x1c, 0x64,
        0x27, 0x82,];

        let mut missing_octets = octets::Octets::with_slice(&mut missing);

        assert!(RtpHeader::from_bytes(&mut missing_octets).is_err());

        let mut invalid_length = [0x90, 0x60, 0x69, 0x8f, 0xd9, 0xc2, 0x93, 0xda, 0x1c, 0x64,
            0x27, 0x82, 0x99, 0x99, 0x99, 0x99,];

        let mut invalid_length_octets = octets::Octets::with_slice(&mut invalid_length);

        assert!(RtpHeader::from_bytes(&mut invalid_length_octets).is_err());
    }

        #[test]
    fn rtp_packet_parse_test(){
        let mut raw_packet = [0x90, 0xe0, 0x69, 0x8f, 0xd9, 0xc2, 0x93, 0xda, 0x1c, 0x64,
            0x27, 0x82, 0x00, 0x01, 0x00, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0x98, 0x36, 0xbe, 0x88, 0x9e];

        let mut raw_octet = octets::Octets::with_slice(&mut raw_packet);

        let packet = RtpPacket::from_bytes(&mut raw_octet).unwrap();

        let parsed_packet = RtpPacket{
            header : RtpHeader{
                version : 2,
                padding : None,
                extension : Option::Some(RtpHeaderExtension{
                    profile : 1,
                    payload : vec![0xFF, 0xFF, 0xFF, 0xFF],
                }),
                marker : true,
                payload_type : 96,
                sequence_number : 27023,
                timestamp : 3653407706,
                ssrc : 476325762,
                csrc : Vec::new(),
            },
            payload : raw_packet[20..].to_vec(), 
        };
        assert_eq!(packet, parsed_packet);
    }

    #[test]
    fn rtp_packet_with_extension_test(){

        let mut missing = [0x90u8, 0x60, 0x69, 0x8f, 0xd9, 0xc2, 0x93, 0xda, 0x1c, 0x64,
        0x27, 0x82,];

        let mut missing_octets = octets::Octets::with_slice(&mut missing);

        assert!(RtpPacket::from_bytes(&mut missing_octets).is_err());

        let mut err = RtpPacket::from_bytes(&mut missing_octets);
        //assert_eq!(err.unwrap() , RtpError::InvalidPacketHeader );

        let mut invalid_length = [0x90, 0x60, 0x69, 0x8f, 0xd9, 0xc2, 0x93, 0xda, 0x1c, 0x64,
            0x27, 0x82, 0x99, 0x99, 0x99, 0x99,];

        let mut invalid_length_octets = octets::Octets::with_slice(&mut invalid_length);

        assert!(RtpPacket::from_bytes(&mut invalid_length_octets).is_err());
    }



    /*
    #[bench]
    unstableだから後回し
    */
}