use std::fmt::Display;
use crate::result::Result;

/*
  Represents a KV storage engine, where both keys and values are arbitrary byte strings between
  size 0 B - 2 GB.

  The key-value pairs are ordered lexicographically based on the keys.
  NOTE : Lexicographical ordering is essentially alphabetical ordering, but for byte strings. The
  keys will be arranged in ascending order based on their byte values.

  Writes are only guaranteed durable, after they're flushed (using flush( )).
*/
pub trait StorageEngine
  : Display + Send + Sync
{
  // Stores the key-value pair.
  // NOTE : If the key already exists, then the value is overwritten.
  fn set(&mut self, key: &[u8], value: Vec<u8>) -> Result<( )>;

  // Flushes any buffered (in-memory) data to the underlying storage medium.
  fn flush(&mut self) -> Result<( )>;

  // Returns the value stored against the given key (if it exists).
  fn get(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>>;

  // Deletes a key.
  // NOTE : Does nothing if the key doesn't exist.
  fn delete(&mut self, key: &[u8]) -> Result<( )>;

  // Returns the status of the storage engine.
  fn status(&self) -> Result<StorageEngineStatus>;
}

pub struct StorageEngineStatus {
  pub name: String,

  pub keyCount: u64,

  /*
    Logical size reflects the size of the data as it is. For example, if you have a text file
    containing 1000 characters, the logical size of the file would be the sum of the bytes required
    to store those characters.

    On-disk size = Logical size + metadata size + FS overhead + (- optimization due to compression) + etc...
  */

  // Logical size of live (usefull) key-value pairs.
  pub logicalSize: u64,

  pub diskSize: u64, // On-disk size of live (usefull) key-value pairs.
  pub garbageDiskSize: u64,
  pub totalDiskSize: u64
}