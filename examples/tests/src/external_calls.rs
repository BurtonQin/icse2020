use call_graph;

use call_graph::static_dispatch::Trait2;
use call_graph::static_dispatch::Trait;

//use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt};
//use std::io::Cursor;
//
//use rand::{thread_rng, Rng};
//use rand::RngCore;
//
//pub fn use_trait() {
//    // let mut buf = [0; 4];
//    // LittleEndian::write_u32(&mut buf, 1_000_000);
//    // let y = LittleEndian::read_u32(&buf);
//    // println!("{}", y);
//
//    let mut rdr = Cursor::new(vec![2, 5, 3, 0]);
//    rdr.read_u16::<BigEndian>().unwrap();
//}
//
//pub unsafe fn unsafe_no_reason() {
//    let mut i = 0; i +=1;
//}

//pub fn test_rand() {
//    let mut rng = thread_rng();
//    let x: u32 = rng.gen();
//    println!("{}", x);
//    let y = rng.next_u32();
//}
//pub fn call_extern1() {
//    call_graph::static_dispatch::m();
//}
//
//pub fn call_extern2() {
//    let t = call_graph::static_dispatch::Impl{};
//    t.m3(&t,&t);
//}
//
//pub fn call_extern3() {
//    let a = call_graph::static_dispatch::A{};
//    a.m1();
//    a.m2();
//}