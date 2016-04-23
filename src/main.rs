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

fn newtonian_gravity(m1: f64, m2: f64, p1: [f64; 2], p2: [f64;2]) -> [f64; 2] {
    scal_mul(- m1 * m2 / norm(diff(p2, p1)).powi(2), diff(p2, p1))
}

fn scal_mul(a:f64, v: [f64; 2]) -> [f64; 2] {
    [a * v[0], a * v[1]]
}

fn add(p1: [f64; 2], p2: [f64; 2]) -> [f64; 2] {
    [p1[0] + p2[0], p1[1] + p2[1]]
}

fn diff(p1: [f64; 2], p2: [f64; 2]) -> [f64; 2] {
    [p1[0] - p2[0], p1[1] - p2[1]]
}

fn norm(v: [f64; 2]) -> f64 {
    (v[0] * v[0] + v[1] * v[1]).sqrt()
}

fn force_between_particles(particle_1: GravityParticle, particle_2: GravityParticle) -> [f64; 2] {
    newtonian_gravity(particle_1.mass, particle_2.mass, particle_1.position, particle_2.position)
}

fn sum_force<'a, I: Iterator<Item = &'a GravityParticle>>(particle: GravityParticle, all_particles: I) -> [f64; 2] {
    all_particles.filter(|x| **x != particle).
        map(|x| force_between_particles(*x, particle)).fold([0f64, 0f64], add)
}

fn main() {
    let p1 = GravityParticle { position: [0.0f64, 0.0f64], mass: 1.0 };
    let p2 = GravityParticle { position: [0.0f64, 1.0f64], mass: 1.0 };
    let p3 = GravityParticle { position: [0.0f64, 2.0f64], mass: 1.0 };

    let p = [p1, p2, p3];
    println!("{}", sum_force(p1, p.into_iter())[1]);
}
