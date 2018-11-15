use std::path;

pub trait Encoder {
    /// The error type for method results.
    type Error;
    fn emit_u8(&mut self, v: u8) -> Result<(), Self::Error>;
    fn emit_seq<F>(&mut self, len: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>;
    fn emit_seq_elt<F>(&mut self, idx: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>;
}

struct EmptyEncoder{}

impl Encoder for EmptyEncoder {

    type Error = usize;

    fn emit_u8(&mut self, v: u8) -> Result<(), Self::Error> { Err(1) }
    fn emit_seq<F>(&mut self, len: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>{ Err(1) }
    fn emit_seq_elt<F>(&mut self, idx: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>{ Err(1) }
}

pub trait Encodable {
    /// Serialize a value using an `Encoder`.
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error>;
}

impl Encodable for u8 {
    fn encode<S1: Encoder>(&self, s: &mut S1) -> Result<(), S1::Error> {
        s.emit_u8(*self)
    }
}

impl<T:Encodable> Encodable for [T] {
    fn encode<S2: Encoder>(&self, s: &mut S2) -> Result<(), S2::Error> {
        s.emit_seq(self.len(), |s| {
            for (i, e) in self.iter().enumerate() {
                try!(s.emit_seq_elt(i, |s| e.encode(s)))
            }
            Ok(())
        })
    }
}

impl Encodable for path::Path {
    #[cfg(unix)]
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        use std::os::unix::prelude::*;
        self.as_os_str().as_bytes().encode(e)
    }
}

fn test() {
    let mut encoder = EmptyEncoder{};
    let path = path::Path::new("./foo/bar.txt");
    path.encode(&mut encoder);
}
