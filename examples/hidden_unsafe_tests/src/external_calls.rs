use std::io::Cursor;
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt, BigEndian};

pub fn use_trait() {
    let mut buf = [0; 4];
    LittleEndian::write_u32(&mut buf, 1_000_000);
    let y = LittleEndian::read_u32(&buf);
    println!("{}",y);

    let mut rdr = Cursor::new(vec![2, 5, 3, 0]);
    rdr.read_u16::<BigEndian>().unwrap();
    rdr.read_u16::<BigEndian>().unwrap();
}
