extern crate term_cursor as cursor;

fn main() {
    print!("{}", cursor::Clear);

    print!("{}Circle ->{}Rectangle ->", cursor::Goto(15, 6), cursor::Right(13));
    filled_rectangle((50, 3), (8, 8));
    circle((30, 6), 3.0);

    print!(
        "{}Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam.",
        cursor::Goto(0, 0)
    );
    print!(
        "{}-- Hello! This shows how to use the crate. --",
        cursor::Goto(10, 0)
    );
    print!("{}Look at me!", cursor::Goto(10, 10));
    print!("{}I'm one line down!", cursor::Down(1));
    println!(
        "{}And another one!{}I'm on the right!",
        cursor::Down(1),
        cursor::Right(10)
    );
}

fn filled_rectangle((x, y): (i32, i32), (width, height): (i32, i32)) {
    for x in x..x + width {
        for y in y..y + height {
            print!("{}#", cursor::Goto(x, y));
        }
    }
}

fn circle((origin_x, origin_y): (i32, i32), radius: f32) {
    let full_circle = 2.0 * std::f32::consts::PI;
    let step = full_circle / (full_circle * radius);

    let mut alpha = 0.0;
    while alpha < full_circle {
        let (x, y) = (alpha.cos() * radius, alpha.sin() * radius);
        let (x, y) = (x.round() as i32 + origin_x, y.round() as i32 + origin_y);

        println!("{}*", cursor::Goto(x, y));

        alpha += step;
    }
}
