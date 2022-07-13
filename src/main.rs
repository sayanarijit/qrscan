use anyhow::Result;
use clap::Parser;
use image::io::Reader as ImageReader;
use image::DynamicImage;
use nokhwa::Camera;
use nokhwa::CameraFormat;
use nokhwa::FrameFormat;
use qrcode::render::unicode::Dense1x2;
use qrcode::render::unicode::Dense1x2::Dark;
use qrcode::render::unicode::Dense1x2::Light;
use qrcode::QrCode;
use std::path::PathBuf;
use std::time::Duration;

static PROGRESS: &[&str] = &[".  ", ".. ", "..."];

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Path to the image to scan. If not specified, the system camera will be used.
    #[clap(value_parser)]
    image: Option<PathBuf>,

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

    /// Invert the ascii QR code colors (works with --qr)
    #[clap(long)]
    invert_colors: bool,

    /// Do not add quiet zone to the ascii QR code (works with --qr)
    #[clap(long)]
    no_quiet_zone: bool,
}

fn capture(args: &Args) -> Result<()> {
    let format = CameraFormat::new_from(640, 480, FrameFormat::MJPEG, 30);
    let mut camera = Camera::new(0, Some(format))?;
    let mut spinner = 0;

    camera.open_stream()?;

    loop {
        let frame = camera.frame()?;
        let image = DynamicImage::ImageRgb8(frame);

        if print_image(&args, image).is_err() {
            eprint!("\rScanning via camera{}", PROGRESS[spinner]);
            spinner = (spinner + 1) % 3;
        } else {
            break;
        }
    }

    Ok(())
}

fn scan(args: &Args, path: &PathBuf) -> Result<()> {
    let image = ImageReader::open(path)?.decode()?;

    print_image(&args, image)
}

fn print_image(args: &Args, image: DynamicImage) -> Result<()> {
    let image = image.to_luma8();
    let mut img = rqrr::PreparedImage::prepare(image);
    let grids = img.detect_grids();

    if let Some(grid) = grids.first() {
        let (meta, content) = grid.decode()?;
        eprint!("\r                        \r");
        if args.qr {
            let qrcode = QrCode::new(&content)?;

            let (dark, light) = if args.invert_colors {
                (Dark, Light)
            } else {
                (Light, Dark)
            };

            let image = qrcode
                .render::<Dense1x2>()
                .dark_color(dark)
                .light_color(light)
                .quiet_zone(!args.no_quiet_zone)
                .build();

            println!("{}", image);
        }

        if args.metadata {
            if args.qr {
                println!()
            };

            println!("Version: {}", meta.version.0);
            println!("Grid Size: {}", meta.version.to_size());
            println!("EC Level: {}", meta.ecc_level);
            println!("Mask: {}", meta.mask);
        }

        if !args.no_content {
            if args.qr || args.metadata {
                println!();
            };
            println!("{}", content);
        }
    } else {
        std::thread::sleep(Duration::from_millis(args.inverval));
        anyhow::bail!("not found")
    };

    Ok(())
}

fn main() {
    let args = Args::parse();
    let mut rc = 0;

    if let Some(path) = args.image.as_ref() {
        if !path.exists() {
            eprintln!("\r{}: No such file    ", path.display());
            rc = 3;
        } else if path.is_dir() {
            eprintln!("cannot scan {}: Is a directory", path.display());
            rc = 2;
        } else if let Err(err) = scan(&args, path) {
            eprintln!("{}", err);
            rc = 1;
        }
    } else if let Err(err) = capture(&args) {
        eprintln!("{}", err);
        rc = 1;
    }

    std::process::exit(rc);
}
