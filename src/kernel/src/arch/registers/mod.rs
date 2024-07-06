pub mod csr;
pub mod gpr;
pub mod mmapped;

pub trait ReadFrom {
    type Out;
    unsafe fn read(&self) -> Self::Out;
}

pub trait WriteInto {
    type In;
    unsafe fn write(&self, val: Self::In);
}

pub trait AddressOf {
    fn addr_of(&self) -> usize;
}
