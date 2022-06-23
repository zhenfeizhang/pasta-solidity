// SPDX-License-Identifier: MIT
//
//
// Copyright 2022 Zhenfei Zhang
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
// The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

pragma solidity ^0.8.0;

/// @notice Barreto-Naehrig curve over a 254 bit prime field
library Pallas {
    //
    // Pallas curve:
    //   p = 28948022309329048855892746252171976963363056481941560715954676764349967630337
    //   r = 28948022309329048855892746252171976963363056481941647379679742748393362948097
    // E has the equation:
    //   E: y^2 = x^3 + 5

    uint256 public constant P_MOD =
        28948022309329048855892746252171976963363056481941560715954676764349967630337;
    uint256 public constant R_MOD =
        28948022309329048855892746252171976963363056481941647379679742748393362948097;

    uint256 private constant _THREE_OVER_TWO =
        14474011154664524427946373126085988481681528240970780357977338382174983815170;

    struct Point {
        uint256 x;
        uint256 y;
    }

    /// @return the generator of G1
    // solhint-disable-next-line func-name-mixedcase
    function P1() internal pure returns (Point memory) {
        return Point(P_MOD - 1, 2);
    }

    /// @dev check if a G1 point is Infinity
    /// @notice precompile bn256Add at address(6) takes (0, 0) as Point of Infinity,
    /// some crypto libraries (such as arkwork) uses a boolean flag to mark PoI, and
    /// just use (0, 1) as affine coordinates (not on curve) to represents PoI.
    function isInfinity(Point memory point) internal pure returns (bool result) {
        assembly {
            let x := mload(point)
            let y := mload(add(point, 0x20))
            result := and(iszero(x), iszero(y))
        }
    }

    /// @return r the negation of p, i.e. p.add(p.negate()) should be zero.
    function negate(Point memory p) internal pure returns (Point memory) {
        if (isInfinity(p)) {
            return p;
        }
        return Point(p.x, P_MOD - (p.y % P_MOD));
    }

    /// @return res = -fr the negation of scalar field element.
    function negate(uint256 fr) internal pure returns (uint256 res) {
        return R_MOD - (fr % R_MOD);
    }

    function double(Point memory point) internal view returns (Point memory) {
        if (isInfinity(point)) {
            return point;
        }

        uint256 lambda;
        uint256 x = point.x;
        uint256 y = point.y;
        uint256 yInv = invert(point.y, P_MOD);
        uint256 xPrime;
        uint256 yPrime;

        assembly {
            // lambda = 3x^2/2y
            lambda := mulmod(x, x, P_MOD)
            lambda := mulmod(lambda, yInv, P_MOD)
            lambda := mulmod(lambda, _THREE_OVER_TWO, P_MOD)

            // x' = lambda^2 - 2x
            xPrime := mulmod(lambda, lambda, P_MOD)
            xPrime := add(xPrime, P_MOD)
            xPrime := add(xPrime, P_MOD)
            xPrime := sub(xPrime, x)
            xPrime := sub(xPrime, x)
            xPrime := mod(xPrime, P_MOD)

            // y' = lambda * (x-x') - y
            yPrime := add(x, P_MOD)
            yPrime := sub(yPrime, xPrime)
            yPrime := mulmod(lambda, yPrime, P_MOD)
            yPrime := add(yPrime, P_MOD)
            yPrime := sub(yPrime, y)
            yPrime := mod(yPrime, P_MOD)
        }

        return Point(xPrime, yPrime);
    }

    /// @return r the sum of two points of G1
    function add(Point memory p1, Point memory p2) internal view returns (Point memory) {
        if (isInfinity(p1)) {
            return p2;
        }

        if (isInfinity(p2)) {
            return p1;
        }

        uint256 lambda;
        uint256 tmp;
        uint256 x1 = p1.x;
        uint256 y1 = p1.y;
        uint256 x2 = p2.x;
        uint256 y2 = p2.y;
        uint256 x3;
        uint256 y3;

        // lambda = (y1-y2)/(x1-x2)
        assembly {
            lambda := add(x1, P_MOD)
            lambda := sub(lambda, x2)
            tmp := add(y1, P_MOD)
            tmp := sub(tmp, y2)
        }
        if (lambda > P_MOD) {
            lambda -= P_MOD;
        }
        lambda = invert(lambda, P_MOD);
        assembly {
            // lambda = (y1-y2)/(x1-x2)
            lambda := mulmod(lambda, tmp, P_MOD)

            // x3 = lambda^2 - x1 - x2
            x3 := mulmod(lambda, lambda, P_MOD)
            x3 := add(x3, P_MOD)
            x3 := add(x3, P_MOD)
            x3 := sub(x3, x1)
            x3 := sub(x3, x2)
            x3 := mod(x3, P_MOD)

            // y' = lambda * (x-x') - y
            y3 := add(x1, P_MOD)
            y3 := sub(y3, x3)
            y3 := mulmod(lambda, y3, P_MOD)
            y3 := add(y3, P_MOD)
            y3 := sub(y3, y1)
            y3 := mod(y3, P_MOD)
        }

        return Point(x3, y3);
    }

    /// @return r the product of a point on G1 and a scalar, i.e.
    /// p == p.mul(1) and p.add(p) == p.mul(2) for all points p.
    function scalarMul(Point memory p, uint256 s) internal view returns (Point memory r) {
        uint256 bit;
        uint256 i = 0;
        Point memory tmp = p;
        r = Point(0, 0);

        for (i = 0; i < 256; i++) {
            bit = s & 1;
            s /= 2;
            if (bit == 1) {
                r = add(r, tmp);
            }
            tmp = double(tmp);
        }
    }

    /// @dev Multi-scalar Mulitiplication (MSM)
    /// @return r = \Prod{B_i^s_i} where {s_i} are `scalars` and {B_i} are `bases`
    function multiScalarMul(Point[] memory bases, uint256[] memory scalars)
        internal
        view
        returns (Point memory r)
    {
        require(scalars.length == bases.length, "MSM error: length does not match");

        r = scalarMul(bases[0], scalars[0]);
        for (uint256 i = 1; i < scalars.length; i++) {
            r = add(r, scalarMul(bases[i], scalars[i]));
        }
    }

    /// @dev Compute f^-1 for f \in Fr scalar field
    /// @notice credit: Aztec, Spilsbury Holdings Ltd
    function invert(uint256 fr, uint256 modulus) internal view returns (uint256 output) {
        bool success;
        assembly {
            let mPtr := mload(0x40)
            mstore(mPtr, 0x20)
            mstore(add(mPtr, 0x20), 0x20)
            mstore(add(mPtr, 0x40), 0x20)
            mstore(add(mPtr, 0x60), fr)
            mstore(add(mPtr, 0x80), sub(modulus, 2))
            mstore(add(mPtr, 0xa0), modulus)
            success := staticcall(gas(), 0x05, mPtr, 0xc0, 0x00, 0x20)
            output := mload(0x00)
        }
        require(success, "Pallas: pow precompile failed!");
    }

    /**
     * validate the following:
     *   x != 0
     *   y != 0
     *   x < p
     *   y < p
     *   y^2 = x^3 + 5 mod p
     */
    /// @dev validate G1 point and check if it is on curve
    /// @notice credit: Aztec, Spilsbury Holdings Ltd
    function validateCurvePoint(Point memory point) internal pure {
        bool isWellFormed;
        uint256 p = P_MOD;
        assembly {
            let x := mload(point)
            let y := mload(add(point, 0x20))

            isWellFormed := and(
                and(and(lt(x, p), lt(y, p)), not(or(iszero(x), iszero(y)))),
                eq(mulmod(y, y, p), addmod(mulmod(x, mulmod(x, x, p), p), 5, p))
            )
        }
        require(isWellFormed, "Pallas: invalid point");
    }

    /// @dev Validate scalar field, revert if invalid (namely if fr > r_mod).
    /// @notice Writing this inline instead of calling it might save gas.
    function validateScalarField(uint256 fr) internal pure {
        bool isValid;
        assembly {
            isValid := lt(fr, R_MOD)
        }
        require(isValid, "Pallas: invalid scalar field");
    }

    function fromLeBytesModOrder(bytes memory leBytes) internal pure returns (uint256 ret) {
        // TODO: Can likely be gas optimized by copying the first 31 bytes directly.
        for (uint256 i = 0; i < leBytes.length; i++) {
            ret = mulmod(ret, 256, R_MOD);
            ret = addmod(ret, uint256(uint8(leBytes[leBytes.length - 1 - i])), R_MOD);
        }
    }

    /// @dev Check if y-coordinate of G1 point is negative.
    function isYNegative(Point memory point) internal pure returns (bool) {
        return point.y < P_MOD / 2;
    }

    // @dev Perform a modular exponentiation.
    // @return base^exponent (mod modulus)
    // This method is ideal for small exponents (~64 bits or less), as it is cheaper than using the pow precompile
    // @notice credit: credit: Aztec, Spilsbury Holdings Ltd
    function powSmall(
        uint256 base,
        uint256 exponent,
        uint256 modulus
    ) internal pure returns (uint256) {
        uint256 result = 1;
        uint256 input = base;
        uint256 count = 1;

        assembly {
            let endpoint := add(exponent, 0x01)
            for {

            } lt(count, endpoint) {
                count := add(count, count)
            } {
                if and(exponent, count) {
                    result := mulmod(result, input, modulus)
                }
                input := mulmod(input, input, modulus)
            }
        }

        return result;
    }
}
