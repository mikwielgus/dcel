// SPDX-FileCopyrightText: 2026 dcel contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

mod iter;

pub use iter::{
    CcwEdgesIter, CcwEdgesWalker, CcwHalfEdgesIter, CcwHalfEdgesWalker, CwEdgesIter, CwEdgesWalker,
    CwHalfEdgesIter, CwHalfEdgesWalker, FaceEdgesIter, FaceEdgesWalker, FaceHalfEdgesIter,
    FaceHalfEdgesWalker,
};
use maplike::{Get, Insert, Push, Remove};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VertexId(usize);

impl VertexId {
    #[inline]
    pub fn id(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HalfEdgeId(usize);

impl HalfEdgeId {
    #[inline]
    pub fn id(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EdgeId(HalfEdgeId, HalfEdgeId);

impl EdgeId {
    #[inline]
    pub fn forward(self) -> HalfEdgeId {
        self.0
    }

    #[inline]
    pub fn backward(self) -> HalfEdgeId {
        self.1
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FaceId(usize);

impl FaceId {
    #[inline]
    pub fn id(self) -> usize {
        self.0
    }
}

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
    half_edge_weight_marker: std::marker::PhantomData<HEW>,
    face_weight_marker: std::marker::PhantomData<FW>,
}

impl<VW, HEW, FW: Default, VC: Default, HEC: Default, FC: Default + Push<usize, Item = FW>>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    #[inline]
    pub fn new() -> Self {
        let mut faces = FC::default();

        // Push the outermost face.
        faces.push(FW::default());

        Self {
            vertexes: VC::default(),
            half_edges: HEC::default(),
            faces,
            vertex_weight_marker: std::marker::PhantomData,
            half_edge_weight_marker: std::marker::PhantomData,
            face_weight_marker: std::marker::PhantomData,
        }
    }
}

impl<VW, HEW, FW: Default, VC: Default, HEC: Default, FC: Default + Push<usize, Item = FW>> Default
    for Dcel<VW, HEW, FW, VC, HEC, FC>
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<VW, HEW, FW, VC, HEC, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    #[inline]
    pub fn from_parts(vertexes: VC, half_edges: HEC, faces: FC) -> Self {
        Self {
            vertexes,
            half_edges,
            faces,
            vertex_weight_marker: std::marker::PhantomData,
            half_edge_weight_marker: std::marker::PhantomData,
            face_weight_marker: std::marker::PhantomData,
        }
    }
}

impl<VW, HEW, FW, VC, HEC, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    #[inline]
    pub fn vertexes(&self) -> &VC {
        &self.vertexes
    }

    #[inline]
    pub fn half_edges(&self) -> &HEC {
        &self.half_edges
    }

    #[inline]
    pub fn faces(&self) -> &FC {
        &self.faces
    }

    #[inline]
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
    pub fn insert_polygon(&mut self, vertex_weights: impl IntoIterator<Item = VW>) {
        self.insert_polygon_in_face(self.unbounded_face(), vertex_weights);
    }

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
    pub fn insert_polygon_with_all_weights(
        &mut self,
        vertex_weights: impl IntoIterator<Item = VW>,
        edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
        face_weight: FW,
    ) {
        self.insert_polygon_in_face_with_all_weights(
            self.unbounded_face(),
            vertex_weights,
            edge_weights,
            face_weight,
        );
    }

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

        for ((from_vertex, to_vertex), (forward_half_edge_weight, backward_half_edge_weight)) in
            vertexes_circular_pair_windows.zip(edge_weights.clone().into_iter())
        {
            let edge = self.add_unwired_edge(
                *from_vertex,
                *to_vertex,
                new_face,
                target_face,
                forward_half_edge_weight,
                backward_half_edge_weight,
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
    pub fn triangulate_face_around_vertex_with_all_weights(
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
            first_face.id(),
            Face {
                incident_half_edge: self.faces.get(&first_face.id()).unwrap().incident_half_edge,
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
        let mut face_vertexes_walker = self.face_vertexes(perimeter_face).walker();
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
        let mut perimeter_half_edges_walker = self.face_half_edges(perimeter_face).walker();

        for (inner_edge, next_inner_edge) in inner_edges_circular_pair_windows {
            let perimeter_half_edge = perimeter_half_edges_walker.next(self).unwrap();

            self.wire_face_edges_vertexes(
                perimeter_face,
                &[
                    self.full_edge(perimeter_half_edge),
                    self.full_edge(next_inner_edge.forward()),
                    self.full_edge(inner_edge.backward()),
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
                .get(&inner_vertex.id())
                .unwrap()
                .outward_half_edge,
        );
        self.absorb_faces_around_vertex(absorbing_face, inner_vertex);
    }

    pub fn absorb_faces_around_vertex(&mut self, absorbing_face: FaceId, inner_vertex: VertexId) {
        let initial_half_edge = self
            .vertexes
            .get(&inner_vertex.id())
            .unwrap()
            .outward_half_edge;
        let initial_edge = self.full_edge(
            self.vertexes
                .get(&inner_vertex.id())
                .unwrap()
                .outward_half_edge,
        );
        let inner_edges: Vec<EdgeId> = self.cw_edges(initial_edge).collect();
        let perimeter_edges: Vec<EdgeId> = self
            .edges_with_excludes(initial_edge, inner_edges.clone())
            .collect();

        self.remove_faces(
            self.cw_faces(self.face_in_front(initial_half_edge))
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
                        .get(&absorbing_face.id())
                        .unwrap()
                        .incident_half_edge
                        .unwrap(),
                ),
                edges.clone(),
            )
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
        self.wire_face(face, edges[0].forward());

        let edges_circular_pair_windows = edges
            .iter()
            .zip(edges.iter().skip(1).chain(edges.iter().take(1)));

        for (edge, next_edge) in edges_circular_pair_windows {
            self.wire_edge(*edge, *next_edge);
            self.wire_vertex(self.origin(edge.forward()), edge.forward());
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
        self.vertexes.remove(&vertex.id());
    }
}

impl<VW: Clone, HEW, FW, VC: Get<usize, Item = Vertex<VW>> + Insert<usize>, HEC, FC>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    fn wire_vertex(&mut self, vertex: VertexId, outward_half_edge: HalfEdgeId) {
        self.vertexes.insert(
            vertex.id(),
            Vertex {
                outward_half_edge,
                weight: self.vertexes.get(&vertex.id()).unwrap().weight.clone(),
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
        let forward_half_edge = HalfEdgeId(self.half_edges.push(HalfEdge {
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

        let backward_half_edge = HalfEdgeId(self.half_edges.push(HalfEdge {
            origin: twin_origin,
            twin: forward_half_edge,
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
            forward_half_edge.id(),
            HalfEdge {
                origin,
                // Uninitialized as edge 0 before until the twin is created in the next few lines of this method.
                twin: backward_half_edge,
                // Uninitialized as edge 0. Initializing `.prev` and `.next` to
                // correct value is the responsibility of the caller.
                prev: HalfEdgeId(0),
                next: HalfEdgeId(0),
                face,
                weight,
            },
        );

        EdgeId(forward_half_edge, backward_half_edge)
    }
}

impl<VW, HEW, FW, VC, HEC: Remove<usize, Item = HalfEdge<HEW>>, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    fn remove_edges(&mut self, edges: impl IntoIterator<Item = EdgeId>) {
        for edge in edges.into_iter() {
            self.remove_edge(edge);
        }
    }

    fn remove_edge(&mut self, edge: EdgeId) {
        self.half_edges.remove(&edge.forward().id());
        self.half_edges.remove(&edge.backward().id());
    }
}

impl<VW, HEW: Clone, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>> + Insert<usize>, FC>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    fn wire_edge(&mut self, edge: EdgeId, next_edge: EdgeId) {
        self.half_edges.insert(
            edge.forward().id(),
            HalfEdge {
                next: next_edge.forward(),
                ..self.half_edges.get(&edge.forward().id()).unwrap().clone()
            },
        );
        self.half_edges.insert(
            next_edge.backward().id(),
            HalfEdge {
                next: edge.backward(),
                ..self.half_edges.get(&edge.forward().id()).unwrap().clone()
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
        self.faces.remove(&face.id());
    }
}

impl<VW, HEW, FW: Clone, VC, HEC, FC: Get<usize, Item = Face<FW>> + Insert<usize>>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    fn wire_face(&mut self, face: FaceId, incident_half_edge: HalfEdgeId) {
        self.faces.insert(
            face.id(),
            Face {
                incident_half_edge: Some(incident_half_edge),
                weight: self.faces.get(&face.id()).unwrap().weight.clone(),
            },
        );
    }
}

impl<VW, HEW, FW, VC: Get<usize, Item = Vertex<VW>>, HEC, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    #[inline]
    fn outward_half_edge(&self, vertex: VertexId) -> HalfEdgeId {
        self.vertexes.get(&vertex.id()).unwrap().outward_half_edge
    }
}

impl<VW, HEW, FW, VC: Get<usize, Item = Vertex<VW>>, HEC, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    #[inline]
    fn vertex_weight(&self, vertex: VertexId) -> &VW {
        &self.vertexes.get(&vertex.id()).unwrap().weight
    }
}

impl<VW, HEW, FW, VC: Get<usize, Item = Vertex<VW>>, HEC: Get<usize, Item = HalfEdge<HEW>>, FC>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    #[inline]
    fn prev_vertex(&self, vertex: VertexId) -> VertexId {
        self.origin(self.prev_half_edge(self.outward_half_edge(vertex)))
    }

    #[inline]
    fn next_vertex(&self, vertex: VertexId) -> VertexId {
        self.origin(self.next_half_edge(self.outward_half_edge(vertex)))
    }
}

impl<VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    #[inline]
    pub fn origin(&self, half_edge: HalfEdgeId) -> VertexId {
        self.half_edges.get(&half_edge.id()).unwrap().origin
    }

    #[inline]
    pub fn endpoints(&self, edge: EdgeId) -> (VertexId, VertexId) {
        (self.origin(edge.forward()), self.origin(edge.backward()))
    }

    #[inline]
    pub fn twin(&self, half_edge: HalfEdgeId) -> HalfEdgeId {
        self.half_edges.get(&half_edge.id()).unwrap().twin
    }

    #[inline]
    pub fn full_edge(&self, half_edge: HalfEdgeId) -> EdgeId {
        EdgeId(half_edge, self.twin(half_edge))
    }

    #[inline]
    pub fn face_in_front(&self, half_edge: HalfEdgeId) -> FaceId {
        self.half_edges.get(&half_edge.id()).unwrap().face
    }

    #[inline]
    pub fn face_behind(&self, half_edge: HalfEdgeId) -> FaceId {
        self.face_in_front(self.twin(half_edge))
    }

    #[inline]
    pub fn edge_faces(&self, edge: EdgeId) -> (FaceId, FaceId) {
        (
            self.face_in_front(edge.forward()),
            self.face_behind(edge.backward()),
        )
    }

    #[inline]
    pub fn prev_half_edge(&self, half_edge: HalfEdgeId) -> HalfEdgeId {
        self.half_edges.get(&half_edge.id()).unwrap().prev
    }

    #[inline]
    pub fn next_half_edge(&self, half_edge: HalfEdgeId) -> HalfEdgeId {
        self.half_edges.get(&half_edge.id()).unwrap().next
    }

    #[inline]
    pub fn prev_edge(&self, edge: EdgeId) -> EdgeId {
        let next_forward_half_edge = self.half_edges.get(&edge.forward().id()).unwrap().prev;
        let next_backward_half_edge = self
            .half_edges
            .get(&next_forward_half_edge.id())
            .unwrap()
            .twin;

        EdgeId(next_forward_half_edge, next_backward_half_edge)
    }

    #[inline]
    pub fn next_edge(&self, edge: EdgeId) -> EdgeId {
        let next_forward_half_edge = self.half_edges.get(&edge.forward().id()).unwrap().next;
        let next_backward_half_edge = self
            .half_edges
            .get(&next_forward_half_edge.id())
            .unwrap()
            .twin;

        EdgeId(next_forward_half_edge, next_backward_half_edge)
    }

    #[inline]
    pub fn cw_half_edge(&self, half_edge: HalfEdgeId) -> HalfEdgeId {
        self.twin(self.prev_half_edge(half_edge))
    }

    #[inline]
    pub fn ccw_half_edge(&self, half_edge: HalfEdgeId) -> HalfEdgeId {
        self.next_half_edge(self.twin(half_edge))
    }

    #[inline]
    pub fn cw_edge(&self, edge: EdgeId) -> EdgeId {
        self.full_edge(self.cw_half_edge(edge.forward()))
    }

    #[inline]
    pub fn ccw_edge(&self, edge: EdgeId) -> EdgeId {
        self.full_edge(self.ccw_half_edge(edge.forward()))
    }

    #[inline]
    pub fn half_edge_weight(&self, half_edge: HalfEdgeId) -> &HEW {
        &self.half_edges.get(&half_edge.id()).unwrap().weight
    }

    #[inline]
    pub fn edge_weights(&self, edge: EdgeId) -> (&HEW, &HEW) {
        (
            self.half_edge_weight(edge.forward()),
            self.half_edge_weight(edge.backward()),
        )
    }
}

impl<VW, HEW, FW, VC, HEC, FC: Get<usize, Item = Face<FW>>> Dcel<VW, HEW, FW, VC, HEC, FC> {
    #[inline]
    pub fn incident_half_edge(&self, face: FaceId) -> Option<HalfEdgeId> {
        self.faces.get(&face.id()).unwrap().incident_half_edge
    }

    #[inline]
    pub fn face_weight(&self, face: FaceId) -> &FW {
        &self.faces.get(&face.id()).unwrap().weight
    }
}

impl<VW, HEW, FW, VC, HEC, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    /// Returns the id of the unbounded face.
    ///
    /// The unbounded face is always the first element of the face list.
    #[inline]
    pub fn unbounded_face(&self) -> FaceId {
        FaceId(0)
    }
}
