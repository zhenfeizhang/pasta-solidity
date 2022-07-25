#![cfg(test)]

use crate::{
    assertion::Matcher,
    ethereum::{deploy, get_funded_deployer},
    types::{
        field_to_u256, TestVesta, VestaAffinePoint as AffinePoint,
        VestaProjectivePoint as ProjectivePoint,
    },
};
use anyhow::Result;
use ark_ec::msm::VariableBaseMSM;
use ark_ec::AffineCurve;
use ark_ec::{group::Group, ProjectiveCurve};
use ark_ff::{field_new, to_bytes, Field};
use ark_ff::{FpParameters, PrimeField};
use ark_std::UniformRand;
use ark_std::Zero;
use ark_vesta::{Affine, Fq, Fr, Projective};
use ethers::core::k256::ecdsa::SigningKey;
use ethers::prelude::*;
use rand::RngCore;
use std::path::Path;

async fn deploy_contract() -> Result<TestVesta<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>>
{
    let client = get_funded_deployer().await.unwrap();
    let contract = deploy(
        client.clone(),
        Path::new("../abi/contracts/mocks/TestVesta.sol/TestVesta"),
        (),
    )
    .await
    .unwrap();
    Ok(TestVesta::new(contract.address(), client))
}

#[tokio::test]
async fn test_add() -> Result<()> {
    let rng = &mut ark_std::test_rng();
    let contract = deploy_contract().await?;

    let p1 = Projective::rand(rng);
    let p2 = Projective::rand(rng);
    println!(
        "gas cost: affine addition: {}",
        contract
            .affine_add(p1.into_affine().into(), p2.into_affine().into())
            .estimate_gas()
            .await?
    );

    let p1 = Projective::rand(rng);
    let p2 = Projective::rand(rng);
    println!(
        "gas cost: projective addition: {}",
        contract
            .projective_add(p1.into(), p2.into())
            .estimate_gas()
            .await?
    );

    // test random group addition
    for _ in 0..10 {
        let p1: Affine = Projective::rand(rng).into();
        let p2: Affine = Projective::rand(rng).into();
        let res: AffinePoint = contract
            .affine_add(p1.into(), p2.into())
            .call()
            .await?
            .into();
        assert_eq!(res, (p1 + p2).into());

        let p1 = Projective::rand(rng);
        let p2 = Projective::rand(rng);
        let res: ProjectivePoint = contract
            .projective_add(p1.into(), p2.into())
            .call()
            .await?
            .into();
        assert_eq!(res, (p1 + p2).into());

        let p1 = Projective::rand(rng);
        let res: ProjectivePoint = contract
            .projective_add(p1.into(), p1.into())
            .call()
            .await?
            .into();
        assert_eq!(res, (p1 + p1).into());
    }

    // test point of infinity, O_E + P = P
    let zero = Affine::zero();
    let p: Affine = Projective::rand(rng).into();
    let res: AffinePoint = contract
        .affine_add(p.into(), zero.into())
        .call()
        .await?
        .into();
    assert_eq!(res, p.into());

    Ok(())
}

#[tokio::test]
async fn test_group_generators() -> Result<()> {
    let contract = deploy_contract().await?;

    let gen = Affine::prime_subgroup_generator();
    let gen_sol: AffinePoint = contract.affine_generator().call().await?.into();
    assert_eq!(gen_sol, gen.into());

    let gen = Projective::prime_subgroup_generator();
    let gen_sol: ProjectivePoint = contract.projective_generator().call().await?.into();
    assert_eq!(gen_sol, gen.into());

    Ok(())
}

#[tokio::test]
async fn test_info_affine() -> Result<()> {
    let rng = &mut ark_std::test_rng();
    let contract = deploy_contract().await?;

    let gen = Affine::prime_subgroup_generator();
    let gen_sol = contract.projective_generator().call().await?;
    let gen_sol_affine = contract.to_affine(gen_sol).call().await?;
    assert_eq!(gen_sol_affine, gen.into());

    let p = Projective::rand(rng);
    println!(
        "gas cost: to affine: {}",
        contract.to_affine(p.into()).estimate_gas().await?
    );

    for _ in 0..10 {
        let p = Projective::rand(rng);
        let p_sol: Affine = contract.to_affine(p.into()).call().await?.into();
        assert_eq!(p_sol, p.into_affine());
    }

    Ok(())
}

#[tokio::test]
async fn test_is_infinity() -> Result<()> {
    let rng = &mut ark_std::test_rng();
    let contract = deploy_contract().await?;

    let zero = Affine::zero();
    assert!(contract.is_affine_infinity(zero.into()).call().await?);
    let zero = zero.into_projective();
    assert!(contract.is_projective_infinity(zero.into()).call().await?);
    for _ in 0..10 {
        let non_zero: Affine = Projective::rand(rng).into();
        assert!(!contract.is_affine_infinity(non_zero.into()).call().await?);
        let non_zero = non_zero.into_projective();
        assert!(
            !contract
                .is_projective_infinity(non_zero.into())
                .call()
                .await?
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_negate() -> Result<()> {
    let rng = &mut ark_std::test_rng();
    let contract = deploy_contract().await?;

    for _ in 0..10 {
        let p: Affine = Projective::rand(rng).into();
        let minus_p_sol: AffinePoint = contract.affine_negate(p.into()).call().await?.into();
        assert_eq!(minus_p_sol, (-p).into());

        let p = Projective::rand(rng);
        let minus_p_sol: ProjectivePoint =
            contract.projective_negate(p.into()).call().await?.into();
        assert_eq!(minus_p_sol, (-p).into());
    }

    Ok(())
}

#[tokio::test]
async fn test_scalar_mul() -> Result<()> {
    let rng = &mut ark_std::test_rng();
    let contract = deploy_contract().await?;

    let p = Projective::rand(rng);
    let s = Fr::rand(rng);
    println!(
        "gas cost: affine scalar mul: {}",
        contract
            .affine_scalar_mul(p.into_affine().into(), field_to_u256(s))
            .estimate_gas()
            .await?
    );

    let p = Projective::rand(rng);
    let s = Fr::rand(rng);
    println!(
        "gas cost: projective scalar mul: {}",
        contract
            .projective_scalar_mul(p.into(), field_to_u256(s))
            .estimate_gas()
            .await?
    );

    for _ in 0..10 {
        let p = Projective::rand(rng);
        let s = Fr::rand(rng);

        let res: AffinePoint = contract
            .affine_scalar_mul(p.into_affine().into(), field_to_u256(s))
            .call()
            .await?
            .into();
        assert_eq!(res, Group::mul(&p, &s).into_affine().into());
    }

    for _ in 0..10 {
        let p = Projective::rand(rng);
        let s = Fr::rand(rng);

        let res: Projective = contract
            .projective_scalar_mul(p.into(), field_to_u256(s))
            .call()
            .await?
            .into();
        assert_eq!(res.into_affine(), Group::mul(&p, &s).into_affine());
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
        let p_solidity: Vec<AffinePoint> = p_rust.iter().map(|&x| x.into()).collect();

        let s_rust: Vec<Fr> = (0..length).map(|_| Fr::rand(rng)).collect();
        let s_solidity: Vec<U256> = s_rust.iter().map(|&x| field_to_u256(x)).collect();
        let s_rust: Vec<_> = s_rust.iter().map(|&x| x.into_repr()).collect();

        println!(
            "gas cost: {} msm: {}",
            length,
            contract
                .test_multi_scalar_mul(p_solidity.clone(), s_solidity.clone())
                .estimate_gas()
                .await?
        );

        let res: AffinePoint = contract
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
        contract: &TestVesta<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
        bad_p: AffinePoint,
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
    let mut bad_p: AffinePoint = p.clone().into();
    bad_p.x = U256::MAX;
    should_fail_validation(&contract, bad_p).await;

    // y > p should fail
    let mut bad_p: AffinePoint = p.clone().into();
    bad_p.y = U256::MAX;
    should_fail_validation(&contract, bad_p).await;

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
                "28948022309329048855892746252171976963363056481941560715954676764349967630337",
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

    let p = Projective::rand(rng);
    println!(
        "gas cost: affine doubling: {}",
        contract
            .affine_double(p.into_affine().into())
            .estimate_gas()
            .await?
    );
    println!(
        "gas cost: projective doubling: {}",
        contract.projective_double(p.into()).estimate_gas().await?
    );

    for _ in 0..10 {
        let p = Projective::rand(rng);
        let p2 = ProjectiveCurve::double(&p);

        let res: AffinePoint = contract
            .affine_double(p.into_affine().into())
            .call()
            .await?
            .into();
        assert_eq!(res, p2.into_affine().into());

        let res: ProjectivePoint = contract.projective_double(p.into()).call().await?;
        let res = Projective::from(res);

        assert_eq!(res, p2);
    }

    Ok(())
}
