
/// THIS FILE HAS BEEN MODIFIED HEAVILY BY Ceikry IN ORDER TO ACCOMODATE ADDITIONAL FEATURES, AND SEPARATE NAMING BASED ON PERSONAL PREFERENCE.
/// Much love to the original authors who provided the groundwork <3 (Such authors are mentioned in the attributions in cargo.toml)
extern crate byteorder;

use byteorder::{ByteOrder, BigEndian};
use std::{io::{Read, Write, Result}, convert::TryInto};

/// A data buffer object specifically turned to easily read and write binary values
pub struct DataBuffer {
    data: Vec<u8>,
    wpos: usize,
    rpos: usize,
    rbit: usize,
    wbit: usize,
    header: PacketHeader
}

pub enum PacketHeader {
    NORMAL,
    SHORT,
    BYTE
}

impl DataBuffer {
    /// Construct a new, empty, DataBuffer
    pub fn new() -> DataBuffer {
        DataBuffer {
            data: vec![],
            wpos: 0,
            rpos: 0,
            rbit: 0,
            wbit: 0,
            header: PacketHeader::NORMAL
        }
    }

    /// Borrows the data from a [`Vec<u8>`] to produce a [`DataBuffer`]. 
    /// This method is the equivalent of creating a new() [`DataBuffer`]
    /// and calling the .write_bytes() method to effectively
    /// copy all bytes from the borrowed vector into this buffer's underlying
    /// vector.
    pub fn from_bytes(bytes: &[u8]) -> DataBuffer {
        let mut buffer = DataBuffer::new();
        buffer.write_bytes(bytes);
        buffer
    }

    /// Takes ownership of a [`Vec<u8>`] and uses it for this buffer's
    /// underlying vector. This is not a borrow, you will not be able to use
    /// the original vector anymore after calling this method.
    /// This method is useful to avoid excessive copying of data.
    pub fn with_vec(bytes: Vec<u8>) -> DataBuffer {
        DataBuffer { data: bytes, wpos: 0, rpos: 0, rbit: 0, wbit: 0, header: PacketHeader::NORMAL }
    }

    pub fn create(opcode: u8, header: PacketHeader) -> DataBuffer {
        let mut buff = DataBuffer::new();
        buff.write_u8(opcode);

        match header {
            PacketHeader::NORMAL => {},
            PacketHeader::BYTE => {
                buff.write_u8(255);
            }
            PacketHeader::SHORT => {
                buff.write_u16(65535);
            }
        }

        buff.header = header;

        buff
    }

    pub fn finish(&mut self){
        let mut size = self.get_wpos();

        self.set_wpos(1);
        self.set_rpos(1);

        let header_bytes = match self.header {
            PacketHeader::NORMAL => 0,
            PacketHeader::BYTE => 1,
            PacketHeader::SHORT => 2
        };

        size = ((size as u8) - header_bytes - 1).try_into().unwrap();

        match self.header {
            PacketHeader::NORMAL => {},
            PacketHeader::BYTE => {
                self.write_u8(size.try_into().unwrap());
            }
            PacketHeader::SHORT => {
                self.write_u16(size.try_into().unwrap());
            }
        }

        self.set_rpos(0);
    }

    /// Return the buffer size
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Clear the buffer and reinitialize the reading and writing cursor
    pub fn clear(&mut self) {
        self.data.clear();
        self.wpos = 0;
        self.rpos = 0;
    }

    /// Change the buffer size to size.
    ///
    /// _Note_: You cannot shrink a buffer with this method
    pub fn resize(&mut self, size: usize) {
        let diff = size - self.data.len();
        if diff > 0 {
            self.data.extend(std::iter::repeat(0).take(diff))
        }
    }

    // Write operations

    /// Append a byte array to the buffer. The buffer is automatically extended if needed
    ///
    /// #Example
    ///
    /// ```
    /// # use databuffer::*;
    /// let mut buffer = DataBuffer::new();
    /// buffer.write_bytes(&vec![0x1, 0xFF, 0x45]); // buffer contains [0x1, 0xFF, 0x45]
    /// ```
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.flush_bit();

        let size = bytes.len() + self.wpos;

        if size > self.data.len() {
            self.resize(size);
        }

        for v in bytes {
            self.data[self.wpos] = *v;
            self.wpos += 1;
        }
    }

    /// Append a byte (8 bits value) to the buffer
    ///
    /// #Example
    ///
    /// ```
    /// #  use databuffer::*;
    /// let mut buffer = DataBuffer::new();
    /// buffer.write_u8(1) // buffer contains [0x1]
    /// ```
    pub fn write_u8(&mut self, val: u8) {
        self.write_bytes(&[val]);
    }

    /// Same as `write_u8()` but for signed values
    pub fn write_i8(&mut self, val: i8) {
        self.write_u8(val as u8);
    }

    /// Append a word (16 bits value) to the buffer
    ///
    /// #Example
    ///
    /// ```
    /// #  use databuffer::*;
    /// let mut buffer = DataBuffer::new();
    /// buffer.write_u16(1) // buffer contains [0x00, 0x1] if little endian
    /// ```
    pub fn write_u16(&mut self, val: u16) {
        let mut buf = [0; 2];
        BigEndian::write_u16(&mut buf, val);
        self.write_bytes(&buf);
    }

    /// Same as `write_u16()` but for signed values
    pub fn write_i16(&mut self, val: i16) {
        self.write_u16(val as u16);
    }

    /// Append a double word (32 bits value) to the buffer
    ///
    /// #Example
    ///
    /// ```
    /// #  use databuffer::*;
    /// let mut buffer = DataBuffer::new();
    /// buffer.write_u32(1) // buffer contains [0x00, 0x00, 0x00, 0x1] if little endian
    /// ```
    pub fn write_u32(&mut self, val: u32) {
        let mut buf = [0; 4];
        BigEndian::write_u32(&mut buf, val);
        self.write_bytes(&buf);
    }

    /// Same as `write_u32()` but for signed values
    pub fn write_i32(&mut self, val: i32) {
        self.write_u32(val as u32);
    }

    /// Append a quaddruple word (64 bits value) to the buffer
    ///
    /// #Example
    ///
    /// ```
    /// #  use databuffer::*;
    /// let mut buffer = DataBuffer::new();
    /// buffer.write_u64(1) // buffer contains [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1] if little endian
    /// ```
    pub fn write_u64(&mut self, val: u64) {
        let mut buf = [0; 8];
        BigEndian::write_u64(&mut buf, val);
        self.write_bytes(&buf);
    }

    /// Same as `write_u64()` but for signed values
    pub fn write_i64(&mut self, val: i64) {
        self.write_u64(val as u64);
    }

    /// Append a 32 bits floating point number to the buffer.
    ///
    /// #Example
    ///
    /// ```
    /// #  use databuffer::*;
    /// let mut buffer = DataBuffer::new();
    /// buffer.write_f32(0.1)
    /// ```
    pub fn write_f32(&mut self, val: f32) {
        let mut buf = [0; 4];
        BigEndian::write_f32(&mut buf, val);
        self.write_bytes(&buf);
    }

    /// Append a 64 bits floating point number to the buffer.
    ///
    /// #Example
    ///
    /// ```
    /// #  use databuffer::*;
    /// let mut buffer = DataBuffer::new();
    /// buffer.write_f64(0.1)
    /// ```
    pub fn write_f64(&mut self, val: f64) {
        let mut buf = [0; 8];
        BigEndian::write_f64(&mut buf, val);
        self.write_bytes(&buf);
    }

    /// Append a string to the buffer.
    ///
    /// *Format* The format is `(u32)size + size * (u8)characters`
    ///
    /// #Example
    ///
    /// ```
    /// #  use databuffer::*;
    /// let mut buffer = DataBuffer::new();
    /// buffer.write_str("Hello")
    /// ```
    pub fn write_str(&mut self, val: &str) {
        self.write_u32(val.len() as u32);
        self.write_bytes(val.as_bytes());
    }

    /// Write a single null-terminated string to the buffer.
    /// 
    /// #Example
    ///
    ///  ```
    /// # use databuffer::*;
    /// let mut buffer = DataBuffer::new();
    /// buffer.write_ntstr("Hello");
    /// ```
    pub fn write_ntstr(&mut self, val: &str) {
        self.write_bytes(val.as_bytes());
        self.write_u8(0);
    }

    /// Write a null-bookeneded string to the buffer.
    /// 
    /// #Example
    /// ```
    /// # use databuffer::*;
    /// let mut buffer = DataBuffer::new();
    /// buffer.write_dntstr("Hello");
    /// ```
    pub fn write_dntstr(&mut self, val: &str){
        self.write_u8(0);
        self.write_bytes(val.as_bytes());
        self.write_u8(0);
    }

    /// Writes a smart to the buffer, which is a dynamically-sized unit with a max value of 32768.
    pub fn write_smart(&mut self, val: u16){
        if val > 128 {
            self.write_u16(val.wrapping_add(32768));
        } else {
            self.write_u8(val as u8);
        }
    }

    /// Writes a medium to the buffer, which is a tribyte word.
    pub fn write_medium(&mut self, val: u32) {
        self.write_i8((val >> 16) as i8);
        self.write_i8(((val >> 8) & 0xFF) as i8);
        self.write_i8((val & 0xFF) as i8);
    }

    /// Writes an inverted-signededness u8 to the buffer.
    pub fn write_u8_neg(&mut self, val: u8) {
        self.write_i8(-(val as i8));    
    }

    /// Writes an inverted-signededness i8 to the buffer.
    pub fn write_i8_neg(&mut self, val: i8) {
        self.write_i8(val.wrapping_neg());    
    }

    /// Writes a u8 + 128 to the buffer.
    pub fn write_u8_add(&mut self, val: u8) {
        self.write_u8(val.wrapping_add(128));
    }

    /// Writes a i8 + 128 to the buffer.
    pub fn write_i8_add(&mut self, val: i8) {
        self.write_i8(val.wrapping_add(127).wrapping_add(1));
    }

    /// Writes a u8 - 128 to the buffer.
    pub fn write_u8_sub(&mut self, val: u8) {
        self.write_i8(val.wrapping_sub(128) as i8);
    }

    /// Writes a i8 - 128 to the buffer.
    pub fn write_i8_sub(&mut self, val: i8) {
        self.write_i8(val.wrapping_sub(127).wrapping_sub(1));
    }
    
    /// Writes a little-endian u16 to the buffer.
    pub fn write_u16_le(&mut self, val: u16) {
        self.write_u8(val as u8);
        self.write_u8((val >> 8) as u8);
    }

    /// Writes a little-endian i16 to the buffer.
    pub fn write_i16_le(&mut self, val: i16) {
        self.write_i8(val as i8);
        self.write_i8((val >> 8) as i8);
    }

    /// Writes a little-endian u16 + 128 to the buffer.
    pub fn write_u16_le_add(&mut self, val: u16) {
        self.write_u8(val.wrapping_add(128) as u8);
        self.write_u8((val >> 8) as u8);
    }

    /// Writes a little-endian u16 + 128 to the buffer.
    pub fn write_i16_le_add(&mut self, val: i16) {
        self.write_i8_add(val as i8);
        self.write_i8((val >> 8) as i8);
    }

    /// Writes a little-endian u32 to the buffer.
    pub fn write_u32_le(&mut self, val: u32){
        self.write_u8(val as u8);
        self.write_u8((val >> 8) as u8);
        self.write_u8((val >> 16) as u8);
        self.write_u8((val >> 24) as u8);
    }

    /// Writes a little-endian i32 to the buffer.
    pub fn write_i32_le(&mut self, val: i32){
        self.write_i8(val as i8);
        self.write_i8((val >> 8) as i8);
        self.write_i8((val >> 16) as i8);
        self.write_i8((val >> 24) as i8);
    }

    /// Writes a mixed-endian u32 to the buffer.
    pub fn write_u32_me(&mut self, val: u32){
        self.write_u8((val >> 16) as u8);
        self.write_u8((val >> 24) as u8);
        self.write_u8(val as u8);
        self.write_u8((val >> 8) as u8);
    }

    /// Writes a reverse mixed-endian u32 to the buffer.
    pub fn write_u32_me_reversed(&mut self, val: u32){
        self.write_u8((val >> 8) as u8);
        self.write_u8(val as u8);
        self.write_u8((val >> 24) as u8);
        self.write_u8((val >> 16) as u8);
    }

    // Read operations

    /// Read a defined amount of raw bytes. The program crash if not enough bytes are available
    pub fn read_bytes(&mut self, size: usize) -> Vec<u8> {
        self.flush_bit();
        assert!(self.rpos + size <= self.data.len());
        let range = self.rpos..self.rpos + size;
        let mut res = Vec::<u8>::new();
        res.write(&self.data[range]).unwrap();
        self.rpos += size;
        res
    }

    /// Read one byte. The program crash if not enough bytes are available
    ///
    /// #Example
    ///
    /// ```
    /// #  use databuffer::*;
    /// let mut buffer = DataBuffer::from_bytes(&vec![0x1]);
    /// let value = buffer.read_u8(); //Value contains 1
    /// ```
    pub fn read_u8(&mut self) -> u8 {
        self.flush_bit();
        assert!(self.rpos < self.data.len());
        let pos = self.rpos;
        self.rpos += 1;
        self.data[pos]
    }

    /// Same as `read_u8()` but for signed values
    pub fn read_i8(&mut self) -> i8 {
        self.read_u8() as i8
    }

    /// Read a 2-bytes long value. The program crash if not enough bytes are available
    ///
    /// #Example
    ///
    /// ```
    /// #  use databuffer::*;
    /// let mut buffer = DataBuffer::from_bytes(&vec![0x0, 0x1]);
    /// let value = buffer.read_u16(); //Value contains 1
    /// ```
    pub fn read_u16(&mut self) -> u16 {
        self.flush_bit();
        assert!(self.rpos + 2 <= self.data.len());
        let range = self.rpos..self.rpos + 2;
        self.rpos += 2;
        BigEndian::read_u16(&self.data[range])
    }

    /// Same as `read_u16()` but for signed values
    pub fn read_i16(&mut self) -> i16 {
        self.read_u16() as i16
    }

    /// Read a four-bytes long value. The program crash if not enough bytes are available
    ///
    /// #Example
    ///
    /// ```
    /// #  use databuffer::*;
    /// let mut buffer = DataBuffer::from_bytes(&vec![0x0, 0x0, 0x0, 0x1]);
    /// let value = buffer.read_u32(); // Value contains 1
    /// ```
    pub fn read_u32(&mut self) -> u32 {
        self.flush_bit();
        assert!(self.rpos + 4 <= self.data.len());
        let range = self.rpos..self.rpos + 4;
        self.rpos += 4;
        BigEndian::read_u32(&self.data[range])
    }

    /// Same as `read_u32()` but for signed values
    pub fn read_i32(&mut self) -> i32 {
        self.read_u32() as i32
    }

    /// Read an eight bytes long value. The program crash if not enough bytes are available
    ///
    /// #Example
    ///
    /// ```
    /// #  use databuffer::*;
    /// let mut buffer = DataBuffer::from_bytes(&vec![0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1]);
    /// let value = buffer.read_u64(); //Value contains 1
    /// ```
    pub fn read_u64(&mut self) -> u64 {
        self.flush_bit();
        assert!(self.rpos + 8 <= self.data.len());
        let range = self.rpos..self.rpos + 8;
        self.rpos += 8;
        BigEndian::read_u64(&self.data[range])
    }

    /// Same as `read_u64()` but for signed values
    pub fn read_i64(&mut self) -> i64 {
        self.read_u64() as i64
    }

    /// Read a 32 bits floating point value. The program crash if not enough bytes are available
    pub fn read_f32(&mut self) -> f32 {
        self.flush_bit();
        assert!(self.rpos + 4 <= self.data.len());
        let range = self.rpos..self.rpos + 4;
        self.rpos += 4;
        BigEndian::read_f32(&self.data[range])
    }

    /// Read a 64 bits floating point value. The program crash if not enough bytes are available
    pub fn read_f64(&mut self) -> f64 {
        self.flush_bit();
        assert!(self.rpos + 8 <= self.data.len());
        let range = self.rpos..self.rpos + 8;
        self.rpos += 8;
        BigEndian::read_f64(&self.data[range])
    }

    /// Read a string.
    ///
    /// *Note* : First it reads a 32 bits value representing the size, the read 'size' raw bytes.
    pub fn read_string(&mut self) -> String {
        let size = self.read_u32();
        String::from_utf8(self.read_bytes(size as usize)).unwrap()
    }

    /// Read a null-terminated string.
    pub fn read_ntstr(&mut self) -> String {
        let mut string = String::new();

        let mut current = self.read_u8();
        while current != 0 {
            string.push(current as char);
            current = self.read_u8();
        }

        string
    }

    /// Read a null-bookended string.
    pub fn read_dntstr(&mut self) -> String {
        let mut string = String::new();

        assert_eq!(0, self.read_u8());

        let mut current = self.read_u8();
        while current != 0 {
            string.push(current as char);
            current = self.read_u8();
        }

        string
    }

    /// Reads a smart from the buffer, which is a dynamically-sized unit with a max value of 32768.
    pub fn read_smart(&mut self) -> u16 {
        if self.data[self.rpos] >= 128 {
            return self.read_u16().wrapping_sub(32768);
        }
        else {
            return self.read_u8() as u16;
        }
    }

    /// Reads a medium from the buffer, which is a tribyte word.
    pub fn read_medium(&mut self) -> u32 {
        return ((self.read_u8() as u32) << 16) + ((self.read_u8() as u32) << 8) + (self.read_u8() as u32);
    }

    /// Reads an inverted-signededness u8 from the buffer.
    pub fn read_u8_neg(&mut self) -> u8 {
        return -self.read_i8() as u8;   
    }

    /// Reads an inverted-signededness i8 from the buffer.
    pub fn read_i8_neg(&mut self) -> i8 {
        return self.read_i8().wrapping_neg();
    }

    /// Reads a u8 + 128 from the buffer, and subtracts the extra 128.
    pub fn read_u8_add(&mut self) -> u8 {
        return self.read_u8().wrapping_sub(128);
    }

    /// Reads a i8 + 128 from the buffer, and subtracts the extra 128.
    pub fn read_i8_add(&mut self) -> i8 {
        return self.read_u8().wrapping_sub(128) as i8;
    }

    /// Reads a u8 - 128 from the buffer, and adds back the missing 128.
    pub fn read_u8_sub(&mut self) -> u8 {
        return self.read_u8().wrapping_add(128) as u8;
    }

    /// Reads a i8 - 128 from the buffer, and adds back the missing 128.
    pub fn read_i8_sub(&mut self) -> i8 {
        return self.read_i8().wrapping_add(127).wrapping_add(1) as i8;
    }

    /// Reads a little-endian u16 from the buffer.
    pub fn read_u16_le(&mut self) -> u16 {
        let mut bytes: [u8; 2] = [0; 2];
        let _ = self.read(&mut bytes);
        return u16::from_le_bytes(bytes);
    }

    /// Reads a little-endian i16 from the buffer.
    pub fn read_i16_le(&mut self) -> i16 {
        let mut bytes: [u8; 2] = [0; 2];
        let _ = self.read(&mut bytes);
        return i16::from_le_bytes(bytes);
    }

    /// Reads a little-endian u16 + 128 from the buffer, and subtracts the extra 128.
    pub fn read_u16_le_add(&mut self) -> u16 {
        let mut bytes: [u8; 2] = [0; 2];
        let _ = self.read(&mut bytes);
        bytes[0] = bytes[0].wrapping_sub(128);
        return u16::from_le_bytes(bytes);
    }

    /// Reads a little-endian i16 + 128 from the buffer, and subtracts the extra 128.
    pub fn read_i16_le_add(&mut self) -> i16 {
        let mut bytes: [u8; 2] = [0; 2];
        let _ = self.read(&mut bytes);
        bytes[0] = bytes[0].wrapping_sub(128);
        return i16::from_le_bytes(bytes);
    }

    /// Reads a little-endian u32 from the buffer.
    pub fn read_u32_le(&mut self) -> u32 {
        let mut bytes: [u8; 4] = [0; 4];
        let _ = self.read(&mut bytes);
        return u32::from_le_bytes(bytes);
    }

    /// Reads a little-endian u32 from the buffer.
    pub fn read_i32_le(&mut self) -> i32 {
        let mut bytes: [u8; 4] = [0; 4];
        let _ = self.read(&mut bytes);
        return i32::from_le_bytes(bytes);
    }

    /// Reads a mixed-endian u32 from the buffer.
    pub fn read_u32_me(&mut self) -> u32 {
        return ((self.read_u8() as u32) << 16) + ((self.read_u8() as u32) << 24) + ((self.read_u8() as u32)) + ((self.read_u8() as u32) << 8);
    }

    /// Reads a reverse mixed-endian u32 from the buffer.
    pub fn read_u32_me_reversed(&mut self) -> u32 {
        return ((self.read_u8() as u32) << 8) + (self.read_u8() as u32) + ((self.read_u8() as u32) << 24) + ((self.read_u8() as u32) << 16);
    }

    // Other

    /// Dump the byte buffer to a string.
    pub fn to_string(&self) -> String {
        let mut str = String::new();
        for b in &self.data {
            str = str + &format!("0x{:01$x} ", b, 2);
        }
        str.pop();
        str
    }

    /// Return the position of the reading cursor
    pub fn get_rpos(&self) -> usize {
        self.rpos
    }

    /// Set the reading cursor position.
    /// *Note* : Set the reading cursor to `min(newPosition, self.len())` to prevent overflow
    pub fn set_rpos(&mut self, rpos: usize) {
        self.rpos = std::cmp::min(rpos, self.data.len());
    }

    /// Return the writing cursor position
    pub fn get_wpos(&self) -> usize {
        self.wpos
    }

    /// Set the writing cursor position.
    /// *Note* : Set the writing cursor to `min(newPosition, self.len())` to prevent overflow
    pub fn set_wpos(&mut self, wpos: usize) {
        self.wpos = std::cmp::min(wpos, self.data.len());
    }

    /// Return the raw byte buffer.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.data.to_vec()
    }

    //Bit manipulation functions

    /// Read 1 bit. Return true if the bit is set to 1, otherwhise, return false.
    ///
    /// **Note** Bits are read from left to right
    ///
    /// #Example
    ///
    /// ```
    /// #  use databuffer::*;
    /// let mut buffer = DataBuffer::from_bytes(&vec![128]); // 10000000b
    /// let value1 = buffer.read_bit(); //value1 contains true (eg: bit is 1)
    /// let value2 = buffer.read_bit(); //value2 contains false (eg: bit is 0)
    /// ```
    pub fn read_bit(&mut self) -> bool {
        assert!(self.rpos <= self.data.len());
        let bit = self.data[self.rpos] & (1 << 7 - self.rbit) != 0;
        self.rbit += 1;
        if self.rbit > 7 {
            self.rbit = 0;
            self.rpos += 1;
        }
        bit
    }

    /// Read n bits. an return the corresponding value an u64.
    ///
    /// **Note 1** : We cannot read more than 64 bits
    ///
    /// **Note 2** Bits are read from left to right
    ///
    /// #Example
    ///
    /// ```
    /// #  use databuffer::*;
    /// let mut buffer = DataBuffer::from_bytes(&vec![128]); // 10000000b
    /// let value = buffer.read_bits(3); // value contains 4 (eg: 100b)
    /// ```
    pub fn read_bits(&mut self, n: u8) -> u64 {
        // TODO : Assert that n <= 64
        if n > 0 {
            ((if self.read_bit() { 1 } else { 0 }) << n - 1) | self.read_bits(n - 1)
        } else {
            0
        }
    }

    /// Discard all the pending bits available for reading or writing and place the the corresponding cursor to the next byte.
    ///
    /// **Note 1** : If no bits are currently read or written, this function does nothing.
    /// **Note 2** : This function is automatically called for each write or read operations.
    /// #Example
    ///
    /// ```text
    /// 10010010 | 00000001
    /// ^
    /// 10010010 | 00000001 // read_bit called
    ///  ^
    /// 10010010 | 00000001 // flush_bit() called
    ///            ^
    /// ```
    pub fn flush_bit(&mut self) {
        if self.rbit > 0 {
            self.rpos += 1;
            self.rbit = 0
        }

        if self.wbit > 0 {
            self.wpos += 1;
            self.wbit = 0
        }
    }

    /// Append 1 bit value to the buffer.
    /// The bit is happened like this :
    ///
    /// ```text
    /// ...| XXXXXXXX | 10000000 |....
    /// ```
    pub fn write_bit(&mut self, bit: bool) {
        let size = self.wpos + 1;
        if size > self.data.len() {
            self.resize(size);
        }

        if bit {
            self.data[self.wpos] |= 1 << (7 - self.wbit);
        }

        self.wbit += 1;

        if self.wbit > 7 {
            self.wbit = 0;
            self.wpos += 1;
        }
    }

    /// Write the given value as a sequence of n bits
    ///
    /// #Example
    ///
    /// ```
    /// #  use databuffer::*;
    /// let mut buffer = DataBuffer::new();
    /// buffer.write_bits(4, 3); // append 100b
    /// ```
    pub fn write_bits(&mut self, value: u64, n: u8) {
        if n > 0 {
            self.write_bit((value >> n - 1) & 1 != 0);
            self.write_bits(value, n - 1);
        } else {
            self.write_bit((value & 1) != 0);
        }
    }
}

impl Read for DataBuffer {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.flush_bit();
        let read_len = std::cmp::min(self.data.len() - self.rpos, buf.len());
        let range = self.rpos..self.rpos + read_len;
        for (i, val) in (&self.data[range]).iter().enumerate() {
            buf[i] = *val;
        }
        self.rpos += read_len;
        Ok(read_len)
    }
}

impl Write for DataBuffer {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.write_bytes(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl std::fmt::Debug for DataBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let rpos = if self.rbit > 0 {
            self.rpos + 1
        } else {
            self.rpos
        };

        let read_len = self.data.len() - rpos;
        let mut remaining_data = vec![0; read_len];
        let range = rpos..rpos + read_len;
        for (i, val) in (&self.data[range]).iter().enumerate() {
            remaining_data[i] = *val;
        }

        write!(f, "DataBuffer {{ remaining_data: {:?}, total_data: {:?} }}",
               remaining_data, self.data)
    }
}
