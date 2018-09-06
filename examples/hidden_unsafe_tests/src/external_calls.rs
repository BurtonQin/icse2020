
use byteorder::{ByteOrder, LittleEndian};

pub fn use_trait() {
    let mut buf = [0; 4];
    LittleEndian::write_u32(&mut buf, 1_000_000);
    let y = LittleEndian::read_u32(&buf);
    println!("{}",y);
}
