pub struct TDFWriter<T>
where
    T: std::io::Write + std::io::Seek,
{
    writer: T,
}

impl<T> TDFWriter<T> where T: std::io::Write + std::io::Seek {}
