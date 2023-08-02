use std::time::Duration;
use macroquad::prelude::*;

const SCALE_FACTOR:f32 = 0.75;

#[macroquad::main("BasicShapes")]
async fn main() {
    let port_name = "/dev/ttyACM0";
    let baud_rate = 57600;

    let mut port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(10))
        .open()
        .unwrap();

    let mut serial_buf = vec![0; 1000];
    
    const FRAME_CAPACITY:usize = 300;
    let mut frame:Vec<i32> = Vec::with_capacity(FRAME_CAPACITY);

    loop {
        let ser_out = port.read(serial_buf.as_mut_slice());
        if let Ok(so) = ser_out {
            let readings = &serial_buf[..so];

            let mut int_readings = readings
                .split(|&r| r == 10)
                .filter(|r| r.len() == 4)
                .map(|r| {
                    if r.len() == 0 {
                        panic!("length zero");
                    }

                    let str = std::str::from_utf8(&r[..3]).unwrap();
                    let real = str.trim().parse::<i32>().unwrap();
                    return real;
                })
                .collect::<Vec<i32>>();

            let ir_len = int_readings.len();
            let frame_len = frame.len();
            let frame_cap = FRAME_CAPACITY - frame_len;

            // if there is room in our frame, then just add
            if ir_len <= frame_cap {
                frame.append(&mut int_readings);
            } else {
                // otherwise, we need to determine how much we can add
                let diff = ir_len - frame_cap;
                let mut slice_start = int_readings[..diff - 1].to_vec();
                let mut slice_remainder = int_readings[diff..].to_vec();
                frame.append(&mut slice_start);

                // now that frame is full, we can draw it
                draw_frame(frame.clone()).await;

                // empty frame
                frame.clear();
                // add in the remainder
                frame.append(&mut slice_remainder);
            }
        }
    }
}

async fn draw_frame(frame_readings: Vec<i32>) {
    clear_background(BLACK);
    let mut start_x = 0.0;
    let start_bar = screen_height() - 150.0;

    for r in frame_readings {
        start_x += 5.0 * SCALE_FACTOR;
        let bar_width = 3.0 * SCALE_FACTOR;
        let bar_height = (-r as f32/5.0) * SCALE_FACTOR;
        draw_rectangle(start_x, start_bar, bar_width, bar_height, WHITE);
    }
    next_frame().await
}
