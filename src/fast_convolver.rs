use crate::ring_buffer::RingBuffer;

pub struct FastConvolver {
    impulse_response: Vec<f32>,
    buffer: RingBuffer<f32>,
    mode: ConvolutionMode,
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
                let buffer_size = impulse_response.len() - 1;  // Adjust based on expected maximum length
                FastConvolver {
                    impulse_response: impulse_response.to_vec(),
                    mode: mode,
                    buffer: RingBuffer::new(buffer_size)
                    
                }
            },
            _ => unimplemented!("Frequency domain is not supported in this example"),
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
                todo!("Implement FrequencyDomain convolution");
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

}
