// SPDX-License-Identifier: GPL-3.0-or-later
pub mod config;

/// Native grid dimensions in device-independent pixels. Keep several complete
/// rows visible even at the minimum supported client size.
pub fn grid_dimensions(width: i32, height: i32) -> (i32, i32, i32) {
    let available = (width - 24).max(1);
    let columns = (available / 88).max(1);
    (available / columns, (height - 394).max(52), 52)
}

/// Unicode packets are UTF-16 code units, not Rust chars or UTF-8 bytes.
pub fn utf16_packets(text: &str) -> Vec<(u16, bool)> {
    text.encode_utf16()
        .flat_map(|unit| [(unit, false), (unit, true)])
        .collect()
}
pub fn studio_fragment(text: &str) -> String {
    text.bytes().map(|b| format!("%{b:02X}")).collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn compact_grid_keeps_multiple_rows_at_minimum_size() {
        for (width, height) in [(400, 560), (576, 700), (900, 900)] {
            let (column, grid, row) = super::grid_dimensions(width, height);
            assert!(grid / row >= 3);
            assert!((width - 24) / column >= 4);
        }
    }
    #[test]
    fn packets_preserve_sequences_and_surrogate_pairs() {
        let packets = super::utf16_packets("t͡ʃ\u{1df00}");
        assert_eq!(packets.len(), 10);
        assert_eq!(
            packets.iter().step_by(2).map(|p| p.0).collect::<Vec<_>>(),
            [0x74, 0x361, 0x283, 0xd837, 0xdf00]
        );
        assert!(packets
            .as_chunks::<2>()
            .0
            .iter()
            .all(|p| !p[0].1 && p[1].1 && p[0].0 == p[1].0));
        assert_eq!(super::studio_fragment("œ#"), "%C5%93%23");
    }
}
