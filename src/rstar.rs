// SPDX-FileCopyrightText: 2026 dcel contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::BTreeSet;
use std::hash::Hash;

use maplike::{Get, Insert, Push, StableRemove};
use rstar::{
    AABB, Envelope, Point, RTree,
    primitives::{GeomWithData, Rectangle},
};
use stable_vec::StableVec;

use crate::{
    Dcel, EdgeId, Face, FaceId, HalfEdge, HalfEdgeId, Vertex, VertexId,
    track::{HalfEdgesCounter, VertexesCounter},
};

pub struct RTreedDcel<
    P: Point,
    VW = P,
    HEW = (),
    FW = (),
    VC = Vec<Vertex<VW>>,
    HEC = Vec<HalfEdge<HEW>>,
    FC = Vec<Face<FW>>,
> {
    dcel: Dcel<VW, HEW, FW, VC, HEC, FC>,
    edges_rtree: RTree<GeomWithData<Rectangle<P>, EdgeId>>,
    faces_rtree: RTree<GeomWithData<Rectangle<P>, FaceId>>,
}

pub type RTreedStableDcel<P, VW = P, HEW = (), FW = ()> = RTreedDcel<
    P,
    VW,
    HEW,
    FW,
    StableVec<Vertex<VW>>,
    StableVec<HalfEdge<HEW>>,
    StableVec<Face<FW>>,
>;

impl<
    P: Point,
    VW,
    HEW,
    FW: Default,
    VC: Default,
    HEC: Default,
    FC: Default + Push<usize, Value = Face<FW>>,
> RTreedDcel<P, VW, HEW, FW, VC, HEC, FC>
{
    #[inline]
    pub fn new() -> Self {
        Self {
            dcel: Dcel::new(),
            edges_rtree: RTree::new(),
            faces_rtree: RTree::new(),
        }
    }

    #[inline]
    pub fn dcel(&self) -> &Dcel<VW, HEW, FW, VC, HEC, FC> {
        &self.dcel
    }

    #[inline]
    pub fn edges_rtree(&self) -> &RTree<GeomWithData<Rectangle<P>, EdgeId>> {
        &self.edges_rtree
    }

    #[inline]
    pub fn faces_rtree(&self) -> &RTree<GeomWithData<Rectangle<P>, FaceId>> {
        &self.faces_rtree
    }
}

impl<
    P: Point,
    VW: Clone + Into<P> + Eq + Hash,
    HEW: Clone + Default,
    FW: Clone + Default,
    VC: Get<usize, Value = Vertex<VW>> + Insert<usize> + Push<usize>,
    HEC: Get<usize, Value = HalfEdge<HEW>> + Insert<usize> + Push<usize>,
    FC: Get<usize, Value = Face<FW>> + Insert<usize> + Push<usize>,
> RTreedDcel<P, VW, HEW, FW, VC, HEC, FC>
{
    pub fn insert_mesh(
        &mut self,
        face_polygons: impl IntoIterator<Item = impl IntoIterator<Item = VW>>,
    ) -> Vec<FaceId> {
        let faces = self.dcel.insert_mesh(face_polygons);
        let mut deduplicating_edge_set = BTreeSet::new();

        for face in &faces {
            self.add_face_to_rtree(*face);

            for edge in self.dcel.face_edges(*face).collect::<Vec<EdgeId>>() {
                if deduplicating_edge_set.insert(edge) {
                    self.add_edge_to_rtree(edge);
                }
            }
        }

        faces
    }

    pub fn insert_mesh_in_face(
        &mut self,
        face_polygons: impl IntoIterator<Item = impl IntoIterator<Item = VW>>,
    ) -> Vec<FaceId> {
        let faces = self.dcel.insert_mesh_in_face(face_polygons);
        let mut deduplicating_edge_set = BTreeSet::new();

        for face in &faces {
            self.add_face_to_rtree(*face);

            for edge in self.dcel.face_edges(*face).collect::<Vec<EdgeId>>() {
                if deduplicating_edge_set.insert(edge) {
                    self.add_edge_to_rtree(edge);
                }
            }
        }

        faces
    }
}

impl<
    P: Point,
    VW: Clone + Into<P>,
    HEW: Clone + Default,
    FW: Clone + Default,
    VC: Get<usize, Value = Vertex<VW>> + Insert<usize> + Push<usize>,
    HEC: Get<usize, Value = HalfEdge<HEW>> + Insert<usize> + Push<usize>,
    FC: Get<usize, Value = Face<FW>> + Insert<usize> + Push<usize>,
> RTreedDcel<P, VW, HEW, FW, VC, HEC, FC>
{
    pub fn insert_polygon(&mut self, vertex_weights: impl IntoIterator<Item = VW>) -> FaceId {
        let vertex_weights: Vec<VW> = vertex_weights.into_iter().collect();
        let face = self.dcel.insert_polygon(vertex_weights.clone());
        self.add_face_with_edges_to_rtrees_with_bbox(
            face,
            Self::rectangle_from_vertex_weights(vertex_weights),
        );

        face
    }

    pub fn insert_polygon_in_face(
        &mut self,
        outer_face: FaceId,
        vertex_weights: impl IntoIterator<Item = VW>,
    ) -> FaceId {
        let vertex_weights: Vec<VW> = vertex_weights.into_iter().collect();

        let face = self
            .dcel
            .insert_polygon_in_face(outer_face, vertex_weights.clone());
        self.add_face_with_edges_to_rtrees_with_bbox(
            face,
            Self::rectangle_from_vertex_weights(vertex_weights),
        );

        face
    }
}

impl<
    P: Point,
    VW: Clone + Into<P>,
    HEW: Clone + Default,
    FW: Clone + Default,
    VC: Get<usize, Value = Vertex<VW>> + Insert<usize> + Push<usize>,
    HEC: Get<usize, Value = HalfEdge<HEW>> + Insert<usize> + Push<usize>,
    FC: Get<usize, Value = Face<FW>> + Insert<usize> + Push<usize>,
> RTreedDcel<P, VW, HEW, FW, VC, HEC, FC>
{
    pub fn insert_edge(&mut self, from: VertexId, to: VertexId) -> FaceId {
        self.split_face_by_edge(from, to, self.dcel.vertexes_common_face(from, to).unwrap())
    }

    pub fn insert_edge_chain(
        &mut self,
        from: VertexId,
        to: VertexId,
        vertex_weights: impl IntoIterator<Item = VW>,
    ) -> FaceId {
        self.split_face_by_edge_chain(
            from,
            to,
            vertex_weights,
            self.dcel.vertexes_common_face(from, to).unwrap(),
        )
    }
}

impl<
    P: Point,
    VW: Clone + Into<P>,
    HEW: Clone,
    FW: Clone + Default,
    VC: Get<usize, Value = Vertex<VW>> + Insert<usize> + Push<usize>,
    HEC: Get<usize, Value = HalfEdge<HEW>> + Insert<usize> + Push<usize>,
    FC: Get<usize, Value = Face<FW>> + Insert<usize> + Push<usize>,
> RTreedDcel<P, VW, HEW, FW, VC, HEC, FC>
{
    fn insert_edge_chain_with_edge_weights(
        &mut self,
        from: VertexId,
        to: VertexId,
        vertex_weights: impl IntoIterator<Item = VW>,
        edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
    ) -> FaceId {
        self.split_face_by_edge_chain_with_all_weights(
            from,
            to,
            vertex_weights,
            edge_weights,
            self.dcel.vertexes_common_face(from, to).unwrap(),
            FW::default(),
        )
    }
}

impl<
    P: Point,
    VW: Clone + Into<P>,
    HEW: Clone,
    FW: Clone,
    VC: Get<usize, Value = Vertex<VW>> + Insert<usize> + Push<usize>,
    HEC: Get<usize, Value = HalfEdge<HEW>> + Insert<usize> + Push<usize>,
    FC: Get<usize, Value = Face<FW>> + Insert<usize> + Push<usize>,
> RTreedDcel<P, VW, HEW, FW, VC, HEC, FC>
{
    pub fn insert_polygon_with_all_weights(
        &mut self,
        vertex_weights: impl IntoIterator<Item = VW>,
        edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
        face_weight: FW,
    ) -> FaceId {
        let vertex_weights: Vec<VW> = vertex_weights.into_iter().collect();

        let face = self.dcel.insert_polygon_with_all_weights(
            vertex_weights.clone(),
            edge_weights,
            face_weight,
        );
        self.add_face_with_edges_to_rtrees_with_bbox(
            face,
            Self::rectangle_from_vertex_weights(vertex_weights),
        );

        face
    }

    pub fn insert_polygon_in_face_with_all_weights(
        &mut self,
        outer_face: FaceId,
        vertex_weights: impl IntoIterator<Item = VW>,
        edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
        face_weight: FW,
    ) -> FaceId {
        let vertex_weights: Vec<VW> = vertex_weights.into_iter().collect();

        let face = self.dcel.insert_polygon_in_face_with_all_weights(
            outer_face,
            vertex_weights.clone(),
            edge_weights,
            face_weight,
        );
        self.add_face_with_edges_to_rtrees_with_bbox(
            face,
            Self::rectangle_from_vertex_weights(vertex_weights),
        );

        face
    }
}

impl<
    P: Point,
    VW: Clone + Into<P>,
    HEW: Clone,
    FW: Clone,
    VC: Get<usize, Value = Vertex<VW>> + Insert<usize> + StableRemove<usize>,
    HEC: Get<usize, Value = HalfEdge<HEW>> + Insert<usize> + StableRemove<usize>,
    FC: Get<usize, Value = Face<FW>> + Insert<usize> + StableRemove<usize>,
> RTreedDcel<P, VW, HEW, FW, VC, HEC, FC>
{
    pub fn remove_edge(&mut self, edge: EdgeId) -> FaceId {
        let endpoints = self.dcel.endpoints(edge);

        self.edges_rtree.remove(&GeomWithData::new(
            Rectangle::from_corners(
                Into::<P>::into(self.dcel.vertex_weight(endpoints.0).clone()),
                Into::<P>::into(self.dcel.vertex_weight(endpoints.1).clone()),
            ),
            edge,
        ));

        // Remove the absorbing and absorbed faces from the faces R-tree before
        // they shape change, which would otherwise invalidate their bboxes and
        // make them impossible to access anymore.
        self.faces_rtree.remove(&GeomWithData::new(
            Self::rectangle_from_vertex_weights(
                self.dcel
                    .face_vertexes(self.dcel.face_in_front(edge.lesser()))
                    .map(|vertex| self.dcel.vertex_weight(vertex).clone()),
            ),
            self.dcel.face_in_front(edge.lesser()),
        ));
        self.faces_rtree.remove(&GeomWithData::new(
            Self::rectangle_from_vertex_weights(
                self.dcel
                    .face_vertexes(self.dcel.face_behind(edge.lesser()))
                    .map(|vertex| self.dcel.vertex_weight(vertex).clone()),
            ),
            self.dcel.face_behind(edge.lesser()),
        ));

        let absorbing_face = self.dcel.remove_edge(edge);

        // Insert the absorbing face back in the R-tree now that its bbox is
        // done changing.
        self.add_face_to_rtree(absorbing_face);

        absorbing_face
    }
}

impl<
    P: Point,
    VW: Clone + Into<P>,
    HEW: Clone,
    FW: Clone,
    VC: Get<usize, Value = Vertex<VW>> + Insert<usize> + StableRemove<usize>,
    HEC: Get<usize, Value = HalfEdge<HEW>> + Insert<usize> + StableRemove<usize>,
    FC: Get<usize, Value = Face<FW>> + Insert<usize> + StableRemove<usize>,
> RTreedDcel<P, VW, HEW, FW, VC, HEC, FC>
{
    pub fn merge_faces_around_vertex(&mut self, inner_vertex: VertexId) {
        let absorbing_face = self
            .dcel
            .face_in_front(self.dcel.outgoing_next_half_edge(inner_vertex));
        self.absorb_faces_around_vertex(absorbing_face, inner_vertex);
    }

    pub fn absorb_faces_around_vertex(&mut self, absorbing_face: FaceId, inner_vertex: VertexId) {
        let initial_half_edge = self.dcel.outgoing_next_half_edge(inner_vertex);

        let inner_edges: Vec<EdgeId> = self.dcel.spokes(initial_half_edge).collect();
        let perimeter_half_edges: Vec<HalfEdgeId> = self
            .dcel
            .vertex_rim_half_edges(inner_vertex)
            .collect::<Vec<HalfEdgeId>>();

        self.absorb_faces_over_edges_and_vertexes_in_perimeter(
            absorbing_face,
            self.dcel
                .interspokes(initial_half_edge)
                .filter(|face| face.id() != absorbing_face.id())
                .collect::<Vec<FaceId>>(),
            inner_edges,
            [inner_vertex],
            &perimeter_half_edges,
        );
    }

    pub fn merge_faces_over_edges_and_vertexes(
        &mut self,
        faces: impl IntoIterator<Item = FaceId>,
        edges: impl IntoIterator<Item = EdgeId>,
        vertexes: impl IntoIterator<Item = VertexId>,
    ) {
        let mut faces = faces.into_iter();
        let absorbing_face = faces.next().unwrap();

        self.absorb_faces_over_edges_and_vertexes(
            absorbing_face,
            faces.filter(|face| face.id() != absorbing_face.id()),
            edges,
            vertexes,
        );
    }

    pub fn absorb_faces_over_edges_and_vertexes(
        &mut self,
        absorbing_face: FaceId,
        faces: impl IntoIterator<Item = FaceId>,
        edges: impl IntoIterator<Item = EdgeId>,
        vertexes: impl IntoIterator<Item = VertexId>,
    ) {
        let edges: Vec<EdgeId> = edges.into_iter().collect();
        let excluded_half_edges: Vec<HalfEdgeId> = edges
            .iter()
            .flat_map(|edge| [edge.lesser(), edge.greater()])
            .collect();

        // Find an initial half-edge that is not in the excluded list. Otherwise,
        // the circulator could end up starting from an excluded half-edge, which would
        // result in an infinite loop, as the termination condition depends on
        // returning to the initial edge.
        let initial_half_edge = self
            .dcel
            .face_half_edges(absorbing_face)
            .find(|half_edge| !excluded_half_edges.contains(half_edge))
            .unwrap();

        let perimeter_half_edges: Vec<HalfEdgeId> = self
            .dcel
            .circulate_half_edges_with_excludes(initial_half_edge, excluded_half_edges.clone())
            .collect();

        self.absorb_faces_over_edges_and_vertexes_in_perimeter(
            absorbing_face,
            faces,
            edges,
            vertexes,
            &perimeter_half_edges,
        );
    }

    pub fn merge_faces(&mut self, faces: impl IntoIterator<Item = FaceId>) {
        let mut faces = faces.into_iter();
        let absorbing_face = faces.next().unwrap();

        self.absorb_faces(
            absorbing_face,
            faces.filter(|face| face.id() != absorbing_face.id()),
        );
    }

    pub fn absorb_faces(
        &mut self,
        absorbing_face: FaceId,
        faces: impl IntoIterator<Item = FaceId>,
    ) {
        let mut half_edges_counter = HalfEdgesCounter::new();
        let mut vertex_weights_counter = VertexesCounter::new();
        let faces: Vec<FaceId> = faces.into_iter().collect();

        // To detect the merged edges and vertexes correctly, the absorbing face
        // has to be visited in addition to the absorbed faces.
        half_edges_counter.visit_face_half_edges(&self.dcel, absorbing_face);
        vertex_weights_counter.visit_face_vertexes(&self.dcel, absorbing_face);

        for &face in &faces {
            half_edges_counter.visit_face_half_edges(&self.dcel, face);
            vertex_weights_counter.visit_face_vertexes(&self.dcel, face);
        }

        self.absorb_faces_over_edges_and_vertexes_in_perimeter(
            absorbing_face,
            faces,
            half_edges_counter
                .inner_edges(&self.dcel)
                .collect::<Vec<EdgeId>>(),
            vertex_weights_counter
                .visited_vertexes()
                .filter(|&vertex| {
                    self.dcel
                        .spokes_reverse(self.dcel.outgoing_next_half_edge(vertex))
                        .all(|edge| half_edges_counter.is_inner_edge(edge))
                })
                // PERF: Needless collect?
                .collect::<Vec<VertexId>>(),
            &half_edges_counter
                .outer_edges(&self.dcel)
                .map(|(half_edge, _)| half_edge)
                .collect::<Vec<HalfEdgeId>>(),
        );
    }

    fn absorb_faces_over_edges_and_vertexes_in_perimeter(
        &mut self,
        absorbing_face: FaceId,
        faces_to_absorb: impl IntoIterator<Item = FaceId>,
        edges_to_remove: impl IntoIterator<Item = EdgeId>,
        vertexes_to_remove: impl IntoIterator<Item = VertexId>,
        perimeter_half_edges: &[HalfEdgeId],
    ) {
        let faces_to_absorb: Vec<FaceId> = faces_to_absorb.into_iter().collect();
        let edges_to_remove: Vec<EdgeId> = edges_to_remove.into_iter().collect();

        for &face_to_absorb in &faces_to_absorb {
            self.faces_rtree.remove(&GeomWithData::new(
                Self::rectangle_from_vertex_weights(
                    self.dcel
                        .face_vertexes(face_to_absorb)
                        .map(|vertex| self.dcel.vertex_weight(vertex).clone()),
                ),
                face_to_absorb,
            ));
        }

        for &edge_to_remove in &edges_to_remove {
            let endpoints = self.dcel.endpoints(edge_to_remove);

            self.edges_rtree.remove(&GeomWithData::new(
                Rectangle::from_corners(
                    Into::<P>::into(self.dcel.vertex_weight(endpoints.0).clone()),
                    Into::<P>::into(self.dcel.vertex_weight(endpoints.1).clone()),
                ),
                edge_to_remove,
            ));
        }

        // Remove the absorbing face from the faces R-tree before its shape
        // changes, which would otherwise invalidate its bbox and make it
        // impossible to access anymore.
        self.faces_rtree.remove(&GeomWithData::new(
            Self::rectangle_from_vertex_weights(
                self.dcel
                    .face_vertexes(absorbing_face)
                    .map(|vertex| self.dcel.vertex_weight(vertex).clone()),
            ),
            absorbing_face,
        ));

        self.dcel.absorb_faces_over_edges_and_vertexes_in_perimeter(
            absorbing_face,
            faces_to_absorb,
            edges_to_remove,
            vertexes_to_remove,
            perimeter_half_edges,
        );

        // Insert the absorbing face back in the R-tree now that its bbox is
        // done changing.
        self.add_face_to_rtree(absorbing_face);
    }
}

impl<
    P: Point,
    VW: Clone + Into<P>,
    HEW: Clone + Default,
    FW: Clone + Default,
    VC: Get<usize, Value = Vertex<VW>> + Insert<usize> + Push<usize>,
    HEC: Get<usize, Value = HalfEdge<HEW>> + Insert<usize> + Push<usize>,
    FC: Get<usize, Value = Face<FW>> + Insert<usize> + Push<usize>,
> RTreedDcel<P, VW, HEW, FW, VC, HEC, FC>
{
    pub fn split_edge_by_vertex(&mut self, edge_to_split: EdgeId, vertex: VW) -> EdgeId {
        let original_endpoints = self.dcel.endpoints(edge_to_split);

        // Remove the edge to split from the edges R-tree before its shape
        // changes, which would otherwise invalidate its bbox and make it
        // impossible to access anymore.
        self.edges_rtree.remove(&GeomWithData::new(
            Rectangle::from_corners(
                Into::<P>::into(self.dcel.vertex_weight(original_endpoints.0).clone()),
                Into::<P>::into(self.dcel.vertex_weight(original_endpoints.1).clone()),
            ),
            edge_to_split,
        ));

        let new_edge = self.dcel.split_edge_by_vertex(edge_to_split, vertex);
        self.add_edges_to_rtree([new_edge]);

        // Insert the split edge back in the edges R-tree now that its bbox is
        // done changing.
        self.add_edges_to_rtree([edge_to_split]);

        new_edge
    }

    pub fn split_face_by_edge(
        &mut self,
        from: VertexId,
        to: VertexId,
        face_to_split: FaceId,
    ) -> FaceId {
        let (new_faces, _) = self.update_face_rtree(face_to_split, |dcel| {
            let new_face = dcel.split_face_by_edge(from, to, face_to_split);
            (vec![new_face], Vec::new())
        });

        new_faces[0]
    }

    pub fn split_face_by_edge_chain(
        &mut self,
        from: VertexId,
        to: VertexId,
        vertex_weights: impl IntoIterator<Item = VW>,
        face_to_split: FaceId,
    ) -> FaceId {
        let (new_faces, _) = self.update_face_rtree(face_to_split, |dcel| {
            let new_face = dcel.split_face_by_edge_chain(from, to, vertex_weights, face_to_split);
            (vec![new_face], Vec::new())
        });

        new_faces[0]
    }
}

impl<
    P: Point,
    VW: Clone + Into<P>,
    HEW: Clone,
    FW: Clone,
    VC: Get<usize, Value = Vertex<VW>> + Insert<usize> + Push<usize>,
    HEC: Get<usize, Value = HalfEdge<HEW>> + Insert<usize> + Push<usize>,
    FC: Get<usize, Value = Face<FW>> + Insert<usize> + Push<usize>,
> RTreedDcel<P, VW, HEW, FW, VC, HEC, FC>
{
    pub fn split_face_by_edge_chain_with_all_weights(
        &mut self,
        from: VertexId,
        to: VertexId,
        vertex_weights: impl IntoIterator<Item = VW>,
        edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
        face_to_split: FaceId,
        new_face_weight: FW,
    ) -> FaceId {
        let (new_faces, _) = self.update_face_rtree(face_to_split, |dcel| {
            let new_face = dcel.split_face_by_edge_chain_with_all_weights(
                from,
                to,
                vertex_weights,
                edge_weights,
                face_to_split,
                new_face_weight,
            );
            (vec![new_face], Vec::new())
        });

        new_faces[0]
    }
}

impl<
    P: Point,
    VW: Clone + Into<P>,
    HEW: Clone + Default,
    FW: Clone + Default,
    VC: Get<usize, Value = Vertex<VW>> + Insert<usize> + Push<usize>,
    HEC: Get<usize, Value = HalfEdge<HEW>> + Insert<usize> + Push<usize>,
    FC: Get<usize, Value = Face<FW>> + Insert<usize> + Push<usize>,
> RTreedDcel<P, VW, HEW, FW, VC, HEC, FC>
{
    /// Partition a face into triangles by inserting a vertex inside and then
    /// adding edges between it and the original face's vertexes.
    ///
    /// The original face is reused for the first triangle. New faces are
    /// created for all the other triangles.
    ///
    /// Returns the new vertex id together with the face ids of all the new
    /// triangles.
    pub fn triangulate_around_vertex(
        &mut self,
        perimeter_face: FaceId,
        inner_vertex_weight: VW,
    ) -> (Vec<FaceId>, Vec<EdgeId>) {
        self.update_face_rtree(perimeter_face, |dcel| {
            dcel.triangulate_around_vertex(perimeter_face, inner_vertex_weight)
        })
    }

    pub fn fan_triangulate(
        &mut self,
        perimeter_face: FaceId,
        apex: VertexId,
    ) -> (Vec<FaceId>, Vec<EdgeId>) {
        self.update_face_rtree(perimeter_face, |dcel| {
            dcel.fan_triangulate(perimeter_face, apex)
        })
    }
}

impl<
    P: Point,
    VW: Clone + Into<P>,
    HEW: Clone,
    FW: Clone,
    VC: Get<usize, Value = Vertex<VW>> + Insert<usize> + Push<usize>,
    HEC: Get<usize, Value = HalfEdge<HEW>> + Insert<usize> + Push<usize>,
    FC: Get<usize, Value = Face<FW>> + Insert<usize> + Push<usize>,
> RTreedDcel<P, VW, HEW, FW, VC, HEC, FC>
{
    pub fn triangulate_around_vertex_with_all_weights(
        &mut self,
        perimeter_face: FaceId,
        inner_vertex_weight: VW,
        inner_edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
        triangle_face_weights: impl IntoIterator<Item = FW>,
    ) -> (Vec<FaceId>, Vec<EdgeId>) {
        self.update_face_rtree(perimeter_face, |dcel| {
            dcel.triangulate_around_vertex_with_all_weights(
                perimeter_face,
                inner_vertex_weight,
                inner_edge_weights,
                triangle_face_weights,
            )
        })
    }

    pub fn fan_triangulate_with_all_weights(
        &mut self,
        perimeter_face: FaceId,
        apex: VertexId,
        inner_edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
        triangle_face_weights: impl IntoIterator<Item = FW>,
    ) -> (Vec<FaceId>, Vec<EdgeId>) {
        self.update_face_rtree(perimeter_face, |dcel| {
            dcel.fan_triangulate_with_all_weights(
                perimeter_face,
                apex,
                inner_edge_weights,
                triangle_face_weights,
            )
        })
    }
}

impl<
    P: Point,
    VW: Clone + Into<P>,
    HEW,
    FW,
    VC: Get<usize, Value = Vertex<VW>>,
    HEC: Get<usize, Value = HalfEdge<HEW>>,
    FC: Get<usize, Value = Face<FW>>,
> RTreedDcel<P, VW, HEW, FW, VC, HEC, FC>
{
    fn update_face_rtree<F>(
        &mut self,
        face_to_update: FaceId,
        mutate_fn: F,
    ) -> (Vec<FaceId>, Vec<EdgeId>)
    where
        F: FnOnce(&mut Dcel<VW, HEW, FW, VC, HEC, FC>) -> (Vec<FaceId>, Vec<EdgeId>),
    {
        // The updated face id is reused as one of the new faces. So, we need
        // to remove it from the faces R-tree before its shape changes, as it
        // would otherwise invalidate its bbox and make it impossible to access
        // anymore.
        self.faces_rtree.remove(&GeomWithData::new(
            Self::rectangle_from_vertex_weights(
                self.dcel
                    .face_vertexes(face_to_update)
                    .map(|vertex| self.dcel.vertex_weight(vertex).clone()),
            ),
            face_to_update,
        ));

        let (new_faces, new_edges) = mutate_fn(&mut self.dcel);

        // Insert the updated face back in the faces R-tree now that its bbox
        // is done changing.
        self.add_face_to_rtree(face_to_update);

        self.add_faces_to_rtree(new_faces.clone());
        self.add_edges_to_rtree(new_edges.clone());

        (new_faces, new_edges)
    }

    fn add_faces_to_rtree(&mut self, faces: impl IntoIterator<Item = FaceId>) {
        for face in faces {
            self.add_face_to_rtree(face);
        }
    }

    fn add_face_to_rtree(&mut self, face: FaceId) {
        self.add_face_to_rtree_with_bbox(
            face,
            Self::rectangle_from_vertex_weights(
                self.dcel
                    .face_vertexes(face)
                    .map(|vertex| self.dcel.vertex_weight(vertex).clone()),
            ),
        )
    }

    fn add_face_to_rtree_with_bbox(&mut self, face: FaceId, face_rectangle: Rectangle<P>) {
        self.faces_rtree
            .insert(GeomWithData::new(face_rectangle, face));
    }

    fn add_face_with_edges_to_rtrees(&mut self, face: FaceId) {
        self.add_face_with_edges_to_rtrees_with_bbox(
            face,
            Self::rectangle_from_vertex_weights(
                self.dcel
                    .face_vertexes(face)
                    .map(|vertex| self.dcel.vertex_weight(vertex).clone()),
            ),
        )
    }

    fn add_face_with_edges_to_rtrees_with_bbox(
        &mut self,
        face: FaceId,
        face_rectangle: Rectangle<P>,
    ) {
        self.add_face_to_rtree_with_bbox(face, face_rectangle);
        self.add_edges_to_rtree(self.dcel.face_edges(face).collect::<Vec<EdgeId>>());
    }

    fn add_edges_to_rtree(&mut self, edges: impl IntoIterator<Item = EdgeId>) {
        for edge in edges {
            self.add_edge_to_rtree(edge);
        }
    }

    fn add_edge_to_rtree(&mut self, edge: EdgeId) {
        let endpoints = self.dcel.endpoints(edge);
        self.edges_rtree.insert(GeomWithData::new(
            Rectangle::from_corners(
                Into::<P>::into(self.dcel.vertex_weight(endpoints.0).clone()),
                Into::<P>::into(self.dcel.vertex_weight(endpoints.1).clone()),
            ),
            edge,
        ));
    }
}

impl<P: Point, VW: Into<P>, HEW, FW, VC, HEC, FC> RTreedDcel<P, VW, HEW, FW, VC, HEC, FC> {
    fn rectangle_from_vertex_weights(weights: impl IntoIterator<Item = VW>) -> Rectangle<P> {
        Rectangle::from_aabb(weights.into_iter().fold(AABB::new_empty(), |aabb, weight| {
            aabb.merged(&AABB::from_point(Into::<P>::into(weight)))
        }))
    }
}

#[cfg(all(test, feature = "rstar", feature = "stable-vec"))]
mod test {
    use rstar::{
        RTreeObject,
        primitives::{GeomWithData, Rectangle},
    };

    use crate::{
        EdgeId, FaceId, HalfEdgeId, RTreedStableDcel, VertexId, assert_face_boundary,
        init_dcel_with_3x3_hex_mesh,
    };

    #[test]
    fn test_insert_polygon() {
        let mut rtreed_dcel = RTreedStableDcel::<(i32, i32)>::new();
        let face = rtreed_dcel.insert_polygon([(0, 0), (10, 0), (10, 10), (0, 10)]);

        assert_eq!(rtreed_dcel.dcel.faces().num_elements(), 2);
        assert_eq!(rtreed_dcel.faces_rtree.size(), 1);
        assert_eq!(rtreed_dcel.edges_rtree.size(), 4);

        assert_face_boundary!(rtreed_dcel.dcel, 0, 0);
        assert_face_boundary!(rtreed_dcel.dcel, face.id(), 4);

        assert_face_bbox_validity(&rtreed_dcel, face.id());
    }

    #[test]
    fn test_insert_edge() {
        let mut rtreed_dcel = init_dcel_with_3x3_hex_mesh!(RTreedStableDcel<(i32, i32)>);
        let face_to_split = FaceId::new(5);
        let face_to_split_vertexes: Vec<VertexId> =
            rtreed_dcel.dcel.face_vertexes(face_to_split).collect();

        // Split face 5 in two with a single edge.
        rtreed_dcel.insert_edge(face_to_split_vertexes[0], face_to_split_vertexes[3]);

        // There are now eleven faces in total: one unbounded and ten bounded.
        assert_eq!(rtreed_dcel.dcel.faces().num_elements(), 11);
        assert_eq!(rtreed_dcel.faces_rtree.size(), 10);

        // The original hexagon is now split into two quads.
        assert_face_boundary!(rtreed_dcel.dcel, 0, 0);
        assert_face_boundary!(rtreed_dcel.dcel, 1, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 2, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 3, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 4, 6);
        // First quad face.
        assert_face_boundary!(rtreed_dcel.dcel, 5, 4);
        assert_face_boundary!(rtreed_dcel.dcel, 6, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 7, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 8, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 9, 6);
        // Second quad face.
        assert_face_boundary!(&rtreed_dcel.dcel, 10, 4);

        assert_face_bbox_validity(&rtreed_dcel, 1);
        assert_face_bbox_validity(&rtreed_dcel, 2);
        assert_face_bbox_validity(&rtreed_dcel, 3);
        assert_face_bbox_validity(&rtreed_dcel, 4);
        assert_face_bbox_validity(&rtreed_dcel, 5);
        assert_face_bbox_validity(&rtreed_dcel, 6);
        assert_face_bbox_validity(&rtreed_dcel, 7);
        assert_face_bbox_validity(&rtreed_dcel, 8);
        assert_face_bbox_validity(&rtreed_dcel, 9);
        assert_face_bbox_validity(&rtreed_dcel, 10);
    }

    // TODO: Test remove edge.

    #[test]
    fn test_merge_faces_around_vertex() {
        let mut rtreed_dcel = init_dcel_with_3x3_hex_mesh!(RTreedStableDcel<(i32, i32)>);
        rtreed_dcel.merge_faces_around_vertex(VertexId::new(8));

        // There are now eight faces in total: one unbounded and seven bounded.
        assert_eq!(rtreed_dcel.dcel.faces().num_elements(), 8);
        assert_eq!(rtreed_dcel.faces_rtree.size(), 7);

        // Among the remaining faces, one is now a dodecagon, and the remaining
        // six are hexagons.
        assert_face_boundary!(rtreed_dcel.dcel, 0, 0);
        assert_face_boundary!(rtreed_dcel.dcel, 1, 6);
        // Face 2 does not exist.
        assert_face_boundary!(rtreed_dcel.dcel, 3, 6);
        // Face 4 does not exist.
        assert_face_boundary!(rtreed_dcel.dcel, 5, 12);
        assert_face_boundary!(rtreed_dcel.dcel, 6, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 7, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 8, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 9, 6);

        assert_face_bbox_validity(&rtreed_dcel, 1);
        // Face 2 does not exist.
        assert_face_bbox_validity(&rtreed_dcel, 3);
        // Face 4 does not exist.
        assert_face_bbox_validity(&rtreed_dcel, 6);
        assert_face_bbox_validity(&rtreed_dcel, 7);
        assert_face_bbox_validity(&rtreed_dcel, 8);
        assert_face_bbox_validity(&rtreed_dcel, 9);
    }

    #[test]
    fn test_absorb_faces_around_vertex() {
        let mut rtreed_dcel = init_dcel_with_3x3_hex_mesh!(RTreedStableDcel<(i32, i32)>);
        rtreed_dcel.absorb_faces_around_vertex(FaceId::new(2), VertexId::new(8));

        // There are now eight faces in total: one unbounded and seven bounded.
        assert_eq!(rtreed_dcel.dcel.faces().num_elements(), 8);
        assert_eq!(rtreed_dcel.faces_rtree.size(), 7);

        // Among the remaining faces, one is now a dodecagon, and the remaining
        // six are hexagons.
        assert_face_boundary!(rtreed_dcel.dcel, 0, 0);
        assert_face_boundary!(rtreed_dcel.dcel, 1, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 2, 12);
        assert_face_boundary!(rtreed_dcel.dcel, 3, 6);
        // Face 4 does not exist.
        // Face 5 does not exist.
        assert_face_boundary!(rtreed_dcel.dcel, 6, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 7, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 8, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 9, 6);

        assert_face_bbox_validity(&rtreed_dcel, 1);
        assert_face_bbox_validity(&rtreed_dcel, 2);
        assert_face_bbox_validity(&rtreed_dcel, 3);
        // Face 4 does not exist.
        // Face 5 does not exist.
        assert_face_bbox_validity(&rtreed_dcel, 6);
        assert_face_bbox_validity(&rtreed_dcel, 7);
        assert_face_bbox_validity(&rtreed_dcel, 8);
        assert_face_bbox_validity(&rtreed_dcel, 9);
    }

    #[test]
    fn test_merge_faces_over_edge() {
        let mut rtreed_dcel = init_dcel_with_3x3_hex_mesh!(RTreedStableDcel<(i32, i32)>);
        rtreed_dcel.merge_faces_over_edges_and_vertexes(
            [FaceId::new(5), FaceId::new(6)],
            [EdgeId::new(HalfEdgeId::new(44), HalfEdgeId::new(45))],
            [],
        );

        assert_eq!(rtreed_dcel.dcel.faces().num_elements(), 9);
        assert_eq!(rtreed_dcel.faces_rtree.size(), 8);

        assert_face_boundary!(rtreed_dcel.dcel, 0, 0);
        assert_face_boundary!(rtreed_dcel.dcel, 1, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 2, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 3, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 4, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 5, 10);
        // Face 6 does not exist.
        assert_face_boundary!(rtreed_dcel.dcel, 7, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 8, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 9, 6);

        assert_face_bbox_validity(&rtreed_dcel, 1);
        assert_face_bbox_validity(&rtreed_dcel, 2);
        assert_face_bbox_validity(&rtreed_dcel, 3);
        assert_face_bbox_validity(&rtreed_dcel, 4);
        assert_face_bbox_validity(&rtreed_dcel, 5);
        // Face 6 does not exist.
        assert_face_bbox_validity(&rtreed_dcel, 7);
        assert_face_bbox_validity(&rtreed_dcel, 8);
        assert_face_bbox_validity(&rtreed_dcel, 9);
    }

    #[test]
    fn test_absorb_face_into_face_over_edge() {
        let mut rtreed_dcel = init_dcel_with_3x3_hex_mesh!(RTreedStableDcel<(i32, i32)>);
        rtreed_dcel.absorb_faces_over_edges_and_vertexes(
            FaceId::new(6),
            [FaceId::new(5)],
            [EdgeId::new(HalfEdgeId::new(44), HalfEdgeId::new(45))],
            [],
        );

        assert_eq!(rtreed_dcel.dcel.faces().num_elements(), 9);
        assert_eq!(rtreed_dcel.faces_rtree.size(), 8);

        assert_face_boundary!(rtreed_dcel.dcel, 0, 0);
        assert_face_boundary!(rtreed_dcel.dcel, 1, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 2, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 3, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 4, 6);
        // Face 5 does not exist.
        assert_face_boundary!(rtreed_dcel.dcel, 6, 10);
        assert_face_boundary!(rtreed_dcel.dcel, 7, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 8, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 9, 6);

        assert_face_bbox_validity(&rtreed_dcel, 1);
        assert_face_bbox_validity(&rtreed_dcel, 2);
        assert_face_bbox_validity(&rtreed_dcel, 3);
        assert_face_bbox_validity(&rtreed_dcel, 4);
        // Face 5 does not exist.
        assert_face_bbox_validity(&rtreed_dcel, 6);
        assert_face_bbox_validity(&rtreed_dcel, 7);
        assert_face_bbox_validity(&rtreed_dcel, 8);
        assert_face_bbox_validity(&rtreed_dcel, 9);
    }

    #[test]
    fn merge_two_faces() {
        let mut rtreed_dcel = init_dcel_with_3x3_hex_mesh!(RTreedStableDcel<(i32, i32)>);
        rtreed_dcel.merge_faces([FaceId::new(7), FaceId::new(8)]);

        assert_eq!(rtreed_dcel.dcel.faces().num_elements(), 9);
        assert_eq!(rtreed_dcel.faces_rtree.size(), 8);

        assert_face_boundary!(rtreed_dcel.dcel, 0, 0);
        assert_face_boundary!(rtreed_dcel.dcel, 1, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 2, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 3, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 4, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 5, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 6, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 7, 10);
        // Face 8 does not exist.
        assert_face_boundary!(rtreed_dcel.dcel, 9, 6);

        assert_face_bbox_validity(&rtreed_dcel, 1);
        assert_face_bbox_validity(&rtreed_dcel, 2);
        assert_face_bbox_validity(&rtreed_dcel, 3);
        assert_face_bbox_validity(&rtreed_dcel, 4);
        assert_face_bbox_validity(&rtreed_dcel, 5);
        assert_face_bbox_validity(&rtreed_dcel, 6);
        assert_face_bbox_validity(&rtreed_dcel, 7);
        // Face 8 does not exist.
        assert_face_bbox_validity(&rtreed_dcel, 9);
    }

    #[test]
    fn absorb_face_into_face() {
        let mut rtreed_dcel = init_dcel_with_3x3_hex_mesh!(RTreedStableDcel<(i32, i32)>);
        rtreed_dcel.absorb_faces(FaceId::new(8), [FaceId::new(7)]);

        assert_eq!(rtreed_dcel.dcel.faces().num_elements(), 9);
        assert_eq!(rtreed_dcel.faces_rtree.size(), 8);

        assert_face_boundary!(rtreed_dcel.dcel, 0, 0);
        assert_face_boundary!(rtreed_dcel.dcel, 1, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 2, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 3, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 4, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 5, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 6, 6);
        // Face 7 does not exist.
        assert_face_boundary!(rtreed_dcel.dcel, 8, 10);
        assert_face_boundary!(rtreed_dcel.dcel, 9, 6);

        assert_face_bbox_validity(&rtreed_dcel, 1);
        assert_face_bbox_validity(&rtreed_dcel, 2);
        assert_face_bbox_validity(&rtreed_dcel, 3);
        assert_face_bbox_validity(&rtreed_dcel, 4);
        assert_face_bbox_validity(&rtreed_dcel, 5);
        assert_face_bbox_validity(&rtreed_dcel, 6);
        // Face 7 does not exist.
        assert_face_bbox_validity(&rtreed_dcel, 8);
        assert_face_bbox_validity(&rtreed_dcel, 9);
    }

    #[test]
    fn test_split_face_by_edge() {
        let mut rtreed_dcel = init_dcel_with_3x3_hex_mesh!(RTreedStableDcel<(i32, i32)>);
        let face_to_split = FaceId::new(5);
        let face_to_split_vertexes: Vec<VertexId> =
            rtreed_dcel.dcel.face_vertexes(face_to_split).collect();

        // Split face 5 in two with a single edge.
        rtreed_dcel.split_face_by_edge(
            face_to_split_vertexes[0],
            face_to_split_vertexes[3],
            face_to_split,
        );

        // There are now eleven faces in total: one unbounded and ten bounded.
        assert_eq!(rtreed_dcel.dcel.faces().num_elements(), 11);
        assert_eq!(rtreed_dcel.faces_rtree.size(), 10);

        // The original hexagon is now split into two quads.
        assert_face_boundary!(rtreed_dcel.dcel, 0, 0);
        assert_face_boundary!(rtreed_dcel.dcel, 1, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 2, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 3, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 4, 6);
        // Second quad face.
        assert_face_boundary!(rtreed_dcel.dcel, 5, 4);
        assert_face_boundary!(rtreed_dcel.dcel, 6, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 7, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 8, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 9, 6);
        // Second quad face.
        assert_face_boundary!(&rtreed_dcel.dcel, 10, 4);

        assert_face_bbox_validity(&rtreed_dcel, 1);
        assert_face_bbox_validity(&rtreed_dcel, 2);
        assert_face_bbox_validity(&rtreed_dcel, 3);
        assert_face_bbox_validity(&rtreed_dcel, 4);
        assert_face_bbox_validity(&rtreed_dcel, 5);
        assert_face_bbox_validity(&rtreed_dcel, 6);
        assert_face_bbox_validity(&rtreed_dcel, 7);
        assert_face_bbox_validity(&rtreed_dcel, 8);
        assert_face_bbox_validity(&rtreed_dcel, 9);
        assert_face_bbox_validity(&rtreed_dcel, 10);
    }

    #[test]
    fn test_split_edge_by_vertex() {
        let mut rtreed_dcel = init_dcel_with_3x3_hex_mesh!(RTreedStableDcel<(i32, i32)>);
        let edge = EdgeId::new(HalfEdgeId::new(38), HalfEdgeId::new(39));
        let original_endpoints = rtreed_dcel.dcel.endpoints(edge);
        let original_edge_count = rtreed_dcel.edges_rtree.size();

        let new_edge = rtreed_dcel.split_edge_by_vertex(edge, (259, 150));

        assert_eq!(rtreed_dcel.dcel.vertexes().num_elements(), 31);
        assert_eq!(rtreed_dcel.dcel.half_edges().num_elements(), 78);
        assert_eq!(rtreed_dcel.dcel.faces().num_elements(), 10);
        assert_eq!(rtreed_dcel.edges_rtree.size(), 39);
        assert_eq!(rtreed_dcel.faces_rtree.size(), 9);

        assert_face_boundary!(rtreed_dcel.dcel, 0, 0);
        assert_face_boundary!(rtreed_dcel.dcel, 1, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 2, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 3, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 4, 7);
        assert_face_boundary!(rtreed_dcel.dcel, 5, 7);
        assert_face_boundary!(rtreed_dcel.dcel, 6, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 7, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 8, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 9, 6);

        assert_edge_bbox_validity(&rtreed_dcel, edge);
        assert_edge_bbox_validity(&rtreed_dcel, new_edge);
    }

    #[test]
    fn test_split_face_by_chain_of_two_edges() {
        let mut rtreed_dcel = init_dcel_with_3x3_hex_mesh!(RTreedStableDcel<(i32, i32)>);
        let face_to_split = FaceId::new(5);
        let face_to_split_vertexes: Vec<VertexId> =
            rtreed_dcel.dcel.face_vertexes(face_to_split).collect();

        // Split face 5 in two with a chain of two edges, with their common
        // point around the face's center.
        rtreed_dcel.split_face_by_edge_chain(
            face_to_split_vertexes[0],
            face_to_split_vertexes[3],
            [(259, 150)],
            face_to_split,
        );

        // There are now eleven faces in total: one unbounded and ten bounded.
        assert_eq!(rtreed_dcel.dcel.faces().num_elements(), 11);
        assert_eq!(rtreed_dcel.faces_rtree.size(), 10);
        // TODO: Test number of elements here and further below.

        // The original hexagon is now split into two pentagons.

        assert_face_boundary!(rtreed_dcel.dcel, 0, 0);
        assert_face_boundary!(rtreed_dcel.dcel, 1, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 2, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 3, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 4, 6);
        // First pentagon face.
        assert_face_boundary!(rtreed_dcel.dcel, 5, 5);
        assert_face_boundary!(rtreed_dcel.dcel, 6, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 7, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 8, 6);
        assert_face_boundary!(rtreed_dcel.dcel, 9, 6);
        // Second pentagon face.
        assert_face_boundary!(rtreed_dcel.dcel, 10, 5);

        assert_face_bbox_validity(&rtreed_dcel, 1);
        assert_face_bbox_validity(&rtreed_dcel, 2);
        assert_face_bbox_validity(&rtreed_dcel, 3);
        assert_face_bbox_validity(&rtreed_dcel, 4);
        assert_face_bbox_validity(&rtreed_dcel, 5);
        assert_face_bbox_validity(&rtreed_dcel, 6);
        assert_face_bbox_validity(&rtreed_dcel, 7);
        assert_face_bbox_validity(&rtreed_dcel, 8);
        assert_face_bbox_validity(&rtreed_dcel, 9);
        assert_face_bbox_validity(&rtreed_dcel, 10);
    }

    #[test]
    fn test_triangulate_around_vertex() {
        let mut rtreed_dcel = init_dcel_with_3x3_hex_mesh!(RTreedStableDcel<(i32, i32)>);
        let (new_faces, new_edges) =
            rtreed_dcel.triangulate_around_vertex(FaceId::new(5), (260, 125));

        assert_eq!(new_faces.len(), 5);
        assert_eq!(new_edges.len(), 6);

        assert_eq!(rtreed_dcel.dcel.faces().num_elements(), 15);
        assert_eq!(rtreed_dcel.faces_rtree.size(), 14);

        assert_face_boundary!(rtreed_dcel.dcel, 5, 3);

        // All new faces are triangles.
        for face in &new_faces {
            let face_vertexes: Vec<VertexId> = rtreed_dcel.dcel.face_vertexes(*face).collect();
            assert_eq!(face_vertexes.len(), 3);
        }

        // The new edges are in the edges rtree.
        assert_eq!(
            rtreed_dcel
                .edges_rtree
                .iter()
                .filter(|e| new_edges.contains(&e.data))
                .count(),
            6
        );

        assert_face_bbox_validity(&rtreed_dcel, 5);
        for face in &new_faces {
            assert!(
                rtreed_dcel
                    .faces_rtree
                    .locate_in_envelope(
                        &RTreedStableDcel::<(i32, i32)>::rectangle_from_vertex_weights(
                            rtreed_dcel
                                .dcel
                                .face_vertexes(*face)
                                .map(|vertex| rtreed_dcel.dcel.vertex_weight(vertex).clone()),
                        )
                        .envelope(),
                    )
                    .any(|element| element.data == *face)
            );
        }
    }

    // TODO: Test fan triangulation.

    fn assert_face_bbox_validity(rtreed_dcel: &RTreedStableDcel<(i32, i32)>, face: usize) {
        let rectangle = face_rectangle(rtreed_dcel, FaceId::new(face));
        assert!(
            rtreed_dcel
                .faces_rtree
                .locate_in_envelope(&rectangle.envelope())
                .any(|&element| element == GeomWithData::new(rectangle, FaceId::new(face),))
        );
    }

    fn assert_edge_bbox_validity(rtreed_dcel: &RTreedStableDcel<(i32, i32)>, edge: EdgeId) {
        let endpoints = rtreed_dcel.dcel.endpoints(edge);
        let rectangle = Rectangle::from_corners(
            rtreed_dcel.dcel.vertex_weight(endpoints.0).clone(),
            rtreed_dcel.dcel.vertex_weight(endpoints.1).clone(),
        );

        assert!(
            rtreed_dcel
                .edges_rtree
                .locate_in_envelope(&rectangle.envelope())
                .any(|&element| element == GeomWithData::new(rectangle, edge))
        );
    }

    fn face_rectangle(
        rtreed_dcel: &RTreedStableDcel<(i32, i32)>,
        face: FaceId,
    ) -> Rectangle<(i32, i32)> {
        RTreedStableDcel::<(i32, i32)>::rectangle_from_vertex_weights(
            rtreed_dcel
                .dcel
                .face_vertexes(face)
                .map(|vertex| rtreed_dcel.dcel.vertex_weight(vertex).clone()),
        )
    }
}
