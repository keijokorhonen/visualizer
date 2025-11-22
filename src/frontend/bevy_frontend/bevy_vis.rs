use crate::fft_data::FFTData;

pub struct BevyApp {
    pub fft: FFTData,
}

impl BevyApp {
    pub fn new(fft: FFTData) -> Self { Self { fft } }
    pub fn run(self) {
        println!("Bevy frontend not implemented.");
    }
}