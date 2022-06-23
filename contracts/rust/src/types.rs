use ark_ff::{to_bytes, PrimeField, Zero};
use ark_pallas::{Affine, Fq};
use ethers::prelude::*;

abigen!(
    AssetRegistry,
    "../abi/contracts/AssetRegistry.sol/AssetRegistry/abi.json",
    event_derives(serde::Deserialize, serde::Serialize);

    TestPallas,
    "../abi/contracts/mocks/TestPallas.sol/TestPallas/abi.json",
    event_derives(serde::Deserialize, serde::Serialize);

    Greeter,
    "../abi/contracts/Greeter.sol/Greeter/abi.json",
    event_derives(serde::Deserialize, serde::Serialize);
);

impl From<Affine> for Point {
    fn from(p: Affine) -> Self {
        if p.is_zero() {
            // Solidity precompile have a different affine repr for Point of Infinity
            Self {
                x: U256::from(0),
                y: U256::from(0),
            }
        } else {
            Self {
                x: U256::from_little_endian(&to_bytes!(p.x).unwrap()[..]),
                y: U256::from_little_endian(&to_bytes!(p.y).unwrap()[..]),
            }
        }
    }
}

impl From<(Fq, Fq)> for Point {
    fn from(p: (Fq, Fq)) -> Self {
        let zero = Affine::zero();
        if p.0 == zero.x && p.1 == zero.y {
            // Solidity repr of infinity/zero
            Self {
                x: U256::from(0),
                y: U256::from(0),
            }
        } else {
            Self {
                x: U256::from_little_endian(&to_bytes!(p.0).unwrap()[..]),
                y: U256::from_little_endian(&to_bytes!(p.1).unwrap()[..]),
            }
        }
    }
}

impl From<Point> for Affine {
    fn from(p_sol: Point) -> Self {
        if p_sol.x.is_zero() && p_sol.y.is_zero() {
            Self::zero()
        } else {
            Self::new(u256_to_field(p_sol.x), u256_to_field(p_sol.y), false)
        }
    }
}

/// convert a field element (at most BigInteger256).
pub fn field_to_u256<F: PrimeField>(f: F) -> U256 {
    if F::size_in_bits() > 256 {
        panic!("Don't support field size larger than 256 bits.");
    }
    U256::from_little_endian(&to_bytes!(&f).unwrap())
}

/// convert a U256 to a field element.
pub fn u256_to_field<F: PrimeField>(v: U256) -> F {
    let mut bytes = vec![0u8; 32];
    v.to_little_endian(&mut bytes);
    F::from_le_bytes_mod_order(&bytes)
}

/// a helper trait to help with fully-qualified generic into synatx:
/// `x.generic_into::<DestType>();`
/// This is particularly helpful in a chained `generic_into()` statements.
pub trait GenericInto {
    fn generic_into<T>(self) -> T
    where
        Self: Into<T>,
    {
        self.into()
    }
}

// blanket implementation
impl<T: ?Sized> GenericInto for T {}

#[cfg(test)]
mod test {
    use super::*;
    use ark_ff::field_new;
    use ark_pallas::{Affine, Fq, Fr};
    use ark_std::UniformRand;

    #[test]
    fn field_types_conversion() {
        let rng = &mut ark_std::test_rng();
        let f1 = Fr::rand(rng);
        let f2 = Fq::rand(rng);
        // trivial test, prevent accidental change to the function
        assert_eq!(
            field_to_u256(f1),
            U256::from_little_endian(&to_bytes!(f1).unwrap())
        );
        assert_eq!(
            field_to_u256(f2),
            U256::from_little_endian(&to_bytes!(f2).unwrap())
        );

        assert_eq!(f1, u256_to_field(field_to_u256(f1)));
        assert_eq!(f2, u256_to_field(field_to_u256(f2)));
    }

    #[test]
    fn group_types_conversion() {
        // special case: point of infinity (zero)
        let p1 = Affine::default();
        let p1_sol: Point = p1.into();
        assert_eq!(p1_sol.x, U256::from(0));
        assert_eq!(p1_sol.y, U256::from(0));
        assert_eq!(p1, p1_sol.generic_into::<Affine>());

        // a point (not on the curve, which doesn't matter since we only check conversion)
        let p2 = Affine::new(field_new!(Fq, "12345"), field_new!(Fq, "2"), false);
        let p2_sol: Point = p2.into();
        assert_eq!(p2_sol.x, U256::from(12345));
        assert_eq!(p2_sol.y, U256::from(2));
        assert_eq!(p2, p2_sol.generic_into::<Affine>());
    }
}
