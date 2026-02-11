// SPDX-FileCopyrightText: 2026 dcel contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

mod get;
mod insert;
mod iter;
mod merge;
mod remove;
mod split;
mod track;
mod triangulate;
mod walkers;

#[cfg(test)]
pub mod test_common;

#[cfg(feature = "stable-vec")]
mod stable_vec;

#[cfg(feature = "stable-vec")]
pub use stable_vec::StableDcel;

#[cfg(feature = "rstar")]
mod rstar;

#[cfg(feature = "rstar")]
pub use rstar::{RTreedDcel, RTreedStableDcel};

use maplike::{Get, Insert, Push};

pub use walkers::{
    HalfSpokesIter, HalfSpokesReverseIter, HalfSpokesReverseWalker, HalfSpokesWalker, SpokesIter,
    SpokesReverseIter, SpokesReverseWalker, SpokesWalker,
};

/// An index pointing to a vertex.
///
/// This is just a thin newtype wrapper over [usize] for clarity and to
/// disambiguate it from other index types. Use the [id()] method to access the
/// underlying index.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct VertexId(usize);

impl VertexId {
    /// Wrap a vertex index in a newtype struct.
    #[inline]
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    /// Returns the underlying index.
    #[inline]
    pub fn id(self) -> usize {
        self.0
    }
}

/// An index pointing to a half-edge.
///
/// This is just a thin newtype wrapper over [usize] for clarity and to
/// disambiguate it from other index types. Use the [id()] method to access the
/// underlying index.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HalfEdgeId(usize);

impl HalfEdgeId {
    /// Wrap a half-edge index in a newtype struct.
    #[inline]
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    /// Returns the underlying index.
    #[inline]
    pub fn id(self) -> usize {
        self.0
    }
}

/// A unique identifier to an edge made of an ascendingly sorted pair of
/// half-edge ids.
///
/// The ascending sort is to ensure uniqueness and correctness of comparison
/// by having only one possible representation. Otherwise, for each edge there
/// would be two possible values, (x, y) and (y, x).
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EdgeId(HalfEdgeId, HalfEdgeId);

impl EdgeId {
    /// Construct a new edge id from two half-edge ids, sorting them if
    /// necessary.
    ///
    /// The order in which the two half-edge ids are passed does not matter.
    #[inline]
    pub(crate) fn new(half_edge1: HalfEdgeId, half_edge2: HalfEdgeId) -> EdgeId {
        Self(
            std::cmp::min(half_edge1, half_edge2),
            std::cmp::max(half_edge1, half_edge2),
        )
    }

    /// Returns the smaller of the two half-edge indexes.
    #[inline]
    pub fn lesser(self) -> HalfEdgeId {
        self.0
    }

    /// Returns the greater of the two half-edge indexes.
    #[inline]
    pub fn greater(self) -> HalfEdgeId {
        self.1
    }
}

/// An index pointing to a face.
///
/// This is just a thin newtype wrapper over [usize] for clarity and to
/// disambiguate it from other index types. Use the [id()] method to access the
/// underlying index.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FaceId(usize);

impl FaceId {
    /// Wrap a face index in a newtype struct.
    #[inline]
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    /// Returns the underlying index.
    #[inline]
    pub fn id(self) -> usize {
        self.0
    }
}

/// The data which describes a vertex.
#[derive(Clone, Debug)]
pub struct Vertex<VW> {
    representative: HalfEdgeId,
    weight: VW,
}

/// The data which describes a half-edge.
#[derive(Clone, Debug)]
pub struct HalfEdge<HEW> {
    source: VertexId,
    twin: HalfEdgeId,
    prev: HalfEdgeId,
    next: HalfEdgeId,
    face: FaceId,
    weight: HEW,
}

/// The data which describes a face.
#[derive(Clone, Debug)]
pub struct Face<FW> {
    representative: Option<HalfEdgeId>,
    weight: FW,
}

/// A doubly-connected edge list (DCEL).
#[derive(Clone, Debug)]
pub struct Dcel<
    VW,
    HEW = (),
    FW = (),
    VC = Vec<Vertex<VW>>,
    HEC = Vec<HalfEdge<HEW>>,
    FC = Vec<Face<FW>>,
> {
    vertices: VC,
    half_edges: HEC,
    faces: FC,
    vertex_weight_marker: std::marker::PhantomData<VW>,
    half_edge_weight_marker: std::marker::PhantomData<HEW>,
    face_weight_marker: std::marker::PhantomData<FW>,
}

impl<VW, HEW, FW: Default, VC: Default, HEC: Default, FC: Default + Push<usize, Value = Face<FW>>>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    #[inline]
    pub fn new() -> Self {
        let mut faces = FC::default();

        // Push the outermost face.
        faces.push(Face {
            representative: None,
            weight: FW::default(),
        });

        Self {
            vertices: VC::default(),
            half_edges: HEC::default(),
            faces,
            vertex_weight_marker: std::marker::PhantomData,
            half_edge_weight_marker: std::marker::PhantomData,
            face_weight_marker: std::marker::PhantomData,
        }
    }
}

impl<VW, HEW, FW: Default, VC: Default, HEC: Default, FC: Default + Push<usize, Value = Face<FW>>>
    Default for Dcel<VW, HEW, FW, VC, HEC, FC>
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<VW, HEW, FW, VC, HEC, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    #[inline]
    pub fn from_collections(vertices: VC, half_edges: HEC, faces: FC) -> Self {
        Self {
            vertices,
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
        self.vertices.into_iter().map(|id| VertexId(*id))
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
    VC: Get<usize, Value = Vertex<VW>> + Insert<usize>,
    HEC: Get<usize, Value = HalfEdge<HEW>> + Insert<usize>,
    FC: Get<usize, Value = Face<FW>> + Insert<usize>,
> Dcel<VW, HEW, FW, VC, HEC, FC>
{
    fn wire_inner_half_edge_chain(&mut self, face: FaceId, half_edges: &[HalfEdgeId]) {
        let half_edges_circular_tuple_windows = half_edges
            .iter()
            .zip(half_edges.iter().skip(1).chain(half_edges.iter().take(1)));

        for (&half_edge, &next_half_edge) in half_edges_circular_tuple_windows {
            self.link_vertex_with_half_edge(self.source(half_edge), half_edge);
            self.link_subsequent_half_edges(half_edge, next_half_edge);
            self.link_face_with_half_edge(face, half_edge);
        }
    }

    fn wire_outer_half_edge_chain_circularly(&mut self, outer_half_edges: &[HalfEdgeId]) {
        let outer_half_edges_circular_tuple_windows = outer_half_edges.iter().zip(
            outer_half_edges
                .iter()
                .skip(1)
                .chain(outer_half_edges.iter().take(1)),
        );

        for (&outer_half_edge, &next_outer_half_edge) in outer_half_edges_circular_tuple_windows {
            self.link_subsequent_half_edges(outer_half_edge, next_outer_half_edge);
        }
    }

    fn wire_outer_half_edge_chain_adjoiningly(
        &mut self,
        outer_face: FaceId,
        outer_half_edges: &[HalfEdgeId],
    ) {
        let outer_half_edges_circular_tuple_windows = outer_half_edges.iter().zip(
            outer_half_edges
                .iter()
                .skip(1)
                .chain(outer_half_edges.iter().take(1)),
        );

        for (&outer_half_edge, &next_outer_half_edge) in outer_half_edges_circular_tuple_windows {
            let is_edge_outward = self.incident_face(outer_half_edge) == outer_face;
            let is_next_edge_outward = self.incident_face(next_outer_half_edge) == outer_face;

            if is_edge_outward && is_next_edge_outward {
                self.link_subsequent_half_edges(next_outer_half_edge, outer_half_edge);
            } else if !is_edge_outward && is_next_edge_outward {
                self.link_subsequent_half_edges(next_outer_half_edge, self.turn(outer_half_edge));
            } else if is_edge_outward && !is_next_edge_outward {
                self.link_subsequent_half_edges(
                    self.twin(self.turn_back(self.twin(next_outer_half_edge))),
                    outer_half_edge,
                );
            } else {
                // Both subsequent edges are pre-existing, so they are already
                // fully wired. Nothing to do here.
            }
        }
    }
}

impl<VW, HEW, FW, VC: Push<usize, Value = Vertex<VW>>, HEC, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    fn add_unwired_vertex(&mut self, weight: VW) -> VertexId {
        VertexId(self.vertices.push(Vertex {
            // Since we do not use optionals, we cannot use `None` as the uninitialized value.
            // So instead uninitialized edge ids are edge 0.
            // Initializing `.outward_edge` to a correct value is the
            // responsibility of the caller.
            representative: HalfEdgeId(0),
            weight,
        }))
    }
}

impl<VW: Clone, HEW, FW, VC: Get<usize, Value = Vertex<VW>> + Insert<usize>, HEC, FC>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    fn link_vertex_with_half_edge(&mut self, vertex: VertexId, outgoing_half_edge: HalfEdgeId) {
        self.vertices.insert(
            vertex.id(),
            Vertex {
                representative: outgoing_half_edge,
                weight: self.vertices.get(&vertex.id()).unwrap().weight.clone(),
            },
        )
    }
}

impl<VW, HEW: Clone, FW, VC, HEC: Insert<usize, Value = HalfEdge<HEW>> + Push<usize>, FC>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    fn add_unwired_edge(
        &mut self,
        source: VertexId,
        twin_source: VertexId,
        face: FaceId,
        twin_face: FaceId,
        weight: HEW,
        twin_weight: HEW,
    ) -> (HalfEdgeId, HalfEdgeId) {
        let forward_half_edge = HalfEdgeId(self.half_edges.push(HalfEdge {
            source,
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
            source: twin_source,
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
                source,
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

        (forward_half_edge, backward_half_edge)
    }
}

impl<VW, HEW: Clone, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>> + Insert<usize>, FC>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    fn link_subsequent_half_edges(&mut self, half_edge: HalfEdgeId, next_half_edge: HalfEdgeId) {
        self.half_edges.insert(
            half_edge.id(),
            HalfEdge {
                next: next_half_edge,
                ..self.half_edges.get(&half_edge.id()).unwrap().clone()
            },
        );
        self.half_edges.insert(
            next_half_edge.id(),
            HalfEdge {
                prev: half_edge,
                ..self.half_edges.get(&next_half_edge.id()).unwrap().clone()
            },
        );
    }
}

impl<VW, HEW, FW, VC, HEC, FC: Push<usize, Value = Face<FW>>> Dcel<VW, HEW, FW, VC, HEC, FC> {
    fn add_unwired_face(&mut self, weight: FW) -> FaceId {
        FaceId(self.faces.push(Face {
            representative: None,
            weight,
        }))
    }
}

impl<
    VW,
    HEW: Clone,
    FW: Clone,
    VC,
    HEC: Get<usize, Value = HalfEdge<HEW>> + Insert<usize>,
    FC: Get<usize, Value = Face<FW>> + Insert<usize>,
> Dcel<VW, HEW, FW, VC, HEC, FC>
{
    fn link_face_with_half_edge(&mut self, face: FaceId, half_edge: HalfEdgeId) {
        self.half_edges.insert(
            half_edge.id(),
            HalfEdge {
                face,
                ..self.half_edges.get(&half_edge.id()).unwrap().clone()
            },
        );
        self.faces.insert(
            face.id(),
            Face {
                representative: Some(half_edge),
                weight: self.faces.get(&face.id()).unwrap().weight.clone(),
            },
        );
    }
}
