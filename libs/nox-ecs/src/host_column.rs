use std::{collections::BTreeMap, ops::Deref};

use bytemuck::Pod;
use conduit::{ComponentType, ComponentValue, EntityId, Metadata};
use nox::{
    xla::{ArrayElement, PjRtBuffer},
    Client, NoxprNode,
};
use smallvec::SmallVec;

use crate::{Component, DynArrayView, Error};

/// A type erased columnar data store located on the host CPU
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct HostColumn {
    pub buf: Vec<u8>,
    pub len: usize,
    pub metadata: Metadata,
}

impl HostColumn {
    pub fn new(metadata: Metadata) -> Self {
        HostColumn {
            buf: vec![],
            len: 0,
            metadata,
        }
    }

    pub fn entity_map(&self) -> BTreeMap<EntityId, usize> {
        if self.metadata.name != EntityId::NAME {
            return BTreeMap::default();
        }
        self.iter::<u64>()
            .map(EntityId)
            .enumerate()
            .map(|(offset, id)| (id, offset))
            .collect()
    }

    pub fn push<T: Component + 'static>(&mut self, val: T) {
        assert_eq!(self.metadata.component_type, T::component_type());
        let op = val.into_op();
        let NoxprNode::Constant(c) = op.deref() else {
            panic!("push into host column must be constant expr");
        };
        self.push_raw(c.data.raw_buf());
    }

    pub fn push_raw(&mut self, raw: &[u8]) {
        self.buf.extend_from_slice(raw);
        self.len += 1;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn copy_to_client(&self, client: &Client) -> Result<PjRtBuffer, Error> {
        let mut dims = self.metadata.component_type.shape.clone();
        dims.insert(0, self.len as i64);
        client
            .copy_raw_host_buffer(
                self.metadata.component_type.primitive_ty.element_type(),
                &self.buf,
                &dims[..],
            )
            .map_err(Error::from)
    }

    pub fn values_iter(&self) -> impl Iterator<Item = ComponentValue<'_>> + '_ {
        let mut buf_offset = 0;
        std::iter::from_fn(move || {
            let buf = self.buf.get(buf_offset..)?;
            let (offset, value) = self.metadata.component_type.parse_value(buf).ok()?;
            buf_offset += offset;
            Some(value)
        })
    }

    pub fn iter<T: conduit::Component>(&self) -> impl Iterator<Item = T> + '_ {
        assert_eq!(self.metadata.component_type, T::component_type());
        self.values_iter()
            .filter_map(|v| T::from_component_value(v))
    }

    pub fn component_type(&self) -> ComponentType {
        self.metadata.component_type.clone()
    }

    pub fn raw_buf(&self) -> &[u8] {
        &self.buf
    }

    pub fn typed_buf<T: ArrayElement + Pod>(&self) -> Option<&[T]> {
        if self.metadata.component_type.primitive_ty.element_type() != T::TY {
            return None;
        }
        bytemuck::try_cast_slice(self.buf.as_slice()).ok()
    }

    pub fn typed_buf_mut<T: ArrayElement + Pod>(&mut self) -> Option<&mut [T]> {
        if self.metadata.component_type.primitive_ty.element_type() != T::TY {
            return None;
        }
        bytemuck::try_cast_slice_mut(self.buf.as_mut_slice()).ok()
    }

    pub fn ndarray<T: ArrayElement + Pod>(&self) -> Option<ndarray::ArrayViewD<'_, T>> {
        let comp_shape = self.metadata.component_type.shape.iter().map(|n| *n as _);
        let shape: SmallVec<[usize; 4]> = std::iter::once(self.len).chain(comp_shape).collect();
        let buf = self.typed_buf::<T>()?;
        ndarray::ArrayViewD::from_shape(&shape[..], buf).ok()
    }

    pub fn dyn_ndarray(&self) -> Option<DynArrayView<'_>> {
        let elem_type = self.metadata.component_type.primitive_ty.element_type();
        match elem_type {
            nox::xla::ElementType::Pred => {
                todo!()
            }
            nox::xla::ElementType::S8 => {
                let buf = self.ndarray::<i8>()?;
                Some(DynArrayView::I8(buf))
            }
            nox::xla::ElementType::S16 => {
                let buf = self.ndarray::<i16>()?;
                Some(DynArrayView::I16(buf))
            }
            nox::xla::ElementType::S32 => {
                let buf = self.ndarray::<i32>()?;
                Some(DynArrayView::I32(buf))
            }
            nox::xla::ElementType::S64 => {
                let buf = self.ndarray::<i64>()?;
                Some(DynArrayView::I64(buf))
            }
            nox::xla::ElementType::U8 => {
                let buf = self.ndarray::<u8>()?;
                Some(DynArrayView::U8(buf))
            }
            nox::xla::ElementType::U16 => {
                let buf = self.ndarray::<u16>()?;
                Some(DynArrayView::U16(buf))
            }
            nox::xla::ElementType::U32 => {
                let buf = self.ndarray::<u32>()?;
                Some(DynArrayView::U32(buf))
            }
            nox::xla::ElementType::U64 => {
                let buf = self.ndarray::<u64>()?;
                Some(DynArrayView::U64(buf))
            }
            nox::xla::ElementType::F32 => {
                let buf = self.ndarray::<f32>()?;
                Some(DynArrayView::F32(buf))
            }
            nox::xla::ElementType::F64 => {
                let buf = self.ndarray::<f64>()?;
                Some(DynArrayView::F64(buf))
            }
            nox::xla::ElementType::F16 => {
                todo!()
            }
            nox::xla::ElementType::Bf16 => {
                todo!()
            }
            nox::xla::ElementType::C64 => {
                todo!()
            }
            nox::xla::ElementType::C128 => {
                todo!()
            }
        }
    }
}
