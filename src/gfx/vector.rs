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
pub trait Cast<T> {
    fn cast(self) -> T;
}

pub trait ToIntUnchecked<T> {
    unsafe fn to_int_unchecked(self) -> T;
}

macro_rules! impl_cast_trait {
    () => {
        impl_cast_trait!(u8 as int);
        impl_cast_trait!(i8 as int);
        impl_cast_trait!(u16 as int);
        impl_cast_trait!(i16 as int);
        impl_cast_trait!(u32 as int);
        impl_cast_trait!(i32 as int);
        impl_cast_trait!(u64 as int);
        impl_cast_trait!(i64 as int);
        impl_cast_trait!(f32 as float);
        impl_cast_trait!(f64 as float);
        impl_cast_trait!(usize as int);
        impl_cast_trait!(isize as int);
    };
    ($from:ty as int) => {
        impl_cast_trait!($from);

        impl ToIntUnchecked<$from> for f32 {
            unsafe fn to_int_unchecked(self) -> $from {
                f32::to_int_unchecked(self)
            }
        }
        impl ToIntUnchecked<$from> for f64 {
            unsafe fn to_int_unchecked(self) -> $from {
                f64::to_int_unchecked(self)
            }
        }
    };
    ($from:ty as float) => {
        impl_cast_trait!($from);
    };
    ($from:ty) => {
        impl_cast_trait!($from => u8);
        impl_cast_trait!($from => i8);
        impl_cast_trait!($from => u16);
        impl_cast_trait!($from => i16);
        impl_cast_trait!($from => u32);
        impl_cast_trait!($from => i32);
        impl_cast_trait!($from => u64);
        impl_cast_trait!($from => i64);
        impl_cast_trait!($from => f32);
        impl_cast_trait!($from => f64);
        impl_cast_trait!($from => usize);
        impl_cast_trait!($from => isize);
    };
    ($from:ty => $to:ty) => {
        impl Cast<$to> for $from {
            fn cast(self) -> $to {
                self as $to
            }
        }
    };
}

impl_cast_trait!();

#[macro_export]
macro_rules! impl_vector_overload {
    ($struct:ident $x:ident $y:ident) => (
        impl<T: Copy> $struct<T> {
            pub const fn new($x: T, $y: T) -> $struct<T> {
                $struct { $x, $y }
            }

            pub const fn splat(value: T) -> Self {
                Self::new(value, value)
            }

            pub const fn to_array(&self) -> [T; 2] {
                [self.$x, self.$y]
            }

            pub fn iter(&self) -> std::array::IntoIter<T, 2> {
                self.to_array().into_iter()
            }
        }

        impl<T: Copy> Vector2<T> for $struct<T> {
            fn x(&self) -> T {
                self.$x
            }
            fn y(&self) -> T {
                self.$y
            }
        }

        impl<T: Copy> std::iter::FromIterator<T> for $struct<T> {
            fn from_iter<I>(iter: I) -> $struct<T>
            where
                I: IntoIterator<Item = T>
            {
                let mut iter = iter.into_iter();
                let expect = "initialized a vector with a small iter";

                Self::new(
                    iter.next().expect(expect),
                    iter.next().expect(expect)
                )
            }
        }

        impl<T: Copy> From<T> for $struct<T> {
            fn from(value: T) -> Self {
                Self::new(value, value)
            }
        }

        impl<T: Copy> From<(T, T)> for $struct<T> {
            fn from((x, y): (T, T)) -> Self {
                Self::new(x, y)
            }
        }

        impl<T: Copy> From<[T; 2]> for $struct<T> {
            fn from(array: [T; 2]) -> Self {
                Self::new(array[0], array[1])
            }
        }

        crate::impl_vector_traits!($struct Vector2);
    );
    ($struct:ident $x:ident $y:ident $z:ident) => (
        impl<T: Copy> $struct<T> {
            pub const fn new($x: T, $y: T, $z: T) -> $struct<T> {
                $struct { $x, $y, $z }
            }

            pub const fn splat(value: T) -> Self {
                Self::new(value, value, value)
            }

            pub const fn to_array(&self) -> [T; 3] {
                [self.$x, self.$y, self.$z]
            }

            pub fn iter(&self) -> std::array::IntoIter<T, 3> {
                self.to_array().into_iter()
            }
        }

        impl<T: Copy> Vector3<T> for $struct<T> {
            fn x(&self) -> T {
                self.$x
            }
            fn y(&self) -> T {
                self.$y
            }
            fn z(&self) -> T {
                self.$z
            }
        }

        impl<T: Copy> std::iter::FromIterator<T> for $struct<T> {
            fn from_iter<I>(iter: I) -> $struct<T>
            where
                I: IntoIterator<Item = T>
            {
                let mut iter = iter.into_iter();
                let expect = "initialized a vector with a small iter";

                Self::new(
                    iter.next().expect(expect),
                    iter.next().expect(expect),
                    iter.next().expect(expect)
                )
            }
        }

        impl<T: Copy> From<T> for $struct<T> {
            fn from(value: T) -> Self {
                Self::new(value, value, value)
            }
        }

        impl<T: Copy> From<(T, T, T)> for $struct<T> {
            fn from((x, y, z): (T, T, T)) -> Self {
                Self::new(x, y, z)
            }
        }

        impl<T: Copy> From<[T; 3]> for $struct<T> {
            fn from(array: [T; 3]) -> Self {
                Self::new(array[0], array[1], array[2])
            }
        }

        crate::impl_vector_traits!($struct Vector3);
    );
}

#[macro_export]
macro_rules! impl_vector_traits {
    ($struct:ident $vector:ident) => {
        impl<T: Copy> $struct<T> {
            pub fn dot<U>(&self, rhs: U) -> T
            where
                U: Into<$struct<T>>,
                T: std::ops::Mul<T, Output = T> + std::iter::Sum,
            {
                (self * rhs.into()).sum()
            }

            pub fn sum(&self) -> T
            where
                T: std::iter::Sum,
            {
                self.iter().sum::<T>()
            }

            pub fn cast<U>(&self) -> $struct<U>
            where
                T: super::Cast<U>,
                U: Copy
            {
                self.map(|v| v.cast())
            }

            pub fn map<U, F>(&self, f: F) -> $struct<U>
            where
                U: Copy,
                F: FnMut(T) -> U
            {
                self.iter().map(f).collect()
            }

            pub fn reduce<F>(&self, f: F) -> T
            where
                T: Default,
                F: FnMut(T, T) -> T
            {
                self.iter().fold(<T as Default>::default(), f)
            }

            pub fn min_val(&self) -> T
            where
                T: Default + Ord
            {
                self.reduce(|a, b| a.min(b))
            }

            pub fn max_val(&self) -> T
            where
                T: Default + Ord
            {
                self.reduce(|a, b| a.max(b))
            }
        }

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

        crate::impl_vector_traits!($struct $vector Add add);
        crate::impl_vector_traits!($struct $vector Sub sub);
        crate::impl_vector_traits!($struct $vector Mul mul);
        crate::impl_vector_traits!($struct $vector Div div);
        crate::impl_vector_traits!($struct $vector BitOr bitor);
        crate::impl_vector_traits!($struct $vector BitXor bitxor);
        crate::impl_vector_traits!($struct $vector BitAnd bitand);
    };
    ($struct:ident $vector:ident $type:ident) => (
        impl $struct<$type> {
            pub fn avg_with<T>(&self, rhs: T) -> Self
            where
                T: Into<Self>
            {
                let rhs = rhs.into();

                (self & rhs) + (self ^ rhs) / 2
            }
        }
    );
    ($struct:ident $vector:ident $type:ident float) => (
        impl $struct<$type> {
            pub unsafe fn to_int_unchecked<U>(&self) -> $struct<U>
            where
                $type: super::ToIntUnchecked<U>,
                U: Copy
            {
                self.map(|x| <$type as super::ToIntUnchecked<U>>::to_int_unchecked(x))
            }

            pub fn mul_add<M, A>(&self, mul: M, add: A) -> Self
            where
                M: Into<Self>,
                A: Into<Self>,
            {
                self.iter()
                    .zip(mul.into().iter())
                    .zip(add.into().iter())
                    .map(|((x, y), z)| x.mul_add(y, z))
                    .collect()
            }

            pub fn round(&self) -> Self {
                self.map(|v| v.round())
            }

            pub fn min<U>(&self, min: U) -> Self
            where
                U: Into<Self>
            {
                self.iter()
                    .zip(min.into().iter())
                    .map(|(x, y)| x.min(y))
                    .collect()
            }

            pub fn max<U>(&self, max: U) -> Self
            where
                U: Into<Self>
            {
                self.iter()
                    .zip(max.into().iter())
                    .map(|(x, y)| x.max(y))
                    .collect()
            }

            pub fn clamp<U>(&self, min: U, max: U) -> Self
            where
                U: Into<Self>
            {
                self.iter()
                    .zip(min.into().iter())
                    .zip(max.into().iter())
                    .map(|((x, y), z)| x.clamp(y, z))
                    .collect()
            }
        }
    );
    ($struct:ident $vector:ident $trait:ident $name:ident) => {
        impl<T: Copy> $struct<T> {
            pub fn $name<U>(&self, rhs: U) -> Self
            where
                T: std::ops::$trait<T, Output = T>,
                U: Copy + Into<Self>
            {
                self.iter()
                    .zip(rhs.into().iter())
                    .map(|(x, y)| x.$name(y))
                    .collect()
            }
        }

        impl<T, U> std::ops::$trait<U> for $struct<T>
        where
            T: Copy + std::ops::$trait<T, Output = T>,
            U: Copy + Into<$struct<T>>,
        {
            type Output = $struct<T>;

            fn $name(self, rhs: U) -> Self::Output {
                $struct::$name(&self, rhs)
            }
        }

        impl<'a, T, U> std::ops::$trait<U> for &'a $struct<T>
        where
            T: Copy + std::ops::$trait<T, Output = T>,
            U: Copy + Into<$struct<T>>,
        {
            type Output = $struct<T>;

            fn $name(self, rhs: U) -> Self::Output {
                $struct::$name(&self, rhs)
            }
        }
    };
}
