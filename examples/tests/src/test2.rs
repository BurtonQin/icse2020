//use sysinfo::{System, SystemExt};

//use std::ops;

use websocket::sync::Server;

//use rand::{thread_rng, Rng};

//pub fn test2() {
//    System::new();
//}

//pub struct Led{}
//
//pub struct Leds {
//    leds: [Led; 8],
//}
//
//impl ops::Index<usize> for Leds {
//    type Output = Led;
//
//    fn index(&self, i: usize) -> &Led {
//        &self.leds[i]
//    }
//}

//fn test2() {
//    thread_rng();
//}


type WsServer = websocket::server::WsServer<
    websocket::server::NoTlsAcceptor,
    std::net::TcpListener,
>;

fn accept_one(mut server: WsServer) {
    let request = server.accept().ok().unwrap();
}