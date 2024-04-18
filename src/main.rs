use std::{fs::File, io::Write, env, path::Path};

use std::time::Instant;
mod ring_buffer;
mod fast_convolver;
use fast_convolver::{FastConvolver, ConvolutionMode};
use ring_buffer::RingBuffer;
use rand::Rng;


fn show_info() {
    eprintln!("MUSI-6106 Assignment Executable");
    eprintln!("(c) 2024 Stephen Garrett & Ian Clester");
}

fn main_time() {

    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: {} <input wave filename> <output wave filename> <impulse response filename>", args[0]);
        return;
    }

    // Load impulse response from a file
    let impulse_response = load_impulse_response(&args[3]);

    // Create an instance of FastConvolver
    let mut convolver = FastConvolver::new(&impulse_response, ConvolutionMode::TimeDomain);

    // Read input wave file
    let mut reader = hound::WavReader::open(&args[1]).unwrap();
    let spec = reader.spec();
    let channels = spec.channels;
    assert!(channels == 1, "Only mono audio input is supported.");

    // Output will also be a WAV file
    let mut writer = hound::WavWriter::create(&args[2], spec).unwrap();

    // Define block size and create buffers
    let block_size = 1024; // or whatever is suitable based on the application
    let mut input_buffer = vec![0.0_f32; block_size];
    let mut output_buffer = vec![0.0_f32; block_size];

    // Process audio in blocks
    let mut sample_iter = reader.samples::<i16>().map(|s| s.unwrap() as f32 / 32768.0_f32);
    loop {
        let mut count = 0;
        for sample in input_buffer.iter_mut() {
            if let Some(s) = sample_iter.next() {
                *sample = s;
                count += 1;
            } else {
                break;
            }
        }

        if count == 0 {
            break;
        }

        convolver.process(&input_buffer[..count], &mut output_buffer[..count]);

        for &sample in &output_buffer[..count] {
            writer.write_sample((sample * 32768.0).round() as i16).unwrap();
        }

        if count < block_size {
            break;
        }
    }

    // Flush any remaining samples
    writer.finalize().unwrap();

}

fn main_freq() {


    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: {} <input wave filename> <output wave filename> <impulse response filename>", args[0]);
        return;
    }

    // Load impulse response from a file
    let impulse_response = load_impulse_response(&args[3]);

    // Create an instance of FastConvolver
    let mut convolver = FastConvolver::new(&impulse_response, ConvolutionMode::TimeDomain);

    // Read input wave file
    let mut reader = hound::WavReader::open(&args[1]).unwrap();
    let spec = reader.spec();
    let channels = spec.channels;
    assert!(channels == 1, "Only mono audio input is supported.");

    // Output will also be a WAV file
    let mut writer = hound::WavWriter::create(&args[2], spec).unwrap();

    // Define block size and create buffers
    let block_size = 1024; // or whatever is suitable based on the application
    let mut input_buffer = vec![0.0_f32; block_size];
    let mut output_buffer = vec![0.0_f32; block_size];

    // Process audio in blocks
    let mut sample_iter = reader.samples::<i16>().map(|s| s.unwrap() as f32 / 32768.0_f32);
    loop {
        let mut count = 0;
        for sample in input_buffer.iter_mut() {
            if let Some(s) = sample_iter.next() {
                *sample = s;
                count += 1;
            } else {
                break;
            }
        }

        if count == 0 {
            break;
        }

        convolver.process(&input_buffer[..count], &mut output_buffer[..count]);

        for &sample in &output_buffer[..count] {
            writer.write_sample((sample * 32768.0).round() as i16).unwrap();
        }

        if count < block_size {
            break;
        }
    }

    // Flush any remaining samples
    writer.finalize().unwrap();

}

fn main() {
    show_info();
    let start_time = Instant::now();
    main_time();
    let time_duration = start_time.elapsed();

    let start_freq = Instant::now();
    main_freq();
    let freq_duration = start_freq.elapsed();

    println!("Time domain function took: {:?}", time_duration);
    println!("Frequency domain function took: {:?}", freq_duration);
}

fn load_impulse_response(filename: &str) -> Vec<f32> {
    let mut reader = hound::WavReader::open(Path::new(filename)).unwrap();
    reader.samples::<i16>().map(|s| s.unwrap() as f32 / 32768.0_f32).collect()
}





#[cfg(test)]
mod tests {
    use super::*;

    fn generate_random_impulse_response(len: usize) -> Vec<f32> {
        let mut rng = rand::thread_rng();
        (0..len).map(|_| rng.gen::<f32>()).collect()
    }

    #[test]
    fn test_identity_time() {
        let impulse_response = generate_random_impulse_response(51);
        let mut convolver = FastConvolver::new(&impulse_response, ConvolutionMode::TimeDomain);
        let mut input = vec![0.0; 10];
        input[3] = 1.0; 
        let mut output = vec![0.0; 10];
        convolver.process(&input, &mut output);

        for i in 0..10 {
            assert_eq!(output[i], if i >= 3 { impulse_response[i - 3] } else { 0.0 });
        }
    }

    #[test]
    fn test_flush_time() {
        let impulse_response = generate_random_impulse_response(51);
        let mut convolver = FastConvolver::new(&impulse_response, ConvolutionMode::TimeDomain);
        let mut input = vec![0.0; 10];
        input[3] = 1.0; 
        let mut output = vec![0.0; 10];
        convolver.process(&input, &mut output);
        let mut tail = vec![0.0; 50]; 
        convolver.flush(&mut tail);

        // Validate the reverb tail
        for i in 0..44 { 
            assert_eq!(tail[i], impulse_response[output.len()-3+i]);
        }
        for i in 45..50 { 
            assert_eq!(tail[i], 0.0);
        }
    }

    #[test]
    fn test_blocksize_time() {
        let impulse_response = generate_random_impulse_response(51);
        let mut convolver = FastConvolver::new(&impulse_response, ConvolutionMode::TimeDomain);
        let mut input = vec![0.0; 10];
        input[3] = 1.0; 
        let block_sizes = [1, 13, 1023, 2048, 1, 17, 5000, 1897];
        let mut output_full = vec![0.0; 10000];

        for &block_size in &block_sizes {
            for (i, chunk) in input.chunks(block_size).enumerate() {
                let mut output = vec![0.0; chunk.len()];
                convolver.process(chunk, &mut output);
                for (j, &sample) in output.iter().enumerate() {
                    output_full[i * block_size + j] = sample;
                }
            }
        }


        for i in 0..10 {
            assert_eq!(output_full[i], if i >= 3 { impulse_response[i - 3] } else { 0.0 });
        }
    }

    #[test]
    fn test_overlap_add_time(){
        let impulse_response = generate_random_impulse_response(51);
        let mut convolver = FastConvolver::new(&impulse_response, ConvolutionMode::TimeDomain);
        let mut input = vec![0.0; 10];
        input[3] = 1.0; 
        let mut output = vec![0.0; 10];
        let block_size = 5;
        let full_output = convolver.overlap_add(&input, &mut output, block_size);

        for i in 0..10 {
            assert_eq!(full_output[i], if i >= 3 { impulse_response[i - 3] } else { 0.0 });
        }
    }


    #[test]
    fn test_identity_freq() {
        let impulse_response = generate_random_impulse_response(52);
        let mut convolver = FastConvolver::new(&impulse_response, ConvolutionMode::FrequencyDomain{block_size: 2});
        let mut input = vec![0.0; 10];
        let epsilon = 1e-5;
        println!("impulse r {:?} ", impulse_response);
        input[3] = 1.0; 
        let mut output = vec![0.0; 10];
        convolver.process(&input, &mut output);
        println!("output {:?} ", output);


        for i in 0..10 {
            assert!(
                (output[i] - if i >= 3 { impulse_response[i - 3] } else { 0.0 }).abs() <= epsilon,
                "Values at index {} are not within epsilon: {} != {}", 
                i, 
                output[i], 
                if i >= 3 { impulse_response[i - 3] } else { 0.0 }
            );
        }
    }

    #[test]
    fn test_flush_freq() {
        let impulse_response = generate_random_impulse_response(51);
        let mut convolver = FastConvolver::new(&impulse_response, ConvolutionMode::FrequencyDomain{block_size: 8});
        let mut input = vec![0.0; 10];
        input[3] = 1.0; 
        let mut output = vec![0.0; 10];
        convolver.process(&input, &mut output);
        let mut tail = vec![0.0; 50]; 
        convolver.flush(&mut tail);

        // Validate the reverb tail
        for i in 0..44 { 
            assert_eq!(tail[i], impulse_response[output.len()-3+i]);
        }
        for i in 45..50 { 
            assert_eq!(tail[i], 0.0);
        }
    }

    #[test]
    fn test_blocksize_freq() {
        let impulse_response = generate_random_impulse_response(51);
        let mut convolver = FastConvolver::new(&impulse_response, ConvolutionMode::FrequencyDomain{block_size: 8});
        let mut input = vec![0.0; 10];
        input[3] = 1.0; 
        let block_sizes = [1, 13, 1023, 2048, 1, 17, 5000, 1897];
        let mut output_full = vec![0.0; 10000];

        for &block_size in &block_sizes {
            for (i, chunk) in input.chunks(block_size).enumerate() {
                let mut output = vec![0.0; chunk.len()];
                convolver.process(chunk, &mut output);
                for (j, &sample) in output.iter().enumerate() {
                    output_full[i * block_size + j] = sample;
                }
            }
        }

        // Check the output against the impulse response
        for i in 0..10 {
            assert_eq!(output_full[i], if i >= 3 { impulse_response[i - 3] } else { 0.0 });
        }
    }

}