pub mod model;

use crate::model::Model;

fn main() {
    let mut m = match Model::from_image("3Bricks.png", 4){
        Some(m) => m,
        _ => return
    };
    m.collapse(128, 128);
    m.dump("image.ppm");
}

