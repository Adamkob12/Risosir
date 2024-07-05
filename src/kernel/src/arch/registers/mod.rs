pub mod csr;

pub trait ReadFrom {
    type Out;
    unsafe fn read() -> Self::Out;
}

pub trait WriteInto {
    type In;
    unsafe fn write(val: Self::In);
}
