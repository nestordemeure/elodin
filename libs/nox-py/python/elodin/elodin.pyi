from __future__ import annotations
import jax
from typing import Any, Optional, Union, Tuple, ClassVar, List
import numpy
import polars as pl

class Entity:
    def id(self) -> EntityId: ...
    def insert(self, archetype: Any) -> Entity: ...
    def metadata(self, metadata: EntityMetadata) -> Entity: ...
    def name(self, name: str) -> Entity: ...

class PrimitiveType:
    F64: PrimitiveType

class ComponentType:
    def __init__(self, ty: PrimitiveType, shape: Tuple[int]): ...
    ty: PrimitiveType
    shape: jax.typing.ArrayLike
    U64: ClassVar[ComponentType]
    F64: ClassVar[ComponentType]
    F32: ClassVar[ComponentType]
    Edge: ClassVar[ComponentType]
    Quaternion: ClassVar[ComponentType]
    SpatialPosF64: ClassVar[ComponentType]
    SpatialMotionF64: ClassVar[ComponentType]

class ComponentId:
    def __init__(self, id: Union[int, str]): ...

class PipelineBuilder:
    def init_var(self, id: ComponentId, ty: ComponentType): ...
    def var_arrays(self) -> list[jax.typing.ArrayLike]: ...

class WorldBuilder:
    def spawn(self, archetype: Any) -> Entity: ...
    def spawn_with_entity_id(self, id: EntityId, archetype: Any) -> Entity: ...
    def insert_asset(self, asset: Any): ...
    def run(
        self,
        sys: Any,
        time_step: Optional[float] = None,
        client: Optional[Client] = None,
    ): ...
    def build(self, sys: Any, time_step: Optional[float] = None) -> Exec: ...

class EntityId:
    def __init__(self, id: int): ...

class Client:
    @staticmethod
    def cpu() -> Client: ...

class SpatialTransform:
    shape: jax.typing.ArrayLike
    def __init__(self, arr: jax.typing.ArrayLike): ...
    @staticmethod
    def from_linear(linear: jax.typing.ArrayLike) -> SpatialTransform: ...
    @staticmethod
    def from_angular(linear: jax.typing.ArrayLike) -> SpatialTransform: ...
    @staticmethod
    def from_axis_angle(
        self, axis: jax.typing.ArrayLike, angle: jax.typing.ArrayLike
    ) -> SpatialTransform: ...
    def flatten(self) -> Any: ...
    @staticmethod
    def unflatten(aux: Any, jax: Any) -> Any: ...
    @staticmethod
    def from_array(arr: jax.typing.ArrayLike) -> SpatialTransform: ...
    @staticmethod
    def zero() -> SpatialTransform: ...
    def linear(self) -> jax.Array: ...
    def angular(self) -> Quaternion: ...
    def asarray(self) -> jax.typing.ArrayLike: ...

class SpatialForce:
    shape: jax.typing.ArrayLike
    def __init__(self, arr: jax.typing.ArrayLike): ...
    @staticmethod
    def from_array(arr: jax.typing.ArrayLike) -> SpatialForce: ...
    def flatten(self) -> Any: ...
    @staticmethod
    def unflatten(aux: Any, jax: Any) -> Any: ...
    def asarray(self) -> jax.typing.ArrayLike: ...
    @staticmethod
    def zero() -> SpatialForce: ...
    @staticmethod
    def from_linear(linear: jax.typing.ArrayLike) -> SpatialForce: ...
    @staticmethod
    def from_torque(linear: jax.typing.ArrayLike) -> SpatialForce: ...
    def force(self) -> jax.typing.ArrayLike: ...
    def torque(self) -> jax.typing.ArrayLike: ...

class SpatialMotion:
    shape: jax.typing.ArrayLike
    def __init__(self, angular: jax.typing.ArrayLike, linear: jax.typing.ArrayLike): ...
    @staticmethod
    def from_array(arr: jax.typing.ArrayLike) -> SpatialMotion: ...
    def flatten(self) -> Any: ...
    @staticmethod
    def unflatten(aux: Any, jax: Any) -> Any: ...
    def asarray(self) -> jax.typing.ArrayLike: ...
    @staticmethod
    def zero() -> SpatialMotion: ...
    @staticmethod
    def from_linear(linear: jax.typing.ArrayLike) -> SpatialMotion: ...
    @staticmethod
    def from_angular(linear: jax.typing.ArrayLike) -> SpatialMotion: ...
    def linear(self) -> jax.Array: ...
    def angular(self) -> jax.typing.ArrayLike: ...

class SpatialInertia:
    shape: jax.typing.ArrayLike
    def __init__(self, mass: jax.typing.ArrayLike, inertia: jax.typing.ArrayLike): ...
    @staticmethod
    def from_array(arr: jax.typing.ArrayLike) -> SpatialInertia: ...
    def flatten(self) -> Any: ...
    @staticmethod
    def unflatten(aux: Any, jax: Any) -> Any: ...
    @staticmethod
    def zero() -> SpatialInertia: ...
    @staticmethod
    def from_mass(mass: jax.typing.ArrayLike) -> SpatialInertia: ...
    def mass(self) -> jax.typing.ArrayLike: ...

class Quaternion:
    shape: jax.typing.ArrayLike
    def __init__(self, arr: jax.typing.ArrayLike): ...
    @staticmethod
    def from_array(arr: jax.typing.ArrayLike) -> Quaternion: ...
    def flatten(self) -> Any: ...
    @staticmethod
    def unflatten(aux: Any, jax: Any) -> Any: ...
    def asarray(self) -> jax.typing.ArrayLike: ...
    @staticmethod
    def zero() -> Quaternion: ...
    @staticmethod
    def from_axis_angle(
        axis: jax.typing.ArrayLike, angle: jax.typing.ArrayLike
    ) -> Quaternion: ...
    def vector(self) -> jax.Array: ...
    def normalize(self) -> Quaternion: ...

class RustSystem: ...

class Mesh:
    @staticmethod
    def cuboid(x: float, y: float, z: float) -> Mesh: ...
    @staticmethod
    def sphere(radius: float) -> Mesh: ...
    def bytes(self) -> bytes: ...

class Material:
    def bytes(self) -> bytes: ...
    @staticmethod
    def color(r: float, g: float, b: float) -> Material: ...

class Texture: ...

class Handle:
    def flatten(self) -> Any: ...
    @staticmethod
    def unflatten(aux: Any, jax: Any) -> Any: ...

class Pbr:
    def __init__(self, mesh: Mesh, material: Material): ...
    @staticmethod
    def from_url(url: str) -> Pbr: ...
    @staticmethod
    def from_path(path: str) -> Pbr: ...
    def bytes(self) -> bytes: ...

class EntityMetadata:
    def __init__(self, name: str, color: Optional[Color] = None): ...
    def asset_id(self) -> int: ...
    def bytes(self) -> bytes: ...

class Metadata: ...

class QueryInner:
    def join_query(self, other: QueryInner) -> QueryInner: ...
    def arrays(self) -> list[jax.Array]: ...
    def map(self, ty: jax.Array, f: Metadata) -> Any: ...
    @staticmethod
    def from_builder(
        builder: PipelineBuilder, ids: list[ComponentId]
    ) -> QueryInner: ...
    def insert_into_builder(self, builder: PipelineBuilder) -> None: ...

class GraphQueryInner:
    def arrays(self) -> dict[int, Tuple[list[jax.Array], list[jax.Array]]]: ...
    @staticmethod
    def from_builder(
        builder: PipelineBuilder, edge_id: ComponentId, ids: list[ComponentId]
    ) -> GraphQueryInner: ...
    def insert_into_builder(self, builder: PipelineBuilder) -> None: ...
    def map(self, ty: jax.typing.ArrayLike, f: Metadata) -> QueryInner: ...

class Edge:
    def __init__(self, a: EntityId, b: EntityId): ...
    def flatten(self) -> Any: ...
    @staticmethod
    def unflatten(aux: Any, jax: Any) -> Any: ...

class Component:
    ty: ComponentType
    asset: bool
    name: Optional[str]
    def __init__(
        self,
        id: Union[ComponentId, str],
        ty: ComponentType,
        asset: bool = False,
        name: Optional[str] = None,
    ): ...
    def to_metadata(self) -> Metadata: ...
    @staticmethod
    def id(component: Any) -> ComponentId: ...

class Conduit:
    @staticmethod
    def tcp(addr: str) -> Conduit: ...

class Exec:
    def run(self, client: Client): ...
    def history(self) -> pl.DataFrame: ...
    def column_array(self, id: ComponentId) -> numpy.ndarray: ...

class Color:
    def __init__(self, r: float, g: float, b: float): ...

class Gizmo:
    @staticmethod
    def vector(id: ComponentId, offset: int, color: Color) -> jax.Array: ...

class Panel:
    @staticmethod
    def viewport(
        track_entity: Optional[EntityId] = None,
        track_rotation: bool = True,
        fov: Optional[float] = None,
        active: bool = False,
        pos: Union[List[float], jax.Array, None] = None,
        looking_at: Union[List[float], jax.Array, None] = None,
    ) -> Panel: ...

def six_dof(time_step: float, sys: Any = None) -> Any: ...
