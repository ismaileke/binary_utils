pub mod binary {
    /// A binary stream for reading and writing primitive types
    /// in both big-endian and little-endian byte order.
    ///
    /// Commonly used for RakNet & Minecraft Bedrock protocol data.
    pub struct Stream {
        buffer: Vec<u8>,
        offset: u32
    }

    impl Stream {
        /// Creates a new stream from an existing buffer and offset.
        #[inline]
        pub fn new(buffer: Vec<u8>, offset: u32) -> Self {
            Self { buffer, offset }
        }

        /// Creates an empty stream with a specified capacity.
        #[inline]
        pub fn with_capacity(capacity: usize) -> Self {
            Self {
                buffer: Vec::with_capacity(capacity),
                offset: 0,
            }
        }

        /// Resets the stream offset to the beginning.
        #[inline]
        pub fn rewind(&mut self) {
            self.offset = 0;
        }

        /// Returns `true` if the stream has reached the end of the buffer.
        #[inline]
        pub fn feof(&self) -> bool {
            self.offset >= self.buffer.len() as u32
        }

        /// Sets the read/write offset.
        ///
        /// Panics if the offset is greater than the buffer length.
        #[inline]
        pub fn set_offset(&mut self, offset: u32) {
            if offset <= self.buffer.len() as u32 {
                self.offset = offset;
            } else {
                panic!("Offset out of bounds");
            }
        }

        /// Returns the current read/write offset.
        #[inline]
        pub fn get_offset(&self) -> u32 {
            self.offset
        }

        /// Returns the entire buffer as a byte slice.
        #[inline]
        pub fn get_buffer(&self) -> &[u8] {
            &self.buffer
        }

        /// Reads a specific number of bytes and advances the offset.
        ///
        /// Panics if there are not enough bytes remaining.
        pub fn get(&mut self, length: u32) -> Vec<u8> {
            let end_index: usize = (self.offset + length) as usize;

            if end_index <= self.buffer.len() {
                let start_index: usize = self.offset as usize;
                self.offset += length;
                self.buffer[start_index..end_index].to_vec()
            } else {
                panic!(
                    "The specified range is invalid. Offset: {}, Length: {}, Buffer size: {}, Trying to read until: {}",
                    self.offset, length, self.buffer.len(), end_index
                );
            }
        }

        /// Returns the remaining unread bytes.
        ///
        /// Panics if the stream is already at EOF.
        pub fn get_remaining(&self) -> Vec<u8> {
            if self.offset >= self.buffer.len() as u32 {
                panic!("Error get_remaining(): No bytes left to read");
            } else {
                self.buffer[(self.offset as usize)..].to_vec()
            }
        }

        /// Writes a raw byte vector to the buffer.
        #[inline]
        pub fn put(&mut self, value: Vec<u8>) {
            self.buffer.extend(value);
        }

        /// Reads a single byte and returns `true` if it’s nonzero.
        #[inline]
        pub fn get_bool(&mut self) -> bool {
            let bytes = self.get(1);
            bytes[0] != 0
        }

        /// Writes a boolean value (`0x01` for true, `0x00` for false).
        #[inline]
        pub fn put_bool(&mut self, value: bool) {
            self.buffer.push(if value { 0x01 } else { 0x00 });
        }

        /// Reads a single unsigned byte.
        #[inline]
        pub fn get_byte(&mut self) -> u8 {
            self.get(1)[0]
        }

        /// Writes a single unsigned byte.
        #[inline]
        pub fn put_byte(&mut self, value: u8) {
            self.buffer.push(value);
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

        // ===== 24-bit (triad) integers =====

        /// Reads a 24-bit unsigned integer (big-endian).
        #[inline]
        pub fn get_u24_be(&mut self) -> u32 {
            let bytes = self.get(3);
            u32::from_be_bytes([0, bytes[0], bytes[1], bytes[2]])
        }

        /// Writes a 24-bit unsigned integer (big-endian).
        #[inline]
        pub fn put_u24_be(&mut self, value: u32) {
            let bytes = value.to_be_bytes();
            self.buffer.extend_from_slice(&bytes[1..4]);
        }

        /// Reads a 24-bit unsigned integer (little-endian).
        #[inline]
        pub fn get_u24_le(&mut self) -> u32 {
            let bytes = self.get(3);
            u32::from_le_bytes([bytes[0], bytes[1], bytes[2], 0])
        }

        /// Writes a 24-bit unsigned integer (little-endian).
        #[inline]
        pub fn put_u24_le(&mut self, value: u32) {
            let bytes = value.to_le_bytes();
            self.buffer.extend_from_slice(&bytes[0..3]);
        }

        // ===== 32-bit integers =====

        /// Reads an unsigned 32-bit integer (big-endian).
        #[inline]
        pub fn get_u32_be(&mut self) -> u32 {
            let bytes = self.get(4);
            u32::from_be_bytes(bytes.try_into().unwrap())
        }

        /// Writes an unsigned 32-bit integer (big-endian).
        #[inline]
        pub fn put_u32_be(&mut self, value: u32) {
            self.buffer.extend_from_slice(&value.to_be_bytes());
        }

        /// Reads an unsigned 32-bit integer (little-endian).
        #[inline]
        pub fn get_u32_le(&mut self) -> u32 {
            let bytes = self.get(4);
            u32::from_le_bytes(bytes.try_into().unwrap())
        }

        /// Writes an unsigned 32-bit integer (little-endian).
        #[inline]
        pub fn put_u32_le(&mut self, value: u32) {
            self.buffer.extend_from_slice(&value.to_le_bytes());
        }

        /// Reads a signed 32-bit integer (big-endian).
        #[inline]
        pub fn get_i32_be(&mut self) -> i32 {
            let bytes = self.get(4);
            i32::from_be_bytes(bytes.try_into().unwrap())
        }

        /// Writes a signed 32-bit integer (big-endian).
        #[inline]
        pub fn put_i32_be(&mut self, value: i32) {
            self.buffer.extend_from_slice(&value.to_be_bytes());
        }

        /// Reads a signed 32-bit integer (little-endian).
        #[inline]
        pub fn get_i32_le(&mut self) -> i32 {
            let bytes = self.get(4);
            i32::from_le_bytes(bytes.try_into().unwrap())
        }

        /// Writes a signed 32-bit integer (little-endian).
        #[inline]
        pub fn put_i32_le(&mut self, value: i32) {
            self.buffer.extend_from_slice(&value.to_le_bytes());
        }

        // ===== 32-bit floats =====

        /// Reads a 32-bit floating-point number (big-endian).
        #[inline]
        pub fn get_f32_be(&mut self) -> f32 {
            let bytes = self.get(4);
            f32::from_be_bytes(bytes.try_into().unwrap())
        }

        /// Writes a 32-bit floating-point number (big-endian).
        #[inline]
        pub fn put_f32_be(&mut self, value: f32) {
            self.buffer.extend_from_slice(&value.to_be_bytes());
        }

        /// Reads a 32-bit floating-point number (little-endian).
        #[inline]
        pub fn get_f32_le(&mut self) -> f32 {
            let bytes = self.get(4);
            f32::from_le_bytes(bytes.try_into().unwrap())
        }

        /// Writes a 32-bit floating-point number (little-endian).
        #[inline]
        pub fn put_f32_le(&mut self, value: f32) {
            self.buffer.extend_from_slice(&value.to_le_bytes());
        }

        // ===== 64-bit floats =====

        /// Reads a 64-bit double-precision float (big-endian).
        #[inline]
        pub fn get_f64_be(&mut self) -> f64 {
            let bytes = self.get(8);
            f64::from_be_bytes(bytes.try_into().unwrap())
        }

        /// Writes a 64-bit double-precision float (big-endian).
        #[inline]
        pub fn put_f64_be(&mut self, value: f64) {
            self.buffer.extend_from_slice(&value.to_be_bytes());
        }

        /// Reads a 64-bit double-precision float (little-endian).
        #[inline]
        pub fn get_f64_le(&mut self) -> f64 {
            let bytes = self.get(8);
            f64::from_le_bytes(bytes.try_into().unwrap())
        }

        /// Writes a 64-bit double-precision float (little-endian).
        #[inline]
        pub fn put_f64_le(&mut self, value: f64) {
            self.buffer.extend_from_slice(&value.to_le_bytes());
        }

        // ===== 64-bit integers =====

        /// Reads a signed 64-bit integer (big-endian).
        #[inline]
        pub fn get_i64_be(&mut self) -> i64 {
            let bytes = self.get(8);
            i64::from_be_bytes(bytes.try_into().unwrap())
        }

        /// Writes a signed 64-bit integer (big-endian).
        #[inline]
        pub fn put_i64_be(&mut self, value: i64) {
            self.buffer.extend_from_slice(&value.to_be_bytes());
        }

        /// Reads a signed 64-bit integer (little-endian).
        #[inline]
        pub fn get_i64_le(&mut self) -> i64 {
            let bytes = self.get(8);
            i64::from_le_bytes(bytes.try_into().unwrap())
        }

        /// Writes a signed 64-bit integer (little-endian).
        #[inline]
        pub fn put_i64_le(&mut self, value: i64) {
            self.buffer.extend_from_slice(&value.to_le_bytes());
        }

        /// Reads an unsigned 64-bit integer (big-endian).
        #[inline]
        pub fn get_u64_be(&mut self) -> u64 {
            let bytes = self.get(8);
            u64::from_be_bytes(bytes.try_into().unwrap())
        }

        /// Writes an unsigned 64-bit integer (big-endian).
        #[inline]
        pub fn put_u64_be(&mut self, value: u64) {
            self.buffer.extend_from_slice(&value.to_be_bytes());
        }

        /// Reads an unsigned 64-bit integer (little-endian).
        #[inline]
        pub fn get_u64_le(&mut self) -> u64 {
            let bytes = self.get(8);
            u64::from_le_bytes(bytes.try_into().unwrap())
        }

        /// Writes an unsigned 64-bit integer (little-endian).
        #[inline]
        pub fn put_u64_le(&mut self, value: u64) {
            self.buffer.extend_from_slice(&value.to_le_bytes());
        }

        // ===== Variable-length integers =====

        /// Reads an unsigned variable-length integer (VarInt).
        pub fn get_var_u32(&mut self) -> u32 {
            let mut value = 0u32;
            for i in 0..5 {
                let b = self.get_byte();
                value |= ((b & 0x7f) as u32) << (i * 7);
                if b & 0x80 == 0 {
                    return value;
                }
            }
            panic!("VarInt did not terminate after 5 bytes!");
        }

        /// Writes an unsigned variable-length integer (VarInt).
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

        /// Writes a signed variable-length integer (ZigZag encoded).
        #[inline]
        pub fn put_var_i32(&mut self, value: i32) {
            let mut encoded = (value << 1) as u32;
            if value < 0 {
                encoded = !encoded;
            }
            self.put_var_u32(encoded);
        }

        /// Reads an unsigned 64-bit variable-length integer (VarLong).
        pub fn get_var_u64(&mut self) -> u64 {
            let mut value = 0u64;
            for i in 0..10 {
                let b = self.get_byte();
                value |= ((b & 0x7f) as u64) << (i * 7);
                if b & 0x80 == 0 {
                    return value;
                }
            }
            panic!("VarLong did not terminate after 10 bytes!");
        }

        /// Writes an unsigned 64-bit variable-length integer (VarLong).
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

        /// Writes a signed 64-bit variable-length integer (ZigZag encoded).
        #[inline]
        pub fn put_var_i64(&mut self, value: i64) {
            let mut encoded = (value << 1) as u64;
            if value < 0 {
                encoded = !encoded;
            }
            self.put_var_u64(encoded);
        }
    }
}
