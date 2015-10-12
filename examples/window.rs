extern crate wingui;

use wingui::*;
use wingui::text::*;

fn main() {
    wingui::start(
        "WinGUI Test Window", 
        |b| b.on_create(|window| { 
            Label::builder(window, ())
                .text("Hello, world!")
                .build();
            }
        )
    );
}
