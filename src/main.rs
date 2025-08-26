#![feature(path_file_prefix)]

use clap::{Args, Parser, Subcommand};
use dialoguer::Select;
use image::{DynamicImage, GenericImage, ImageReader, Rgba};
use imageproc::map::*;
use core::panic;
use std::path::Path;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
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
        Some(Commands::SplitMulti(args)) => {
            split(args.multi_path);
        }
        Some(Commands::JoinMulti(args)) => {
            join(args.r_path, args.g_path, args.b_path, args.a_path);
        },
        Some(Commands::FixNormal(args)) => {
            normal(args.normal_path);
        },
        None => {
            let items = ["1 - Fix Normal", "2 - Split Multi", "3 - Join Multi"];
            let command = Select::new()
            .with_prompt("Choose a command (Use arrows and enter or space to confirm)")
            .items(&items)
            .default(0)
            .interact()
            .unwrap();
            match command {
                0 => {
                    let normal_path = native_dialog::FileDialog::new()
                    .set_title("Select Normal Map")
                    .add_filter("Image", &["png", "tga"])
                    .show_open_single_file()
                    .unwrap()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();

                    normal(normal_path);                        
                }

                1 => {
                    let multi_path = native_dialog::FileDialog::new()
                    .set_title("Select Multi Map")
                    .add_filter("Image", &["png", "tga"])
                    .show_open_single_file()
                    .unwrap()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();

                    split(multi_path);    
                }

                2 => {
                    let paths = native_dialog::FileDialog::new()
                    .set_title("Select Multi Map Channel Images")
                    .add_filter("Image", &["png", "tga"])
                    .show_open_multiple_file()
                    .unwrap();

                    if paths.len() != 4 {
                        panic!("Four images should be selected");
                    } else {
                        let r_path = paths.iter()
                        .find(|f| f.as_path().file_prefix().unwrap().to_str().unwrap().ends_with("_R"))
                        .expect("Should have a Red Channel Image.")                        
                        .to_str()
                        .unwrap()
                        .to_string();

                        let g_path = paths.iter()
                        .find(|f| f.as_path().file_prefix().unwrap().to_str().unwrap().ends_with("_G"))
                        .expect("Should have a Green Channel Image.")                        
                        .to_str()
                        .unwrap()
                        .to_string();

                        let b_path = paths.iter()
                        .find(|f| f.as_path().file_prefix().unwrap().to_str().unwrap().ends_with("_B"))
                        .expect("Should have a Blue Channel Image.")                        
                        .to_str()
                        .unwrap()
                        .to_string();
                    
                        let a_path = paths.iter()
                        .find(|f| f.as_path().file_prefix().unwrap().to_str().unwrap().ends_with("_A"))
                        .expect("Should have a Alpha Channel Image.")
                        .to_str()
                        .unwrap()
                        .to_string();

                        join(r_path, g_path, b_path, a_path);
                    }
                }

                _ => {
                    panic!("Invalid Command");
                }
            }
        }
    }
}

fn split(multi_path: String) {
    let path = Path::new(&multi_path);
    let image = ImageReader::open(path).expect("Multi Path should be an image.");
    let binding = image.decode().unwrap().to_rgba8();
    for x in 0..4 {
        let buf = map_colors(&binding, |p| Rgba([p[x], p[x], p[x], 255]));
        let ext = match x {
            0 => "_R",
            1 => "_G",
            2 => "_B",
            3 => "_A",
            _ => "_O"
           
        };
        let new_file = path.file_prefix().unwrap().to_str().unwrap().to_string() + ext + ".tga";
        let save_path = path.with_file_name(new_file);
        buf.save_with_format(save_path, image::ImageFormat::Tga).unwrap();
    }
}

fn join(r_path: String, g_path: String, b_path: String, a_path: String,) {
    let r_image = ImageReader::open(Path::new(&r_path)).expect("R Path should be an image.").decode().unwrap().to_rgba8();
    let g_image = ImageReader::open(Path::new(&g_path)).expect("G Path should be an image.").decode().unwrap().to_rgba8();
    let b_image = ImageReader::open(Path::new(&b_path)).expect("B Path should be an image.").decode().unwrap().to_rgba8();
    let a_image = ImageReader::open(Path::new(&a_path)).expect("A Path should be an image.").decode().unwrap().to_rgba8();

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
    let save_path = Path::new(&r_path).file_prefix().unwrap().to_str().unwrap().to_string() + "GBA.tga";
    multi.save_with_format(save_path, image::ImageFormat::Tga).unwrap();
}

fn normal(normal_path: String) {
    let path = Path::new(&normal_path);
    let image = ImageReader::open(path).expect("Normal Path should be an image.");
    let binding = image.decode().unwrap().to_rgba8();
    let rgb = map_colors(&binding, |p| { Rgba([p[3], p[1], p[0], p[0]]) });
    let new_file = path.file_prefix().unwrap().to_str().unwrap().to_string() + "_Fixed.png";
    let save_path = path.with_file_name(new_file);
    rgb.save_with_format(save_path, image::ImageFormat::Png).unwrap();
}