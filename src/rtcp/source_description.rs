
use crate::rtcp::{Result,RtcpError};
use crate::octets;

fn get_padding(len : usize) -> usize{
    if len % 4 == 0{
        return 0
    }
    return 4 - (len % 4)
}

struct RtcpSourceDescriptionItem{
    item_type : u8,
    data : Vec<u8>,
}

struct RtcpSourceDescriptionChunk{
    ssrc : u32, // 4bytes
    items : Vec<RtcpSourceDescriptionItem>,
}

impl RtcpSourceDescriptionChunk{
    pub fn get_length(&self) -> u32 {
        let mut b_length = 4;
        b_length += self.items.iter().fold(0, |sum, a| sum + 2 + a.data.len());
        b_length += 1;
        b_length += get_padding(b_length);

        b_length as u32
    }

    pub fn to_bytes(&self, out: &mut octets::Octets) -> Result<()>{
        out.put_u32(self.ssrc)?;
        for item in &self.items{
            out.put_u8(item.item_type)?;
            out.put_u8(item.data.len() as u8)?;
            out.put_bytes(&item.data )?;
        }
        // add END flag
        out.put_u8(0)?;
        // padding
        let padding = get_padding( out.off() );
        match padding{
            0 => {},
            1 => {out.put_u8( 0)?;},
            2 => {out.put_u16(0)?;},
            3 => {out.put_u24(0)?;},
            _ => {return Err(RtcpError::InvalidPaddingSize)} // unreachable
        }

        Ok(())
    }

    pub fn from_bytes(bytes : &mut octets::Octets) -> Result<RtcpSourceDescriptionChunk>{
        let ssrc = bytes.get_u32()?;
        let mut items = Vec::new();
        loop{
            let item_type = bytes.get_u8()?;

            if item_type == 0 {
                // END check.
                let padding = get_padding( bytes.off() );
                if padding > 0{
                    // remove padding
                    bytes.get_bytes(padding)?;
                }
                break;
            }
            let length = bytes.get_u8()?;
            let data = bytes.get_bytes(length as usize)?.to_vec();

            items.push(RtcpSourceDescriptionItem{item_type, data});
        }
        Ok(RtcpSourceDescriptionChunk{ssrc, items})
    }
}

pub struct RtcpSourceDescriptionPacket{
    chunks : Vec<RtcpSourceDescriptionChunk>
}

impl RtcpSourceDescriptionPacket{
    pub fn get_length(&self) -> u32 {
        self.chunks.iter().fold(0, |sum, a| sum + a.get_length())
    }

    pub fn get_chunks_length(&self) -> u8 {
        self.chunks.len() as u8
    }

    pub fn to_bytes(&self, out: &mut octets::Octets) -> Result<()>{
        for chunk in &self.chunks{
            chunk.to_bytes(out)?;
        }

        Ok(())
    }

    pub fn from_bytes(bytes : &mut octets::Octets, count : u8) -> Result<RtcpSourceDescriptionPacket>{
        
        let mut chunks = Vec::new();
        for _ in 0..count{
            let chunk = RtcpSourceDescriptionChunk::from_bytes(bytes)?;
            chunks.push(chunk);
        }
        

        Ok(RtcpSourceDescriptionPacket{chunks})
    }
}