extern crate wingui;

fn main() {
    wingui::start(
        "WinGUI Test Window", 
        |b| b.on_create(|_| println!("Window opened!"))
    );
}
