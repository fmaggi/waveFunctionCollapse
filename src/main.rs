pub mod model;

use crate::model::Model;

fn main() {
    let mut m = Model::new(0);
    while m.collapse() {
        println!("Entry");
        m.print();
    }
    println!("Entry");
    m.print();
}
