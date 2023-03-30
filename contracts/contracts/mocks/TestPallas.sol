//SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.0;

import {Pallas as C} from "../libraries/Pallas.sol";

contract TestPallas {
    constructor() {}

    // solhint-disable-next-line func-name-mixedcase
    function affineGenerator() public pure returns (C.PallasAffinePoint memory) {
        return C.AffineGenerator();
    }

    // solhint-disable-next-line func-name-mixedcase
    function projectiveGenerator() public pure returns (C.PallasProjectivePoint memory) {
        return C.ProjectiveGenerator();
    }

    // solhint-disable-next-line func-name-mixedcase
    function toAffine(C.PallasProjectivePoint memory point)
        public
        view
        returns (C.PallasAffinePoint memory)
    {
        return C.IntoAffine(point);
    }

    // solhint-disable-next-line func-name-mixedcase
    function toProjective(C.PallasAffinePoint memory point)
        public
        pure
        returns (C.PallasProjectivePoint memory)
    {
        return C.IntoProjective(point);
    }

    function isProjectiveInfinity(C.PallasProjectivePoint memory point)
        public
        pure
        returns (bool)
    {
        return C.isInfinity(point);
    }

    function isAffineInfinity(C.PallasAffinePoint memory point) public pure returns (bool) {
        return C.isInfinity(point);
    }

    function affineNegate(C.PallasAffinePoint memory p)
        public
        pure
        returns (C.PallasAffinePoint memory r)
    {
        return C.negate(p);
    }

    function projectiveNegate(C.PallasProjectivePoint memory p)
        public
        pure
        returns (C.PallasProjectivePoint memory r)
    {
        return C.negate(p);
    }

    function affineDouble(C.PallasAffinePoint memory p)
        public
        view
        returns (C.PallasAffinePoint memory)
    {
        return C.double(p);
    }

    function projectiveDouble(C.PallasProjectivePoint memory p)
        public
        pure
        returns (C.PallasProjectivePoint memory)
    {
        return C.double(p);
    }

    function affineAdd(C.PallasAffinePoint memory p1, C.PallasAffinePoint memory p2)
        public
        view
        returns (C.PallasAffinePoint memory)
    {
        return C.add(p1, p2);
    }

    function projectiveAdd(C.PallasProjectivePoint memory p1, C.PallasProjectivePoint memory p2)
        public
        pure
        returns (C.PallasProjectivePoint memory)
    {
        return C.add(p1, p2);
    }

    function affineScalarMul(C.PallasAffinePoint memory p, uint256 s)
        public
        view
        returns (C.PallasAffinePoint memory r)
    {
        return C.scalarMul(p, s);
    }

    function projectiveScalarMul(C.PallasProjectivePoint memory p, uint256 s)
        public
        pure
        returns (C.PallasProjectivePoint memory r)
    {
        return C.scalarMul(p, s);
    }

    function invertFr(uint256 fr) public view returns (uint256 output) {
        return C.invert(fr, C.R_MOD);
    }

    function invertFq(uint256 fq) public view returns (uint256 output) {
        return C.invert(fq, C.P_MOD);
    }

    function validateCurvePoint(C.PallasAffinePoint memory point) public pure {
        C.validateCurvePoint(point);
    }

    function validateScalarField(uint256 fr) public pure {
        C.validateScalarField(fr);
    }

    function fromLeBytesModOrder(bytes memory leBytes) public pure returns (uint256) {
        return C.fromLeBytesModOrder(leBytes);
    }

    function isYNegative(C.PallasAffinePoint memory p) public pure returns (bool) {
        return C.isYNegative(p);
    }

    function powSmall(
        uint256 base,
        uint256 exponent,
        uint256 modulus
    ) public pure returns (uint256) {
        return C.powSmall(base, exponent, modulus);
    }

    function testMultiScalarMul(C.PallasAffinePoint[] memory bases, uint256[] memory scalars)
        public
        view
        returns (C.PallasAffinePoint memory)
    {
        return C.multiScalarMul(bases, scalars);
    }
}
