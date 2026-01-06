// SPDX-FileCopyrightText: 2026 dcel contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

mod walkers;

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

impl<VW, HEW, FW: Default, VC: Default, HEC: Default, FC: Default + maplike::Push<usize, Item = FW>>
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

impl<VW, HEW, FW: Default, VC: Default, HEC: Default, FC: Default + maplike::Push<usize, Item = FW>>
    Default for Dcel<VW, HEW, FW, VC, HEC, FC>
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
    VC: maplike::Get<usize, Item = Vertex<VW>> + maplike::Insert<usize> + maplike::Push<usize>,
    HEC: maplike::Get<usize, Item = HalfEdge<HEW>> + maplike::Insert<usize> + maplike::Push<usize>,
    FC: maplike::Get<usize, Item = Face<FW>> + maplike::Insert<usize> + maplike::Push<usize>,
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
    VC: maplike::Get<usize, Item = Vertex<VW>> + maplike::Insert<usize> + maplike::Push<usize>,
    HEC: maplike::Get<usize, Item = HalfEdge<HEW>> + maplike::Insert<usize> + maplike::Push<usize>,
    FC: maplike::Get<usize, Item = Face<FW>> + maplike::Insert<usize> + maplike::Push<usize>,
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
        let edges = self.add_unwired_polygon_edges(edge_weights, &vertexes, new_face, outer_face);

        self.wire_face_edges_vertexes(new_face, &edges);
    }

    fn add_unwired_polygon_vertexes(
        &mut self,
        vertex_weights: impl IntoIterator<Item = VW>,
    ) -> Vec<VertexId> {
        let vertexes: Vec<_> = vertex_weights
            .into_iter()
            .map(|vertex_weight| self.add_unwired_vertex(vertex_weight))
            .collect();

        vertexes
    }

    fn add_unwired_polygon_edges(
        &mut self,
        edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
        vertexes: &Vec<VertexId>,
        new_face: FaceId,
        target_face: FaceId,
    ) -> Vec<EdgeId> {
        let vertexes_circular_pair_windows = vertexes
            .iter()
            .zip(vertexes.iter().skip(1).chain(vertexes.iter().take(1)));

        let mut edges = vec![];
        let edge_weights: Vec<_> = edge_weights.into_iter().collect();

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
    FW: Clone,
    VC: maplike::Get<usize, Item = Vertex<VW>> + maplike::Insert<usize>,
    HEC: maplike::Get<usize, Item = HalfEdge<HEW>> + maplike::Insert<usize>,
    FC: maplike::Get<usize, Item = Face<FW>> + maplike::Insert<usize>,
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

impl<VW, HEW, FW, VC: maplike::Push<usize, Item = Vertex<VW>>, HEC, FC>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
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

impl<
    VW: Clone,
    HEW,
    FW,
    VC: maplike::Get<usize, Item = Vertex<VW>> + maplike::Insert<usize>,
    HEC,
    FC,
> Dcel<VW, HEW, FW, VC, HEC, FC>
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

impl<
    VW,
    HEW: Clone,
    FW,
    VC,
    HEC: maplike::Get<usize, Item = HalfEdge<HEW>> + maplike::Insert<usize>,
    FC,
> Dcel<VW, HEW, FW, VC, HEC, FC>
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

impl<
    VW,
    HEW: Clone,
    FW,
    VC,
    HEC: maplike::Insert<usize, Item = HalfEdge<HEW>> + maplike::Push<usize>,
    FC,
> Dcel<VW, HEW, FW, VC, HEC, FC>
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

impl<VW, HEW, FW: Clone, VC, HEC, FC: maplike::Get<usize, Item = Face<FW>> + maplike::Insert<usize>>
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

impl<VW, HEW, FW, VC, HEC, FC: maplike::Push<usize, Item = Face<FW>>>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    fn add_unwired_face(&mut self, weight: FW) -> FaceId {
        FaceId(self.faces.push(Face {
            incident_half_edge: None,
            weight,
        }))
    }
}

impl<VW, HEW, FW, VC, HEC: maplike::Get<usize, Item = HalfEdge<HEW>>, FC>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    fn origin(&self, half_edge: HalfEdgeId) -> VertexId {
        self.half_edges.get(&half_edge.0).unwrap().origin
    }

    fn endpoints(&self, edge: EdgeId) -> (VertexId, VertexId) {
        (self.origin(edge.0), self.origin(edge.1))
    }

    fn twin(&self, half_edge: HalfEdgeId) -> HalfEdgeId {
        self.half_edges.get(&half_edge.0).unwrap().twin
    }

    fn full_edge(&self, half_edge: HalfEdgeId) -> EdgeId {
        EdgeId(half_edge, self.twin(half_edge))
    }

    fn prev_half_edge(&self, half_edge: HalfEdgeId) -> HalfEdgeId {
        self.half_edges.get(&half_edge.0).unwrap().prev
    }

    fn next_half_edge(&self, half_edge: HalfEdgeId) -> HalfEdgeId {
        self.half_edges.get(&half_edge.0).unwrap().next
    }

    fn prev_edge(&self, edge: EdgeId) -> EdgeId {
        let next_half_edge = self.half_edges.get(&edge.0.0).unwrap().prev;
        let next_twin_half_edge = self.half_edges.get(&next_half_edge.0).unwrap().twin;

        EdgeId(next_half_edge, next_twin_half_edge)
    }

    fn next_edge(&self, edge: EdgeId) -> EdgeId {
        let next_half_edge = self.half_edges.get(&edge.0.0).unwrap().next;
        let next_twin_half_edge = self.half_edges.get(&next_half_edge.0).unwrap().twin;

        EdgeId(next_half_edge, next_twin_half_edge)
    }

    fn cw_half_edge(&self, half_edge: HalfEdgeId) -> HalfEdgeId {
        self.twin(self.prev_half_edge(half_edge))
    }

    fn ccw_half_edge(&self, half_edge: HalfEdgeId) -> HalfEdgeId {
        self.next_half_edge(self.twin(half_edge))
    }

    fn cw_edge(&self, edge: EdgeId) -> EdgeId {
        self.full_edge(self.cw_half_edge(edge.0))
    }

    fn ccw_edge(&self, edge: EdgeId) -> EdgeId {
        self.full_edge(self.ccw_half_edge(edge.0))
    }
}
