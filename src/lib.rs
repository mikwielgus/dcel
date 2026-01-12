// SPDX-FileCopyrightText: 2026 dcel contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

mod insert;
mod iter;
mod merge;
mod split;
mod track;
mod triangulate;

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
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    #[inline]
    pub fn id(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HalfEdgeId(usize);

impl HalfEdgeId {
    #[inline]
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    #[inline]
    pub fn id(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EdgeId(HalfEdgeId, HalfEdgeId);

impl EdgeId {
    #[inline]
    pub fn new(forward: HalfEdgeId, backward: HalfEdgeId) -> EdgeId {
        Self(forward, backward)
    }

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
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    #[inline]
    pub fn id(self) -> usize {
        self.0
    }
}

#[derive(Clone, Debug)]
pub struct Vertex<VW> {
    outgoing_next_half_edge: HalfEdgeId,
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

impl<VW, HEW, FW: Default, VC: Default, HEC: Default, FC: Default + Push<usize, Item = Face<FW>>>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    #[inline]
    pub fn new() -> Self {
        let mut faces = FC::default();

        // Push the outermost face.
        faces.push(Face {
            incident_half_edge: None,
            weight: FW::default(),
        });

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

impl<VW, HEW, FW: Default, VC: Default, HEC: Default, FC: Default + Push<usize, Item = Face<FW>>>
    Default for Dcel<VW, HEW, FW, VC, HEC, FC>
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
    HEW: Clone,
    FW: Clone,
    VC: Get<usize, Item = Vertex<VW>> + Insert<usize>,
    HEC: Get<usize, Item = HalfEdge<HEW>> + Insert<usize>,
    FC: Get<usize, Item = Face<FW>> + Insert<usize>,
> Dcel<VW, HEW, FW, VC, HEC, FC>
{
    fn wire_edge_chain(&mut self, face: FaceId, edges: &[EdgeId]) {
        self.wire_face(face, edges[0].forward());

        let edges_circular_pair_windows = edges
            .iter()
            .zip(edges.iter().skip(1).chain(edges.iter().take(1)));

        for (edge, next_edge) in edges_circular_pair_windows {
            self.wire_edge(*edge, *next_edge);
            self.wire_vertex(self.origin(edge.forward()), edge.forward());
        }
    }

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
            outgoing_next_half_edge: HalfEdgeId(0),
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
    fn wire_vertex(&mut self, vertex: VertexId, outgoing_half_edge: HalfEdgeId) {
        self.vertexes.insert(
            vertex.id(),
            Vertex {
                outgoing_next_half_edge: outgoing_half_edge,
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
                ..self
                    .half_edges
                    .get(&next_edge.backward().id())
                    .unwrap()
                    .clone()
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
    fn outgoing_next_half_edge(&self, vertex: VertexId) -> HalfEdgeId {
        self.vertexes
            .get(&vertex.id())
            .unwrap()
            .outgoing_next_half_edge
    }
}

impl<VW, HEW, FW, VC: Get<usize, Item = Vertex<VW>>, HEC: Get<usize, Item = HalfEdge<HEW>>, FC>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    #[inline]
    fn incoming_next_half_edge(&self, vertex: VertexId) -> HalfEdgeId {
        self.twin(self.outgoing_next_half_edge(vertex))
    }

    #[inline]
    fn vertex_next_edge(&self, vertex: VertexId) -> EdgeId {
        EdgeId(
            self.outgoing_next_half_edge(vertex),
            self.incoming_next_half_edge(vertex),
        )
    }

    #[inline]
    fn incoming_prev_half_edge(&self, vertex: VertexId) -> HalfEdgeId {
        self.prev_half_edge(self.outgoing_next_half_edge(vertex))
    }

    #[inline]
    fn outgoing_prev_half_edge(&self, vertex: VertexId) -> HalfEdgeId {
        self.twin(self.incoming_prev_half_edge(vertex))
    }

    #[inline]
    fn vertex_prev_edge(&self, vertex: VertexId) -> EdgeId {
        EdgeId(
            self.incoming_prev_half_edge(vertex),
            self.outgoing_prev_half_edge(vertex),
        )
    }
}

impl<VW, HEW, FW, VC: Get<usize, Item = Vertex<VW>>, HEC, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    #[inline]
    pub fn vertex_weight(&self, vertex: VertexId) -> &VW {
        &self.vertexes.get(&vertex.id()).unwrap().weight
    }
}

impl<VW, HEW, FW, VC: Get<usize, Item = Vertex<VW>>, HEC: Get<usize, Item = HalfEdge<HEW>>, FC>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    #[inline]
    fn prev_vertex(&self, vertex: VertexId) -> VertexId {
        self.origin(self.prev_half_edge(self.outgoing_next_half_edge(vertex)))
    }

    #[inline]
    fn next_vertex(&self, vertex: VertexId) -> VertexId {
        self.origin(self.next_half_edge(self.outgoing_next_half_edge(vertex)))
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
