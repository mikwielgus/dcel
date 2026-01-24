// SPDX-FileCopyrightText: 2026 dcel contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

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

pub type RTreedStableDcel<P, VW, HEW, FW> = RTreedDcel<
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

        for face in &faces {
            self.add_face_with_edges_to_rtrees(
                *face,
                Self::rectangle_from_vertex_weights(
                    self.dcel
                        .face_vertexes(*face)
                        .map(|vertex| self.dcel.vertex_weight(vertex).clone()),
                ),
            );
        }

        faces
    }

    pub fn insert_mesh_in_face(
        &mut self,
        face_polygons: impl IntoIterator<Item = impl IntoIterator<Item = VW>>,
    ) -> Vec<FaceId> {
        let faces = self.dcel.insert_mesh_in_face(face_polygons);

        for face in &faces {
            self.add_face_with_edges_to_rtrees(
                *face,
                Self::rectangle_from_vertex_weights(
                    self.dcel
                        .face_vertexes(*face)
                        .map(|vertex| self.dcel.vertex_weight(vertex).clone()),
                ),
            );
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
        self.add_face_with_edges_to_rtrees(
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
        self.add_face_with_edges_to_rtrees(
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
        self.add_face_with_edges_to_rtrees(
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
        self.add_face_with_edges_to_rtrees(
            face,
            Self::rectangle_from_vertex_weights(vertex_weights),
        );

        face
    }

    fn add_face_with_edges_to_rtrees(&mut self, face: FaceId, face_rectangle: Rectangle<P>) {
        self.faces_rtree
            .insert(GeomWithData::new(face_rectangle, face));

        for edge in self.dcel.face_edges(face) {
            let endpoints = self.dcel.endpoints(edge);

            self.edges_rtree.insert(GeomWithData::new(
                Rectangle::from_corners(
                    self.dcel.vertex_weight(endpoints.0).clone().into(),
                    self.dcel.vertex_weight(endpoints.1).clone().into(),
                ),
                edge,
            ));
        }
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
        let absorbing_face = self.dcel.face_in_front(
            self.dcel
                .vertexes
                .get(&inner_vertex.id())
                .unwrap()
                .outgoing_next_half_edge,
        );
        self.absorb_faces_around_vertex(absorbing_face, inner_vertex);
    }

    pub fn absorb_faces_around_vertex(&mut self, absorbing_face: FaceId, inner_vertex: VertexId) {
        let initial_half_edge = self
            .dcel
            .vertexes
            .get(&inner_vertex.id())
            .unwrap()
            .outgoing_next_half_edge;
        let initial_edge = self.dcel.full_edge(
            self.dcel
                .vertexes
                .get(&inner_vertex.id())
                .unwrap()
                .outgoing_next_half_edge,
        );
        let inner_edges: Vec<EdgeId> = self.dcel.spokes(initial_edge).collect();
        let perimeter_edges: Vec<EdgeId> = self
            .dcel
            .vertex_rim_edges(inner_vertex)
            .collect::<Vec<EdgeId>>();

        self.absorb_faces_over_edges_and_vertexes_in_perimeter(
            absorbing_face,
            self.dcel
                .interspokes(initial_half_edge)
                .filter(|face| face.id() != absorbing_face.id())
                .collect::<Vec<FaceId>>(),
            inner_edges,
            [inner_vertex],
            &perimeter_edges,
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
        let perimeter_edges: Vec<EdgeId> = self
            .dcel
            .circulate_edges_with_excludes(
                self.dcel.full_edge(
                    self.dcel
                        .faces
                        .get(&absorbing_face.id())
                        .unwrap()
                        .incident_half_edge
                        .unwrap(),
                ),
                edges.clone(),
            )
            .collect();

        self.absorb_faces_over_edges_and_vertexes_in_perimeter(
            absorbing_face,
            faces,
            edges,
            vertexes,
            &perimeter_edges,
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
                        .spokes_reverse(self.dcel.vertex_next_edge(vertex))
                        .all(|edge| half_edges_counter.is_inner_edge(edge))
                })
                // PERF: Needless collect?
                .collect::<Vec<VertexId>>(),
            &half_edges_counter
                .outer_edges(&self.dcel)
                .collect::<Vec<EdgeId>>(),
        );
    }

    fn absorb_faces_over_edges_and_vertexes_in_perimeter(
        &mut self,
        absorbing_face: FaceId,
        faces_to_absorb: impl IntoIterator<Item = FaceId>,
        edges_to_remove: impl IntoIterator<Item = EdgeId>,
        vertexes_to_remove: impl IntoIterator<Item = VertexId>,
        perimeter_edges: &[EdgeId],
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

        self.dcel.absorb_faces_over_edges_and_vertexes_in_perimeter(
            absorbing_face,
            faces_to_absorb,
            edges_to_remove,
            vertexes_to_remove,
            perimeter_edges,
        );
    }
}

impl<P: Point, VW: Into<P>, HEW, FW, VC, HEC, FC> RTreedDcel<P, VW, HEW, FW, VC, HEC, FC> {
    fn rectangle_from_vertex_weights(weights: impl IntoIterator<Item = VW>) -> Rectangle<P> {
        Rectangle::from_aabb(weights.into_iter().fold(AABB::new_empty(), |aabb, weight| {
            aabb.merged(&AABB::from_point(Into::<P>::into(weight)))
        }))
    }
}
