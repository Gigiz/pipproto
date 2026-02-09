#[repr(u8)]
#[derive(Clone, Copy, Debug)]
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

#[derive(Debug)]
struct Frame {
    version: u8,
    msg_type: MsgType,
    counter: u64,
}

impl Frame {
    fn encode_header(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(b'P');
        out.push(b'P');
        out.push(self.version);
        out.push(self.msg_type as u8);
        out.extend_from_slice(&self.counter.to_be_bytes());
        out
    }

    fn decode_header(input: &[u8]) -> Result<Frame, &'static str> {
        // 2 (magic) + 1 (version) + 1 (type) + 8 (counter) = 12
        if input.len() < 12 {
            return Err("too short");
        }

        // Magic
        if input[0] != b'P' || input[1] != b'P' {
            return Err("bad magic");
        }

        let version = input[2];
        let msg_type = MsgType::from_u8(input[3]).ok_or("unknown msg_type")?;

        // Counter: 8 bytes big-endian
        let counter_bytes: [u8; 8] = input[4..12].try_into().map_err(|_| "bad counter")?;
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
        version: 1,
        msg_type: MsgType::Command,
        counter: 42,
    };

    let bytes = original.encode_header();
    let parsed = Frame::decode_header(&bytes).unwrap();

    println!("parsed = {:?}", parsed);
}
