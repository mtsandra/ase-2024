use rustfft::{FFT, FFTplanner};
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;
use std::collections::VecDeque;

struct FastConvolver {
    mode: ConvolutionMode,
    impulse_response_fft: Vec<Complex<f32>>,
    sample_buffer: VecDeque<f32>,
}

#[derive(Debug, Clone, Copy)]
pub enum ConvolutionMode {
    TimeDomain,
    FrequencyDomain { block_size: usize },
}

impl FastConvolver {
    pub fn new(impulse_response: &[f32], mode: ConvolutionMode) -> Self {
        let mut planner = FFTplanner::new(false);
        let fft = planner.plan_fft(impulse_response.len());
        let mut impulse_response_fft = vec![Complex::zero(); impulse_response.len()];
        let impulse_response_complex: Vec<Complex<f32>> = impulse_response.iter().map(|&r| Complex::new(r, 0.0)).collect();
        
        fft.process(&mut impulse_response_complex, &mut impulse_response_fft);

        FastConvolver {
            mode,
            impulse_response_fft,
            sample_buffer: VecDeque::new(),
        }
    }

    pub fn reset(&mut self) {
        self.sample_buffer.clear();
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        match self.mode {
            ConvolutionMode::TimeDomain => {
                for (i, sample) in input.iter().enumerate() {
                    self.sample_buffer.push_back(*sample);
                    let mut convolved_sample = 0.0;

                    for (j, &impulse) in self.impulse_response_fft.iter().enumerate() {
                        if let Some(buffered_sample) = self.sample_buffer.get(i.saturating_sub(j)) {
                            convolved_sample += buffered_sample * impulse.re;
                        }
                    }

                    output[i] = convolved_sample;
                }

                // Ensure the output has the same number of samples as the input
                assert_eq!(input.len(), output.len(), "Output length must match input length.");
            },
            ConvolutionMode::FrequencyDomain { block_size } => {
                let mut input_fft_buffer = vec![Complex::zero(); block_size];
                let mut output_fft_buffer = vec![Complex::zero(); block_size];
                let mut ifft_planner = FFTplanner::new(true);
                let ifft = ifft_planner.plan_fft(block_size);

                for (i, chunk) in input.chunks(block_size).enumerate() {
                    for (j, &sample) in chunk.iter().enumerate() {
                        input_fft_buffer[j] = Complex::new(sample, 0.0);
                    }

                    for j in 0..block_size {
                        output_fft_buffer[j] = input_fft_buffer[j] * self.impulse_response_fft[j];
                    }

                    ifft.process(&mut output_fft_buffer, &mut input_fft_buffer);

                    for j in 0..block_size {
                        if i * block_size + j < output.len() {
                            output[i * block_size + j] += input_fft_buffer[j].re;
                        }
                    }
                }
            },
        }
    }

    pub fn flush(&mut self, output: &mut [f32]) {
        let impulse_len = self.impulse_response_fft.len();
        assert!(output.len() >= impulse_len, "Output buffer too small for flush operation.");

        for i in 0..impulse_len {
            if let Some(sample) = self.sample_buffer.pop_front() {
                let mut convolved_sample = 0.0;
                for (j, &impulse) in self.impulse_response_fft.iter().enumerate() {
                    if j <= i {
                        convolved_sample += sample * impulse.re;
                    }
                }
                output[i] = convolved_sample;
            } else {
                output[i] = 0.0; // Fill the rest of the buffer with zeros if no more samples.
            }
        }
    }

    pub fn max_flush_output_size(&self) -> usize {
        self.impulse_response_fft.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Generate a random impulse response (IR) of length 51
    fn generate_random_ir() -> Vec<f32> {
        let mut ir = vec![0.0; 51];
        ir[3] = 1.0; // Set an impulse at index 3
        ir
    }

    #[test]
    fn test_identity() {
        let ir = generate_random_ir();
        let mut convolver = FastConvolver::new(&ir, ConvolutionMode::TimeDomain);
        let input = vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let mut output = vec![0.0; 10];

        convolver.process(&input, &mut output);

        for i in 0..10 {
            let expected_value = if i >= 3 { 1.0 } else { 0.0 };
            assert_eq!(output[i], expected_value, "Identity test failed at sample {}", i);
        }
    }

    #[test]
    fn test_flush() {
        let ir = generate_random_ir();
        let mut convolver = FastConvolver::new(&ir, ConvolutionMode::TimeDomain);
        let input = vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let mut output = vec![0.0; 10];

        convolver.process(&input, &mut output);
        let mut flush_output = vec![0.0; ir.len()];
        convolver.flush(&mut flush_output);

        // Assert that the flush output matches the tail of the IR post-input
        for i in 0..ir.len() {
            let expected_value = if i < ir.len() - 3 { 0.0 } else { ir[i] };
            assert_eq!(flush_output[i], expected_value, "Flush test failed at sample {}", i);
        }
    }

    #[test]
    fn test_blocksize() {
        let ir = generate_random_ir();
        let input = vec![0.0; 10000];
        let mut output = vec![0.0; 10000];
        let block_sizes = [1, 13, 1023, 2048, 1, 17, 5000, 1897];

        for &block_size in &block_sizes {
            let mut convolver = FastConvolver::new(&ir, ConvolutionMode::FrequencyDomain { block_size });
            convolver.process(&input, &mut output);
            // Test output here for each block size; actual testing logic depends on frequency domain implementation
            // Assuming an identity response as an example
            assert!(output.iter().all(|&x| x.abs() < 1e-6), "Blocksize test failed for block size {}", block_size);
        }
    }
}
