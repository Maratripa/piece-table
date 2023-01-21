//! # PieceTable
//!
//! A piece table data structure implementation.

use std::{
    fs::{self, File},
    io::{self, BufReader, Read, Write},
    path::{Path, PathBuf},
};

#[derive(Copy, Clone, Debug, PartialEq)]
enum BufChoice {
    ReadOnly,
    AppendOnly,
}

#[derive(Debug)]
enum ReadBuffer {
    Str(String),
    File(PathBuf),
}

#[derive(Debug)]
struct Piece {
    buffer: BufChoice,
    start: usize,
    length: usize,
}

#[derive(Debug)]
pub struct PieceTable {
    read_buf: ReadBuffer,
    append_buf: String,
    pieces: Vec<Piece>,
}

impl PieceTable {
    /// Create new PieceTable using a file as read_buffer.
    ///
    /// # Examples
    /// ```
    /// use piecetable::PieceTable;
    /// use std::path::Path;
    ///
    /// let mut piecetable = PieceTable::from_file(Path::new("test.txt")).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Possible error when opening file.
    ///
    /// Possible error when reading metadata of file.
    pub fn from_file(path: &Path) -> Result<PieceTable, io::Error> {
        let file = File::open(path)?;
        let file_size = file.metadata()?.len();

        Ok(PieceTable {
            read_buf: ReadBuffer::File(PathBuf::from(path)),
            append_buf: String::new(),
            pieces: vec![Piece {
                buffer: BufChoice::ReadOnly,
                start: 0,
                length: file_size as usize,
            }],
        })
    }

    /// Create new PieceTable using a base string as read_buffer.
    ///
    /// # Examples
    /// ```
    /// use piecetable::PieceTable;
    ///
    /// let mut piecetable = PieceTable::from_str("Buenos dias");
    /// ```
    pub fn from_str(buf: &str) -> PieceTable {
        let buf = buf.to_string();
        let buf_size = buf.len();

        PieceTable {
            read_buf: ReadBuffer::Str(buf),
            append_buf: String::new(),
            pieces: vec![Piece {
                buffer: BufChoice::ReadOnly,
                start: 0,
                length: buf_size,
            }],
        }
    }

    /// Join pieces reading from read_buffer and append_buffer.
    ///
    /// # Examples
    /// ```
    /// use piecetable::PieceTable;
    ///
    /// let piecetable = PieceTable::from_str("Buenos dias");
    ///
    /// println!("{}", piecetable.display_result().unwrap());
    /// ```
    ///
    /// # Errors
    ///
    /// Posible error when reading file to string.
    pub fn display_result(&self) -> Result<String, io::Error> {
        match &self.read_buf {
            ReadBuffer::Str(buf) => Ok(self.connect_pieces(buf)),
            ReadBuffer::File(path) => {
                let read_buf = fs::read_to_string(path)?;
                Ok(self.connect_pieces(&read_buf))
            }
        }
    }

    fn connect_pieces(&self, read_buf: &String) -> String {
        let mut result = String::new();

        for piece in self.pieces.iter() {
            let range = piece.start..piece.start + piece.length;

            match piece.buffer {
                BufChoice::ReadOnly => result.push_str(&read_buf[range]),
                BufChoice::AppendOnly => result.push_str(&self.append_buf[range]),
            }
        }

        result
    }

    /// Insert string slice in piece table.
    ///
    /// split_index is the global index for the starting character or the new string slice.
    ///
    /// # Examples
    /// ```
    /// use piecetable::PieceTable;
    ///
    /// let buffer = "Buenos dias, que buen clima hoy";
    /// //                       |
    /// //                      11
    /// let mut piece_table = PieceTable::from_str(buffer);
    ///
    /// piece_table.insert(" Matias", 11);
    ///
    /// assert_eq!("Buenos dias Matias, que buen clima hoy", piece_table.display_result().unwrap());
    /// ```
    pub fn insert(&mut self, buf: &str, index: usize) {
        let append_buf_len = self.append_buf.len();
        let length = buf.len();
        let idx = self.find_piece_at_position(index);
        let is_border = self.position_is_at_border(index);

        // check if insert is at the end of an append piece and is an extension of last added to append buffer
        if is_border && idx != 0 {
            let prev_piece = &self.pieces[idx - 1];
            if prev_piece.buffer == BufChoice::AppendOnly {
                if prev_piece.start + prev_piece.length == append_buf_len {
                    self.append_buf.push_str(buf);
                    self.pieces[idx - 1].length += length;
                    return;
                }
            }
        };

        // check if it is at a border
        if is_border {
            self.pieces.insert(
                idx,
                Piece {
                    buffer: BufChoice::AppendOnly,
                    start: append_buf_len,
                    length,
                },
            );
            self.append_buf.push_str(buf);
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
                buffer: BufChoice::AppendOnly,
                start: append_buf_len,
                length,
            },
        );

        // Add characters to append buffer
        self.append_buf.push_str(buf);
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

    /// Delete character at position _char_index_
    ///
    /// # Examples
    ///
    /// ```
    /// use piecetable::PieceTable;
    ///
    /// let buffer = "Mucho gus8to";
    /// //                     |
    /// //                     9
    /// let mut piece_table = PieceTable::from_str(buffer);
    ///
    /// piece_table.delete_char(9);
    ///
    /// assert_eq!("Mucho gusto", piece_table.display_result().unwrap());
    /// ```
    pub fn delete_char(&mut self, char_index: usize) {
        let (piece_index, index_in_piece) = self.split_piece_index_and_lenght(char_index);

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

    fn _total_length(&self) -> usize {
        self.pieces.iter().map(|x| x.length).sum()
    }

    /// Save piecetable to file
    ///
    /// # Expamples
    /// ```
    /// use piecetable::PieceTable;
    /// use std::{
    ///     path::Path,
    ///     fs,
    ///     io::Write
    /// };
    ///
    /// {
    ///     let mut file = fs::File::create("test.txt").unwrap();
    ///     file.write_all(b"Buenos dias").unwrap();
    /// }
    ///
    /// let mut piecetable = PieceTable::from_file(Path::new("test.txt")).unwrap();
    /// piecetable.insert(" Matias", 11);
    ///
    /// let _file_length: usize = piecetable.save_file().unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if piecetable does not contain a filepath to save the pieces to.
    pub fn save_file(&mut self) -> Result<usize, &str> {
        if let ReadBuffer::File(path) = &self.read_buf {
            let input = match File::open(path) {
                Ok(x) => x,
                Err(_e) => return Err("Could not open file"),
            };

            let mut reader = BufReader::new(input);

            let mut read_buffer = String::new();

            let _len = match reader.read_to_string(&mut read_buffer) {
                Ok(x) => x,
                Err(_e) => return Err("Could not read file"),
            };

            match self.save(path, &read_buffer) {
                Ok(len) => {
                    self.pieces = vec![Piece {
                        buffer: BufChoice::ReadOnly,
                        start: 0,
                        length: len,
                    }];
                    Ok(len)
                }
                Err(_e) => Err("Error saving file"),
            }
        } else {
            Err("PieceTable does not contain a filepath. Run PieceTable::save_to_file(path) to save to a file.")
        }
    }

    /// Save piecetable to new filepath and modify filename of piecetable if it exists.
    ///
    /// # Examples
    /// ```
    /// use piecetable::PieceTable;
    /// use std::path::Path;
    ///
    /// let mut piecetable = PieceTable::from_str("Buenos dias");
    /// let _new_file_length = piecetable.save_to_file(Path::new("test.txt")).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Error creating file
    /// Error opening file
    /// Error reading file
    /// Error reading metadata
    pub fn save_to_file(&mut self, path: &Path) -> Result<usize, io::Error> {
        let mut _len = 0;
        match &self.read_buf {
            ReadBuffer::Str(buf) => {
                _len = self.save(path, buf)?;
            }
            ReadBuffer::File(saved_path) => {
                let input = File::open(saved_path)?;

                let mut reader = BufReader::new(input);

                let mut read_buffer = String::new();

                let _read_len = reader.read_to_string(&mut read_buffer)?;

                _len = self.save(path, &read_buffer)?;
            }
        }

        self.read_buf = ReadBuffer::File(PathBuf::from(path));
        self.pieces = vec![Piece {
            buffer: BufChoice::ReadOnly,
            start: 0,
            length: _len,
        }];

        Ok(_len)
    }

    fn save(&self, path: &Path, read_buffer: &String) -> Result<usize, io::Error> {
        let mut file = File::create(path)?;
        write!(file, "{}", self.connect_pieces(&read_buffer))?;

        Ok(file.metadata()?.len() as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_func() {
        let initial_buffer = "Buenos dias, el clima se ve muy bien";
        let mut pt = PieceTable::from_str(initial_buffer);

        pt.delete_char(11);

        assert_eq!(2, pt.pieces.len());
        assert_eq!(
            "Buenos dias el clima se ve muy bien",
            pt.display_result().unwrap()
        );
    }

    #[test]
    fn test_file() {
        {
            let mut file = fs::File::create("test.txt").unwrap();
            file.write_all(b"Buenos dias").unwrap();
        }
        {
            let mut pt = PieceTable::from_file(Path::new("test.txt")).unwrap();

            pt.insert(" Matias", 11);

            pt.save_file().unwrap();
        }

        let contents = fs::read_to_string("test.txt").unwrap();

        assert_eq!("Buenos dias Matias", contents);
    }
}
