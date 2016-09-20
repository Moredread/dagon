#![cfg_attr(feature="herbie-lint", feature(plugin))]
#![cfg_attr(feature="herbie-lint", plugin(herbie_lint))]

#![feature(plugin)]
#![plugin(clippy)]

extern crate rand;
extern crate nalgebra as na;
extern crate num;
extern crate rmp_serialize as msgpack;
extern crate rustc_serialize;
extern crate rayon;
extern crate acacia;

use acacia::{AssociatedData, DataQuery, Node, Position, Tree};
use acacia::partition::Ncube;
use msgpack::Encoder;
use na::{FloatPoint, Norm, Origin, Point3, Vector3, zero};
use num::Zero;
use rand::distributions::{IndependentSample, Range};
use rayon::prelude::*;
use rustc_serialize::Encodable;

use std::fs::*;
use std::io::prelude::*;
use std::ops::Add;
use std::io;

pub type Vector = Vector3<f64>;
pub type Point = Point3<f64>;

#[derive(RustcEncodable, RustcDecodable, PartialEq, Copy, Clone)]
pub struct GravityParticle {
    pub position: Point,
    pub velocity: Vector,
    pub mass: f64,
}

impl Position for GravityParticle {
    type Point = Point;
    fn position(&self) -> Point {
        self.position
    }
}

pub fn newtonian_gravity_force(m1: f64, m2: f64, p1: Point, p2: Point) -> Vector {
    (-m1 * m2 / (p2 - p1).norm_squared()) * (p2 - p1).normalize()
}

// Force of particle source_particle on target_particle
pub fn force_between_particles(target_particle: GravityParticle, source_particle: GravityParticle) -> Vector {
    newtonian_gravity_force(target_particle.mass,
                            source_particle.mass,
                            target_particle.position,
                            source_particle.position)
}

pub fn forces_by_direct_summation<'a, I: Iterator<Item = &'a GravityParticle>>(target_particle: GravityParticle, all_particles: I) -> Vector {
    all_particles.filter(|source_particle| **source_particle != target_particle)
        .map(|x| force_between_particles(*x, target_particle))
        .fold(Vector::zero(), Vector::add)
}

pub fn forces_from_tree(target_particle: GravityParticle, tree: &Tree<acacia::partition::Ncube<Point, f64>, &GravityParticle, (Point, f64)>) -> Vector {

    let theta = 0.5; // A bit arbitrary but this appears to work

    // This is the recursion criterion. If a branch node passes this, the
    // query continues on its children.
    tree.query_data(|node| {
                    let &(ref center_of_mass, _) = node.data();
                    let d = FloatPoint::distance(&target_particle.position, center_of_mass);
                    let delta = FloatPoint::distance(&node.partition().center(), center_of_mass);
                    d < node.partition().width() / theta + delta
                })
                // This collects our force term from each piece of associated data the
                // tree encounters during recursion.
                .map(|&(center_of_mass, mass)| newtonian_gravity_force(mass, target_particle.mass, center_of_mass, target_particle.position))
                .fold(zero(), |a, b| a + b)
}

pub fn try_makedir(path: &str) -> std::io::Result<()> {
    match metadata(path) {
        Ok(meta) => {
            if !meta.is_dir() {
                Err(std::io::Error::new(std::io::ErrorKind::Other,
                                        "Target path exists, but is not a directory"))
            } else {
                Ok(())
            }
        }
        Err(_) => create_dir(path),
    }
}

pub fn make_random_initial_conditions(n: u64) -> Vec<GravityParticle> {
    let domain_range = Range::new(-1., 1.);
    let mass_range = Range::new(0.1, 1.);
    let mut rng = rand::thread_rng();

    (0..n)
        .map(|_| {
            GravityParticle {
                position: Point::new(domain_range.ind_sample(&mut rng),
                                     domain_range.ind_sample(&mut rng),
                                     domain_range.ind_sample(&mut rng)),
                mass: mass_range.ind_sample(&mut rng),
                velocity: Vector::zero(),
            }
        })
        .collect()
}