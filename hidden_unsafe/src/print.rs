pub trait Print{
    fn print(&self) -> ();
}

pub struct EmptyPrinter{}

impl Print for EmptyPrinter {
    fn print(&self) -> (){}
}
