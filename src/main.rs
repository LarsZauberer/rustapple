use image::io::Reader as ImageReader;
use log::{debug, info, warn};
use simple_logger::SimpleLogger;

const CHARS: [char; 11] = ['@', '#', 'S', '%', '?', '*', '+', ';', ':', ',', ' '];

// #[derive(Debug)]
struct Image {
    data: Vec<u8>,
    width: u32,
    height: u32,
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
        println!("{:?}", i);
    }
}

fn read_img(path: &str) -> Result<Image, std::io::Error> {
    let img = ImageReader::open(path)?.decode().unwrap().into_luma8();
    let image: Image = Image {
        data: img.to_vec(),
        width: img.width(),
        height: img.height(),
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
    let mut res: Vec<Image> = Vec::new();
    for i in paths {
        let path: String = i.unwrap().path().to_str().unwrap().to_string();
        debug!("Found image file: {}", path);
        res.push(read_img(&path)?);
    }
    Ok(res)
}
