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

        for face in &faces {
            self.add_face_with_edges_to_rtrees(*face);
        }

        faces
    }

    pub fn insert_mesh_in_face(
        &mut self,
        face_polygons: impl IntoIterator<Item = impl IntoIterator<Item = VW>>,
    ) -> Vec<FaceId> {
        let faces = self.dcel.insert_mesh_in_face(face_polygons);

        for face in &faces {
            self.add_face_with_edges_to_rtrees(*face);
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
    pub fn merge_faces_around_vertex(&mut self, inner_vertex: VertexId) {
        let absorbing_face = self
            .dcel
            .face_in_front(self.dcel.outgoing_next_half_edge(inner_vertex));
        self.absorb_faces_around_vertex(absorbing_face, inner_vertex);
    }

    pub fn absorb_faces_around_vertex(&mut self, absorbing_face: FaceId, inner_vertex: VertexId) {
        let initial_half_edge = self.dcel.outgoing_next_half_edge(inner_vertex);
        let initial_edge = self.dcel.full_edge(initial_half_edge);

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

        // Find an initial edge that is not in the excluded list. Otherwise, the
        // circulator could end up starting from an excluded edge, which would
        // result in an infinite loop, as the termination condition depends on
        // returning to the initial edge.
        let initial_edge = self
            .dcel
            .face_edges(absorbing_face)
            .find(|edge| !edges.contains(edge))
            .unwrap();

        let perimeter_edges: Vec<EdgeId> = self
            .dcel
            .circulate_edges_with_excludes(initial_edge, edges.clone())
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

        // Remove the absorbing face from the R-tree before its shape changes,
        // which would otherwise invalidate its bbox and make it impossible to
        // access anymore.
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
            perimeter_edges,
        );

        // Insert the absorbing face back in the R-tree, with the updated bbox.
        self.add_face_to_rtree(absorbing_face);
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

        for edge in self.dcel.face_edges(face) {
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
    use rstar::{RTreeObject, primitives::GeomWithData};

    use crate::{
        EdgeId, FaceId, HalfEdgeId, RTreedStableDcel, StableDcel, VertexId, assert_face_boundary,
        init_dcel_with_3x3_hex_mesh,
    };

    #[test]
    fn test_merge_faces_around_vertex() {
        let mut dcel = init_dcel_with_3x3_hex_mesh!(RTreedStableDcel<(i32, i32)>);
        dcel.merge_faces_around_vertex(VertexId::new(8));

        // There are now eight faces in total: one unbounded and seven bounded.
        assert_eq!(dcel.dcel.faces().num_elements(), 8);
        assert_eq!(dcel.faces_rtree.size(), 7);

        // Among the remaining faces, one is now a dodecagon, and the remaining
        // six are hexagons.
        assert_face_boundary!(dcel.dcel, 0, 0);
        assert_face_boundary!(dcel.dcel, 1, 6);
        // Face 2 does not exist.
        assert_face_boundary!(dcel.dcel, 3, 6);
        // Face 4 does not exist.
        assert_face_boundary!(dcel.dcel, 5, 12);
        assert_face_boundary!(dcel.dcel, 6, 6);
        assert_face_boundary!(dcel.dcel, 7, 6);
        assert_face_boundary!(dcel.dcel, 8, 6);
        assert_face_boundary!(dcel.dcel, 9, 6);

        assert_face_bbox_validity(&dcel, 1);
        // Face 2 does not exist.
        assert_face_bbox_validity(&dcel, 3);
        // Face 4 does not exist.
        assert_face_bbox_validity(&dcel, 6);
        assert_face_bbox_validity(&dcel, 7);
        assert_face_bbox_validity(&dcel, 8);
        assert_face_bbox_validity(&dcel, 9);
    }

    #[test]
    fn test_absorb_faces_around_vertex() {
        let mut dcel = init_dcel_with_3x3_hex_mesh!(RTreedStableDcel<(i32, i32)>);
        dcel.absorb_faces_around_vertex(FaceId::new(2), VertexId::new(8));

        // There are now eight faces in total: one unbounded and seven bounded.
        assert_eq!(dcel.dcel.faces().num_elements(), 8);
        assert_eq!(dcel.faces_rtree.size(), 7);

        // Among the remaining faces, one is now a dodecagon, and the remaining
        // six are hexagons.
        assert_face_boundary!(dcel.dcel, 0, 0);
        assert_face_boundary!(dcel.dcel, 1, 6);
        assert_face_boundary!(dcel.dcel, 2, 12);
        assert_face_boundary!(dcel.dcel, 3, 6);
        // Face 4 does not exist.
        // Face 5 does not exist.
        assert_face_boundary!(dcel.dcel, 6, 6);
        assert_face_boundary!(dcel.dcel, 7, 6);
        assert_face_boundary!(dcel.dcel, 8, 6);
        assert_face_boundary!(dcel.dcel, 9, 6);

        assert_face_bbox_validity(&dcel, 1);
        assert_face_bbox_validity(&dcel, 2);
        assert_face_bbox_validity(&dcel, 3);
        // Face 4 does not exist.
        // Face 5 does not exist.
        assert_face_bbox_validity(&dcel, 6);
        assert_face_bbox_validity(&dcel, 7);
        assert_face_bbox_validity(&dcel, 8);
        assert_face_bbox_validity(&dcel, 9);
    }

    #[test]
    fn test_merge_faces_over_edge() {
        let mut dcel = init_dcel_with_3x3_hex_mesh!(RTreedStableDcel<(i32, i32)>);
        dcel.merge_faces_over_edges_and_vertexes(
            [FaceId::new(5), FaceId::new(6)],
            [EdgeId::new(HalfEdgeId::new(44), HalfEdgeId::new(45))],
            [],
        );

        assert_eq!(dcel.dcel.faces().num_elements(), 9);
        assert_eq!(dcel.faces_rtree.size(), 8);

        assert_face_boundary!(dcel.dcel, 0, 0);
        assert_face_boundary!(dcel.dcel, 1, 6);
        assert_face_boundary!(dcel.dcel, 2, 6);
        assert_face_boundary!(dcel.dcel, 3, 6);
        assert_face_boundary!(dcel.dcel, 4, 6);
        assert_face_boundary!(dcel.dcel, 5, 10);
        // Face 6 does not exist.
        assert_face_boundary!(dcel.dcel, 7, 6);
        assert_face_boundary!(dcel.dcel, 8, 6);
        assert_face_boundary!(dcel.dcel, 9, 6);

        assert_face_bbox_validity(&dcel, 1);
        assert_face_bbox_validity(&dcel, 2);
        assert_face_bbox_validity(&dcel, 3);
        assert_face_bbox_validity(&dcel, 4);
        assert_face_bbox_validity(&dcel, 5);
        // Face 6 does not exist.
        assert_face_bbox_validity(&dcel, 7);
        assert_face_bbox_validity(&dcel, 8);
        assert_face_bbox_validity(&dcel, 9);
    }

    #[test]
    fn test_absorb_face_into_face_over_edge() {
        let mut dcel = init_dcel_with_3x3_hex_mesh!(RTreedStableDcel<(i32, i32)>);
        dcel.absorb_faces_over_edges_and_vertexes(
            FaceId::new(6),
            [FaceId::new(5)],
            [EdgeId::new(HalfEdgeId::new(44), HalfEdgeId::new(45))],
            [],
        );

        assert_eq!(dcel.dcel.faces().num_elements(), 9);
        assert_eq!(dcel.faces_rtree.size(), 8);

        assert_face_boundary!(dcel.dcel, 0, 0);
        assert_face_boundary!(dcel.dcel, 1, 6);
        assert_face_boundary!(dcel.dcel, 2, 6);
        assert_face_boundary!(dcel.dcel, 3, 6);
        assert_face_boundary!(dcel.dcel, 4, 6);
        // Face 5 does not exist.
        assert_face_boundary!(dcel.dcel, 6, 10);
        assert_face_boundary!(dcel.dcel, 7, 6);
        assert_face_boundary!(dcel.dcel, 8, 6);
        assert_face_boundary!(dcel.dcel, 9, 6);

        assert_face_bbox_validity(&dcel, 1);
        assert_face_bbox_validity(&dcel, 2);
        assert_face_bbox_validity(&dcel, 3);
        assert_face_bbox_validity(&dcel, 4);
        // Face 5 does not exist.
        assert_face_bbox_validity(&dcel, 6);
        assert_face_bbox_validity(&dcel, 7);
        assert_face_bbox_validity(&dcel, 8);
        assert_face_bbox_validity(&dcel, 9);
    }

    #[test]
    fn merge_two_faces() {
        let mut dcel = init_dcel_with_3x3_hex_mesh!(RTreedStableDcel<(i32, i32)>);
        dcel.merge_faces([FaceId::new(7), FaceId::new(8)]);

        assert_eq!(dcel.dcel.faces().num_elements(), 9);
        assert_eq!(dcel.faces_rtree.size(), 8);

        assert_face_boundary!(dcel.dcel, 0, 0);
        assert_face_boundary!(dcel.dcel, 1, 6);
        assert_face_boundary!(dcel.dcel, 2, 6);
        assert_face_boundary!(dcel.dcel, 3, 6);
        assert_face_boundary!(dcel.dcel, 4, 6);
        assert_face_boundary!(dcel.dcel, 5, 6);
        assert_face_boundary!(dcel.dcel, 6, 6);
        assert_face_boundary!(dcel.dcel, 7, 10);
        // Face 8 does not exist.
        assert_face_boundary!(dcel.dcel, 9, 6);

        assert_face_bbox_validity(&dcel, 1);
        assert_face_bbox_validity(&dcel, 2);
        assert_face_bbox_validity(&dcel, 3);
        assert_face_bbox_validity(&dcel, 4);
        assert_face_bbox_validity(&dcel, 5);
        assert_face_bbox_validity(&dcel, 6);
        assert_face_bbox_validity(&dcel, 7);
        // Face 8 does not exist.
        assert_face_bbox_validity(&dcel, 9);
    }

    #[test]
    fn absorb_face_into_face() {
        let mut dcel = init_dcel_with_3x3_hex_mesh!(RTreedStableDcel<(i32, i32)>);
        dcel.absorb_faces(FaceId::new(8), [FaceId::new(7)]);

        assert_eq!(dcel.dcel.faces().num_elements(), 9);
        assert_eq!(dcel.faces_rtree.size(), 8);

        assert_face_boundary!(dcel.dcel, 0, 0);
        assert_face_boundary!(dcel.dcel, 1, 6);
        assert_face_boundary!(dcel.dcel, 2, 6);
        assert_face_boundary!(dcel.dcel, 3, 6);
        assert_face_boundary!(dcel.dcel, 4, 6);
        assert_face_boundary!(dcel.dcel, 5, 6);
        assert_face_boundary!(dcel.dcel, 6, 6);
        // Face 7 does not exist.
        assert_face_boundary!(dcel.dcel, 8, 10);
        assert_face_boundary!(dcel.dcel, 9, 6);

        assert_face_bbox_validity(&dcel, 1);
        assert_face_bbox_validity(&dcel, 2);
        assert_face_bbox_validity(&dcel, 3);
        assert_face_bbox_validity(&dcel, 4);
        assert_face_bbox_validity(&dcel, 5);
        assert_face_bbox_validity(&dcel, 6);
        // Face 7 does not exist.
        assert_face_bbox_validity(&dcel, 8);
        assert_face_bbox_validity(&dcel, 9);
    }

    // TODO: Triangulation tests.

    fn assert_face_bbox_validity(dcel: &RTreedStableDcel<(i32, i32)>, face: usize) {
        assert!(
            dcel.faces_rtree
                .locate_in_envelope(
                    &RTreedStableDcel::<(i32, i32)>::rectangle_from_vertex_weights(
                        dcel.dcel
                            .face_vertexes(FaceId::new(face))
                            .map(|vertex| dcel.dcel.vertex_weight(vertex).clone()),
                    )
                    .envelope(),
                )
                .any(|&element| element
                    == GeomWithData::new(
                        RTreedStableDcel::<(i32, i32)>::rectangle_from_vertex_weights(
                            dcel.dcel
                                .face_vertexes(FaceId::new(face))
                                .map(|vertex| dcel.dcel.vertex_weight(vertex).clone()),
                        ),
                        FaceId::new(face)
                    ))
        );
    }
}
