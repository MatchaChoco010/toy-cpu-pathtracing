use clap::Parser;

pub mod camera;
pub mod filter;
pub mod math;
pub mod renderer;
pub mod sampler;
pub mod scene;
mod scene_loader;
pub mod spectrum;

use camera::Camera;
use filter::BoxFilter;
use renderer::{NormalRenderer, RendererArgs, RendererImage};
use sampler::RandomSamplerFactory;
use scene::create_scene;

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
    #[arg(long, default_value = "normal")]
    renderer: String,
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
    let mut camera = Camera::new(45.0, width, height, filter);

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

    // レンダラーを作成する。
    let renderer_args = RendererArgs {
        width,
        height,
        spp,
        scene: &scene,
        camera: &camera,
        sampler_factory: &sampler_factory,
    };
    let renderer = match args.renderer.as_str() {
        "normal" => NormalRenderer::new(renderer_args),
        _ => panic!("Invalid renderer"),
    };
    let mut image = RendererImage::new(width, height, renderer);

    // レンダリングを開始する。
    println!("Start rendering...");
    let start = std::time::Instant::now();

    image.render();

    let end = start.elapsed();
    println!("Finish rendering: {} seconds.", end.as_secs_f32());

    // 画像を保存する。
    image.save("output.png");
}
