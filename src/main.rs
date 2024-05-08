use image::io::Reader as ImageReader;
use log::{debug, info, warn};
use simple_logger::SimpleLogger;

const CHARS: [char; 11] = ['@', '#', 'S', '%', '?', '*', '+', ';', ':', ',', ' '];
const FPS: u64 = 30;

// #[derive(Debug)]
struct Image {
    data: Vec<u8>,
    width: u32,
    height: u32,
    number: usize,
}

impl std::fmt::Debug for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, "{}x{}", self.width, self.height)
        let lines: Vec<String> = convert_frame(self);
        let mut frame: String = "".to_owned();
        for i in lines {
            frame = frame + &i + "\n";
        }
        write!(f, "{}", frame)
    }
}

fn main() {
    SimpleLogger::new().init().unwrap();
    debug!("Logging initialized");
    let imgs: Vec<Image> = read_image_directory("./images").unwrap();
    for i in imgs {
        print!("\x1B[2J\x1B[1;1H");
        println!("{}", i.number);
        println!("{:?}", i);
        std::thread::sleep(std::time::Duration::from_millis(1000 / FPS)); // Ensure time duration
    }
}

fn read_img(path: &str) -> Result<Image, std::io::Error> {
    let mut img = ImageReader::open(path)?.decode().unwrap().into_luma8();
    // Resizing image
    img = image::imageops::resize(&img, 50, 50, image::imageops::FilterType::Nearest);

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

fn read_image_directory(path: &str) -> Result<Vec<Image>, std::io::Error> {
    let paths: std::fs::ReadDir = std::fs::read_dir(path).unwrap();
    let counter_paths: std::fs::ReadDir = std::fs::read_dir(path).unwrap();

    let n: usize = counter_paths.count();
    let mut counter: usize = 0;

    let monitor: std::sync::Mutex<usize> = std::sync::Mutex::new(1);
    let mut res: Vec<Image> = Vec::with_capacity(n);

    let mut threads: Vec<std::thread::JoinHandle<()>> = Vec::new();
    for i in paths {
        let path: String = i.unwrap().path().to_str().unwrap().to_string();
        // debug!("Found image file: {}", path);
        print!("\x1B[2J\x1B[1;1H");
        println!("Loading process: {}/{}", counter, n);
        /* threads.push(std::thread::spawn(move || {
            let img: Image = read_img(&path).unwrap();
            {
                monitor.lock();
                res.push(img);
            }
        })); */
        let img: Image = read_img(&path).unwrap();
        res.push(img);
        counter += 1;
    }

    for i in threads {
        i.join();
    }

    // Sort the image correctly
    res.sort_by_key(|img| img.number);
    for i in &res {
        debug!("Images loaded: {}", i.number);
    }
    Ok(res)
}
