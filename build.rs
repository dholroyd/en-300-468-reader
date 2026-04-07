//! Build script: reads Huffman table CSVs and generates packed binary tries
//! for Freesat/Freeview text decoding.
//!
//! Each trie node is a `u16`:
//! - Leaf:     `0x8000 | char_value`  (bit 15 set, bits 0–7 = output byte)
//! - Internal: index of the left child; right child is at index + 1
//!
//! Per-state root offsets are stored in a separate array.

use std::collections::HashMap;
use std::env;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

const LEAF_FLAG: u16 = 0x8000;
const NUM_STATES: usize = 128;

struct TrieBuilder {
    /// Flat array of u16 nodes.
    nodes: Vec<u16>,
}

impl TrieBuilder {
    fn new() -> Self {
        TrieBuilder { nodes: Vec::new() }
    }

    fn alloc_internal(&mut self) -> usize {
        let idx = self.nodes.len();
        // left child slot, right child slot
        self.nodes.push(0);
        self.nodes.push(0);
        idx
    }

    /// Insert a code into the trie rooted at `root`.
    /// `bits` is the bit pattern left-aligned in a u32, `num_bits` is the code length.
    fn insert(&mut self, root: usize, bits: u32, num_bits: u8, ch: u8) {
        let mut node = root;
        for i in 0..num_bits {
            let bit = (bits >> (31 - i)) & 1;
            let child_slot = node + bit as usize;
            let child = self.nodes[child_slot];
            if child == 0 {
                if i == num_bits - 1 {
                    // Final bit: place leaf
                    self.nodes[child_slot] = LEAF_FLAG | ch as u16;
                } else {
                    // Allocate internal node
                    let new_node = self.alloc_internal();
                    self.nodes[child_slot] = new_node as u16;
                    node = new_node;
                }
            } else if child & LEAF_FLAG != 0 {
                panic!(
                    "Conflict: inserting {}-bit code for char {} but slot is already a leaf (char {})",
                    num_bits,
                    ch,
                    child & !LEAF_FLAG
                );
            } else {
                // Existing internal node
                node = child as usize;
            }
        }
    }

    /// Build tries for all states from parsed CSV entries.
    /// Returns (nodes, state_roots) where state_roots[i] is the root index for state i,
    /// or u16::MAX if the state has no entries.
    fn build(entries: &[(u8, u32, u8, u8)]) -> (Vec<u16>, Vec<u16>) {
        let mut builder = TrieBuilder::new();
        // Group entries by state
        let mut by_state: HashMap<u8, Vec<(u32, u8, u8)>> = HashMap::new();
        for &(state, bits, num_bits, ch) in entries {
            by_state
                .entry(state)
                .or_default()
                .push((bits, num_bits, ch));
        }

        let mut state_roots = vec![u16::MAX; NUM_STATES];
        for state in 0..NUM_STATES as u8 {
            if let Some(codes) = by_state.get(&state) {
                let root = builder.alloc_internal();
                state_roots[state as usize] = root as u16;
                for &(bits, num_bits, ch) in codes {
                    builder.insert(root, bits, num_bits, ch);
                }
            }
        }
        (builder.nodes, state_roots)
    }
}

fn parse_csv(path: &Path) -> Vec<(u8, u32, u8, u8)> {
    let file =
        fs::File::open(path).unwrap_or_else(|e| panic!("Cannot open {}: {}", path.display(), e));
    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    for (line_num, line) in reader.lines().enumerate() {
        let line = line.unwrap();
        if line_num == 0 {
            continue; // skip header
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();
        assert_eq!(
            fields.len(),
            4,
            "{}:{}: expected 4 fields, got {}",
            path.display(),
            line_num + 1,
            fields.len()
        );
        let state: u8 = fields[0].parse().unwrap();
        let bits: u32 = u32::from_str_radix(fields[1].trim_start_matches("0x"), 16).unwrap();
        let num_bits: u8 = fields[2].parse().unwrap();
        let ch: u8 = fields[3].parse().unwrap();
        entries.push((state, bits, num_bits, ch));
    }
    entries
}

fn write_array_u16(out: &mut String, name: &str, data: &[u16]) {
    writeln!(out, "static {name}: &[u16] = &[").unwrap();
    for (i, chunk) in data.chunks(16).enumerate() {
        write!(out, "    ").unwrap();
        for (j, &val) in chunk.iter().enumerate() {
            if i * 16 + j + 1 < data.len() {
                write!(out, "{val},").unwrap();
            } else {
                write!(out, "{val}").unwrap();
            }
        }
        writeln!(out).unwrap();
    }
    writeln!(out, "];").unwrap();
}

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let src = Path::new(&manifest_dir).join("src");

    println!("cargo:rerun-if-changed=src/huffman_table1.csv");
    println!("cargo:rerun-if-changed=src/huffman_table2.csv");

    let entries1 = parse_csv(&src.join("huffman_table1.csv"));
    let entries2 = parse_csv(&src.join("huffman_table2.csv"));

    let (nodes1, roots1) = TrieBuilder::build(&entries1);
    let (nodes2, roots2) = TrieBuilder::build(&entries2);

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let out_path = Path::new(&out_dir).join("huffman_tries.rs");

    let mut code = String::new();
    writeln!(code, "// Generated by build.rs - do not edit").unwrap();
    writeln!(code).unwrap();
    writeln!(
        code,
        "/// Bit 15 set indicates a leaf node; bits 0–7 hold the output byte."
    )
    .unwrap();
    writeln!(code, "const LEAF_FLAG: u16 = 0x8000;").unwrap();
    writeln!(code).unwrap();

    write_array_u16(&mut code, "TRIE_1", &nodes1);
    writeln!(code).unwrap();
    write_array_u16(&mut code, "TRIE_ROOTS_1", &roots1);
    writeln!(code).unwrap();
    write_array_u16(&mut code, "TRIE_2", &nodes2);
    writeln!(code).unwrap();
    write_array_u16(&mut code, "TRIE_ROOTS_2", &roots2);

    fs::write(&out_path, code).unwrap();
}
