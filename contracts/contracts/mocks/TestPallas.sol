//SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.0;

import {Pallas as C} from "../libraries/Pallas.sol";

contract TestPallas {
    constructor() {}

    // solhint-disable-next-line func-name-mixedcase
    function P1() public pure returns (C.Point memory) {
        return C.P1();
    }

    function isInfinity(C.Point memory point) public pure returns (bool) {
        return C.isInfinity(point);
    }

    function negate(C.Point memory p) public pure returns (C.Point memory r) {
        return C.negate(p);
    }

    function add(C.Point memory p1, C.Point memory p2) public view returns (C.Point memory) {
        return C.add(p1, p2);
    }

    function scalarMul(C.Point memory p, uint256 s) public view returns (C.Point memory r) {
        return C.scalarMul(p, s);
    }

    function invert(uint256 fr) public view returns (uint256 output) {
        return C.invert(fr);
    }

    function validateCurvePoint(C.Point memory point) public pure {
        C.validateCurvePoint(point);
    }

    function validateScalarField(uint256 fr) public pure {
        C.validateScalarField(fr);
    }

    function fromLeBytesModOrder(bytes memory leBytes) public pure returns (uint256) {
        return C.fromLeBytesModOrder(leBytes);
    }

    function isYNegative(C.Point memory p) public pure returns (bool) {
        return C.isYNegative(p);
    }

    function powSmall(
        uint256 base,
        uint256 exponent,
        uint256 modulus
    ) public pure returns (uint256) {
        return C.powSmall(base, exponent, modulus);
    }

    function testMultiScalarMul(C.Point[] memory bases, uint256[] memory scalars)
        public
        view
        returns (C.Point memory)
    {
        return C.multiScalarMul(bases, scalars);
    }
}
