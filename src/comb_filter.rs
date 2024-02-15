pub struct CombFilter {
    filter_type: FilterType,
    sample_rate_hz: f32,
    num_channels: usize,
    gain: f32,
    delay_secs: f32,
    delay_samples: usize,
    delay_buffers: Vec<Vec<f32>>,
}

#[derive(Debug, Clone, Copy)]
pub enum FilterType {
    FIR,
    IIR,
}

#[derive(Debug, Clone, Copy)]
pub enum FilterParam {
    Gain,
    Delay,
}

#[derive(Debug, Clone)]
pub enum Error {
    InvalidValue { param: FilterParam, value: f32 },
}

impl CombFilter {
    pub fn new(filter_type: FilterType, max_delay_secs: f32, sample_rate_hz: f32, num_channels: usize) -> Self {
        let delay_samples = (sample_rate_hz * max_delay_secs) as usize;
        let delay_buffers = vec![vec![0.0; delay_samples]; num_channels];
        
        CombFilter {
            filter_type,
            sample_rate_hz,
            num_channels,
            gain: 0.0, 
            delay_secs: max_delay_secs,
            delay_samples,
            delay_buffers,
        }
    }

    pub fn reset(&mut self) {
        for buffer in &mut self.delay_buffers {
            buffer.fill(0.0);
        }
    }

    pub fn process(&mut self, input: &[&[f32]], output: &mut [&mut [f32]]) {
        for (channel_idx, channel_buffers) in input.iter().enumerate() {
            let delay_buffer = &mut self.delay_buffers[channel_idx];
            let mut read_idx = self.delay_samples % delay_buffer.len(); 

            for (sample_idx, &input_sample) in channel_buffers.iter().enumerate() {

                let output_sample = match self.filter_type {
                    FilterType::FIR => {

                        input_sample + delay_buffer[read_idx] * self.gain
                    },
                    FilterType::IIR => {

                        input_sample + delay_buffer[read_idx] * self.gain
                    },
                };

                delay_buffer[read_idx] = match self.filter_type {
                    FilterType::FIR => input_sample,
                    FilterType::IIR => output_sample,
                };
    
                output[channel_idx][sample_idx] = output_sample;


                read_idx = (read_idx + 1) % delay_buffer.len();
            }
        }
    }

    pub fn set_param(&mut self, param: FilterParam, value: f32) -> Result<(), Error> {
        match param {
            FilterParam::Gain => {
                self.gain = value;
                Ok(())
            },
            FilterParam::Delay => {
                self.delay_secs = value;
                let delay_samples = (self.sample_rate_hz * value) as usize;
                
                if delay_samples > self.delay_samples {
                    Err(Error::InvalidValue { param: (FilterParam::Delay), value: (value) })
                } else {
                    self.delay_samples = (self.sample_rate_hz * self.delay_secs) as usize;
                    Ok(())
                }
                
            },
        }
    }

    pub fn get_param(&self, param: FilterParam) -> f32 {
        match param {
            FilterParam::Gain => self.gain,
            FilterParam::Delay => self.delay_secs,
        }
    }
        // TODO: feel free to define other functions for your own use
}
// TODO: feel free to define other types (here or in other modules) for your own use