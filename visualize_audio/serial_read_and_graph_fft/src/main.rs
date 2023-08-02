use std::time::Duration;
use macroquad::prelude::*;

const FRAME_HISTORY_SIZE:usize = 10;
const FRAME_CAPACITY:usize = 256;
const SCALE_FACTOR:f32 = 2.00;

#[macroquad::main("BasicShapes")]
async fn main() {
    let port_name = "/dev/ttyACM0";
    let baud_rate = 57600;

    let mut port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(10))
        .open()
        .unwrap();

    let mut serial_buf = vec![0; 1000];
    

    let mut frame:Vec<i32> = Vec::with_capacity(FRAME_CAPACITY);
    let mut frame_amplitude_history: Vec<Vec<i32>> = Vec::with_capacity(FRAME_CAPACITY * FRAME_HISTORY_SIZE);

    let mut running_avg: Vec<f32> = vec![0.0; FRAME_CAPACITY/2];
    let mut running_avg_complete = false;

    let mut analysis_frames: Vec<Vec<i32>> = Vec::with_capacity(FRAME_CAPACITY * FRAME_HISTORY_SIZE);
    let mut last_std_dev_analysis: Vec<f32> = vec![0.0; FRAME_CAPACITY/2];


    loop {
        let start = std::time::SystemTime::now();
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

            if frame.len() < FRAME_CAPACITY {
                frame.append(&mut int_readings);
            } else {
                let end = std::time::SystemTime::now();
                let runtime = end.duration_since(start).unwrap().as_nanos();
                let sampling_rate =  FRAME_CAPACITY as f32/(runtime as f32 / 1000000000.0);

                let mut samples: [f32; FRAME_CAPACITY] = frame.iter().map(|&f| f as f32).collect::<Vec<f32>>().try_into().unwrap();
                let spectrum = microfft::real::rfft_256(&mut samples);

                // since the real-valued coefficient at the Nyquist frequency is packed into the
                // imaginary part of the DC bin, it must be cleared before computing the amplitudes
                spectrum[0].im = 0.0;
                spectrum[0].re = 0.0;

                // the spectrum has a spike at index `signal_freq`
                let mut amplitudes: Vec<_> = spectrum.iter().map(|c| c.norm() as i32).collect();
                // first reading is trash always, removing it
                amplitudes[0] = 0;

                // we only want to calculate the avg once
                // process_spectral_subtraction(
                //     &mut amplitudes, 
                //     &mut frame_amplitude_history, 
                //     &mut running_avg,
                //     running_avg_complete
                // );

                // filters
                process_std_dev_filter(&mut analysis_frames, &mut last_std_dev_analysis, &mut amplitudes);

                // create frequencies
                let mut max_freq = 0.0;
                let frequencies = (0..FRAME_CAPACITY)
                    .map(|i| {
                        let freq = i as f32 / (FRAME_CAPACITY as f32 * (1.0 / sampling_rate));
                        if freq > max_freq {
                            max_freq = freq;
                        }
                        return freq; 
                    })
                    .collect::<Vec<f32>>();

                let nyquist_freqs = frequencies.iter().cloned().take(frequencies.len() / 2).collect::<Vec<f32>>();

                draw_frame(amplitudes).await;
                frame.clear();
            }
        }
    }
}

async fn draw_frame(frame_readings: Vec<i32>) {
    clear_background(BLACK);
    let mut start_x = 0.0;
    let start_bar = screen_height();

    for r in frame_readings {
        start_x += 2.0 * SCALE_FACTOR;
        let bar_width = 1.0 * SCALE_FACTOR;
        let bar_height = (-r as f32/5.0) * SCALE_FACTOR;
        draw_rectangle(start_x, start_bar, bar_width, bar_height, WHITE);
    }
    next_frame().await
}



// FILTERS
fn process_std_dev_filter(
    analysis_frames: &mut Vec<Vec<i32>>, 
    last_std_dev_analysis: &mut Vec<f32>,
    amplitudes: &mut Vec<i32>
) {
    const STD_DEV_FILTER_LVL:f32 = 285.0;
    // std deviation analysis
    if analysis_frames.len() < FRAME_HISTORY_SIZE {
        analysis_frames.push(amplitudes.clone());
    } else {
        analysis_frames.push(amplitudes.clone());
        filter_std_dev(analysis_frames, last_std_dev_analysis); 
        println!("std dev analysis: {:?}", last_std_dev_analysis);
        analysis_frames.clear();
    }

    *amplitudes = amplitudes
        .iter()
        .zip(last_std_dev_analysis.iter())
        .map(|(&a, &std)| {
            if std > STD_DEV_FILTER_LVL {
                return a;
            }
            return 0;
        })
        .collect::<Vec<i32>>();
}

fn filter_std_dev(analysis_frames: &mut Vec<Vec<i32>>, last_std_dev_analysis: &mut Vec<f32>) {
    for (_io, af) in analysis_frames.iter().enumerate() {
        for (ii, _f) in af.iter().enumerate() {
            let cols = (0..FRAME_HISTORY_SIZE) 
                .map(|r| {
                    let group = &analysis_frames.clone()[r];
                    return group[ii]
                })
                .collect::<Vec<i32>>();

            let mean: i32 = (cols.iter().sum::<i32>()) / cols.len() as i32;

            let mut sum = 0;
            for c in cols {
                sum += (c - mean).pow(2);
            }

            let std_dev = (sum as f32/FRAME_HISTORY_SIZE as f32).sqrt();
            last_std_dev_analysis[ii] = std_dev; // directly index for speed
        }
    } 
}


fn process_spectral_subtraction(
    amplitudes: &mut Vec<i32>,
    frame_amplitude_history: &mut Vec<Vec<i32>>,
    running_avg: &mut Vec<f32>,
    running_avg_complete: bool
) {
    if frame_amplitude_history.len() < FRAME_HISTORY_SIZE {
        frame_amplitude_history.push(amplitudes.clone());
    }

    // calculate the average once...for now, maybe we do it more times later...brute
    // force simple approach ftw
    if frame_amplitude_history.len() == FRAME_HISTORY_SIZE && !running_avg_complete {
        // need to calculate the avg for all ten bins in all ten frames
        for f in frame_amplitude_history.iter() {
            for (ib, _bin) in f.iter().enumerate() {
                let cols = (0..FRAME_HISTORY_SIZE) 
                    .map(|r| {
                        let group = &frame_amplitude_history.clone()[r];
                        return group[ib]
                    })
                    .collect::<Vec<i32>>();

                let avg = cols.iter().map(|&c| c as f32).sum::<f32>() / FRAME_HISTORY_SIZE as f32;
                running_avg[ib] = avg;
            }
        }
    }
}
