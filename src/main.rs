use image::io::Reader as ImageReader;

const CHARS: [char; 11] = ['@', '#', 'S', '%', '?', '*', '+', ';', ':', ',', ' '];

// #[derive(Debug)]
struct Image {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

impl std::fmt::Debug for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

fn main() {
    println!("{:?}", read_img("/home/lars/lix/assets/wallpaper/nix.png"));
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

fn convert_pixel(pixel: u32) -> char {
    assert!(((pixel / 25) as usize) < CHARS.len());
    CHARS[(pixel / 25) as usize]
}

fn convert_frame(img: &Image) -> Vec<&String> {
    let res: Vec<&String> = Vec::with_capacity((img.width * img.height) as usize);
    for i in 0..img.height {
        let line: &str;
        for e in 0..img.width {
            line = line + (convert_pixel(img.data[i * img.width + e]) as str);
        }
        res.push(line);
    }
    res
}
