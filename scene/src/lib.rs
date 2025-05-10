mod bvh;
mod geometry;
mod light_sampler;
mod material;
mod primitive;
mod scene;

pub use geometry::{Geometry, GeometryIndex};
pub use material::MaterialId;
pub use primitive::{
    CreatePrimitiveDesc, InteractGeometryInfo, Interaction, Intersection, LightIrradiance,
    LightSampleRadiance, PrimitiveBvh, PrimitiveIndex,
};
pub use scene::{Scene, SceneId, WorldToRender, internal};
