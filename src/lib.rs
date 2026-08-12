pub mod binary {
    /// A high-performance binary stream for reading and writing primitive types
    /// in both big-endian and little-endian byte order.
    ///
    /// Commonly used for RakNet & Minecraft Bedrock protocol data.
    pub struct Reader<'a> {
        buffer: &'a[u8],
        offset: usize
    }

    impl<'a> Reader<'a> {
        /// Creates a new stream from an existing buffer and offset.
        #[inline]
        pub const fn new(buffer: &'a[u8]) -> Self {
            Self { buffer, offset: 0 }
        }

        /// Returns the current offset.
        #[inline]
        pub const fn offset(&self) -> usize {
            self.offset
        }


        /// Number of unread bytes.
        #[inline]
        pub const fn remaining_byte_count(&self) -> usize {
            self.buffer.len() - self.offset
        }

        /// Returns `true` if the buffer is empty.
        #[inline]
        pub const fn is_empty(&self) -> bool {
            self.offset >= self.buffer.len()
        }

        /// Returns the entire buffer as a byte slice.
        #[inline]
        pub fn get_buffer(&self) -> &'a [u8] {
            self.buffer
        }

        /// Returns the remaining unread bytes.
        #[inline]
        pub fn remaining(&self) -> &'a [u8] {
            &self.buffer[self.offset..]
        }

        /// Sets the buffer offset.
        #[inline]
        pub fn set_offset(&mut self, offset: usize) {
            assert!(offset <= self.buffer.len(), "Offset out of bounds");
            self.offset = offset;
        }

        /// Resets the stream offset to the beginning.
        #[inline]
        pub fn rewind(&mut self) {
            self.offset = 0;
        }

        /// Returns `true` if the stream has reached the end of the buffer.
        #[inline]
        pub fn feof(&self) -> bool {
            self.offset >= self.buffer.len()
        }

        /// Internal: Returns a slice without allocation (zero-copy)
        #[inline]
        pub fn get(&mut self, length: usize) -> &'a [u8] {
            let start = self.offset;
            let end = start + length;

            assert!(
                end <= self.buffer.len(),
                "Buffer underflow: offset={}, length={}, buffer_size={}, trying to read until={}",
                start, length, self.buffer.len(), end
            );

            self.offset = end;
            &self.buffer[start..end]
        }

        /// Reads a single unsigned byte.
        #[inline]
        pub fn get_u8(&mut self) -> u8 {
            match self.buffer.get(self.offset) {
                Some(&b) => {
                    self.offset += 1;
                    b
                },
                None => panic!("Buffer underflow"),
            }
        }

        /// Reads a single signed byte.
        #[inline]
        pub fn get_i8(&mut self) -> i8 {
            self.get_u8() as i8
        }

        /// Reads a single byte and returns `true` if it's nonzero.
        #[inline]
        pub fn get_bool(&mut self) -> bool {
            self.get_u8() != 0
        }

        // ===== 16-bit (short) integers =====

        /// Reads an unsigned 16-bit integer (big-endian).
        #[inline]
        pub fn get_u16_be(&mut self) -> u16 {
            let bytes = self.get(2);
            u16::from_be_bytes([bytes[0], bytes[1]])
        }

        /// Reads a signed 16-bit integer (big-endian).
        #[inline]
        pub fn get_i16_be(&mut self) -> i16 {
            let bytes = self.get(2);
            i16::from_be_bytes([bytes[0], bytes[1]])
        }

        /// Reads an unsigned 16-bit integer (little-endian).
        #[inline]
        pub fn get_u16_le(&mut self) -> u16 {
            let bytes = self.get(2);
            u16::from_le_bytes([bytes[0], bytes[1]])
        }

        /// Reads a signed 16-bit integer (little-endian).
        #[inline]
        pub fn get_i16_le(&mut self) -> i16 {
            let bytes = self.get(2);
            i16::from_le_bytes([bytes[0], bytes[1]])
        }

        // ===== 24-bit (triad) integers =====

        /// Reads a 24-bit unsigned integer (big-endian).
        #[inline]
        pub fn get_u24_be(&mut self) -> u32 {
            let bytes = self.get(3);
            u32::from_be_bytes([0, bytes[0], bytes[1], bytes[2]])
        }

        /// Reads a 24-bit unsigned integer (little-endian).
        #[inline]
        pub fn get_u24_le(&mut self) -> u32 {
            let bytes = self.get(3);
            u32::from_le_bytes([bytes[0], bytes[1], bytes[2], 0])
        }

        // ===== 32-bit integers =====

        /// Reads an unsigned 32-bit integer (big-endian).
        #[inline]
        pub fn get_u32_be(&mut self) -> u32 {
            let bytes = self.get(4);
            u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
        }

        /// Reads an unsigned 32-bit integer (little-endian).
        #[inline]
        pub fn get_u32_le(&mut self) -> u32 {
            let bytes = self.get(4);
            u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
        }

        /// Reads a signed 32-bit integer (big-endian).
        #[inline]
        pub fn get_i32_be(&mut self) -> i32 {
            let bytes = self.get(4);
            i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
        }

        /// Reads a signed 32-bit integer (little-endian).
        #[inline]
        pub fn get_i32_le(&mut self) -> i32 {
            let bytes = self.get(4);
            i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
        }

        // ===== 32-bit floats =====

        /// Reads a 32-bit floating-point number (big-endian).
        #[inline]
        pub fn get_f32_be(&mut self) -> f32 {
            let bytes = self.get(4);
            f32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
        }

        /// Reads a 32-bit floating-point number (little-endian).
        #[inline]
        pub fn get_f32_le(&mut self) -> f32 {
            let bytes = self.get(4);
            f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
        }

        // ===== 64-bit floats =====

        /// Reads a 64-bit double-precision float (big-endian).
        #[inline]
        pub fn get_f64_be(&mut self) -> f64 {
            let bytes = self.get(8);
            f64::from_be_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3],
                bytes[4], bytes[5], bytes[6], bytes[7],
            ])
        }

        /// Reads a 64-bit double-precision float (little-endian).
        #[inline]
        pub fn get_f64_le(&mut self) -> f64 {
            let bytes = self.get(8);
            f64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3],
                bytes[4], bytes[5], bytes[6], bytes[7],
            ])
        }

        // ===== 64-bit integers =====

        /// Reads a signed 64-bit integer (big-endian).
        #[inline]
        pub fn get_i64_be(&mut self) -> i64 {
            let bytes = self.get(8);
            i64::from_be_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3],
                bytes[4], bytes[5], bytes[6], bytes[7],
            ])
        }

        /// Reads a signed 64-bit integer (little-endian).
        #[inline]
        pub fn get_i64_le(&mut self) -> i64 {
            let bytes = self.get(8);
            i64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3],
                bytes[4], bytes[5], bytes[6], bytes[7],
            ])
        }

        /// Reads an unsigned 64-bit integer (big-endian).
        #[inline]
        pub fn get_u64_be(&mut self) -> u64 {
            let bytes = self.get(8);
            u64::from_be_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3],
                bytes[4], bytes[5], bytes[6], bytes[7],
            ])
        }

        /// Reads an unsigned 64-bit integer (little-endian).
        #[inline]
        pub fn get_u64_le(&mut self) -> u64 {
            let bytes = self.get(8);
            u64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3],
                bytes[4], bytes[5], bytes[6], bytes[7],
            ])
        }

        // ===== Variable-length integers =====

        /// Reads an unsigned variable-length integer (VarInt).
        #[inline]
        pub fn get_var_u32(&mut self) -> u32 {
            let mut value = 0u32;
            let mut shift = 0;

            for i in 0..5 {
                let b = self.get_u8();
                if i == 4 && b & 0xF0 != 0 {
                    panic!("VarInt overflows u32");
                }
                value |= ((b & 0x7f) as u32) << shift;

                if b & 0x80 == 0 {
                    return value;
                }

                shift += 7;
            }

            unreachable!("VarInt did not terminate after 5 bytes!");
        }

        /// Reads a signed variable-length integer (ZigZag encoded).
        #[inline]
        pub fn get_var_i32(&mut self) -> i32 {
            let raw = self.get_var_u32();
            let mut value = (raw >> 1) as i32;
            if raw & 1 != 0 {
                value = !value;
            }
            value
        }

        /// Reads an unsigned 64-bit variable-length integer (VarLong).
        #[inline]
        pub fn get_var_u64(&mut self) -> u64 {
            let mut value = 0u64;
            let mut shift = 0;

            for i in 0..10 {
                let b = self.get_u8();
                if i == 9 && b & 0xFE != 0 {
                    panic!("VarLong overflows u64");
                }
                value |= ((b & 0x7f) as u64) << shift;

                if b & 0x80 == 0 {
                    return value;
                }

                shift += 7;
            }

            unreachable!("VarLong did not terminate after 10 bytes!");
        }

        /// Reads a signed 64-bit variable-length integer (ZigZag encoded).
        #[inline]
        pub fn get_var_i64(&mut self) -> i64 {
            let raw = self.get_var_u64();
            let mut value = (raw >> 1) as i64;
            if raw & 1 != 0 {
                value = !value;
            }
            value
        }
    }

    pub struct Writer {
        buffer: Vec<u8>
    }

    impl Writer {
        #[inline]
        pub const fn new() -> Writer {
            Writer { buffer: Vec::new() }
        }

        #[inline]
        pub fn with_capacity(capacity: usize) -> Writer {
            Writer {
                buffer: Vec::with_capacity(capacity),
            }
        }

        /// Wraps an existing buffer (appends to it).
        #[inline]
        pub fn from_vec(buffer: Vec<u8>) -> Writer {
            Writer { buffer }
        }

        #[inline]
        pub fn clear(&mut self) {
            self.buffer.clear();
        }

        #[inline]
        pub fn len(&self) -> usize {
            self.buffer.len()
        }

        #[inline]
        pub fn is_empty(&self) -> bool {
            self.buffer.is_empty()
        }

        #[inline]
        pub fn as_slice(&self) -> &[u8] {
            &self.buffer
        }

        #[inline]
        pub fn as_mut_slice(&mut self) -> &mut [u8] {
            &mut self.buffer
        }

        #[inline]
        pub fn into_vec(self) -> Vec<u8> {
            self.buffer
        }

        #[inline]
        pub fn reserve(&mut self, additional: usize) {
            self.buffer.reserve(additional);
        }

        #[inline]
        pub fn truncate(&mut self, len: usize) {
            self.buffer.truncate(len);
        }

        #[inline]
        pub fn resize(&mut self, len: usize, value: u8) {
            self.buffer.resize(len, value);
        }

        /// A `Reader` over what has been written so far.
        #[inline]
        pub fn reader(&self) -> Reader<'_> {
            Reader::new(&self.buffer)
        }

        /// Writes a raw byte slice to the buffer.
        #[inline]
        pub fn put(&mut self, value: &[u8]) {
            self.buffer.extend_from_slice(value);
        }

        /// Writes a single unsigned byte.
        #[inline]
        pub fn put_u8(&mut self, value: u8) {
            self.buffer.push(value);
        }

        /// Writes a single signed byte.
        #[inline]
        pub fn put_i8(&mut self, value: i8) {
            self.buffer.push(value as u8);
        }

        /// Writes a boolean value (`0x01` for true, `0x00` for false).
        #[inline]
        pub fn put_bool(&mut self, value: bool) {
            self.buffer.push(value as u8);
        }

        /// Writes an unsigned 16-bit integer (big-endian).
        #[inline]
        pub fn put_u16_be(&mut self, value: u16) {
            self.buffer.extend_from_slice(&value.to_be_bytes());
        }

        /// Writes a signed 16-bit integer (big-endian).
        #[inline]
        pub fn put_i16_be(&mut self, value: i16) {
            self.buffer.extend_from_slice(&value.to_be_bytes());
        }

        /// Writes an unsigned 16-bit integer (little-endian).
        #[inline]
        pub fn put_u16_le(&mut self, value: u16) {
            self.buffer.extend_from_slice(&value.to_le_bytes());
        }

        /// Writes a signed 16-bit integer (little-endian).
        #[inline]
        pub fn put_i16_le(&mut self, value: i16) {
            self.buffer.extend_from_slice(&value.to_le_bytes());
        }

        /// Writes a 24-bit unsigned integer (little-endian).
        #[inline]
        pub fn put_u24_le(&mut self, value: u32) {
            let bytes = value.to_le_bytes();
            self.buffer.extend_from_slice(&bytes[0..3]);
        }

        /// Writes a 24-bit unsigned integer (big-endian).
        #[inline]
        pub fn put_u24_be(&mut self, value: u32) {
            let bytes = value.to_be_bytes();
            self.buffer.extend_from_slice(&bytes[1..4]);
        }

        /// Writes a signed 32-bit integer (little-endian).
        #[inline]
        pub fn put_i32_le(&mut self, value: i32) {
            self.buffer.extend_from_slice(&value.to_le_bytes());
        }

        /// Writes an unsigned 32-bit integer (little-endian).
        #[inline]
        pub fn put_u32_le(&mut self, value: u32) {
            self.buffer.extend_from_slice(&value.to_le_bytes());
        }

        /// Writes a signed 32-bit integer (big-endian).
        #[inline]
        pub fn put_i32_be(&mut self, value: i32) {
            self.buffer.extend_from_slice(&value.to_be_bytes());
        }

        /// Writes an unsigned 32-bit integer (big-endian).
        #[inline]
        pub fn put_u32_be(&mut self, value: u32) {
            self.buffer.extend_from_slice(&value.to_be_bytes());
        }

        /// Writes a 32-bit floating-point number (little-endian).
        #[inline]
        pub fn put_f32_le(&mut self, value: f32) {
            self.buffer.extend_from_slice(&value.to_le_bytes());
        }

        /// Writes a 32-bit floating-point number (big-endian).
        #[inline]
        pub fn put_f32_be(&mut self, value: f32) {
            self.buffer.extend_from_slice(&value.to_be_bytes());
        }

        /// Writes an unsigned 64-bit integer (little-endian).
        #[inline]
        pub fn put_u64_le(&mut self, value: u64) {
            self.buffer.extend_from_slice(&value.to_le_bytes());
        }

        /// Writes an unsigned 64-bit integer (big-endian).
        #[inline]
        pub fn put_u64_be(&mut self, value: u64) {
            self.buffer.extend_from_slice(&value.to_be_bytes());
        }

        /// Writes a 64-bit double-precision float (little-endian).
        #[inline]
        pub fn put_f64_le(&mut self, value: f64) {
            self.buffer.extend_from_slice(&value.to_le_bytes());
        }

        /// Writes a signed 64-bit integer (little-endian).
        #[inline]
        pub fn put_i64_le(&mut self, value: i64) {
            self.buffer.extend_from_slice(&value.to_le_bytes());
        }

        /// Writes a 64-bit double-precision float (big-endian).
        #[inline]
        pub fn put_f64_be(&mut self, value: f64) {
            self.buffer.extend_from_slice(&value.to_be_bytes());
        }

        /// Writes a signed variable-length integer (ZigZag encoded).
        #[inline]
        pub fn put_var_i32(&mut self, value: i32) {
            let mut encoded = (value << 1) as u32;
            if value < 0 {
                encoded = !encoded;
            }
            self.put_var_u32(encoded);
        }

        /// Writes an unsigned variable-length integer (VarInt).
        #[inline]
        pub fn put_var_u32(&mut self, mut value: u32) {
            loop {
                if value >= 0x80 {
                    self.buffer.push((value as u8) | 0x80);
                    value >>= 7;
                } else {
                    self.buffer.push(value as u8);
                    break;
                }
            }
        }

        /// Writes a signed 64-bit integer (big-endian).
        #[inline]
        pub fn put_i64_be(&mut self, value: i64) {
            self.buffer.extend_from_slice(&value.to_be_bytes());
        }

        /// Writes a signed 64-bit variable-length integer (ZigZag encoded).
        #[inline]
        pub fn put_var_i64(&mut self, value: i64) {
            let mut encoded = (value << 1) as u64;
            if value < 0 {
                encoded = !encoded;
            }
            self.put_var_u64(encoded);
        }

        /// Writes an unsigned 64-bit variable-length integer (VarLong).
        #[inline]
        pub fn put_var_u64(&mut self, mut value: u64) {
            loop {
                if value >= 0x80 {
                    self.buffer.push((value as u8) | 0x80);
                    value >>= 7;
                } else {
                    self.buffer.push(value as u8);
                    break;
                }
            }
        }
    }
}