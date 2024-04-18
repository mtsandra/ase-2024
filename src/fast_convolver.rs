use crate::ring_buffer::RingBuffer;

use rustfft::{num_complex::Complex, num_traits::Zero, Fft, FftPlanner};


pub struct FastConvolver {
    impulse_response: Vec<f32>,
    buffer: RingBuffer<f32>,
    mode: ConvolutionMode,
    // block_size: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum ConvolutionMode {
    TimeDomain,
    FrequencyDomain { block_size: usize },
}

impl FastConvolver {
    // Creates a new FastConvolver
    pub fn new(impulse_response: &[f32], mode: ConvolutionMode) -> Self {
        match mode {
            ConvolutionMode::TimeDomain => {
                let buffer_size = impulse_response.len() - 1; 
                FastConvolver {
                    impulse_response: impulse_response.to_vec(),
                    mode,
                    buffer: RingBuffer::new(buffer_size),
                    // block_size: block_size
                    
                }
            },
            ConvolutionMode::FrequencyDomain { block_size } => {
                let buffer_size = impulse_response.len() - 1; 
                FastConvolver {
                    impulse_response: impulse_response.to_vec(),
                    mode,
                    buffer: RingBuffer::new(buffer_size),
                    // block_size: block_size
                }
            
            }
        }
    }

    // Resets the convolver
    pub fn reset(&mut self) {
        self.buffer.reset();

    }

    // Processes the input and performs convolution
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {

        match self.mode {
            ConvolutionMode::TimeDomain => {self.time_domain_convolution(input, output)}
            ConvolutionMode::FrequencyDomain { block_size } => {
                // To be implemented based on requirements
                self.overlap_add_freq(input, output, block_size);
            }
    }

    }

    // Sync the flush output tail to the buffer that stores the tail
    pub fn flush (&mut self, output: &mut [f32]) {

        for i in 0..output.len() {
            output[i] = self.buffer.pop();
        }
        
        
    }


    fn time_domain_convolution (&mut self, input: &[f32], output: &mut [f32]) {
        let mut full_output = vec![0.0; output.len() + self.impulse_response.len() - 1];
        // compute the convolution
        for i in 0..full_output.len() {
            for j in 0..self.impulse_response.len() {
                if i >= j && i - j < input.len() {
                    full_output[i] += input[i - j] * self.impulse_response[j];
                    
                }
            }
        }
        // copy the output to the output vector
        output.copy_from_slice(&full_output[..output.len()]);

        // update the buffer

        for i in input.len()..full_output.len() {
            self.buffer.push(full_output[i]);
        }



    }

    fn block_signals(&mut self, mut input: Vec<f32>, block_size: usize) -> Vec<Vec<f32>> {
        let length = input.len();
        let num_blocks = if length % block_size == 0 { length / block_size } else { length / block_size + 1 };
        let mut blocks =  Vec::new();
    
        for i in 0..num_blocks {
            let start = i * block_size;
            let end = std::cmp::min((i + 1) * block_size, length);
            blocks.push(input[start..end].to_vec());
        }
    
        blocks
    }
    
    pub fn overlap_add_freq(&mut self, input: &[f32], output: &mut [f32], block_size: usize) {
        let input_blocks = self.block_signals(input.to_vec(), block_size);
        let ir_blocks = self.block_signals(self.impulse_response.clone(), block_size);
        let mut full_output = vec![0.0; output.len() + self.impulse_response.len() - 1];
        for (i, input_block) in input_blocks.iter().enumerate() {
            for (j, ir_block )in ir_blocks.iter().enumerate() {

                self.impulse_response = ir_block.clone();
                let mut block_convolution = vec![0.0; block_size];
                self.fft_based_convolution(input_block, &mut block_convolution);
                // println!("block_convolution: {:?}", block_convolution);
                let output_begin_index = i*input_block.len() + j*ir_block.len();
                let output_end_index = output_begin_index + input_block.len() - 1;
                // add the output up to block size
                for s in 0..(input_block.len()-1) {
                    full_output[output_begin_index+s] += block_convolution[s];
                }
                // println!("full_output: {:?}", full_output);
                // add the reverb tail
                self.flush(&mut full_output[output_end_index..output_end_index + self.impulse_response.len() - 1]);
            }

        }

        output.copy_from_slice(&full_output[..output.len()]);
    
    }
    // calls self.process instead, for general use
    pub fn overlap_add(&mut self, input: &[f32], output: &mut [f32], block_size: usize) -> Vec<f32>{
        let input_blocks = self.block_signals(input.to_vec(), block_size);
        let ir_blocks = self.block_signals(self.impulse_response.clone(), block_size);
        let mut full_output = vec![0.0; output.len() + self.impulse_response.len() - 1];
        for (i, input_block) in input_blocks.iter().enumerate() {
            for (j, ir_block )in ir_blocks.iter().enumerate() {
                self.impulse_response = ir_block.clone();
                let mut block_convolution = vec![0.0; block_size];
                self.process(input_block, &mut block_convolution);
                let output_begin_index = i*block_size + j*block_size;
                let output_end_index = output_begin_index + block_size;
                // add the output up to block size
                for s in 0..block_size {
                    full_output[output_begin_index+s] += block_convolution[s];
                }
                // add the reverb tail
                self.flush(&mut full_output[output_end_index..output_end_index + self.impulse_response.len() - 1]);
            }

        }
        full_output
    
    }

    pub fn fft_based_convolution(&mut self, input: &[f32], output: &mut [f32]) {
        let n = input.len() + self.impulse_response.len() - 1;
        let mut input_padded: Vec<Complex<f32>> = vec![Complex::zero(); n];
        let mut ir_padded: Vec<Complex<f32>> = vec![Complex::zero(); n];

        for i in 0..input.len() {
            input_padded[i] = Complex::new(input[i], 0.0);
        }
        for i in 0..self.impulse_response.len() {
            ir_padded[i] = Complex::new(self.impulse_response[i], 0.0);
        }

        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(n);
        let ifft = planner.plan_fft_inverse(n);
        // println!("input_padded: {:?}", input_padded);
        // println!("ir_padded: {:?}", ir_padded);

        fft.process(&mut input_padded);
        fft.process(&mut ir_padded);
        // println!("AFTER FFT input_padded: {:?}", input_padded);
        // println!("AFTER FFT ir_padded: {:?}", ir_padded);

        let mut fft_output: Vec<Complex<f32>> = input_padded.iter().zip(ir_padded.iter()).map(|(a, b)| a * b).collect();

        ifft.process(&mut fft_output);
        // println!("AFTER IFFT fft_output: {:?}", fft_output);

        // normalize and extract real part
        let fft_output_re: Vec<f32> = fft_output.iter().map(|x| x.re / n as f32).collect();
        // println!("fft_output_re: {:?}", fft_output_re);
        // if input.len() < self.block_size {
        //     output.copy_from_slice(&fft_output_re[..input.len()]);
        
        // } else {
        //     output.copy_from_slice(&fft_output_re[..output.len()]);
        // }
        output.copy_from_slice(&fft_output_re[..input.len()]);
        // println!("OUTPUT FFT: {:?}", output);
        

        for i in input.len()..fft_output_re.len() {
            self.buffer.push(fft_output_re[i]);
        }
        // println!("buffer: {:?}", self.buffer.peek());


    }




}
