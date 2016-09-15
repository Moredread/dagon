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

use msgpack::Encoder;
use na::{Norm, Origin, Point3, Vector3};
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

fn newtonian_gravity_force(m1: f64, m2: f64, p1: Point, p2: Point) -> Vector {
    (-m1 * m2 / (p2 - p1).norm_squared()) * (p2 - p1).normalize()
}

fn force_between_particles(particle_1: GravityParticle, particle_2: GravityParticle) -> Vector {
    newtonian_gravity_force(particle_1.mass,
                            particle_2.mass,
                            particle_1.position,
                            particle_2.position)
}

fn sum_force<'a, I: Iterator<Item = &'a GravityParticle>>(particle: GravityParticle,
                                                          all_particles: I)
                                                          -> Vector {
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
    let domain_range = Range::new(0f64, 1.);
    let mut rng = rand::thread_rng();

    (0..n)
        .map(|_| {
            GravityParticle {
                position: Point::new(domain_range.ind_sample(&mut rng),
                                     domain_range.ind_sample(&mut rng),
                                     domain_range.ind_sample(&mut rng)),
                mass: domain_range.ind_sample(&mut rng),
                velocity: Vector::zero(),
            }
        })
        .collect()
}

fn main() {
    let timestep = 0.1f64;
    let mut current_time = 0.0f64;
    let finish_time = 10.0f64;
    let n = 10000;
    let mut step = 0u64;

    try_makedir("data").expect("Couldn't create dir");

    let mut ps = make_ics(n);

    while current_time < finish_time {
        println!("Timestep {}: time {}", step, current_time);

        let mut forces = Vec::<Vector>::new();
        ps.par_iter()
            .weight_max()
            .map(|p| sum_force(*p, ps.iter()))
            .collect_into(&mut forces);

        for i in 0..ps.len() {
            ps[i].velocity = ps[i].velocity + timestep * forces[i];
            ps[i].position = ps[i].position + timestep * ps[i].velocity;
        }

        let mut buf = Vec::new();
        let _ = ps.encode(&mut Encoder::new(&mut buf));

        let mut f = File::create(format!("data/data_{}.dat", step)).expect("Couldn't open file");
        f.write_all(buf.as_ref()).expect("Couldn't write to file");

        current_time += timestep;
        step += 1;
    }
}
