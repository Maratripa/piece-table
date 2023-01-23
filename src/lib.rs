//! # PieceTable
//!
//! A piece table data structure implementation.

#[derive(Copy, Clone, Debug, PartialEq)]
enum Buffer {
    Read,
    Add,
}

#[derive(Debug)]
struct Piece {
    buffer: Buffer,
    start: usize,
    length: usize,
}

#[derive(Debug)]
pub struct PieceTable<'a, T: 'a + Clone> {
    read_buf: &'a [T],
    add_buf: Vec<T>,
    pieces: Vec<Piece>,
}

pub struct Iter<'a, T: 'a + Clone> {
    table: &'a PieceTable<'a, T>,
    piece_idx: usize,
    iter: std::slice::Iter<'a, T>,
}

impl<'a, T: 'a + Clone> PieceTable<'a, T> {
    /// Create a new, empty PieceTable.
    ///
    /// # Examples
    /// ```
    /// use piecetable::PieceTable;
    ///
    /// let piece_table = PieceTable::<char>::new();
    /// ```
    pub fn new() -> PieceTable<'a, T> {
        PieceTable {
            read_buf: &[],
            add_buf: vec![],
            pieces: vec![],
        }
    }

    /// Create a new PieceTable with capacity for append buffer and piece buffer
    /// for insertion without reallocation.
    ///
    /// # Examples
    /// ```
    /// use piecetable::PieceTable;
    ///
    /// let piece_table = PieceTable::<char>::with_capacity(10, 10);
    /// ```
    pub fn with_capacity(buffer_capacity: usize, piece_capacity: usize) -> PieceTable<'a, T> {
        PieceTable {
            read_buf: &[],
            add_buf: Vec::with_capacity(buffer_capacity),
            pieces: Vec::with_capacity(piece_capacity),
        }
    }

    /// Set a source as the read only buffer for the piece table.
    ///
    /// # Examples
    /// ```
    /// use piecetable::PieceTable;
    ///
    /// let mut piece_table = PieceTable::new();
    ///
    /// let source = "Buenos dias".as_bytes();
    ///
    /// piece_table.src(source);
    /// ```
    pub fn src(&mut self, src: &'a [T]) {
        if src.len() > 0 {
            self.pieces.push(Piece {
                start: 0,
                length: src.len(),
                buffer: Buffer::Read,
            })
        }

        self.read_buf = src;
    }

    /// Create new PieceTable using a base string as read_buffer.
    ///
    /// # Examples
    /// ```
    /// use piecetable::PieceTable;
    ///
    /// let mut piecetable = PieceTable::<u8>::from_str("Buenos dias");
    /// ```
    pub fn from_str(buf: &str) -> PieceTable<u8> {
        let buf_size = buf.len();

        PieceTable {
            read_buf: buf.as_bytes(),
            add_buf: vec![],
            pieces: vec![Piece {
                buffer: Buffer::Read,
                start: 0,
                length: buf_size,
            }],
        }
    }

    /// Reserve capacity for at least 'additional' more elements in the append buffer.
    pub fn reserve_buffer(&mut self, additional: usize) {
        self.add_buf.reserve(additional);
    }

    /// Reserve capacity for at least 'additional' more elements in the piece buffer.
    pub fn reserve_piece(&mut self, additional: usize) {
        self.pieces.reserve(additional);
    }

    /// Insert 'element' to piece table at position 'index'.
    ///
    /// # Examples
    /// ```
    /// use piecetable::PieceTable;
    ///
    /// let buffer = "Buenos dias, que buen clima hoy";
    /// //                       |
    /// //                      11
    /// let mut piece_table = PieceTable::<u8>::from_str(buffer);
    ///
    /// piece_table.insert(b'M', 11);
    /// ```
    pub fn insert(&mut self, element: T, index: usize) {
        let append_buf_len = self.add_buf.len();
        let idx = self.find_piece_at_position(index);
        let is_border = self.position_is_at_border(index);

        self.add_buf.push(element);

        // check if insert is at the end of an append piece and is an extension of last added to append buffer
        if is_border && idx != 0 {
            let prev_piece = &self.pieces[idx - 1];
            if prev_piece.buffer == Buffer::Add {
                if prev_piece.start + prev_piece.length == append_buf_len {
                    self.pieces[idx - 1].length += 1;
                    return;
                }
            }
        };

        // check if it is at a border
        if is_border {
            self.pieces.insert(
                idx,
                Piece {
                    buffer: Buffer::Add,
                    start: append_buf_len,
                    length: 1,
                },
            );
            return;
        }

        // split piece in which is inserted and insert new piece
        // get split index
        let mut counter = 0;
        for i in 0..idx {
            counter += self.pieces[i].length;
        }
        let split_index = index - counter;

        // divide pieces
        self.divide_piece(idx, split_index);
        self.pieces.insert(
            idx + 1,
            Piece {
                buffer: Buffer::Add,
                start: append_buf_len,
                length: 1,
            },
        );
    }

    /// Insert array of elements to piece table at position 'index'.
    ///
    /// # Examples
    /// ```
    /// use piecetable::PieceTable;
    ///
    /// let mut piece_table = PieceTable::new();
    /// piece_table.src(b"Buenos dias");
    ///
    /// piece_table.insert_slice(b" Matias", 11);
    /// ```
    pub fn insert_slice(&mut self, slice: &[T], index: usize) {
        for (i, c) in slice.iter().enumerate() {
            self.insert(c.clone(), index + i);
        }
    }

    fn find_piece_at_position(&self, position: usize) -> usize {
        if position == 0 {
            return 0;
        }

        let mut counter = 0;
        for (i, piece) in self.pieces.iter().enumerate() {
            if position < counter + piece.length {
                return i;
            }

            counter += piece.length;
        }

        self.pieces.len()
    }

    fn position_is_at_border(&self, position: usize) -> bool {
        if position == 0 {
            return true;
        }

        let mut counter = 0;
        for piece in self.pieces.iter() {
            if position == piece.length + counter {
                return true;
            }

            counter += piece.length;
        }

        false
    }

    /// Delete character at position 'char_index'.
    ///
    /// # Examples
    ///
    /// ```
    /// use piecetable::PieceTable;
    ///
    /// let buffer = "Mucho gus8to";
    /// //                     |
    /// //                     9
    /// let mut piece_table = PieceTable::<u8>::from_str(buffer);
    ///
    /// piece_table.delete_char(9);
    /// ```
    pub fn delete(&mut self, index: usize) {
        let (piece_index, index_in_piece) = self.split_piece_index_and_lenght(index);

        let mut piece = &mut self.pieces[piece_index];

        // deletes char at beggining of piece
        // only case of last remaining char in piece
        if index_in_piece == 0 {
            piece.start += 1;
            piece.length -= 1;

            if piece.length == 0 {
                self.delete_and_join(piece_index);
            }

            return;
        }

        // deletes char at end of piece
        if index_in_piece == piece.length - 1 {
            piece.length -= 1;
            return;
        }

        // deletes char in the middle of piece
        self.divide_piece(piece_index, index_in_piece);

        self.pieces[piece_index + 1].start += 1;
        self.pieces[piece_index + 1].length -= 1;
    }

    fn split_piece_index_and_lenght(&self, split_index: usize) -> (usize, usize) {
        let mut counter: usize = 0;

        for (i, piece) in self.pieces.iter().enumerate() {
            if split_index < counter + piece.length {
                return (i, split_index - counter);
            }

            counter += piece.length;
        }

        (
            self.pieces.len() - 1,
            match self.pieces.last() {
                Some(x) => x.length - 1,
                None => 0,
            },
        )
    }

    fn divide_piece(&mut self, piece_index: usize, split_length: usize) {
        let piece = self.pieces.remove(piece_index);

        self.pieces.insert(
            piece_index,
            Piece {
                buffer: piece.buffer,
                start: piece.start,
                length: split_length,
            },
        );
        self.pieces.insert(
            piece_index + 1,
            Piece {
                buffer: piece.buffer,
                start: piece.start + split_length,
                length: piece.length - split_length,
            },
        );
    }

    fn delete_and_join(&mut self, piece_index: usize) {
        self.pieces.remove(piece_index);

        if piece_index == 0 || piece_index == self.pieces.len() - 1 {
            return;
        }

        let prev = &self.pieces[piece_index - 1];
        let next = &self.pieces[piece_index];

        if prev.buffer == next.buffer && prev.start + prev.length == next.start {
            self.pieces[piece_index - 1].length += next.length;
            self.pieces.remove(piece_index);
        }
    }

    /// Get total length of piece table.
    pub fn len(&self) -> usize {
        self.pieces.iter().map(|x| x.length).sum()
    }

    fn get_buffer(&'a self, piece: &Piece) -> &'a [T] {
        match piece.buffer {
            Buffer::Read => self.read_buf,
            Buffer::Add => &self.add_buf,
        }
    }

    pub fn iter(&'a self) -> Iter<'a, T> {
        let piece = &self.pieces[0];
        let buf = self.get_buffer(piece);
        let iter = buf[piece.start .. piece.start + piece.length].iter();
        Iter {
            table: &self,
            piece_idx: 0,
            iter,
        }
    }
}

impl<'a, T: 'a + Clone> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.iter.next() {
            return Some(next)
        }

        self.piece_idx += 1;

        if self.piece_idx >= self.table.pieces.len() {
            return None
        }

        let piece = &self.table.pieces[self.piece_idx];
        let buf = self.table.get_buffer(piece);

        self.iter = buf[piece.start .. piece.start + piece.length].iter();

        self.next()
    }
}
