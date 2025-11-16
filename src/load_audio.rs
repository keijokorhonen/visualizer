use std::fs::File;
use std::path::Path;

use symphonia::default::{get_codecs, get_probe};
use symphonia::core::formats::FormatOptions;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::probe::Hint;
use symphonia::core::codecs::{CODEC_TYPE_NULL, DecoderOptions};
use symphonia::core::audio::SampleBuffer;

pub fn load_samples_from_file(path: &str) -> (Vec<f32>, u32) {
    // Open the media source.
    let path = Path::new(path);
    let src = File::open(&path).expect("failed to open media");
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    let mss = MediaSourceStream::new(Box::new(src), Default::default());

    let mut hint = Hint::new();
    hint.with_extension(extension);

    // Use the default options for metadata and format readers.
    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();

    let probe = get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)
        .expect("unsupported format");

    // Get the instantiated format reader.
    let mut format = probe.format;
    
    // Find the first audio track with a known (decodeable) codec.
    let track = format.tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .expect("no supported audio tracks");

    // Use the default options for the decoder.
    let dec_opts: DecoderOptions = Default::default();

    // Create a decoder for the track.
    let mut decoder = get_codecs().make(&track.codec_params, &dec_opts)
        .expect("unsupported codec");

    let sample_rate = decoder.codec_params().sample_rate.unwrap();
    let num_channels = decoder.codec_params().channels.unwrap().count();
    println!("Loading file: {}", path.file_name().unwrap().to_string_lossy());

    let mut samples: Vec<f32> = Vec::new();
    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(_) => break, // Reached the end of the stream.
        };
        
        // Decode the packet into audio samples.
        let decoded = decoder.decode(&packet).unwrap();
        let mut sample_buffer = SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());

        sample_buffer.copy_interleaved_ref(decoded);

        // Average all channels (for stereo, just left and right)
        for frame in sample_buffer.samples().chunks(num_channels) {
            let avg = frame.iter().copied().sum::<f32>() / num_channels as f32;
            samples.push(avg);
        }
    }

    (samples, sample_rate)
}