//! Shared exact predicate helpers for triangulation topology.
//!
//! These wrappers keep algorithm modules from depending directly on predicate
//! provenance details. Exact code only consumes certified decisions from
//! `hyperlimit`, while enum growth and exact overlap refinements stay
//! centralized in the predicate crate.

#![allow(dead_code)]

use crate::error::{Error, Result};
use crate::types::{Point2, Real, Sign, TriangleLocation};
use hyperlimit::PredicateOutcome;
use std::cmp::Ordering;

pub use hyperlimit::SegmentIntersection;

/// Decide the orientation of three points.
pub(crate) fn orient2d(a: &Point2, b: &Point2, c: &Point2) -> Result<Sign> {
    decide_hyperlimit_sign(
        hyperlimit::orient2d(
            &predicate_point(a),
            &predicate_point(b),
            &predicate_point(c),
        ),
        "orient2d",
    )
}

/// Decide the in-circle relation of four points.
pub(crate) fn incircle2d(a: &Point2, b: &Point2, c: &Point2, d: &Point2) -> Result<Sign> {
    decide_hyperlimit_sign(
        hyperlimit::incircle2d(
            &predicate_point(a),
            &predicate_point(b),
            &predicate_point(c),
            &predicate_point(d),
        ),
        "incircle2d",
    )
}

/// Compare two exact scalar values.
pub(crate) fn compare_reals(left: &Real, right: &Real) -> Result<Ordering> {
    match hyperlimit::compare_reals(left, right) {
        PredicateOutcome::Decided { value, .. } => Ok(value),
        PredicateOutcome::Unknown { .. } => Err(Error::PredicateUndecided {
            predicate: "compare_reals",
        }),
    }
}

/// Return the exact midpoint between two points.
pub(crate) fn midpoint(left: &Point2, right: &Point2) -> Result<Point2> {
    let two = Real::from(2);
    let x = &left.x + &right.x;
    let y = &left.y + &right.y;
    Ok(Point2::new(divide(&x, &two)?, divide(&y, &two)?))
}

/// Classify a point relative to a triangle.
pub(crate) fn classify_point_triangle(
    a: &Point2,
    b: &Point2,
    c: &Point2,
    point: &Point2,
) -> Result<TriangleLocation> {
    match hyperlimit::classify_point_triangle(
        &predicate_point(a),
        &predicate_point(b),
        &predicate_point(c),
        &predicate_point(point),
    ) {
        PredicateOutcome::Decided { value, .. } => Ok(map_triangle_location(value)),
        PredicateOutcome::Unknown { .. } => Err(Error::PredicateUndecided {
            predicate: "classify_point_triangle",
        }),
    }
}

/// Decide whether `point` lies inside or on the boundary of `abc`.
pub(crate) fn point_in_or_on_triangle(
    a: &Point2,
    b: &Point2,
    c: &Point2,
    point: &Point2,
) -> Result<bool> {
    Ok(matches!(
        classify_point_triangle(a, b, c, point)?,
        TriangleLocation::Inside | TriangleLocation::OnEdge | TriangleLocation::OnVertex
    ))
}

/// Decide whether a point lies on a closed segment.
pub(crate) fn point_on_segment(a: &Point2, b: &Point2, point: &Point2) -> Result<bool> {
    // This is a direct boundary predicate, not triangulation topology. Route it
    // through hyperlimit's segment classifier so the exact interval and
    // degenerate-segment rules have a single owner.
    decide_hyperlimit_bool(
        hyperlimit::point_on_segment(
            &predicate_point(a),
            &predicate_point(b),
            &predicate_point(point),
        ),
        "point_on_segment",
    )
}

/// Decide whether a point is inside a closed indexed ring by even-odd parity.
pub(crate) fn point_in_ring_even_odd(
    vertices: &[Point2],
    ring: &[usize],
    point: &Point2,
) -> Result<bool> {
    // Hyperlimit owns the exact crossing-number and boundary predicates;
    // hypertri supplies only index topology.
    let predicate_vertices: Vec<_> = vertices.iter().map(predicate_point).collect();
    decide_hyperlimit_bool(
        hyperlimit::point_in_indexed_ring_even_odd(
            &predicate_vertices,
            ring,
            &predicate_point(point),
        ),
        "point_in_indexed_ring_even_odd",
    )
}

/// Classify two closed line segments.
pub(crate) fn segment_intersection(
    a: &Point2,
    b: &Point2,
    c: &Point2,
    d: &Point2,
) -> Result<SegmentIntersection> {
    // Segment intersection is the canonical topological predicate for CDT edge
    // recovery and ear visibility. Keep the four-orientation classifier in
    // hyperlimit, where determinant and interval decisions are implemented.
    // Hypertri consumes only the decided combinatorial relation.
    match hyperlimit::classify_segment_intersection(
        &predicate_point(a),
        &predicate_point(b),
        &predicate_point(c),
        &predicate_point(d),
    ) {
        PredicateOutcome::Decided { value, .. } => Ok(value),
        PredicateOutcome::Unknown { .. } => Err(Error::PredicateUndecided {
            predicate: "segment_intersection",
        }),
    }
}

fn predicate_point(point: &Point2) -> hyperlimit::Point2 {
    hyperlimit::Point2::new(point.x.clone(), point.y.clone())
}

fn divide(left: &Real, right: &Real) -> Result<Real> {
    (left / right).map_err(|_| Error::InvalidInput {
        reason: "Real division failed",
    })
}

fn decide_hyperlimit_sign(
    outcome: PredicateOutcome<hyperlimit::Sign>,
    predicate: &'static str,
) -> Result<Sign> {
    match outcome {
        PredicateOutcome::Decided { value, .. } => Ok(map_sign(value)),
        PredicateOutcome::Unknown { .. } => Err(Error::PredicateUndecided { predicate }),
    }
}

const fn map_sign(sign: hyperlimit::Sign) -> Sign {
    match sign {
        hyperlimit::Sign::Negative => Sign::Negative,
        hyperlimit::Sign::Zero => Sign::Zero,
        hyperlimit::Sign::Positive => Sign::Positive,
    }
}

const fn map_triangle_location(location: hyperlimit::TriangleLocation) -> TriangleLocation {
    match location {
        hyperlimit::TriangleLocation::Degenerate => TriangleLocation::Degenerate,
        hyperlimit::TriangleLocation::Inside => TriangleLocation::Inside,
        hyperlimit::TriangleLocation::OnEdge => TriangleLocation::OnEdge,
        hyperlimit::TriangleLocation::OnVertex => TriangleLocation::OnVertex,
        hyperlimit::TriangleLocation::Outside => TriangleLocation::Outside,
    }
}

fn decide_hyperlimit_bool(
    outcome: PredicateOutcome<bool>,
    predicate: &'static str,
) -> Result<bool> {
    match outcome {
        PredicateOutcome::Decided { value, .. } => Ok(value),
        PredicateOutcome::Unknown { .. } => Err(Error::PredicateUndecided { predicate }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(x: i64, y: i64) -> Point2 {
        Point2::new(Real::from(x), Real::from(y))
    }

    #[test]
    fn segment_intersection_delegates_identical_segments_to_hyperlimit() {
        assert_eq!(
            segment_intersection(&p(0, 0), &p(4, 0), &p(4, 0), &p(0, 0)).unwrap(),
            SegmentIntersection::Identical
        );
    }

    #[test]
    fn point_on_segment_delegates_degenerate_segments_to_hyperlimit() {
        assert!(point_on_segment(&p(2, 3), &p(2, 3), &p(2, 3)).unwrap());
        assert!(!point_on_segment(&p(2, 3), &p(2, 3), &p(2, 4)).unwrap());
    }

    #[test]
    fn orientation_and_incircle_delegate_to_hyperlimit() {
        let a = p(0, 0);
        let b = p(2, 0);
        let c = p(0, 2);

        assert_eq!(orient2d(&a, &b, &c).unwrap(), Sign::Positive);
        assert_eq!(incircle2d(&a, &b, &c, &p(1, 1)).unwrap(), Sign::Positive);
        assert_eq!(incircle2d(&a, &b, &c, &p(2, 2)).unwrap(), Sign::Zero);
        assert_eq!(incircle2d(&a, &b, &c, &p(3, 3)).unwrap(), Sign::Negative);
    }
}
