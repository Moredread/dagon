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
use na::{Norm, Origin, Point3, Vector3, zero, FloatPoint};
use num::Zero;
use rand::distributions::{IndependentSample, Range};
use rayon::prelude::*;
use rustc_serialize::Encodable;
use std::fs::*;
use std::io::prelude::*;
use std::ops::Add;

type Vector = Vector3<f64>;
type Point = Point3<f64>;

#[derive(RustcEncodable, RustcDecodable, PartialEq, Copy, Clone)]
struct GravityParticle {
    position: Point,
    velocity: Vector,
    mass: f64,
}

impl Position for GravityParticle {
    type Point = Point;
    fn position(&self) -> Point {
        self.position
    }
}

fn newtonian_gravity_force(m1: f64, m2: f64, p1: Point, p2: Point) -> Vector {
    (-m1 * m2 / (p2 - p1).norm_squared()) * (p2 - p1).normalize()
}

// Force of particle source_particle on target_particle
fn force_between_particles(target_particle: GravityParticle, source_particle: GravityParticle) -> Vector {
    newtonian_gravity_force(target_particle.mass,
                            source_particle.mass,
                            target_particle.position,
                            source_particle.position)
}

fn sum_force<'a, I: Iterator<Item = &'a GravityParticle>>(particle: GravityParticle, all_particles: I) -> Vector {
    all_particles.filter(|x| **x != particle)
        .map(|x| force_between_particles(*x, particle))
        .fold(Vector::zero(), Vector::add)
}

fn try_makedir(path: &str) -> std::io::Result<()> {
    match std::fs::metadata(path) {
        Ok(meta) => {
            if !meta.is_dir() {
                Err(std::io::Error::new(std::io::ErrorKind::Other,
                                        "Target path exists, but is not a directory"))
            } else {
                Ok(())
            }
        }
        Err(_) => std::fs::create_dir(path),
    }
}

fn make_ics(n: u64) -> Vec<GravityParticle> {
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

fn main() {
    let timestep = 0.1f64;
    let mut current_time = 0.0f64;
    let finish_time = 10.0f64;
    let n = 1000;
    let mut step = 0u64;

    try_makedir("data").expect("Couldn't create dir");

    let mut particles = make_ics(n);

    while current_time < finish_time {
        println!("Timestep {}: time {}", step, current_time);
        let origin: Point = Origin::origin();
        let mut forces = Vec::<Vector>::new();

        {
            // Construct the tree, taken from acacia example
            let tree = Tree::new(// Pass in an iterator over the objects to store in the tree
                                 // In this case we pass in &PointMass, which implements Positionable.
                                 particles.iter(),
                                 // Shape of the root node
                                 Ncube::new(origin, 30000.0),
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
                                 }).expect("Couldn't construct tree");

            let theta = 0.5; // A bit arbitrary but this appears to work
            let tree_gravity: Vector =
                // This is the recursion criterion. If a branch node passes this, the
                // query continues on its children.
                tree.query_data(|node| {
                    let &(ref center_of_mass, _) = node.data();
                    let d = FloatPoint::distance(&particles[0].position, center_of_mass);
                    let delta = FloatPoint::distance(&node.partition().center(), center_of_mass);
                    d < node.partition().width() / theta + delta
                })
                // This collects our force term from each piece of associated data the
                // tree encounters during recursion.
                .map(|&(center_of_mass, mass)| newtonian_gravity_force(mass, particles[0].mass, center_of_mass, particles[0].position))
                .fold(zero(), |a, b| a + b);
        }

        particles.par_iter()
            .weight_max()
            .map(|p| sum_force(*p, particles.iter()))
            .collect_into(&mut forces);

        for i in 0..particles.len() {
            particles[i].velocity = particles[i].velocity + timestep * forces[i] / particles[i].mass;
            particles[i].position = particles[i].position + timestep * particles[i].velocity;
        }

        let mut buf = Vec::new();
        let _ = particles.encode(&mut Encoder::new(&mut buf));

        let mut f = File::create(format!("data/data_{}.dat", step)).expect("Couldn't open file");
        f.write_all(buf.as_ref()).expect("Couldn't write to file");

        current_time += timestep;
        step += 1;
    }
}
