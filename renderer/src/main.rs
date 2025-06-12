use clap::Parser;

pub mod camera;
pub mod filter;
pub mod renderer;
pub mod sampler;
pub mod scene;
pub mod tone_map;

use camera::Camera;
use filter::BoxFilter;
use renderer::{
    NormalRenderer, RendererArgs, RendererImage, SrgbRendererMis, SrgbRendererNee, SrgbRendererPt,
};
use sampler::{RandomSampler, ZSobolSampler};
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
    /// Seed for the random number generator
    #[arg(long, default_value_t = 0)]
    seed: u32,
    /// Filter type: [box, ]
    #[arg(long, default_value = "box")]
    filter: String,
    /// Sampler type: [random, sobol]
    #[arg(long, default_value = "random")]
    sampler: String,
    /// Renderer type: [normal, ]
    #[arg(long, default_value = "normal")]
    renderer: String,
    /// Output image width
    #[arg(long, default_value_t = 800)]
    width: u32,
    /// Output image height
    #[arg(long, default_value_t = 600)]
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

    let width = args.width;
    let height = args.height;

    let mut scene = ::scene::create_scene!(scene);
    let mut camera = Camera::new(45.0, width, height, filter);

    match args.scene {
        0 => scene::load_scene_0(&mut scene, &mut camera),
        1 => scene::load_scene_1(&mut scene, &mut camera),
        2 => scene::load_scene_2(&mut scene, &mut camera),
        3 => scene::load_scene_3(&mut scene, &mut camera),
        4 => scene::load_scene_4(&mut scene, &mut camera),
        5 => scene::load_scene_5(&mut scene, &mut camera),
        _ => panic!("Invalid scene number"),
    };

    let spp = args.spp;

    let seed = args.seed;

    let max_depth = args.max_depth;

    let output = args.output;

    // シーンのビルド。
    println!("Start build scene...");
    let start = std::time::Instant::now();

    scene.build(&camera);

    let end = start.elapsed();
    println!("Finish build scene: {} seconds.", end.as_secs_f32());

    match args.sampler.as_str() {
        "random" => {
            render_with_sampler::<_, _, RandomSampler>(
                args.renderer.as_str(),
                width,
                height,
                spp,
                seed,
                &scene,
                &camera,
                max_depth,
                output,
            );
        }
        "sobol" => {
            render_with_sampler::<_, _, ZSobolSampler>(
                args.renderer.as_str(),
                width,
                height,
                spp,
                seed,
                &scene,
                &camera,
                max_depth,
                output,
            );
        }
        _ => panic!("Invalid sampler type"),
    }
}

fn render_with_sampler<Id: ::scene::SceneId, F: filter::Filter, S: sampler::Sampler>(
    renderer_type: &str,
    width: u32,
    height: u32,
    spp: u32,
    seed: u32,
    scene: &::scene::Scene<Id>,
    camera: &Camera<F>,
    max_depth: usize,
    output: String,
) {
    let renderer_args = RendererArgs {
        resolution: glam::uvec2(width, height),
        spp,
        scene,
        camera,
        seed,
    };

    match renderer_type {
        "normal" => {
            let renderer = NormalRenderer::new(renderer_args);
            let mut image = RendererImage::new(width, height, renderer);

            println!("Start rendering...");
            let start = std::time::Instant::now();

            image.render::<S>();

            let end = start.elapsed();
            println!("Finish rendering: {} seconds.", end.as_secs_f32());

            image.save(output);
        }
        "pt" => {
            let tone_map = ReinhardToneMap::new();
            let renderer = SrgbRendererPt::new(renderer_args, tone_map, 0.01, max_depth);
            let mut image = RendererImage::new(width, height, renderer);

            println!("Start rendering...");
            let start = std::time::Instant::now();

            image.render::<S>();

            let end = start.elapsed();
            println!("Finish rendering: {} seconds.", end.as_secs_f32());

            image.save(output);
        }
        "nee" => {
            let tone_map = ReinhardToneMap::new();
            let renderer = SrgbRendererNee::new(renderer_args, tone_map, 0.01, max_depth);
            let mut image = RendererImage::new(width, height, renderer);

            println!("Start rendering...");
            let start = std::time::Instant::now();

            image.render::<S>();

            let end = start.elapsed();
            println!("Finish rendering: {} seconds.", end.as_secs_f32());

            image.save(output);
        }
        "mis" => {
            let tone_map = ReinhardToneMap::new();
            let renderer = SrgbRendererMis::new(renderer_args, tone_map, 0.01, max_depth);
            let mut image = RendererImage::new(width, height, renderer);

            println!("Start rendering...");
            let start = std::time::Instant::now();

            image.render::<S>();

            let end = start.elapsed();
            println!("Finish rendering: {} seconds.", end.as_secs_f32());

            image.save(output);
        }
        _ => panic!("Invalid renderer"),
    }
}
