//! プリミティブを保持するリポジトリの実装。

use std::marker::PhantomData;

use math::{Render, Transform, World};

use crate::scene::{
    Geometry, GeometryRepository, SceneId,
    primitive::{CreatePrimitiveDesc, Primitive, impls::*},
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
    primitives: Vec<Primitive<Id>>,
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
        let primitive = match desc {
            CreatePrimitiveDesc::GeometryPrimitive {
                geometry_index,
                material_id,
                transform,
            } => {
                let geometry = geometry_repository.get(geometry_index);
                match geometry {
                    Geometry::TriangleMesh(_) => {
                        // TODO: materialがemissiveか判断する
                        Primitive::TriangleMesh(TriangleMesh::new(
                            geometry_index,
                            material_id,
                            transform,
                        ))
                    }
                }
            }
            CreatePrimitiveDesc::SingleTrianglePrimitive {
                positions,
                normals,
                uvs,
                material_id,
                transform,
            } => {
                // TODO: materialがemissiveか判断する
                Primitive::SingleTriangle(SingleTriangle::new(
                    positions,
                    normals,
                    uvs,
                    material_id,
                    transform,
                ))
            }
            CreatePrimitiveDesc::PointLightPrimitive {
                intensity,
                transform,
            } => Primitive::PointLight(PointLight::new(intensity, transform)),
            CreatePrimitiveDesc::SpotLightPrimitive {
                angle,
                intensity,
                transform,
            } => Primitive::SpotLight(SpotLight::new(angle, intensity, transform)),
            CreatePrimitiveDesc::DirectionalLightPrimitive {
                intensity,
                transform,
            } => Primitive::DirectionalLight(DirectionalLight::new(intensity, transform)),
            CreatePrimitiveDesc::EnvironmentLightPrimitive {
                intensity,
                texture_path,
                transform,
            } => Primitive::EnvironmentLight(EnvironmentLight::new(
                intensity,
                texture_path,
                transform,
            )),
        };
        self.primitives.push(primitive);
        primitive_index
    }

    /// プリミティブを取得する。
    pub fn get(&self, index: PrimitiveIndex<Id>) -> &Primitive<Id> {
        &self.primitives[index.0]
    }

    /// プリミティブを可変参照で取得する。
    pub fn get_mut(&mut self, index: PrimitiveIndex<Id>) -> &mut Primitive<Id> {
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
