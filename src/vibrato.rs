// create a vibrato processor that is able to add vibrato effect to each block of audio data that works with multiple channels.

use crate::ring_buffer::RingBuffer;

use crate::lfo::LFO; 

/// Vibrato is a struct that contains a delay line, an LFO, and a width, sample rate, and delay.
pub struct Vibrato {
    delay_line: Vec<RingBuffer<f32>>,
    lfo: LFO,
    width: f32,
    sample_rate: f32,
    delay: f32,
}

impl Vibrato {
    /// Create a new vibrato processor with a given sample rate, maximum delay, delay, width, frequency, and number of channels.
    /// # Arguments
    /// * `sample_rate` - The sample rate of the audio data.
    /// * `max_delay` - The maximum delay of the vibrato effect.
    /// * `delay` - The delay of the vibrato effect.
    /// * `width` - The width of the vibrato effect.
    /// * `frequency` - The frequency of the vibrato effect.
    /// * `channels` - The number of channels of the audio data.
    pub fn new(sample_rate: f32, max_delay: f32, delay: f32, width: f32, frequency: f32, channels: usize) -> Vibrato {
        // throw an error if width is bigger than max_delay
        if width > max_delay {
            panic!("Width is bigger than max_delay");
        }
        let mut delay_line = Vec::new();
        let width = (width * sample_rate).round();
        let lfo = LFO::new(sample_rate, frequency, 1.0);
        let delay = (delay * sample_rate).round();
        let max_delay = (max_delay * sample_rate).round();

        let len_delay_line =delay + width * 2.0;
        for _ in 0..channels {
            let mut ring_buffer = RingBuffer::new((len_delay_line) as usize);
            ring_buffer.set_read_index(0);
            ring_buffer.set_write_index((len_delay_line-1.0) as usize);
            delay_line.push(ring_buffer);
        }


        Vibrato {
            delay_line,
            lfo,
            width,
            sample_rate,
            delay,
        }
    }

    // process a block of audio data by adding vibrato effect to it
    /// Process a block of audio data by adding vibrato effect to it.
    pub fn process(&mut self, input: &mut [&mut [f32]], output: &mut [&mut [f32]]) {
        for channel in 0..input.len() {
            for sample in 0..input[channel].len() {
                let delay = self.lfo.get_sample()*self.width + self.delay+1.0;
                // dbg!(delay);
                let read_index = self.delay_line[channel].get_read_index();
                // dbg!(read_index);
                // dbg!(self.delay_line[channel].peek());  
                let write_index = self.delay_line[channel].get_write_index();
                // dbg!(write_index);
                let mut value = self.delay_line[channel].get_frac(delay);
                // dbg!(value);

                self.delay_line[channel].push(input[channel][sample]);
                self.delay_line[channel].pop();
                output[channel][sample] = value;
                // dbg!(output[channel][sample]);
                
            }
        }
    }
    /// Set the parameters of the vibrato processor.
    pub fn set_params(&mut self, delay: f32, width: f32, frequency: f32) {
        self.delay = (delay * self.sample_rate).round();
        self.width = (width * self.sample_rate).round();
        self.lfo.set_frequency(frequency);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_freq_0() {
        // test that output equals delayed input when modulation amplitude is 0 
        let sample_rate = 44100.0;
        let max_delay = 0.01;
        let width = 0.001;
        let frequency = 5.0;
        let channels = 2;
        let delay = 2  as f32 / 44100 as f32;
        let mut vibrato = Vibrato::new(sample_rate, max_delay, delay, width, frequency, channels);
        let mut channel1: [f32; 5] = [1.0, 2.0, 3.0, 4.0, 5.0];
        let mut channel2: [f32; 5] = [6.0, 7.0, 8.0, 9.0, 10.0];
        let mut block: [&mut [f32]; 2] = [&mut channel1, &mut channel2];
        let mut output: [&mut [f32]; 2] = [&mut [0.0; 5], &mut [0.0; 5]];
        vibrato.process(&mut block, &mut output);
        assert_eq!(output[1], [0.0, 0.0, 6.0, 7.0, 8.0]);
        assert_eq!(output[0], [0.0, 0.0, 1.0, 2.0, 3.0]);
    }

    #[test]
    // test that DC input results in DC output, regardless of parameters
    fn test_dc_input() {
        let sample_rate = 44100.0;
        let max_delay = 0.01;
        let width = (1/44100) as f32;
        let frequency = 5.0;
        let channels = 2;
        let delay = 2  as f32 / 44100 as f32;
        let mut vibrato = Vibrato::new(sample_rate, max_delay, delay, width, frequency, channels);
        let mut channel1: [f32; 5] = [1.0, 1.0, 1.0, 1.0, 1.0];
        let mut channel2: [f32; 5] = [1.0, 1.0, 1.0, 1.0, 1.0];
        let mut block: [&mut [f32]; 2] = [&mut channel1, &mut channel2];
        let mut output: [&mut [f32]; 2] = [&mut [0.0; 5], &mut [0.0; 5]];
        vibrato.process(&mut block, &mut output);
        // dbg!(&output);
        assert_eq!(output[1], [0.0, 0.0, 1.0, 1.0, 1.0]);
        assert_eq!(output[0], [0.0, 0.0, 1.0, 1.0, 1.0]);
    }
    #[test]
    fn test_zero_signal() {
        // test that zero input results in zero output
        let sample_rate = 44100.0;
        let max_delay = 0.01;
        let width = (1/44100) as f32;
        let frequency = 5.0;
        let channels = 2;
        let delay = 2  as f32 / 44100 as f32;
        let mut vibrato = Vibrato::new(sample_rate, max_delay, delay, width, frequency, channels);
        let mut channel1: [f32; 5] = [0.0, 0.0, 0.0, 0.0, 0.0];
        let mut channel2: [f32; 5] = [0.0, 0.0, 0.0, 0.0, 0.0];
        let mut block: [&mut [f32]; 2] = [&mut channel1, &mut channel2];
        let mut output: [&mut [f32]; 2] = [&mut [0.0; 5], &mut [0.0; 5]];
        vibrato.process(&mut block, &mut output);
        assert_eq!(output[1], [0.0, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(output[0], [0.0, 0.0, 0.0, 0.0, 0.0]);
    }
    
}



