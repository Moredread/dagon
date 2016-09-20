#![cfg_attr(feature="herbie-lint", feature(plugin))]
#![cfg_attr(feature="herbie-lint", plugin(herbie_lint))]

#![feature(plugin)]
#![plugin(clippy)]

#![feature(test)]

extern crate rand;
extern crate nalgebra as na;
extern crate num;
extern crate rayon;
extern crate acacia;
extern crate test;
extern crate dagon;

use dagon::*;

use acacia::{AssociatedData, DataQuery, Node, Position, Tree};
use acacia::partition::Ncube;
use na::{FloatPoint, Norm, Origin, Point3, Vector3, zero};
use num::Zero;
use rand::distributions::{IndependentSample, Range};
use rayon::prelude::*;
use rand::thread_rng;
use test::Bencher;

const N: u64 = 2000;

#[bench]
fn bench_tree_construction(b: &mut Bencher) {
    let particles = make_random_initial_conditions(N);
    let origin: Point = Origin::origin();
    b.iter(|| {
        // Construct the tree, taken from acacia example
        let tree = Tree::new(// Pass in an iterator over the objects to store in the tree
                             // In this case we pass in &PointMass, which implements Positionable.
                             particles.iter(),
                             // Shape of the root node
                             Ncube::new(origin, 2.0),
                             // The value for the associated data of empty nodes. Here, we associate
                             // a center of mass and a total mass to each node.
                             (origin, 0.0),
                             // This closure associates data to a leaf node from its object.
                             &|obj| (obj.position, obj.mass),
                             // This combines two pieces of associated data and thus prescribes how
                             // branch nodes on higher levels get their associated data.
                             &|&(com1, m1), &(com2, m2)| if m1 + m2 > 0.0 {
                                 (origin + (com1.to_vector() * m1 + com2.to_vector() * m2) / (m1 + m2), m1 + m2)
                             } else {
                                 (origin, 0.0)
                             })
            .expect("Couldn't construct tree");
    })
}

#[bench]
fn bench_tree_max_weight(b: &mut Bencher) {
    let particles = make_random_initial_conditions(N);
    let origin: Point = Origin::origin();
        // Construct the tree, taken from acacia example
        let tree = Tree::new(// Pass in an iterator over the objects to store in the tree
                             // In this case we pass in &PointMass, which implements Positionable.
                             particles.iter(),
                             // Shape of the root node
                             Ncube::new(origin, 2.0),
                             // The value for the associated data of empty nodes. Here, we associate
                             // a center of mass and a total mass to each node.
                             (origin, 0.0),
                             // This closure associates data to a leaf node from its object.
                             &|obj| (obj.position, obj.mass),
                             // This combines two pieces of associated data and thus prescribes how
                             // branch nodes on higher levels get their associated data.
                             &|&(com1, m1), &(com2, m2)| if m1 + m2 > 0.0 {
                                 (origin + (com1.to_vector() * m1 + com2.to_vector() * m2) / (m1 + m2), m1 + m2)
                             } else {
                                 (origin, 0.0)
                             })
            .expect("Couldn't construct tree");

    b.iter(|| {


        let mut forces = Vec::<Vector>::new();
        particles
                .par_iter()
                .weight_max()
                .map(|p| forces_from_tree(*p, &tree))
                .collect_into(&mut forces);
    })
}

#[bench]
fn bench_direct_max_weight(b: &mut Bencher) {
    let particles = make_random_initial_conditions(N);
    b.iter(|| {
        let mut forces = Vec::<Vector>::new();
        particles
                .par_iter()
                .weight_max()
                .map(|p| forces_by_direct_summation(*p, particles.iter()))
                .collect_into(&mut forces);
    })
}

#[bench]
fn bench_tree(b: &mut Bencher) {
    let particles = make_random_initial_conditions(N);
    let origin: Point = Origin::origin();
        // Construct the tree, taken from acacia example
        let tree = Tree::new(// Pass in an iterator over the objects to store in the tree
                             // In this case we pass in &PointMass, which implements Positionable.
                             particles.iter(),
                             // Shape of the root node
                             Ncube::new(origin, 2.0),
                             // The value for the associated data of empty nodes. Here, we associate
                             // a center of mass and a total mass to each node.
                             (origin, 0.0),
                             // This closure associates data to a leaf node from its object.
                             &|obj| (obj.position, obj.mass),
                             // This combines two pieces of associated data and thus prescribes how
                             // branch nodes on higher levels get their associated data.
                             &|&(com1, m1), &(com2, m2)| if m1 + m2 > 0.0 {
                                 (origin + (com1.to_vector() * m1 + com2.to_vector() * m2) / (m1 + m2), m1 + m2)
                             } else {
                                 (origin, 0.0)
                             })
            .expect("Couldn't construct tree");

    b.iter(|| {
        let mut forces = Vec::<Vector>::new();
        particles
                .par_iter()
                .map(|p| forces_from_tree(*p, &tree))
                .collect_into(&mut forces);
    })
}

#[bench]
fn bench_direct(b: &mut Bencher) {
    let particles = make_random_initial_conditions(N);
    b.iter(|| {
        let mut forces = Vec::<Vector>::new();
        particles
                .par_iter()
                .map(|p| forces_by_direct_summation(*p, particles.iter()))
                .collect_into(&mut forces);
    })
}