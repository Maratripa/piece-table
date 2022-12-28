#[derive(Copy, Clone, Debug, PartialEq)]
enum BufChoice {
    ReadOnly,
    AppendOnly
}

#[derive(Debug)]
struct Piece {
    buffer: BufChoice,
    start: usize,
    length: usize
}

pub struct PieceTable {
    read_buf: String,
    append_buf: String,
    pieces: Vec<Piece> // TODO: Change to VecDeque
}

impl PieceTable {
    pub fn new(read_buf: &str) -> PieceTable {
        let buf = read_buf.to_string();
        let size = buf.len();

        PieceTable { 
            read_buf: buf,
            append_buf: String::new(),
            pieces: vec![Piece {
                buffer: BufChoice::ReadOnly, 
                start: 0, 
                length: size
            }]
        }
    }

    pub fn display_result(&self) -> String {
        let mut result = String::new();

        for piece in self.pieces.iter() {
            match piece.buffer {
                BufChoice::ReadOnly => result.push_str(&self.read_buf[piece.start..piece.start+piece.length]),
                BufChoice::AppendOnly => result.push_str(&self.append_buf[piece.start..piece.start+piece.length])
            }
        }

        result
    }
    
    /// Insert string slice in piece table. split_index is the global
    /// index for the starting character or the new string slice.
    pub fn insert(&mut self, buf: &str, split_index: usize) {
        let start = self.append_buf.len();
        let length = buf.len();

        // Split piece table and get index to insert
        let insert_index = self.split_read_only_table(split_index);

        self.pieces.insert(insert_index, Piece {
            buffer: BufChoice::AppendOnly,
            start,
            length
        });

        // Add characters to append buffer
        self.append_buf.push_str(buf);
    }

    pub fn delete(&mut self, char_index: usize) {
        let (piece_index, index_in_piece) = self.piece_index_split_length(char_index);

        let mut piece = &mut self.pieces[piece_index];
        
        // deletes char at beggining of piece
        if index_in_piece == 0 {
            piece.start += 1;
        }

        // deletes char at end of piece
        if index_in_piece == piece.length - 1 {
            piece.length -= 1;
        }

        // deletes char in the middle of piece
        self.divide_piece(piece_index, index_in_piece);

        self.pieces[piece_index + 1].start += 1;

    }


    /// Calls when inserting a str slice into the piece table.
    /// 
    /// returns the index at which the next piece needs to be 
    /// inserted.
    fn split_read_only_table(&mut self, split_index: usize) -> usize {
        if split_index == 0 {
            return 0
        }

        if split_index >= self.total_length() {
            return self.pieces.len()
        }

        let (piece_index, split_length) = self.piece_index_split_length(split_index);

        self.divide_piece(piece_index, split_length);

        piece_index + 1
    }

    fn piece_index_split_length(&self, split_index: usize) -> (usize, usize) {
        let mut counter: usize = 0;

        for (i, piece) in self.pieces.iter().enumerate() {
            if split_index < counter + piece.length {
                return (i, split_index - counter)
            }

            counter += piece.length;
        }

        (self.pieces.len() - 1,
        match self.pieces.last() {
            Some(x) => x.length - 1,
            None => 0
        })
    }

    fn divide_piece(&mut self, piece_index: usize, split_length: usize) {
        let piece = self.pieces.remove(piece_index);

        self.pieces.insert(piece_index, Piece {
            buffer: piece.buffer,
            start: piece.start,
            length: split_length,
        });
        self.pieces.insert(piece_index + 1, Piece {
            buffer: piece.buffer,
            start: split_length,
            length: piece.length - split_length
        });
    }

    fn total_length(&self) -> usize {
        self.pieces
            .iter()
            .map(|x| x.length)
            .sum()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_table_without_append() {
        let initial_buffer = "Buenos dias, el clima se ve muy bien";
        let mut pt = PieceTable::new(initial_buffer);

        let i = pt.split_read_only_table(11);

        assert_eq!(1, i);
        assert_eq!(2, pt.pieces.len());
        assert_eq!(11, pt.pieces[1].start);
    }

    #[test]
    fn test_insert_middle() {
        let initial_buffer = "Buenos dias, el clima se ve muy bien";
        let mut pt = PieceTable::new(initial_buffer);

        pt.insert(" Matias", 11);

        assert_eq!(" Matias", pt.append_buf);
        assert_eq!(3, pt.pieces.len());
        assert_eq!(BufChoice::AppendOnly, pt.pieces[1].buffer);
    }
}
