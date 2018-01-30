extern crate term_cursor as cursor;

#[derive(Clone, Copy)]
struct Rect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl Rect {
    fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

fn main() {
    print!("{}", cursor::Clear);
    window(Rect::new(1, 1, 50, 20), "Window title", "Window content");

    // Move the cursor down under the window, so the command prompt doesn't draw into the window.
    print!("{}", cursor::Goto(0, 26));
}

fn window(rect: Rect, title: &str, content: &str) {
    let client_rect = window_border(rect, title);
    let client_rect = window_scroll_bar(client_rect);
    print!("{}{}", cursor::Goto(client_rect.x, client_rect.y), content);
}

fn window_border(rect: Rect, title: &str) -> Rect {
    // Top border
    print!(
        "{}╔{}╗",
        cursor::Goto(rect.x, rect.y),
        cursor::Right(rect.width - 2)
    );
    for x in rect.x + 1..rect.x + rect.width - 1 {
        print!("{}═", cursor::Goto(x, rect.y));
    }

    // Left and right border
    for y in rect.y + 1..rect.y + rect.height {
        print!(
            "{}║{}║",
            cursor::Goto(rect.x, y),
            cursor::Right(rect.width - 2)
        );
    }

    // Bottom border
    print!(
        "{}╚{}╝",
        cursor::Goto(rect.x, rect.y + rect.height),
        cursor::Right(rect.width - 2)
    );
    for x in rect.x + 1..rect.x + rect.width - 1 {
        print!("{}═", cursor::Goto(x, rect.y + rect.height));
    }

    // Close button
    print!(
        "{}{}[×]",
        cursor::Goto(rect.x + rect.width, rect.y),
        cursor::Left(5)
    );

    // Title bar
    print!(
        "{}{}{}",
        cursor::Goto(rect.x + 2, rect.y + 1),
        title,
        cursor::Goto(rect.x + 1, rect.y + 2)
    );
    for _ in rect.x + 1..rect.x + rect.width - 1 {
        print!("─");
    }
    print!(
        "{}╟{}╢",
        cursor::Goto(rect.x, rect.y + 2),
        cursor::Right(rect.width - 2)
    );

    Rect::new(rect.x + 2, rect.y + 3, rect.width - 4, rect.height - 3)
}

fn window_scroll_bar(rect: Rect) -> Rect {
    for y in rect.y..rect.y + rect.height {
        print!("{}│░", cursor::Goto(rect.x + rect.width - 1, y));
    }

    // Top and bottom T-connectors
    print!(
        "{}╧{}┬",
        cursor::Relative(-2, 1),
        cursor::Goto(rect.x + rect.width - 1, rect.y - 1)
    );

    // Buttons
    print!(
        "{}▲{}▼",
        cursor::Down(1),
        cursor::Goto(rect.x + rect.width, rect.y + rect.height - 1)
    );

    // Draggable bar
    let mid = rect.height / 2;
    print!(
        "{}█",
        cursor::Goto(rect.x + rect.width, rect.y + (mid - mid / 2))
    );

    Rect::new(rect.x, rect.y, rect.width - 2, rect.height)
}
