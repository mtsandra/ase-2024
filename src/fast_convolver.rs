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
                    mode,
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





}
