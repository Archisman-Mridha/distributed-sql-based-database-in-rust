use serde::Serialize;

pub fn serialize<T: Serialize>(key: &T) {
  let mut serializer = Serializer::default();
}

#[derive(Default)]
struct Serializer {
  output: Vec<u8>,
}

impl serde::Serializer for &mut Serializer {
  type Ok;

  type Error;

  type SerializeSeq;

  type SerializeTuple;

  type SerializeTupleStruct;

  type SerializeTupleVariant;

  type SerializeMap;

  type SerializeStruct;

  type SerializeStructVariant;

  fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
    let serializedValue = v as u8;
    self.output.push(serializedValue);

    Ok(())
  }

  fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
    let serializedValue = v.to_be_bytes();
    self.output.extend(serializedValue);

    Ok(())
  }

  fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
    let serializedValue = v.to_be_bytes();

    /*
      Signed integers are represented using 2's complement and in the Big Endian (BE) format.

      The very first bit of the signed integer, represents the sign of the integer. So, if it's 1,
      that means the integer is negative, otherwise it's positive.
      So, -1 will be represented as 1111....1111 and 1 will be represented as 0000....0001.

      Now, if you try to lexicographically order these signed integers, you'll notice that the
      positive integers are coming first (due to their very first bit being 0) and the negative
      integers are coming after them (due to their very first bit being 1).
      That's why, we will always flip the first bit of the signed integer, so that the negative
      integers come before the positive integers.
    */
    // Flip the very first bit using an XOR operation.
    serializedValue[0] ^= 1 << 7;

    self.output.extend(serializedValue);

    Ok(())
  }

  fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
    unimplemented!()
  }

  fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
    unimplemented!()
  }

  fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
    unimplemented!()
  }

  fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
    unimplemented!()
  }

  fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
  where
    T: ?Sized + Serialize,
  {
    unimplemented!()
  }

  fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
    unimplemented!()
  }

  fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
    unimplemented!()
  }

  fn serialize_unit_variant(
    self,
    name: &'static str,
    variant_index: u32,
    variant: &'static str,
  ) -> Result<Self::Ok, Self::Error> {
    unimplemented!()
  }

  fn serialize_newtype_struct<T>(
    self,
    name: &'static str,
    value: &T,
  ) -> Result<Self::Ok, Self::Error>
  where
    T: ?Sized + Serialize,
  {
    unimplemented!()
  }

  fn serialize_newtype_variant<T>(
    self,
    name: &'static str,
    variant_index: u32,
    variant: &'static str,
    value: &T,
  ) -> Result<Self::Ok, Self::Error>
  where
    T: ?Sized + Serialize,
  {
    unimplemented!()
  }

  fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
    unimplemented!()
  }

  fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
    unimplemented!()
  }

  fn serialize_tuple_struct(
    self,
    name: &'static str,
    len: usize,
  ) -> Result<Self::SerializeTupleStruct, Self::Error> {
    unimplemented!()
  }

  fn serialize_tuple_variant(
    self,
    name: &'static str,
    variant_index: u32,
    variant: &'static str,
    len: usize,
  ) -> Result<Self::SerializeTupleVariant, Self::Error> {
    unimplemented!()
  }

  fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
    unimplemented!()
  }

  fn serialize_struct(
    self,
    name: &'static str,
    len: usize,
  ) -> Result<Self::SerializeStruct, Self::Error> {
    unimplemented!()
  }

  fn serialize_struct_variant(
    self,
    name: &'static str,
    variant_index: u32,
    variant: &'static str,
    len: usize,
  ) -> Result<Self::SerializeStructVariant, Self::Error> {
    unimplemented!()
  }

  // Unimplemented traits.

  fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
    unimplemented!()
  }

  fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
    unimplemented!()
  }

  fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
    unimplemented!()
  }

  fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
    unimplemented!()
  }

  fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
    unimplemented!()
  }

  fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
    unimplemented!()
  }

  fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
    unimplemented!()
  }

  fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
    unimplemented!()
  }
}
