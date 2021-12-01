extern crate databuffer;

use databuffer::*;
use std::io::{Read, Write};

#[test]
fn test_empty() {
    let mut buffer = DataBuffer::new();
    buffer.write_u8(1);
    assert_eq!(buffer.len(), 1);
}

#[test]
fn test_u8() {
    let mut buffer = DataBuffer::new();
    buffer.write_u8(0xF0);
    assert_eq!(buffer.read_u8(), 0xF0);
}

#[test]
fn test_u16() {
    let mut buffer = DataBuffer::new();
    buffer.write_u16(0xF0E1);
    assert_eq!(buffer.read_u16(), 0xF0E1);
}

#[test]
fn test_u32() {
    let mut buffer = DataBuffer::new();
    buffer.write_u32(0xF0E1D2C3);
    assert_eq!(buffer.read_u32(), 0xF0E1D2C3);
}

#[test]
fn test_u64() {
    let mut buffer = DataBuffer::new();
    buffer.write_u64(0xF0E1D2C3B4A59687);
    assert_eq!(buffer.read_u64(), 0xF0E1D2C3B4A59687);
}

#[test]
fn test_signed() {
    let mut buffer = DataBuffer::new();
    buffer.write_i8(-1);
    assert_eq!(buffer.read_u8(), 0xFF);
}

#[test]
fn test_string() {
    let mut buffer = DataBuffer::new();
    buffer.write_str("hello");
    assert_eq!(buffer.read_string(), "hello");
}

#[test]
fn test_nt_string() {
    let mut buffer = DataBuffer::new();
    buffer.write_ntstr("Hello.");
    assert_eq!("Hello.", buffer.read_ntstr());
}

#[test]
fn test_dnt_string() {
    let mut buffer = DataBuffer::new();
    buffer.write_dntstr("Hello.");
    assert_eq!("Hello.", buffer.read_dntstr());
}

#[test]
fn test_mixed() {
    let mut buffer = DataBuffer::new();
    buffer.write_i16(-1);
    buffer.write_str("hello");
    buffer.write_u64(0xF0E1D2C3B4A59687);
    assert_eq!(buffer.read_i16(), -1);
    assert_eq!(buffer.read_string(), "hello");
    assert_eq!(buffer.read_u64(), 0xF0E1D2C3B4A59687);
}

#[test]
fn test_to_string() {
    let mut buffer = DataBuffer::new();
    buffer.write_str("hello");
    assert_eq!(buffer.to_string(), "0x00 0x00 0x00 0x05 0x68 0x65 0x6c 0x6c 0x6f");
}


#[test]
fn test_wpos() {
    let mut buffer = DataBuffer::new();
    buffer.write_u32(0);
    buffer.set_wpos(1);
    buffer.write_u8(0xFF);
    buffer.write_u8(0x11);
    assert_eq!(buffer.read_u32(), 0x00FF1100);
}

#[test]
fn test_rpos() {
    let mut buffer = DataBuffer::new();
    buffer.write_u32(0x0000FF00);
    buffer.set_rpos(2);
    assert_eq!(buffer.read_u8(), 0xFF);
}

#[test]
fn test_to_bytes() {
    let mut buffer = DataBuffer::new();
    buffer.write_u8(0xFF);
    assert_eq!(buffer.to_bytes(), vec![0xFF]);
}

#[test]
fn test_from_bytes() {
    let mut buffer = DataBuffer::from_bytes(&vec![1, 2]);
    assert_eq!(buffer.read_u8() + buffer.read_u8(), 3);
}

#[test]
fn test_read_bit() {
    let mut buffer = DataBuffer::from_bytes(&vec![128]);
    let bit1 = buffer.read_bit();
    assert_eq!(bit1, true);
    let bit2 = buffer.read_bit();
    assert_eq!(bit2, false);
}

#[test]
fn test_write_bit() {
    let mut buffer = DataBuffer::new();
    buffer.write_bit(true);
    buffer.write_bit(true);
    buffer.write_bit(false);
    assert_eq!(buffer.to_bytes()[0], 128 + 64);
}

#[test]
fn test_write_bits() {
    let mut buffer = DataBuffer::new();
    buffer.write_bits(6, 3); // 110b
    assert_eq!(buffer.to_bytes()[0], 128 + 64);
}

#[test]
fn test_flush_bit() {
    let mut buffer = DataBuffer::new();
    buffer.write_bit(true);
    buffer.write_i8(1);

    let buffer_result_1 = buffer.to_bytes();
    assert_eq!(buffer_result_1[0], 128);
    assert_eq!(buffer_result_1[1], 1);

    let mut buffer2 = DataBuffer::from_bytes(&vec![0xFF, 0x01]);
    let bit1 = buffer2.read_bit();
    let number1 = buffer2.read_i8();

    assert_eq!(bit1, true);
    assert_eq!(number1, 1);
}

#[test]
fn test_read_empty_buffer() {
    let mut buffer = DataBuffer::new();
    buffer.write_u8(0xFF);
    let mut res = [];
    let _ = buffer.read(&mut res);
}

#[test]
fn test_read_exact_buffer() {
    let mut buffer = DataBuffer::new();
    buffer.write_u8(0xFF);
    let mut res = [0; 1];
    let _ = buffer.read(&mut res);
    assert_eq!(res[0], 0xFF);
}

#[test]
fn test_read_larger_buffer() {
    let mut buffer = DataBuffer::new();
    buffer.write_u8(0xFF);
    let mut res = [0; 2];
    let _ = buffer.read(&mut res);
    assert_eq!(res[0], 0xFF);
    assert_eq!(res[1], 0);
}

#[test]
fn test_read_larger_buffer_twice() {
    let mut buffer = DataBuffer::new();
    buffer.write_u8(0xFF);
    let mut res = [0; 2];
    let _ = buffer.read(&mut res);
    // Check for overflow on second read
    let _ = buffer.read(&mut res);
    assert_eq!(res[0], 0xFF);
    assert_eq!(res[1], 0);
}

#[test]
fn test_write() {
    let mut buffer = DataBuffer::new();
    let _ = buffer.write(&[0x1, 0xFF, 0x45]);
    assert_eq!(buffer.read_bytes(3), &[0x1, 0xFF, 0x45]);
}

#[test]
fn test_flush() {
    let mut buffer = DataBuffer::new();
    let _ = buffer.flush();
}

#[test]
fn test_byte_packet_header() {
    let mut buffer = DataBuffer::create(3, PacketHeader::BYTE);
    buffer.write_u8(255);
    buffer.write_u8(255);
    buffer.finish();

    let bytes = buffer.to_bytes();

    assert_eq!(bytes[0], 3);
    assert_eq!(bytes[1], 2);
}

#[test]
fn test_short_packet_header() {
    let mut buffer = DataBuffer::create(3, PacketHeader::SHORT);
    buffer.write_u8(255);
    buffer.write_u8(255);
    buffer.finish();

    let bytes = buffer.to_bytes();

    assert_eq!(bytes[0], 3);
    assert_eq!(bytes[1] + bytes[2], 2);
}

#[test]
fn test_no_packet_header() {
    let mut buffer = DataBuffer::create(3, PacketHeader::NORMAL);
    buffer.write_u8(255);
    buffer.write_u8(255);
    buffer.finish();

    let bytes = buffer.to_bytes();

    assert_eq!(bytes[0], 3);
    assert_eq!(bytes[1], 255);
    assert_eq!(bytes[2], 255);
}