// SPDX-FileCopyrightText: 2026 dcel contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{Dcel, EdgeId, Face, FaceId, HalfEdge, HalfEdgeId};

pub struct FaceHalfEdgesWalker {
    initial_half_edge: HalfEdgeId,
    curr_half_edge: HalfEdgeId,
}

impl FaceHalfEdgesWalker {
    pub fn next<VW, HEW, FW, VC, HEC: maplike::Get<usize, Item = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<HalfEdgeId> {
        let next_half_edge = dcel.next_half_edge(self.curr_half_edge);

        (next_half_edge != self.initial_half_edge)
            .then(|| std::mem::replace(&mut self.curr_half_edge, next_half_edge))
    }

    pub fn iter<'a, VW, HEW, FW, VC, HEC, FC>(
        self,
        dcel: &'a Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> FaceHalfEdgesIter<'a, VW, HEW, FW, VC, HEC, FC> {
        FaceHalfEdgesIter { walker: self, dcel }
    }
}

pub struct FaceHalfEdgesIter<'a, VW, HEW, FW, VC, HEC, FC> {
    walker: FaceHalfEdgesWalker,
    dcel: &'a Dcel<VW, HEW, FW, VC, HEC, FC>,
}

impl<'a, VW, HEW, FW, VC, HEC: maplike::Get<usize, Item = HalfEdge<HEW>>, FC> Iterator
    for FaceHalfEdgesIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = HalfEdgeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.dcel)
    }
}

impl<VW, HEW, FW, VC, HEC, FC: maplike::Get<usize, Item = Face<FW>>>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    pub(crate) fn face_half_edges(&self, face: FaceId) -> FaceHalfEdgesWalker {
        let initial_half_edge = self.faces.get(&face.0).unwrap().incident_half_edge.unwrap();

        FaceHalfEdgesWalker {
            initial_half_edge,
            curr_half_edge: initial_half_edge,
        }
    }
}

pub struct FaceEdgesWalker {
    initial_edge: EdgeId,
    curr_edge: EdgeId,
}

impl FaceEdgesWalker {
    pub fn next<VW, HEW, FW, VC, HEC: maplike::Get<usize, Item = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<EdgeId> {
        let next_edge = dcel.next_edge(self.curr_edge);

        (next_edge != self.initial_edge).then(|| std::mem::replace(&mut self.curr_edge, next_edge))
    }

    pub fn iter<'a, VW, HEW, FW, VC, HEC, FC>(
        self,
        dcel: &'a Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> FaceEdgesIter<'a, VW, HEW, FW, VC, HEC, FC> {
        FaceEdgesIter { walker: self, dcel }
    }
}

pub struct FaceEdgesIter<'a, VW, HEW, FW, VC, HEC, FC> {
    walker: FaceEdgesWalker,
    dcel: &'a Dcel<VW, HEW, FW, VC, HEC, FC>,
}

impl<'a, VW, HEW, FW, VC, HEC: maplike::Get<usize, Item = HalfEdge<HEW>>, FC> Iterator
    for FaceEdgesIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = EdgeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.dcel)
    }
}

impl<
    VW,
    HEW,
    FW,
    VC,
    HEC: maplike::Get<usize, Item = HalfEdge<HEW>>,
    FC: maplike::Get<usize, Item = Face<FW>>,
> Dcel<VW, HEW, FW, VC, HEC, FC>
{
    pub(crate) fn face_edges(&self, face: FaceId) -> FaceEdgesWalker {
        let initial_edge =
            self.full_edge(self.faces.get(&face.0).unwrap().incident_half_edge.unwrap());

        FaceEdgesWalker {
            initial_edge,
            curr_edge: initial_edge,
        }
    }
}

pub struct CwHalfEdgesWalker {
    initial_half_edge: HalfEdgeId,
    curr_half_edge: HalfEdgeId,
}

impl CwHalfEdgesWalker {
    pub fn next<VW, HEW, FW, VC, HEC: maplike::Get<usize, Item = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<HalfEdgeId> {
        let next_half_edge = dcel.cw_half_edge(self.curr_half_edge);

        (next_half_edge != self.initial_half_edge)
            .then(|| std::mem::replace(&mut self.curr_half_edge, next_half_edge))
    }

    pub fn iter<'a, VW, HEW, FW, VC, HEC, FC>(
        self,
        dcel: &'a Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> CwHalfEdgesIter<'a, VW, HEW, FW, VC, HEC, FC> {
        CwHalfEdgesIter { walker: self, dcel }
    }
}

pub struct CwHalfEdgesIter<'a, VW, HEW, FW, VC, HEC, FC> {
    walker: CwHalfEdgesWalker,
    dcel: &'a Dcel<VW, HEW, FW, VC, HEC, FC>,
}

impl<'a, VW, HEW, FW, VC, HEC: maplike::Get<usize, Item = HalfEdge<HEW>>, FC> Iterator
    for CwHalfEdgesIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = HalfEdgeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.dcel)
    }
}

impl<VW, HEW, FW, VC, HEC, FC: maplike::Get<usize, Item = Face<FW>>>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    pub(crate) fn cw_half_edges(&self, face: FaceId) -> CwHalfEdgesWalker {
        let initial_half_edge = self.faces.get(&face.0).unwrap().incident_half_edge.unwrap();

        CwHalfEdgesWalker {
            initial_half_edge,
            curr_half_edge: initial_half_edge,
        }
    }
}

pub struct CcwHalfEdgesWalker {
    initial_half_edge: HalfEdgeId,
    curr_half_edge: HalfEdgeId,
}

impl CcwHalfEdgesWalker {
    pub fn next<VW, HEW, FW, VC, HEC: maplike::Get<usize, Item = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<HalfEdgeId> {
        let next_half_edge = dcel.ccw_half_edge(self.curr_half_edge);

        (next_half_edge != self.initial_half_edge)
            .then(|| std::mem::replace(&mut self.curr_half_edge, next_half_edge))
    }

    pub fn iter<'a, VW, HEW, FW, VC, HEC, FC>(
        self,
        dcel: &'a Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> CcwHalfEdgesIter<'a, VW, HEW, FW, VC, HEC, FC> {
        CcwHalfEdgesIter { walker: self, dcel }
    }
}

pub struct CcwHalfEdgesIter<'a, VW, HEW, FW, VC, HEC, FC> {
    walker: CcwHalfEdgesWalker,
    dcel: &'a Dcel<VW, HEW, FW, VC, HEC, FC>,
}

impl<'a, VW, HEW, FW, VC, HEC: maplike::Get<usize, Item = HalfEdge<HEW>>, FC> Iterator
    for CcwHalfEdgesIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = HalfEdgeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.dcel)
    }
}

impl<VW, HEW, FW, VC, HEC, FC: maplike::Get<usize, Item = Face<FW>>>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    pub(crate) fn ccw_half_edges(&self, face: FaceId) -> CcwHalfEdgesWalker {
        let initial_half_edge = self.faces.get(&face.0).unwrap().incident_half_edge.unwrap();

        CcwHalfEdgesWalker {
            initial_half_edge,
            curr_half_edge: initial_half_edge,
        }
    }
}

pub struct CwEdgesWalker {
    initial_edge: EdgeId,
    curr_edge: EdgeId,
}

impl CwEdgesWalker {
    pub fn next<VW, HEW, FW, VC, HEC: maplike::Get<usize, Item = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<EdgeId> {
        let next_edge = dcel.cw_edge(self.curr_edge);

        (next_edge != self.initial_edge).then(|| std::mem::replace(&mut self.curr_edge, next_edge))
    }

    pub fn iter<'a, VW, HEW, FW, VC, HEC, FC>(
        self,
        dcel: &'a Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> CwEdgesIter<'a, VW, HEW, FW, VC, HEC, FC> {
        CwEdgesIter { walker: self, dcel }
    }
}

pub struct CwEdgesIter<'a, VW, HEW, FW, VC, HEC, FC> {
    walker: CwEdgesWalker,
    dcel: &'a Dcel<VW, HEW, FW, VC, HEC, FC>,
}

impl<'a, VW, HEW, FW, VC, HEC: maplike::Get<usize, Item = HalfEdge<HEW>>, FC> Iterator
    for CwEdgesIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = EdgeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.dcel)
    }
}

impl<
    VW,
    HEW,
    FW,
    VC,
    HEC: maplike::Get<usize, Item = HalfEdge<HEW>>,
    FC: maplike::Get<usize, Item = Face<FW>>,
> Dcel<VW, HEW, FW, VC, HEC, FC>
{
    pub(crate) fn cw_edges(&self, face: FaceId) -> CwEdgesWalker {
        let initial_edge =
            self.full_edge(self.faces.get(&face.0).unwrap().incident_half_edge.unwrap());

        CwEdgesWalker {
            initial_edge,
            curr_edge: initial_edge,
        }
    }
}

pub struct CcwEdgesWalker {
    initial_edge: EdgeId,
    curr_edge: EdgeId,
}

impl CcwEdgesWalker {
    pub fn next<VW, HEW, FW, VC, HEC: maplike::Get<usize, Item = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<EdgeId> {
        let next_edge = dcel.ccw_edge(self.curr_edge);

        (next_edge != self.initial_edge).then(|| std::mem::replace(&mut self.curr_edge, next_edge))
    }

    pub fn iter<'a, VW, HEW, FW, VC, HEC, FC>(
        self,
        dcel: &'a Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> CcwEdgesIter<'a, VW, HEW, FW, VC, HEC, FC> {
        CcwEdgesIter { walker: self, dcel }
    }
}

pub struct CcwEdgesIter<'a, VW, HEW, FW, VC, HEC, FC> {
    walker: CcwEdgesWalker,
    dcel: &'a Dcel<VW, HEW, FW, VC, HEC, FC>,
}

impl<'a, VW, HEW, FW, VC, HEC: maplike::Get<usize, Item = HalfEdge<HEW>>, FC> Iterator
    for CcwEdgesIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = EdgeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.dcel)
    }
}

impl<
    VW,
    HEW,
    FW,
    VC,
    HEC: maplike::Get<usize, Item = HalfEdge<HEW>>,
    FC: maplike::Get<usize, Item = Face<FW>>,
> Dcel<VW, HEW, FW, VC, HEC, FC>
{
    pub(crate) fn ccw_edges(&self, face: FaceId) -> CcwEdgesWalker {
        let initial_edge =
            self.full_edge(self.faces.get(&face.0).unwrap().incident_half_edge.unwrap());

        CcwEdgesWalker {
            initial_edge,
            curr_edge: initial_edge,
        }
    }
}
