//! プリミティブを保持するリポジトリの実装。

use std::marker::PhantomData;

use math::{Render, Transform, World};

use crate::{
    SceneId,
    geometry::{self, GeometryRepository},
    primitive::{self, CreatePrimitiveDesc, traits::Primitive},
};

/// PrimitiveRepositoryに登録したPrimitiveのIndex。
/// PrimitiveRepositoryからこのIndexを使ってPrimitiveを取得できる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrimitiveIndex<Id: SceneId>(pub usize, PhantomData<Id>);
impl<Id: SceneId> PrimitiveIndex<Id> {
    /// 新しいPrimitiveIndexを作成する。
    pub fn new(index: usize) -> Self {
        Self(index, PhantomData)
    }
}

/// プリミティブを保持するリポジトリの構造体。
pub struct PrimitiveRepository<Id: SceneId> {
    primitives: Vec<Box<dyn Primitive<Id>>>,
}
impl<Id: SceneId> PrimitiveRepository<Id> {
    /// 新しいプリミティブリポジトリを作成する。
    pub fn new() -> Self {
        Self {
            primitives: Vec::new(),
        }
    }

    /// プリミティブをCreatePrimitiveDescから作成し登録する。
    pub fn create_primitive(
        &mut self,
        geometry_repository: &GeometryRepository<Id>,
        desc: CreatePrimitiveDesc<Id>,
    ) -> PrimitiveIndex<Id> {
        let primitive_index = PrimitiveIndex(self.primitives.len(), PhantomData);
        let primitive: Box<dyn Primitive<Id>> = match desc {
            CreatePrimitiveDesc::GeometryPrimitive {
                geometry_index,
                surface_material,
                transform,
            } => {
                let geometry = geometry_repository.get(geometry_index);

                if geometry.as_any().is::<geometry::impls::TriangleMesh<Id>>() {
                    // ジオメトリがTriangleMeshの場合。
                    let triangle_mesh = geometry
                        .as_any()
                        .downcast_ref::<geometry::impls::TriangleMesh<Id>>()
                        .unwrap();

                    if surface_material.edf.is_some() {
                        Box::new(primitive::impls::EmissiveTriangleMesh::new(
                            triangle_mesh,
                            geometry_index,
                            surface_material,
                            transform,
                        ))
                    } else {
                        Box::new(primitive::impls::TriangleMesh::new(
                            geometry_index,
                            surface_material,
                            transform,
                        ))
                    }
                } else {
                    // 未実装のジオメトリ。
                    unimplemented!("Geometry type not supported");
                }
            }
            CreatePrimitiveDesc::SingleTrianglePrimitive {
                positions,
                normals,
                uvs,
                surface_material,
                transform,
            } => {
                if surface_material.edf.is_some() {
                    Box::new(primitive::impls::EmissiveSingleTriangle::new(
                        positions,
                        normals,
                        uvs,
                        surface_material,
                        transform,
                    ))
                } else {
                    Box::new(primitive::impls::SingleTriangle::new(
                        positions,
                        normals,
                        uvs,
                        surface_material,
                        transform,
                    ))
                }
            }
            CreatePrimitiveDesc::PointLightPrimitive {
                intensity,
                spectrum,
                transform,
            } => Box::new(primitive::impls::PointLight::new(
                intensity, spectrum, transform,
            )),
            CreatePrimitiveDesc::SpotLightPrimitive {
                angle_inner,
                angle_outer,
                intensity,
                spectrum,
                transform,
            } => Box::new(primitive::impls::SpotLight::new(
                angle_inner,
                angle_outer,
                intensity,
                spectrum,
                transform,
            )),
            CreatePrimitiveDesc::DirectionalLightPrimitive {
                intensity,
                spectrum,
                transform,
            } => Box::new(primitive::impls::DirectionalLight::new(
                intensity, spectrum, transform,
            )),
            CreatePrimitiveDesc::EnvironmentLightPrimitive {
                intensity,
                texture_path,
                transform,
            } => Box::new(primitive::impls::EnvironmentLight::new(
                intensity,
                texture_path,
                transform,
            )),
        };
        self.primitives.push(primitive);
        primitive_index
    }

    /// プリミティブを取得する。
    pub fn get(&self, index: PrimitiveIndex<Id>) -> &Box<dyn Primitive<Id>> {
        &self.primitives[index.0]
    }

    /// プリミティブを可変参照で取得する。
    pub fn get_mut(&mut self, index: PrimitiveIndex<Id>) -> &mut Box<dyn Primitive<Id>> {
        &mut self.primitives[index.0]
    }

    /// プリミティブのインデックスのイテレーターを取得する。
    pub fn get_all_primitive_indices(&self) -> impl Iterator<Item = PrimitiveIndex<Id>> {
        (0..self.primitives.len()).map(move |i| PrimitiveIndex::new(i))
    }

    /// world_to_renderの座標変換を更新する。
    pub fn update_world_to_render(&mut self, world_to_render: &Transform<World, Render>) {
        for primitive in &mut self.primitives {
            primitive.update_world_to_render(world_to_render);
        }
    }
}
