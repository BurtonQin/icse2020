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

pub trait Encodable {
    /// Serialize a value using an `Encoder`.
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error>;
}

impl Encodable for u8 {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_u8(*self)
    }
}

impl<T:Encodable> Encodable for [T] {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
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
