//! # Complex sequences.
//! src/main.rs  

mod structures;
mod support;
mod gui;
mod fractals;

/// Basic interactivity entry point.
fn main() {
    println!("# Complex sequences.");
 
    // We use closure preset. 
    gui::defaults::launch_default();

    eprintln!("(?) main::main() End !");
}
