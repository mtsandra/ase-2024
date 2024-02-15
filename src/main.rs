use std::{fs::File, io::Write};
use std::io::BufWriter;
use hound::{WavReader, WavWriter};

mod comb_filter;
use comb_filter::{CombFilter, FilterType, FilterParam};

fn show_info() {
    eprintln!("MUSI-6106 Assignment Executable");
    eprintln!("(c) 2024 Stephen Garrett & Ian Clester");
}

fn main() {
    show_info();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 6 {
        eprintln!("Usage: {} <input wav filename> <output wav filename> <FIR/IIR> <gain> <delay in seconds>", args[0]);
        return
    }

    let input_path = &args[1];
    let output_path = &args[2];
    let filter_type = match args[3].as_str() {
        "FIR" => FilterType::FIR,
        "IIR" => FilterType::IIR,
        _ => {
            eprintln!("Invalid filter type. Choose 'FIR' or 'IIR'.");
            std::process::exit(1);
        },
    };
    let gain: f32 = args[4].parse().unwrap();
    let delay_secs: f32 = args[5].parse().unwrap();


    let input_file = File::open(input_path).unwrap();
    let mut reader = WavReader::new(input_file).unwrap();
    let spec = reader.spec();

    let output_file = File::create(output_path).unwrap();
    let mut writer = WavWriter::new(BufWriter::new(output_file), spec).unwrap();

    let sample_rate_hz = spec.sample_rate as f32;
    let channels = spec.channels as usize;

    let mut comb_filter = CombFilter::new(filter_type, delay_secs, sample_rate_hz, channels);
    comb_filter.set_param(FilterParam::Gain, gain).unwrap();

    let block_size = 1024; 
    let samples_per_block = block_size * channels;

    while let Ok(block) = reader.samples::<i16>().take(samples_per_block).collect::<Result<Vec<_>, _>>() {
        if block.is_empty() {
            break;
        }
        let mut processed_samples = vec![0f32; block.len()];
        let block_samples_f32: Vec<f32> = block.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
    
        for channel in 0..channels {
            let mut channel_samples = Vec::with_capacity(block.len() / channels);
            for (i, sample) in block_samples_f32.iter().enumerate() {
                if i % channels == channel {
                    channel_samples.push(*sample);
                }
            }
    
            let mut processed_channel_samples = vec![0f32; channel_samples.len()];
            comb_filter.process(&[&channel_samples], &mut [&mut processed_channel_samples]);
    
            for (i, &sample) in processed_channel_samples.iter().enumerate() {
                processed_samples[i * channels + channel] = sample;
            }
        }
        for &sample in &processed_samples {
            writer.write_sample((sample * i16::MAX as f32) as i16).unwrap();
        }
    }

    writer.finalize().unwrap();
    println!("File saved");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_sine_wave(frequency: f32, sample_rate: f32, duration_secs: f32) -> Vec<f32> {
        let num_samples = (sample_rate * duration_secs).round() as usize;
        (0..num_samples).map(|i| {
            let t = i as f32 / sample_rate;
            (2.0 * std::f32::consts::PI * frequency * t).sin()
        }).collect()
    }
    fn calculate_rms(signal: &[f32]) -> f32 {
        if signal.is_empty() {
            return 0.0;
        }
    
        let sum_of_squares: f32 = signal.iter().map(|&value| value.powi(2)).sum();
        let mean_of_squares = sum_of_squares / signal.len() as f32;
        mean_of_squares.sqrt()
    }

    #[test]
    fn test1() {

        let input_signal = generate_sine_wave(200.0, 8000.0, 2.0);

        let mut comb_filter = CombFilter::new(FilterType::FIR, 0.0025, 8000.0, 1);

        comb_filter.set_param(FilterParam::Gain, 1.0).unwrap();

        let mut output_signal = vec![0.0; 8000 * 2];
        comb_filter.process(&[&input_signal], &mut [&mut output_signal]);

        assert!(output_signal[8000..].iter().all(|&sample| sample.abs() < 1e-2));
    }
    #[test]
    fn test2() {
        let input_signal = generate_sine_wave(440.0, 8000.0, 1.0);
    
        let mut filter = CombFilter::new(FilterType::IIR, 0.50, 8000.0, 1);
        filter.set_param(FilterParam::Gain, 0.5).unwrap();
    
        let mut output_signal = vec![0.0; input_signal.len()];
        filter.process(&[&input_signal], &mut [&mut output_signal]);
    
        let input_magnitude = calculate_rms(&input_signal);
        let output_magnitude = calculate_rms(&output_signal);
        assert!(output_magnitude / input_magnitude >= 1.0);
    }

    #[test]
    fn test3() {
        // test whether for test 1 if output goes to 0 for different block sizes
        let input_signal = generate_sine_wave(200.0, 8000.0, 2.0); 
        let block_sizes = vec![1024, 2048]; 

        for &block_size in &block_sizes {
            let mut comb_filter = CombFilter::new(FilterType::FIR, 0.0025, 8000.0, 1); 
            comb_filter.set_param(FilterParam::Gain, 1.0).unwrap(); 

            let mut output_signal = Vec::with_capacity((8000.0 * 2.0) as usize); 


            for chunk in input_signal.chunks(block_size) {
                let mut block_output = vec![0f32; chunk.len()]; 
                comb_filter.process(&[chunk], &mut [&mut block_output]); 
                output_signal.extend_from_slice(&block_output); 
            }
            println!("{:?}", &block_size);
            assert!(output_signal[8000..].iter().all(|&sample| sample.abs() < 1e-2));
        }
    }

    #[test]
    fn test4() {
        let sample_rate_hz = 8000.0; 
        let duration_secs = 1.0; 
        let num_samples = (sample_rate_hz * duration_secs) as usize;
        let zero_input_signal = vec![0.0; num_samples]; 

        let mut fir_filter = CombFilter::new(FilterType::FIR, 0.0025, sample_rate_hz, 1);
        fir_filter.set_param(FilterParam::Gain, 1.0).unwrap();

        let mut fir_output_signal = vec![0.0; num_samples];
        fir_filter.process(&[&zero_input_signal], &mut [&mut fir_output_signal]);

        assert!(fir_output_signal.iter().all(|&sample| sample == 0.0));

        let mut iir_filter = CombFilter::new(FilterType::IIR, 0.0025, sample_rate_hz, 1);
        iir_filter.set_param(FilterParam::Gain, 1.0).unwrap();

        let mut iir_output_signal = vec![0.0; num_samples];
        iir_filter.process(&[&zero_input_signal], &mut [&mut iir_output_signal]);

        assert!(iir_output_signal.iter().all(|&sample| sample == 0.0));
    }    

    #[test]
    //testing for impulse response
    fn test5() {
        let sample_rate_hz = 8000.0;
        let impulse_position = 0; 
        let mut impulse_signal = vec![0.0; 8000]; 
        impulse_signal[impulse_position] = 1.0; 

        let mut fir_filter = CombFilter::new(FilterType::FIR, 0.0025, sample_rate_hz, 1);
        fir_filter.set_param(FilterParam::Gain, 1.0).unwrap();

        let mut fir_output_signal = vec![0.0; impulse_signal.len()];
        fir_filter.process(&[&impulse_signal], &mut [&mut fir_output_signal]);

        assert!(fir_output_signal.iter().any(|&sample| sample != 0.0));

        let mut iir_filter = CombFilter::new(FilterType::IIR, 0.0025, sample_rate_hz, 1);
        iir_filter.set_param(FilterParam::Gain, 0.5).unwrap(); 

        let mut iir_output_signal = vec![0.0; impulse_signal.len()];
        iir_filter.process(&[&impulse_signal], &mut [&mut iir_output_signal]);
        assert!(iir_output_signal.iter().skip(impulse_position + 1).any(|&sample| sample != 0.0));
    }





}
