// Freesat/Freeview Huffman decoder using packed binary tries.
//
// The trie data is generated at build time from resources/huffman_table{1,2}.csv
// by build.rs.  Each trie node is a u16:
//   - Leaf:     LEAF_FLAG | char_value  (bit 15 set, bits 0–7 = output byte)
//   - Internal: left-child index; right child is at index + 1

include!(concat!(env!("OUT_DIR"), "/huffman_tries.rs"));

const START: u8 = 0;
const STOP: u8 = 0;
const ESCAPE: u8 = 1;

/// Decode Freesat/Freeview Huffman-compressed text.
///
/// The `encoding_type_id` selects the Huffman table (1 or 2).
/// `data` should be the raw compressed bytes *after* the encoding_type_id byte.
///
/// Returns decompressed bytes in ISO 8859-1 (Latin-1) encoding.
pub(crate) fn decode(encoding_type_id: u8, data: &[u8]) -> Option<Vec<u8>> {
    let (trie, roots) = match encoding_type_id {
        1 => (TRIE_1, TRIE_ROOTS_1),
        2 => (TRIE_2, TRIE_ROOTS_2),
        _ => return None,
    };

    let mut out = Vec::new();
    let mut byte_pos: usize = 0;
    let mut bit_pos: u8 = 0; // 0..8 within current byte
    let mut last_ch = START;

    loop {
        let ch = if last_ch == ESCAPE {
            // Escape: next 8 bits are a literal character
            let literal = read_bits(data, &mut byte_pos, &mut bit_pos, 8)? as u8;
            if literal & 0x80 == 0 {
                last_ch = literal;
            }
            // Suppress non-printable control chars (except newline) from output,
            // but still use the value for state tracking above.
            if literal < 0x20 && literal != b'\n' {
                if last_ch == STOP {
                    break;
                }
                continue;
            }
            literal
        } else {
            // Walk the trie for this state
            let root = roots[last_ch as usize];
            if root == u16::MAX {
                break;
            }
            let mut node = root as usize;
            loop {
                let bit = read_bits(data, &mut byte_pos, &mut bit_pos, 1)?;
                let entry = trie[node + bit as usize];
                if entry & LEAF_FLAG != 0 {
                    let ch = (entry & !LEAF_FLAG) as u8;
                    last_ch = ch;
                    break ch;
                }
                node = entry as usize;
            }
        };

        if ch == STOP {
            break;
        }
        if ch != ESCAPE {
            out.push(ch);
        }
    }

    Some(out)
}

/// Read `count` bits (1–8) from `data` starting at the current byte/bit position.
/// Returns `None` if there are not enough bits remaining.
fn read_bits(data: &[u8], byte_pos: &mut usize, bit_pos: &mut u8, count: u8) -> Option<u32> {
    let mut value: u32 = 0;
    for _ in 0..count {
        if *byte_pos >= data.len() {
            return None;
        }
        value = (value << 1) | ((data[*byte_pos] >> (7 - *bit_pos)) & 1) as u32;
        *bit_pos += 1;
        if *bit_pos == 8 {
            *bit_pos = 0;
            *byte_pos += 1;
        }
    }
    Some(value)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn unknown_table() {
        assert_eq!(decode(3, &[0xFF, 0xFF, 0xFF, 0xFF]), None);
    }

    #[test]
    fn decode_table_1() {
        assert_eq!(
            decode(
                1,
                &[
                    0xC7, 0x0B, 0x5F, 0xED, 0x6C, 0xD0, 0xEF, 0x99, 0xF3, 0x5F, 0x8B, 0x82, 0x13,
                    0x98
                ]
            )
            .as_deref(),
            Some(b"Formula Drift Series 2017".as_slice()),
        );
    }

    #[test]
    fn decode_table_2() {
        assert_eq!(
            decode(2, &[
                0x69, 0x36, 0xE0, 0x7B, 0x8B, 0xD7, 0x7D, 0x2D, 0x7C, 0x9B, 0x57, 0xC0, 0x50,
                0xBF, 0x24, 0x78, 0xCF, 0xF0, 0xFB, 0xB3, 0xEC, 0xD9, 0x42, 0xCC, 0x9D, 0xF0,
                0x79, 0xA3, 0xBD, 0xA4, 0x2E, 0x52, 0xE3, 0x9F, 0x7A, 0xEF, 0x3D, 0x71, 0x53,
                0x5B, 0xF1, 0x4C, 0xBB, 0x98, 0x5E, 0x33, 0xA3, 0xB0, 0xA6, 0x0B, 0xBC, 0xFB,
                0xE9, 0x6B, 0xE4, 0xDA, 0xB5, 0x7A, 0xCB, 0xF9, 0x1D, 0x9B, 0x66, 0x9F, 0x66,
                0xC5, 0x08,
            ]).as_deref(),
            Some(b"For luxury jewellery without the expensive price tag, join TJC as we present a stunning selection of opulent jewellery at affordable prices.".as_slice()),
        );
    }
}
