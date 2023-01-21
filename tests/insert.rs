use piecetable::PieceTable;

fn main() {
    let mut pt = PieceTable::from_str("HolaMatias.");
    println!("{}", pt.display_result().unwrap());

    pt.insert(",", 4);
    println!("{}", pt.display_result().unwrap());

    pt.insert(" ", 5);
    println!("{}", pt.display_result().unwrap());

    pt.insert("Hola, Martin. ", 0);
    println!("{}", pt.display_result().unwrap());

    pt.insert(", buenos dias", 18);
    println!("{}", pt.display_result().unwrap());

    pt.insert(" Saludos!", 40);
    println!("{}", pt.display_result().unwrap());
}
