use clap::Parser;
use image::{Rgb, RgbImage};
use rayon::prelude::*;

pub mod camera;
pub mod filter;
pub mod math;
pub mod sampler;
pub mod scene;
mod scene_loader;
pub mod spectrum;

use camera::Camera;
use filter::BoxFilter;
use sampler::{RandomSamplerFactory, Sampler, SamplerFactory};
use scene::{Interaction, create_scene};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t = 0)]
    scene: u32,
    #[arg(short, long, default_value_t = 64)]
    spp: u32,
    #[arg(long, default_value = "box")]
    filter: String,
    #[arg(long, default_value = "random")]
    sampler: String,
    #[arg(short, long, default_value_t = 800)]
    width: u32,
    #[arg(short, long, default_value_t = 600)]
    height: u32,
}

fn main() {
    // コマンドライン引数をパースする。
    let args = Args::parse();

    let filter = match args.filter.as_str() {
        "box" => BoxFilter::new(1.0),
        _ => panic!("Invalid filter"),
    };

    let sampler_factory = match args.sampler.as_str() {
        "random" => RandomSamplerFactory::new(),
        _ => panic!("Invalid sampler"),
    };

    let width = args.width;
    let height = args.height;

    let mut scene = create_scene!(scene);
    let mut camera = Camera::new(45.0, (width, height), filter);

    match args.scene {
        0 => scene_loader::load_scene_0(&mut scene, &mut camera),
        1 => scene_loader::load_scene_1(&mut scene, &mut camera),
        _ => panic!("Invalid scene number"),
    };

    let spp = args.spp;

    // シーンのビルド。
    println!("Start build scene...");
    let start = std::time::Instant::now();

    scene.build(&camera);

    let end = start.elapsed();
    println!("Finish build scene: {} seconds.", end.as_secs_f32());

    // レンダリングを開始する。
    println!("Start rendering...");
    let start = std::time::Instant::now();

    let mut img = RgbImage::new(width, height);
    img.enumerate_pixels_mut()
        .collect::<Vec<(u32, u32, &mut Rgb<u8>)>>()
        .par_iter_mut()
        .for_each({
            let sampler_factory = sampler_factory.clone();
            move |(x, y, pixel)| {
                let mut sampler = sampler_factory.create_sampler(*x, *y);

                let mut acc_color = glam::Vec3::ZERO;
                for dimension in 0..spp {
                    sampler.start_pixel_sample(dimension);

                    let uv = sampler.get_2d_pixel();
                    let rs = camera.sample_ray(*x, *y, uv);

                    let intersect = scene.intersect(&rs.ray, f32::MAX);

                    let color = match intersect {
                        Some(intersect) => match intersect.interaction {
                            Interaction::Surface { shading_normal, .. } => {
                                shading_normal.to_vec3() * 0.5 + glam::Vec3::splat(0.5)
                            }
                        },
                        None => glam::Vec3::ZERO,
                    };

                    acc_color += color * rs.weight;
                }
                let color = acc_color / spp as f32;

                pixel[0] = (color.x * 255.0) as u8;
                pixel[1] = (color.y * 255.0) as u8;
                pixel[2] = (color.z * 255.0) as u8;
            }
        });

    let end = start.elapsed();
    println!("Finish rendering: {} seconds.", end.as_secs_f32());

    // 画像を保存する。
    img.save("output.png").unwrap();
}
