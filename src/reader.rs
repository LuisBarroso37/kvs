use std::io::{self, BufReader, Read, Seek, SeekFrom};

/// BufReader from std::io with byte's position tracking
#[derive(Debug)]
pub struct BufReaderWithPos<R: Read + Seek> {
  pub reader: BufReader<R>,
  pub pos: u64 // Save current byte's position
}

impl<R: Read + Seek> BufReaderWithPos<R> {
  pub fn new(file: R) -> Self {    
    Self {
      reader: BufReader::new(file),
      pos: 0
    }
  }
}

impl<R: Read + Seek> Read for BufReaderWithPos<R> {
  // Pull some bytes from the file into the specified buffer and set it
  // as the current position
  // Returns how many bytes were read
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    let len = self.reader.read(buf)?;

    self.pos += len as u64;
    Ok(len)
  }
}

impl<R: Read + Seek> Seek for BufReaderWithPos<R> {
  // Find the given position (file offset) in the file and
  // set it as current position
  fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
    self.pos = self.reader.seek(pos)?;

    Ok(self.pos)
  }
}