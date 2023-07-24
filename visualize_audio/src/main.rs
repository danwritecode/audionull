use std::time::Duration;
use macroquad::prelude::*;

const SCALE_FACTOR:f32 = 1.0;

#[macroquad::main("BasicShapes")]
async fn main() {
    let port_name = "/dev/ttyACM0";
    let baud_rate = 57600;

    let mut port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(10))
        .open()
        .unwrap();

    let mut serial_buf = vec![0; 1000];
    
    const FRAME_CAPACITY:usize = 250;
    let mut frame = vec![];

    loop {
        let ser_out = port.read(serial_buf.as_mut_slice());
        if let Ok(so) = ser_out {
            let reading = &serial_buf[..so];

            let mut int_readings = reading
                .split(|&r| r == 10)
                .filter(|r| r.len() == 3)
                .map(|r| {
                    let str = std::str::from_utf8(r).unwrap();
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
                frame = vec![];
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
    let start_line = screen_height() - 250.0;

    for r in frame_readings {
        start_x += 5.0 * SCALE_FACTOR;

        let line_y_pos = (start_line - r as f32) * SCALE_FACTOR;
        let line_pixel_size_x_y = 3.0 * SCALE_FACTOR;
        draw_rectangle(start_x, line_y_pos, line_pixel_size_x_y, line_pixel_size_x_y, WHITE);

        let bar_width = 3.0 * SCALE_FACTOR;
        let bar_height = (-r as f32/5.0) * SCALE_FACTOR;
        draw_rectangle(start_x, start_bar, bar_width, bar_height, WHITE);
    }
    next_frame().await
}
