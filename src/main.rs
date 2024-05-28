use clap::Parser;
use image::imageops::{resize, FilterType};
use log::{debug, error};
use rodio::{source::Source, Decoder, OutputStream};
use simple_logger::SimpleLogger;
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread::{spawn, JoinHandle};

// Constants for the program
const CHARS: [char; 11] = ['@', '#', 'S', '%', '?', '*', '+', ';', ':', ',', ' '];

#[derive(Clone)]
struct Image {
    data: Vec<u8>,
    width: u32,
    height: u32,
    number: usize,
}

impl Debug for Image {
    /// Prints out the frame in ascii art
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let lines: Vec<String> = convert_image_to_ascii_line(self);
        let mut frame: String = "".to_owned();
        for i in lines {
            frame = frame + &i + "\n";
        }
        write!(f, "{}", frame)
    }
}

#[derive(Debug, Clone)]
struct Video {
    fps: usize,
    duration: usize, // Duration in seconds
    images: Vec<Image>,
}

#[derive(Parser, Debug)]
#[command(version)]
struct Cli {
    /// The video file for the application
    #[arg(short = 'f', long = "file", default_value_t = String::from("rustapple.mp4"))]
    file: String,

    /// The audio file for the application
    #[arg(short = 'a', long = "audio", default_value_t = String::from("rustapple_audio.mp3"))]
    audio_file: String,

    /// Number of threads for video processing
    // TODO: Doesn't work yet
    #[arg(short = 't', long = "threads", default_value_t = 4)]
    threads: usize,

    /// YT URL to download the video from
    #[arg(short = 'y', long = "ytdownload", default_value_t = String::from(""))]
    yt_url: String,

    /// Terminal width
    #[arg(short = 'w', long = "width", default_value_t = 100)]
    width: usize,
}

fn main() {
    // Initialized the logging system
    SimpleLogger::new().init().unwrap();
    debug!("Logging initialized");

    // CLI Parser
    let mut cli: Cli = Cli::parse();

    // Get video_file path
    if cli.yt_url != "" {
        (cli.file, cli.audio_file) = yt_download(&cli.yt_url);
    }

    // Check if video file exists
    if !Path::new(&cli.file).exists() {
        error!("No video file found");
        panic!("No video file found");
    }

    // Create the array of frames
    let vid: Video = read_video_file(&cli.file, &cli.width, &cli.threads);

    // Create the audio thread
    let mut handler: Option<JoinHandle<()>> = None;
    if cli.audio_file != "" {
        handler = Some(spawn(move || {
            play_audio(&cli.audio_file, vid.duration as u64);
        }));
    }

    // Go through all the frames and print it to the terminal
    for i in vid.images {
        print!("\x1B[2J\x1B[1;1H"); // Clear console
                                    // println!("{}", i.number);
        println!("{:?}", i);
        // Ensure time duration
        std::thread::sleep(std::time::Duration::from_millis(1000 / (vid.fps as u64)));
    }

    // Join the audio thread
    if let Some(handler) = handler {
        let _ = handler.join();
    }
}

fn play_audio(path: &str, duration: u64) {
    // Rodio Audio thread
    // Get path for audio file
    let path: &Path = Path::new(path);
    if !path.exists() {
        error!("The audio file '{:?}' doesn't exist. No such file.", path);
        panic!("The audio file '{:?}' doesn't exist. No such file.", path)
    }

    // Play audio file
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let file = BufReader::new(File::open(path).unwrap());
    let source = Decoder::new(file).unwrap();
    let _ = stream_handle.play_raw(source.convert_samples());
    std::thread::sleep(std::time::Duration::from_secs(duration)); // Ensure time duration
}

/// YT-Downloader. Downloads a specified youtube video and the video is split in video and audio.
/// The output is always in the current directory at `rustapple.mp4` (video) and `rustapple_audio.mp3`
/// (audio)
fn yt_download(url: &str) -> (String, String) {
    use log::info;
    use std::process::Command;

    info!("Starting to download the youtube video: {}", url);

    // Check if yt-dlp is installed
    Command::new("yt-dlp").arg("--version").output().expect("Failed to execute command `yt-dlp --version`. This is probably because yt-dlp is not installed. Please install yt-dlp to download youtube videos.");

    // Clean up existing rustapple files
    Command::new("rm")
        .arg("rustapple.opus")
        .output()
        .expect("Failed to remove all existing rustapple files");
    Command::new("rm")
        .arg("rustapple.mp4")
        .output()
        .expect("Failed to remove all existing rustapple files");

    // Download video and audio
    Command::new("yt-dlp")
        .arg(url)
        .arg("-o")
        .arg("rustapple.mp4")
        .arg("-f")
        .arg("mp4")
        .output()
        .expect("Failed to download video file");

    Command::new("yt-dlp")
        .arg(url)
        .arg("-o")
        .arg("rustapple_audio.mp3")
        .arg("-x")
        .arg("--audio-format")
        .arg("mp3")
        .arg("--force-overwrites")
        .output()
        .expect("Failed to download audio file");

    info!("Finished download. Files are in `rustapple.mp4` and `rustapple_audio.mp3`");

    // Return default paths
    (
        "rustapple.mp4".to_string(),
        "rustapple_audio.mp3".to_string(),
    )
}

/// Converts a single pixel to the corresponding ascii symbol
fn convert_pixel(pixel: u8) -> char {
    assert!(((pixel / 25) as usize) < CHARS.len());
    CHARS[(pixel / 25) as usize]
}

/// Converts an image to multiple lines of ascii art
fn convert_image_to_ascii_line(img: &Image) -> Vec<String> {
    let mut res: Vec<String> = Vec::with_capacity((img.width * img.height) as usize);
    for i in 0..img.height {
        let mut line: String = "".to_owned();
        for e in 0..img.width {
            line = line + &(convert_pixel(img.data[(i * img.width + e) as usize]).to_string());
        }
        res.push(line);
    }
    res
}

/// Reads a specified video file (tested with `.mp4`) and returns a `Video` struct.
fn read_video_file(path: &str, width: &usize, threads: &usize) -> Video {
    use video_rs::decode::Decoder;

    let url: &Path = Path::new(path);
    let decoder: Decoder = Decoder::new(url).expect("Failed to create decoder");
    Video {
        duration: decoder.duration().unwrap().as_secs() as usize,
        fps: decoder.frame_rate() as usize,
        images: convert_video_to_images(decoder, *width, *threads),
    }
}

/// Converts the frames from the video decoder to an array of images that are downscaled and
/// grayscale
fn convert_video_to_images(
    decoder: video_rs::decode::Decoder,
    width: usize,
    threads_count: usize,
) -> Vec<Image> {
    use image::DynamicImage::ImageRgb8;
    use image::RgbImage;

    let counter: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
    let dec: Arc<Mutex<video_rs::decode::Decoder>> = Arc::new(Mutex::new(decoder));
    let res: Arc<Mutex<Vec<Image>>> = Arc::new(Mutex::new(Vec::with_capacity(7000))); // Upperbound for bad apple

    let mut threads: Vec<JoinHandle<()>> = Vec::with_capacity(threads_count);

    for _ in 0..threads_count {
        let dec = Arc::clone(&dec);
        let counter = Arc::clone(&counter);
        let res = Arc::clone(&res);

        let thread: JoinHandle<()> = spawn(move || {
            let mut img_array: Vec<Image> = Vec::with_capacity(7000);
            loop {
                // Try to get a frame
                let (frame, n) = {
                    let mut dec = dec.lock().unwrap();
                    let mut n = counter.lock().unwrap();
                    *n += 1;
                    (dec.decode_raw(), *n - 1)
                };
                // Successfully got frame
                if let Ok(frame) = frame {
                    // Make an image::ImageBuffer from the frame
                    let mut img = ImageRgb8(
                        RgbImage::from_vec(frame.width(), frame.height(), frame.data(0).to_vec())
                            .unwrap(),
                    )
                    .to_luma8();
                    // Resize
                    img = resize(&img, width as u32, (width / 2) as u32, FilterType::Nearest);

                    // Critical section of changing the image counter
                    {
                        // Add image to array
                        img_array.push(Image {
                            data: img.to_vec(),
                            number: n as usize,
                            width: width as u32,
                            height: (width / 2) as u32,
                        });

                        // Progress display
                        print!("\x1B[2J\x1B[1;1H");
                        println!("Frames processed: {}", n);
                    }
                } else {
                    // All frames have been processes -> End Thread
                    break;
                }
            }
            {
                let mut r = res.lock().unwrap();
                r.append(&mut img_array);
            }
        });

        threads.push(thread);
    }

    for i in threads {
        let _ = i.join();
    }

    // Put frames in the correct order
    let mut r = res.lock().unwrap().to_vec();
    r.sort_by_key(|i| i.number);
    r
}
