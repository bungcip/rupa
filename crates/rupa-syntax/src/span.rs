//! Source span tracking for Rupa.
//!
//! A source location is a 24-bit byte offset paired with an 8-bit file id, so a
//! [`Span`] is two packed 32-bit words (DECISION-043). Line and column numbers
//! are not stored; diagnostics derive them from a [`LineIndex`].

use serde::{Deserialize, Serialize};
use std::fmt;

/// Maximum byte offset representable in a packed location.
///
/// This is `2^24 - 1` bytes (16 MiB - 1). A file may not be longer than this
/// because a span's end offset can equal the file length.
pub const MAX_SOURCE_BYTES: u32 = (1 << 24) - 1;

/// Maximum number of distinct source files in one compilation.
pub const MAX_FILES: u32 = 1 << 8;

const OFFSET_MASK: u32 = MAX_SOURCE_BYTES;

/// File identifier
///
/// One compilation may reference at most 256 files, so an 8-bit identifier is
/// sufficient and keeps [`Location`] packable (DECISION-043).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FileId(pub u8);

impl FileId {
    /// Maximum number of distinct file ids, including id zero.
    pub const MAX_FILES: u32 = MAX_FILES;

    /// Create a file id from a zero-based index, or `None` when it exceeds the
    /// 256-file limit.
    pub fn new(index: u32) -> Option<Self> {
        if index < MAX_FILES {
            Some(Self(u8::try_from(index).ok()?))
        } else {
            None
        }
    }

    /// Return the zero-based numeric index.
    pub fn index(self) -> u32 {
        u32::from(self.0)
    }
}

/// Byte offset from the start of a source file (zero-based).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Position {
    /// Byte offset from start of file.
    pub offset: u32,
}

impl Position {
    /// Create a position from a byte offset.
    ///
    /// Offsets above [`MAX_SOURCE_BYTES`] cannot be stored in a span.
    pub const fn new(offset: u32) -> Self {
        Self { offset }
    }

    /// Create a position from a `usize` offset, returning `None` when it does
    /// not fit the compact source limit.
    pub fn from_usize(offset: usize) -> Option<Self> {
        let offset = u32::try_from(offset).ok()?;
        (offset <= MAX_SOURCE_BYTES).then_some(Self { offset })
    }

    /// Return the byte offset as a `usize` for slicing.
    pub fn offset_usize(self) -> Option<usize> {
        usize::try_from(self.offset).ok()
    }
}

/// A packed file id and byte offset.
///
/// The high 8 bits hold [`FileId`], the low 24 bits hold the offset. This makes
/// [`Span`] exactly two machine words on both 32-bit and 64-bit targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Location(u32);

impl Location {
    /// Pack a file id and position into one word.
    pub fn new(file_id: FileId, position: Position) -> Self {
        assert!(
            position.offset <= MAX_SOURCE_BYTES,
            "source offset exceeds the 24-bit limit"
        );
        Self((file_id.index() << 24) | position.offset)
    }

    /// Return the file id stored in this location.
    pub fn file_id(self) -> FileId {
        FileId((self.0 >> 24) as u8)
    }

    /// Return the byte offset stored in this location.
    pub fn offset(self) -> u32 {
        self.0 & OFFSET_MASK
    }

    /// Return the byte offset as a [`Position`].
    pub fn position(self) -> Position {
        Position::new(self.offset())
    }

    /// Return the byte offset as a `usize` for slicing.
    pub fn offset_usize(self) -> Option<usize> {
        usize::try_from(self.offset()).ok()
    }
}

/// Source span (start to end)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    /// Packed start location.
    pub start: Location,
    /// Packed end location.
    pub end: Location,
}

impl Span {
    /// Create a new span.
    pub fn new(file_id: FileId, start: Position, end: Position) -> Self {
        Self {
            start: Location::new(file_id, start),
            end: Location::new(file_id, end),
        }
    }

    /// Create a single-point span.
    pub fn single(file_id: FileId, pos: Position) -> Self {
        Self {
            start: Location::new(file_id, pos),
            end: Location::new(file_id, pos),
        }
    }

    /// Merge two spans, keeping this span's start and the other span's end.
    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start,
            end: other.end,
        }
    }

    /// File id of the span's start location.
    pub fn file_id(self) -> FileId {
        self.start.file_id()
    }

    /// Start position as a byte offset.
    pub fn start_position(self) -> Position {
        self.start.position()
    }

    /// End position as a byte offset.
    pub fn end_position(self) -> Position {
        self.end.position()
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.offset)
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start.file_id() != self.end.file_id() {
            write!(
                f,
                "{:?}:{}..{:?}:{}",
                self.start.file_id(),
                self.start.offset(),
                self.end.file_id(),
                self.end.offset()
            )
        } else {
            write!(f, "{}..{}", self.start.offset(), self.end.offset())
        }
    }
}

/// One-based line and Unicode-scalar column derived from a byte offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineColumn {
    /// One-based line number.
    pub line: u32,
    /// One-based column number, counted in Unicode scalar values.
    pub column: u32,
}

/// Byte-offset to line/column index built from source text.
///
/// This is the only place line and column information lives. It is built on
/// demand at diagnostic boundaries, so the AST never stores it (DECISION-043).
#[derive(Debug, Clone)]
pub struct LineIndex<'a> {
    source: &'a str,
    /// Byte offset of the first character of each line.
    starts: Vec<u32>,
}

impl<'a> LineIndex<'a> {
    /// Build an index for `source`.
    pub fn new(source: &'a str) -> Self {
        let mut starts = Vec::new();
        starts.push(0);
        for (index, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                starts.push(u32::try_from(index + 1).unwrap_or(u32::MAX));
            }
        }
        Self { source, starts }
    }

    /// Resolve a position to a one-based line and column.
    pub fn line_col(&self, position: Position) -> LineColumn {
        self.line_col_offset(position.offset)
    }

    /// Resolve a byte offset to a one-based line and column.
    ///
    /// Offsets beyond the source length are clamped. If an offset lands inside
    /// a multi-byte scalar, the nearest preceding boundary is used.
    pub fn line_col_offset(&self, offset: u32) -> LineColumn {
        let mut offset = usize::try_from(offset)
            .unwrap_or(usize::MAX)
            .min(self.source.len());
        while offset > 0 && !self.source.is_char_boundary(offset) {
            offset -= 1;
        }

        let index = self
            .starts
            .partition_point(|&start| usize::try_from(start).unwrap_or(usize::MAX) <= offset)
            .saturating_sub(1);
        let line_start = usize::try_from(self.starts[index]).unwrap_or(usize::MAX);
        let column = self.source[line_start..offset]
            .chars()
            .count()
            .saturating_add(1);

        LineColumn {
            line: u32::try_from(index + 1).unwrap_or(u32::MAX),
            column: u32::try_from(column).unwrap_or(u32::MAX),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    fn position(offset: u32) -> Position {
        Position::new(offset)
    }

    #[test]
    fn span_merge_keeps_the_start_and_the_other_end() {
        let start = position(0);
        let middle = position(4);
        let end = position(10);
        let merged = Span::single(FileId(0), start).merge(Span::new(FileId(0), middle, end));

        assert_eq!(merged.start_position(), start);
        assert_eq!(merged.end_position(), end);
    }

    #[test]
    fn display_uses_byte_offsets() {
        let start = position(42);
        assert_eq!(start.to_string(), "42");
        assert_eq!(Span::single(FileId(0), start).to_string(), "42..42");

        let span = Span::new(FileId(0), start, position(49));
        assert_eq!(span.to_string(), "42..49");
    }

    #[test]
    fn from_usize_checks_the_source_limit() {
        assert_eq!(Position::from_usize(19), Some(position(19)));
        assert_eq!(
            Position::from_usize(MAX_SOURCE_BYTES as usize),
            Some(position(MAX_SOURCE_BYTES))
        );
        assert_eq!(Position::from_usize(MAX_SOURCE_BYTES as usize + 1), None);
    }

    #[test]
    fn offset_usize_round_trips_on_this_target() {
        let position = position(1234);
        assert_eq!(position.offset_usize(), Some(1234));
    }

    #[test]
    fn packed_locations_round_trip_file_and_offset() {
        let location = Location::new(FileId(255), position(MAX_SOURCE_BYTES));
        assert_eq!(location.file_id(), FileId(255));
        assert_eq!(location.offset(), MAX_SOURCE_BYTES);

        let location = Location::new(FileId(3), position(0x12_3456));
        assert_eq!(location.file_id(), FileId(3));
        assert_eq!(location.offset(), 0x12_3456);
    }

    #[test]
    fn file_id_enforces_the_256_file_limit() {
        assert_eq!(FileId::new(0), Some(FileId(0)));
        assert_eq!(FileId::new(255), Some(FileId(255)));
        assert_eq!(FileId::new(256), None);
    }

    #[test]
    fn line_index_counts_unicode_scalars_and_lines() {
        let source = "ab\néx\n";
        let index = LineIndex::new(source);

        assert_eq!(index.line_col_offset(0), LineColumn { line: 1, column: 1 });
        assert_eq!(index.line_col_offset(1), LineColumn { line: 1, column: 2 });
        assert_eq!(index.line_col_offset(3), LineColumn { line: 2, column: 1 });
        assert_eq!(index.line_col_offset(5), LineColumn { line: 2, column: 2 });
        assert_eq!(
            index.line_col_offset(source.len() as u32),
            LineColumn { line: 3, column: 1 }
        );
    }

    #[test]
    fn line_index_clamps_out_of_range_and_mid_scalar_offsets() {
        let source = "é";
        let index = LineIndex::new(source);
        assert_eq!(index.line_col_offset(1), LineColumn { line: 1, column: 1 });
        assert_eq!(index.line_col_offset(99), LineColumn { line: 1, column: 2 });
    }

    #[test]
    fn compact_layout_stays_within_the_target() {
        assert_eq!(size_of::<FileId>(), 1);
        assert_eq!(size_of::<Position>(), 4);
        assert_eq!(size_of::<Location>(), 4);
        assert_eq!(size_of::<Span>(), 8);
    }
}
