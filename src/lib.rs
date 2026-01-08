// SPDX-FileCopyrightText: 2026 dcel contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

mod walkers;

use maplike::{Get, Insert, Push, Remove};
pub use walkers::{
    CcwEdgesIter, CcwEdgesWalker, CcwHalfEdgesIter, CcwHalfEdgesWalker, CwEdgesIter, CwEdgesWalker,
    CwHalfEdgesIter, CwHalfEdgesWalker, FaceEdgesIter, FaceEdgesWalker, FaceHalfEdgesIter,
    FaceHalfEdgesWalker,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VertexId(usize);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HalfEdgeId(usize);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EdgeId(HalfEdgeId, HalfEdgeId);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FaceId(usize);

#[derive(Clone, Debug)]
pub struct Vertex<VW> {
    outward_half_edge: HalfEdgeId,
    weight: VW,
}

#[derive(Clone, Debug)]
pub struct HalfEdge<HEW> {
    origin: VertexId,
    twin: HalfEdgeId,
    prev: HalfEdgeId,
    next: HalfEdgeId,
    face: FaceId,
    weight: HEW,
}

#[derive(Clone, Debug)]
pub struct Face<FW> {
    incident_half_edge: Option<HalfEdgeId>,
    weight: FW,
}

#[derive(Clone, Debug)]
pub struct Dcel<
    VW,
    HEW = (),
    FW = (),
    VC = Vec<Vertex<VW>>,
    HEC = Vec<HalfEdge<HEW>>,
    FC = Vec<Face<FW>>,
> {
    vertexes: VC,
    half_edges: HEC,
    faces: FC,
    vertex_weight_marker: std::marker::PhantomData<VW>,
    edge_weight_marker: std::marker::PhantomData<HEW>,
    face_weight_marker: std::marker::PhantomData<FW>,
}

impl<VW, HEW, FW: Default, VC: Default, HEC: Default, FC: Default + Push<usize, Item = FW>>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    #[inline(always)]
    pub fn new() -> Self {
        let mut faces = FC::default();

        // Push the outermost face.
        faces.push(FW::default());

        Self {
            vertexes: VC::default(),
            half_edges: HEC::default(),
            faces,
            vertex_weight_marker: std::marker::PhantomData,
            edge_weight_marker: std::marker::PhantomData,
            face_weight_marker: std::marker::PhantomData,
        }
    }
}

impl<VW, HEW, FW: Default, VC: Default, HEC: Default, FC: Default + Push<usize, Item = FW>> Default
    for Dcel<VW, HEW, FW, VC, HEC, FC>
{
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl<VW, HEW, FW, VC, HEC, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    #[inline(always)]
    pub fn from_parts(vertexes: VC, half_edges: HEC, faces: FC) -> Self {
        Self {
            vertexes,
            half_edges,
            faces,
            vertex_weight_marker: std::marker::PhantomData,
            edge_weight_marker: std::marker::PhantomData,
            face_weight_marker: std::marker::PhantomData,
        }
    }
}

impl<VW, HEW, FW, VC, HEC, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    #[inline(always)]
    pub fn vertexes(&self) -> &VC {
        &self.vertexes
    }

    #[inline(always)]
    pub fn half_edges(&self) -> &HEC {
        &self.half_edges
    }

    #[inline(always)]
    pub fn faces(&self) -> &FC {
        &self.faces
    }

    #[inline(always)]
    pub fn dissolve(self) -> (VC, HEC, FC) {
        (self.vertexes, self.half_edges, self.faces)
    }
}

impl<
    VW: Clone,
    HEW: Clone + Default,
    FW: Clone + Default,
    VC: Get<usize, Item = Vertex<VW>> + Insert<usize> + Push<usize>,
    HEC: Get<usize, Item = HalfEdge<HEW>> + Insert<usize> + Push<usize>,
    FC: Get<usize, Item = Face<FW>> + Insert<usize> + Push<usize>,
> Dcel<VW, HEW, FW, VC, HEC, FC>
{
    pub fn insert_polygon_in_face(
        &mut self,
        target_face: FaceId,
        vertex_weights: impl IntoIterator<Item = VW>,
    ) {
        self.insert_polygon_in_face_with_all_weights(
            target_face,
            vertex_weights,
            std::iter::repeat((HEW::default(), HEW::default())),
            FW::default(),
        );
    }
}

impl<
    VW: Clone,
    HEW: Clone,
    FW: Clone,
    VC: Get<usize, Item = Vertex<VW>> + Insert<usize> + Push<usize>,
    HEC: Get<usize, Item = HalfEdge<HEW>> + Insert<usize> + Push<usize>,
    FC: Get<usize, Item = Face<FW>> + Insert<usize> + Push<usize>,
> Dcel<VW, HEW, FW, VC, HEC, FC>
{
    pub fn insert_polygon_in_face_with_all_weights(
        &mut self,
        outer_face: FaceId,
        vertex_weights: impl IntoIterator<Item = VW>,
        edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
        face_weight: FW,
    ) {
        let new_face = self.add_unwired_face(face_weight);
        let vertexes = self.add_unwired_polygon_vertexes(vertex_weights);
        let edges = self.add_unwired_polygon_edges(&vertexes, new_face, outer_face, edge_weights);

        self.wire_face_edges_vertexes(new_face, &edges);
    }

    fn add_unwired_polygon_vertexes(
        &mut self,
        vertex_weights: impl IntoIterator<Item = VW>,
    ) -> Vec<VertexId> {
        let vertexes: Vec<VertexId> = vertex_weights
            .into_iter()
            .map(|vertex_weight| self.add_unwired_vertex(vertex_weight))
            .collect();

        vertexes
    }

    fn add_unwired_polygon_edges(
        &mut self,
        vertexes: &[VertexId],
        new_face: FaceId,
        target_face: FaceId,
        edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
    ) -> Vec<EdgeId> {
        let vertexes_circular_pair_windows = vertexes
            .iter()
            .zip(vertexes.iter().skip(1).chain(vertexes.iter().take(1)));

        let mut edges = vec![];
        let edge_weights: Vec<(HEW, HEW)> = edge_weights.into_iter().collect();

        for ((from_vertex, to_vertex), (half_edge_weight, twin_half_edge_weight)) in
            vertexes_circular_pair_windows.zip(edge_weights.clone().into_iter())
        {
            let edge = self.add_unwired_edge(
                *from_vertex,
                *to_vertex,
                new_face,
                target_face,
                half_edge_weight,
                twin_half_edge_weight,
            );
            edges.push(edge);
        }

        edges
    }
}

impl<
    VW: Clone,
    HEW: Clone,
    FW: Clone + Get<usize>,
    VC: Get<usize, Item = Vertex<VW>> + Insert<usize> + Push<usize>,
    HEC: Get<usize, Item = HalfEdge<HEW>> + Insert<usize> + Push<usize>,
    FC: Get<usize, Item = Face<FW>> + Insert<usize> + Push<usize>,
> Dcel<VW, HEW, FW, VC, HEC, FC>
{
    /// Partition a face into triangles by inserting a vertex inside and then
    /// adding edges between it and the original face's vertexes.
    ///
    /// The original face is reused for the first triangle. New faces are
    /// created for all the other triangles.
    ///
    /// Returns the new vertex id together with the face ids of all the new
    /// triangles.
    pub fn triangulate_around_vertex_with_all_weights(
        &mut self,
        perimeter_face: FaceId,
        inner_vertex_weight: VW,
        inner_edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
        triangle_face_weights: impl IntoIterator<Item = FW>,
    ) {
        let inner_vertex = self.add_unwired_vertex(inner_vertex_weight);
        let triangle_faces =
            self.add_unwired_triangulation_faces(perimeter_face, triangle_face_weights);
        let inner_edges = self.add_unwired_triangulation_edges(
            perimeter_face,
            inner_vertex,
            &triangle_faces,
            inner_edge_weights,
        );
        self.wire_triangulation_faces_edges_vertexes(perimeter_face, &inner_edges);
    }

    fn add_unwired_triangulation_faces(
        &mut self,
        first_face: FaceId,
        triangle_face_weights: impl IntoIterator<Item = FW>,
    ) -> Vec<FaceId> {
        let mut triangle_faces = vec![];

        let mut face_weights_iter = triangle_face_weights.into_iter();
        self.faces.insert(
            first_face.0,
            Face {
                incident_half_edge: self.faces.get(&first_face.0).unwrap().incident_half_edge,
                weight: face_weights_iter.next().unwrap(),
            },
        );
        triangle_faces.push(first_face);

        for face_weight in face_weights_iter {
            triangle_faces.push(self.add_unwired_face(face_weight));
        }

        triangle_faces
    }

    fn add_unwired_triangulation_edges(
        &mut self,
        perimeter_face: FaceId,
        inner_vertex: VertexId,
        triangle_faces: &[FaceId],
        inner_edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
    ) -> Vec<EdgeId> {
        let mut inner_edge_weights = inner_edge_weights.into_iter();
        let mut face_vertexes_walker = self.face_vertexes(perimeter_face);
        let mut edges = vec![];

        let mut i: usize = 0;

        while let Some(perimeter_vertex) = face_vertexes_walker.next(self) {
            let (weight, twin_weight) = inner_edge_weights.next().unwrap();

            edges.push(self.add_unwired_edge(
                perimeter_vertex,
                inner_vertex,
                triangle_faces[i.wrapping_sub_signed(1)],
                triangle_faces[i],
                weight,
                twin_weight,
            ));

            i += 1;
        }

        edges
    }

    fn wire_triangulation_faces_edges_vertexes(
        &mut self,
        perimeter_face: FaceId,
        inner_edges: &[EdgeId],
    ) {
        let inner_edges_circular_pair_windows = inner_edges
            .iter()
            .zip(inner_edges.iter().skip(1).chain(inner_edges.iter().take(1)));
        let mut perimeter_half_edges_walker = self.face_half_edges(perimeter_face);

        for (inner_edge, next_inner_edge) in inner_edges_circular_pair_windows {
            let perimeter_half_edge = perimeter_half_edges_walker.next(self).unwrap();

            self.wire_face_edges_vertexes(
                perimeter_face,
                &[
                    self.full_edge(perimeter_half_edge),
                    self.full_edge(next_inner_edge.0),
                    self.full_edge(inner_edge.1),
                ],
            );
        }
    }
}

impl<
    VW: Clone,
    HEW: Clone,
    FW: Clone,
    VC: Get<usize, Item = Vertex<VW>> + Insert<usize> + Remove<usize>,
    HEC: Get<usize, Item = HalfEdge<HEW>> + Insert<usize> + Remove<usize>,
    FC: Get<usize, Item = Face<FW>> + Insert<usize> + Remove<usize>,
> Dcel<VW, HEW, FW, VC, HEC, FC>
{
    pub fn merge_faces_around_vertex(&mut self, inner_vertex: VertexId) {
        let absorbing_face = self.face_in_front(
            self.vertexes
                .get(&inner_vertex.0)
                .unwrap()
                .outward_half_edge,
        );
        self.absorb_faces_around_vertex(absorbing_face, inner_vertex);
    }

    pub fn absorb_faces_around_vertex(&mut self, absorbing_face: FaceId, inner_vertex: VertexId) {
        let initial_half_edge = self
            .vertexes
            .get(&inner_vertex.0)
            .unwrap()
            .outward_half_edge;
        let initial_edge = self.full_edge(
            self.vertexes
                .get(&inner_vertex.0)
                .unwrap()
                .outward_half_edge,
        );
        let inner_edges: Vec<EdgeId> = self.cw_edges(initial_edge).iter(self).collect();
        let perimeter_edges: Vec<EdgeId> = self
            .edges_with_excludes(initial_edge, inner_edges.clone())
            .iter(self)
            .collect();

        self.remove_faces(
            self.cw_faces(self.face_in_front(initial_half_edge))
                .iter(self)
                .collect::<Vec<FaceId>>(),
        );
        self.remove_edges(inner_edges);
        self.remove_vertex(inner_vertex);

        self.wire_face_edges_vertexes(absorbing_face, &perimeter_edges);
    }
}

impl<
    VW: Clone,
    HEW: Clone,
    FW: Clone,
    VC: Get<usize, Item = Vertex<VW>> + Insert<usize> + Remove<usize>,
    HEC: Get<usize, Item = HalfEdge<HEW>> + Insert<usize> + Remove<usize>,
    FC: Get<usize, Item = Face<FW>> + Insert<usize> + Remove<usize>,
> Dcel<VW, HEW, FW, VC, HEC, FC>
{
    pub fn merge_faces_over_edges_and_vertexes(
        &mut self,
        faces: impl IntoIterator<Item = FaceId>,
        edges: impl IntoIterator<Item = EdgeId>,
        vertexes: impl IntoIterator<Item = VertexId>,
    ) {
        let mut faces = faces.into_iter();
        let absorbing_face = faces.next().unwrap();

        self.absorb_faces_over_edges_and_vertexes(absorbing_face, faces, edges, vertexes)
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
            .edges_with_excludes(
                self.full_edge(
                    self.faces
                        .get(&absorbing_face.0)
                        .unwrap()
                        .incident_half_edge
                        .unwrap(),
                ),
                edges.clone(),
            )
            .iter(self)
            .collect();

        self.remove_faces(faces);
        self.remove_edges(edges);
        self.remove_vertexes(vertexes);

        self.wire_face_edges_vertexes(absorbing_face, &perimeter_edges);
    }
}

impl<
    VW: Clone,
    HEW: Clone,
    FW: Clone,
    VC: Get<usize, Item = Vertex<VW>> + Insert<usize>,
    HEC: Get<usize, Item = HalfEdge<HEW>> + Insert<usize>,
    FC: Get<usize, Item = Face<FW>> + Insert<usize>,
> Dcel<VW, HEW, FW, VC, HEC, FC>
{
    fn wire_face_edges_vertexes(&mut self, face: FaceId, edges: &[EdgeId]) {
        self.wire_face(face, edges[0].0);

        let edges_circular_pair_windows = edges
            .iter()
            .zip(edges.iter().skip(1).chain(edges.iter().take(1)));

        for (edge, next_edge) in edges_circular_pair_windows {
            self.wire_edge(*edge, *next_edge);
            self.wire_vertex(self.origin(edge.0), edge.0);
        }
    }
}

impl<VW, HEW, FW, VC: Push<usize, Item = Vertex<VW>>, HEC, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    fn add_unwired_vertex(&mut self, weight: VW) -> VertexId {
        VertexId(self.vertexes.push(Vertex {
            // Since we do not use optionals, we cannot use `None` as the uninitialized value.
            // So instead uninitialized edge ids are edge 0.
            // Initializing `.outward_edge` to a correct value is the
            // responsibility of the caller.
            outward_half_edge: HalfEdgeId(0),
            weight,
        }))
    }
}

impl<VW, HEW, FW, VC: Remove<usize, Item = Vertex<VW>>, HEC, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    fn remove_vertexes(&mut self, vertexes: impl IntoIterator<Item = VertexId>) {
        for vertex in vertexes.into_iter() {
            self.remove_vertex(vertex);
        }
    }

    fn remove_vertex(&mut self, vertex: VertexId) {
        self.vertexes.remove(&vertex.0);
    }
}

impl<VW: Clone, HEW, FW, VC: Get<usize, Item = Vertex<VW>> + Insert<usize>, HEC, FC>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    fn wire_vertex(&mut self, vertex: VertexId, outward_half_edge: HalfEdgeId) {
        self.vertexes.insert(
            vertex.0,
            Vertex {
                outward_half_edge,
                weight: self.vertexes.get(&vertex.0).unwrap().weight.clone(),
            },
        )
    }
}

impl<VW, HEW: Clone, FW, VC, HEC: Insert<usize, Item = HalfEdge<HEW>> + Push<usize>, FC>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    fn add_unwired_edge(
        &mut self,
        origin: VertexId,
        twin_origin: VertexId,
        face: FaceId,
        twin_face: FaceId,
        weight: HEW,
        twin_weight: HEW,
    ) -> EdgeId {
        let half_edge = HalfEdgeId(self.half_edges.push(HalfEdge {
            origin,
            // Uninitialized as edge 0 before until the twin is created in the next few lines of this method.
            twin: HalfEdgeId(0),
            // Uninitialized as edge 0. Initializing `.prev` and `.next` to
            // correct value is the responsibility of the caller.
            prev: HalfEdgeId(0),
            next: HalfEdgeId(0),
            face,
            weight: weight.clone(),
        }));

        let twin_half_edge = HalfEdgeId(self.half_edges.push(HalfEdge {
            origin: twin_origin,
            twin: half_edge,
            // Uninitialized as edge 0. Initializing `.prev` and `.next` to
            // correct value is the responsibility of the caller.
            prev: HalfEdgeId(0),
            next: HalfEdgeId(0),
            face: twin_face,
            weight: twin_weight,
        }));

        // Now actually initialize `halfedge0`'s twin.
        // PERF: This could actually be optimized away by making it the
        // responsibility of the caller
        self.half_edges.insert(
            half_edge.0,
            HalfEdge {
                origin,
                // Uninitialized as edge 0 before until the twin is created in the next few lines of this method.
                twin: twin_half_edge,
                // Uninitialized as edge 0. Initializing `.prev` and `.next` to
                // correct value is the responsibility of the caller.
                prev: HalfEdgeId(0),
                next: HalfEdgeId(0),
                face,
                weight,
            },
        );

        EdgeId(half_edge, twin_half_edge)
    }
}

impl<VW, HEW, FW, VC, HEC: Remove<usize, Item = HalfEdge<HEW>>, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    fn remove_edges(&mut self, edges: impl IntoIterator<Item = EdgeId>) {
        for edge in edges.into_iter() {
            self.remove_edge(edge);
        }
    }

    fn remove_edge(&mut self, edge: EdgeId) {
        self.half_edges.remove(&edge.0.0);
        self.half_edges.remove(&edge.1.0);
    }
}

impl<VW, HEW: Clone, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>> + Insert<usize>, FC>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    fn wire_edge(&mut self, edge: EdgeId, next_edge: EdgeId) {
        self.half_edges.insert(
            edge.0.0,
            HalfEdge {
                next: next_edge.0,
                ..self.half_edges.get(&edge.0.0).unwrap().clone()
            },
        );
        self.half_edges.insert(
            next_edge.1.0,
            HalfEdge {
                next: edge.1,
                ..self.half_edges.get(&edge.0.0).unwrap().clone()
            },
        );
    }
}

impl<VW, HEW, FW, VC, HEC, FC: Push<usize, Item = Face<FW>>> Dcel<VW, HEW, FW, VC, HEC, FC> {
    fn add_unwired_face(&mut self, weight: FW) -> FaceId {
        FaceId(self.faces.push(Face {
            incident_half_edge: None,
            weight,
        }))
    }
}

impl<VW, HEW, FW, VC, HEC, FC: Remove<usize>> Dcel<VW, HEW, FW, VC, HEC, FC> {
    fn remove_faces(&mut self, faces: impl IntoIterator<Item = FaceId>) {
        for face in faces.into_iter() {
            self.remove_face(face);
        }
    }

    fn remove_face(&mut self, face: FaceId) {
        self.faces.remove(&face.0);
    }
}

impl<VW, HEW, FW: Clone, VC, HEC, FC: Get<usize, Item = Face<FW>> + Insert<usize>>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    fn wire_face(&mut self, face: FaceId, incident_half_edge: HalfEdgeId) {
        self.faces.insert(
            face.0,
            Face {
                incident_half_edge: Some(incident_half_edge),
                weight: self.faces.get(&face.0).unwrap().weight.clone(),
            },
        );
    }
}

impl<VW, HEW, FW, VC: Get<usize, Item = Vertex<VW>>, HEC, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    fn outward_half_edge(&self, vertex: VertexId) -> HalfEdgeId {
        self.vertexes.get(&vertex.0).unwrap().outward_half_edge
    }
}

impl<VW, HEW, FW, VC: Get<usize, Item = Vertex<VW>>, HEC: Get<usize, Item = HalfEdge<HEW>>, FC>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    fn prev_vertex(&self, vertex: VertexId) -> VertexId {
        self.origin(self.prev_half_edge(self.outward_half_edge(vertex)))
    }

    fn next_vertex(&self, vertex: VertexId) -> VertexId {
        self.origin(self.next_half_edge(self.outward_half_edge(vertex)))
    }
}

impl<VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    pub fn origin(&self, half_edge: HalfEdgeId) -> VertexId {
        self.half_edges.get(&half_edge.0).unwrap().origin
    }

    pub fn endpoints(&self, edge: EdgeId) -> (VertexId, VertexId) {
        (self.origin(edge.0), self.origin(edge.1))
    }

    pub fn twin(&self, half_edge: HalfEdgeId) -> HalfEdgeId {
        self.half_edges.get(&half_edge.0).unwrap().twin
    }

    pub fn full_edge(&self, half_edge: HalfEdgeId) -> EdgeId {
        EdgeId(half_edge, self.twin(half_edge))
    }

    pub fn face_in_front(&self, half_edge: HalfEdgeId) -> FaceId {
        self.half_edges.get(&half_edge.0).unwrap().face
    }

    pub fn face_behind(&self, half_edge: HalfEdgeId) -> FaceId {
        self.face_in_front(self.twin(half_edge))
    }

    pub fn edge_faces(&self, edge: EdgeId) -> (FaceId, FaceId) {
        (self.face_in_front(edge.0), self.face_behind(edge.1))
    }

    pub fn prev_half_edge(&self, half_edge: HalfEdgeId) -> HalfEdgeId {
        self.half_edges.get(&half_edge.0).unwrap().prev
    }

    pub fn next_half_edge(&self, half_edge: HalfEdgeId) -> HalfEdgeId {
        self.half_edges.get(&half_edge.0).unwrap().next
    }

    pub fn prev_edge(&self, edge: EdgeId) -> EdgeId {
        let next_half_edge = self.half_edges.get(&edge.0.0).unwrap().prev;
        let next_twin_half_edge = self.half_edges.get(&next_half_edge.0).unwrap().twin;

        EdgeId(next_half_edge, next_twin_half_edge)
    }

    pub fn next_edge(&self, edge: EdgeId) -> EdgeId {
        let next_half_edge = self.half_edges.get(&edge.0.0).unwrap().next;
        let next_twin_half_edge = self.half_edges.get(&next_half_edge.0).unwrap().twin;

        EdgeId(next_half_edge, next_twin_half_edge)
    }

    pub fn cw_half_edge(&self, half_edge: HalfEdgeId) -> HalfEdgeId {
        self.twin(self.prev_half_edge(half_edge))
    }

    pub fn ccw_half_edge(&self, half_edge: HalfEdgeId) -> HalfEdgeId {
        self.next_half_edge(self.twin(half_edge))
    }

    pub fn cw_edge(&self, edge: EdgeId) -> EdgeId {
        self.full_edge(self.cw_half_edge(edge.0))
    }

    pub fn ccw_edge(&self, edge: EdgeId) -> EdgeId {
        self.full_edge(self.ccw_half_edge(edge.0))
    }
}
