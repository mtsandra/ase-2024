use std::{fs::File, io::Write, env, path::Path};

mod ring_buffer;
mod fast_convolver;
use fast_convolver::{FastConvolver, ConvolutionMode};
use ring_buffer::RingBuffer;
use rand::Rng;


fn show_info() {
    eprintln!("MUSI-6106 Assignment Executable");
    eprintln!("(c) 2024 Stephen Garrett & Ian Clester");
}

fn main() {
    show_info();

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

fn load_impulse_response(filename: &str) -> Vec<f32> {
    let mut reader = hound::WavReader::open(Path::new(filename)).unwrap();
    reader.samples::<i16>().map(|s| s.unwrap() as f32 / 32768.0_f32).collect()
}


#[cfg(test)]
mod tests {
    use super::*;

    // Helper to generate a random impulse response of a given length
    fn generate_random_impulse_response(len: usize) -> Vec<f32> {
        let mut rng = rand::thread_rng();
        (0..len).map(|_| rng.gen::<f32>()).collect()
    }

    #[test]
    fn test_identity() {
        let impulse_response = generate_random_impulse_response(51);
        let mut convolver = FastConvolver::new(&impulse_response, ConvolutionMode::TimeDomain);
        let mut input = vec![0.0; 10];
        input[3] = 1.0; // Impulse at index 3
        let mut output = vec![0.0; 10];
        convolver.process(&input, &mut output);

        // Check the output against the impulse response
        for i in 0..10 {
            assert_eq!(output[i], if i >= 3 { impulse_response[i - 3] } else { 0.0 });
        }
    }

    #[test]
    fn test_flush() {
        let impulse_response = generate_random_impulse_response(51);
        let mut convolver = FastConvolver::new(&impulse_response, ConvolutionMode::TimeDomain);
        let mut input = vec![0.0; 10];
        input[3] = 1.0; // Impulse at index 3
        let mut output = vec![0.0; 10];
        convolver.process(&input, &mut output);
        let mut tail = vec![0.0; 50]; // Buffer to catch the reverb tail
        convolver.flush(&mut tail);

        // Validate the reverb tail
        for i in 0..47 { // Check up to the length of impulse_response - 1
            assert_eq!(tail[i], impulse_response[3 + i + 1]);
        }
    }

    #[test]
    fn test_blocksize() {
        let impulse_response = generate_random_impulse_response(51);
        let mut convolver = FastConvolver::new(&impulse_response, ConvolutionMode::TimeDomain);
        let input = vec![1.0; 10000]; // Constant input signal of length 10000
        let block_sizes = [1, 13, 1023, 2048, 1, 17, 5000, 1897];
        let mut output_full = vec![0.0; 10000];

        for &block_size in &block_sizes {
            let mut output = vec![0.0; block_size];
            for (start, chunk) in input.chunks(block_size).enumerate() {
                let output_chunk = &mut output[..chunk.len()];
                convolver.process(chunk, output_chunk);
                output_full[start * block_size..start * block_size + chunk.len()].copy_from_slice(output_chunk);
            }
        }

        // Check if processing in blocks yields the expected consistent output
        assert!(output_full.iter().all(|&sample| (sample - output_full[0]).abs() < f32::EPSILON));
    }
}

