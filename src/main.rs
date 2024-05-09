use image::io::Reader as ImageReader;
use log::debug;
use rodio::{source::Source, Decoder, OutputStream};
use simple_logger::SimpleLogger;
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::thread::{spawn, JoinHandle};

// Constants for the program
const CHARS: [char; 11] = ['@', '#', 'S', '%', '?', '*', '+', ';', ':', ',', ' '];
const FPS: u64 = 30;
const SIZE: (u32, u32) = (100, 50);
const LENGTH: u64 = 3 * 60 + 40;
const THREAD_COUNT: usize = 8;

#[derive(Clone)]
struct Image {
    data: Vec<u8>,
    width: u32,
    height: u32,
    number: usize,
}

impl std::fmt::Debug for Image {
    /// Prints out the frame in ascii art
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lines: Vec<String> = convert_frame(self);
        let mut frame: String = "".to_owned();
        for i in lines {
            frame = frame + &i + "\n";
        }
        write!(f, "{}", frame)
    }
}

fn main() {
    // Initialized the logging system
    SimpleLogger::new().init().unwrap();
    debug!("Logging initialized");

    // Create the array of frames
    let imgs: Vec<Image> = read_image_directory("./images").unwrap();

    // Create the audio thread
    let handler = spawn(|| {
        play_audio();
    });

    // Go through all the frames and print it to the terminal
    for i in imgs {
        print!("\x1B[2J\x1B[1;1H"); // Clear console
                                    // println!("{}", i.number);
        println!("{:?}", i);
        std::thread::sleep(std::time::Duration::from_millis(1000 / FPS)); // Ensure time duration
    }

    // Join the audio thread
    let _ = handler.join();
}

fn play_audio() {
    // Rodio Audio thread
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let file = BufReader::new(File::open("./badapple.mp3").unwrap());
    let source = Decoder::new(file).unwrap();
    let _ = stream_handle.play_raw(source.convert_samples());
    std::thread::sleep(std::time::Duration::from_secs(LENGTH)); // Ensure time duration
}

fn read_img(path: &str) -> Result<Image, std::io::Error> {
    // Load the image file and convert the image to grayscale
    let mut img = ImageReader::open(path)?.decode().unwrap().into_luma8();
    // Resizing image
    img = image::imageops::resize(&img, SIZE.0, SIZE.1, image::imageops::FilterType::Nearest);

    // Safe image in struct
    let image: Image = Image {
        data: img.to_vec(),
        width: img.width(),
        height: img.height(),
        // Get the file number from the string
        number: path
            .split(".png")
            .next()
            .unwrap()
            .split("images/")
            .collect::<Vec<&str>>()[1]
            .parse::<usize>()
            .unwrap(),
    };
    Ok(image)
}

/// Converts a single pixel to the corresponding ascii symbol
fn convert_pixel(pixel: u8) -> char {
    assert!(((pixel / 25) as usize) < CHARS.len());
    CHARS[(pixel / 25) as usize]
}

fn convert_frame(img: &Image) -> Vec<String> {
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

/// Takes a `std::fs::ReadDir` and consumes it. It returns all the paths to the files in a
/// `Vec<String>`
fn get_path_strings(reader: std::fs::ReadDir) -> Vec<String> {
    let paths = reader.collect::<Vec<Result<std::fs::DirEntry, std::io::Error>>>();
    let mut res: Vec<String> = Vec::with_capacity(paths.len());
    for i in paths {
        res.push(i.unwrap().path().to_str().unwrap().to_string());
    }
    res
}

fn read_image_directory(path: &str) -> Result<Vec<Image>, std::io::Error> {
    // Load content of the image directory
    let paths_reader: std::fs::ReadDir = std::fs::read_dir(path).unwrap();
    let paths = Arc::new(get_path_strings(paths_reader));
    let n: usize = paths.len(); // Count of the images

    // Thread safety
    let monitor: Arc<Mutex<Vec<Image>>> = Arc::new(Mutex::new(Vec::with_capacity(n))); // Mutex for
                                                                                       // the result array
    let counter_monitor: Arc<Mutex<i32>> = Arc::new(Mutex::new(0)); // Mutex for the display
                                                                    // counter
    let mut threads: Vec<JoinHandle<()>> = Vec::with_capacity(n);

    // Size of the workload for a single thread
    let chunk: usize = (&n / THREAD_COUNT) as usize;

    for i in 1..THREAD_COUNT {
        // Copy the ownership of the data
        let m = Arc::clone(&monitor);
        let counter = Arc::clone(&counter_monitor);
        let data_chunk = Arc::clone(&paths);

        // Create the thread
        let handle: JoinHandle<()> = spawn(move || {
            for p in data_chunk[(&i - 1) * &chunk..&i * &chunk].iter() {
                let img: Image = read_img(&p).unwrap();
                {
                    let mut res = m.lock().unwrap();
                    res.push(img);
                }
                {
                    let mut c = counter.lock().unwrap();
                    *c += 1;
                    print!("\x1B[2J\x1B[1;1H");
                    println!("Loading process: {}/{}", c, n);
                }
            }
        });
        threads.push(handle);
    }

    // Wait for the threads to join
    for i in threads {
        i.join().unwrap();
    }
    let mut res: Vec<Image> = monitor.lock().unwrap().to_owned();

    // Sort the images correctly
    res.sort_by_key(|img| img.number);
    for i in &res {
        debug!("Images loaded: {}", i.number);
    }
    Ok(res)
}
