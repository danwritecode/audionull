use std::convert::TryInto;
use std::time::Duration;
use plotters::prelude::*;

fn main() {
    let port_name = "/dev/ttyACM0";
    let baud_rate = 57600;

    let mut port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(10))
        .open()
        .unwrap();

    let mut serial_buf = vec![0; 1000];
    
    const FRAME_CAPACITY:usize = 1024;
    let mut frame:Vec<i32> = Vec::with_capacity(FRAME_CAPACITY);

    let start = std::time::SystemTime::now();

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

            if frame.len() < FRAME_CAPACITY {
                frame.append(&mut int_readings);
            } else {
                let end = std::time::SystemTime::now();
                let runtime = end.duration_since(start).unwrap().as_nanos();
                let sampling_rate =  FRAME_CAPACITY as f32/(runtime as f32 / 1000000000.0);

                println!("sampling rate: {}", sampling_rate);

                let mut samples: [f32; 1024] = frame.iter().map(|&f| f as f32).collect::<Vec<f32>>().try_into().unwrap();
                let spectrum = microfft::real::rfft_1024(&mut samples);

                // since the real-valued coefficient at the Nyquist frequency is packed into the
                // imaginary part of the DC bin, it must be cleared before computing the amplitudes
                spectrum[0].im = 0.0;

                // the spectrum has a spike at index `signal_freq`
                let mut amplitudes: Vec<_> = spectrum.iter().map(|c| c.norm() as u32).collect();

                // first reading is trash always, removing it
                amplitudes[0] = 0;

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

                plot(nyquist_freqs, amplitudes, max_freq/2.0, 50000.0).unwrap();
                panic!("STOP");
            }
        }
    }
}


fn plot(frequencies: Vec<f32>, amplitudes: Vec<u32>, max_freq:f32, max_mag:f32) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("plot_sine_400hz.png", (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .margin(5)
        .x_label_area_size(50)
        .y_label_area_size(75)
        .build_cartesian_2d(0.0..max_freq, 0.0..max_mag)?;

    let zipped = frequencies.iter().zip(amplitudes.iter()).map(|(&x, &y)| (x, y as f32)).collect::<Vec<(f32,f32)>>();
    println!("freq last: {:?}", frequencies.last().unwrap());
    println!("amp last: {:?}", amplitudes.last().unwrap());
    println!("freq len: {:?}", frequencies.len());
    println!("amp len: {:?}", amplitudes.len());
    println!("zipped last: {:?}", zipped.last().unwrap());

    chart.configure_mesh().draw()?;
    chart.draw_series(LineSeries::new(
        frequencies.iter().zip(amplitudes.iter()).map(|(&x, &y)| (x, y as f32)),
        &RED,
    ))?;

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;

    Ok(())
}
