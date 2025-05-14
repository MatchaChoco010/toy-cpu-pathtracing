use clap::Parser;

pub mod camera;
pub mod filter;
pub mod renderer;
pub mod sampler;
pub mod scene;
pub mod tone_map;

use camera::Camera;
use filter::BoxFilter;
use renderer::{NormalRenderer, RendererArgs, RendererImage, SrgbRendererNee, SrgbRendererPt};
use sampler::RandomSamplerFactory;
use tone_map::ReinhardToneMap;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Scene number to render
    #[arg(long, default_value_t = 0)]
    scene: u32,
    /// Number of samples per pixel
    #[arg(short, long, default_value_t = 64)]
    spp: u32,
    /// Filter type: [box, ]
    #[arg(long, default_value = "box")]
    filter: String,
    /// Sampler type: [random, ]
    #[arg(long, default_value = "random")]
    sampler: String,
    /// Renderer type: [normal, ]
    #[arg(long, default_value = "normal")]
    renderer: String,
    /// Output image width
    #[arg(short, long, default_value_t = 800)]
    width: u32,
    /// Output image height
    #[arg(short, long, default_value_t = 600)]
    height: u32,
    /// Maximum depth for the renderer
    #[arg(short = 'd', long, default_value_t = 16)]
    max_depth: usize,
    /// Output image file name
    #[arg(short, long, default_value = "output.png")]
    output: String,
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

    let mut scene = ::scene::create_scene!(scene);
    let mut camera = Camera::new(45.0, width, height, filter);

    match args.scene {
        0 => scene::load_scene_0(&mut scene, &mut camera),
        1 => scene::load_scene_1(&mut scene, &mut camera),
        2 => scene::load_scene_2(&mut scene, &mut camera),
        _ => panic!("Invalid scene number"),
    };

    let spp = args.spp;

    let max_depth = args.max_depth;

    let output = args.output;

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
    match args.renderer.as_str() {
        "normal" => {
            let renderer = NormalRenderer::new(renderer_args);
            let mut image = RendererImage::new(width, height, renderer);

            // レンダリングを開始する。
            println!("Start rendering...");
            let start = std::time::Instant::now();

            image.render();

            let end = start.elapsed();
            println!("Finish rendering: {} seconds.", end.as_secs_f32());

            // 画像を保存する。
            image.save(output);
        }
        "pt" => {
            let tone_map = ReinhardToneMap::new();
            let renderer = SrgbRendererPt::new(renderer_args, tone_map, 0.01, max_depth);
            let mut image = RendererImage::new(width, height, renderer);

            // レンダリングを開始する。
            println!("Start rendering...");
            let start = std::time::Instant::now();

            image.render();

            let end = start.elapsed();
            println!("Finish rendering: {} seconds.", end.as_secs_f32());

            // 画像を保存する。
            image.save(output);
        }
        "nee" => {
            let tone_map = ReinhardToneMap::new();
            let renderer = SrgbRendererNee::new(renderer_args, tone_map, 0.01, max_depth);
            let mut image = RendererImage::new(width, height, renderer);

            // レンダリングを開始する。
            println!("Start rendering...");
            let start = std::time::Instant::now();

            image.render();

            let end = start.elapsed();
            println!("Finish rendering: {} seconds.", end.as_secs_f32());

            // 画像を保存する。
            image.save(output);
        }
        _ => panic!("Invalid renderer"),
    };
}
