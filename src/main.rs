extern crate rand;
extern crate nalgebra as na;

use rand::distributions::{IndependentSample, Range};
use na::{Vector2};

type Vector = Vector2<f64>;

struct Data {
    density: f64,
    velocity: Vector,
}

struct Mesh {
    cells: Vec<Data>,
    n_width: usize,
    cell_width: f64,
}

#[derive(PartialEq, Copy, Clone)]
struct GravityParticle {
    position: Vector,
    velocity: Vector,
    mass: f64,
}

fn newtonian_gravity(m1: f64, m2: f64, p1: Vector, p2: Vector) -> Vector {
    scal_mul(- m1 * m2 / norm(diff(p2, p1)).powi(2), diff(p2, p1))
}

fn scal_mul(a:f64, v: Vector) -> Vector {
    Vector::new(a * v[0], a * v[1])
}

fn add(p1: Vector, p2: Vector) -> Vector {
    Vector::new(p1[0] + p2[0], p1[1] + p2[1])
}

fn diff(p1: Vector, p2: Vector) -> Vector {
    Vector::new(p1[0] - p2[0], p1[1] - p2[1])
}

fn norm(v: Vector) -> f64 {
    (v[0] * v[0] + v[1] * v[1]).sqrt()
}

fn force_between_particles(particle_1: GravityParticle, particle_2: GravityParticle) -> Vector {
    newtonian_gravity(particle_1.mass, particle_2.mass, particle_1.position, particle_2.position)
}

fn sum_force<'a, I: Iterator<Item = &'a GravityParticle>>(particle: GravityParticle, all_particles: I) -> Vector {
    all_particles.filter(|x| **x != particle).
        map(|x| force_between_particles(*x, particle)).fold(Vector::new(0f64, 0f64), add)
}

fn main() {
    let between = Range::new(0f64, 1.);
    let mut rng = rand::thread_rng();

    let timestep = 0.1f64;
    let mut t = 0.0f64;
    let t_end = 10.0f64;
    let n = 1000;

    let mut ps: Vec<GravityParticle> = (0..n).map(|_| GravityParticle {
        position: Vector::new(between.ind_sample(&mut rng), between.ind_sample(&mut rng)),
        mass: between.ind_sample(&mut rng),
        velocity: Vector::new(0.0, 0.0),
    }).collect();

    while t < t_end {
        let mut forces = Vec::<Vector>::with_capacity(ps.len());

        for i in 0..ps.len() {
            forces.push(sum_force(ps[i], ps.iter()));
        }

        for i in 0..ps.len() {
            ps[i].velocity = add(ps[i].velocity, scal_mul(timestep, forces[i]));
            ps[i].position = add(ps[i].position, scal_mul(timestep, ps[i].velocity));
        }

        t += timestep;

        println!("{} {} {}", t, ps[0].position[0], ps[0].position[1]);
    }
}
