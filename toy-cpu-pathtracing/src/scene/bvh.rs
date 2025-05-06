use std::marker::PhantomData;

use crate::math::{Bounds, CoordinateSystem, Ray};

/// BVHのヒット情報を表す構造体。
pub struct HitInfo<Intersection> {
    pub intersection: Intersection,
    pub t_hit: f32,
}

/// BVHのアイテムの外部データを表すトレイト。
/// アイテムがインデックスのみを保持する場合、
/// 実データのバウンディングボックスや交差の計算には外部のジオメトリデータなどが必要になる。
/// それらBVHのアイテムの計算の際に渡す参照用の実データを表している。
pub trait BvhItemData<Item> {
    /// アイテムのイテレーターを生成して返す。
    fn item_list(&self) -> impl Iterator<Item = Item>;
}

/// BVHのアイテムを表すトレイト。
/// 外部のジオメトリデータなどを渡して、
/// そのデータを参照しながらバウンディングボックスや交差を計算する。
pub trait BvhItem<C: CoordinateSystem>: Clone + Copy {
    /// BVHのアイテムの外部データを表す型。
    type Data<'a>: BvhItemData<Self>
    where
        Self: 'a;

    /// BVHのアイテムとレイの交差情報を表す型。
    type Intersection;

    /// アイテムのバウンディングボックスを計算する。
    fn bounds<'a>(&self, data: &Self::Data<'a>) -> Bounds<C>
    where
        Self: 'a;

    /// アイテムとレイの交差判定を行う。
    fn intersect<'a>(
        &self,
        data: &Self::Data<'a>,
        ray: &Ray<C>,
        t_max: f32,
    ) -> Option<HitInfo<Self::Intersection>>
    where
        Self: 'a;
}

/// BVHのビルド中に使う、BvhItemのリストの分割結果を表す構造体。
/// 分割された左側と右側のアイテムリストと、SAHのコストを持つ。
struct SplitInfo<C: CoordinateSystem, Item: BvhItem<C>> {
    first: BvhItemList<C, Item>,
    second: BvhItemList<C, Item>,
    cost: f32,
}

/// BVHのビルド中に使うBVHのアイテムリストを表す構造体。
struct BvhItemList<C: CoordinateSystem, I: BvhItem<C>> {
    items: Vec<I>,
    _coordinate_system: PhantomData<C>,
}
impl<C: CoordinateSystem, Item: BvhItem<C>> BvhItemList<C, Item> {
    /// 新しいBvhItemListを作成する。
    fn new(items: Vec<Item>) -> Self {
        Self {
            items,
            _coordinate_system: PhantomData,
        }
    }

    /// アイテムリストの数を返す。
    fn len(&self) -> usize {
        self.items.len()
    }

    /// アイテムリスト全体のバウンディングボックスを計算する。
    fn bounds<'a>(&self, data: &Item::Data<'a>) -> Bounds<C> {
        let mut bounds = self.items[0].bounds(data);
        for item in &self.items[1..] {
            bounds = bounds.merge(item.bounds(data));
        }
        bounds
    }

    /// アイテムリストを軸に沿ってSAHを最小にするインデックスで2つに分割する。
    fn split<'a>(
        &self,
        data: &Item::Data<'a>,
        axis: usize,
        parent_area: f32,
    ) -> SplitInfo<C, Item> {
        let mut items = self.items.clone();
        assert!(items.len() >= 2);

        // アイテムリストを軸に沿ってソートする。
        items.sort_by(|a, b| {
            let a = a.bounds(data).center().to_vec3()[axis];
            let b = b.bounds(data).center().to_vec3()[axis];
            a.partial_cmp(&b).unwrap()
        });

        // 分割した際の最小コストを記録し、最小になる分割のfirstとsecondを記録する。
        let mut min_cost = f32::INFINITY;
        let mut min_first = None;
        let mut min_second = None;

        // ソートしたアイテムリストを前から順に分割していく。
        for i in 1..(items.len()) {
            let mut list0 = items.clone();
            let list1 = list0.split_off(i);
            let items0 = BvhItemList::new(list0);
            let items1 = BvhItemList::new(list1);

            // 分割したアイテムリストのバウンディングボックスをそれぞれ計算する。
            let bounds0 = items0.bounds(data);
            let bounds1 = items1.bounds(data);

            // 分割した際のSAHのコストを計算する。
            let cost = BvhBuilder::<C, Item>::COST_NODE
                + BvhBuilder::<C, Item>::COST_LEAF * bounds0.area() / parent_area
                    * items0.len() as f32
                + BvhBuilder::<C, Item>::COST_LEAF * bounds1.area() / parent_area
                    * items1.len() as f32;

            // 分割した際のコストがこれまでより小さい場合は、コストと分割を更新する。
            if cost < min_cost {
                min_cost = cost;
                min_first = Some(items0);
                min_second = Some(items1);
            }
        }
        SplitInfo {
            first: min_first.unwrap(),
            second: min_second.unwrap(),
            cost: min_cost,
        }
    }
}

/// BVHのビルド中に使う中間データの列挙型。
enum BvhBuilder<C: CoordinateSystem, Item: BvhItem<C>> {
    /// BVHのノードの節を表す。
    /// ノードのバウンディングボックスと、2つの子ノードを持つ。
    Node {
        bounds: Bounds<C>,
        first: Box<BvhBuilder<C, Item>>,
        second: Box<BvhBuilder<C, Item>>,
    },
    /// BVHのノードの葉を表す。
    /// リーフノードのバウンディングボックスと、アイテムリストを持つ。
    Leaf {
        bounds: Bounds<C>,
        item_list: BvhItemList<C, Item>,
    },
}
impl<C: CoordinateSystem, Item: BvhItem<C>> BvhBuilder<C, Item> {
    /// SAHのリーフに対するコスト。
    const COST_LEAF: f32 = 1.0;
    /// SAHのノードに対するコスト。
    const COST_NODE: f32 = 1.0;

    /// BVHのビルド用中間データを作成する。
    fn new<'a>(data: &Item::Data<'a>) -> Self {
        let item_list = data.item_list().collect::<Vec<_>>();
        let item_list = BvhItemList::new(item_list);
        let bounds = item_list.bounds(data);
        BvhBuilder::Leaf { bounds, item_list }
    }

    /// BVHをビルドする。
    /// リーフノードをSAHを基準に必要なだけ分割をした結果を返す。
    fn build<'a>(self, data: &Item::Data<'a>) -> BvhBuilder<C, Item> {
        match self {
            BvhBuilder::Leaf { bounds, item_list } => {
                // アイテムが1つ以下の場合はリーフノードとする。
                if item_list.len() <= 1 {
                    return BvhBuilder::Leaf { bounds, item_list };
                }

                // 分割しなかったときのコストをmin_costの初期値とする。
                let mut min_cost = BvhBuilder::<C, Item>::COST_LEAF * item_list.len() as f32;
                let mut min_split = None;

                // x, y, zの各軸でSAHに応じて分割する。
                for axis in 0..3 {
                    let SplitInfo {
                        first,
                        second,
                        cost,
                    } = item_list.split(data, axis, bounds.area());
                    let first = BvhBuilder::Leaf {
                        bounds: first.bounds(data),
                        item_list: first,
                    };
                    let second = BvhBuilder::Leaf {
                        bounds: second.bounds(data),
                        item_list: second,
                    };
                    // この分割が既存のmin_costよりも小さい場合は、分割を更新する。
                    if cost < min_cost {
                        min_cost = cost;
                        min_split = Some((first, second));
                    }
                }

                // min_costが一番小さい分割を選択する。
                // 分割したほうがコストが小さい場合は、そのリーフノードを再帰的にビルドして分割して、
                // その結果をfirstとsecondに格納したNodeを返す。
                // 分割しない場合が一番コストが低い場合は、そのままLeafとして返す。
                if let Some((first, second)) = min_split {
                    let first = first.build(data);
                    let second = second.build(data);
                    return BvhBuilder::Node {
                        bounds,
                        first: Box::new(first),
                        second: Box::new(second),
                    };
                } else {
                    return BvhBuilder::Leaf { bounds, item_list };
                }
            }
            _ => panic!("Already built"),
        }
    }

    /// BVH構築用中間データをフラット化してBVHを作成する。
    /// buildを呼び出した後に呼び出すことを想定している。
    fn flatten(self) -> Bvh<C, Item> {
        // 順次ノードをたどりながら、ノードをフラットな配列に格納していく。
        fn traverse<'a, C: CoordinateSystem, Item: BvhItem<C>>(
            builder: BvhBuilder<C, Item>,
            nodes: &mut Vec<BvhNode<C, Item>>,
            index: &mut usize,
        ) {
            match builder {
                BvhBuilder::Node {
                    bounds,
                    first,
                    second,
                } => {
                    // Nodeを格納する。
                    let node_index = *index;
                    nodes.push(BvhNode::Node {
                        bounds,
                        second_offset: 0, // 仮の値
                    });

                    *index += 1;

                    // 左側のノードをフラット化し格納していく。
                    traverse(*first, nodes, index);

                    // 左側のノードが終わった時点でのindexを元にNodeのsecond_offsetを計算する。
                    let node = &mut nodes[node_index];
                    match node {
                        BvhNode::Node { second_offset, .. } => {
                            *second_offset = *index as u32 - node_index as u32;
                        }
                        _ => unreachable!(),
                    }

                    // 右側のノードをフラット化し格納していく。
                    traverse(*second, nodes, index);
                }
                BvhBuilder::Leaf { bounds, item_list } => {
                    // 格納するノードの分だけindexを進める。
                    *index += item_list.len() + 1;

                    // Leafを格納する。
                    // Leafにぶら下がっているアイテムの数を記録する。
                    nodes.push(BvhNode::Leaf {
                        bounds,
                        item_count: item_list.len() as u32,
                    });

                    // Leafにぶら下がっているアイテムを続けて全部格納する。
                    for item in item_list.items {
                        nodes.push(BvhNode::Item { item });
                    }
                }
            }
        }

        let mut nodes = vec![];
        let mut index = 0;
        traverse(self, &mut nodes, &mut index);

        Bvh { nodes }
    }
}

/// BVHのノードを表す列挙型。
#[derive(Debug, Clone)]
pub enum BvhNode<C: CoordinateSystem, Item: BvhItem<C>> {
    /// BVHのノードを表す。
    /// ノードのバウンディングボックスと、2つの子ノードを持つ。
    /// 1つめのノードはこのノードの直後に格納されている。
    /// 2つめのノードは、second_offsetで指定されたオフセットの位置に格納されている。
    Node {
        bounds: Bounds<C>,
        second_offset: u32,
    },
    /// BVHのリーフを表す。
    /// リーフノードのバウンディングボックスと、アイテムの数を持つ。
    /// アイテムはこのノードの直後にアイテムの数だけ連続して格納されている。
    Leaf { bounds: Bounds<C>, item_count: u32 },
    /// BVHのアイテムを表す。
    Item { item: Item },
}

/// BVHを表す構造体。
/// BVHのノードをフラットな配列として持つ。
pub struct Bvh<C: CoordinateSystem, Item: BvhItem<C>> {
    nodes: Vec<BvhNode<C, Item>>,
}
impl<C: CoordinateSystem, Item: BvhItem<C>> Bvh<C, Item> {
    /// 新しいBVHを作成する。
    pub fn build<'a>(data: &Item::Data<'a>) -> Self {
        // BVHのビルド用中間データを作成する。
        let builder = BvhBuilder::<C, Item>::new(data);
        // BVHをビルドする。
        let builder = builder.build(data);
        // ビルドしたBVHをフラット化して返す。
        builder.flatten()
    }

    /// バウンディングボックスを取得する。
    pub fn bounds(&self) -> Bounds<C> {
        match &self.nodes[0] {
            BvhNode::Node { bounds, .. } => bounds.clone(),
            BvhNode::Leaf { bounds, .. } => bounds.clone(),
            _ => unreachable!(),
        }
    }

    /// 交差判定を行う。
    pub fn intersect<'a>(
        &self,
        data: &Item::Data<'a>,
        ray: &Ray<C>,
        t_max: f32,
    ) -> Option<Item::Intersection> {
        // ルートノードから再帰的に探索を行う。
        // flat化されたノードをindexでたどることで、ノードを探索する。
        fn traverse<'a, C: CoordinateSystem, Item: BvhItem<C>>(
            data: &Item::Data<'a>,
            nodes: &Vec<BvhNode<C, Item>>,
            index: usize,
            ray: &Ray<C>,
            t_max: f32,
            inv_dir: glam::Vec3,
        ) -> Option<HitInfo<Item::Intersection>> {
            let item = &nodes[index];
            match item {
                BvhNode::Node {
                    bounds,
                    second_offset,
                } => {
                    // ノードのバウンディングボックスとレイの交差判定を行い、交差していなければスキップ。
                    if bounds.intersect(ray, t_max, inv_dir).is_none() {
                        return None;
                    }

                    // firstとsecondのノードのそれぞれの交差判定を行う。
                    let first = traverse(data, nodes, index + 1, ray, t_max, inv_dir);
                    let second = traverse(
                        data,
                        nodes,
                        index + *second_offset as usize,
                        ray,
                        t_max,
                        inv_dir,
                    );

                    // 交差判定のうち近い方を返す。
                    if first.is_some() && second.is_some() {
                        let first = first.unwrap();
                        let second = second.unwrap();
                        if first.t_hit < second.t_hit {
                            return Some(first);
                        } else {
                            return Some(second);
                        }
                    } else if first.is_some() {
                        return first;
                    } else if second.is_some() {
                        return second;
                    } else {
                        return None;
                    }
                }
                BvhNode::Leaf { bounds, item_count } => {
                    // ノードのバウンディングボックスとレイの交差判定を行い、交差していなければスキップ。
                    if bounds.intersect(ray, t_max, inv_dir).is_none() {
                        return None;
                    }

                    // 交差のうち最も近いものを保持する。
                    let mut min_intersection: Option<HitInfo<Item::Intersection>> = None;

                    // アイテムの数だけループして、交差判定を行う。
                    for i in 1..(*item_count + 1) {
                        let item = match &nodes[index + i as usize] {
                            BvhNode::Item { item } => *item,
                            _ => unreachable!(),
                        };
                        let intersection = item.intersect(data, ray, t_max);

                        // 交差判定の結果がある場合は、過去の交差点と比べて近ければそちらに更新する。
                        if let Some(intersection) = intersection {
                            if min_intersection.is_some() {
                                if intersection.t_hit < min_intersection.as_ref().unwrap().t_hit {
                                    min_intersection = Some(intersection);
                                }
                            } else {
                                min_intersection = Some(intersection);
                            }
                        }
                    }

                    // 最も近い交差判定を返す。
                    // 交差しなかった場合はNoneを返す。
                    min_intersection
                }
                _ => unreachable!(),
            }
        }

        // レイのヒット計算で利用する、レイの方向ベクトルの要素ごとの逆数を計算する。
        let inv_dir = 1.0 / ray.dir.to_vec3();

        // レイのヒット計算を行う。
        let hit_info = traverse(data, &self.nodes, 0, ray, t_max, inv_dir);

        // ヒット情報がある場合は内部のヒット情報を返す。
        if let Some(hit_info) = hit_info {
            Some(hit_info.intersection)
        } else {
            None
        }
    }
}
