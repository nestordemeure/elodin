from jax import numpy as np
from jax.numpy import linalg as la
from elodin import *

def gravity_impl(pos, inertia, force):
  G = 6.67430e-11
  big_m = 1/G
  mass = inertia.mass()
  r = pos.linear()
  norm = la.norm(r)
  f = -r / norm * norm * norm * G * big_m * mass
  return Force.from_linear(f)

@system
def gravity(q: Query[WorldPos, Inertia, Force]) -> ComponentArray[Force]:
  return q.map(Force, gravity_impl)

w = WorldBuilder()
w.spawn(Body(
    world_pos = WorldPos.from_linear(np.array([0.,0.,0.01])),
    world_vel = WorldVel.from_linear(np.array([0.,0.,0.])),
    inertia = Inertia.from_mass(1.0),
    mesh = w.insert_asset(Mesh.sphere(1)),
    material = w.insert_asset(Material.color(25.3, 18.4, 1.0))
))
w.spawn(Body(
    world_pos = WorldPos.from_linear(np.array([5.,0.,0.])),
    world_vel = WorldVel.from_linear(np.array([0.,0.,10.])),
    inertia = Inertia.from_mass(1.0),
    mesh = w.insert_asset(Mesh.sphere(0.2)),
    material = w.insert_asset(Material.color(1.0, 1.0, 1.0))
))
w.spawn(Body(
    world_pos = WorldPos.from_linear(np.array([8.,0.,0.])),
    world_vel = WorldVel.from_linear(np.array([0.,0.,24.])),
    inertia = Inertia.from_mass(2.0),
    mesh = w.insert_asset(Mesh.sphere(0.3)),
    material = w.insert_asset(Material.color(1.0, 1.0, 1.0))
))
sys = six_dof(1.0 / 60.0, gravity)
sim = w.run(sys)
