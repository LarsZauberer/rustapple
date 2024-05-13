use image::imageops::{resize, FilterType};
use log::debug;
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
const SIZE: (u32, u32) = (100, 50);
const THREAD_COUNT: usize = 32;

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

fn main() {
    // Initialized the logging system
    SimpleLogger::new().init().unwrap();
    debug!("Logging initialized");

    // Create the array of frames
    let vid: Video = read_video_file("./badapple.mp4");
    // let imgs: Vec<Image> = read_image_directory("./images").unwrap();

    // Create the audio thread
    let handler: JoinHandle<()> = spawn(move || {
        play_audio(vid.duration as u64);
    });

    // Go through all the frames and print it to the terminal
    for i in vid.images {
        print!("\x1B[2J\x1B[1;1H"); // Clear console
                                    // println!("{}", i.number);
        println!("{:?}", i);
        // Ensure time duration
        std::thread::sleep(std::time::Duration::from_millis(1000 / (vid.fps as u64)));
    }

    // Join the audio thread
    let _ = handler.join();
}

fn play_audio(duration: u64) {
    // Rodio Audio thread
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let file = BufReader::new(File::open("./badapple.mp3").unwrap());
    let source = Decoder::new(file).unwrap();
    let _ = stream_handle.play_raw(source.convert_samples());
    std::thread::sleep(std::time::Duration::from_secs(duration)); // Ensure time duration
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

fn read_video_file(path: &str) -> Video {
    use video_rs::decode::Decoder;

    let url: &Path = Path::new(path);
    let mut decoder: Decoder = Decoder::new(url).expect("Failed to create decoder");
    Video {
        duration: decoder.duration().unwrap().as_secs() as usize,
        fps: decoder.frame_rate() as usize,
        images: convert_video_to_images(&mut decoder),
    }
}

/// Converts the frames from the video decoder to an array of images that are downscaled and
/// grayscale
fn convert_video_to_images(decoder: &mut video_rs::decode::Decoder) -> Vec<Image> {
    use image::DynamicImage::ImageRgb8;
    use image::RgbImage;
    let counter: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));

    let mut frames: Vec<ffmpeg_next::frame::Video> = Vec::with_capacity(7000); // Upper bound for badapple

    for frame in decoder.decode_raw_iter() {
        if let Ok(frame) = frame {
            frames.push(frame);
        } else {
            break;
        }
    }

    let frames_arc: Arc<Vec<ffmpeg_next::frame::Video>> = Arc::new(frames);

    let data_chunk: usize = frames_arc.len() / THREAD_COUNT;
    let res: Arc<Mutex<Vec<Image>>> = Arc::new(Mutex::new(Vec::with_capacity(frames_arc.len())));

    let mut threads: Vec<JoinHandle<()>> = Vec::with_capacity(THREAD_COUNT);

    for i in 0..THREAD_COUNT {
        let counter = Arc::clone(&counter);
        let res = Arc::clone(&res);
        let frames = Arc::clone(&frames_arc);
        let thread: JoinHandle<()> = spawn(move || {
            let mut img_array: Vec<Image> = Vec::with_capacity(data_chunk);
            let mut index: usize = 0;
            for frame in &frames[i * data_chunk..(i + 1) * data_chunk] {
                // Make an image::ImageBuffer from the frame
                let mut img = ImageRgb8(
                    RgbImage::from_vec(frame.width(), frame.height(), frame.data(0).to_vec())
                        .unwrap(),
                )
                .to_luma8();
                // Resize
                img = resize(&img, SIZE.0, SIZE.1, FilterType::Nearest);

                // Add image to array
                img_array.push(Image {
                    data: img.to_vec(),
                    number: i * data_chunk + index,
                    width: SIZE.0,
                    height: SIZE.1,
                });

                // Critical section of changing the image counter
                {
                    let mut n = counter.lock().unwrap();

                    // Progress display
                    *n += 1;
                    print!("\x1B[2J\x1B[1;1H");
                    println!("Frames processed: {}", n);
                }
                index += 1;
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

    let mut r = res.lock().unwrap().to_vec();
    r.sort_by_key(|i| i.number);
    r
}
