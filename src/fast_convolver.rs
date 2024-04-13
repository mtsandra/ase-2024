
use crate::ring_buffer::RingBuffer;

pub struct FastConvolver {
    impulse_response: Vec<f32>,
    ring_buffer: RingBuffer<f32>,
    mode: ConvolutionMode,
}

#[derive(Debug, Clone, Copy)]
pub enum ConvolutionMode {
    TimeDomain,
    FrequencyDomain { block_size: usize },
}

impl FastConvolver {
    // Initialize the FastConvolver with an impulse response and mode
    pub fn new(impulse_response: &[f32], mode: ConvolutionMode) -> Self {
        let buffer_size = match mode {
            ConvolutionMode::TimeDomain => impulse_response.len(),
            ConvolutionMode::FrequencyDomain { block_size } => block_size,
        };

        FastConvolver {
            impulse_response: impulse_response.to_vec(),
            ring_buffer: RingBuffer::new(buffer_size),
            mode,
        }
    }

    // Reset the convolver's internal state
    pub fn reset(&mut self) {
        self.ring_buffer.reset();
    }

    // Process a block of input and store the result in output
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        match self.mode {
            ConvolutionMode::TimeDomain => {
                for (i, &input_sample) in input.iter().enumerate() {
                    self.ring_buffer.push(input_sample);
                    let mut acc = 0.0;
                    for (j, &impulse) in self.impulse_response.iter().enumerate() {
                        acc += impulse * self.ring_buffer.get(j);
                    }
                    output[i] = acc;
                }
            }
            _ => unimplemented!(),
        }
    }

    pub fn flush(&mut self, output: &mut [f32]) {
        if output.len() < self.impulse_response.len() {
            panic!("Output buffer too small to hold the entire reverb tail. It must be at least {} samples long.", self.impulse_response.len());
        }

        // Clear the provided output buffer
        output.fill(0.0);

        // Produce the reverb tail
        for i in 0..self.impulse_response.len() - 1 {
            let mut acc = 0.0;
            for (j, &impulse) in self.impulse_response.iter().enumerate() {
                if i >= j {
                    acc += impulse * self.ring_buffer.get(i - j);
                }
            }
            output[i] = acc;
        }

        // Reset the ring buffer to clear all state
        self.ring_buffer.reset();
    }

    pub fn required_flush_buffer_size(&self) -> usize {
        self.impulse_response.len() - 1
    }
}
