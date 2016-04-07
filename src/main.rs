use std::ops::Add;

struct Data {
    density: f64,
    velocity: [f64; 2],
}

struct Mesh {
    cells: Vec<Data>,
    n_width: usize,
    cell_width: f64,
}

impl Mesh {

}

#[derive(PartialEq, Copy, Clone)]
struct GravityParticle {
    position: [f64; 2],
    mass: f64,
}

fn newt_onion_gravity(m1: f64, m2: f64, r: f64) -> f64 {
    - m1 * m2 / r.powi(2)
}

fn norm(v: [f64; 2]) -> f64 {
    (v[0] * v[0] + v[1] * v[1]).sqrt()
}

fn distance(pos1: [f64;2], pos2: [f64; 2]) -> f64 {
    norm([pos2[0] - pos1[0], pos2[1] - pos1[1]])
}

fn force_between_particles(particle_1: GravityParticle, particle_2: GravityParticle) -> f64 {
    newt_onion_gravity(particle_1.mass, particle_2.mass, distance(particle_1.position, particle_2.position))
}

fn sum_force<I: Iterator<Item = GravityParticle>>(particle: GravityParticle, all_particles: I) -> f64 {
    all_particles.filter(|&x| x != particle).
        map(|x| force_between_particles(x, particle)).fold(0f64, Add::add)
}

fn main() {
    
}
