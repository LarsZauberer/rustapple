use image::io::Reader as ImageReader;

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
