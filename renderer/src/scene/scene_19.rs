//! シーン19: Environment Light test

use std::path::PathBuf;

use color::ColorSrgbLinear;
use color::{ColorSrgb, tone_map::NoneToneMap};
use math::{Point3, Transform, Vector3};
use scene::{
    CreatePrimitiveDesc, FloatParameter, FloatTexture, LambertMaterial, NormalParameter,
    NormalTexture, PlasticMaterial, RgbTexture, SimpleClearcoatPbrMaterial, SimplePbrMaterial,
    SpectrumParameter, SpectrumType,
};
use spectrum::RgbAlbedoSpectrum;

use crate::{camera::Camera, filter::Filter};

pub fn load_scene_19<Id: scene::SceneId, F: Filter>(
    scene: &mut scene::Scene<Id>,
    camera: &mut Camera<F>,
) {
    // Environment Light test scene

    // 床オブジェクト (yuka.obj)
    let floor_geom = scene.load_obj("./renderer/assets/yuka.obj");

    // 床の材質（ライトグレー）
    let floor_color =
        RgbAlbedoSpectrum::<ColorSrgb<NoneToneMap>>::new(ColorSrgb::new(0.8, 0.8, 0.8));

    // 床をシーンに追加
    let _floor_primitive = scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: floor_geom,
        surface_material: LambertMaterial::new(
            SpectrumParameter::constant(floor_color),
            NormalParameter::none(),
        ),
        transform: Transform::identity(),
    });

    // ドラゴンモデル
    let dragon_geom = scene.load_obj("./renderer/assets/dragon.min.obj");

    // BaseColorテクスチャ
    let base_color_texture =
        RgbTexture::load_srgb("./renderer/assets/dragon-material/BaseColor.png")
            .expect("Failed to load base color texture");
    let base_color_param = SpectrumParameter::texture(base_color_texture, SpectrumType::Albedo);

    // Metallicテクスチャ
    let metallic_texture =
        FloatTexture::load("./renderer/assets/dragon-material/Metallic.png", false)
            .expect("Failed to load metallic texture");
    let metallic_param = FloatParameter::texture(metallic_texture);

    // Roughnessテクスチャ
    let roughness_texture =
        FloatTexture::load("./renderer/assets/dragon-material/Roughness.png", false)
            .expect("Failed to load roughness texture");
    let roughness_param = FloatParameter::texture(roughness_texture);

    // ノーマルマップテクスチャ
    let normal_texture = NormalTexture::load("./renderer/assets/dragon-material/Normal.png", false)
        .expect("Failed to load normal texture");
    let normal_param = NormalParameter::texture(normal_texture);

    // 屈折率設定
    let ior_param = FloatParameter::constant(1.5);

    let _dragon_primitive = scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: dragon_geom,
        surface_material: SimplePbrMaterial::new(
            base_color_param,
            metallic_param,
            roughness_param,
            normal_param,
            ior_param,
        ),
        transform: Transform::identity(),
    });

    // 銀色のベースカラー
    let base_color_spectrum =
        RgbAlbedoSpectrum::<ColorSrgb<NoneToneMap>>::new(ColorSrgb::new(0.8, 0.8, 0.8));
    let base_color_param = SpectrumParameter::constant(base_color_spectrum);

    // metallic = 1.0 (完全金属)
    let metallic_param = FloatParameter::constant(1.0);

    // roughness
    let roughness_param = FloatParameter::constant(0.7);

    // ior
    let ior_param = FloatParameter::constant(1.5);

    // clearcoat設定
    let clearcoat_ior_param = FloatParameter::constant(1.5);
    let clearcoat_roughness_param = FloatParameter::constant(0.01);

    // 青いtint
    let clearcoat_tint_spectrum =
        RgbAlbedoSpectrum::<ColorSrgb<NoneToneMap>>::new(ColorSrgb::new(0.7, 0.8, 1.0));
    let clearcoat_tint_param = SpectrumParameter::constant(clearcoat_tint_spectrum);

    // thickness（0.8mm）
    let clearcoat_thickness_param = FloatParameter::constant(0.8);

    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: dragon_geom,
        surface_material: SimpleClearcoatPbrMaterial::new(
            base_color_param,
            metallic_param,
            roughness_param,
            NormalParameter::none(),
            ior_param,
            clearcoat_ior_param,
            clearcoat_roughness_param,
            clearcoat_tint_param,
            clearcoat_thickness_param,
        ),
        transform: Transform::identity().translate(glam::vec3(0.5, 0.0, 0.5)),
    });

    // 青いプラスチック
    let color_spectrum =
        RgbAlbedoSpectrum::<ColorSrgbLinear<NoneToneMap>>::new(ColorSrgbLinear::new(0.4, 0.9, 1.0));

    scene.create_primitive(CreatePrimitiveDesc::GeometryPrimitive {
        geometry_index: dragon_geom,
        surface_material: PlasticMaterial::new(
            1.5,
            SpectrumParameter::constant(color_spectrum),
            NormalParameter::none(),
            false,
            FloatParameter::constant(0.0),
        ),
        transform: Transform::identity().translate(glam::vec3(-0.5, 0.0, -0.5)),
    });

    // Environment Light を追加
    let _environment_light =
        scene.create_primitive(CreatePrimitiveDesc::EnvironmentLightPrimitive {
            intensity: 1.0,
            texture_path: PathBuf::from("./renderer/assets/sky/scythian_tombs_2_1k.exr"),
            transform: Transform::identity(),
        });

    // カメラ設定 - ドラゴンを正面から斜めに映す
    camera.set_look_to(
        Point3::new(-1.5, 0.8, 2.5),
        Vector3::new(1.5, -0.4, -2.5).normalize(),
        Vector3::new(0.0, 1.0, 0.0),
    );
}
