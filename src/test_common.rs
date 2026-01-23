// SPDX-FileCopyrightText: 2026 dcel contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::Dcel;

pub const HEX_MESH_3X3: [[[(i32, i32); 6]; 3]; 3] = [
    [
        [
            (87, -50),
            (0, -100),
            (-87, -50),
            (-87, 50),
            (0, 100),
            (87, 50),
        ],
        [
            (260, -50),
            (173, -100),
            (87, -50),
            (87, 50),
            (173, 100),
            (260, 50),
        ],
        [
            (433, -50),
            (346, -100),
            (260, -50),
            (260, 50),
            (346, 100),
            (433, 50),
        ],
    ],
    [
        [
            (173, 100),
            (87, 50),
            (0, 100),
            (0, 200),
            (87, 250),
            (173, 200),
        ],
        [
            (346, 100),
            (260, 50),
            (173, 100),
            (173, 200),
            (260, 250),
            (346, 200),
        ],
        [
            (520, 100),
            (433, 50),
            (346, 100),
            (346, 200),
            (433, 250),
            (520, 200),
        ],
    ],
    [
        [
            (87, 250),
            (0, 200),
            (-87, 250),
            (-87, 350),
            (0, 400),
            (87, 350),
        ],
        [
            (260, 250),
            (173, 200),
            (87, 250),
            (87, 350),
            (173, 400),
            (260, 350),
        ],
        [
            (433, 250),
            (346, 200),
            (260, 250),
            (260, 350),
            (346, 400),
            (433, 350),
        ],
    ],
];

pub fn init_dcel_with_3x3_hex_mesh() -> Dcel<(i32, i32)> {
    let mesh: Vec<Vec<(i32, i32)>> = HEX_MESH_3X3
        .iter()
        .flat_map(|row| row.iter())
        .map(|face| face.iter().copied().collect())
        .collect();
    let mut dcel: Dcel<(i32, i32)> = Dcel::new();
    dcel.insert_mesh(mesh);

    dcel
}

#[macro_export]
macro_rules! assert_vertex_rim {
    ($dcel:expr, $id:expr, $count:expr) => {{
        use $crate::{EdgeId, HalfEdgeId, VertexId};

        let vertex_rim_vertexes: Vec<VertexId> =
            $dcel.vertex_rim_vertexes(VertexId::new($id)).collect();
        assert_eq!(vertex_rim_vertexes.len(), $count);

        let vertex_rim_vertexes_reverse: Vec<VertexId> = $dcel
            .vertex_rim_vertexes_reverse(VertexId::new($id))
            .collect();
        assert_eq!(
            vertex_rim_vertexes,
            vertex_rim_vertexes_reverse
                .into_iter()
                .rev()
                .collect::<Vec<VertexId>>()
        );

        let vertex_rim_half_edges: Vec<HalfEdgeId> =
            $dcel.vertex_rim_half_edges(VertexId::new($id)).collect();
        assert_eq!(vertex_rim_half_edges.len(), $count);

        let vertex_rim_half_edges_reverse: Vec<HalfEdgeId> = $dcel
            .vertex_rim_half_edges_reverse(VertexId::new($id))
            .collect();
        assert_eq!(
            vertex_rim_half_edges,
            vertex_rim_half_edges_reverse
                .into_iter()
                .rev()
                .collect::<Vec<HalfEdgeId>>()
        );

        let vertex_rim_edges: Vec<EdgeId> =
            $dcel.vertex_rim_edges(VertexId::new($id)).collect();
        assert_eq!(vertex_rim_edges.len(), $count);

        let vertex_rim_edges_reverse: Vec<EdgeId> = $dcel
            .vertex_rim_edges_reverse(VertexId::new($id))
            .collect();
        assert_eq!(
            vertex_rim_edges,
            vertex_rim_edges_reverse
                .into_iter()
                .rev()
                .collect::<Vec<EdgeId>>()
        );
    }};
}

#[macro_export]
macro_rules! assert_vertex_spokes_interspokes {
    ($dcel:expr, $id:expr, $count:expr) => {{
        use $crate::{EdgeId, FaceId, HalfEdgeId, VertexId};

        let vertex_half_spokes: Vec<HalfEdgeId> =
            $dcel.vertex_half_spokes(VertexId::new($id)).collect();
        assert_eq!(vertex_half_spokes.len(), $count);

        let vertex_half_spokes_reverse: Vec<HalfEdgeId> = $dcel
            .vertex_half_spokes_reverse(VertexId::new($id))
            .collect();
        assert_eq!(
            vertex_half_spokes.first(),
            vertex_half_spokes_reverse.first()
        );
        assert_eq!(
            vertex_half_spokes
                .into_iter()
                .skip(1)
                .collect::<Vec<HalfEdgeId>>(),
            vertex_half_spokes_reverse
                .into_iter()
                .skip(1)
                .rev()
                .collect::<Vec<HalfEdgeId>>()
        );

        let vertex_spokes: Vec<EdgeId> = $dcel.vertex_spokes(VertexId::new($id)).collect();
        assert_eq!(vertex_spokes.len(), $count);

        let vertex_spokes_reverse: Vec<EdgeId> = $dcel
            .vertex_spokes_reverse(VertexId::new($id))
            .collect();
        assert_eq!(vertex_spokes.first(), vertex_spokes_reverse.first());
        assert_eq!(
            vertex_spokes.into_iter().skip(1).collect::<Vec<EdgeId>>(),
            vertex_spokes_reverse
                .into_iter()
                .skip(1)
                .rev()
                .collect::<Vec<EdgeId>>()
        );

        let vertex_interspokes: Vec<FaceId> =
            $dcel.vertex_interspokes(VertexId::new($id)).collect();
        assert_eq!(vertex_interspokes.len(), $count);

        let vertex_interspokes_reverse: Vec<FaceId> = $dcel
            .vertex_interspokes_reverse(VertexId::new($id))
            .collect();
        assert_eq!(
            vertex_interspokes.first(),
            vertex_interspokes_reverse.first()
        );
        assert_eq!(
            vertex_interspokes
                .into_iter()
                .skip(1)
                .collect::<Vec<FaceId>>(),
            vertex_interspokes_reverse
                .into_iter()
                .skip(1)
                .rev()
                .collect::<Vec<FaceId>>()
        );
    }};
}

#[macro_export]
macro_rules! assert_face_boundary {
    ($dcel:expr, $id:expr, $count:expr) => {{
        use $crate::{EdgeId, FaceId, HalfEdgeId, VertexId};

        let face_vertexes: Vec<VertexId> = $dcel.face_vertexes(FaceId::new($id)).collect();
        assert_eq!(face_vertexes.len(), $count);

        let face_vertexes_reverse: Vec<VertexId> = $dcel
            .face_vertexes_reverse(FaceId::new($id))
            .collect();
        assert_eq!(face_vertexes.first(), face_vertexes_reverse.first());
        assert_eq!(
            face_vertexes.into_iter().skip(1).collect::<Vec<VertexId>>(),
            face_vertexes_reverse
                .into_iter()
                .skip(1)
                .rev()
                .collect::<Vec<VertexId>>()
        );

        let face_half_edges: Vec<HalfEdgeId> =
            $dcel.face_half_edges(FaceId::new($id)).collect();
        assert_eq!(face_half_edges.len(), $count);

        let face_half_edges_reverse: Vec<HalfEdgeId> = $dcel
            .face_half_edges_reverse(FaceId::new($id))
            .collect();
        assert_eq!(face_half_edges.first(), face_half_edges_reverse.first());
        assert_eq!(
            face_half_edges
                .into_iter()
                .skip(1)
                .collect::<Vec<HalfEdgeId>>(),
            face_half_edges_reverse
                .into_iter()
                .skip(1)
                .rev()
                .collect::<Vec<HalfEdgeId>>()
        );

        let face_edges: Vec<EdgeId> = $dcel.face_edges(FaceId::new($id)).collect();
        assert_eq!(face_edges.len(), $count);

        let face_edges_reverse: Vec<EdgeId> =
            $dcel.face_edges_reverse(FaceId::new($id)).collect();
        assert_eq!(face_edges.first(), face_edges_reverse.first());
        assert_eq!(
            face_edges.into_iter().skip(1).collect::<Vec<EdgeId>>(),
            face_edges_reverse
                .into_iter()
                .skip(1)
                .rev()
                .collect::<Vec<EdgeId>>()
        );
    }};
}
