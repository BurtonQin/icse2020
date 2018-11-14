
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct Point {
    x: u32,
    y: u32,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Type {
    Safe,
    Unsafe,
}
