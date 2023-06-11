/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as beet from "@metaplex-foundation/beet";
export type Schedule = {
  releaseTime: beet.bignum;
  amount: beet.bignum;
};

/**
 * @category userTypes
 * @category generated
 */
export const scheduleBeet = new beet.BeetArgsStruct<Schedule>(
  [
    ["releaseTime", beet.u64],
    ["amount", beet.u64],
  ],
  "Schedule"
);