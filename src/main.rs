use anyhow::Result;
use clap::Parser;
use csscolorparser::Color;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::io::Reader as ImageReader;
use image::ColorType;
use image::DynamicImage;
use image::EncodableLayout;
use image::ImageBuffer;
use image::Rgba;
use nokhwa::Camera;
use nokhwa::CameraFormat;
use nokhwa::FrameFormat;
use qrcode::render::svg;
use qrcode::render::unicode::Dense1x2;
use qrcode::render::unicode::Dense1x2::Dark;
use qrcode::render::unicode::Dense1x2::Light;
use qrcode::QrCode;
use std::io::Cursor;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

static PROGRESS: &[&str] = &[".  ", ".. ", "..."];

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Path to the image to scan. If not specified, the system camera will be used
    ///
    /// Examples:
    ///
    ///   qrscan /path/to/input.png
    ///
    ///   cat /path/to/input.png | qrscan -
    #[clap(value_parser)]
    image: Option<PathBuf>,

    /// Preview the camera on the terminal (if compatible)
    #[clap(long, short)]
    preview: bool,

    /// Preview display's x coordinate (works with --preview)
    #[clap(long, default_value = "0")]
    preview_x: u16,

    /// Preview diaplay's y coordinate (works with --preview)
    #[clap(long, default_value = "0")]
    preview_y: i16,

    /// Preview width (works with --preview)
    #[clap(long)]
    preview_w: Option<u32>,

    /// Preview height (works with --preview)
    #[clap(long)]
    preview_h: Option<u32>,

    /// Print metadata
    #[clap(long, short)]
    metadata: bool,

    /// Print the QR code
    #[clap(long)]
    qr: bool,

    /// Do not print the content
    #[clap(long, short)]
    no_content: bool,

    /// Interval between scans in milisecond
    #[clap(long, short, default_value = "200")]
    inverval: u64,

    /// Invert the QR code colors
    #[clap(long)]
    invert_colors: bool,

    /// Specify the QR code foreground color (when exporting image)
    #[clap(long, default_value = "#000")]
    fg: String,

    /// Specify the QR code background color (when exporting image)
    #[clap(long, default_value = "#fff")]
    bg: String,

    /// Do not add quiet zone to the QR code
    #[clap(long)]
    no_quiet_zone: bool,

    /// Export the QR code as ascii text to the given path
    #[clap(long)]
    ascii: Option<PathBuf>,

    /// Export the QR code as svg image to the given path
    #[clap(long)]
    svg: Option<PathBuf>,

    /// Export the QR code as png image to the given path
    #[clap(long)]
    png: Option<PathBuf>,

    /// Export the QR code as jpeg image to the given path
    #[clap(long)]
    jpeg: Option<PathBuf>,
}

fn capture(args: &Args) -> Result<()> {
    let format = CameraFormat::new_from(640, 480, FrameFormat::MJPEG, 30);
    let mut camera = Camera::new(0, Some(format))?;
    let mut spinner = 0;

    let preview = viuer::Config {
        x: args.preview_x,
        y: args.preview_y,
        restore_cursor: false,
        transparent: false,
        absolute_offset: true,
        width: args.preview_w,
        height: args.preview_h,
        ..Default::default()
    };

    camera.open_stream()?;

    loop {
        let frame = camera.frame()?;
        let image = DynamicImage::ImageRgb8(frame);

        if print_image(args, &image).is_err() {
            if args.preview {
                viuer::print(&image, &preview)?;
            } else {
                eprint!("\rScanning via camera{}", PROGRESS[spinner]);
                spinner = (spinner + 1) % 3;
            };
        } else {
            break;
        }
    }

    Ok(())
}

fn scan_stdin(args: &Args) -> Result<()> {
    let mut buf = vec![];
    let mut stdin = std::io::stdin().lock();
    stdin.read_to_end(&mut buf)?;

    let image = ImageReader::new(Cursor::new(buf))
        .with_guessed_format()?
        .decode()?;

    print_image(args, &image)
}

fn scan_file(args: &Args, path: &PathBuf) -> Result<()> {
    let image = ImageReader::open(path)?.decode()?;
    print_image(args, &image)
}

fn build_binary_image(
    content: &str,
    (dr, dg, db, da): (u8, u8, u8, u8),
    (lr, lg, lb, la): (u8, u8, u8, u8),
    quiet_zone: bool,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let img = QrCode::new(content)?
        .render::<Rgba<u8>>()
        .dark_color(Rgba([dr, dg, db, da]))
        .light_color(Rgba([lr, lg, lb, la]))
        .quiet_zone(quiet_zone)
        .build();
    Ok(img)
}

fn print_image(args: &Args, image: &DynamicImage) -> Result<()> {
    let image = image.to_luma8();
    let mut img = rqrr::PreparedImage::prepare(image);
    let grids = img.detect_grids();

    if let Some(grid) = grids.first() {
        let (meta, content) = grid.decode()?;
        eprint!("\r                        \r");

        // Ansi
        if args.qr {
            if args.preview {
                println!();
            }

            let (dark, light) = if args.invert_colors {
                (Dark, Light)
            } else {
                (Light, Dark)
            };

            let image = QrCode::new(&content)?
                .render::<Dense1x2>()
                .dark_color(dark)
                .light_color(light)
                .quiet_zone(!args.no_quiet_zone)
                .build();

            println!("{}", image);
        }

        // Metadata
        if args.metadata {
            if args.preview || args.qr {
                println!()
            };

            println!("Version: {}", meta.version.0);
            println!("Grid Size: {}", meta.version.to_size());
            println!("EC Level: {}", meta.ecc_level);
            println!("Mask: {}", meta.mask);
        }

        // Content
        if !args.no_content {
            if args.preview || args.qr || args.metadata {
                println!();
            };
            println!("{}", content);
        }

        // Output image colors
        let (dark, light) = if args.invert_colors {
            (&args.bg, &args.fg)
        } else {
            (&args.fg, &args.bg)
        };

        // SVG
        if let Some(path) = args.svg.as_ref() {
            let image = QrCode::new(&content)?
                .render()
                .dark_color(svg::Color(dark))
                .light_color(svg::Color(light))
                .quiet_zone(!args.no_quiet_zone)
                .build()
                .into_bytes();

            if path.to_str() == Some("-") {
                std::io::stdout().write_all(&image)?;
            } else {
                std::fs::write(path, image)?
            }
        }

        // Ascii
        if let Some(path) = args.ascii.as_ref() {
            let image = QrCode::new(&content)?
                .render::<char>()
                .module_dimensions(2, 1)
                .quiet_zone(!args.no_quiet_zone)
                .build()
                .into_bytes();

            if path.to_str() == Some("-") {
                std::io::stdout().write_all(&image)?;
            } else {
                std::fs::write(path, image)?
            }
        }

        // RGB colors
        let dark = dark.parse::<Color>()?.to_linear_rgba_u8();
        let light = light.parse::<Color>()?.to_linear_rgba_u8();

        // PNG
        if let Some(path) = args.png.as_ref() {
            let image = build_binary_image(&content, dark, light, !args.no_quiet_zone)?;
            let bytes = image.as_bytes();

            let mut result: Vec<u8> = Default::default();
            let encoder = PngEncoder::new(&mut result);
            encoder.encode(bytes, image.width(), image.height(), ColorType::Rgba8)?;

            if path.to_str() == Some("-") {
                std::io::stdout().write_all(&result)?;
            } else {
                std::fs::write(path, result)?
            }
        }

        // JPEG
        if let Some(path) = args.jpeg.as_ref() {
            let image = build_binary_image(&content, dark, light, !args.no_quiet_zone)?;
            let bytes = image.as_bytes();

            let mut result: Vec<u8> = Default::default();
            let mut encoder = JpegEncoder::new(&mut result);
            encoder.encode(bytes, image.width(), image.height(), ColorType::Rgba8)?;

            if path.to_str() == Some("-") {
                std::io::stdout().write_all(&result)?;
            } else {
                std::fs::write(path, result)?
            }
        }
    } else {
        std::thread::sleep(Duration::from_millis(args.inverval));
        anyhow::bail!("failed to read")
    };

    Ok(())
}

fn main() {
    let args = Args::parse();
    let mut rc = 0;

    if let Some(path) = args.image.as_ref() {
        if path.to_str() == Some("-") {
            if let Err(err) = scan_stdin(&args) {
                eprintln!("error: qrscan: {}", err);
                rc = 1;
            }
        } else if !path.exists() {
            eprintln!("error: qrscan: {}: No such file", path.display());
            rc = 3;
        } else if path.is_dir() {
            eprintln!(
                "error: qrscan: cannot scan {}: Is a directory",
                path.display()
            );
            rc = 2;
        } else if let Err(err) = scan_file(&args, path) {
            eprintln!("error: qrscan: {}", err);
            rc = 1;
        }
    } else if let Err(err) = capture(&args) {
        eprintln!("error: qrscan: {}", err);
        rc = 1;
    }

    std::process::exit(rc);
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use assert_cmd::prelude::OutputOkExt;

    fn qrscan() -> assert_cmd::Command {
        assert_cmd::Command::cargo_bin("qrscan").unwrap()
    }

    pub struct TestFile {
        pub path: PathBuf,
    }

    impl TestFile {
        pub fn new(id: &str, ext: &str) -> Self {
            let path = PathBuf::from(format!("test_{id}.{}", ext));
            let header = format!("Accept: image/{}", ext);
            let data = format!("foo {}", ext);

            std::process::Command::new("curl")
                .arg("https://qrcode.show")
                .arg("-H")
                .arg(&header)
                .arg("-d")
                .arg(&data)
                .arg("-o")
                .arg(&path)
                .unwrap();

            Self { path }
        }
    }

    impl Drop for TestFile {
        fn drop(&mut self) {
            std::fs::remove_file(&self.path).unwrap();
        }
    }

    #[test]
    fn test_help() {
        qrscan().arg("-h").assert().success();
        qrscan().arg("--help").assert().success();
    }

    #[test]
    fn test_scan_jpeg_file() {
        let file = TestFile::new("scan_jpeg_file", "jpeg");
        qrscan()
            .arg(&file.path)
            .assert()
            .success()
            .stdout("foo jpeg\n");
    }

    #[test]
    fn test_scan_png_file() {
        let file = TestFile::new("scan_png_file", "png");
        qrscan()
            .arg(&file.path)
            .assert()
            .success()
            .stdout("foo png\n");
    }

    #[test]
    fn test_scan_from_stdin() {
        let file = TestFile::new("scan_from_stdin", "png");
        qrscan()
            .arg("-")
            .pipe_stdin(&file.path)
            .unwrap()
            .assert()
            .success()
            .stdout("foo png\n");
    }

    #[test]
    fn test_scan_no_content() {
        let file = TestFile::new("scan_no_content", "png");
        qrscan()
            .arg(&file.path)
            .arg("-n")
            .assert()
            .success()
            .stdout("");

        qrscan()
            .arg(&file.path)
            .arg("--no-content")
            .assert()
            .success()
            .stdout("");
    }

    #[test]
    fn test_export_files() {
        let file = TestFile::new("export_files", "png");
        qrscan()
            .arg(&file.path)
            .arg("--ascii")
            .arg("test.ascii")
            .arg("--svg")
            .arg("test.svg")
            .arg("--jpeg")
            .arg("test.jpeg")
            .arg("--png")
            .arg("test.png")
            .assert()
            .success()
            .stdout("foo png\n");

        assert!(PathBuf::from("test.ascii").exists());
        assert!(PathBuf::from("test.svg").exists());
        assert!(PathBuf::from("test.jpeg").exists());
        assert!(PathBuf::from("test.png").exists());

        std::fs::remove_file("test.ascii").unwrap();
        std::fs::remove_file("test.svg").unwrap();
        std::fs::remove_file("test.jpeg").unwrap();
        std::fs::remove_file("test.png").unwrap();
    }
}
