extern crate hound;

use std::env;
use std::fs::File;
use std::io::Write;

fn show_info() {
    eprintln!("MUSI-6106 Assignment Executable");
    eprintln!("(c) 2024 Stephen Garrett & Ian Clester");
}

fn main() {
    show_info();
    // Parse command line arguments
    // First argument is input .wav file, second argument is output text file.
    // TODO: your code here
    let args: Vec<String> = env::args().collect();

    let wav_path = &args[1];
    let txt_path = &args[2];
    // Open the input wave file and determine number of channels
    // TODO: your code here; see `hound::WavReader::open`.
    // Read audio data and write it to the output text file (one column per channel)
    // TODO: your code here; we suggest using `hound::WavReader::samples`, `File::create`, and `write!`.
    //       Remember to convert the samples to floating point values and respect the number of channels!
    let mut reader = hound::WavReader::open(wav_path).expect("Cannot open wav file");

    let mut output_file = File::create(txt_path).expect("Cannot create text file");


    let samples: Vec<f32> = reader.samples::<i16>()
        .filter_map(Result::ok)
        .map(|s| s as f32 / i16::MAX as f32)
        .collect();

    for sample in samples.chunks(2) {
        if let [left, right] = *sample {
            writeln!(output_file, "{} {}", left, right).expect("Cannot save text file");
        }
    }
}


