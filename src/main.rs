mod chip8;

extern crate gio;
extern crate glib;
extern crate gtk;

use std::env::args;

use gio::prelude::*;
use gtk::prelude::*;
use gtk::Builder;
use std::rc::Rc;
use std::cell::RefCell;
use crate::chip8::Chip8;

pub fn build_ui(application: &gtk::Application, video: Rc<RefCell<Chip8>>) {
    let glade_src = include_str!("text_viewer.glade");
    let builder = Builder::new();
    builder
        .add_from_string(glade_src)
        .expect("Couldn't add from string");

    let window: gtk::ApplicationWindow = builder.get_object("window").expect("Couldn't get window");
    window.set_application(Some(application));

    let canvas : gtk::DrawingArea = builder.get_object("canvas").expect("Couldn't get builder");
    let pixel_size : f64 = 10 as f64;

    canvas.connect_draw(move |_w, c|{
        println!("{}", video.borrow().test_flag);
        for y in 0..32 {
            for x in 0..64 {
                if video.borrow().get_pixelc(x,y) > 0 {
                    c.set_source_rgb(1 as f64, 1 as f64, 1 as f64);
                } else {
                    c.set_source_rgb(0 as f64, 0 as f64, 0 as f64);
                }
                c.rectangle(x as f64 * pixel_size, y as f64 * pixel_size, pixel_size, pixel_size);
                c.fill();
            }
        }
        gtk::Inhibit(false)
    });

    window.show_all();
}


fn main() {
    let chip = Rc::new(RefCell::new(Chip8::new()));
    let application = gtk::Application::new(
        Some("com.github.waterfl0w.chip8plus1"),
        Default::default(),
    ).expect("Initialization failed...");

    application.connect_activate(move |app| {
        let a = chip.clone();
        build_ui(app, a);
    });
    (*chip.borrow()).load_rom();

    println!("updated test flag");

    application.run(&args().collect::<Vec<_>>());
}

