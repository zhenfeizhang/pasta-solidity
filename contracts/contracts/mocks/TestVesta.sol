//SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.0;

import {Vesta as C} from "../libraries/Vesta.sol";

contract TestVesta {
    constructor() {}

    // solhint-disable-next-line func-name-mixedcase
    function P1() public pure returns (C.VestaPoint memory) {
        return C.P1();
    }

    function isInfinity(C.VestaPoint memory point) public pure returns (bool) {
        return C.isInfinity(point);
    }

    function negate(C.VestaPoint memory p) public pure returns (C.VestaPoint memory r) {
        return C.negate(p);
    }

    function double(C.VestaPoint memory p) public view returns (C.VestaPoint memory) {
        return C.double(p);
    }

    function add(C.VestaPoint memory p1, C.VestaPoint memory p2)
        public
        view
        returns (C.VestaPoint memory)
    {
        return C.add(p1, p2);
    }

    function scalarMul(C.VestaPoint memory p, uint256 s)
        public
        view
        returns (C.VestaPoint memory r)
    {
        return C.scalarMul(p, s);
    }

    function invertFr(uint256 fr) public view returns (uint256 output) {
        return C.invert(fr, C.R_MOD);
    }

    function invertFq(uint256 fq) public view returns (uint256 output) {
        return C.invert(fq, C.P_MOD);
    }

    function validateCurvePoint(C.VestaPoint memory point) public pure {
        C.validateCurvePoint(point);
    }

    function validateScalarField(uint256 fr) public pure {
        C.validateScalarField(fr);
    }

    function fromLeBytesModOrder(bytes memory leBytes) public pure returns (uint256) {
        return C.fromLeBytesModOrder(leBytes);
    }

    function isYNegative(C.VestaPoint memory p) public pure returns (bool) {
        return C.isYNegative(p);
    }

    function powSmall(
        uint256 base,
        uint256 exponent,
        uint256 modulus
    ) public pure returns (uint256) {
        return C.powSmall(base, exponent, modulus);
    }

    function testMultiScalarMul(C.VestaPoint[] memory bases, uint256[] memory scalars)
        public
        view
        returns (C.VestaPoint memory)
    {
        return C.multiScalarMul(bases, scalars);
    }
}
