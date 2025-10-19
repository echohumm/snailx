extern crate core;

import! {
    mem::size_of
}

#[inline(always)]
pub fn is_valid_utf8(v: &[u8]) -> bool {
    const ASCII_BS: usize = 2 * size_of::<usize>();
    const ALIGN_MASK: usize = size_of::<usize>() - 1;

    let mut index = 0;
    let len = v.len();

    let ptr = v.as_ptr();
    let ptr_addr = ptr as usize;

    while index < len {
        // SAFETY: guarded by `index < len`
        let first = unsafe { *v.get_unchecked(index) };

        if first < 128 {
            // ASCII fast path: if aligned, scan two machine words per step.
            if ((ptr_addr + index) & ALIGN_MASK) == 0 {
                while index + ASCII_BS <= len {
                    // SAFETY: `ptr.add(index)` is aligned by the check above,
                    // and we only read inside `v` due to the `<= len` guard.
                    unsafe {
                        let block = ptr.add(index).cast();
                        if contains_nonascii(*block) || contains_nonascii(*block.add(1)) {
                            break;
                        }
                    }
                    index += ASCII_BS;
                }
                // Finish the trailing ASCII tail one byte at a time.
                while index < len {
                    // SAFETY: `index < len` here.
                    if unsafe { *v.get_unchecked(index) } < 128 {
                        index += 1;
                    } else {
                        break;
                    }
                }
            } else {
                index += 1;
            }
            continue;
        }

        // Non-ASCII: validate the full UTF-8 sequence without per-byte OOB checks.
        let w = utf8_char_width(first);
        if w == 0 || index + w > len {
            return false;
        }

        match w {
            2 => {
                // SAFETY: `index + w <= len` above guarantees in-bounds
                let b1 = unsafe { *v.get_unchecked(index + 1) };
                if !is_continuation(b1) {
                    return false;
                }
            }
            3 => {
                let b1 = unsafe { *v.get_unchecked(index + 1) };
                #[allow(clippy::unnested_or_patterns)]
                match (first, b1) {
                    (0xE0, 0xA0..=0xBF)
                    | (0xE1..=0xEC, 0x80..=0xBF)
                    | (0xED, 0x80..=0x9F)
                    | (0xEE..=0xEF, 0x80..=0xBF) => {}
                    _ => return false
                }
                let b2 = unsafe { *v.get_unchecked(index + 2) };
                if !is_continuation(b2) {
                    return false;
                }
            }
            4 => {
                let b1 = unsafe { *v.get_unchecked(index + 1) };
                match (first, b1) {
                    (0xF0, 0x90..=0xBF) | (0xF1..=0xF3, 0x80..=0xBF) | (0xF4, 0x80..=0x8F) => {}
                    _ => return false
                }
                let b2 = unsafe { *v.get_unchecked(index + 2) };
                if !is_continuation(b2) {
                    return false;
                }
                let b3 = unsafe { *v.get_unchecked(index + 3) };
                if !is_continuation(b3) {
                    return false;
                }
            }
            _ => return false
        }

        index += w;
    }

    true
}

#[must_use]
#[inline]
pub const fn utf8_char_width(b: u8) -> usize {
    UTF8_CHAR_WIDTH[b as usize] as usize
}

const NONASCII_MASK: usize = usize::from_ne_bytes([0x80; size_of::<usize>()]);

#[inline]
const fn contains_nonascii(x: usize) -> bool {
    (x & NONASCII_MASK) != 0
}

const CONT_BYTES: u8 = 0b1000_0000;
const CONT_MASK: u8 = 0b1100_0000;

#[inline]
const fn is_continuation(b: u8) -> bool {
    (b & CONT_MASK) == CONT_BYTES
}

// https://tools.ietf.org/html/rfc3629
const UTF8_CHAR_WIDTH: &[u8; 256] = &[
    // 1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 1
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 2
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 3
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 4
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 5
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 6
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 7
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 8
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 9
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // A
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // B
    0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // C
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // D
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // E
    4, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 // F
];
