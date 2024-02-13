use std::iter::Filter;

pub struct CombFilter {
    // TODO: your code here
    filter_type: FilterType,
    gain: f32,
    delay_m: usize,
    sample_rate_hz: f32,
    num_channels: usize,
    buffer: Vec<f32>,
    buffer_index: usize,
    
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
    InvalidValue { param: FilterParam, value: f32 }
}

impl CombFilter {
    pub fn new(filter_type: FilterType, max_delay_secs: f32, sample_rate_hz: f32, num_channels: usize) -> Self {
        let delay_m = (max_delay_secs*sample_rate_hz) as usize;
        CombFilter { filter_type: (filter_type), gain: (0.0), delay_m: (delay_m), sample_rate_hz: (sample_rate_hz), num_channels: (num_channels), buffer: vec![0.0; delay_m], buffer_index: 0, }
    }

    pub fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.buffer_index = 0;
        
    }

    pub fn process(&mut self, input: &[&[f32]], output: &mut [&mut [f32]]) {
        match self.filter_type {
            FilterType::FIR => self.fir(input, output),
            FilterType::IIR => self.iir(input, output),
        }
    }

    fn fir(&mut self, input: &[f32], output: &mut [f32]) {

    }

    fn iir(&mut self, input: &[f32], output: &mut [f32]) {
        
    }

    pub fn set_param(&mut self, param: FilterParam, value: f32) -> Result<(), Error> {
        match param {
            FilterParam::Delay => {
                let delay_m = (value * self.sample_rate_hz) as usize;
                if delay_m > self.buffer.len() {
                    Err(Error::InvalidValue { param: (FilterParam::Delay), value: (value) })
                } else {
                    self.delay_m = delay_m;
                    Ok(())
                }

            }
            FilterParam::Gain => {
                self.gain = value;
                Ok(())
            }
        }
    }

    pub fn get_param(&self, param: FilterParam) -> f32 {
        match param {

            FilterParam::Gain => self.gain,
            FilterParam::Delay => self.delay_m as f32 / self.sample_rate_hz,

        }
    }

    // TODO: feel free to define other functions for your own use
}

// TODO: feel free to define other types (here or in other modules) for your own use
