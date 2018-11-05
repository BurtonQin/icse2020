pub trait RngCore {
    fn next_u32(&mut self);
}
pub trait BlockRngCore {
    type Item;
    fn generate(&mut self);
}
pub struct BlockRng<R: BlockRngCore> {
    pub core: R,
}

impl<R: BlockRngCore> BlockRng<R> {
    pub fn generate_and_set(&mut self) {
        self.core.generate();
    }
}

impl<R: BlockRngCore<Item=u32>> RngCore for BlockRng<R>
{
    fn next_u32(&mut self)  {
        self.generate_and_set();
    }
}