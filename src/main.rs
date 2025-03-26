#![feature(path_file_prefix)]

use clap::{Args, Parser, Subcommand};
use image::{DynamicImage, GenericImage, ImageReader, Rgba};
use imageproc::map::*;
use core::panic;
use std::path::Path;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Fix normal map channels
    FixNormal(NormalArgs),
    /// Split a multi map to RGBA images
    SplitMulti(SplitArgs),
    /// Join RGBA images to a multimap
    JoinMulti(JoinArgs),
}

#[derive(Args)]
struct SplitArgs {
    multi_path: String,
}

#[derive(Args)]
struct JoinArgs {
    r_path: String,
    g_path: String,
    b_path: String,
    a_path: String,
}

#[derive(Args)]
struct NormalArgs {
    normal_path: String,
}

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::SplitMulti(args) => {
            split(args);
        }
        Commands::JoinMulti(args) => {
            join(args);
        },
        Commands::FixNormal(args) => {
            normal(args);
        },
    }
}

fn split(args: SplitArgs) {
    let path = Path::new(&args.multi_path);
    let image = ImageReader::open(path).expect("Multi Path should be an image.");
    let binding = image.decode().unwrap().to_rgba8();
    for x in 0..4 {
        let channel = map_colors(&binding, |p| { Rgba([p[x], p[x], p[x], 255]) });
        let ext = match x {
            0 => "_R",
            1 => "_G",
            2 => "_B",
            3 => "_A",
            _ => "_O"
           
        };
        let new_file = path.file_prefix().unwrap().to_str().unwrap().to_string() + ext + ".png";
        let save_path = path.with_file_name(new_file);
        channel.save_with_format(save_path, image::ImageFormat::Png).unwrap();
    }
}

fn join(args: JoinArgs) {
    let r_image = ImageReader::open(Path::new(&args.r_path)).expect("R Path should be an image.").decode().unwrap().to_rgba8();
    let g_image = ImageReader::open(Path::new(&args.g_path)).expect("G Path should be an image.").decode().unwrap().to_rgba8();
    let b_image = ImageReader::open(Path::new(&args.b_path)).expect("B Path should be an image.").decode().unwrap().to_rgba8();
    let a_image = ImageReader::open(Path::new(&args.a_path)).expect("A Path should be an image.").decode().unwrap().to_rgba8();

    if r_image.width() != ( g_image.width() | b_image.width() | a_image.width() ) {
        panic!("Multi channels do not match one or more widths.");
    } else if r_image.height() != ( g_image.height() | b_image.height() | a_image.height() ) {
        panic!("Multi channels do not match one or more height.");
    }

    let mut multi = DynamicImage::new(r_image.width(), r_image.height() , image::ColorType::Rgba8);
    for w in 0..multi.width() {
        for h in 0..multi.height() {
            let r_pixel = r_image.get_pixel(w, h)[0];
            let g_pixel = g_image.get_pixel(w, h)[1];
            let b_pixel = b_image.get_pixel(w, h)[2];
            let a_pixel = a_image.get_pixel(w, h)[3];
            let pixel = Rgba::from([r_pixel, g_pixel, b_pixel, a_pixel]);
            multi.put_pixel(w, h, pixel);
        }
    }
    let save_path = Path::new(&args.r_path).file_prefix().unwrap().to_str().unwrap().to_string() + "GBA.png";
    multi.save_with_format(save_path, image::ImageFormat::Png).unwrap();
}

fn normal(args: NormalArgs) {
    let path = Path::new(&args.normal_path);
    let image = ImageReader::open(path).expect("Normal Path should be an image.");
    let binding = image.decode().unwrap().to_rgba8();
    let rgb = map_colors(&binding, |p| { Rgba([p[3], p[1], p[0], p[0]]) });
    let new_file = path.file_prefix().unwrap().to_str().unwrap().to_string() + "_Fixed.png";
    let save_path = path.with_file_name(new_file);
    rgb.save_with_format(save_path, image::ImageFormat::Png).unwrap();
}