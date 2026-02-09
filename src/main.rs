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

struct Frame {
    version: u8,
    msg_type: MsgType,
    counter: u64,
}

fn main() {
    let frame = Frame {
        version: 1,
        msg_type: MsgType::Event,
        counter: 1,
    };

    let t: u8 = frame.msg_type as u8;
    println!("MsgType byte = 0x{:02x}", t);

    // esempio di parse
    let parsed = MsgType::from_u8(0x02).unwrap();
    println!("Parsed = {:?}", parsed);
}
