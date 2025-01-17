use crate::{Component, ComponentType, ComponentValue, PrimitiveTy};
use ndarray::{array, CowArray};
use smallvec::smallvec;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
pub struct Camera;

impl Component for Camera {
    const NAME: &'static str = "camera";

    const ASSET: bool = false;

    fn component_type() -> ComponentType {
        ComponentType {
            primitive_ty: PrimitiveTy::U64,
            shape: smallvec![0],
        }
    }

    fn component_value<'a>(&self) -> crate::ComponentValue<'a> {
        let arr = array![0].into_dyn();
        ComponentValue::U64(CowArray::from(arr))
    }

    fn from_component_value(_: crate::ComponentValue<'_>) -> Option<Self>
    where
        Self: Sized,
    {
        Some(Self)
    }
}
