// SPDX-FileCopyrightText: 2026 dcel contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

mod get;
mod insert;
mod iter;
mod merge;
mod split;
mod track;
mod triangulate;
mod walkers;

use maplike::{Get, Insert, Push, Remove};

pub use walkers::{
    CcwEdgesIter, CcwEdgesWalker, CcwHalfEdgesIter, CcwHalfEdgesWalker, CwEdgesIter, CwEdgesWalker,
    CwHalfEdgesIter, CwHalfEdgesWalker, FaceEdgesIter, FaceEdgesWalker, FaceHalfEdgesIter,
    FaceHalfEdgesWalker,
};

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
    pub fn from_collections(vertexes: VC, half_edges: HEC, faces: FC) -> Self {
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

impl<VW, HEW, FW, VC, HEC, FC> Dcel<VW, HEW, FW, VC, HEC, FC>
where
    for<'a> &'a VC: IntoIterator<Item = &'a usize>,
{
    #[inline]
    pub fn vertex_ids(&self) -> impl Iterator<Item = VertexId> {
        self.vertexes.into_iter().map(|id| VertexId(*id))
    }
}

impl<VW, HEW, FW, VC, HEC, FC> Dcel<VW, HEW, FW, VC, HEC, FC>
where
    for<'a> &'a HEC: IntoIterator<Item = &'a usize>,
{
    #[inline]
    pub fn half_edge_ids(&self) -> impl Iterator<Item = HalfEdgeId> {
        self.half_edges.into_iter().map(|id| HalfEdgeId(*id))
    }
}

impl<VW, HEW, FW, VC, HEC, FC> Dcel<VW, HEW, FW, VC, HEC, FC>
where
    for<'a> &'a FC: IntoIterator<Item = &'a usize>,
{
    #[inline]
    pub fn face_ids(&self) -> impl Iterator<Item = FaceId> {
        self.faces.into_iter().map(|id| FaceId(*id))
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
    fn wire_inner_half_edge_chain(&mut self, face: FaceId, edges: &[EdgeId]) {
        self.wire_face(face, edges[0].forward());

        let edges_circular_tuple_windows = edges
            .iter()
            .zip(edges.iter().skip(1).chain(edges.iter().take(1)));

        for (edge, next_edge) in edges_circular_tuple_windows {
            self.link_subsequent_half_edges(edge.forward(), next_edge.forward());
            self.link_vertex_to_edge(self.origin(edge.forward()), edge.forward());
        }
    }

    fn wire_outer_half_edge_chain_circularly(&mut self, edges: &[EdgeId]) {
        let edges_circular_tuple_windows = edges
            .iter()
            .zip(edges.iter().skip(1).chain(edges.iter().take(1)));

        for (edge, next_edge) in edges_circular_tuple_windows {
            self.link_subsequent_half_edges(edge.backward(), next_edge.backward());
        }
    }

    fn wire_outer_half_edge_chain_adjoiningly(&mut self, outer_face: FaceId, edges: &[EdgeId]) {
        let edges_circular_tuple_windows = edges
            .iter()
            .zip(edges.iter().skip(1).chain(edges.iter().take(1)));

        for (edge, next_edge) in edges_circular_tuple_windows {
            let is_edge_outward = self.face_behind(edge.forward()) == outer_face;
            let is_next_edge_outward = self.face_behind(next_edge.forward()) == outer_face;

            if is_edge_outward && is_next_edge_outward {
                self.link_subsequent_half_edges(next_edge.backward(), edge.backward());
            } else if !is_edge_outward && is_next_edge_outward {
                self.link_subsequent_half_edges(
                    next_edge.backward(),
                    self.turn_half_edge(edge.backward()),
                );
            } else if is_edge_outward && !is_next_edge_outward {
                self.link_subsequent_half_edges(
                    self.twin(self.turn_back_half_edge(next_edge.forward())),
                    edge.backward(),
                );
            } else {
                // Both subsequent edges are pre-existing, so they are already
                // fully wired. Nothing to do here.
            }
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
    fn link_vertex_to_edge(&mut self, vertex: VertexId, outgoing_half_edge: HalfEdgeId) {
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
    fn link_subsequent_half_edges(&mut self, half_edge: HalfEdgeId, next_half_edge: HalfEdgeId) {
        // Link the forward (inner) half-edge of `edge` with the forward
        // half-edge of `next_edge`.
        // half_edge.next := next_half_edge
        self.half_edges.insert(
            half_edge.id(),
            HalfEdge {
                next: next_half_edge,
                ..self.half_edges.get(&half_edge.id()).unwrap().clone()
            },
        );
        // next_half_edge.prev := half_edge
        self.half_edges.insert(
            next_half_edge.id(),
            HalfEdge {
                prev: half_edge,
                ..self.half_edges.get(&next_half_edge.id()).unwrap().clone()
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
