use std::simd::{self};

pub trait Vector2<T>
where
    T: Copy,
{
    fn x(&self) -> T;
    fn y(&self) -> T;
}

pub trait Vector3<T>
where
    T: Copy,
{
    fn x(&self) -> T;
    fn y(&self) -> T;
    fn z(&self) -> T;
}

pub trait VectorElement {
    type Vector2;
    type Vector3;
}

impl VectorElement for u8 {
    type Vector2 = simd::u8x4;
    type Vector3 = simd::u8x4;
}
impl VectorElement for i8 {
    type Vector2 = simd::i8x4;
    type Vector3 = simd::i8x4;
}
impl VectorElement for u16 {
    type Vector2 = simd::u16x2;
    type Vector3 = simd::u16x4;
}
impl VectorElement for i16 {
    type Vector2 = simd::i16x2;
    type Vector3 = simd::i16x4;
}
impl VectorElement for u32 {
    type Vector2 = simd::u32x2;
    type Vector3 = simd::u32x4;
}
impl VectorElement for i32 {
    type Vector2 = simd::i32x2;
    type Vector3 = simd::i32x4;
}
impl VectorElement for u64 {
    type Vector2 = simd::u64x2;
    type Vector3 = simd::u64x4;
}
impl VectorElement for i64 {
    type Vector2 = simd::i64x2;
    type Vector3 = simd::i64x4;
}
impl VectorElement for f32 {
    type Vector2 = simd::f32x2;
    type Vector3 = simd::f32x4;
}
impl VectorElement for f64 {
    type Vector2 = simd::f64x2;
    type Vector3 = simd::f64x4;
}
impl VectorElement for usize {
    type Vector2 = simd::usizex2;
    type Vector3 = simd::usizex4;
}
impl VectorElement for isize {
    type Vector2 = simd::isizex2;
    type Vector3 = simd::isizex4;
}

pub trait CreateVector2<T> {
    fn create(a: T, b: T) -> Self;
}
pub trait CreateVector3<T> {
    fn create(a: T, b: T, z: T) -> Self;
}

#[macro_export]
macro_rules! impl_vector_overload {
    ($struct:ident $x:ident $y:ident = $default:ty) => (
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct $struct<T: Copy + super::VectorElement = $default> {
            simd: T::Vector2
        }

        impl<T: Copy + super::VectorElement> $struct<T> {
            pub fn new($x: T, $y: T) -> $struct<T>
            where
                $struct<T>: super::Vector2<T>,
                $struct<T>: super::CreateVector2<T>,
            {
                <$struct<T> as super::CreateVector2<T>>::create($x, $y)
            }
        }

        crate::impl_simd2_overload!($struct $x $y u8 4);
        crate::impl_simd2_overload!($struct $x $y i8 4);
        crate::impl_simd2_overload!($struct $x $y u16 2);
        crate::impl_simd2_overload!($struct $x $y i16 2);
        crate::impl_simd2_overload!($struct $x $y u32 2);
        crate::impl_simd2_overload!($struct $x $y i32 2);
        crate::impl_simd2_overload!($struct $x $y u64 2);
        crate::impl_simd2_overload!($struct $x $y i64 2);
        crate::impl_simd2_overload!($struct $x $y f32 2);
        crate::impl_simd2_overload!($struct $x $y f64 2);
        crate::impl_simd2_overload!($struct $x $y usize 2);
        crate::impl_simd2_overload!($struct $x $y isize 2);

        crate::impl_vector_traits!($struct Vector2);
    );
    ($struct:ident $x:ident $y:ident $z:ident = $default:ty) => (
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct $struct<T: Copy = $default>
        where
            T: super::VectorElement
        {
            simd: T::Vector3
        }

        impl<T: Copy + super::VectorElement> $struct<T> {
            pub fn new($x: T, $y: T, $z: T) -> $struct<T>
            where
                $struct<T>: super::Vector3<T>,
                $struct<T>: super::CreateVector3<T>,
            {
                <$struct<T> as super::CreateVector3<T>>::create($x, $y, $z)
            }
        }

        crate::impl_simd3_overload!($struct $x $y $z u8 4);
        crate::impl_simd3_overload!($struct $x $y $z i8 4);
        crate::impl_simd3_overload!($struct $x $y $z u16 4);
        crate::impl_simd3_overload!($struct $x $y $z i16 4);
        crate::impl_simd3_overload!($struct $x $y $z u32 4);
        crate::impl_simd3_overload!($struct $x $y $z i32 4);
        crate::impl_simd3_overload!($struct $x $y $z u64 4);
        crate::impl_simd3_overload!($struct $x $y $z i64 4);
        crate::impl_simd3_overload!($struct $x $y $z f32 4);
        crate::impl_simd3_overload!($struct $x $y $z f64 4);
        crate::impl_simd3_overload!($struct $x $y $z usize 4);
        crate::impl_simd3_overload!($struct $x $y $z isize 4);

        crate::impl_vector_traits!($struct Vector3);
    );
}

#[macro_export]
macro_rules! impl_vector_traits {
    ($struct:ident $vector:ident) => {
        crate::impl_vector_traits!($struct $vector i8);
        crate::impl_vector_traits!($struct $vector u8);
        crate::impl_vector_traits!($struct $vector i16);
        crate::impl_vector_traits!($struct $vector u16);
        crate::impl_vector_traits!($struct $vector i32);
        crate::impl_vector_traits!($struct $vector u32);
        crate::impl_vector_traits!($struct $vector i64);
        crate::impl_vector_traits!($struct $vector u64);
        crate::impl_vector_traits!($struct $vector isize);
        crate::impl_vector_traits!($struct $vector usize);
        crate::impl_vector_traits!($struct $vector f32 float);
        crate::impl_vector_traits!($struct $vector f64 float);
    };
    ($struct:ident $vector:ident $type:ident) => (
        crate::impl_vector_traits!($struct $vector $type ops);
        crate::impl_vector_traits!($struct $vector BitOr bitor $type);
        crate::impl_vector_traits!($struct $vector BitXor bitxor $type);
        crate::impl_vector_traits!($struct $vector BitAnd bitand $type);

        impl $struct<$type>
        where
            $type: super::VectorElement
        {
            pub fn avg_with<T>(self, rhs: T) -> Self
            where
                T: Into<$struct<$type>>
            {
                let rhs = rhs.into();

                (self & rhs) + (self ^ rhs) / 2
            }
        }
    );
    ($struct:ident $vector:ident $type:ident float) => (
        crate::impl_vector_traits!($struct $vector $type ops);

        impl $struct<$type>
        where
            $type: super::VectorElement
        {
            pub fn mul_add<A, B>(self, a: A, b: B) -> Self
            where
                A: Into<$struct<$type>>,
                B: Into<$struct<$type>>,
            {
                std::simd::StdFloat::mul_add(
                    self.to_simd(),
                    a.into().to_simd(),
                    b.into().to_simd(),
                ).into()
            }

            pub fn round(&self) -> Self {
                std::simd::StdFloat::round(
                    self.to_simd()
                ).into()
            }

            pub fn min<T>(&self, min: T) -> Self
            where
                T: Into<$struct<$type>>
            {
                std::simd::SimdFloat::simd_min(
                    self.to_simd(),
                    min.into().to_simd()
                ).into()
            }

            pub fn max<T>(&self, max: T) -> Self
            where
                T: Into<$struct<$type>>
            {
                std::simd::SimdFloat::simd_max(
                    self.to_simd(),
                    max.into().to_simd()
                ).into()
            }

            pub fn clamp<T>(&self, min: T, max: T) -> Self
            where
                T: Into<$struct<$type>>
            {
                std::simd::SimdFloat::simd_clamp(
                    self.to_simd(),
                    min.into().to_simd(),
                    max.into().to_simd()
                ).into()
            }
        }
    );
    ($struct:ident $vector:ident $type:ident ops) => (
        crate::impl_vector_traits!($struct $vector Add add $type);
        crate::impl_vector_traits!($struct $vector Sub sub $type);
        crate::impl_vector_traits!($struct $vector Mul mul $type);
        crate::impl_vector_traits!($struct $vector Div div $type);

        impl $struct<$type>
        where
            $type: super::VectorElement
        {
            pub fn dot<T>(self, rhs: T) -> $type
            where
                T: Into<$struct<$type>>
            {
                (self * rhs).sum()
            }
        }
    );
    ($struct:ident $vector:ident $trait:ident $name:ident $type:ident) => {
        impl<T> std::ops::$trait<T> for $struct<$type>
        where
            T: Into<$struct<$type>>
        {
            type Output = $struct<$type>;

            fn $name(self, rhs: T) -> Self::Output {
                self.to_simd().$name(rhs.into().to_simd()).into()
            }
        }
    };
}

#[macro_export]
macro_rules! impl_simd2_overload {
    ($struct:ident $x:ident $y:ident $type:ident $lanes:expr) => {
        impl super::CreateVector2<$type> for $struct<$type> {
            fn create($x: $type, $y: $type) -> $struct<$type> {
                $struct::<$type>::create($x, $y)
            }
        }

        impl $struct<$type> {
            pub const fn create($x: $type, $y: $type) -> $struct<$type> {
                let mut array = [1 as $type; $lanes];

                array[0] = $x;
                array[1] = $y;

                $struct {
                    simd: std::simd::Simd::from_array(array),
                }
            }

            pub const fn splat(value: $type) -> Self {
                Self::create(value, value)
            }

            pub const fn $x(self) -> $type {
                self.simd.as_array()[0]
            }
            pub const fn $y(self) -> $type {
                self.simd.as_array()[1]
            }

            pub fn sum(self) -> $type
            where
                $type: std::ops::Add<$type, Output = $type>,
            {
                self.$x() + self.$y()
            }
        }

        impl Vector2<$type> for $struct<$type> {
            fn x(&self) -> $type {
                $struct::<$type>::$x(*self)
            }
            fn y(&self) -> $type {
                $struct::<$type>::$y(*self)
            }
        }

        impl From<$type> for $struct<$type> {
            fn from(value: $type) -> $struct<$type> {
                $struct::<$type>::create(value, value)
            }
        }

        impl From<[$type; $lanes]> for $struct<$type> {
            fn from(array: [$type; $lanes]) -> $struct<$type> {
                $struct::<$type>::create(array[0], array[1])
            }
        }

        impl From<($type, $type)> for $struct<$type> {
            fn from(tuple: ($type, $type)) -> $struct<$type> {
                $struct::<$type>::create(tuple.0, tuple.1)
            }
        }

        impl Into<std::simd::Simd<$type, $lanes>> for $struct<$type> {
            fn into(self) -> std::simd::Simd<$type, $lanes> {
                self.to_simd()
            }
        }

        impl Into<$struct<$type>> for std::simd::Simd<$type, $lanes> {
            fn into(self) -> $struct<$type> {
                $struct::<$type>::from_simd(self)
            }
        }

        impl $struct<$type> {
            pub const fn from_simd(simd: std::simd::Simd<$type, $lanes>) -> Self {
                $struct { simd }
            }

            pub const fn to_simd(&self) -> std::simd::Simd<$type, $lanes> {
                self.simd
            }

            pub fn cast<U>(&self) -> $struct<U>
            where
                U: std::simd::SimdElement + super::VectorElement,
                std::simd::Simd<U, $lanes>: Into<$struct<U>>,
            {
                self.to_simd().cast().into()
            }
        }
    };
}

#[macro_export]
macro_rules! impl_simd3_overload {
    ($struct:ident $x:ident $y:ident $z:ident $type:ident $lanes:expr) => {
        impl super::CreateVector3<$type> for $struct<$type> {
            fn create($x: $type, $y: $type, $z: $type) -> $struct<$type> {
                $struct::<$type>::create($x, $y, $z)
            }
        }
        impl $struct<$type> {
            pub const fn create($x: $type, $y: $type, $z: $type) -> $struct<$type> {
                let mut array = [1 as $type; $lanes];

                array[0] = $x;
                array[1] = $y;
                array[2] = $z;

                $struct {
                    simd: std::simd::Simd::from_array(array),
                }
            }

            pub const fn splat(value: $type) -> Self {
                Self::create(value, value, value)
            }

            pub const fn $x(self) -> $type {
                self.simd.as_array()[0]
            }
            pub const fn $y(self) -> $type {
                self.simd.as_array()[1]
            }
            pub const fn $z(self) -> $type {
                self.simd.as_array()[2]
            }

            pub fn sum(self) -> $type
            where
                $type: std::ops::Add<$type, Output = $type>,
            {
                self.$x() + self.$y() + self.$z()
            }
        }

        impl Vector3<$type> for $struct<$type> {
            fn x(&self) -> $type {
                self.$x()
            }
            fn y(&self) -> $type {
                self.$y()
            }
            fn z(&self) -> $type {
                self.$z()
            }
        }

        impl From<$type> for $struct<$type> {
            fn from(value: $type) -> $struct<$type> {
                $struct::<$type>::create(value, value, value)
            }
        }

        impl From<[$type; $lanes]> for $struct<$type> {
            fn from(array: [$type; $lanes]) -> $struct<$type> {
                $struct::<$type>::create(array[0], array[1], array[2])
            }
        }

        impl From<($type, $type, $type)> for $struct<$type> {
            fn from(tuple: ($type, $type, $type)) -> $struct<$type> {
                $struct::<$type>::create(tuple.0, tuple.1, tuple.2)
            }
        }

        impl Into<std::simd::Simd<$type, $lanes>> for $struct<$type> {
            fn into(self) -> std::simd::Simd<$type, $lanes> {
                self.to_simd()
            }
        }

        impl Into<$struct<$type>> for std::simd::Simd<$type, $lanes> {
            fn into(self) -> $struct<$type> {
                $struct::<$type>::from_simd(self)
            }
        }

        impl $struct<$type> {
            pub const fn from_simd(simd: std::simd::Simd<$type, $lanes>) -> Self {
                $struct { simd }
            }

            pub const fn to_simd(self) -> std::simd::Simd<$type, $lanes> {
                self.simd
            }

            pub fn cast<U>(self) -> $struct<U>
            where
                U: std::simd::SimdElement + super::VectorElement,
                std::simd::Simd<U, $lanes>: Into<$struct<U>>,
            {
                self.to_simd().cast().into()
            }
        }
    };
}
