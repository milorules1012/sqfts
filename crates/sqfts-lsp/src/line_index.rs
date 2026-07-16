//! Byte-offset ↔ LSP UTF-16 position conversion.

use tower_lsp::lsp_types::Position;

/// Line index over a UTF-8 source string.
#[derive(Debug, Clone)]
pub struct LineIndex {
    /// Byte offset of the start of each line (line 0 at 0).
    line_starts: Vec<usize>,
    /// Source length in bytes.
    len: usize,
}

impl LineIndex {
    /// Build from source text.
    #[must_use]
    pub fn new(text: &str) -> Self {
        let mut line_starts = vec![0usize];
        let bytes = text.as_bytes();
        let mut i = 0usize;
        while i < bytes.len() {
            if bytes[i] == b'\n' {
                line_starts.push(i + 1);
            } else if bytes[i] == b'\r' {
                if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                    line_starts.push(i + 2);
                    i += 1;
                } else {
                    line_starts.push(i + 1);
                }
            }
            i += 1;
        }
        Self {
            line_starts,
            len: text.len(),
        }
    }

    /// Convert a byte offset to an LSP Position (UTF-16 columns).
    #[must_use]
    pub fn position(&self, text: &str, byte_offset: usize) -> Position {
        let offset = byte_offset.min(self.len);
        let line = match self.line_starts.binary_search(&offset) {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
        };
        let line_start = self.line_starts[line];
        let col_bytes = &text.as_bytes()[line_start..offset];
        // SAFETY: line_start..offset is within a line and at char boundaries for valid UTF-8
        // if offset is on a char boundary; clamp otherwise.
        let slice = match std::str::from_utf8(col_bytes) {
            Ok(s) => s,
            Err(_) => {
                // Walk back to char boundary
                let mut end = offset;
                while end > line_start && !text.is_char_boundary(end) {
                    end -= 1;
                }
                &text[line_start..end]
            }
        };
        let character = slice.encode_utf16().count() as u32;
        Position {
            line: line as u32,
            character,
        }
    }

    /// Convert an LSP Position to a byte offset.
    #[must_use]
    pub fn offset(&self, text: &str, pos: Position) -> usize {
        let line = pos.line as usize;
        if line >= self.line_starts.len() {
            return self.len;
        }
        let line_start = self.line_starts[line];
        let line_end = self.line_starts.get(line + 1).copied().unwrap_or(self.len);
        let line_text = &text[line_start..line_end];
        let mut utf16 = 0u32;
        for (byte_idx, ch) in line_text.char_indices() {
            if utf16 >= pos.character {
                return line_start + byte_idx;
            }
            utf16 += ch.len_utf16() as u32;
        }
        line_end.saturating_sub(if line_text.ends_with("\r\n") {
            2
        } else if line_text.ends_with('\n') || line_text.ends_with('\r') {
            1
        } else {
            0
        })
    }

    /// Byte range → LSP Range.
    #[must_use]
    pub fn range(&self, text: &str, start: usize, end: usize) -> tower_lsp::lsp_types::Range {
        tower_lsp::lsp_types::Range {
            start: self.position(text, start),
            end: self.position(text, end),
        }
    }
}

/// Identify the identifier under / next to a byte offset.
#[must_use]
pub fn identifier_at(text: &str, byte_offset: usize) -> Option<(std::ops::Range<usize>, String)> {
    let bytes = text.as_bytes();
    if bytes.is_empty() {
        return None;
    }
    let mut i = byte_offset.min(bytes.len());
    if i == bytes.len() {
        i -= 1;
    }
    // If on a non-ident char, try one left
    if !is_ident_byte(bytes[i]) && i > 0 {
        i -= 1;
    }
    if !is_ident_byte(bytes[i]) {
        return None;
    }
    let mut start = i;
    while start > 0 && is_ident_byte(bytes[start - 1]) {
        start -= 1;
    }
    let mut end = i + 1;
    while end < bytes.len() && is_ident_byte(bytes[end]) {
        end += 1;
    }
    let name = text.get(start..end)?.to_string();
    Some((start..end, name))
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_positions() {
        let text = "hello\nworld";
        let idx = LineIndex::new(text);
        assert_eq!(idx.position(text, 0).line, 0);
        assert_eq!(idx.position(text, 0).character, 0);
        assert_eq!(idx.position(text, 6).line, 1);
        assert_eq!(idx.position(text, 6).character, 0);
        assert_eq!(idx.position(text, 8).character, 2);
    }

    #[test]
    fn utf16_emoji() {
        // 😀 is 4 UTF-8 bytes, 2 UTF-16 code units
        let text = "a😀b";
        let idx = LineIndex::new(text);
        let pos = idx.position(text, text.find('b').unwrap());
        assert_eq!(pos.character, 3); // 'a' + 2 for emoji
        let off = idx.offset(
            text,
            Position {
                line: 0,
                character: 3,
            },
        );
        assert_eq!(off, text.find('b').unwrap());
    }

    #[test]
    fn identifier_at_works() {
        let text = "call TAG_fnc_foo;";
        let (_, name) = identifier_at(text, 8).unwrap();
        assert_eq!(name, "TAG_fnc_foo");
    }
}
