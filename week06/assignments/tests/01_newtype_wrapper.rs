//! Run this file with `cargo test --test 01_newtype_wrapper`.

//! implement a declarative macro named `define_id_type`, which will create a newtype that
//! wraps an inner type.
//! The created type should be copyable, comparable, hashable, formattable (both `Debug` and
//! `Display`) and it should also be possible to create it from a value of the inner type using
//! `From`.
//! The created type should have a constructor called `new` that creates it from a value of the inner
//! type and a method called `as_inner`, which will return the inner type.
//!
//! If you invoke the macro with a single argument, it should create a type with the given name that
//! wraps `u32`.
//! If you pass two arguments to it, the second argument will determine the inner type.
//!
//! The macro should be hygienic - in particular, it should not assume that certain traits or types
//! are available within the scope where the macro will be used.
#![allow(unused)]

macro_rules! define_id_type {
    ($name:ident) => {
        define_id_type!($name, u32);
    };
    ($name:ident, $inner:ty) => {
        #[derive(
            ::core::marker::Copy,
            ::core::clone::Clone,
            ::core::cmp::Eq,
            ::core::cmp::PartialEq,
            ::core::hash::Hash,
            ::core::fmt::Debug,
        )]
        struct $name($inner);
        impl $name {
            fn new(value: $inner) -> Self {
                $name(value)
            }

            fn as_inner(&self) -> $inner {
                self.0
            }
        }

        impl ::core::convert::From<$inner> for $name {
            fn from(value: $inner) -> Self {
                $name(value)
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(
                &self,
                f: &mut ::std::fmt::Formatter<'_>,
            ) -> ::std::result::Result<(), ::std::fmt::Error> {
                ::core::write!(f, "{}", self.0)
            }
        }
    };
}

/// Below you can find a set of unit tests.
#[cfg(test)]
mod tests {
    use std::hash::Hash;

    #[test]
    fn default_type_u32() {
        define_id_type!(CarId);
        let c = CarId::new(5u32);
        assert_eq!(c.as_inner(), 5u32);
    }

    #[test]
    fn custom_type() {
        define_id_type!(CarId, u64);
        CarId::new(5u64);

        define_id_type!(Speed, u8);
        Speed::new(50u8);
    }

    #[test]
    fn derived_traits() {
        define_id_type!(CarId, u64);
        let a = CarId::new(0);
        let b = CarId::new(0);
        let c = a;

        assert_eq!(format!("{a:?}"), "CarId(0)");
        assert_eq!(a, b);
        assert_ne!(a, CarId::new(5));
    }

    #[test]
    fn hash() {
        fn take_hash<H: Hash>(_: H) {}

        define_id_type!(Foo);

        take_hash(Foo::new(42));
    }

    #[test]
    fn from() {
        define_id_type!(DriverId, u64);
        let c: DriverId = 5u64.into();
        assert_eq!(c.as_inner(), 5u64);
    }

    #[test]
    fn display() {
        define_id_type!(CarId, u64);
        assert_eq!(&format!("{}", CarId::new(42)), "42");
    }

    #[test]
    fn test_hygiene() {
        trait From {
            fn not_me(&self);
        }

        mod std {
            pub mod convert {
                pub trait From {
                    fn still_nope(&self);
                }
            }
            pub mod fmt {
                trait Display {
                    fn not_me_either(&self);
                }
                struct Result<NOTME>(NOTME);
                struct Formatter;
            }
        }

        struct Formatter;
        trait Display {
            fn no_fmt_here(&self);
        }
        struct Result<NOPE>(NOPE);

        define_id_type!(CarId, u64);
        CarId::new(5u64);
    }
}
