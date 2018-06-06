
use std::io::Cursor;
use std::io::Read;
use std::io;
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use bytes::{Buf, BufMut, BytesMut};
use std::collections::HashMap;

pub struct DHCPHeader {
    pub op: u8,
    pub htype: u8,
    pub hlen: u8,
    pub hops: u8,
    pub xid: u32,
    pub secs: u16,
    pub flags: u16,
    pub ciaddr: u32,
    pub yiaddr: u32,
    pub siaddr: u32,
    pub giaddr: u32,
    pub chaddr: u64,
}

#[derive(Message)]
pub struct DHCPPacket {
    pub header: DHCPHeader,
    pub options: HashMap<u8, Vec<u8>>
}

impl DHCPPacket {
    pub fn from_vec(v: Vec<u8>) -> io::Result<Self> {
        let packet: DHCPPacket;
        let mut cursor = Cursor::new(v);

        if cursor.remaining() < 236 { return Err(io::Error::from(io::ErrorKind::UnexpectedEof));}

        let op = cursor.get_u8();
        let htype = cursor.get_u8();
        let hlen = cursor.get_u8();
        let hops = cursor.get_u8();
        let xid = cursor.get_u32_be();
        let secs = cursor.get_u16_be();
        let flags = cursor.get_u16_be();
        let ciaddr = cursor.get_u32_be();
        let yiaddr = cursor.get_u32_be();
        let siaddr = cursor.get_u32_be();
        let giaddr = cursor.get_u32_be();
        let chaddr = cursor.read_uint::<NetworkEndian>(6).unwrap();

        cursor.advance(202);

        let cookie = cursor.get_u32_be();
        if cookie != 0x63_82_53_63 {return Err(io::Error::from(io::ErrorKind::UnexpectedEof));}

        let header = DHCPHeader {
            op: op,
            htype: htype,
            hlen: hlen,
            hops: hops,
            xid: xid,
            secs: secs,
            flags: flags,
            ciaddr: ciaddr,
            yiaddr: yiaddr,
            siaddr: siaddr,
            giaddr: giaddr,
            chaddr: chaddr,
        };

        let mut options = HashMap::new();

        while let Some(code) = cursor.read_u8().ok() {
            match code {
                0 => (),
                255 => break,
                _ => {
                    let len = cursor.read_u8()?;
                    let mut v = vec![0u8; len as usize];
                    cursor.read_exact(&mut v[..])?;
                    options.insert(code, v);
                }
            }
        }

        packet = DHCPPacket {
            header: header,
            options: options,
        };

        Ok(packet)
    }

    pub fn into_vec(self) -> Vec<u8> {
        let mut output: Vec<u8> = Vec::with_capacity(236);
        let mut header = self.header;

        output.put_u8(header.op);
        output.put_u8(header.htype);
        output.put_u8(header.hlen);
        output.put_u8(header.hops);
        output.put_u32_be(header.xid);
        output.put_u16_be(header.secs);
        output.put_u16_be(header.flags);
        output.put_u32_be(header.ciaddr);
        output.put_u32_be(header.yiaddr);
        output.put_u32_be(header.siaddr);
        output.put_u32_be(header.giaddr);
        output.write_uint::<NetworkEndian>(header.chaddr, 6).unwrap();

        output.put_slice(&[0u8; 202]);
        output.put_u32_be(0x63_82_53_63u32);

        let mut options: HashMap<u8, Vec<u8>> = self.options;

        for (code, v) in options {
            output.put_u8(code);
            output.put_u8(v.len() as u8);
            output.put(v);
        }

        output.put_u8(255u8);


        output
    }
}