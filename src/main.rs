use minifb::{Window, WindowOptions, Key};
use std::time::{SystemTime, UNIX_EPOCH};

mod chip8;

fn main() {
    let mut chip = chip8::Chip8::new();

    let mut buffer: Vec<u32> = vec![0; 64 * 32];

    let mut window = Window::new(
        "Test - ESC to exit",
        1280,
        640,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    chip.load_rom();

    let mut start = SystemTime::now();

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    loop {

        let mut keys_pressed = [false; 16];
        match window.get_keys() {
            None => {},
            Some(keys) => {
                for key in keys {
                    match key {
                        Key::Key1 => {
                            keys_pressed[0] = true;
                        },
                        Key::Key2 => {
                            keys_pressed[1] = true;
                        },
                        Key::Key3 => {
                            keys_pressed[2] = true;
                        },
                        Key::Key4 => {
                            keys_pressed[3] = true;
                        },
                        Key::Q => {
                            keys_pressed[4] = true;
                        },
                        Key::W => {
                            keys_pressed[5] = true;
                        },
                        Key::E => {
                            keys_pressed[6] = true;
                        },
                        Key::R => {
                            keys_pressed[7] = true;
                        },
                        Key::A => {
                            keys_pressed[8] = true;
                        },
                        Key::S => {
                            keys_pressed[9] = true;
                        },
                        Key::D => {
                            keys_pressed[10] = true;
                        },
                        Key::F => {
                            keys_pressed[11] = true;
                        },
                        Key::Y => {
                            keys_pressed[12] = true;
                        },
                        Key::X => {
                            keys_pressed[13] = true;
                        },
                        Key::C => {
                            keys_pressed[14] = true;
                        },
                        Key::V => {
                            keys_pressed[15] = true;
                        },
                        _ => {}
                    }
                }
            },
        }

        let r = chip.execute_instruction(&keys_pressed);

        if r.video_changed {
            start = SystemTime::now();
            let mut c = 0;
            for i in buffer.iter_mut() {
                let x = chip.get_pixel(c);
                if x > 0 {
                    *i = 0xFFFFFF;
                } else {
                    *i = 0;
                }
                c += 1;
            }

            let since_the_epoch = SystemTime::now()
                .duration_since(start)
                .expect("Time went backwards");
            println!("{:?}", since_the_epoch);

            // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
            window
                .update_with_buffer(&buffer, 64, 32)
                .unwrap();
        }

    }

}

