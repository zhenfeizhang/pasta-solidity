#![cfg(test)]

use crate::{
    assertion::Matcher,
    ethereum::{deploy, get_funded_deployer},
    types::{field_to_u256, Point, TestPallas},
};
use anyhow::Result;
use ark_ec::msm::VariableBaseMSM;
use ark_ec::AffineCurve;
use ark_ec::{group::Group, ProjectiveCurve};
use ark_ff::{field_new, to_bytes, Field};
use ark_ff::{FpParameters, PrimeField};
use ark_pallas::{Affine, Fq, Fr, Projective};
use ark_std::UniformRand;
use ark_std::Zero;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::prelude::*;
use rand::RngCore;
use std::path::Path;

async fn deploy_contract(
) -> Result<TestPallas<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>> {
    let client = get_funded_deployer().await.unwrap();
    let contract = deploy(
        client.clone(),
        Path::new("../abi/contracts/mocks/TestPallas.sol/TestPallas"),
        (),
    )
    .await
    .unwrap();
    Ok(TestPallas::new(contract.address(), client))
}

#[tokio::test]
async fn test_add() -> Result<()> {
    let rng = &mut ark_std::test_rng();
    let contract = deploy_contract().await?;

    // test random group addition
    for _ in 0..10 {
        let p1: Affine = Projective::rand(rng).into();
        let p2: Affine = Projective::rand(rng).into();
        let res: Point = contract.add(p1.into(), p2.into()).call().await?.into();
        assert_eq!(res, (p1 + p2).into());
    }

    // test point of infinity, O_E + P = P
    let zero = Affine::zero();
    let p: Affine = Projective::rand(rng).into();
    let res: Point = contract.add(p.into(), zero.into()).call().await?.into();
    assert_eq!(res, p.into());

    Ok(())
}

#[tokio::test]
async fn test_group_generators() -> Result<()> {
    let contract = deploy_contract().await?;

    let g1_gen = Affine::prime_subgroup_generator();
    let g1_gen_sol: Point = contract.p1().call().await?.into();
    assert_eq!(g1_gen_sol, g1_gen.into());

    Ok(())
}

#[tokio::test]
async fn test_is_infinity() -> Result<()> {
    let rng = &mut ark_std::test_rng();
    let contract = deploy_contract().await?;

    let zero = Affine::zero();
    assert!(contract.is_infinity(zero.into()).call().await?);
    for _ in 0..10 {
        let non_zero: Affine = Projective::rand(rng).into();
        assert!(!contract.is_infinity(non_zero.into()).call().await?);
    }

    Ok(())
}

#[tokio::test]
async fn test_negate() -> Result<()> {
    let rng = &mut ark_std::test_rng();
    let contract = deploy_contract().await?;

    for _ in 0..10 {
        let p: Affine = Projective::rand(rng).into();
        let minus_p_sol: Point = contract.negate(p.into()).call().await?.into();
        assert_eq!(minus_p_sol, (-p).into());
    }

    Ok(())
}

#[tokio::test]
async fn test_scalar_mul() -> Result<()> {
    let rng = &mut ark_std::test_rng();
    let contract = deploy_contract().await?;

    for _ in 0..10 {
        let p = Projective::rand(rng);
        let s = Fr::rand(rng);

        let res: Point = contract
            .scalar_mul(p.into_affine().into(), field_to_u256(s))
            .call()
            .await?
            .into();
        assert_eq!(res, Group::mul(&p, &s).into_affine().into());
    }

    Ok(())
}

#[tokio::test]
async fn test_multi_scalar_mul() -> Result<()> {
    let rng = &mut ark_std::test_rng();
    let contract = deploy_contract().await?;

    for length in 1..10 {
        let p_rust: Vec<Affine> = (0..length)
            .map(|_| Projective::rand(rng).into_affine())
            .collect();
        let p_solidity: Vec<Point> = p_rust.iter().map(|&x| x.into()).collect();

        let s_rust: Vec<Fr> = (0..length).map(|_| Fr::rand(rng)).collect();
        let s_solidity: Vec<U256> = s_rust.iter().map(|&x| field_to_u256(x)).collect();
        let s_rust: Vec<_> = s_rust.iter().map(|&x| x.into_repr()).collect();

        let res: Point = contract
            .test_multi_scalar_mul(p_solidity, s_solidity)
            .call()
            .await?
            .into();

        assert_eq!(
            res,
            VariableBaseMSM::multi_scalar_mul(&p_rust, &s_rust)
                .into_affine()
                .into()
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_is_y_negative() -> Result<()> {
    let rng = &mut ark_std::test_rng();
    let contract = deploy_contract().await?;

    for _ in 0..10 {
        let p: Affine = Projective::rand(rng).into();
        // https://github.com/arkworks-rs/algebra/blob/98f43af6cb0a4620b78dbb3f46d3c2794bbfc66f/ec/src/models/short_weierstrass_jacobian.rs#L776
        let is_negative = p.y < -p.y;
        assert_eq!(contract.is_y_negative(p.into()).call().await?, is_negative);
        assert_eq!(
            contract.is_y_negative((-p).into()).call().await?,
            !is_negative
        );
    }
    Ok(())
}

#[tokio::test]
async fn test_invert() -> Result<()> {
    let rng = &mut ark_std::test_rng();
    let contract = deploy_contract().await?;

    for _ in 0..10 {
        let f = Fr::rand(rng);
        assert_eq!(
            contract.invert_fr(field_to_u256(f)).call().await?,
            field_to_u256(f.inverse().unwrap())
        );

        let f = Fq::rand(rng);
        assert_eq!(
            contract.invert_fq(field_to_u256(f)).call().await?,
            field_to_u256(f.inverse().unwrap())
        );
    }
    Ok(())
}

#[tokio::test]
async fn test_validate_curve_point() -> Result<()> {
    let rng = &mut ark_std::test_rng();
    let contract = deploy_contract().await?;
    let p: Affine = Projective::rand(rng).into();
    contract.validate_curve_point(p.into()).call().await?;

    async fn should_fail_validation(
        contract: &TestPallas<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
        bad_p: Point,
    ) {
        contract
            .validate_curve_point(bad_p)
            .call()
            .await
            .should_revert_with_message("Pallas: invalid point");
    }

    // x = 0 should fail
    let mut bad_p = p.clone();
    bad_p.x = field_new!(Fq, "0");
    should_fail_validation(&contract, bad_p.into()).await;

    // y = 0 should fail
    let mut bad_p = p.clone();
    bad_p.y = field_new!(Fq, "0");
    should_fail_validation(&contract, bad_p.into()).await;

    // x > p should fail
    let mut bad_p_g1: Point = p.clone().into();
    bad_p_g1.x = U256::MAX;
    should_fail_validation(&contract, bad_p_g1).await;

    // y > p should fail
    let mut bad_p_g1: Point = p.clone().into();
    bad_p_g1.y = U256::MAX;
    should_fail_validation(&contract, bad_p_g1).await;

    // not on curve point (y^2 = x^3 + 5 mod p) should fail
    let bad_p = Affine::new(field_new!(Fq, "1"), field_new!(Fq, "3"), false);
    should_fail_validation(&contract, bad_p.into()).await;
    Ok(())
}

#[tokio::test]
async fn test_validate_scalar_field() -> Result<()> {
    let rng = &mut ark_std::test_rng();
    let contract = deploy_contract().await?;
    let f = Fr::rand(rng);
    contract
        .validate_scalar_field(field_to_u256(f))
        .call()
        .await?;

    contract
        .validate_scalar_field(
            U256::from_str_radix(
                "28948022309329048855892746252171976963363056481941647379679742748393362948097",
                10,
            )
            .unwrap(),
        )
        .call()
        .await
        .should_revert_with_message("Pallas: invalid scalar field");
    Ok(())
}

#[tokio::test]
async fn test_from_le_bytes_mod_order() -> Result<()> {
    let rng = &mut ark_std::test_rng();
    let contract = deploy_contract().await?;

    for _ in 0..10 {
        let mut bytes = [0u8; 32];
        rng.fill_bytes(&mut bytes);
        assert_eq!(
            contract
                .from_le_bytes_mod_order(bytes.to_vec().into())
                .call()
                .await?,
            field_to_u256(Fr::from_le_bytes_mod_order(&bytes))
        );

        let mut bytes = [0u8; 48];
        rng.fill_bytes(&mut bytes);
        assert_eq!(
            contract
                .from_le_bytes_mod_order(bytes.to_vec().into())
                .call()
                .await?,
            field_to_u256(Fr::from_le_bytes_mod_order(&bytes))
        );
    }
    Ok(())
}

#[tokio::test]
async fn test_pow_small() -> Result<()> {
    let rng = &mut ark_std::test_rng();
    let contract = deploy_contract().await?;
    let modulus = <<Fr as PrimeField>::Params as FpParameters>::MODULUS;

    for _ in 0..10 {
        // pow_small userful when evaluating of Z_H(X) = X^n - 1 at random points
        let base = Fr::rand(rng);
        let exponent = u64::rand(rng); // small exponent (<= 64 bit)
        assert_eq!(
            contract
                .pow_small(
                    field_to_u256(base),
                    field_to_u256(Fr::from(exponent)),
                    U256::from_little_endian(&to_bytes!(modulus).unwrap()),
                )
                .call()
                .await?,
            field_to_u256(base.pow([exponent])),
        );
    }
    Ok(())
}

#[tokio::test]
async fn test_doubling() -> Result<()> {
    let rng = &mut ark_std::test_rng();
    let contract = deploy_contract().await?;

    for _ in 0..10 {
        let p = Projective::rand(rng);
        let p2 = ProjectiveCurve::double(&p);

        let res: Point = contract.double(p.into_affine().into()).call().await?.into();
        assert_eq!(res, p2.into_affine().into());
    }

    Ok(())
}
