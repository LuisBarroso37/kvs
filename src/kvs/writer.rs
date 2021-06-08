use std::io::{self, BufWriter, Write, Seek, SeekFrom};

use crate::Result;

/// BufWriter from std::io with byte's position tracking
#[derive(Debug)]
pub struct BufWriterWithPos<W: Write + Seek> {
  pub writer: BufWriter<W>,
  pub pos: u64 // Save current byte's position
}

impl<W: Write + Seek> BufWriterWithPos<W> {
  // Set the position (file offset) to the size of the file (last byte's position)
  pub fn new(mut file: W) -> Result<Self> {  
    let pos =  file.seek(SeekFrom::End(0))?;

    Ok(Self {
      writer: BufWriter::new(file),
      pos
    })
  }
}

impl<W: Write + Seek> Write for BufWriterWithPos<W> {
  // Write the given buffer into the file, returning how many bytes were written
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    let len = self.writer.write(buf)?;

    self.pos += len as u64;
    Ok(len)
  }

  // Flush this output stream, ensuring that all
  // intermediately buffered contents reach their destination
  fn flush(&mut self) -> io::Result<()> {
    self.writer.flush()
  }
}

impl<R: Write + Seek> Seek for BufWriterWithPos<R> {
  // Find the given position (file offset) in the file and
  // set it as current position
  fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
    self.pos = self.writer.seek(pos)?;

    Ok(self.pos)
  }
}