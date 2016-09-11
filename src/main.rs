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

use rustc_serialize::Encodable;
use msgpack::Encoder;

use rand::distributions::{IndependentSample, Range};
use std::ops::Add;
use na::{Vector3, Norm};
use num::Zero;
use std::io::prelude::*;
use std::fs::File;
use rayon::prelude::*;

type Vector = Vector3<f64>;

struct Data {
    density: f64,
    velocity: Vector,
}

struct Mesh {
    cells: Vec<Data>,
    n_width: usize,
    cell_width: f64,
}

#[derive(RustcEncodable, RustcDecodable, PartialEq, Copy, Clone)]
struct GravityParticle {
    position: Vector,
    velocity: Vector,
    mass: f64,
}

fn newtonian_gravity_force(m1: f64, m2: f64, p1: Vector, p2: Vector) -> Vector {
    (- m1 * m2 / (p2 - p1).norm_squared()) * (p2 - p1).normalize()
}

fn force_between_particles(particle_1: GravityParticle, particle_2: GravityParticle) -> Vector {
    newtonian_gravity_force(particle_1.mass, particle_2.mass, particle_1.position, particle_2.position)
}

fn sum_force<'a, I: Iterator<Item = &'a GravityParticle>>(particle: GravityParticle, all_particles: I) -> Vector {
    all_particles.filter(|x| **x != particle).
        map(|x| force_between_particles(*x, particle)).fold(Vector::zero(), Vector::add)
}

fn main() {
    let between = Range::new(0f64, 1.);
    let mut rng = rand::thread_rng();

    let timestep = 0.1f64;
    let mut t = 0.0f64;
    let t_end = 10.0f64;
    let n = 10000;

    let mut ps: Vec<GravityParticle> = (0..n).map(|_| GravityParticle {
        position: Vector::new(between.ind_sample(&mut rng), between.ind_sample(&mut rng), between.ind_sample(&mut rng)),
        mass: between.ind_sample(&mut rng),
        velocity: Vector::zero(),
    }).collect();

    let mut step = 0u64;

    while t < t_end {
        println!("Timestep {}: time {}", step, t);
        let mut forces = Vec::<Vector>::new();

        ps.par_iter().weight_max().map(
            |p| sum_force(*p, ps.iter())
        ).collect_into(&mut forces);

        for i in 0..ps.len() {
            ps[i].velocity = ps[i].velocity + timestep * forces[i];
            ps[i].position = ps[i].position + timestep * ps[i].velocity;
        }

        let mut buf = Vec::new();
        let _ = ps.encode(&mut Encoder::new(&mut buf));

        let mut f = File::create(format!("data_{}.dat", step)).expect("Couldn't open file");
        f.write_all(buf.as_ref()).expect("Couldn't write to file");

        t += timestep;
        step += 1;
    }
}
