#[derive(Copy, Clone, Debug)]
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
    buf_size: usize,
    append_buf: String,
    pieces: Vec<Piece> // TODO: Change to VecDeque
}

impl PieceTable {
    fn new(read_buf: &str) -> PieceTable {
        let buf = read_buf.to_string();
        let size = buf.len();

        PieceTable { 
            read_buf: buf,
            buf_size: size,
            append_buf: String::new(),
            pieces: vec![Piece {
                buffer: BufChoice::ReadOnly, 
                start: 0, 
                length: size
            }]
        }
    }
    
    pub fn insert(&mut self, buf: &str) {
        let append_start = self.append_buf.len();
        let length = buf.len();

        // Position on read_only buffer
    }

    pub fn delete(&mut self) {

    }


    /// Calls when inserting a str slice into the piece table.
    /// 
    /// returns the index at which the next piece needs to be 
    /// inserted.
    fn split_read_only_table(&mut self, split_index: usize) -> usize {
        if split_index == 0 {
            return 0
        }

        let total_length: usize = self.pieces
            .iter()
            .map(|x| x.length)
            .sum();

        if split_index >= total_length {
            return self.pieces.len()
        }

        let mut counter: usize = 0;
        let mut piece_index = 0;
        let mut split_length: usize = 0;

        for (i, piece) in self.pieces.iter().enumerate() {
            if split_index < counter + piece.length {
                piece_index = i;
                split_length = counter + piece.length - split_index;
                break;
            }

            counter += piece.length;
        }

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

        piece_index + 1
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
    }
}
