#[repr(u8)]
#[derive(Clone, Copy, Debug)]
enum MsgType {
    Event = 0x01,
    Command = 0x02,
    Ack = 0x03,
    Error = 0x04,
}

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
}

fn main() {
    let frame = Frame {
        version: 1,
        msg_type: MsgType::Event,
        counter: 1,
    };

    let bytes = frame.encode_header();

    for b in &bytes {
        print!("{:02x} ", b);
    }
    println!();
}
