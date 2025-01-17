use nalgebra::{constraint::ShapeConstraint, Const, Dyn};
use smallvec::SmallVec;
use std::{
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{Add, Div, Mul, Sub},
};

use crate::{
    AddDim, BroadcastDim, BroadcastedDim, DefaultMap, DefaultMappedDim, Dim, DottedDim, Field,
    MapDim, ReplaceMappedDim, Repr, ScalarDim, TensorDim, XlaDim,
};

pub struct Array<T: Copy, D: ArrayDim> {
    buf: D::Buf<T>,
}

pub trait ArrayDim: TensorDim {
    type Buf<T>: ArrayBuf<T>
    where
        T: Copy;
    type Dim: AsRef<[usize]> + AsMut<[usize]> + Clone;
    fn dim<T: Copy>(_buf: &Self::Buf<T>) -> Self::Dim;
    fn strides<T: Copy>(_buf: &Self::Buf<T>) -> Self::Dim;
}

impl ArrayDim for ScalarDim {
    type Buf<T> = T where T: Copy;

    type Dim = [usize; 0];

    fn dim<T: Copy>(_buf: &Self::Buf<T>) -> Self::Dim {
        []
    }

    fn strides<T: Copy>(_buf: &Self::Buf<T>) -> Self::Dim {
        []
    }
}

impl<const D: usize> ArrayDim for Const<D> {
    type Buf<T> = [T; D] where T: Copy;

    type Dim = [usize; 1];

    #[inline]
    fn dim<T: Copy>(_buf: &Self::Buf<T>) -> Self::Dim {
        [D]
    }

    fn strides<T: Copy>(_buf: &Self::Buf<T>) -> Self::Dim {
        [1]
    }
}

impl<const D1: usize, const D2: usize> ArrayDim for (Const<D1>, Const<D2>) {
    type Buf<T> = [[T; D2]; D1] where T: Copy;

    type Dim = [usize; 2];

    fn dim<T: Copy>(_buf: &Self::Buf<T>) -> Self::Dim {
        [D1, D2]
    }

    fn strides<T: Copy>(_buf: &Self::Buf<T>) -> Self::Dim {
        [D2, 1]
    }
}

impl<const D1: usize, const D2: usize, const D3: usize> ArrayDim
    for (Const<D1>, Const<D2>, Const<D3>)
{
    type Buf<T> = [[[T; D3]; D2]; D1] where T: Copy;
    type Dim = [usize; 3];

    fn dim<T: Copy>(_buf: &Self::Buf<T>) -> Self::Dim {
        [D1, D2, D3]
    }

    fn strides<T: Copy>(_buf: &Self::Buf<T>) -> Self::Dim {
        [D3 * D2, D2, 1]
    }
}

pub trait ArrayBuf<T>: Clone {
    fn as_buf(&self) -> &[T];
    fn as_mut_buf(&mut self) -> &mut [T];
}

pub trait ArrayBufUnit<T>: ArrayBuf<MaybeUninit<T>> {
    fn uninit(dims: &[usize]) -> Self;

    type Init;
    fn assume_init(self) -> Self::Init;
}

impl<T: Clone + Copy> ArrayBuf<T> for ndarray::ArrayD<T> {
    fn as_buf(&self) -> &[T] {
        self.as_slice().expect("ndarray in non-standard order")
    }

    fn as_mut_buf(&mut self) -> &mut [T] {
        self.as_slice_mut().expect("ndarray in non-standard order")
    }
}

impl<T: Clone + Copy> ArrayBufUnit<T> for ndarray::ArrayD<MaybeUninit<T>> {
    fn uninit(dims: &[usize]) -> Self {
        unsafe { ndarray::ArrayD::uninit(dims).assume_init() }
    }

    type Init = ndarray::ArrayD<T>;

    fn assume_init(self) -> Self::Init {
        unsafe { self.assume_init() }
    }
}

impl<T: Copy + Clone> ArrayBuf<T> for T {
    fn as_buf(&self) -> &[T] {
        core::slice::from_ref(self)
    }

    fn as_mut_buf(&mut self) -> &mut [T] {
        core::slice::from_mut(self)
    }
}

impl<T: Copy + Clone> ArrayBufUnit<T> for MaybeUninit<T> {
    fn uninit(_dims: &[usize]) -> Self {
        MaybeUninit::uninit()
    }

    type Init = T;

    fn assume_init(self) -> Self::Init {
        unsafe { self.assume_init() }
    }
}

impl<const D: usize, T: Copy + Clone> ArrayBuf<T> for [T; D] {
    fn as_buf(&self) -> &[T] {
        self
    }

    fn as_mut_buf(&mut self) -> &mut [T] {
        self
    }
}

impl<const D: usize, T: Copy + Clone> ArrayBufUnit<T> for [MaybeUninit<T>; D] {
    fn uninit(_dims: &[usize]) -> Self {
        unsafe { MaybeUninit::<[MaybeUninit<T>; D]>::uninit().assume_init() }
    }

    type Init = [T; D];

    fn assume_init(self) -> Self::Init {
        unsafe { core::mem::transmute_copy(&self) }
    }
}

impl<T: Clone + Copy, const D1: usize, const D2: usize> ArrayBuf<T> for [[T; D1]; D2] {
    fn as_buf(&self) -> &[T] {
        let ptr = self.as_ptr();
        let len = D1 * D2;

        unsafe { std::slice::from_raw_parts(ptr as *const T, len) }
    }

    fn as_mut_buf(&mut self) -> &mut [T] {
        let ptr = self.as_ptr();
        let len = D1 * D2;

        unsafe { std::slice::from_raw_parts_mut(ptr as *mut T, len) }
    }
}

impl<const D1: usize, const D2: usize, T: Copy + Clone> ArrayBufUnit<T>
    for [[MaybeUninit<T>; D1]; D2]
{
    fn uninit(_dims: &[usize]) -> Self {
        unsafe { MaybeUninit::<[[MaybeUninit<T>; D1]; D2]>::uninit().assume_init() }
    }

    type Init = [[T; D1]; D2];

    fn assume_init(self) -> Self::Init {
        unsafe { core::mem::transmute_copy(&self) }
    }
}

impl<T: Clone + Copy, const D1: usize, const D2: usize, const D3: usize> ArrayBuf<T>
    for [[[T; D1]; D2]; D3]
{
    fn as_buf(&self) -> &[T] {
        let ptr = self.as_ptr();
        let len = D1 * D2 * D3;

        unsafe { std::slice::from_raw_parts(ptr as *const T, len) }
    }

    fn as_mut_buf(&mut self) -> &mut [T] {
        let ptr = self.as_ptr();
        let len = D1 * D2 * D3;

        unsafe { std::slice::from_raw_parts_mut(ptr as *mut T, len) }
    }
}

impl<const D1: usize, const D2: usize, const D3: usize, T: Copy + Clone> ArrayBufUnit<T>
    for [[[MaybeUninit<T>; D1]; D2]; D3]
{
    fn uninit(_dims: &[usize]) -> Self {
        unsafe { MaybeUninit::<[[[MaybeUninit<T>; D1]; D2]; D3]>::uninit().assume_init() }
    }

    type Init = [[[T; D1]; D2]; D3];

    fn assume_init(self) -> Self::Init {
        unsafe { core::mem::transmute_copy(&self) }
    }
}

pub trait MaybeUnitMarker {
    type Init;
    fn uninit() -> Self;
}
impl<T> MaybeUnitMarker for MaybeUninit<T> {
    type Init = T;
    fn uninit() -> Self {
        Self::uninit()
    }
}

impl ArrayDim for Dyn {
    type Buf<T> = ndarray::ArrayD<T> where T: Clone + Copy;

    type Dim = SmallVec<[usize; 4]>;

    fn dim<T: Copy>(buf: &Self::Buf<T>) -> Self::Dim {
        buf.shape().iter().copied().collect()
    }

    fn strides<T: Copy>(buf: &Self::Buf<T>) -> Self::Dim {
        buf.strides().iter().map(|x| *x as usize).collect()
    }
}

impl<T1: Copy, D1: ArrayDim + TensorDim + XlaDim> Array<MaybeUninit<T1>, D1>
where
    D1::Buf<MaybeUninit<T1>>: ArrayBufUnit<T1>,
{
    fn uninit(dims: &[usize]) -> Self {
        Self {
            buf: D1::Buf::<MaybeUninit<T1>>::uninit(dims),
        }
    }

    unsafe fn assume_init(self) -> Array<T1, D1>
    where
        <D1 as ArrayDim>::Buf<MaybeUninit<T1>>: ArrayBufUnit<T1, Init = <D1 as ArrayDim>::Buf<T1>>,
    {
        Array {
            buf: self.buf.assume_init(),
        }
    }
}

macro_rules! impl_op {
    ($op:tt, $op_trait:tt, $fn_name:tt) => {
        impl<T1: Copy, D1: ArrayDim + TensorDim + XlaDim> Array<T1, D1> {
            pub fn $fn_name<D2: ArrayDim + TensorDim + XlaDim>(
                &self,
                b: &Array<T1, D2>,
            ) -> Array<T1, BroadcastedDim<D1, D2>>
            where
                <BroadcastedDim<D1, D2> as ArrayDim>::Buf<MaybeUninit<T1>>:
                    ArrayBufUnit<T1, Init = <BroadcastedDim<D1, D2> as ArrayDim>::Buf<T1>>,
                T1: $op_trait<Output = T1>,
                ShapeConstraint: BroadcastDim<D1, D2>,
                <ShapeConstraint as BroadcastDim<D1, D2>>::Output: ArrayDim + XlaDim,
            {
                let d1 = D1::dim(&self.buf);
                let d2 = D2::dim(&b.buf);

                match d1.as_ref().len().cmp(&d2.as_ref().len()) {
                    std::cmp::Ordering::Less | std::cmp::Ordering::Equal => {
                        let mut out: Array<MaybeUninit<T1>, BroadcastedDim<D1, D2>> =
                            Array::uninit(d2.as_ref());
                        let mut broadcast_dims = d2.clone();
                        if !cobroadcast_dims(broadcast_dims.as_mut(), d1.as_ref()) {
                            todo!("handle unbroadcastble dims");
                        }
                        for ((a, b), out) in self
                            .broadcast_iter(broadcast_dims.clone())
                            .unwrap()
                            .zip(b.broadcast_iter(broadcast_dims).unwrap())
                            .zip(out.buf.as_mut_buf().iter_mut())
                        {
                            out.write(*a $op *b);
                        }
                        unsafe { out.assume_init() }
                    }
                    std::cmp::Ordering::Greater => {
                        let mut out: Array<MaybeUninit<T1>, BroadcastedDim<D1, D2>> =
                            Array::uninit(d2.as_ref());
                        let mut broadcast_dims = d1.clone();
                        if !cobroadcast_dims(broadcast_dims.as_mut(), d2.as_ref()) {
                            todo!("handle unbroadcastble dims");
                        }
                        for ((b, a), out) in b
                            .broadcast_iter(broadcast_dims.clone())
                            .unwrap()
                            .zip(self.broadcast_iter(broadcast_dims).unwrap())
                            .zip(out.buf.as_mut_buf().iter_mut())
                        {
                            out.write(*a $op *b);
                        }
                        unsafe { out.assume_init() }
                    }
                }
            }

        }
    }
}

impl_op!(*, Mul, mul);
impl_op!(+, Add, add);
impl_op!(-, Sub, sub);
impl_op!(/, Div, div);

impl<T1: Copy, D1: ArrayDim + TensorDim + XlaDim> Array<T1, D1> {
    pub fn broadcast_iter(
        &self,
        new_dims: impl AsMut<[usize]> + AsRef<[usize]> + Clone,
    ) -> Option<impl Iterator<Item = &'_ T1>> {
        let existing_dims = D1::dim(&self.buf);
        let existing_strides = D1::strides(&self.buf);
        let mut new_strides = new_dims.clone();
        let out_dims = new_dims.clone();
        let mut indexes = new_dims.clone();
        for i in indexes.as_mut().iter_mut() {
            *i = 0
        }
        for (i, ((dim, existing_stride), new_dim)) in existing_dims
            .as_ref()
            .iter()
            .zip(existing_strides.as_ref().iter())
            .zip(new_dims.as_ref().iter())
            .enumerate()
        {
            if dim == new_dim {
                new_strides.as_mut()[i] = *existing_stride;
            } else if *dim == 1 {
                new_strides.as_mut()[i] = 0;
            } else {
                return None;
            }
        }
        for (i, _) in new_dims.as_ref()[existing_dims.as_ref().len()..]
            .iter()
            .enumerate()
        {
            new_strides.as_mut()[i] = 0;
        }
        Some(StrideIterator {
            buf: self.buf.as_buf(),
            stride: new_strides,
            indexes,
            dims: out_dims,
            phantom: PhantomData,
        })
    }

    fn dot<D2>(
        &self,
        right: &Array<T1, D2>,
    ) -> Array<T1, <ShapeConstraint as crate::DotDim<D1, D2>>::Output>
    where
        T1: Field + Copy,
        D1: Dim + ArrayDim,
        D2: Dim + ArrayDim,
        ShapeConstraint: crate::DotDim<D1, D2>,
        <ShapeConstraint as crate::DotDim<D1, D2>>::Output: Dim + ArrayDim,
        <crate::DottedDim<D1, D2> as ArrayDim>::Buf<MaybeUninit<T1>>:
            ArrayBufUnit<T1, Init = <crate::DottedDim<D1, D2> as ArrayDim>::Buf<T1>>,
    {
        let left = self;
        let dim_left = D1::dim(&left.buf);
        let stride_left = D1::strides(&left.buf);
        let dim_right = D2::dim(&right.buf);
        let m = dim_left.as_ref().first().copied().unwrap_or(0);
        let k = dim_right.as_ref().first().copied().unwrap_or(0);
        let n = dim_right.as_ref().get(1).copied().unwrap_or(1);
        let stride_right = D2::strides(&right.buf);
        let (dims, rank) = matmul_dims(dim_left.as_ref(), dim_right.as_ref()).unwrap();
        let dims = &dims[..rank];
        let mut out: Array<MaybeUninit<T1>, DottedDim<D1, D2>> = Array::uninit(dims);
        let stride_out = <crate::DottedDim<D1, D2>>::strides(&out.buf);
        let alpha = T1::one_prim();
        let a = left.buf.as_buf().as_ref().as_ptr();
        let b = right.buf.as_buf().as_ref().as_ptr();
        let rsa = stride_left.as_ref().first().copied().unwrap_or(1) as isize;
        let csa = stride_left.as_ref().get(1).copied().unwrap_or(1) as isize;
        let rsb = stride_right.as_ref().first().copied().unwrap_or(1) as isize;
        let csb = stride_right.as_ref().get(1).copied().unwrap_or(1) as isize;
        let c = out.buf.as_mut_buf().as_mut().as_mut_ptr();
        let rsc = stride_out.as_ref().first().copied().unwrap_or(1) as isize;
        let csc = stride_out.as_ref().get(1).copied().unwrap_or(1) as isize;

        unsafe {
            T1::gemm(
                m,
                k,
                n,
                alpha,
                a,
                rsa,
                csa,
                b,
                rsb,
                csb,
                T1::zero_prim(),
                c as *mut T1,
                rsc,
                csc,
            );
            out.assume_init()
        }
    }

    pub fn concat<D2: Dim + DefaultMap>(
        &self,
        right: &Array<T1, D2>,
    ) -> Array<T1, ConcatDim<D1, D2>>
    where
        DefaultMappedDim<D1>: nalgebra::DimAdd<DefaultMappedDim<D2>> + nalgebra::Dim,
        DefaultMappedDim<D2>: nalgebra::Dim,
        D2::DefaultMapDim: MapDim<D1>,
        D1::DefaultMapDim: MapDim<D2>,
        D1: DefaultMap,
        AddDim<DefaultMappedDim<D1>, DefaultMappedDim<D2>>: Dim,
        <<D2 as DefaultMap>::DefaultMapDim as MapDim<D1>>::MappedDim: nalgebra::Dim,
        ConcatDim<D1, D2>: Dim,
        <ConcatDim<D1, D2> as ArrayDim>::Buf<MaybeUninit<T1>>:
            ArrayBufUnit<T1, Init = <ConcatDim<D1, D2> as ArrayDim>::Buf<T1>>,
    {
        let d1 = D1::dim(&self.buf);
        let d2 = D2::dim(&right.buf);
        let mut out_dims = d2.clone();
        assert_eq!(d1.as_ref(), d2.as_ref());
        out_dims.as_mut()[0] = d1.as_ref()[0] + d2.as_ref()[0];
        let mut out: Array<MaybeUninit<T1>, ConcatDim<D1, D2>> = Array::uninit(out_dims.as_ref());
        self.buf
            .as_buf()
            .iter()
            .chain(right.buf.as_buf().iter())
            .zip(out.buf.as_mut_buf().iter_mut())
            .for_each(|(a, b)| {
                b.write(*a);
            });
        unsafe { out.assume_init() }
    }

    pub fn concat_many<const N: usize>(args: [&Array<T1, D1>; N]) -> Array<T1, ConcatManyDim<D1, N>>
    where
        DefaultMappedDim<D1>: nalgebra::DimMul<Const<N>> + nalgebra::Dim,
        D1::DefaultMapDim: MapDim<D1>,
        D1::DefaultMapDim: MapDim<D1>,
        D1: DefaultMap,
        MulDim<DefaultMappedDim<D1>, Const<N>>: Dim,
        <<D1 as DefaultMap>::DefaultMapDim as MapDim<D1>>::MappedDim: nalgebra::Dim,
        ConcatManyDim<D1, N>: Dim,
        <ConcatManyDim<D1, N> as ArrayDim>::Buf<MaybeUninit<T1>>:
            ArrayBufUnit<T1, Init = <ConcatManyDim<D1, N> as ArrayDim>::Buf<T1>>,
    {
        let mut out_dims = D1::dim(&args[0].buf);
        for arg in args[1..].iter() {
            let d = D1::dim(&arg.buf);
            out_dims.as_mut()[0] += d.as_ref()[0];
        }
        let mut out: Array<MaybeUninit<T1>, ConcatManyDim<D1, N>> =
            Array::uninit(out_dims.as_ref());
        args.into_iter()
            .flat_map(|a| a.buf.as_buf().iter())
            .zip(out.buf.as_mut_buf().iter_mut())
            .for_each(|(a, b)| {
                b.write(*a);
            });

        unsafe { out.assume_init() }
    }

    pub fn get(&self, index: usize) -> Array<T1, GetDim<D1>>
    where
        ShapeConstraint: DimGet<D1>,
        <GetDim<D1> as ArrayDim>::Buf<MaybeUninit<T1>>:
            ArrayBufUnit<T1, Init = <GetDim<D1> as ArrayDim>::Buf<T1>>,
    {
        let arg_dims = D1::dim(&self.buf);
        let stride_dims = D1::strides(&self.buf);
        let stride = stride_dims.as_ref().last().copied().unwrap();
        let out_dims = &arg_dims.as_ref()[1..];
        let buf = &self.buf.as_buf()[index * stride..];
        let mut out: Array<MaybeUninit<T1>, GetDim<D1>> = Array::uninit(out_dims);
        for (a, b) in buf.iter().zip(out.buf.as_mut_buf().iter_mut()) {
            b.write(*a);
        }
        unsafe { out.assume_init() }
    }
}

type ConcatDim<D1, D2> = ReplaceMappedDim<
    <D2 as DefaultMap>::DefaultMapDim,
    D1,
    AddDim<DefaultMappedDim<D1>, DefaultMappedDim<D2>>,
>;

pub type ConcatManyDim<D1, const N: usize> =
    ReplaceMappedDim<<D1 as DefaultMap>::DefaultMapDim, D1, MulDim<DefaultMappedDim<D1>, Const<N>>>;

pub type MulDim<A, B> = <A as nalgebra::DimMul<B>>::Output;

pub trait DimGet<D: Dim> {
    type Output: Dim;
}

pub type GetDim<D> = <ShapeConstraint as DimGet<D>>::Output;

impl<const N: usize> DimGet<Const<N>> for ShapeConstraint {
    type Output = ();
}

macro_rules! impl_dim_get {
    ($($dim:tt),*) => {
        #[allow(unused_parens)]
        impl<D: Dim, $($dim: Dim),*> DimGet<(D, $($dim,)*)> for ShapeConstraint
        where (D, $($dim,)*): Dim,
        ($($dim,)*): Dim,
        {
            type Output = ($($dim),*);
        }
    };
}

impl_dim_get!(D1);
impl_dim_get!(D1, D2);

fn cobroadcast_dims(output: &mut [usize], other: &[usize]) -> bool {
    for (output, other) in output.iter_mut().zip(other.iter()) {
        if *output == *other || *other == 1 {
            continue;
        }
        if *output == 1 {
            *output = *other;
        } else {
            return false;
        }
    }
    true
}

struct StrideIterator<'a, T, S: AsRef<[usize]>, I: AsMut<[usize]>, D: AsRef<[usize]>> {
    buf: &'a [T],
    stride: S,
    indexes: I,
    dims: D,
    phantom: PhantomData<&'a T>,
}

impl<'a, T, S: AsRef<[usize]>, I: AsMut<[usize]>, D: AsRef<[usize]>> Iterator
    for StrideIterator<'a, T, S, I, D>
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let strides = self.stride.as_ref();
        let indexes = self.indexes.as_mut();
        let dims = self.dims.as_ref();
        let i: usize = indexes
            .iter()
            .zip(strides.iter())
            .map(|(&i, &s): (&usize, &usize)| i * s)
            .sum();
        let mut carry = true;
        for (&dim, index) in dims.iter().zip(indexes.iter_mut()).rev() {
            if carry {
                *index += 1;
            }
            carry = *index >= dim;
            if carry {
                *index = 0;
            }
        }

        self.buf.get(i)
    }
}

pub struct LocalBackend;

impl Repr for LocalBackend {
    type Inner<T, D: Dim> = Array<T, D> where T: Copy;

    fn add<T, D1: ArrayDim, D2: ArrayDim>(
        left: &Self::Inner<T, D1>,
        right: &Self::Inner<T, D2>,
    ) -> Self::Inner<T, BroadcastedDim<D1, D2>>
    where
        T: Add<Output = T> + Copy,
        D1: TensorDim + XlaDim,
        D2: TensorDim + XlaDim,
        ShapeConstraint: BroadcastDim<D1, D2>,
        <ShapeConstraint as BroadcastDim<D1, D2>>::Output: ArrayDim + XlaDim,
        <BroadcastedDim<D1, D2> as ArrayDim>::Buf<MaybeUninit<T>>:
            ArrayBufUnit<T, Init = <BroadcastedDim<D1, D2> as ArrayDim>::Buf<T>>,
    {
        left.add(right)
    }

    fn sub<T, D1, D2>(
        left: &Self::Inner<T, D1>,
        right: &Self::Inner<T, D2>,
    ) -> Self::Inner<T, BroadcastedDim<D1, D2>>
    where
        T: Sub<Output = T> + Copy,
        D1: crate::Dim + ArrayDim,
        D2: crate::Dim + ArrayDim,
        ShapeConstraint: BroadcastDim<D1, D2>,
        <ShapeConstraint as BroadcastDim<D1, D2>>::Output: crate::Dim + ArrayDim,
        <BroadcastedDim<D1, D2> as ArrayDim>::Buf<MaybeUninit<T>>:
            ArrayBufUnit<T, Init = <BroadcastedDim<D1, D2> as ArrayDim>::Buf<T>>,
    {
        left.sub(right)
    }

    fn mul<T, D1, D2>(
        left: &Self::Inner<T, D1>,
        right: &Self::Inner<T, D2>,
    ) -> Self::Inner<T, BroadcastedDim<D1, D2>>
    where
        T: Mul<Output = T> + Copy,
        D1: crate::Dim + ArrayDim,
        D2: crate::Dim + ArrayDim,
        ShapeConstraint: BroadcastDim<D1, D2>,
        <ShapeConstraint as BroadcastDim<D1, D2>>::Output: crate::Dim + ArrayDim,
        <BroadcastedDim<D1, D2> as ArrayDim>::Buf<MaybeUninit<T>>:
            ArrayBufUnit<T, Init = <BroadcastedDim<D1, D2> as ArrayDim>::Buf<T>>,
    {
        left.mul(right)
    }

    fn div<T, D1, D2>(
        left: &Self::Inner<T, D1>,
        right: &Self::Inner<T, D2>,
    ) -> Self::Inner<T, BroadcastedDim<D1, D2>>
    where
        T: Div<Output = T> + Copy,
        D1: crate::Dim + ArrayDim,
        D2: crate::Dim + ArrayDim,
        ShapeConstraint: BroadcastDim<D1, D2>,
        <ShapeConstraint as BroadcastDim<D1, D2>>::Output: crate::Dim + ArrayDim,
        <BroadcastedDim<D1, D2> as ArrayDim>::Buf<MaybeUninit<T>>:
            ArrayBufUnit<T, Init = <BroadcastedDim<D1, D2> as ArrayDim>::Buf<T>>,
    {
        left.div(right)
    }

    fn dot<T: Field, D1, D2>(
        left: &Self::Inner<T, D1>,
        right: &Self::Inner<T, D2>,
    ) -> Self::Inner<T, <ShapeConstraint as crate::DotDim<D1, D2>>::Output>
    where
        T: Div<Output = T> + Copy,
        D1: Dim + ArrayDim,
        D2: Dim + ArrayDim,
        ShapeConstraint: crate::DotDim<D1, D2>,
        <ShapeConstraint as crate::DotDim<D1, D2>>::Output: Dim + ArrayDim,
        <crate::DottedDim<D1, D2> as ArrayDim>::Buf<MaybeUninit<T>>:
            ArrayBufUnit<T, Init = <crate::DottedDim<D1, D2> as ArrayDim>::Buf<T>>,
    {
        left.dot(right)
    }

    fn concat_many<T1: Field, D1: Dim, const N: usize>(
        args: [&Self::Inner<T1, D1>; N],
    ) -> Self::Inner<T1, ConcatManyDim<D1, N>>
    where
        DefaultMappedDim<D1>: nalgebra::DimMul<Const<N>> + nalgebra::Dim,
        D1::DefaultMapDim: MapDim<D1>,
        D1::DefaultMapDim: MapDim<D1>,
        D1: DefaultMap,
        MulDim<DefaultMappedDim<D1>, Const<N>>: Dim,
        <<D1 as DefaultMap>::DefaultMapDim as MapDim<D1>>::MappedDim: nalgebra::Dim,
        ConcatManyDim<D1, N>: Dim,
        <ConcatManyDim<D1, N> as ArrayDim>::Buf<MaybeUninit<T1>>:
            ArrayBufUnit<T1, Init = <ConcatManyDim<D1, N> as ArrayDim>::Buf<T1>>,
    {
        Array::concat_many(args)
    }

    fn get<T1: Field, D1: Dim>(
        arg: &Self::Inner<T1, D1>,
        index: usize,
    ) -> Self::Inner<T1, GetDim<D1>>
    where
        ShapeConstraint: DimGet<D1>,
        <GetDim<D1> as ArrayDim>::Buf<MaybeUninit<T1>>:
            ArrayBufUnit<T1, Init = <GetDim<D1> as ArrayDim>::Buf<T1>>,
    {
        arg.get(index)
    }
}

fn matmul_dims(a: &'_ [usize], b: &'_ [usize]) -> Option<([usize; 2], usize)> {
    let mut out = [0; 2];
    match (a.len(), b.len()) {
        (0, _) => {
            for (out, b) in out.iter_mut().zip(b.iter()) {
                *out = *b
            }
            Some((out, 2))
        }
        (1, 1) => Some((out, 0)),
        (2, 1) => {
            if a[1] != b[0] {
                return None;
            };
            out[0] = a[0];
            Some((out, 1))
        }
        (2, 2) => {
            if a[1] != b[0] {
                return None;
            };
            out[0] = a[0];
            out[1] = b[1];
            Some((out, 2))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_broadcast() {
        let a: Array<f32, Const<1>> = Array { buf: [1.] };
        let b: Array<f32, Const<2>> = Array { buf: [1.0; 2] };
        let c: Array<f32, Const<2>> = a.add(&b);
        assert_eq!(c.buf, [2.0; 2]);

        let a: Array<f32, (Const<1>, Const<2>)> = Array { buf: [[1.0, 2.0]] };
        let b: Array<f32, (Const<2>, Const<2>)> = Array {
            buf: [[1.0, 1.0], [2.0, 2.0]],
        };
        let c: Array<f32, (Const<2>, Const<2>)> = a.add(&b);
        assert_eq!(c.buf, [[2.0, 3.0], [3.0, 4.0]]);

        let a: Array<f32, (Const<2>, Const<1>, Const<2>)> = Array {
            buf: [[[1.0, 2.0]], [[1.0, 2.0]]],
        };
        let b: Array<f32, (Const<2>, Const<2>, Const<2>)> = Array {
            buf: [[[1.0, 1.0], [2.0, 2.0]], [[1.0, 1.0], [2.0, 2.0]]],
        };
        let c: Array<f32, (Const<2>, Const<2>, Const<2>)> = a.add(&b);
        assert_eq!(c.buf, [[[2.0, 3.0], [3.0, 4.0]], [[2.0, 3.0], [3.0, 4.0]]]);
    }

    #[test]
    fn test_matmul_broadcast() {
        let a: Array<f32, (Const<2>, Const<2>)> = Array {
            buf: [[0.0, 1.0], [4.0, 2.0]],
        };

        let b: Array<f32, (Const<2>, Const<2>)> = Array {
            buf: [[1.0, 1.0], [2.0, 2.0]],
        };
        let c: Array<f32, (Const<2>, Const<2>)> = a.dot(&b);
        assert_eq!(c.buf, [[2.0, 2.0], [8.0, 8.0]]);

        let a: Array<f32, (Const<3>, Const<3>)> = Array {
            buf: [[1.0, 1.0, 1.0], [2.0, 2.0, 2.0], [3.0, 3.0, 3.0]],
        };
        let b: Array<f32, (Const<3>, Const<1>)> = Array {
            buf: [[0.0], [1.0], [1.0]],
        };
        let c: Array<f32, (Const<3>, Const<1>)> = a.dot(&b);
        assert_eq!(c.buf, [[2.0], [4.0], [6.0]])
    }

    #[test]
    fn test_concat() {
        let a: Array<f32, Const<2>> = Array { buf: [0.0, 1.0] };

        let b: Array<f32, Const<2>> = Array { buf: [2.0, 3.0] };
        let c: Array<f32, Const<4>> = a.concat(&b);
        assert_eq!(c.buf, [0., 1., 2., 3.]);

        let a: Array<f32, (Const<2>, Const<2>)> = Array {
            buf: [[0.0, 1.0], [4.0, 2.0]],
        };

        let b: Array<f32, (Const<2>, Const<2>)> = Array {
            buf: [[1.0, 1.0], [2.0, 2.0]],
        };
        let c: Array<f32, (Const<4>, Const<2>)> = a.concat(&b);
        assert_eq!(c.buf, [[0., 1.], [4., 2.], [1., 1.], [2., 2.]]);

        let a: Array<f32, Const<1>> = Array { buf: [1.0] };
        let b: Array<f32, Const<1>> = Array { buf: [2.0] };
        let c: Array<f32, Const<1>> = Array { buf: [3.0] };
        let d: Array<f32, Const<3>> = Array::concat_many([&a, &b, &c]);
        assert_eq!(d.buf, [1.0, 2.0, 3.0]);
    }
}
