#![cfg_attr(feature="herbie-lint", feature(plugin))]
#![cfg_attr(feature="herbie-lint", plugin(herbie_lint))]

#![feature(plugin)]
#![plugin(clippy)]

#[macro_use] extern crate log;
extern crate env_logger;

extern crate rand;
extern crate nalgebra as na;
extern crate num;
extern crate rmp_serialize as msgpack;
extern crate rustc_serialize;
extern crate rayon;
extern crate acacia;
extern crate dagon;

use acacia::Tree;
use acacia::partition::Ncube;

use dagon::*;
use msgpack::Encoder;
use na::Origin;
use rayon::prelude::*;
use rustc_serialize::Encodable;

use std::fs::*;
use std::io::prelude::*;

fn main() {
    env_logger::init().unwrap();

    let timestep = 0.1f64;
    let mut current_time = 0.0f64;
    let finish_time = 10.0f64;
    let n = 1000;
    let mut step = 0u64;

    try_makedir("data").expect("Couldn't create dir");

    info!("Initialize random particle (n = {}) distribution", n);
    let mut particles = make_random_initial_conditions(n);

    while current_time < finish_time {
        info!("Timestep {}: time {}", step, current_time);
        let origin: Point = Origin::origin();
        let mut forces = Vec::<Vector>::new();

        {
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

            let _: Vec<_> = particles
                .iter()
                //.par_iter()
                //.weight_max()
                .map(|p| forces_from_tree(*p, &tree))
                //.collect_into(&mut forces);
                .collect();
        }

        trace!("Calculate forces");
        particles.par_iter()
            .weight_max()
            .map(|p| forces_by_direct_summation(*p, particles.iter()))
            .collect_into(&mut forces);

        trace!("Advance velocities and positions");
        for i in 0..particles.len() {
            particles[i].velocity = particles[i].velocity + timestep * forces[i] / particles[i].mass;
            particles[i].position = particles[i].position + timestep * particles[i].velocity;
        }

        trace!("Write data");
        let mut buf = Vec::new();
        let _ = particles.encode(&mut Encoder::new(&mut buf));

        let mut f = File::create(format!("data/data_{}.dat", step)).expect("Couldn't open file");
        f.write_all(buf.as_ref()).expect("Couldn't write to file");

        current_time += timestep;
        step += 1;
    }
}
