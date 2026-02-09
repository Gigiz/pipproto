use std::fmt;

const MAGIC: [u8; 2] = *b"PP";
const VERSION_V1: u8 = 0x01;

// Header v1: magic(2) + version(1) + type(1) + flags(1) + device_id(8) + counter(8) = 21
const HEADER_LEN_V1: usize = 21;

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

/// Flags byte (v1)
/// bit0 = ACK_REQUIRED
/// bits1..7 reserved MUST be zero
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Flags(u8);

impl Flags {
    const ACK_REQUIRED: u8 = 0b0000_0001;

    fn new(bits: u8) -> Result<Self, DecodeError> {
        // reserved bits 1..7 must be zero
        if (bits & 0b1111_1110) != 0 {
            return Err(DecodeError::ReservedFlags(bits));
        }
        Ok(Flags(bits))
    }

    fn bits(self) -> u8 {
        self.0
    }

    fn ack_required(self) -> bool {
        (self.0 & Self::ACK_REQUIRED) != 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FrameHeaderV1 {
    version: u8,
    msg_type: MsgType,
    flags: Flags,
    device_id: [u8; 8],
    counter: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FrameV1 {
    header: FrameHeaderV1,
    body: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DecodeError {
    TooShort,
    BadMagic,
    BadVersion(u8),
    UnknownMsgType(u8),
    ReservedFlags(u8),
    BadDeviceIdBytes,
    BadCounterBytes,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::TooShort => write!(f, "input too short"),
            DecodeError::BadMagic => write!(f, "bad magic"),
            DecodeError::BadVersion(v) => write!(f, "unsupported version: 0x{v:02x}"),
            DecodeError::UnknownMsgType(v) => write!(f, "unknown msg_type: 0x{v:02x}"),
            DecodeError::ReservedFlags(b) => write!(f, "reserved flag bits set: 0b{b:08b}"),
            DecodeError::BadDeviceIdBytes => write!(f, "bad device_id bytes"),
            DecodeError::BadCounterBytes => write!(f, "bad counter bytes"),
        }
    }
}

impl FrameHeaderV1 {
    fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(HEADER_LEN_V1);

        out.extend_from_slice(&MAGIC);
        out.push(self.version);
        out.push(self.msg_type as u8);
        out.push(self.flags.bits());
        out.extend_from_slice(&self.device_id);
        out.extend_from_slice(&self.counter.to_be_bytes());

        out
    }

    fn decode(input: &[u8]) -> Result<Self, DecodeError> {
        if input.len() < HEADER_LEN_V1 {
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

        let flags_raw = input[4];
        let flags = Flags::new(flags_raw)?;

        let device_id: [u8; 8] = input[5..13]
            .try_into()
            .map_err(|_| DecodeError::BadDeviceIdBytes)?;

        let counter_bytes: [u8; 8] = input[13..21]
            .try_into()
            .map_err(|_| DecodeError::BadCounterBytes)?;
        let counter = u64::from_be_bytes(counter_bytes);

        Ok(FrameHeaderV1::toggle(
            version, msg_type, flags, device_id, counter,
        ))
    }

    fn toggle(
        version: u8,
        msg_type: MsgType,
        flags: Flags,
        device_id: [u8; 8],
        counter: u64,
    ) -> Self {
        Self {
            version,
            msg_type,
            flags,
            device_id,
            counter,
        }
    }
}

impl FrameV1 {
    fn encode(&self) -> Vec<u8> {
        let mut out = self.header.encode();
        out.extend_from_slice(&self.body);
        out
    }

    fn decode(input: &[u8]) -> Result<Self, DecodeError> {
        let header = FrameHeaderV1::decode(input)?;
        let body = input[HEADER_LEN_V1..].to_vec();
        Ok(FrameV1 { header, body })
    }
}

fn main() {
    let header = FrameHeaderV1 {
        version: VERSION_V1,
        msg_type: MsgType::Event,
        flags: Flags::new(Flags::ACK_REQUIRED).unwrap(),
        device_id: *b"DEV00001",
        counter: 42,
    };

    let frame = FrameV1 {
        header,
        body: b"hello-body".to_vec(),
    };

    let bytes = frame.encode();
    let parsed = FrameV1::decode(&bytes).unwrap();

    println!("{parsed:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_header_v1() {
        let h = FrameHeaderV1 {
            version: VERSION_V1,
            msg_type: MsgType::Command,
            flags: Flags::new(0).unwrap(),
            device_id: *b"ABCDEFGH",
            counter: 123456,
        };

        let bytes = h.encode();
        let parsed = FrameHeaderV1::decode(&bytes).unwrap();
        assert_eq!(parsed, h);
    }

    #[test]
    fn reject_too_short() {
        let bytes = vec![0u8; HEADER_LEN_V1 - 1];
        let err = FrameHeaderV1::decode(&bytes).unwrap_err();
        assert_eq!(err, DecodeError::TooShort);
    }

    #[test]
    fn reject_bad_magic() {
        let mut bytes = vec![0u8; HEADER_LEN_V1];
        bytes[0] = b'X';
        bytes[1] = b'Y';
        bytes[2] = VERSION_V1;
        bytes[3] = MsgType::Event as u8;

        let err = FrameHeaderV1::decode(&bytes).unwrap_err();
        assert_eq!(err, DecodeError::BadMagic);
    }

    #[test]
    fn reject_bad_version() {
        let mut bytes = vec![0u8; HEADER_LEN_V1];
        bytes[0] = b'P';
        bytes[1] = b'P';
        bytes[2] = 0x02;
        bytes[3] = MsgType::Event as u8;

        let err = FrameHeaderV1::decode(&bytes).unwrap_err();
        assert_eq!(err, DecodeError::BadVersion(0x02));
    }

    #[test]
    fn reject_unknown_msg_type() {
        let mut bytes = vec![0u8; HEADER_LEN_V1];
        bytes[0] = b'P';
        bytes[1] = b'P';
        bytes[2] = VERSION_V1;
        bytes[3] = 0x99;

        let err = FrameHeaderV1::decode(&bytes).unwrap_err();
        assert_eq!(err, DecodeError::UnknownMsgType(0x99));
    }

    #[test]
    fn reject_reserved_flags() {
        let mut bytes = vec![0u8; HEADER_LEN_V1];
        bytes[0] = b'P';
        bytes[1] = b'P';
        bytes[2] = VERSION_V1;
        bytes[3] = MsgType::Event as u8;
        bytes[4] = 0b0000_0010; // reserved bit1 set

        let err = FrameHeaderV1::decode(&bytes).unwrap_err();
        assert_eq!(err, DecodeError::ReservedFlags(0b0000_0010));
    }

    #[test]
    fn flags_ack_required() {
        let f = Flags::new(Flags::ACK_REQUIRED).unwrap();
        assert!(f.ack_required());

        let f2 = Flags::new(0).unwrap();
        assert!(!f2.ack_required());
    }

    #[test]
    fn roundtrip_frame_with_body() {
        let header = FrameHeaderV1 {
            version: VERSION_V1,
            msg_type: MsgType::Event,
            flags: Flags::new(0).unwrap(),
            device_id: *b"ABCDEFGH",
            counter: 999,
        };

        let f = FrameV1 {
            header,
            body: vec![1, 2, 3, 4, 5],
        };

        let bytes = f.encode();
        let parsed = FrameV1::decode(&bytes).unwrap();
        assert_eq!(parsed, f);
    }
}
