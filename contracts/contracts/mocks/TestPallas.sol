//SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.0;

import {Pallas as C} from "../libraries/Pallas.sol";

contract TestPallas {
    constructor() {}

    // solhint-disable-next-line func-name-mixedcase
    function P1() public pure returns (C.PallasPoint memory) {
        return C.P1();
    }

    function isInfinity(C.PallasPoint memory point) public pure returns (bool) {
        return C.isInfinity(point);
    }

    function negate(C.PallasPoint memory p) public pure returns (C.PallasPoint memory r) {
        return C.negate(p);
    }

    function double(C.PallasPoint memory p) public view returns (C.PallasPoint memory) {
        return C.double(p);
    }

    function add(C.PallasPoint memory p1, C.PallasPoint memory p2)
        public
        view
        returns (C.PallasPoint memory)
    {
        return C.add(p1, p2);
    }

    function scalarMul(C.PallasPoint memory p, uint256 s)
        public
        view
        returns (C.PallasPoint memory r)
    {
        return C.scalarMul(p, s);
    }

    function invertFr(uint256 fr) public view returns (uint256 output) {
        return C.invert(fr, C.R_MOD);
    }

    function invertFq(uint256 fq) public view returns (uint256 output) {
        return C.invert(fq, C.P_MOD);
    }

    function validateCurvePoint(C.PallasPoint memory point) public pure {
        C.validateCurvePoint(point);
    }

    function validateScalarField(uint256 fr) public pure {
        C.validateScalarField(fr);
    }

    function fromLeBytesModOrder(bytes memory leBytes) public pure returns (uint256) {
        return C.fromLeBytesModOrder(leBytes);
    }

    function isYNegative(C.PallasPoint memory p) public pure returns (bool) {
        return C.isYNegative(p);
    }

    function powSmall(
        uint256 base,
        uint256 exponent,
        uint256 modulus
    ) public pure returns (uint256) {
        return C.powSmall(base, exponent, modulus);
    }

    function testMultiScalarMul(C.PallasPoint[] memory bases, uint256[] memory scalars)
        public
        view
        returns (C.PallasPoint memory)
    {
        return C.multiScalarMul(bases, scalars);
    }
}
