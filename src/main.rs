use std::fmt;

const MAGIC: [u8; 2] = *b"PP";
const VERSION_V1: u8 = 0x01;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MsgType {
    Event = 0x01,
    Command = 0x02,
    Ack = 0x03,
    Error = 0x04,
}

impl MsgType {
    fn from_u8(v: u8) -> Option<MsgType> {
        match v {
            0x01 => Some(MsgType::Event),
            0x02 => Some(MsgType::Command),
            0x03 => Some(MsgType::Ack),
            0x04 => Some(MsgType::Error),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Frame {
    version: u8,
    msg_type: MsgType,
    counter: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DecodeError {
    TooShort,
    BadMagic,
    BadVersion(u8),
    UnknownMsgType(u8),
    BadCounterBytes,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::TooShort => write!(f, "input too short"),
            DecodeError::BadMagic => write!(f, "bad magic"),
            DecodeError::BadVersion(v) => write!(f, "unsupported version: 0x{v:02x}"),
            DecodeError::UnknownMsgType(v) => write!(f, "unknown msg_type: 0x{v:02x}"),
            DecodeError::BadCounterBytes => write!(f, "bad counter bytes"),
        }
    }
}

impl Frame {
    fn encode_header_v1(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(12);
        out.extend_from_slice(&MAGIC); // "PP"
        out.push(self.version);
        out.push(self.msg_type as u8);
        out.extend_from_slice(&self.counter.to_be_bytes());
        out
    }

    fn decode_header_v1(input: &[u8]) -> Result<Frame, DecodeError> {
        // 2 magic + 1 version + 1 type + 8 counter = 12
        if input.len() < 12 {
            return Err(DecodeError::TooShort);
        }

        if input[0..2] != MAGIC {
            return Err(DecodeError::BadMagic);
        }

        let version = input[2];
        if version != VERSION_V1 {
            return Err(DecodeError::BadVersion(version));
        }

        let msg_raw = input[3];
        let msg_type = MsgType::from_u8(msg_raw).ok_or(DecodeError::UnknownMsgType(msg_raw))?;

        let counter_bytes: [u8; 8] = input[4..12]
            .try_into()
            .map_err(|_| DecodeError::BadCounterBytes)?;
        let counter = u64::from_be_bytes(counter_bytes);

        Ok(Frame {
            version,
            msg_type,
            counter,
        })
    }
}

fn main() {
    let original = Frame {
        version: VERSION_V1,
        msg_type: MsgType::Command,
        counter: 42,
    };

    let bytes = original.encode_header_v1();
    let parsed = Frame::decode_header_v1(&bytes).unwrap();

    println!("parsed = {:?}", parsed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_header() {
        let f = Frame {
            version: VERSION_V1,
            msg_type: MsgType::Event,
            counter: 123456,
        };

        let bytes = f.encode_header_v1();
        let parsed = Frame::decode_header_v1(&bytes).unwrap();
        assert_eq!(parsed, f);
    }

    #[test]
    fn reject_too_short() {
        let bytes = vec![0u8; 11];
        let err = Frame::decode_header_v1(&bytes).unwrap_err();
        assert_eq!(err, DecodeError::TooShort);
    }

    #[test]
    fn reject_bad_magic() {
        let mut bytes = vec![0u8; 12];
        bytes[0] = b'X';
        bytes[1] = b'Y';
        bytes[2] = VERSION_V1;
        bytes[3] = MsgType::Event as u8;

        let err = Frame::decode_header_v1(&bytes).unwrap_err();
        assert_eq!(err, DecodeError::BadMagic);
    }

    #[test]
    fn reject_unknown_msg_type() {
        let mut bytes = vec![0u8; 12];
        bytes[0] = b'P';
        bytes[1] = b'P';
        bytes[2] = VERSION_V1;
        bytes[3] = 0x99;

        let err = Frame::decode_header_v1(&bytes).unwrap_err();
        assert_eq!(err, DecodeError::UnknownMsgType(0x99));
    }

    #[test]
    fn reject_bad_version() {
        let mut bytes = vec![0u8; 12];
        bytes[0] = b'P';
        bytes[1] = b'P';
        bytes[2] = 0x02; // unsupported
        bytes[3] = MsgType::Event as u8;

        let err = Frame::decode_header_v1(&bytes).unwrap_err();
        assert_eq!(err, DecodeError::BadVersion(0x02));
    }
}
