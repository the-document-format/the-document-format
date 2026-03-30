use std::io::{Seek, Write};

pub struct TDFWriter<W: Write + Seek> {
    writer: W,
}

impl<W: Write + Seek> TDFWriter<W> {
    pub fn new(writer: W) -> Self { Self { writer } }
    pub fn write(self) -> std::io::Result<()> { todo!() }
}
