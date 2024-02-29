use std::fs::File;
use hound;
use std::env;

mod ring_buffer;
mod vibrato;
mod lfo;

use crate::vibrato::Vibrato;
/// Show info about the class
fn show_info() {
    eprintln!("MUSI-6106 Assignment Executable");
    eprintln!("(c) 2024 Stephen Garrett & Ian Clester");
}
/// Main function to read file and add vibrato and write to output file
fn main() {
    show_info();

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <input wave filename> <output wave filename>", args[0]);
        return;
    }

    let mut reader = hound::WavReader::open(&args[1]).unwrap();
    let spec = reader.spec();
    let sample_rate = spec.sample_rate as f32;
    let channels = spec.channels as usize;

    let max_delay = 0.005;
    let delay = 0.00;
    let width = 0.005;
    let frequency = 5.0;
    let mut vibrato = Vibrato::new(sample_rate, max_delay, delay, width, frequency, channels);

    let output_file = File::create(&args[2]).unwrap();
    let mut writer = hound::WavWriter::new(output_file, spec).unwrap();

    let block_size = 1024;
    // process each block of samples
    while let Some(buffer) = read_block(&mut reader, block_size, channels) {
        let num_samples = buffer.len() / channels;
        let mut input_samples = vec![0.0; num_samples * channels];
        let mut output_samples = vec![0.0; num_samples * channels];
        input_samples.copy_from_slice(&buffer);

        // add vibrato to each channel
        for ch in 0..channels {
            let input_slice = &mut input_samples[ch * num_samples..(ch + 1) * num_samples];
            let output_slice = &mut output_samples[ch * num_samples..(ch + 1) * num_samples];
            vibrato.process(&mut [input_slice], &mut [output_slice]);
        }

        // write output samples to file
        for i in 0..num_samples {
            for ch in 0..channels {
                let sample = output_samples[i * channels + ch] * i16::MAX as f32;
                writer.write_sample(sample as i16).unwrap();
            }
        }
    }

    writer.finalize().unwrap();
}

/// read a block of samples from the input file
fn read_block(reader: &mut hound::WavReader<std::io::BufReader<File>>, block_size: usize, channels: usize) -> Option<Vec<f32>> {
    let mut buffer = Vec::with_capacity(block_size * channels);
    for _ in 0..block_size * channels {
        match reader.samples::<i16>().next() {
                Some(Ok(sample)) => buffer.push(sample as f32 / i16::MAX as f32),
                _ => {
                    if buffer.is_empty() {
                        return None; 
                    } else {
                        buffer.resize(block_size * channels, 0.0);
                        return Some(buffer); 
                    }
                }
            }
    }
    Some(buffer)
}

