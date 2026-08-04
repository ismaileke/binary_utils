//! Integration tests for `binary_utils`.
//!
//! Save as `tests/binary.rs`. Everything under `tests/` is already compiled
//! test-only and links the crate the way a downstream user would, so no
//! `#[cfg(test)]` or wrapper `mod tests` is needed.
//!
//! Layers:
//!
//! 1. **Wire format** — exact bytes on the wire. Catches a symmetric bug where
//!    `Writer` and `Reader` agree with each other but disagree with the
//!    Bedrock protocol.
//! 2. **Round-trip** — put-then-get for every type, at boundary values.
//! 3. **Panic behaviour** — this API signals failure by panicking, so the
//!    panics are part of the contract and get tested like anything else.
//! 4. **Cursor & zero-copy** — offset mechanics and proof that no copies happen.

use binary_utils::binary::{Reader, Writer};

// =============================================================================
// 1. Wire format — exact byte output
// =============================================================================

mod wire_format {
    use super::*;

    #[test]
    fn endianness_is_not_swapped() {
        let mut w = Writer::new();
        w.put_u32_be(0x0A0B_0C0D);
        assert_eq!(w.as_slice(), &[0x0A, 0x0B, 0x0C, 0x0D]);

        w.clear();
        w.put_u32_le(0x0A0B_0C0D);
        assert_eq!(w.as_slice(), &[0x0D, 0x0C, 0x0B, 0x0A]);

        w.clear();
        w.put_u16_be(0xAABB);
        assert_eq!(w.as_slice(), &[0xAA, 0xBB]);

        w.clear();
        w.put_u16_le(0xAABB);
        assert_eq!(w.as_slice(), &[0xBB, 0xAA]);

        w.clear();
        w.put_u64_be(0x0102_0304_0506_0708);
        assert_eq!(w.as_slice(), &[1, 2, 3, 4, 5, 6, 7, 8]);

        w.clear();
        w.put_u64_le(0x0102_0304_0506_0708);
        assert_eq!(w.as_slice(), &[8, 7, 6, 5, 4, 3, 2, 1]);
    }

    #[test]
    fn signed_writers_match_unsigned_layout() {
        let mut a = Writer::new();
        let mut b = Writer::new();
        a.put_i32_be(-2);
        b.put_u32_be(0xFFFF_FFFE);
        assert_eq!(a.as_slice(), b.as_slice());

        a.clear();
        b.clear();
        a.put_i64_le(-2);
        b.put_u64_le(0xFFFF_FFFF_FFFF_FFFE);
        assert_eq!(a.as_slice(), b.as_slice());
    }

    #[test]
    fn triads_drop_the_correct_byte() {
        // A triad is 3 bytes; BE must drop the most significant byte,
        // LE must drop the highest position of the 4-byte layout.
        let mut w = Writer::new();
        w.put_u24_be(0x00AB_CDEF);
        assert_eq!(w.as_slice(), &[0xAB, 0xCD, 0xEF]);

        w.clear();
        w.put_u24_le(0x00AB_CDEF);
        assert_eq!(w.as_slice(), &[0xEF, 0xCD, 0xAB]);
    }

    #[test]
    fn varint_encoding_matches_leb128() {
        let cases: &[(u32, &[u8])] = &[
            (0, &[0x00]),
            (1, &[0x01]),
            (127, &[0x7F]),
            (128, &[0x80, 0x01]),
            (255, &[0xFF, 0x01]),
            (300, &[0xAC, 0x02]),
            (16_383, &[0xFF, 0x7F]),
            (16_384, &[0x80, 0x80, 0x01]),
            (u32::MAX, &[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]),
        ];

        for &(value, expected) in cases {
            let mut w = Writer::new();
            w.put_var_u32(value);
            assert_eq!(w.as_slice(), expected, "encoding {value}");
            assert_eq!(
                Reader::new(expected).get_var_u32(),
                value,
                "decoding {value}"
            );
        }
    }

    #[test]
    fn varlong_encoding_matches_leb128() {
        let cases: &[(u64, &[u8])] = &[
            (0, &[0x00]),
            (127, &[0x7F]),
            (128, &[0x80, 0x01]),
            (
                u64::MAX,
                &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01],
            ),
        ];

        for &(value, expected) in cases {
            let mut w = Writer::new();
            w.put_var_u64(value);
            assert_eq!(w.as_slice(), expected, "encoding {value}");
            assert_eq!(Reader::new(expected).get_var_u64(), value);
        }
    }

    #[test]
    fn varint_length_grows_at_the_right_boundaries() {
        let boundaries: &[(u32, usize)] = &[
            (0x0000_007F, 1),
            (0x0000_0080, 2),
            (0x0000_3FFF, 2),
            (0x0000_4000, 3),
            (0x001F_FFFF, 3),
            (0x0020_0000, 4),
            (0x0FFF_FFFF, 4),
            (0x1000_0000, 5),
            (0xFFFF_FFFF, 5),
        ];
        for &(value, expected_len) in boundaries {
            let mut w = Writer::new();
            w.put_var_u32(value);
            assert_eq!(w.len(), expected_len, "byte length of varint {value:#X}");
        }
    }

    #[test]
    fn zigzag_mapping_is_correct() {
        // ZigZag: 0->0, -1->1, 1->2, -2->3, 2->4, -3->5 ...
        let cases: &[(i32, u32)] = &[
            (0, 0),
            (-1, 1),
            (1, 2),
            (-2, 3),
            (2, 4),
            (-64, 127),
            (i32::MAX, 0xFFFF_FFFE),
            (i32::MIN, 0xFFFF_FFFF),
        ];

        for &(signed, expected_raw) in cases {
            let mut w = Writer::new();
            w.put_var_i32(signed);
            // Read back as unsigned to inspect the raw zigzag value.
            let raw = Reader::new(w.as_slice()).get_var_u32();
            assert_eq!(raw, expected_raw, "zigzag({signed})");
        }
    }

    #[test]
    fn zigzag64_mapping_is_correct() {
        let cases: &[(i64, u64)] = &[
            (0, 0),
            (-1, 1),
            (1, 2),
            (i64::MAX, 0xFFFF_FFFF_FFFF_FFFE),
            (i64::MIN, 0xFFFF_FFFF_FFFF_FFFF),
        ];

        for &(signed, expected_raw) in cases {
            let mut w = Writer::new();
            w.put_var_i64(signed);
            let raw = Reader::new(w.as_slice()).get_var_u64();
            assert_eq!(raw, expected_raw, "zigzag64({signed})");
        }
    }

    #[test]
    fn zigzag_keeps_small_negatives_small() {
        // The whole point of zigzag: -1 must cost 1 byte, not 5.
        let mut w = Writer::new();
        w.put_var_i32(-1);
        assert_eq!(w.len(), 1);

        w.clear();
        w.put_var_i64(-1);
        assert_eq!(w.len(), 1);
    }

    #[test]
    fn bool_writes_canonical_bytes() {
        let mut w = Writer::new();
        w.put_bool(true);
        w.put_bool(false);
        assert_eq!(w.as_slice(), &[0x01, 0x00]);

        // Any nonzero byte reads back as true.
        let mut r = Reader::new(&[0x00, 0x01, 0x02, 0xFF]);
        assert!(!r.get_bool());
        assert!(r.get_bool());
        assert!(r.get_bool());
        assert!(r.get_bool());
    }

    #[test]
    fn float_bit_layout_is_ieee754() {
        let mut w = Writer::new();
        w.put_f32_be(1.0);
        assert_eq!(w.as_slice(), &[0x3F, 0x80, 0x00, 0x00]);

        w.clear();
        w.put_f64_be(1.0);
        assert_eq!(w.as_slice(), &[0x3F, 0xF0, 0, 0, 0, 0, 0, 0]);
    }
}

// =============================================================================
// 2. Round-trip
// =============================================================================

mod round_trip {
    use super::*;

    /// Generates a round-trip test over a list of boundary values.
    macro_rules! rt {
        ($name:ident, $put:ident, $get:ident, $ty:ty, [$($v:expr),* $(,)?]) => {
            #[test]
            fn $name() {
                for value in [$($v as $ty),*] {
                    let mut w = Writer::new();
                    w.$put(value);
                    let mut r = Reader::new(w.as_slice());
                    assert_eq!(r.$get(), value, "round-trip of {value}");
                    assert!(r.is_empty(), "reader left bytes behind for {value}");
                }
            }
        };
    }

    rt!(u8_rt, put_u8, get_u8, u8, [0, 1, 127, 128, 255]);
    rt!(i8_rt, put_i8, get_i8, i8, [0, 1, -1, i8::MIN, i8::MAX]);

    rt!(u16_be_rt, put_u16_be, get_u16_be, u16, [0, 1, 255, 256, u16::MAX]);
    rt!(u16_le_rt, put_u16_le, get_u16_le, u16, [0, 1, 255, 256, u16::MAX]);
    rt!(i16_be_rt, put_i16_be, get_i16_be, i16, [0, -1, i16::MIN, i16::MAX]);
    rt!(i16_le_rt, put_i16_le, get_i16_le, i16, [0, -1, i16::MIN, i16::MAX]);

    rt!(u32_be_rt, put_u32_be, get_u32_be, u32, [0, 1, u32::MAX]);
    rt!(u32_le_rt, put_u32_le, get_u32_le, u32, [0, 1, u32::MAX]);
    rt!(i32_be_rt, put_i32_be, get_i32_be, i32, [0, -1, i32::MIN, i32::MAX]);
    rt!(i32_le_rt, put_i32_le, get_i32_le, i32, [0, -1, i32::MIN, i32::MAX]);

    rt!(u64_be_rt, put_u64_be, get_u64_be, u64, [0, 1, u64::MAX]);
    rt!(u64_le_rt, put_u64_le, get_u64_le, u64, [0, 1, u64::MAX]);
    rt!(i64_be_rt, put_i64_be, get_i64_be, i64, [0, -1, i64::MIN, i64::MAX]);
    rt!(i64_le_rt, put_i64_le, get_i64_le, i64, [0, -1, i64::MIN, i64::MAX]);

    rt!(u24_be_rt, put_u24_be, get_u24_be, u32, [0, 1, 0xFF, 0xFFFF, 0xFF_FFFF]);
    rt!(u24_le_rt, put_u24_le, get_u24_le, u32, [0, 1, 0xFF, 0xFFFF, 0xFF_FFFF]);

    rt!(var_u32_rt, put_var_u32, get_var_u32, u32,
        [0, 1, 127, 128, 16_383, 16_384, 2_097_151, 2_097_152, u32::MAX]);
    rt!(var_i32_rt, put_var_i32, get_var_i32, i32,
        [0, 1, -1, 63, -64, 8192, -8192, i32::MIN, i32::MAX]);
    rt!(var_u64_rt, put_var_u64, get_var_u64, u64,
        [0, 1, 127, 128, u32::MAX as u64, u64::MAX]);
    rt!(var_i64_rt, put_var_i64, get_var_i64, i64,
        [0, 1, -1, i32::MIN as i64, i64::MIN, i64::MAX]);

    #[test]
    fn bool_rt() {
        for value in [true, false] {
            let mut w = Writer::new();
            w.put_bool(value);
            assert_eq!(Reader::new(w.as_slice()).get_bool(), value);
        }
    }

    #[test]
    fn floats_round_trip_bit_exactly() {
        for v in [0.0f32, -0.0, 1.0, -1.5, f32::MIN, f32::MAX, f32::EPSILON] {
            let mut w = Writer::new();
            w.put_f32_be(v);
            // Compare bit patterns so -0.0 and 0.0 stay distinguishable.
            assert_eq!(Reader::new(w.as_slice()).get_f32_be().to_bits(), v.to_bits());

            w.clear();
            w.put_f32_le(v);
            assert_eq!(Reader::new(w.as_slice()).get_f32_le().to_bits(), v.to_bits());
        }

        for v in [0.0f64, -0.0, 1.0, -1.5, f64::MIN, f64::MAX] {
            let mut w = Writer::new();
            w.put_f64_be(v);
            assert_eq!(Reader::new(w.as_slice()).get_f64_be().to_bits(), v.to_bits());

            w.clear();
            w.put_f64_le(v);
            assert_eq!(Reader::new(w.as_slice()).get_f64_le().to_bits(), v.to_bits());
        }
    }

    #[test]
    fn nan_and_infinity_survive() {
        let mut w = Writer::new();
        w.put_f32_le(f32::NAN);
        w.put_f32_le(f32::INFINITY);
        w.put_f32_le(f32::NEG_INFINITY);

        let mut r = Reader::new(w.as_slice());
        assert!(r.get_f32_le().is_nan());
        assert_eq!(r.get_f32_le(), f32::INFINITY);
        assert_eq!(r.get_f32_le(), f32::NEG_INFINITY);
    }

    #[test]
    fn raw_slice_round_trip() {
        let mut w = Writer::new();
        w.put(b"raknet");
        let buf = w.into_vec();
        assert_eq!(Reader::new(&buf).remaining(), b"raknet");
    }

    #[test]
    fn mixed_packet_round_trip() {
        // Approximates a real Bedrock packet body.
        let mut w = Writer::new();
        w.put_var_u32(0x8F); // packet id
        w.put_var_i32(-1024); // runtime entity id delta
        w.put_f32_le(128.5); // x
        w.put_f32_le(64.0); // y
        w.put_f32_le(-77.25); // z
        w.put_bool(true);
        w.put_u64_le(0xDEAD_BEEF_CAFE_0001);
        w.put_u24_le(0x00FF_00FF & 0xFF_FFFF);
        w.put(&[1, 2, 3, 4]);

        let buf = w.into_vec();
        let mut r = Reader::new(&buf);
        assert_eq!(r.get_var_u32(), 0x8F);
        assert_eq!(r.get_var_i32(), -1024);
        assert_eq!(r.get_f32_le(), 128.5);
        assert_eq!(r.get_f32_le(), 64.0);
        assert_eq!(r.get_f32_le(), -77.25);
        assert!(r.get_bool());
        assert_eq!(r.get_u64_le(), 0xDEAD_BEEF_CAFE_0001);
        assert_eq!(r.get_u24_le(), 0xFF_00FF);
        assert_eq!(r.remaining(), &[1, 2, 3, 4]);
    }

    #[test]
    fn deterministic_fuzz_round_trip() {
        // Simple LCG so this stays dependency-free and reproducible.
        let mut state: u64 = 0x2545_F491_4F6C_DD1D;
        let mut next = move || {
            state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            state
        };

        let mut w = Writer::with_capacity(4096);
        for _ in 0..2000 {
            let a = next() as u32;
            let b = next() as i32;
            let c = next();
            let d = next() as i64;
            let e = next() as u16;

            w.clear();
            w.put_var_u32(a);
            w.put_var_i32(b);
            w.put_var_u64(c);
            w.put_var_i64(d);
            w.put_u16_be(e);

            let mut r = w.reader();
            assert_eq!(r.get_var_u32(), a);
            assert_eq!(r.get_var_i32(), b);
            assert_eq!(r.get_var_u64(), c);
            assert_eq!(r.get_var_i64(), d);
            assert_eq!(r.get_u16_be(), e);
            assert!(r.is_empty());
        }
    }
}

// =============================================================================
// 3. Panic behaviour on malformed input
//
// This API reports failure by panicking, so the panics are part of the
// contract. These tests pin that contract down: a truncated packet must abort
// the read rather than return silent garbage.
// =============================================================================

mod panics_on_bad_input {
    use super::*;

    #[test]
    #[should_panic(expected = "Buffer underflow")]
    fn empty_buffer_u8() {
        Reader::new(&[]).get_u8();
    }

    #[test]
    #[should_panic(expected = "Buffer underflow")]
    fn truncated_u16() {
        Reader::new(&[0x01]).get_u16_be();
    }

    #[test]
    #[should_panic(expected = "Buffer underflow")]
    fn truncated_u24() {
        Reader::new(&[0x01, 0x02]).get_u24_be();
    }

    #[test]
    #[should_panic(expected = "Buffer underflow")]
    fn truncated_u32() {
        Reader::new(&[0x01, 0x02, 0x03]).get_u32_be();
    }

    #[test]
    #[should_panic(expected = "Buffer underflow")]
    fn truncated_u64() {
        Reader::new(&[0u8; 7]).get_u64_le();
    }

    #[test]
    #[should_panic(expected = "Buffer underflow")]
    fn truncated_f64() {
        Reader::new(&[0u8; 7]).get_f64_le();
    }

    #[test]
    #[should_panic(expected = "Buffer underflow")]
    fn reading_one_byte_past_the_end() {
        let mut r = Reader::new(&[0xAA, 0xBB]);
        r.get_u16_be();
        r.get_u8();
    }

    #[test]
    #[should_panic(expected = "Buffer underflow")]
    fn varint_truncated_mid_sequence() {
        // Continuation bit set on every byte, then the buffer ends.
        Reader::new(&[0x80, 0x80]).get_var_u32();
    }

    #[test]
    #[should_panic(expected = "VarInt overflows u32")]
    fn varint_never_terminates() {
        Reader::new(&[0x80, 0x80, 0x80, 0x80, 0x80, 0x80]).get_var_u32();
    }

    #[test]
    #[should_panic(expected = "VarLong overflows u64")]
    fn varlong_never_terminates() {
        Reader::new(&[0x80; 12]).get_var_u64();
    }

    #[test]
    #[should_panic(expected = "Buffer underflow")]
    fn varlong_truncated_mid_sequence() {
        Reader::new(&[0x80; 6]).get_var_u64();
    }

    // ---- Known gaps -------------------------------------------------------
    // These document behaviour that is currently wrong. Remove `#[ignore]`
    // once the library is fixed; they should then pass.

    #[test]
    #[should_panic(expected = "VarInt overflows u32")]
    fn varint_overflowing_u32_is_rejected() {
        // 5. byte bit 31'in üstünde bit taşıyor: u32'ye sığmaz.
        Reader::new(&[0xFF, 0xFF, 0xFF, 0xFF, 0x7F]).get_var_u32();
    }

    #[test]
    #[should_panic(expected = "VarLong overflows u64")]
    fn varlong_overflowing_u64_is_rejected() {
        Reader::new(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F])
            .get_var_u64();
    }

    #[test]
    fn maximum_legal_varint_is_still_accepted() {
        // Whatever overflow checking gets added must not reject this.
        assert_eq!(
            Reader::new(&[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]).get_var_u32(),
            u32::MAX
        );
    }

    #[test]
    fn maximum_legal_varlong_is_still_accepted() {
        assert_eq!(
            Reader::new(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01])
                .get_var_u64(),
            u64::MAX
        );
    }
}

// =============================================================================
// 4. Cursor mechanics & zero-copy guarantees
// =============================================================================

mod cursor_and_zero_copy {
    use super::*;

    #[test]
    fn offset_tracking() {
        let mut r = Reader::new(&[0u8; 16]);
        assert_eq!(r.offset(), 0);
        assert_eq!(r.remaining_byte_count(), 16);

        r.get_u32_be();
        assert_eq!(r.offset(), 4);
        assert_eq!(r.remaining_byte_count(), 12);

        r.get_u64_be();
        assert_eq!(r.offset(), 12);
        assert_eq!(r.remaining_byte_count(), 4);

        r.rewind();
        assert_eq!(r.offset(), 0);
        assert_eq!(r.remaining_byte_count(), 16);
    }

    #[test]
    fn is_empty_and_feof_agree() {
        let mut r = Reader::new(&[1, 2]);
        assert!(!r.is_empty());
        assert!(!r.feof());
        r.get_u16_be();
        assert!(r.is_empty());
        assert!(r.feof());
        assert_eq!(r.remaining_byte_count(), 0);
    }

    #[test]
    fn new_reader_on_empty_buffer_is_immediately_empty() {
        let r = Reader::new(&[]);
        assert!(r.is_empty());
        assert!(r.feof());
        assert_eq!(r.remaining_byte_count(), 0);
        assert_eq!(r.remaining(), &[] as &[u8]);
    }

    #[test]
    fn get_buffer_returns_the_whole_buffer_regardless_of_offset() {
        let mut r = Reader::new(&[1, 2, 3, 4]);
        r.get_u16_be();
        assert_eq!(r.get_buffer(), &[1, 2, 3, 4], "must not be offset-relative");
        assert_eq!(r.remaining(), &[3, 4], "remaining() is offset-relative");
    }

    #[test]
    fn remaining_does_not_consume() {
        let mut r = Reader::new(&[1, 2, 3, 4]);
        r.get_u8();
        assert_eq!(r.remaining(), &[2, 3, 4]);
        assert_eq!(r.offset(), 1, "remaining() must not advance the cursor");
        assert_eq!(r.remaining(), &[2, 3, 4], "and must be repeatable");
    }

    #[test]
    fn remaining_borrows_from_the_original_buffer() {
        let buf: Vec<u8> = (0..32).collect();
        let mut r = Reader::new(&buf);
        r.get_u64_be();
        let s = r.remaining();
        assert_eq!(s.len(), 24);
        // Proves no copy was made: the slice points into `buf` itself.
        assert!(std::ptr::eq(s.as_ptr(), buf[8..].as_ptr()));
    }

    #[test]
    fn borrowed_data_outlives_the_reader() {
        let buf = b"bedrock".to_vec();
        let payload: &[u8] = {
            let r = Reader::new(&buf);
            r.remaining()
            // `r` is dropped here; `payload` still points into `buf`.
        };
        assert_eq!(payload, b"bedrock");
    }

    // ---- Known gap --------------------------------------------------------

    #[test]
    fn set_offset_accepts_valid_offsets() {
        let mut r = Reader::new(&[1, 2, 3, 4]);
        r.set_offset(2);
        assert_eq!(r.offset(), 2);
        assert_eq!(r.remaining(), &[3, 4]);

        // Seeking to exactly len() is legal: it means "fully consumed".
        r.set_offset(4);
        assert!(r.is_empty());

        // And seeking back is how you re-read a header.
        r.set_offset(0);
        assert_eq!(r.get_u32_be(), 0x0102_0304);
    }
}

// =============================================================================
// 5. Writer buffer management
// =============================================================================

mod writer_management {
    use super::*;

    #[test]
    fn clear_resets_length_but_keeps_capacity() {
        let mut w = Writer::with_capacity(1024);
        assert_eq!(w.len(), 0);
        assert!(w.is_empty());

        w.put(&[0u8; 512]);
        assert_eq!(w.len(), 512);
        assert!(!w.is_empty());

        w.clear();
        assert_eq!(w.len(), 0);
        assert!(w.is_empty());

        // Reusing across many packets must not grow the buffer.
        for _ in 0..1000 {
            w.clear();
            w.put(b"packet");
            w.put_var_i32(-42);
        }
        assert!(w.len() < 32, "clear() should not leak bytes between packets");
    }

    #[test]
    fn reserve_does_not_touch_contents() {
        let mut w = Writer::with_capacity(4);
        w.put_u8(1);
        w.reserve(4096);
        assert_eq!(w.as_slice(), &[1]);
        assert_eq!(w.len(), 1, "reserve() changes capacity, not length");
    }

    #[test]
    fn from_vec_appends_to_existing_data() {
        let mut w = Writer::from_vec(vec![0xFF, 0xFE]);
        w.put_u8(0x01);
        assert_eq!(w.into_vec(), vec![0xFF, 0xFE, 0x01]);
    }

    #[test]
    fn into_vec_preserves_everything_written() {
        let mut w = Writer::new();
        w.put_u32_be(0xDEAD_BEEF);
        assert_eq!(w.into_vec(), vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn reader_helper_sees_written_bytes() {
        let mut w = Writer::new();
        w.put_var_u32(300);
        w.put_u16_be(0xABCD);
        let mut r = w.reader();
        assert_eq!(r.get_var_u32(), 300);
        assert_eq!(r.get_u16_be(), 0xABCD);
        assert!(r.is_empty());
    }

    #[test]
    fn as_mut_slice_allows_backpatching_a_length_header() {
        // Common pattern: reserve space for a length, write the body, then
        // patch the header once the real length is known.
        let mut w = Writer::new();
        w.put_u32_be(0); // placeholder
        w.put(b"payload");
        let body_len = (w.len() - 4) as u32;
        w.as_mut_slice()[..4].copy_from_slice(&body_len.to_be_bytes());

        let mut r = w.reader();
        assert_eq!(r.get_u32_be(), 7);
        assert_eq!(r.remaining(), b"payload");
    }

    #[test]
    fn writes_append_rather_than_overwrite() {
        let mut w = Writer::new();
        w.put_u8(1);
        w.put_u8(2);
        w.put_u8(3);
        assert_eq!(w.as_slice(), &[1, 2, 3]);
    }
}